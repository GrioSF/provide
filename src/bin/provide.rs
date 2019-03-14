use std::error::Error;
use std::collections::HashMap;
use rusoto_core::{HttpClient, Region};
use rusoto_core::credential::EnvironmentProvider;
use rusoto_ssm::*;
use regex::Regex;

// aws ssm get-parameters-by-path --region us-west-1 --with-decryption --path /moment/staging
fn main() {
    let path = String::from("/moment/staging");
    match get_parameters(path) {
        Ok(params) => {
            let map = to_hash_map(params);
            print!("{}", as_env_format(map));
        }
        Err(err) => println!("{:?}", err),
    };
}

fn get_parameters(path: String) -> Result<Box<Vec<Parameter>>, Box<Error>> {
    fn get_parameters_with_acc(path: String, next_token: Option<String>, mut acc: Box<Vec<Parameter>>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
        println!("Retrieving parameters");
        let region = Region::UsWest1;
        let request_dispatcher = HttpClient::new().unwrap();
        let credentials_provider = EnvironmentProvider::default();
        let ssm_client = SsmClient::new_with(request_dispatcher, credentials_provider, region);
        let request = GetParametersByPathRequest{
            path: path.clone(),
            next_token: next_token,
            recursive: Some(false),
            with_decryption: Some(false),
            parameter_filters: None,
            max_results: None,
        };
        match ssm_client.get_parameters_by_path(request).sync() {
            Ok(output) => {
                acc.append(&mut output.parameters.unwrap());
                match output.next_token {
                    Some(token) => get_parameters_with_acc(path.clone(), Some(token), acc),
                    None => Ok(acc)
                }
            },
            Err(err) => Err(Box::new(err)),
        }
    }
    get_parameters_with_acc(path, None, Box::new(Vec::<Parameter>::new()))
}

fn to_hash_map(params: Box<Vec<Parameter>>) -> HashMap<String, String> {
    let mut hash_map: HashMap<String, String> = HashMap::new();
    for param in params.into_iter() {
        let key = extract_name_from_path(param.name.unwrap().clone());
        let value = param.value.unwrap();
        hash_map.insert(key, value);
    }
    hash_map
}

fn extract_name_from_path(param_path: String) -> String {
    let re = Regex::new(r"^.*/(.*)$").unwrap();
    re.captures(&param_path).take().unwrap().get(1).unwrap().as_str().to_owned()
}

/*
    Outputs String with the following format:
    WHAT="EVERS"\n
    FOO="BAR"\n

    Intended to output to a file or to be evaled
*/
fn as_env_format(map: HashMap<String, String>) -> String {
    let lines: Vec<String> = map.into_iter().map(|(key, value)| format!("{}=\"{}\"\n", key, value)).collect();
    lines.join("")
}
