use std::convert::From;
use rusoto_core::region::ParseRegionError;
use rusoto_ssm::{GetParametersByPathError};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ProvideError {
    InvalidPathError(String),
    GetParametersByPathError(GetParametersByPathError),
    ParseRegionError(ParseRegionError),
}

impl Error for ProvideError {}

impl fmt::Display for ProvideError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            ProvideError::GetParametersByPathError(err) => format!("GetParametersByPathError: {}", err),
            ProvideError::InvalidPathError(message) => format!("InvalidPathError: {}", message),
            ProvideError::ParseRegionError(err) => format!("ParseRegionError: {}", err),
        };
        write!(f , "{}", message)
    }
}

impl From<GetParametersByPathError> for ProvideError {
    fn from(err: GetParametersByPathError) -> Self {
        ProvideError::GetParametersByPathError(err)
    }
}

impl From<ParseRegionError> for ProvideError {
    fn from(err: ParseRegionError) -> Self {
        ProvideError::ParseRegionError(err)
    }
}

