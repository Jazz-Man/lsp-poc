//! Zed extension for PHP LSP

use zed_extension_api::{self as zed, Result};

struct PhpLspExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for PhpLspExtension {
    fn new() -> Self {
        Self { cached_binary_path: None }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = worktree
            .which("php-lsp")
            .ok_or_else(|| "php-lsp not found in PATH".to_string())?;

        Ok(zed::Command {
            command: binary_path,
            args: vec!["--stdio".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(PhpLspExtension);
