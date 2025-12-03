use zed_extension_api::{self as zed, Result};

struct PhpLspExtension;

impl zed::Extension for PhpLspExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _config: zed::LanguageServerConfig,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: "./php-lsp".to_string(),
            args: vec!["--stdio".to_string()],
            env: Default::default(),
        })
    }
}

impl zed::ReloadableExtension for PhpLspExtension {
    fn reload(&mut self) -> Result<()> {
        Ok(())
    }
}

zed::register_extension!(PhpLspExtension);