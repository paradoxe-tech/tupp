use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum TuppError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration directory not found")]
    ConfigDirNotFound,

    #[error("Interaction cancelled by user")]
    Interrupted,
    
    #[error("Data validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TuppError>;
