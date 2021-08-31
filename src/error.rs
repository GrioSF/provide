use base64;
use regex;
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
            ProvideError::InvalidPathError(message) => {
                f.write_fmt(format_args!("InvalidPathError: {}", message))
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

impl From<io::Error> for ProvideError {
    fn from(err: io::Error) -> Self {
        ProvideError::IOError(err.kind(), err.to_string())
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
