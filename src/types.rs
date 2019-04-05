use rusoto_core::{Region};
use rusoto_ssm::{Parameter};
use std::path::PathBuf;

pub struct Options {
    pub include: Option<PathBuf>,
    pub path: String, 
    pub region: Region, 
    pub format: Format
}

pub struct GetConfig {
    pub path: String, 
    pub region: Region, 
    pub next_token: Option<String>, 
    pub acc: Box<Vec<Parameter>>
}

pub struct Pair {
    pub key: String,
    pub val: String
}

impl Pair {
    pub fn new(key: &str, val: &str) -> Pair {
        Pair {
            key: String::from(key),
            val: String::from(val)
        }
    }
}

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