use provider::api::*;
use clap::{AppSettings, App, Arg, SubCommand, ArgMatches};
use std::env;
use rusoto_core::{Region};
use rusoto_ssm::{Parameter};
use std::str::FromStr;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
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
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(matches)) => {
            let (path, region) = process_get_matches(matches)?;
            let parameters: Box<Vec<Parameter>> = get_parameters(path, region)?;
            let map = to_hash_map(parameters);
            print!("{}", as_env_format(map));
            Ok(())
        },
        _ => Ok(())
    }
}

fn process_get_matches(matches: &ArgMatches) -> Result<(String, Region), Box<Error>> {
    let path = matches.value_of("path").unwrap();
    let region_name = matches.value_of("region");
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
    Ok((String::from(path), region))
}
