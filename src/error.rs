//! Error types for the pokeys-thread crate

use std::sync::PoisonError;
use thiserror::Error;

/// Result type used throughout the crate
pub type Result<T> = std::result::Result<T, ThreadError>;

/// Main error type for threading operations
#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Thread not found: {0}")]
    ThreadNotFound(u32),

    #[error("Thread already exists: {0}")]
    ThreadAlreadyExists(u32),

    #[error("Thread creation failed: {0}")]
    ThreadCreationFailed(String),

    #[error("Command send failed: {0}")]
    CommandSendFailed(String),

    #[error("Device error: {0}")]
    DeviceError(#[from] pokeys_lib::PoKeysError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Thread join error")]
    ThreadJoinError,

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Operation timeout")]
    Timeout,

    #[error("Operation not supported")]
    NotSupported,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(String),

    #[error("Channel send error: {0}")]
    ChannelSendError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Initialization error: {0}")]
    InitializationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Invalid thread ID: {0}")]
    InvalidThreadId(usize),

    #[error("Unsupported device type")]
    UnsupportedDevice,

    #[error("Other error: {0}")]
    Other(String),
}

// Implement From for PoisonError
impl<T> From<PoisonError<T>> for ThreadError {
    fn from(err: PoisonError<T>) -> Self {
        ThreadError::LockPoisoned(err.to_string())
    }
}

// Implement From for channel errors
impl<T> From<std::sync::mpsc::SendError<T>> for ThreadError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        ThreadError::ChannelSendError(err.to_string())
    }
}

impl From<std::sync::mpsc::RecvError> for ThreadError {
    fn from(err: std::sync::mpsc::RecvError) -> Self {
        ThreadError::ChannelReceiveError(err.to_string())
    }
}

impl From<std::sync::mpsc::TryRecvError> for ThreadError {
    fn from(err: std::sync::mpsc::TryRecvError) -> Self {
        ThreadError::ChannelReceiveError(err.to_string())
    }
}
