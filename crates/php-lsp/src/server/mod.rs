//! Main Server Module
//!
//! This module orchestrates all the components of the LSP server,
//! including lifecycle management, document synchronization, parsing,
//! and transport handling.

pub mod errors;

use async_lsp::LspService;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializedParams, InitializeParams, InitializeResult,
};
use tokio::sync::Mutex;

use crate::server::types::{create_server_state, LspServerState};
use crate::server::lifecycle;
use crate::server::document_sync;
use crate::server::parsing;
use crate::server::transport;

/// Main LSP server struct that implements the async-lsp handlers
pub struct LspServer {
    state: LspServerState,
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
        // Create the LSP service
        let mut service = LspService::new(Mutex::new(self.clone()));

        // Add handlers for LSP methods
        service.add_method("initialize", Self::handle_initialize);
        service.add_notification("initialized", Self::handle_initialized);
        service.add_method("shutdown", Self::handle_shutdown);
        service.add_notification("exit", Self::handle_exit);
        service.add_notification("textDocument/didOpen", Self::handle_did_open);
        service.add_notification("textDocument/didChange", Self::handle_did_change);
        service.add_notification("textDocument/didClose", Self::handle_did_close);

        // Run the server with stdio transport
        transport::run_stdio_transport(service).await?;

        Ok(())
    }
}

/// Handler implementations for the LSP server
#[async_lsp::async_trait]
impl async_lsp::LspHandler for LspServer {
    /// Handle the initialize request
    #[allow(clippy::unused_async)]
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> async_lsp::Result<InitializeResult> {
        lifecycle::handle_initialize(&self.state, params).await
    }

    /// Handle the initialized notification
    #[allow(clippy::unused_async)]
    async fn initialized(&self, params: InitializedParams) -> async_lsp::Result<()> {
        lifecycle::handle_initialized(&self.state, params).await
    }

    /// Handle the shutdown request
    #[allow(clippy::unused_async)]
    async fn shutdown(&self) -> async_lsp::Result<()> {
        lifecycle::handle_shutdown(&self.state).await?;
        Ok(())
    }

    /// Handle the exit notification
    #[allow(clippy::unused_async)]
    async fn exit(&self) -> async_lsp::Result<()> {
        lifecycle::handle_exit(&self.state).await?;
        Ok(())
    }

    /// Handle the textDocument/didOpen notification
    #[allow(clippy::unused_async)]
    async fn did_open(&self, params: DidOpenTextDocumentParams) -> async_lsp::Result<()> {
        document_sync::handle_did_open(&self.state, params).await?;

        // After opening, we should parse the document
        let _ = parsing::parse_and_cache_document(&self.state, &params.text_document.uri).await;

        Ok(())
    }

    /// Handle the textDocument/didChange notification
    #[allow(clippy::unused_async)]
    async fn did_change(&self, params: DidChangeTextDocumentParams) -> async_lsp::Result<()> {
        document_sync::handle_did_change(&self.state, params).await?;

        // After changes, we should reparse the document
        let uri = &params.text_document.uri;
        let _ = parsing::parse_and_cache_document(&self.state, uri).await;

        Ok(())
    }

    /// Handle the textDocument/didClose notification
    #[allow(clippy::unused_async)]
    async fn did_close(&self, params: DidCloseTextDocumentParams) -> async_lsp::Result<()> {
        document_sync::handle_did_close(&self.state, params).await
    }
}

/// Run the LSP server
pub async fn run() -> anyhow::Result<()> {
    let server = LspServer::new();
    server.run().await
}