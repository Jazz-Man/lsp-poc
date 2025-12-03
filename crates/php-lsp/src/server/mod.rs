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
}

// Define the error type for our LSP server
pub type LspServerError = ResponseError;

impl async_lsp::LanguageServer for LspServer {
    type Error = LspServerError;
    type NotifyResult = std::result::Result<(), Self::Error>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> async_lsp::Result<InitializeResult> {
        // Convert to async block to call async function
        use futures::FutureExt;
        async move {
            handle_initialize(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        }.boxed()
    }

    fn initialized(
        &mut self,
        params: InitializedParams,
    ) -> async_lsp::Result<()> {
        use futures::FutureExt;
        async move {
            handle_initialized(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        }.boxed()
    }

    fn shutdown(
        &mut self,
    ) -> async_lsp::Result<Value> {
        use futures::FutureExt;
        async move {
            handle_shutdown(&self.state).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        }.boxed()
    }

    fn exit(
        &mut self,
    ) -> std::result::Result<(), async_lsp::Error> {
        use futures::FutureExt;
        async move {
            handle_exit(&self.state).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        }.boxed().into()
    }

    fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> async_lsp::Result<()> {
        use futures::FutureExt;
        async move {
            handle_did_open(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))?;

            // After opening, we should parse the document
            let _ = parse_and_cache_document(&self.state, &params.text_document.uri).await;

            Ok(())
        }.boxed()
    }

    fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> async_lsp::Result<()> {
        use futures::FutureExt;
        async move {
            handle_did_change(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))?;

            // After changes, we should reparse the document
            let uri = &params.text_document.uri;
            let _ = parse_and_cache_document(&self.state, uri).await;

            Ok(())
        }.boxed()
    }

    fn did_close(
        &mut self,
        params: DidCloseTextDocumentParams,
    ) -> async_lsp::Result<()> {
        use futures::FutureExt;
        async move {
            handle_did_close(&self.state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::InternalError, e.to_string()))
        }.boxed()
    }
}

/// Run the LSP server
pub async fn run() -> anyhow::Result<()> {
    async_lsp::start_server(
        |client| LspService::new(LspServer::new(), client),
        |socket| async { async_lsp::ServerSocket::stdio() }.boxed()
    ).await?;
    Ok(())
}