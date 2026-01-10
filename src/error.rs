//! Error types for OPC DA Client operations

use crate::types::OpcValueError;

/// Result type for OPC operations
pub type OpcResult<T> = Result<T, OpcError>;

/// Error types for OPC operations
#[derive(Debug, thiserror::Error)]
pub enum OpcError {
    /// General OPC operation error
    #[error("OPC operation failed: {0}")]
    OperationFailed(String),
    
    /// Connection related error
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// Invalid parameters passed to function
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    /// Value conversion error
    #[error("Value conversion error: {0}")]
    ValueConversionError(#[from] OpcValueError),
    
    /// COM initialization error
    #[error("COM initialization failed: {0}")]
    ComInitializationFailed(String),
    
    /// Server not found error
    #[error("Server not found: {0}")]
    ServerNotFound(String),
    
    /// Item not found error
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    /// Group creation error
    #[error("Failed to create group: {0}")]
    GroupCreationFailed(String),
    
    /// Async subscription error
    #[error("Failed to enable async subscription: {0}")]
    AsyncSubscriptionFailed(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl OpcError {
    /// Create a new operation failed error
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        OpcError::OperationFailed(msg.into())
    }
    
    /// Create a new connection failed error
    pub fn connection_failed(msg: impl Into<String>) -> Self {
        OpcError::ConnectionFailed(msg.into())
    }
    
    /// Create a new invalid parameters error
    pub fn invalid_parameters(msg: impl Into<String>) -> Self {
        OpcError::InvalidParameters(msg.into())
    }
}