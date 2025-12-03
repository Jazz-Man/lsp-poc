//! Transport Layer
//!
//! This module handles the communication transport for the LSP server,
//! specifically stdio communication for editor integration.

use async_lsp::{LspService, ServerSocket};
use tokio::time::timeout;
use std::future::Future;
use std::pin::Pin;

use crate::server::LspServer;

/// Run the LSP server with stdio transport
pub async fn run_stdio_transport<T>(
    mut service: LspService<T>,
) -> anyhow::Result<()>
where
    T: async_lsp::LanguageServer,
{
    tracing::info!("Starting LSP server with stdio transport");

    // Run the LSP service with stdio transport
    let socket = ServerSocket::stdio();
    service.serve(socket).await?;

    tracing::info!("LSP server shutting down");
    Ok(())
}