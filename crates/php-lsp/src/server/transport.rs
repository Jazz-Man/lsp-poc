//! Transport Layer
//!
//! This module handles the communication transport for the LSP server,
//! specifically stdio communication for editor integration.

use async_lsp::LspService;
use tokio::time::timeout;

use crate::server::LspServer;

/// Run the LSP server with stdio transport
pub async fn run_stdio_transport(
    service: LspService<crate::server::LspServer>,
) -> anyhow::Result<()> {
    tracing::info!("Starting LSP server with stdio transport");

    // Run the LSP service with stdio transport
    async_lsp::stdio::run(service).await?;

    tracing::info!("LSP server shutting down");
    Ok(())
}