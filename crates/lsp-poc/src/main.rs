//! Proof-of-concept Markdown language server over the LSP protocol, built on
//! the owner's `async-language-server` fork and served over stdio.

mod cli;
mod hovers;
mod server;
mod tracing;

use self::tracing::setup_tracing;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    cli::Cli::new().run().await
}
