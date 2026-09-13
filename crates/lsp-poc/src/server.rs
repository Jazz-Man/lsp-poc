use std::future::{Future, ready};
use std::sync::{Arc, Mutex, PoisonError};

use async_language_server::lsp_types::notification::PublishDiagnostics;
use async_language_server::lsp_types::{
    ClientCapabilities, Diagnostic, DiagnosticOptions, DiagnosticServerCapabilities,
    DidChangeTextDocumentParams, DidChangeWorkspaceFoldersParams, DidOpenTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, Hover, HoverContents, HoverParams, HoverProviderCapability,
    MarkupContent, MarkupKind, PublishDiagnosticsParams, RelatedFullDocumentDiagnosticReport,
    ServerCapabilities, ServerInfo, Url,
};
use async_language_server::server::{DocumentMatcher, Server, ServerResult, ServerState};
use async_language_server::tree_sitter_utils::{
    ts_range_contains_lsp_position, ts_range_to_lsp_range,
};
use tree_sitter_md::MarkdownParser;

use crate::diagnostics;
use crate::links::{self, Target};
use crate::workspace::Index;

#[derive(Clone)]
pub struct PocLanguageServer {
    parser: Arc<Mutex<MarkdownParser>>,
    files: Arc<Index>,
}

impl PocLanguageServer {
    pub fn new() -> Self {
        Self {
            parser: Arc::new(Mutex::new(MarkdownParser::default())),
            files: Arc::new(Index::new()),
        }
    }

    fn parse(&self, text: &str) -> Option<Arc<links::MdIndex>> {
        let mut parser = self.parser.lock().unwrap_or_else(PoisonError::into_inner);
        links::build(&mut parser, text).map(Arc::new)
    }

    fn compute_diagnostics(&self, state: &ServerState, url: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.document(url) else {
            return Vec::new();
        };
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::warn!("markdown parse produced no tree; skipping diagnostics for {url}");
            return Vec::new();
        };
        let open = |open_url: &Url| {
            state
                .document(open_url)
                .and_then(|open_doc| self.parse(&open_doc.text_contents()))
        };
        diagnostics::compute(&index, &|target: &Target| {
            self.files.resolve(&open, url, target)
        })
    }

    fn publish(&self, state: &ServerState, url: &Url) {
        let params = PublishDiagnosticsParams {
            uri: url.clone(),
            diagnostics: self.compute_diagnostics(state, url),
            version: None,
        };
        if let Err(error) = state.client().notify::<PublishDiagnostics>(params) {
            tracing::warn!(%error, "failed to publish diagnostics");
        }
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
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                workspace_diagnostics: false,
                ..DiagnosticOptions::default()
            })),
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

    fn did_open(&self, state: &ServerState, params: &DidOpenTextDocumentParams) {
        self.publish(state, &params.text_document.uri);
    }

    fn did_change(&self, state: &ServerState, params: &DidChangeTextDocumentParams) {
        self.publish(state, &params.text_document.uri);
    }

    fn did_change_workspace_folders(
        &self,
        _state: &ServerState,
        _params: &DidChangeWorkspaceFoldersParams,
    ) {
        self.files.reset_root();
    }

    fn document_diagnostics(
        &self,
        state: ServerState,
        params: DocumentDiagnosticParams,
    ) -> impl Future<Output = ServerResult<DocumentDiagnosticReportResult>> + Send {
        let url = params.text_document.uri;
        let diagnostics = self.compute_diagnostics(&state, &url);
        ready(Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: diagnostics,
                },
            }),
        )))
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
