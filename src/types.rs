use std::path::PathBuf;
use rusoto_core::{Region};
use rusoto_ssm::{Parameter};

pub enum Mode {
  GET,
  SET
}

pub struct Options {
    pub mode: Option<Mode>,
    pub format: Format,
    pub include: Option<PathBuf>,
    pub merge: Option<PathBuf>,
    pub path: String, 
    pub region: Region, 
}

pub struct GetConfig {
    pub path: String, 
    pub region: Region, 
    pub next_token: Option<String>, 
    pub acc: Box<Vec<Parameter>>
}

pub type Pair = (String, String);

pub enum Format {
    EXPORT,
    ENV,
    JSON
}

impl Default for Format {
  fn default() -> Self {
    Format::ENV
  }
}
