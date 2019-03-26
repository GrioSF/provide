use provider::api::*;
use clap::{App, Arg};

fn main() {
    let matches = App::new("provide")
        .about("Provides environment variables from AWS Parameter Store")
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
            .help("Use credentials from a local profile"))
        .arg(Arg::with_name("recursive")
            .short("r")
            .long("recursive")
            .takes_value(false)
            .help("Return values recursively under PATH"))
        .get_matches();

    let path = matches.value_of("path").unwrap();
    let recursive = matches.is_present("recursive");
    let profile = matches.value_of("profile");
    run(path, recursive, profile);
}

fn run(path: &str, recursive: bool, profile: Option<&str>) {
    match get_parameters(path, recursive, profile) {
        Ok(parameters) => {
            let map = to_hash_map(parameters);
            print!("{}", as_env_format(map));
        }
        Err(err) => println!("{:?}", err),
    };
}
