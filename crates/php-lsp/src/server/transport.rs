//! Transport Layer
//!
//! This module handles the communication transport for the LSP server,
//! specifically stdio communication for editor integration.

use async_lsp::{LspService, Response, Server};
use tokio::sync::Mutex;

use crate::server::LspServerState;

/// Run the LSP server with stdio transport
pub async fn run_stdio_transport(
    mut service: LspService<Mutex<LspServerState>>,
) -> anyhow::Result<()> {
    tracing::info!("Starting LSP server with stdio transport");

    // Create a new LSP server instance with stdio as transport
    let (connection, io_handler) = async_lsp::stdio::stdio_transport();

    // Create the server instance
    let server = Server::new(connection);

    // Run the server
    server.run(service, io_handler).await?;

    tracing::info!("LSP server shutting down");
    Ok(())
}