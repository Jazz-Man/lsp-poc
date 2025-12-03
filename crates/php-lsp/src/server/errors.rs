//! Custom Error Types
//! 
//! This module defines custom error types for the PHP LSP server using the thiserror crate.

use thiserror::Error;

/// Errors that can occur in the PHP LSP server
#[derive(Error, Debug)]
pub enum LspError {
    /// Error occurred during document parsing
    #[error("Document parsing failed: {0}")]
    ParseError(String),
    
    /// Error occurred during document operation
    #[error("Document operation failed: {0}")]
    DocumentError(String),
    
    /// Error occurred during transport operation
    #[error("Transport error: {0}")]
    TransportError(String),
    
    /// Error occurred during initialization
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    /// Error occurred due to invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    /// Generic error that doesn't fit other categories
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type that uses our custom error
pub type Result<T> = std::result::Result<T, LspError>;