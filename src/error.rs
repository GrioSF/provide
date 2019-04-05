use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;
use regex;
use rusoto_core::region::ParseRegionError;
use rusoto_ssm::{GetParametersByPathError};

#[derive(Debug, PartialEq)]
pub enum ProvideError {
    BadRegex(regex::Error),
    BadFormat(String),
    InvalidPathError(String),
    GetParametersByPathError(GetParametersByPathError),
    ParseRegionError(ParseRegionError),
    IOError(io::ErrorKind)
}

impl Error for ProvideError {}

impl fmt::Display for ProvideError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            ProvideError::BadRegex(err) => format!("BadRegex: {:?}", err),
            ProvideError::BadFormat(message) => format!("BadFormat: {}", message),
            ProvideError::GetParametersByPathError(err) => format!("GetParametersByPathError: {}", err),
            ProvideError::InvalidPathError(message) => format!("InvalidPathError: {}", message),
            ProvideError::ParseRegionError(err) => format!("ParseRegionError: {}", err),
            ProvideError::IOError(kind) => format!("IOError: {:?}", kind),
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

impl From<io::Error> for ProvideError {
    fn from(err: io::Error) -> Self {
        ProvideError::IOError(err.kind())
    }
}
