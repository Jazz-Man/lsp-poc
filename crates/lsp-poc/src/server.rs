use crate::links::{self, Target};
use crate::references::{self, DocumentSnapshot};
use crate::workspace::{Index, Resolved};
use crate::{definitions, diagnostics};
use async_language_server::lsp_types::notification::PublishDiagnostics;
use async_language_server::lsp_types::{
    ClientCapabilities, Diagnostic, DiagnosticOptions, DiagnosticServerCapabilities,
    DidChangeTextDocumentParams, DidChangeWorkspaceFoldersParams, DidOpenTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, GotoDefinitionParams, GotoDefinitionResponse, Hover,
    HoverContents, HoverParams, HoverProviderCapability, MarkupContent, MarkupKind, OneOf,
    PublishDiagnosticsParams, ReferenceParams, RelatedFullDocumentDiagnosticReport,
    ServerCapabilities, ServerInfo, Url,
};
use async_language_server::server::{DocumentMatcher, Server, ServerResult, ServerState};
use async_language_server::tree_sitter_utils::{
    ts_range_contains_lsp_position, ts_range_to_lsp_range,
};
use std::future::{Future, ready};
use std::sync::{Arc, Mutex, PoisonError};
use tree_sitter_md::MarkdownParser;

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

    /// The target resolver diagnostics and definitions share: open
    /// documents win, then the workspace index resolves against disk.
    fn resolver<'s>(
        &'s self,
        state: &'s ServerState,
        url: &'s Url,
    ) -> impl Fn(&Target) -> Option<Resolved> + 's {
        move |target: &Target| {
            let open = |open_url: &Url| {
                state
                    .document(open_url)
                    .and_then(|open_doc| self.parse(&open_doc.text_contents()))
            };
            self.files.resolve(&open, url, target)
        }
    }

    /// Candidate documents for reference scans: open documents first,
    /// then the workspace index's cached files (own parse per open doc —
    /// the established POC cost).
    fn document_snapshots(&self, state: &ServerState) -> Vec<DocumentSnapshot> {
        let mut snapshots = Vec::new();
        for document in state.documents() {
            if let Some(index) = self.parse(&document.text_contents()) {
                snapshots.push(DocumentSnapshot {
                    url: document.url().clone(),
                    index,
                });
            }
        }
        for (url, index) in self.files.snapshot() {
            if !snapshots.iter().any(|snapshot| snapshot.url == url) {
                snapshots.push(DocumentSnapshot { url, index });
            }
        }
        snapshots
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
        let resolve = self.resolver(state, url);
        diagnostics::compute(&index, &resolve)
    }

    /// Answers one `textDocument/definition` request. Absence is a normal
    /// `None`: no document, no parse, or nothing definition-worthy under
    /// the position.
    fn definition_at(
        &self,
        state: &ServerState,
        params: GotoDefinitionParams,
    ) -> ServerResult<Option<GotoDefinitionResponse>> {
        let url = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(doc) = state.document(&url) else {
            return Ok(None);
        };
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no definition for {url}");
            return Ok(None);
        };
        let resolve = self.resolver(state, &url);
        Ok(definitions::at_position(&index, &url, position, &resolve))
    }

    /// Answers one `textDocument/references` request. Absence is a normal
    /// `None`: no document, no parse, nothing reference-worthy under the
    /// position, or zero matches.
    fn references_at(
        &self,
        state: &ServerState,
        params: &ReferenceParams,
    ) -> ServerResult<Option<Vec<async_language_server::lsp_types::Location>>> {
        let url = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(doc) = state.document(url) else {
            return Ok(None);
        };
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no references for {url}");
            return Ok(None);
        };
        let documents = self.document_snapshots(state);
        let resolve = self.resolver(state, url);
        let files = &self.files;
        let resolve_from = |candidate_url: &Url, target: &Target| {
            let open = |open_url: &Url| {
                state
                    .document(open_url)
                    .and_then(|open_doc| self.parse(&open_doc.text_contents()))
            };
            files.resolve(&open, candidate_url, target)
        };
        Ok(references::at_position(
            &index,
            url,
            position,
            &documents,
            &resolve,
            &resolve_from,
        ))
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
            name: crate::info::server_name().to_owned(),
            version: Some(crate::info::server_version().to_owned()),
        })
    }

    fn server_capabilities(_: ClientCapabilities) -> Option<ServerCapabilities> {
        Some(ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
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

    fn definition(
        &self,
        state: ServerState,
        params: GotoDefinitionParams,
    ) -> impl Future<Output = ServerResult<Option<GotoDefinitionResponse>>> + Send {
        ready(self.definition_at(&state, params))
    }

    fn references(
        &self,
        state: ServerState,
        params: ReferenceParams,
    ) -> impl Future<Output = ServerResult<Option<Vec<async_language_server::lsp_types::Location>>>> + Send
    {
        ready(self.references_at(&state, &params))
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
