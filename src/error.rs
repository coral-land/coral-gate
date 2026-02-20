use std::{io::Error, string::FromUtf8Error};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoralGateError>;

#[derive(Error, Debug)]
pub enum CoralGateError {
    #[error("The environment variable {0} is not set")]
    MissingEnvironment(String),

    #[error("Timeout Error: {0}")]
    TimeoutError(String),

    #[error("Unknown Temp Error")]
    UnknownTempError,

    #[error(transparent)]
    IOError(#[from] Box<Error>),

    #[error("The resource name is missing for {0}")]
    MissingName(String),

    #[error("The reosource namespace is missing for {0}")]
    MissingNamespace(String),

    #[error("Kube Api Error {0}")]
    KubeApiError(#[from] kube::Error),

    #[error("Binary for target {target:?} not found, path: {path:?}")]
    BinaryNotFound { target: String, path: String },

    #[error("Error in command output {0}")]
    CommandOutputError(#[from] std::io::Error),

    #[error("Error output to utf8 {0}")]
    CommandOutputUtf8Error(#[from] FromUtf8Error),
}
