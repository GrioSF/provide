use std::collections::HashMap;
use std::path::MAIN_SEPARATOR;
use std::path::PathBuf;
use std::io::{Cursor, BufRead, BufReader};
use std::fs;
use std::process::{Command, Stdio};
use std::str;
use std::env;
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
    if let Some(include_maps) = match &options.includes {
        Some(path_bufs) => Some(read_pairs_from_files(path_bufs, false)?),
        None => None
    } {
        for include_map in include_maps.into_iter() {
            map.extend(include_map);
        }
    };
    if options.env_vars.is_some() {
        for env_var in options.env_vars.unwrap() {
            let (key, val): Pair = merge_with_env(env_var, false)?;
            map.entry(key).or_insert(val);
        }
    };
    if options.env_vars_base64.is_some() {
        for env_var in options.env_vars_base64.unwrap() {
            let (key, val): Pair = merge_with_env(env_var, true)?;
            map.entry(key).or_insert(val);
        }
    };
    if let Some(merge_maps) = match &options.merges {
        Some(path_bufs) => Some(merge_with_commands(path_bufs, &map)?),
        None => None
    } {
        for merge_map in merge_maps.into_iter() {
            map.extend(merge_map);
        }
    };
    if let Some(app) = options.app {
        map.entry("PROVIDE_APPLICATION".to_owned()).or_insert(app);
    };
    if let Some(target) = options.target {
        map.entry("PROVIDE_TARGET".to_owned()).or_insert(target);
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

pub fn read_pairs_from_files(paths: &Vec<PathBuf>, use_base64: bool) -> Result<Vec<HashMap<String, String>>, ProvideError> {
    paths.iter().map(|path| read_pairs_from_file(path, use_base64)).collect()
}

pub fn read_pairs_from_file(path: &PathBuf, use_base64: bool) -> Result<HashMap<String, String>, ProvideError> {
    let path_buf = fs::canonicalize(path)?;
    let reader = with_file(path_buf)?;
    read_from_reader(reader, use_base64)
}

pub fn read_from_reader(reader: Box<BufRead>, use_base64: bool) -> Result<HashMap<String, String>, ProvideError> {
    let lines_iter = reader.lines().map(|line| {
        match line {
            Ok(text) => parse_line(text, use_base64),
            Err(err) => Err(From::from(err))
        }
    });
    let lines: Result<Vec<Option<Pair>>, ProvideError> = lines_iter.collect();
    match lines {
        Ok(list) => Ok(list.into_iter().filter(|pair| pair.is_some()).map(|p| p.unwrap()).collect()),
        Err(err) => Err(err)
    }
}

fn parse_line(line: String, use_base64: bool) -> Result<Option<Pair>, ProvideError> {
    if line.is_empty() {
        return Ok(None);
    }
    let (key, val) = match line.find("=") {
        Some(0) => Err(ProvideError::BadFormat(String::from("Invalid key has no length"))),
        Some(index) => {
            let key = &line[0..index];
            let encoded_val = &line[index+1..];
            let val = match use_base64 {
                true => str::from_utf8(&base64::decode(encoded_val)?)?.to_owned(),
                false => encoded_val.to_owned()
            };
            Ok((key, val))
        },
        None => Err(ProvideError::BadFormat(String::from(format!("Invalid key=value pair {}", line))))
    }?;
    Ok(Some((key.to_owned(), val.to_owned())))
}

fn with_file(path: PathBuf) -> Result<Box<BufRead>, ProvideError> {
    let f: fs::File = fs::File::open(path)?;
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

pub fn merge_with_given(lines: Vec<String>, use_base64: bool) -> Result<HashMap<String, String>, ProvideError> {
    let lines_iter = lines.into_iter().map(|line| parse_line(line, use_base64));
    let lines: Result<Vec<Option<Pair>>, ProvideError> = lines_iter.collect();
    match lines {
        Ok(list) => Ok(list.into_iter().filter(|pair| pair.is_some()).map(|p| p.unwrap()).collect()),
        Err(err) => Err(err)
    }
}

pub fn merge_with_env(line: String, use_base64: bool) -> Result<Pair, ProvideError> {
    let key = line;
    let env_val = env::var(&key)?;
    let val = if use_base64 {
        str::from_utf8(&base64::decode(&env_val)?)?.to_owned()
    } else {
        env_val
    };
    Ok((key, val))
}

pub fn merge_with_commands(paths: &Vec<PathBuf>, vars: &HashMap<String, String>) -> Result<Vec<HashMap<String, String>>, ProvideError> {
    paths.iter().map(|path| merge_with_command(path, vars)).collect()
}

pub fn merge_with_command(path: &PathBuf, vars: &HashMap<String, String>) -> Result<HashMap<String, String>, ProvideError> {
    let path_buf = fs::canonicalize(path)?;
    let mut command = Command::new(path_buf);
    &command.envs(vars);
    let output = command.output()?;
    match output.status.code() {
        Some(0) => read_from_reader(Box::new(BufReader::new(Cursor::new(output.stdout))), true),
        Some(_) => Err(ProvideError::Error(String::from(str::from_utf8(&output.stderr)?))),
        None => Err(ProvideError::Error(format!("Terminated by signal")))
    }
}

pub fn run(run: Run, vars: HashMap<String, String>) -> Result<(), ProvideError> {
    let filename = run.cmd;
    let mut command = Command::new(&filename);
    &command.envs(vars);
    &command.stdout(Stdio::inherit());
    &command.stderr(Stdio::inherit());
    &command.args(run.args);
    match command.status() {
        Ok(status) => match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(ProvideError::Error(format!("Exit code {}", code))),
            None => Err(ProvideError::Error(format!("Terminated by signal")))
        },
        Err(err) => Err(From::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
 
    fn encode_pair((key, val): Pair, use_base64: bool) -> String {
        let encoded_val = if use_base64 {
            base64::encode(&val)
        } else {
            val
        };
        format!("{}={}", key, encoded_val)
    }

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

    #[test]
    fn test_read_from_reader() {
        let pair1 = encode_pair(("foo".to_owned(), "bar".to_owned()), false);
        let pair2 = encode_pair(("baz".to_owned(), "qux".to_owned()), false);
        let source = format!("{}\n{}\n", pair1, pair2).into_bytes();
        let result = read_from_reader(Box::new(BufReader::new(Cursor::new(source))), false).unwrap();
        let expected: HashMap<String, String> = vec![
            ("foo".to_owned(), "bar".to_owned()),
            ("baz".to_owned(), "qux".to_owned()),
        ].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_read_from_reader_as_base64() {
        let pair1 = encode_pair(("foo".to_owned(), "bar".to_owned()), true);
        let pair2 = encode_pair(("baz".to_owned(), "qux".to_owned()), true);
        let source = format!("{}\n{}\n", pair1, pair2).into_bytes();
        let result = read_from_reader(Box::new(BufReader::new(Cursor::new(source))), true).unwrap();
        let expected: HashMap<String, String> = vec![
            ("foo".to_owned(), "bar".to_owned()),
            ("baz".to_owned(), "qux".to_owned()),
        ].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_read_from_reader_with_extra_lines() {
        let pair1 = encode_pair(("foo".to_owned(), "bar".to_owned()), true);
        let pair2 = encode_pair(("baz".to_owned(), "qux".to_owned()), true);
        let source = format!("{}\n\r\n\n{}\n\n", pair1, pair2).into_bytes();
        let result = read_from_reader(Box::new(BufReader::new(Cursor::new(source))), true).unwrap();
        let expected: HashMap<String, String> = vec![
            ("foo".to_owned(), "bar".to_owned()),
            ("baz".to_owned(), "qux".to_owned()),
        ].into_iter().collect();
        assert_eq!(result, expected);
    }

}
