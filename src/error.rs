use base64;
use regex;
use rusoto_core::RusotoError;
use rusoto_core::region::ParseRegionError;
use rusoto_ssm::GetParametersByPathError;
use std::convert::From;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::str;
use std::string;

#[derive(Debug, PartialEq)]
pub enum ProvideError {
    Error(String),
    BadRegex(regex::Error),
    BadFormat(String),
    InvalidPathError(String),
    GetParametersByPathError(RusotoError<GetParametersByPathError>),
    ParseRegionError(ParseRegionError),
    IOError(io::ErrorKind, String),
    Base64Error(base64::DecodeError),
    UTF8Error(str::Utf8Error),
    EnvError(env::VarError),
}

impl Error for ProvideError {}

impl fmt::Display for ProvideError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProvideError::Error(message) => f.write_fmt(format_args!("Error: {}", message)),
            ProvideError::BadRegex(err) => f.write_fmt(format_args!("BadRegex: {:?}", err)),
            ProvideError::BadFormat(message) => f.write_fmt(format_args!("BadFormat: {}", message)),
            ProvideError::GetParametersByPathError(err) => {
                f.write_fmt(format_args!("GetParametersByPathError: {}", err))
            }
            ProvideError::InvalidPathError(message) => {
                f.write_fmt(format_args!("InvalidPathError: {}", message))
            }
            ProvideError::ParseRegionError(err) => {
                f.write_fmt(format_args!("ParseRegionError: {}", err))
            }
            ProvideError::IOError(kind, message) => {
                f.write_fmt(format_args!("IOError: {:?} {}", kind, message))
            }
            ProvideError::Base64Error(err) => f.write_fmt(format_args!("Base64Error: {}", err)),
            ProvideError::UTF8Error(err) => f.write_fmt(format_args!("UTF8Error: {}", err)),
            ProvideError::EnvError(err) => f.write_fmt(format_args!("EnvError: {}", err)),
        }
    }
}

impl From<RusotoError<GetParametersByPathError>> for ProvideError {
    fn from(err: RusotoError<GetParametersByPathError>) -> Self {
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
        let message = err.description().to_owned();
        ProvideError::IOError(err.kind(), message)
    }
}

impl From<base64::DecodeError> for ProvideError {
    fn from(err: base64::DecodeError) -> Self {
        ProvideError::Base64Error(err)
    }
}

impl From<str::Utf8Error> for ProvideError {
    fn from(err: str::Utf8Error) -> Self {
        ProvideError::UTF8Error(err)
    }
}

impl From<string::FromUtf8Error> for ProvideError {
    fn from(err: string::FromUtf8Error) -> Self {
        ProvideError::UTF8Error(err.utf8_error())
    }
}

impl From<env::VarError> for ProvideError {
    fn from(err: env::VarError) -> Self {
        ProvideError::EnvError(err)
    }
}
