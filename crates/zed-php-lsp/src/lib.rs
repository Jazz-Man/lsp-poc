//! Zed extension for PHP LSP

use zed_extension_api::{self as zed, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LspPocExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for LspPocExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // let binary_path = worktree
        //     .which("lsp-poc")
        //     .ok_or_else(|| "lsp-poc not found in PATH".to_string())?;

        let bin_file = "/Users/vasilsokolik/www/lsp-poc/target/debug/lsp-poc".to_string();

        Ok(zed::Command {
            command: bin_file,
            args: vec!["--stdio".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(LspPocExtension);
