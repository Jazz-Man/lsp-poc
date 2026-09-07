use anyhow::{Context, Result};
use clap::Parser;
// use tracing::debug;

use async_language_server::server::serve;

use crate::server::PocLanguageServer;

#[derive(Debug, Clone, Parser)]
pub struct ServeCommand {
    #[arg(long, alias = "port")]
    pub socket: Option<u16>,
    #[arg(long)]
    pub stdio: bool,
}

impl ServeCommand {
    pub async fn run(self) -> Result<()> {
        let server = PocLanguageServer::new();

        serve(server)
            .await
            .context("encountered fatal error - language server shutting down")
    }
}
