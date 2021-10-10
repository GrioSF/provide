use base64;
use regex;
use std::env;
use std::str;
use aws_sdk_ssm::SdkError;
use aws_sdk_ssm::error::GetParametersByPathError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("ArgError: {0}")]
    ArgError(#[from] clap::Error),
    #[error("BadFormat: {0}")]
    BadFormat(String),
    #[error("BadRegex: {0}")]
    BadRegex(#[from] regex::Error),
    #[error("Base64Error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("EnvError: {0}")]
    EnvError(#[from] env::VarError),
    #[error("Error: {0}")]
    Error(String),
    #[error("GetParametersByPathError: {0}")]
    GetParametersByPathError(#[from] SdkError<GetParametersByPathError>),
    #[error("InvalidPathError: {0}")]
    InvalidPathError(String),
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Utf8Error: {0}")]
    StringUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Utf8Error: {0}")]
    StrUtf8Error(#[from] str::Utf8Error),
}

#[test]
fn test_error_display() {
    assert_eq!("BadFormat: reasons", format!("{}", Error::BadFormat("reasons".into())))
}

