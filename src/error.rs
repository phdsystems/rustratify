//! Error types for Rustratify framework.

use thiserror::Error;

/// Root error type for Rustratify operations.
#[derive(Error, Debug)]
pub enum RustratifyError {
    /// Provider-related errors
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Registry-related errors
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Stream-related errors
    #[error("Stream error: {0}")]
    Stream(String),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

/// Errors that can occur in provider operations.
#[derive(Error, Debug, Clone)]
pub enum ProviderError {
    /// Provider not found for the given key
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// Provider does not support the given input
    #[error("Provider does not support: {0}")]
    NotSupported(String),

    /// Provider execution failed
    #[error("Provider execution failed: {0}")]
    ExecutionFailed(String),

    /// Provider initialization failed
    #[error("Provider initialization failed: {0}")]
    InitializationFailed(String),

    /// Provider configuration error
    #[error("Provider configuration error: {0}")]
    ConfigurationError(String),

    /// IO error during provider operation
    #[error("IO error: {0}")]
    IoError(String),

    /// Timeout during provider operation
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Provider was cancelled
    #[error("Operation was cancelled")]
    Cancelled,
}

/// Errors that can occur in registry operations.
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Provider already registered with this name
    #[error("Provider already registered: {0}")]
    AlreadyRegistered(String),

    /// No provider found matching criteria
    #[error("No matching provider found")]
    NoMatchingProvider,

    /// Registry is empty
    #[error("Registry is empty")]
    Empty,

    /// Invalid provider name
    #[error("Invalid provider name: {0}")]
    InvalidName(String),
}

impl From<std::io::Error> for ProviderError {
    fn from(err: std::io::Error) -> Self {
        ProviderError::IoError(err.to_string())
    }
}

impl From<String> for ProviderError {
    fn from(msg: String) -> Self {
        ProviderError::ExecutionFailed(msg)
    }
}

impl From<&str> for ProviderError {
    fn from(msg: &str) -> Self {
        ProviderError::ExecutionFailed(msg.to_string())
    }
}

/// Result type alias for provider operations.
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Result type alias for registry operations.
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Result type alias for general Rustratify operations.
pub type RustratifyResult<T> = Result<T, RustratifyError>;
