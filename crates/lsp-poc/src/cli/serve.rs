use anyhow::{Context, Result};
use clap::Parser;

use async_language_server::server::{Transport, serve};

use crate::server::PocLanguageServer;

#[derive(Debug, Clone, Parser)]
pub struct ServeCommand {}

impl ServeCommand {
    pub async fn run(self) -> Result<()> {
        let server = PocLanguageServer::new();

        serve(Transport::Stdio, server)
            .await
            .context("encountered fatal error - language server shutting down")
    }
}
