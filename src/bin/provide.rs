use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};
use provider::api;
use provider::error::ProvideError;
use provider::types::*;
use rusoto_core::Region;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

fn main() -> Result<(), ProvideError> {
    let matches = app().get_matches();
    let options = options_from_matches(matches)?;
    let format_config = options.format_config.clone();
    let maybe_run_config = options.run.clone();
    let vars: HashMap<String, String> = api::get_parameters(options)?;
    match maybe_run_config {
        Some(run_config) => Ok(api::run(run_config, vars)?),
        None => Ok(display(format_config, vars)),
    }
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("provide")
        .settings(&[AppSettings::TrailingVarArg])
        .about("Provides environment variables from AWS Parameter Store")

        .arg(Arg::with_name("get")
            .long("get")
            .takes_value(false)
            .requires_all(&["application", "target"])
            .help("Read AWS vars"))

        .group(ArgGroup::with_name("mode")
            .args(&["get"])
            .required(false))

        .arg(Arg::with_name("application")
            .required(false)
            .short("a")
            .long("application")
            .takes_value(true)
            .empty_values(false)
            .env("PROVIDE_APPLICATION")
            .value_name("APPLICATION")
            .help("The application used in path /<application>/<target>/"))

        .arg(Arg::with_name("target")
            .required(false)
            .short("t")
            .long("target")
            .takes_value(true)
            .empty_values(false)
            .env("PROVIDE_TARGET")
            .value_name("TARGET")
            .help("The target environment used in path /<application>/<target>/"))

        .arg(Arg::with_name("profile")
            .required(false)
            .short("p")
            .long("profile")
            .takes_value(true)
            .value_name("NAME")
            .help("Use credentials and region from a local profile"))

        .arg(Arg::with_name("region")
            .required(false)
            .short("r")
            .long("region")
            .takes_value(true)
            .value_name("REGION")
            .help("Specify region (overrides env, profile)"))

        .arg(Arg::with_name("include")
            .required(false)
            .short("i")
            .long("include")
            .multiple(true)
            .takes_value(true)
            .number_of_values(1)
            .value_name("FILE")
            .help("Read env variables in key=value format from a file"))

        .arg(Arg::with_name("merge")
            .required(false)
            .short("m")
            .long("merge")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1)
            .value_name("FILE")
            .help("Provide initial set of variables and execute FILE, merging output into list of variables"))

        .arg(Arg::with_name("format")
            .required(false)
            .short("f")
            .long("format")
            .takes_value(true)
            .value_name("FORMAT")
            .help("Format output, default 'env'"))

        .arg(Arg::with_name("env-var")
            .required(false)
            .short("e")
            .long("env-var")
            .multiple(true)
            .takes_value(true)
            .number_of_values(1)
            .value_name("ENV_VAR_NAME")
            .help("Capture env var"))

        .arg(Arg::with_name("env-var-base64")
            .required(false)
            .short("b")
            .long("env-var-base64")
            .multiple(true)
            .takes_value(true)
            .number_of_values(1)
            .value_name("ENV_VAR_NAME")
            .help("Capture env var where var is base64"))

        .arg(Arg::with_name("raw")
            .required(false)
            .long("raw")
            .takes_value(false)
            .help("Do not base64 encode values on output"))

        // Captures the trailing var args, if any
        .arg(Arg::with_name("cmd")
            .required(false)
            .multiple(true)
            .value_name("CMD")
            .help("Provide vars to given command"))
}

fn options_from_matches(matches: ArgMatches) -> Result<Options, ProvideError> {
    let has_get = matches.is_present("get");
    let has_set = matches.is_present("set");

    let mode = if has_get {
        Some(Mode::GET)
    } else if has_set {
        Some(Mode::SET)
    } else {
        None
    };

    let app = matches.value_of("application").map(|app| app.to_owned());

    let target = matches.value_of("target").map(|app| app.to_owned());

    let path = match (&app, &target) {
        (Some(a), Some(t)) => Some(format!("/{}/{}", a, t)),
        _ => None,
    };

    let region_name = matches.value_of("region");

    let includes: Option<Vec<String>> = match matches.values_of("include") {
        Some(values) => Some(values.map(|val| String::from(val)).collect()),
        None => None,
    };

    let merges: Option<Vec<String>> = match matches.values_of("merge") {
        Some(values) => Some(values.map(|val| String::from(val)).collect()),
        None => None,
    };

    let profile = matches.value_of("profile");
    if profile.is_some() {
        // The only way I've found to have rusoto honor given profile since ProfileProvider
        // ignores it. Note this is only set for the current process.
        env::set_var("AWS_PROFILE", profile.unwrap());
    }

    let region = match region_name {
        Some(name) => Region::from_str(name)?,
        None => Region::default(), // will return a region defined in the env, profile, or default see method doc
    };

    let format = match matches.value_of("format") {
        Some("export") => Ok(Format::EXPORT),
        Some("json") => Ok(Format::JSON),
        Some("env") | None => Ok(Format::ENV),
        Some(format_name) => Err(ProvideError::BadFormat(format!(
            "Unknown format {}",
            format_name
        ))),
    }?;

    let raw = matches.is_present("raw");

    let format_config = FormatConfig { format, raw };

    let env_vars: Option<Vec<String>> = match matches.values_of("env-var") {
        Some(values) => Some(values.map(|v| v.to_owned()).collect()),
        None => None,
    };

    let env_vars_base64: Option<Vec<String>> = match matches.values_of("env-var-base64") {
        Some(values) => Some(values.map(|v| v.to_owned()).collect()),
        None => None,
    };

    let cmds: Option<Vec<String>> = match matches.values_of("cmd") {
        Some(vals) => Some(vals.map(|v| String::from(v)).collect()),
        None => None,
    };

    let run = match cmds {
        Some(vars) => match vars.split_at(1) {
            ([head], tail) => Some(RunConfig {
                cmd: head.to_owned(),
                args: tail.to_owned(),
            }),
            _ => None,
        },
        None => None,
    };

    Ok(Options {
        mode,
        app,
        target,
        path,
        region,
        includes,
        format_config,
        merges,
        run,
        env_vars,
        env_vars_base64,
    })
}

fn display(format_config: FormatConfig, map: HashMap<String, String>) {
    let formatted = match format_config.format {
        Format::ENV => api::as_env_format(map, format_config.raw),
        Format::EXPORT => api::as_export_format(map, format_config.raw),
        Format::JSON => unimplemented!(),
    };
    print!("{}", formatted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_only_accepts_one_value() {
        let m = app().get_matches_from(vec!["provide", "--include", "include_file_1", "cmd"]);
        let options = options_from_matches(m);
        assert_eq!(
            options.unwrap(),
            Options {
                includes: Some(vec!["include_file_1".to_owned()]),
                run: Some(RunConfig {
                    cmd: "cmd".to_owned(),
                    ..RunConfig::default()
                }),
                ..Options::default()
            }
        );
    }

    #[test]
    fn test_merge_only_accepts_one_value() {
        let m = app().get_matches_from(vec!["provide", "--merge", "merge_file_1", "cmd"]);
        let options = options_from_matches(m);
        assert_eq!(
            options.unwrap(),
            Options {
                merges: Some(vec!["merge_file_1".to_owned()]),
                run: Some(RunConfig {
                    cmd: "cmd".to_owned(),
                    ..RunConfig::default()
                }),
                ..Options::default()
            }
        );
    }
}
