use std::error::Error;
use std::collections::HashMap;
use rusoto_core::{Region};
use rusoto_ssm::*;
use regex::Regex;
use std::env;
use crate::types::{GetConfig};

pub fn get_parameters(path: String, recursive: bool, profile: Option<&str>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
    if profile.is_some() {
        // The only way I've found to have rusoto honor given profile since ProfileProvider
        // ignores it. Note this is only set for the current process.
        env::set_var("AWS_PROFILE", profile.unwrap());
    }
    let region = Region::default(); // will return a region defined in the env, profile, or default see method doc
    get_parameters_with_acc(GetConfig {
        path: path, 
        recursive: recursive, 
        region: region, 
        next_token: None, 
        acc: Box::new(Vec::<Parameter>::new())
    })
}

fn get_parameters_with_acc(mut get_config: GetConfig) -> Result<Box<Vec<Parameter>>, Box<Error>> {
    let request = GetParametersByPathRequest{
        path: get_config.path.clone(),
        next_token: get_config.next_token,
        recursive: Some(get_config.recursive),
        with_decryption: Some(false),
        parameter_filters: None,
        max_results: None,
    };
    let ssm_client = SsmClient::new(get_config.region.clone());
    match ssm_client.get_parameters_by_path(request).sync() {
        Ok(output) => {
            get_config.acc.append(&mut output.parameters.unwrap());
            match output.next_token {
                Some(token) => get_parameters_with_acc(GetConfig {
                    path: get_config.path, 
                    recursive: get_config.recursive, 
                    region: get_config.region, 
                    next_token: Some(token), 
                    acc: get_config.acc
                }),
                None => Ok(get_config.acc)
            }
        },
        Err(err) => Err(Box::new(err)),
    }
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
