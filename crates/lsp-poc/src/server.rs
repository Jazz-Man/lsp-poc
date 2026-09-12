use std::future::{Future, ready};

use async_language_server::{
    lsp_types::{
        ClientCapabilities, Hover, HoverContents, HoverParams, HoverProviderCapability,
        MarkupContent, MarkupKind, ServerCapabilities, ServerInfo,
    },
    server::{DocumentMatcher, Server, ServerResult, ServerState},
    tree_sitter_utils::{ts_range_contains_lsp_position, ts_range_to_lsp_range},
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
            name: env!("CARGO_PKG_NAME").to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
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
            DocumentMatcher::new("Markdown")
                .with_url_globs(["**/*.md"])
                .with_lang_strings(["Markdown"])
                .with_lang_grammar(tree_sitter_md::LANGUAGE.into()),
        ]
    }

    fn hover(
        &self,
        state: ServerState,
        params: HoverParams,
    ) -> impl Future<Output = ServerResult<Option<Hover>>> + Send {
        ready(hover(&state, params))
    }
}

fn hover(state: &ServerState, params: HoverParams) -> ServerResult<Option<Hover>> {
    let url = params.text_document_position_params.text_document.uri;
    let pos = params.text_document_position_params.position;

    let Some(doc) = state.document(&url) else {
        return Ok(None);
    };

    let Some(node) = doc.node_at_position_named(pos) else {
        tracing::debug!("Missing node for hover at {}:{}", pos.line, pos.character);
        return Ok(None);
    };

    if !ts_range_contains_lsp_position(node.range(), pos) {
        return Ok(None);
    }

    tracing::debug!("Getting hover for node at {}:{}", pos.line, pos.character);

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "```json\n".to_owned() + &node.to_string() + "\n```",
        }),
        range: Some(ts_range_to_lsp_range(node.range())),
    }))
}
