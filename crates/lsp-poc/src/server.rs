use async_language_server::{
    lsp_types::{
        ClientCapabilities, CompletionOptions, HoverProviderCapability, OneOf, SaveOptions,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
        TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
    },
    server::{DocumentMatcher, Server},
};

use crate::completions::completion_trigger_characters;

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
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(false),
                    })),
                },
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                all_commit_characters: None,
                trigger_characters: Some(completion_trigger_characters()),
                work_done_progress_options: Default::default(),
                ..Default::default()
            }),
            document_symbol_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_range_formatting_provider: Some(OneOf::Left(true)),

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
