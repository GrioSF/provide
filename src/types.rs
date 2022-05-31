use std::{collections::HashMap, iter::FromIterator};

use aws_sdk_ssm::{model::Parameter, Client};

#[derive(Debug, PartialEq)]
pub enum Mode {
  GET,
  SET,
}

#[derive(Debug, PartialEq, Default)]
pub struct ProcessParametersOptions {
  pub app: Option<String>,
  pub env_vars: Option<Vec<String>>,
  pub env_vars_base64: Option<Vec<String>>,
  pub format_config: FormatConfig,
  pub includes: Option<Vec<String>>,
  pub merges: Option<Vec<String>>,
  pub mode: Option<Mode>,
  pub path: Option<String>,
  pub run_config: Option<RunConfig>,
  pub target: Option<String>,
}

pub struct GetAWSParametersOptions {
  pub path: String,
  pub acc: Vec<Parameter>,
  pub client: Client,
}
#[derive(Debug, PartialEq)]
pub struct Pair(pub String, pub String);

impl From<(String, String)> for Pair {
  fn from(tuple: (String, String)) -> Self {
    Pair(tuple.0, tuple.1)
  }
}

impl FromIterator<Pair> for HashMap<String, String> {
  fn from_iter<T: IntoIterator<Item = Pair>>(iter: T) -> Self {
    let mut map = HashMap::<String, String>::new();
    for p in iter {
      map.insert(p.0, p.1);
    }
    map
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Format {
  EXPORT,
  ENV,
  JSON,
}

impl Default for Format {
  fn default() -> Self {
    Format::ENV
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct FormatConfig {
  pub format: Format,
  pub raw: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RunConfig {
  pub cmd: String,
  pub args: Vec<String>,
}
