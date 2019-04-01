use std::collections::HashMap;
use rusoto_core::{Region};
use rusoto_ssm::*;
use std::path::MAIN_SEPARATOR;
use crate::types::{GetConfig};
use crate::error::ProvideError;

pub fn get_parameters(path: String, region: Region) -> Result<Box<Vec<Parameter>>, ProvideError> {
    get_parameters_with_acc(GetConfig {
        path: path, 
        region: region, 
        next_token: None, 
        acc: Box::new(Vec::<Parameter>::new())
    })
}

fn get_parameters_with_acc(mut get_config: GetConfig) -> Result<Box<Vec<Parameter>>, ProvideError> {
    let request = GetParametersByPathRequest{
        path: get_config.path.clone(),
        next_token: get_config.next_token,
        recursive: Some(false),
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
                    region: get_config.region, 
                    next_token: Some(token), 
                    acc: get_config.acc
                }),
                None => Ok(get_config.acc)
            }
        },
        Err(err) => Err(From::from(err)),
    }
}

pub fn to_hash_map(params: Box<Vec<Parameter>>) -> Result<HashMap<String, String>, ProvideError> {
    let mut hash_map: HashMap<String, String> = HashMap::new();
    for param in params.into_iter() {
        let key = extract_key_from_path(&param.name.unwrap())?;
        let value = param.value.unwrap();
        hash_map.insert(key, value);
    }
    Ok(hash_map)
}

// /app/staging/key => key
fn extract_key_from_path(param_path: &str) -> Result<String, ProvideError> {
    let candidate = param_path.trim_start_matches(MAIN_SEPARATOR);
    let mut segments: Vec<String> = candidate.split(MAIN_SEPARATOR).map(|x| x.to_string()).collect();
    match segments.len() {
        3 => Ok(segments.pop().unwrap()),
        _ => Err(ProvideError::InvalidPathError(format!("Invalid path {}", param_path)))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn test_extract_name_from_path() {
        assert_eq!(extract_key_from_path("/app/env/DATABASE_URL").unwrap(), "DATABASE_URL");
        assert_eq!(extract_key_from_path("/app/env/foo").unwrap(), "foo");
        assert_eq!(
            extract_key_from_path("/app/foo").unwrap_err(), 
            ProvideError::InvalidPathError(String::from("Invalid path /app/foo"))
        );
        assert_eq!(
            extract_key_from_path("/app/foo/bar/car").unwrap_err(), 
            ProvideError::InvalidPathError(String::from("Invalid path /app/foo/bar/car"))
        );
    }

    #[test]
    fn test_as_env_format() {
        let map: HashMap<String, String> = vec![
            ("/app/env/one", "bar"),
            ("/app/env/two", "baz"),
            ("/app/env/THREE", "clock"),
        ].into_iter().map(|(k, v)| (extract_key_from_path(k).unwrap(), String::from(v))).collect();
        let env_format = as_env_format(map);
        let mut result: Vec<&str> = env_format.trim().split("\n").collect();
        result.sort();
        assert_eq!(result, vec!["ONE=\"bar\"", "THREE=\"clock\"", "TWO=\"baz\""]);
    }
}
