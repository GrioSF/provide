use provider::api::*;
use clap::{App, Arg, SubCommand};
use std::env;
use rusoto_core::{Region};
use std::str::FromStr;

fn main() {
    let matches = App::new("provide")
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
                    .help("Specify region"))
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(get_matches)) => {
            let path = get_matches.value_of("path").unwrap();
            let region_name = get_matches.value_of("region");
            let profile = get_matches.value_of("profile");
            if profile.is_some() {
                // The only way I've found to have rusoto honor given profile since ProfileProvider
                // ignores it. Note this is only set for the current process.
                env::set_var("AWS_PROFILE", profile.unwrap());
            }
            let region = match region_name {
                Some(name) => match Region::from_str(name) {
                    Ok(region) => region,
                    Err(err) => return println!("{:?}", err)
                },
                None => Region::default() // will return a region defined in the env, profile, or default see method doc
            };
            do_get(String::from(path), region);
        },
        _ => ()
    }
}

fn do_get(path: String, region: Region) {
    match get_parameters(path, region) {
        Ok(parameters) => {
            let map = to_hash_map(parameters);
            print!("{}", as_env_format(map));
        }
        Err(err) => println!("{:?}", err),
    };
}
