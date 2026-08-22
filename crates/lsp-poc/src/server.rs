use async_language_server::{
    lsp_types::{
        ClientCapabilities,
        HoverProviderCapability, ServerCapabilities, ServerInfo,
    },
    server::{DocumentMatcher, Server},
};

#[derive(Debug, Clone)]
pub struct PocLanguageServer {}

impl PocLanguageServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PocLanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Server for PocLanguageServer {
    fn server_info() -> Option<ServerInfo> {
        Some(ServerInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        })
    }

    fn server_capabilities(_: ClientCapabilities) -> Option<ServerCapabilities> {
        Some(ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),

            ..Default::default()
        })
    }

    fn server_document_matchers() -> Vec<DocumentMatcher> {
        vec![
            DocumentMatcher::new("Zap Document")
                .with_url_globs(["*.zap"])
                .with_lang_strings(["Zap"]),
        ]
    }
}
