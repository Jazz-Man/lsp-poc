//! Zed extension launcher for the lsp-poc language server.

use std::path::Path;
use zed_extension_api::{self as zed, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LspPocExtension;

impl zed::Extension for LspPocExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let bin = Path::new(worktree.root_path().as_str())
            .join("target")
            .join("debug")
            .join("lsp-poc")
            .to_string_lossy()
            .to_string();

        Ok(zed::Command::new(bin)
            .args(["serve", "--stdio"])
            .envs(worktree.shell_env()))
    }
}

zed::register_extension!(LspPocExtension);
