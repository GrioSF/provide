use std::env;
use std::str::FromStr;
use std::collections::HashMap;
use std::path::PathBuf;
use clap::{AppSettings, App, Arg, ArgGroup, ArgMatches};
use rusoto_core::{Region};
use provider::api;
use provider::types::*;
use provider::error::{ProvideError};

fn main() -> Result<(), ProvideError> {
    let matches = App::new("provide")
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
            .value_name("APPLICATION")
            .help("The application used in path /<application>/<target>/"))
        .arg(Arg::with_name("target")
            .required(false)
            .short("t")
            .long("target")
            .takes_value(true)
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
            .takes_value(true)
            .value_name("FILE")
            .help("Read env variables in key=value format from a file"))
        .arg(Arg::with_name("merge")
            .required(false)
            .short("m")
            .long("merge")
            .takes_value(true)
            .value_name("FILE")
            .help("Provide initial set of variables and execute FILE, merging output into list of variables"))
        .arg(Arg::with_name("format")
            .required(false)
            .short("f")
            .long("format")
            .takes_value(true)
            .value_name("FORMAT")
            .help("Format output, default 'env'"))
        .arg(Arg::with_name("cmd")
            .required(false)
            .multiple(true)
            .value_name("CMD")
            .help("Provide vars to given command"))
        .get_matches();

    let options = options_from_matches(matches)?;
    let format = options.format.clone();
    let maybe_run = options.run.clone();
    let vars: HashMap<String, String> = api::get_parameters(options)?;
    match maybe_run {
        Some(run) => Ok(api::run(run, vars)?),
        None => Ok(display(vars, format))
    }
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
    let app = matches.value_of("application");
    let target = matches.value_of("target");
    let path = match (app, target) {
        (Some(app), Some(target)) => Some(format!("/{}/{}", app, target)),
        _ => None
    };
    let region_name = matches.value_of("region");
    let include = match matches.value_of("include") {
        Some(file_name) => Some(PathBuf::from(file_name)),
        None => None
    };
    let merge = match matches.value_of("merge") {
        Some(file_name) => Some(PathBuf::from(file_name)),
        None => None
    };
    let profile = matches.value_of("profile");
    if profile.is_some() {
        // The only way I've found to have rusoto honor given profile since ProfileProvider
        // ignores it. Note this is only set for the current process.
        env::set_var("AWS_PROFILE", profile.unwrap());
    }
    let region = match region_name {
        Some(name) => Region::from_str(name)?,
        None => Region::default() // will return a region defined in the env, profile, or default see method doc
    };
    let format = match matches.value_of("format") {
        Some("export") => Ok(Format::EXPORT),
        Some("json") => Ok(Format::JSON),
        Some("env") | None => Ok(Format::ENV),
        Some(format_name) => Err(ProvideError::BadFormat(format!("Unknown format {}", format_name)))
    }?;
    let cmds: Option<Vec<String>> = match matches.values_of("cmd") {
        Some(vals) => Some(vals.map(|v| String::from(v)).collect()),
        None => None
    };
    let run = match cmds {
        Some(vars) => {
            match vars.split_at(1) {
                ([head], tail) => Some(Run{ cmd: PathBuf::from(head), args: tail.to_owned()}),
                _ => None
            }
        },
        None => None
    };
    Ok(Options{ mode: mode, path: path, region: region, include: include, format: format, merge: merge, run: run })
}

fn display(map: HashMap<String, String>, format: Format) {
    let formatted = match format {
        Format::ENV => api::as_env_format(map),
        Format::EXPORT => api::as_export_format(map),
        Format::JSON => unimplemented!()
    };
    print!("{}", formatted);
}
