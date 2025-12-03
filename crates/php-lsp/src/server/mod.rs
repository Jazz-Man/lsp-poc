//! Main Server Module
//!
//! This module orchestrates all the components of the LSP server,
//! including lifecycle management, document synchronization, parsing,
//! and transport handling.

pub mod errors;
pub mod types;
pub mod lifecycle;
pub mod document_sync;
pub mod parsing;
pub mod transport;
pub mod benchmarks;

use async_lsp::{LspService, ResponseError};
use async_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializedParams, InitializeParams, InitializeResult,
};
use serde_json::Value;

use crate::server::types::{LspServerState, create_server_state};
use crate::server::lifecycle::{handle_initialize, handle_initialized, handle_shutdown, handle_exit};
use crate::server::document_sync::{handle_did_open, handle_did_change, handle_did_close};
use crate::server::parsing::parse_and_cache_document;
use crate::server::transport::run_stdio_transport;

/// Main LSP server struct that implements the async-lsp handlers
pub struct LspServer {
    pub state: LspServerState,
}

impl LspServer {
    /// Create a new instance of the LSP server
    pub fn new() -> Self {
        Self {
            state: create_server_state(),
        }
    }

    /// Run the LSP server
    pub async fn run(&self) -> anyhow::Result<()> {
        // Create the LSP service with proper handler methods
        let service = LspService::new(|client| {
            Self {
                state: create_server_state(),
            }
        });

        // Run the server with stdio transport
        run_stdio_transport(service).await?;

        Ok(())
    }
}

// Define the error type for our LSP server
pub type LspServerError = ResponseError;

impl async_lsp::LanguageServer for LspServer {
    type Error = LspServerError;
    type NotifyResult = async_lsp::Result<()>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> Pin<Box<dyn Future<Output = Result<InitializeResult, Self::Error>> + Send>> {
        Box::pin(async move {
            handle_initialize(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        })
    }

    fn initialized(
        &mut self,
        params: InitializedParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>> {
        Box::pin(async move {
            handle_initialized(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        })
    }

    fn shutdown(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Value, Self::Error>> + Send>> {
        Box::pin(async move {
            handle_shutdown(&self.state).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        })
    }

    fn exit(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>> {
        Box::pin(async move {
            handle_exit(&self.state).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        })
    }

    fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>> {
        Box::pin(async move {
            handle_did_open(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))?;

            // After opening, we should parse the document
            let _ = parse_and_cache_document(&self.state, &params.text_document.uri).await;

            Ok(())
        })
    }

    fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>> {
        Box::pin(async move {
            handle_did_change(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))?;

            // After changes, we should reparse the document
            let uri = &params.text_document.uri;
            let _ = parse_and_cache_document(&self.state, uri).await;

            Ok(())
        })
    }

    fn did_close(
        &mut self,
        params: DidCloseTextDocumentParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>> {
        Box::pin(async move {
            handle_did_close(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        })
    }
}

/// Run the LSP server
pub async fn run() -> anyhow::Result<()> {
    let server = LspServer::new();
    let service = LspService::new(server, |client| {
        // This closure allows for custom client handling if needed
    });
    run_stdio_transport(service).await
}