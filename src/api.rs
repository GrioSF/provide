use std::error::Error;
use std::collections::HashMap;
use rusoto_core::{HttpClient, Region};
use rusoto_core::credential::{EnvironmentProvider, ProfileProvider, CredentialsError};
use rusoto_ssm::*;
use regex::Regex;
use std::path::{PathBuf};
use dirs::home_dir;

pub fn get_parameters(path: &str, recursive: bool, profile: Option<&str>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
    fn get_parameters_with_acc(path: &str, recursive: bool, profile: Option<&str>, next_token: Option<String>, mut acc: Box<Vec<Parameter>>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
        let region = Region::UsWest1;
        let request_dispatcher = HttpClient::new().unwrap();
        let ssm_client = match profile {
            Some(name) => {
                let file_path = default_credentials_file_path()?;
                let credentials_provider = ProfileProvider::with_configuration(file_path, name);
                SsmClient::new_with(request_dispatcher, credentials_provider, region)
            },
            None => {
                let credentials_provider = EnvironmentProvider::default();
                SsmClient::new_with(request_dispatcher, credentials_provider, region)
            }
        };
        let request = GetParametersByPathRequest{
            path: String::from(path),
            next_token: next_token,
            recursive: Some(recursive),
            with_decryption: Some(false),
            parameter_filters: None,
            max_results: None,
        };
        match ssm_client.get_parameters_by_path(request).sync() {
            Ok(output) => {
                acc.append(&mut output.parameters.unwrap());
                match output.next_token {
                    Some(token) => get_parameters_with_acc(path, recursive, profile, Some(token), acc),
                    None => Ok(acc)
                }
            },
            Err(err) => Err(Box::new(err)),
        }
    }
    get_parameters_with_acc(path, recursive, profile, None, Box::new(Vec::<Parameter>::new()))
}

pub fn to_hash_map(params: Box<Vec<Parameter>>) -> HashMap<String, String> {
    let mut hash_map: HashMap<String, String> = HashMap::new();
    for param in params.into_iter() {
        let key = extract_name_from_path(param.name.unwrap().clone());
        let value = param.value.unwrap();
        hash_map.insert(key, value);
    }
    hash_map
}

// /app/staging/key => key
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
pub fn as_env_format(map: HashMap<String, String>) -> String {
    let lines: Vec<String> = map.into_iter().map(|(key, value)| format!("{}=\"{}\"\n", key.to_uppercase(), value)).collect();
    lines.join("")
}

fn default_credentials_file_path() -> Result<PathBuf, Box<Error>> {
    match home_dir() {
        Some(mut home_path) => {
            home_path.push(".aws");
            home_path.push("credentials");
            Ok(home_path)
        },
        None => Err(Box::new(CredentialsError::new("Failed to determine home directory.")))
    }
}
