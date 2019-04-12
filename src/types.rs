use std::path::PathBuf;
use rusoto_core::{Region};
use rusoto_ssm::{Parameter};

#[derive(Debug)]
pub enum Mode {
  GET,
  SET
}

#[derive(Debug)]
pub struct Options {
    pub mode: Option<Mode>,
    pub format_config: FormatConfig,
    pub include: Option<PathBuf>,
    pub merge: Option<PathBuf>,
    pub app: Option<String>, 
    pub target: Option<String>, 
    pub path: Option<String>, 
    pub region: Region, 
    pub run: Option<Run>,
    pub env_vars: Option<Vec<String>>,
}

pub struct GetConfig {
    pub path: String, 
    pub region: Region, 
    pub next_token: Option<String>, 
    pub acc: Box<Vec<Parameter>>
}

pub type Pair = (String, String);

#[derive(Copy, Clone, Debug)]
pub enum Format {
    EXPORT,
    ENV,
    JSON
}

#[derive(Copy, Clone, Debug)]
pub struct FormatConfig {
    pub format: Format,
    pub raw: bool
}

impl Default for Format {
  fn default() -> Self {
    Format::ENV
  }
}

#[derive(Clone, Debug)]
pub struct Run {
  pub cmd: PathBuf,
  pub args: Vec<String>
}
