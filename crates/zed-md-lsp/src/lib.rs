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

// zed's register_extension! expands to a pub extern "C" `init-extension`
// guest export (plus wasi-gated glue). The private module keeps that
// macro-generated item out of this crate's public API surface — the docs
// gate then has nothing to fire on — while the linker-level export_name is
// unaffected by the Rust-side module path.
mod register {
    use super::LspPocExtension;

    super::zed::register_extension!(LspPocExtension);
}
