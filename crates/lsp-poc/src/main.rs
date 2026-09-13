//! Proof-of-concept Markdown language server over the LSP protocol, built on
//! the owner's `async-language-server` fork and served over stdio.

mod hovers;
mod links;
mod server;
mod tracing;
mod workspace;

use self::tracing::setup_tracing;
use crate::server::PocLanguageServer;
use anyhow::Context;
use async_language_server::server::serve;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    serve(PocLanguageServer::new())
        .await
        .context("encountered fatal error - language server shutting down")
}
