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
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializedParams, InitializeParams, InitializeResult,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::server::types::{LspServerState, create_server_state};
use crate::server::lifecycle::{handle_initialize, handle_initialized, handle_shutdown, handle_exit};
use crate::server::document_sync::{handle_did_open, handle_did_change, handle_did_close};
use crate::server::parsing::parse_and_cache_document;
use crate::server::transport::run_stdio_transport;

// Import error types
use crate::server::errors::{LspError, Result as LspResult};

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

impl async_lsp::Server for LspServer {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> async_lsp::Result<InitializeResult> {
        handle_initialize(&self.state, params).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))
    }

    async fn initialized(
        &self,
        params: InitializedParams,
    ) -> async_lsp::Result<()> {
        handle_initialized(&self.state, params).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))
    }

    async fn shutdown(
        &self,
    ) -> async_lsp::Result<Value> {
        handle_shutdown(&self.state).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))
    }

    async fn exit(
        &self,
    ) -> async_lsp::Result<()> {
        handle_exit(&self.state).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))
    }

    async fn did_open(
        &self,
        params: DidOpenTextDocumentParams,
    ) -> async_lsp::Result<()> {
        handle_did_open(&self.state, params).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))?;

        // After opening, we should parse the document
        let _ = parse_and_cache_document(&self.state, &params.text_document.uri).await;

        Ok(())
    }

    async fn did_change(
        &self,
        params: DidChangeTextDocumentParams,
    ) -> async_lsp::Result<()> {
        handle_did_change(&self.state, params).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))?;

        // After changes, we should reparse the document
        let uri = &params.text_document.uri;
        let _ = parse_and_cache_document(&self.state, uri).await;

        Ok(())
    }

    async fn did_close(
        &self,
        params: DidCloseTextDocumentParams,
    ) -> async_lsp::Result<()> {
        handle_did_close(&self.state, params).await
            .map_err(|e| ResponseError::new(async_lsp::ErrorCode::UnknownErrorCode, e.to_string()))
    }
}

/// Run the LSP server
pub async fn run() -> anyhow::Result<()> {
    let server = LspServer::new();
    let service = LspService::build(server, |client| {
        // This closure would define the server instance, but isn't needed with our approach
    })
    .finish();

    run_stdio_transport(service).await
}