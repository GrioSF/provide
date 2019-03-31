use provider::api::*;
use clap::{App, Arg, SubCommand};

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
                .arg(Arg::with_name("recursive")
                    .short("r")
                    .long("recursive")
                    .takes_value(false)
                    .help("Return values recursively under PATH"))
        )
        .get_matches();

    match matches.subcommand() {
        ("get", Some(get_matches)) => {
            let path = get_matches.value_of("path").unwrap();
            let recursive = get_matches.is_present("recursive");
            let profile = get_matches.value_of("profile");
            do_get(String::from(path), recursive, profile);
        },
        _ => ()
    }
}

fn do_get(path: String, recursive: bool, profile: Option<&str>) {
    match get_parameters(path, recursive, profile) {
        Ok(parameters) => {
            let map = to_hash_map(parameters);
            print!("{}", as_env_format(map));
        }
        Err(err) => println!("{:?}", err),
    };
}
