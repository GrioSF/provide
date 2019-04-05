use provider::api;
use provider::types::*;
use provider::error::{ProvideError};
use clap::{AppSettings, App, Arg, SubCommand, ArgMatches};
use std::env;
use rusoto_core::{Region};
use std::str::FromStr;
use std::path::PathBuf;

fn main() -> Result<(), ProvideError> {
    let matches = App::new("provide")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Provides environment variables from AWS Parameter Store")
        .subcommand(
            SubCommand::with_name("get")
                .about("Retrieve environment variables")
                .arg(Arg::with_name("path")
                    .required(true)
                    .value_name("PATH")
                    .help("The path to search, e.g. /myapp/staging"))
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
                    .help("Include these env variables from a file"))
                .arg(Arg::with_name("format")
                    .required(false)
                    .short("f")
                    .long("format")
                    .takes_value(true)
                    .value_name("FORMAT")
                    .help("Format output, default 'env'"))
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(matches)) => {
            let options = options_from_matches(matches)?;
            let pairs: Vec<Pair> = api::get_parameters(&options)?;
            Ok(display(pairs, options.format))
        },
        _ => Ok(())
    }
}

fn options_from_matches(matches: &ArgMatches) -> Result<Options, ProvideError> {
    let path = matches.value_of("path").unwrap().to_owned();
    let region_name = matches.value_of("region");
    let include = match matches.value_of("include") {
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
    Ok(Options{ path: path, region: region, include: include, format: format })
}

fn display(pairs: Vec<Pair>, format: Format) {
    let formatted = match format {
        Format::ENV => api::as_env_format(pairs),
        Format::EXPORT => api::as_export_format(pairs),
        Format::JSON => unimplemented!()
    };
    print!("{}", formatted);
}
