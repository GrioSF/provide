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
        .about("Provides environment variables from AWS Parameter Store")
        .arg(Arg::with_name("get")
            .long("get")
            .takes_value(false)
            .help("Read AWS vars"))
        .arg(Arg::with_name("set")
            .long("set")
            .takes_value(false)
            .help("Insert or update AWS vars"))
        .group(ArgGroup::with_name("mode")
            .args(&["get", "set"])
            .required(false))
        .arg(Arg::with_name("application")
            .required(true)
            .short("a")
            .long("appilcation")
            .value_name("APPLICATION")
            .help("The application used in path /<application>/<target>/"))
        .arg(Arg::with_name("target")
            .required(true)
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
        .get_matches();

    let options = options_from_matches(matches)?;
    // validate
    let map: HashMap<String, String> = api::get_parameters(&options)?;
    Ok(display(map, options.format))
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
    let app = matches.value_of("app").unwrap().to_owned();
    let target = matches.value_of("target").unwrap().to_owned();
    let path = format!("/{}/{}", app, target);
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
    Ok(Options{ mode: mode, path: path, region: region, include: include, format: format, merge: merge })
}

fn display(map: HashMap<String, String>, format: Format) {
    let formatted = match format {
        Format::ENV => api::as_env_format(map),
        Format::EXPORT => api::as_export_format(map),
        Format::JSON => unimplemented!()
    };
    print!("{}", formatted);
}
