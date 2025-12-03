//! Transport Layer
//!
//! This module handles the communication transport for the LSP server,
//! specifically stdio communication for editor integration.

use async_lsp::LspService;
use futures::future::BoxFuture;
use futures::FutureExt;

/// Run the LSP server with stdio transport
pub async fn run_stdio_transport<T>(
    service: LspService<T>,
) -> anyhow::Result<()>
where
    T: async_lsp::LanguageServer,
{
    tracing::info!("Starting LSP server with stdio transport");

    // Run the LSP service with stdio transport using the correct method
    async_lsp::start_server(
        service,
        |socket| async { async_lsp::ServerSocket::stdio() }.boxed()
    ).await?;

    tracing::info!("LSP server shutting down");
    Ok(())
}