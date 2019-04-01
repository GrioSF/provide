use rusoto_core::{Region};
use rusoto_ssm::{Parameter};

pub struct GetConfig {
    pub path: String, 
    pub region: Region, 
    pub next_token: Option<String>, 
    pub acc: Box<Vec<Parameter>>
}

pub type App = String;
pub type Env = String;
pub type Key = String;
pub type Val = String;
pub type Triple = (App, Env, Key);
pub type Quad = (App, Env, Key, Val);
