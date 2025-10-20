use std::ffi::NulError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SednaError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Sedna connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Sedna query failed: {0}")]
    QueryFailed(String),

    #[error("Sedna transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Server startup failed: {0}")]
    ServerStartupFailed(String),

    #[error("Server shutdown failed: {0}")]
    ServerShutdownFailed(String),

    #[error("Binary extraction failed: {0}")]
    BinaryExtractionFailed(String),

    #[error("Database creation failed: {0}")]
    DatabaseCreationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid C string: {0}")]
    InvalidCString(#[from] NulError),

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Sedna error: {0}")]
    SednaInternal(String),

    #[error("No more items")]
    NoMoreItems,

    #[error("Result end")]
    ResultEnd,
}

pub type Result<T> = std::result::Result<T, SednaError>;
