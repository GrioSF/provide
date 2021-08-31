use aws_sdk_ssm::{Client, model::Parameter};

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
    pub next_token: Option<String>,
    pub acc: Vec<Parameter>,
    pub client: Client
}

pub type Pair = (String, String);

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
