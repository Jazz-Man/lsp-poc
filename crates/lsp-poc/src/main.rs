mod cli;
mod server;
mod tracing;
mod utils;

use self::tracing::setup_tracing;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    println!("PHP LSP Server - Work in Progress TEST");
    println!("Run with --stdio for LSP mode");

    cli::Cli::new().run().await
}
