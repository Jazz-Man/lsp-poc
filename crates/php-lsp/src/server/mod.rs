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
pub mod benchmarks;

use async_lsp::{ResponseError, ClientSocket};
use async_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializedParams, InitializeParams, InitializeResult,
};
use async_lsp::lsp_types::request::Shutdown;
use async_lsp::lsp_types::notification::Exit;
use futures::future::BoxFuture;
use std::ops::ControlFlow;

use crate::server::types::{LspServerState, create_server_state};
use crate::server::lifecycle::{handle_initialize, handle_initialized, handle_shutdown, handle_exit};
use crate::server::document_sync::{handle_did_open, handle_did_change, handle_did_close};
use crate::server::parsing::parse_and_cache_document;

/// Main LSP server struct that implements the async-lsp handlers
pub struct LspServer {
    pub state: LspServerState,
    pub client: ClientSocket,
}

impl LspServer {
    /// Create a new instance of the LSP server
    pub fn new(client: ClientSocket) -> Self {
        Self {
            state: create_server_state(),
            client,
        }
    }
}

// Define the error type for our LSP server
pub type LspServerError = ResponseError;

impl async_lsp::LanguageServer for LspServer {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        let state = self.state.clone();
        Box::pin(async move {
            handle_initialize(&state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INTERNAL_ERROR, e.to_string()))
        })
    }

    fn initialized(
        &mut self,
        params: InitializedParams,
    ) -> Self::NotifyResult {
        let state = self.state.clone();
        let _ = futures::executor::block_on(async {
            handle_initialized(&state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INVALID_REQUEST, e.to_string()))
        });
        ControlFlow::Continue(())
    }

    fn shutdown(
        &mut self,
        _params: <Shutdown as async_lsp::lsp_types::request::Request>::Params,
    ) -> BoxFuture<'static, Result<<Shutdown as async_lsp::lsp_types::request::Request>::Result, Self::Error>> {
        let state = self.state.clone();
        Box::pin(async move {
            handle_shutdown(&state).await
                .map(|_| ())
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INTERNAL_ERROR, e.to_string()))
        })
    }

    fn exit(
        &mut self,
        _params: <Exit as async_lsp::lsp_types::notification::Notification>::Params,
    ) -> Self::NotifyResult {
        let state = self.state.clone();
        let _ = futures::executor::block_on(async {
            handle_exit(&state).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INVALID_REQUEST, e.to_string()))
        });
        ControlFlow::Continue(())
    }

    fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> Self::NotifyResult {
        let state = self.state.clone();
        let uri = params.text_document.uri.clone();

        let _ = futures::executor::block_on(async {
            handle_did_open(&state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INVALID_REQUEST, e.to_string()))?;

            // After opening, we should parse the document
            let _ = parse_and_cache_document(&state, &uri).await;

            Result::<(), ResponseError>::Ok(())
        });
        ControlFlow::Continue(())
    }

    fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> Self::NotifyResult {
        let state = self.state.clone();
        let uri = params.text_document.uri.clone();

        let _ = futures::executor::block_on(async {
            handle_did_change(&state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INVALID_REQUEST, e.to_string()))?;

            // After changes, we should reparse the document
            let _ = parse_and_cache_document(&state, &uri).await;

            Result::<(), ResponseError>::Ok(())
        });
        ControlFlow::Continue(())
    }

    fn did_close(
        &mut self,
        params: DidCloseTextDocumentParams,
    ) -> Self::NotifyResult {
        let state = self.state.clone();

        let _ = futures::executor::block_on(async {
            handle_did_close(&state, params).await
                .map_err(|e| ResponseError::new(async_lsp::ErrorCode::INVALID_REQUEST, e.to_string()))
        });
        ControlFlow::Continue(())
    }
}

/// Run the LSP server
pub async fn run() -> anyhow::Result<()> {
    use async_lsp::router::Router;
    use async_lsp::server::LifecycleLayer;
    use async_lsp::tracing::TracingLayer;
    use tower::ServiceBuilder;

    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .service(Router::from_language_server(LspServer::new(client)))
    });

    // Prefer truly asynchronous piped stdin/stdout without blocking tasks.
    #[cfg(unix)]
    let (stdin, stdout) = (
        async_lsp::stdio::PipeStdin::lock_tokio()?,
        async_lsp::stdio::PipeStdout::lock_tokio()?,
    );
    // Fallback to spawn blocking read/write otherwise.
    #[cfg(not(unix))]
    let (stdin, stdout) = (
        tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
        tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
    );

    server.run_buffered(stdin, stdout).await?;
    Ok(())
}