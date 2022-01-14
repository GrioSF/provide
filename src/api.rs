use crate::error::Error;
use crate::types::*;
use aws_sdk_ssm::Client;
use base64;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use std::process::{Command, Stdio};
use std::str;
use tokio_stream::StreamExt;

pub async fn process_parameters(
  options: ProcessParametersOptions,
) -> Result<HashMap<String, String>, Error> {
  let mut map = HashMap::<String, String>::new();
  match options.mode {
    Some(Mode::GET) => {
      let params_map = read_from_aws(options.path.unwrap()).await?;
      map.extend(params_map);
    }
    _ => (),
  }
  if let Some(include_maps) = match &options.includes {
    Some(path_bufs) => Some(read_pairs_from_files(path_bufs, true)?),
    None => None,
  } {
    for include_map in include_maps.into_iter() {
      map.extend(include_map);
    }
  };
  if let Some(env_var_map) = match &options.env_vars {
    Some(lines) => Some(merge_with_given(lines, false)?),
    None => None,
  } {
    map.extend(env_var_map);
  };
  if let Some(env_var_map) = match &options.env_vars_base64 {
    Some(lines) => Some(merge_with_given(lines, true)?),
    None => None,
  } {
    map.extend(env_var_map);
  };
  if let Some(merge_maps) = match &options.merges {
    Some(path_bufs) => Some(merge_with_commands(path_bufs, &map)?),
    None => None,
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

async fn read_from_aws(path: String) -> Result<HashMap<String, String>, Error> {
  let shared_config = aws_config::load_from_env().await;
  let client = Client::new(&shared_config);
  let mut map = HashMap::<String, String>::new();
  let mut stream = client
    .get_parameters_by_path()
    .path(path)
    .recursive(false)
    .with_decryption(false)
    .into_paginator()
    .send();

  while let Some(o) = stream.next().await {
    if let Some(parameters) = o?.parameters {
      for p in parameters {
        if let (Some(name), Some(val)) = (p.name(), p.value()) {
          let key = extract_key_from_path(&name)?;
          map.insert(key, val.to_string());
        }
      }
    }
  }
  Ok(map)
}

pub fn read_pairs_from_files(
  paths: &Vec<String>,
  use_base64: bool,
) -> Result<Vec<HashMap<String, String>>, Error> {
  paths
    .iter()
    .map(|path| read_pairs_from_file(path, use_base64))
    .collect()
}

pub fn read_pairs_from_file(
  path: &String,
  use_base64: bool,
) -> Result<HashMap<String, String>, Error> {
  let path_buf = fs::canonicalize(path)?;
  let reader = with_file(path_buf)?;
  read_from_reader(reader, use_base64)
}

pub fn read_from_reader(
  reader: impl BufRead,
  use_base64: bool,
) -> Result<HashMap<String, String>, Error> {
  let lines_iter = reader.lines().map(|line| parse_line(&line?, use_base64));
  let lines: Vec<Option<Pair>> = lines_iter.collect::<Result<Vec<Option<Pair>>, Error>>()?;
  Ok(lines.into_iter().filter_map(|p| p).collect())
}

fn parse_line(line: &str, use_base64: bool) -> Result<Option<Pair>, Error> {
  if line.is_empty() {
    return Ok(None);
  }
  let (key, val) = match line.find("=") {
    Some(0) => Err(Error::BadFormat(String::from("Invalid key has no length"))),
    Some(index) => {
      let key = &line[0..index];
      let encoded_val = &line[index + 1..];
      let val = match use_base64 {
        true => String::from_utf8(base64::decode(encoded_val)?)?,
        false => encoded_val.to_owned(),
      };
      Ok((key, val))
    }
    None => Err(Error::BadFormat(String::from(format!(
      "Invalid key=value pair {}",
      line
    )))),
  }?;
  Ok(Some(Pair(key.to_owned(), val.to_owned())))
}

fn with_file(path: PathBuf) -> Result<impl BufRead, Error> {
  let f: fs::File = fs::File::open(path)?;
  let reader = BufReader::new(f);
  Ok(reader)
}

// /app/staging/key => key
fn extract_key_from_path(param_path: &str) -> Result<String, Error> {
  let candidate = param_path.trim_start_matches(MAIN_SEPARATOR);
  let mut segments: Vec<String> = candidate
    .split(MAIN_SEPARATOR)
    .map(|x| x.to_string())
    .collect();
  match segments.len() {
    3 => Ok(segments.pop().unwrap()),
    _ => Err(Error::InvalidPathError(format!(
      "Invalid path {param_path}"
    ))),
  }
}

/*
    Outputs String with the following format:
    WHAT="EVERS"\n
    FOO="BAR"\n

    Intended to output to a file or to be evaled
*/
pub fn as_env_format(map: HashMap<String, String>, raw: bool) -> String {
  let lines: Vec<String> = map
    .into_iter()
    .map(|(k, v)| {
      let key = k.to_uppercase();
      let val = if raw { v } else { base64::encode(&v) };
      format!("{key}={val}\n")
    })
    .collect();
  lines.join("")
}

pub fn as_export_format(map: HashMap<String, String>, raw: bool) -> String {
  let lines: Vec<String> = map
    .into_iter()
    .map(|(k, v)| {
      let key = k.to_uppercase();
      if raw {
        let val = base64::encode(&v);
        format!("export {key}={val}\n")
      } else {
        let val = base64::encode(&v);
        format!("export {key}=$(base64 --decode <<< \"{val}\")\n")
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

pub fn merge_with_given(
  lines: &Vec<String>,
  use_base64: bool,
) -> Result<HashMap<String, String>, Error> {
  let map = lines
    .iter()
    .map(|line| parse_line(line, use_base64))
    .collect::<Result<Vec<Option<Pair>>, Error>>()?
    .into_iter()
    .filter_map(|p| p)
    .collect();
  Ok(map)
}

pub fn merge_with_env(line: String, use_base64: bool) -> Result<Pair, Error> {
  let key = line;
  let env_val = env::var(&key)?;
  let val = if use_base64 {
    String::from_utf8(base64::decode(&env_val)?)?
  } else {
    env_val
  };
  Ok(Pair(key, val))
}

pub fn merge_with_commands(
  paths: &Vec<String>,
  vars: &HashMap<String, String>,
) -> Result<Vec<HashMap<String, String>>, Error> {
  paths
    .iter()
    .map(|path| merge_with_command(path, vars))
    .collect()
}

pub fn merge_with_command(
  path: &String,
  vars: &HashMap<String, String>,
) -> Result<HashMap<String, String>, Error> {
  let path_buf = fs::canonicalize(path)?;
  let mut command = Command::new(path_buf);
  command.envs(vars);
  let output = command.output()?;
  match output.status.code() {
    Some(0) => read_from_reader(BufReader::new(Cursor::new(output.stdout)), true),
    Some(_) => Err(Error::Error(String::from_utf8(output.stderr)?)),
    None => Err(Error::Error(format!("Terminated by signal"))),
  }
}

pub fn run(run_config: RunConfig, vars: HashMap<String, String>) -> Result<(), Error> {
  let filename = run_config.cmd;
  let mut command = Command::new(&filename);
  command.envs(vars);
  command.stdout(Stdio::inherit());
  command.stderr(Stdio::inherit());
  command.args(run_config.args);
  match command.status() {
    Ok(status) => match status.code() {
      Some(0) => Ok(()),
      Some(code) => Err(Error::Error(format!("Exit code {code}"))),
      None => Err(Error::Error(format!("Terminated by signal"))),
    },
    Err(err) => Err(Error::IOError(err)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn encode_pair(pair: Pair, use_base64: bool) -> String {
    match pair {
      Pair(key, val) => {
        let encoded_val = if use_base64 {
          base64::encode(&val)
        } else {
          val
        };
        format!("{key}={encoded_val}")
      }
    }
  }

  #[test]
  fn test_extract_key_from_path() {
    assert_eq!(
      extract_key_from_path("/app/env/DATABASE_URL").unwrap(),
      "DATABASE_URL"
    );
    assert_eq!(extract_key_from_path("/app/env/foo").unwrap(), "foo");
    assert_eq!(
      extract_key_from_path("/app/foo").unwrap_err().to_string(),
      Error::InvalidPathError(String::from("Invalid path /app/foo")).to_string()
    );
    assert_eq!(
      extract_key_from_path("/app/foo/bar/car")
        .unwrap_err()
        .to_string(),
      Error::InvalidPathError(String::from("Invalid path /app/foo/bar/car")).to_string()
    );
  }

  #[test]
  fn test_as_env_format() {
    let map: HashMap<String, String> = vec![
      ("one".to_owned(), "bar".to_owned()),
      ("two".to_owned(), "baz".to_owned()),
      ("THREE".to_owned(), "clock".to_owned()),
    ]
    .into_iter()
    .collect();
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
    let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), false);
    let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), false);
    let source = format!("{pair1}\n{pair2}\n").into_bytes();
    let result = read_from_reader(BufReader::new(Cursor::new(source)), false).unwrap();
    let expected: HashMap<String, String> = vec![
      ("foo".to_owned(), "bar".to_owned()),
      ("baz".to_owned(), "qux".to_owned()),
    ]
    .into_iter()
    .collect();
    assert_eq!(result, expected);
  }

  #[test]
  fn test_read_from_reader_as_base64() {
    let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), true);
    let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), true);
    let source = format!("{pair1}\n{pair2}\n").into_bytes();
    let result = read_from_reader(BufReader::new(Cursor::new(source)), true).unwrap();
    let expected: HashMap<String, String> = vec![
      ("foo".to_owned(), "bar".to_owned()),
      ("baz".to_owned(), "qux".to_owned()),
    ]
    .into_iter()
    .collect();
    assert_eq!(result, expected);
  }

  #[test]
  fn test_read_from_reader_with_extra_lines() {
    let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), true);
    let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), true);
    let source = format!("{pair1}\n\r\n\n{pair2}\n\n").into_bytes();
    let result = read_from_reader(BufReader::new(Cursor::new(source)), true).unwrap();
    let expected: HashMap<String, String> = vec![
      ("foo".to_owned(), "bar".to_owned()),
      ("baz".to_owned(), "qux".to_owned()),
    ]
    .into_iter()
    .collect();
    assert_eq!(result, expected);
  }
}
