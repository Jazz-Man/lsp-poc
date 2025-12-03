//! Main entry point for the PHP Language Server Protocol implementation
//!
//! This binary initializes the LSP server and handles command-line arguments
//! to determine the communication transport mode.

use std::env;
use php_lsp::server::run_server;

/// Main entry point for the PHP LSP server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    // Determine if the server should run in stdio mode
    let use_stdio = args.iter().any(|arg| arg == "--stdio");

    if use_stdio {
        tracing::info!("Starting PHP LSP server in stdio mode");
        run_server().await?;
    } else {
        eprintln!("Usage: {} --stdio", args[0]);
        std::process::exit(1);
    }

    Ok(())
}