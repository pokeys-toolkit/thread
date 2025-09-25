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

    #[error("Pin capability error: {message}")]
    PinCapabilityError {
        message: String,
        pin: u8,
        capability: String,
        suggestion: Option<String>,
    },

    #[error("Hardware constraint violation: {message}")]
    HardwareConstraint {
        message: String,
        constraint: String,
        suggestion: String,
    },

    #[error("Validation failed: {message}")]
    ValidationError {
        message: String,
        context: String,
        recovery_suggestion: Option<String>,
    },

    #[error("Resource conflict: {message}")]
    ResourceConflict {
        message: String,
        resource: String,
        conflicting_operation: String,
    },

    #[error("Other error: {0}")]
    Other(String),
}

impl ThreadError {
    /// Create a pin capability error with context
    pub fn pin_capability_error(pin: u8, capability: &str, suggestion: Option<String>) -> Self {
        Self::PinCapabilityError {
            message: format!("Pin {} does not support {}", pin, capability),
            pin,
            capability: capability.to_string(),
            suggestion,
        }
    }

    /// Create a hardware constraint error
    pub fn hardware_constraint(constraint: &str, suggestion: &str) -> Self {
        Self::HardwareConstraint {
            message: format!("Hardware constraint violated: {}", constraint),
            constraint: constraint.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    /// Create a validation error with recovery suggestion
    pub fn validation_error(message: &str, context: &str, recovery: Option<&str>) -> Self {
        Self::ValidationError {
            message: message.to_string(),
            context: context.to_string(),
            recovery_suggestion: recovery.map(|s| s.to_string()),
        }
    }

    /// Create a resource conflict error
    pub fn resource_conflict(resource: &str, operation: &str) -> Self {
        Self::ResourceConflict {
            message: format!("Resource {} is already in use", resource),
            resource: resource.to_string(),
            conflicting_operation: operation.to_string(),
        }
    }

    /// Get recovery suggestion if available
    pub fn recovery_suggestion(&self) -> Option<&str> {
        match self {
            Self::PinCapabilityError { suggestion, .. } => suggestion.as_deref(),
            Self::HardwareConstraint { suggestion, .. } => Some(suggestion),
            Self::ValidationError {
                recovery_suggestion,
                ..
            } => recovery_suggestion.as_deref(),
            _ => None,
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::PinCapabilityError { .. }
                | Self::ValidationError { .. }
                | Self::InvalidParameter(_)
                | Self::ConfigurationError(_)
        )
    }
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
