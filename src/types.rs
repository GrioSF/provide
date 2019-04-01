use rusoto_core::{Region};
use rusoto_ssm::{Parameter};

pub struct Options {
    pub path: String, 
    pub region: Region, 
}

pub struct GetConfig {
    pub path: String, 
    pub region: Region, 
    pub next_token: Option<String>, 
    pub acc: Box<Vec<Parameter>>
}
