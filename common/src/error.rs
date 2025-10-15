//! Error types for TSLM operations.

use thiserror::Error;

/// Application-level error type.
///
/// Represents various error conditions that can occur in the TSLM system.
#[derive(Debug, Error)]
pub enum AppError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Channel send error
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Lock poisoned error
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// Channel not found
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    /// Endpoint not found
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Message too large
    #[error("Message too large: {size} bytes (max: {max} bytes)")]
    MessageTooLarge { size: usize, max: usize },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Connection limit exceeded
    #[error("Connection limit exceeded: {0}")]
    ConnectionLimitExceeded(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

impl AppError {
    /// Create a generic error from a string message.
    pub fn msg(msg: String) -> Self {
        AppError::Generic(msg)
    }

    /// Create a generic error from a string slice.
    pub fn msg_str(msg: &str) -> Self {
        AppError::Generic(msg.to_string())
    }

    /// Convert any error type into an AppError.
    pub fn from<E: std::error::Error>(error: E) -> Self {
        AppError::Generic(error.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        AppError::LockPoisoned(err.to_string())
    }
}
