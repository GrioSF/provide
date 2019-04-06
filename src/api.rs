use std::collections::HashMap;
use std::path::MAIN_SEPARATOR;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::fs::File;
use rusoto_ssm::*;
use regex::{Regex};
use crate::types::*;
use crate::error::ProvideError;

pub fn get_parameters(options: &Options) -> Result<Vec<Pair>, ProvideError> {
    let aws_parameters = get_parameters_with_acc(GetConfig {
        path: options.path.to_owned(), 
        region: options.region.to_owned(), 
        next_token: None, 
        acc: Box::new(Vec::<Parameter>::new())
    })?;
    let mut pairs = as_pairs(aws_parameters)?;
    if let Some(mut file_pairs) = match &options.include {
        Some(path_buf) => Some(read_pairs_from_file(path_buf)?),
        None => None
    } {
        pairs.append(file_pairs.as_mut());
    };
    Ok(pairs)
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

pub fn read_pairs_from_file(path: &PathBuf) -> Result<Vec<Pair>, ProvideError> {
    let reader = with_file(path.to_path_buf())?;
    let lines_iter = reader.lines().map(|line| {
        match line {
            Ok(text) => parse_line(text),
            Err(err) => Err(From::from(err))
        }
    });
    let lines: Result<Vec<Option<Pair>>, ProvideError> = lines_iter.collect();
    match lines {
        Ok(list) => Ok(list.into_iter().filter(|pair| pair.is_some()).map(|p| p.unwrap()).collect()),
        Err(err) => Err(err)
    }
}

fn parse_line(line: String) -> Result<Option<Pair>, ProvideError> {
    if line.is_empty() {
        return Ok(None);
    }
    let (key, val) = match line.find("=") {
        Some(0) => Err(ProvideError::BadFormat(String::from("Invalid key has no length"))),
        Some(index) => {
            let key = &line[0..index];
            let val = &line[index+1..];
            Ok((key, val))
        },
        None => Err(ProvideError::BadFormat(String::from("Invalid key=value pair")))
    }?;
    Ok(Some(Pair::new(key, val)))
}

fn with_file(path: PathBuf) -> Result<Box<BufRead>, ProvideError> {
    let f: File = File::open(path)?;
    let reader: Box<BufRead> = Box::new(BufReader::new(f));
    Ok(reader)
}

pub fn as_pairs(params: Box<Vec<Parameter>>) -> Result<Vec<Pair>, ProvideError> {
    params.into_iter().map(|param| {
        let key = extract_key_from_path(&param.name.unwrap())?;
        let val = param.value.unwrap();
        Ok(Pair::new(&key, &val))
    }).collect()
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

pub fn as_hash_map(pairs: Vec<Pair>) -> Result<HashMap<String, String>, ProvideError> {
    Ok(pairs.into_iter().map(|pair| (pair.key, pair.val)).collect())
}

/*
    Outputs String with the following format:
    WHAT="EVERS"\n
    FOO="BAR"\n

    Intended to output to a file or to be evaled
*/
pub fn as_env_format(pairs: Vec<Pair>) -> String {
    let lines: Vec<String> = pairs.into_iter()
        .map(|pair| format!("{}={}\n", pair.key.to_uppercase(), pair.val))
        .collect();
    lines.join("")
}

pub fn as_export_format(pairs: Vec<Pair>) -> String {
    let lines: Vec<String> = pairs.into_iter()
        .map(|pair| format!("export {}=\"{}\"\n", pair.key.to_uppercase(), escape_for_bash(&pair.val)))
        .collect();
    lines.join("")
}

lazy_static! {
    static ref RE: Regex = Regex::new(r#"([$`"!\)\\])"#).unwrap();
}

// Escape $`"!)\ for use in bash
pub fn escape_for_bash(val: &str) -> String {
    RE.replace_all(val, "\\$1").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn test_extract_key_from_path() {
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
        let pairs = vec![
            Pair::new("one", "bar"),
            Pair::new("two", "baz"),
            Pair::new("THREE", "clock"),
        ];
        let env_format = as_env_format(pairs);
        let mut result: Vec<&str> = env_format.trim().split("\n").collect();
        result.sort();
        assert_eq!(result, vec!["ONE=bar", "THREE=clock", "TWO=baz"]);
    }

    #[test]
    fn test_escape_for_bash() {
        assert_eq!(escape_for_bash(r#"a$`"\'!)&"#), r#"a\$\`\"\\'\!\)&"#);
    }
}
