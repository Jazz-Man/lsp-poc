//! PHP Language Server Protocol implementation
//!
//! This crate provides a complete LSP server for PHP, implementing the
//! Language Server Protocol specification with focus on PHP-specific features.

pub mod server;

use anyhow::Result;

/// Runs the PHP LSP server using stdio for communication
pub async fn run_server() -> Result<()> {
    server::run().await
}