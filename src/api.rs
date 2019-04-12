use std::collections::HashMap;
use std::path::MAIN_SEPARATOR;
use std::path::PathBuf;
use std::io::{Cursor, BufRead, BufReader};
use std::fs::File;
use std::process::{Command, Stdio};
use std::str;
use rusoto_ssm::*;
use regex::{Regex};
use base64;
use crate::types::*;
use crate::error::ProvideError;

pub fn get_parameters(options: Options) -> Result<HashMap<String, String>, ProvideError> {
    let mut map = HashMap::<String,String>::new();
    match options.mode {
        Some(Mode::GET) => {
            let aws_parameters = get_parameters_with_acc(GetConfig {
                path: options.path.unwrap(), 
                region: options.region, 
                next_token: None, 
                acc: Box::new(Vec::<Parameter>::new())
            })?;
            let params_map = params_as_hash_map(aws_parameters)?;
            map.extend(params_map);
        },
        _ => ()
    }
    if let Some(include_map) = match &options.include {
        Some(path_buf) => Some(read_pairs_from_file(path_buf)?),
        None => None
    } {
        map.extend(include_map);
    };
    if options.env_vars.is_some() {
        let given_map = merge_with_given(options.env_vars.unwrap())?;
        map.extend(given_map);
    };
    if let Some(merge_map) = match &options.merge {
        Some(path_buf) => Some(merge_with_command(path_buf)?),
        None => None
    } {
        map.extend(merge_map);
    };
    Ok(map)
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

pub fn read_pairs_from_file(path: &PathBuf) -> Result<HashMap<String, String>, ProvideError> {
    let reader = with_file(path.to_path_buf())?;
    read_from_reader(reader)
}

pub fn read_from_reader(reader: Box<BufRead>) -> Result<HashMap<String, String>, ProvideError> {
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
            let encoded_val = &line[index+1..];
            let val = str::from_utf8(&base64::decode(encoded_val)?)?.to_owned();
            Ok((key, val))
        },
        None => Err(ProvideError::BadFormat(String::from(format!("Invalid key=value pair {}", line))))
    }?;
    Ok(Some((key.to_owned(), val.to_owned())))
}

fn with_file(path: PathBuf) -> Result<Box<BufRead>, ProvideError> {
    let f: File = File::open(path)?;
    let reader: Box<BufRead> = Box::new(BufReader::new(f));
    Ok(reader)
}

pub fn params_as_hash_map(params: Box<Vec<Parameter>>) -> Result<HashMap<String, String>, ProvideError> {
    params.into_iter().map(|param| {
        let key = extract_key_from_path(&param.name.unwrap())?;
        let val = param.value.unwrap();
        Ok((key.to_owned(), val.to_owned()))
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

/*
    Outputs String with the following format:
    WHAT="EVERS"\n
    FOO="BAR"\n

    Intended to output to a file or to be evaled
*/
pub fn as_env_format(map: HashMap<String, String>, raw: bool) -> String {
    let lines: Vec<String> = map.into_iter()
        .map(|(k, v)| {
            let key = k.to_uppercase();
            let val = if raw { v } else { base64::encode(&v) };
            format!("{}={}\n", key, val)
        })
        .collect();
    lines.join("")
}

pub fn as_export_format(map: HashMap<String, String>, raw: bool) -> String {
    let lines: Vec<String> = map.into_iter()
        .map(|(k, v)| {
            let key = k.to_uppercase();
            if raw {
                let val = base64::encode(&v);
                format!("export {}={}\n", key, val)
            } else {
                let val = base64::encode(&v);
                format!("export {}=$(base64 --decode <<< \"{}\")\n", key, val)
            }
        })
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

pub fn merge_with_given(lines: Vec<String>) -> Result<HashMap<String, String>, ProvideError> {
    let lines_iter = lines.into_iter().map(|line| parse_line(line));
    let lines: Result<Vec<Option<Pair>>, ProvideError> = lines_iter.collect();
    match lines {
        Ok(list) => Ok(list.into_iter().filter(|pair| pair.is_some()).map(|p| p.unwrap()).collect()),
        Err(err) => Err(err)
    }
}

pub fn merge_with_command(path: &PathBuf) -> Result<HashMap<String, String>, ProvideError> {
    let output = Command::new(path).output()?;
    read_from_reader(Box::new(BufReader::new(Cursor::new(output.stdout))))
}

pub fn run(run: Run, vars: HashMap<String, String>) -> Result<(), ProvideError> {
    let filename = run.cmd;
    let mut command = Command::new(&filename);
    &command.envs(vars);
    &command.stdout(Stdio::inherit());
    &command.stderr(Stdio::inherit());
    &command.args(run.args);
    match command.spawn() {
        Ok(_) => Ok(()),
        Err(err) => Err(From::from(err))
    }
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
        let map: HashMap<String, String> = vec![
            ("one".to_owned(), "bar".to_owned()),
            ("two".to_owned(), "baz".to_owned()),
            ("THREE".to_owned(), "clock".to_owned()),
        ].into_iter().collect();
        let env_format = as_env_format(map, true);
        let mut result: Vec<&str> = env_format.trim().split("\n").collect();
        result.sort();
        assert_eq!(result, vec!["ONE=bar", "THREE=clock", "TWO=baz"]);
    }

    #[test]
    fn test_escape_for_bash() {
        assert_eq!(escape_for_bash(r#"a$`"\'!)&"#), r#"a\$\`\"\\'\!\)&"#);
    }
}
