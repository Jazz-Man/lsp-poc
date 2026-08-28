# async-language-server — documentSymbol & workspaceSymbol support (requirements handoff)

**Date:** 2026-08-28
**For:** the `async-language-server` project (`/Users/vasilsokolik/www/async-language-server`, owner's fork of the upstream framework)
**From:** lsp-poc — a downstream Markdown language server built on this framework (`tree-sitter-md`, `Server` trait, stdio transport)
**Purpose:** input for this project's own brainstorming cycle. Design decisions, naming, error conventions, and process belong to **this** project and its rules; the handoff below states only what the downstream needs and the source-verified facts that shape it.

## Why

lsp-poc implements the feature set of a reference Markdown LSP, which includes `textDocument/documentSymbol` and `workspace/symbol`. Today that is impossible on this framework:

- The `Server` trait (`src/server_trait.rs:37-257`) has methods for hover, completion, code actions, links, declaration/definition, references, rename, formatting, and diagnostics — **but none for documentSymbol or workspaceSymbol**.
- The async-lsp `Router` is assembled inside `serve()`'s private closure (`src/serve.rs:59-73`, `Router::from_language_server(LanguageServerWithState::new(...))`), and `LanguageServerWithState` is `pub(crate)` (`src/server_with_state.rs:129`) — a downstream server cannot register its own request handlers, and abandoning `serve()` means losing the document store (`ServerState` constructors are `pub(crate)`, `src/server_state.rs:97-105`).

So the capability must be added here.

## Requirements

**R1 — trait surface.** Add `document_symbol` and `workspace_symbol` to the `Server` trait in the existing handler shape (like `hover` at `src/server_trait.rs:69`):

```rust
fn document_symbol(&self, state: ServerState, params: DocumentSymbolParams)
    -> impl Future<Output = ServerResult<Option<DocumentSymbolResponse>>> + Send;
fn workspace_symbol(&self, state: ServerState, params: WorkspaceSymbolParams)
    -> impl Future<Output = ServerResult<Option<Vec<SymbolInformation>>>> + Send;
```

Exact signatures (response types, Option wrapping) are this project's call — lsp_types 0.95 shapes are the reference. Defaults for unimplemented methods should match the existing behavior (unimplemented handler → `METHOD_NOT_FOUND` RPC error, `src/server_trait.rs:259-264`).

**R2 — delegation.** `LanguageServerWithState` routes `textDocument/documentSymbol` and `workspace/symbol` to the new trait methods, wired exactly like the existing request handlers (the request-dispatch macro in `src/server_with_state.rs:33-90`).

**R3 — capabilities.** A server advertising `textDocument.documentSymbolProvider` / `workspaceSymbolProvider` through its `server_capabilities()` return must reach the client unchanged. Per downstream research, `initialize` takes the impl's capabilities, merges workspace-diagnostics options, and overwrites only `position_encoding` and `text_document_sync` (`src/server_with_state.rs:147-247`) — so provider fields should pass through already; please verify.

**R4 — position/range encoding.** Handlers see UTF-8 positions, and responses are converted back to the negotiated encoding in `src/requests.rs:46-282`. `DocumentSymbol`/`SymbolInformation` carry `Range`s in **responses** — the new handlers must go through the same conversion path (this is the subtle part; hover/definition return at most one range, symbol responses return many).

**R5 — bounds.** Everything must hold under `serve()`'s requirements (`S: Server + Clone + Send + Sync + 'static`, `src/serve.rs:52-55`) and the `ConcurrencyLayer(8)` request limit (`src/serve.rs:17,66`).

**R6 — tests** per this project's conventions; if examples exist per capability (`examples/tree_sitter.rs` is the canonical one downstream studied), extend or add one.

## Constraints

- The downstream pins this framework by rev (`v0.0.1` = `d7795c4` today). After this work ships, the owner cuts a new tag; lsp-poc bumps the pin. Semver discipline is this project's call.
- Nothing else changes for downstream: the document store, matcher API, and existing trait methods stay as they are.
- Sequencing note: downstream phases 1–7 (hover, TOC, definition, references, rename, completion, diagnostics) run in parallel with this work and do not depend on it; only the symbols phase waits for the release.

## Acceptance

A downstream server that (a) implements the two trait methods and (b) advertises the two providers receives `textDocument/documentSymbol` and `workspace/symbol` requests, and its symbol responses arrive with correctly encoded ranges in a real editor session.

## Research provenance

The file:line citations above were verified against this repository's sources at `d7795c4` by the lsp-poc research workflow (`docs/superpowers/research/2026-08-28-md-lsp-research.md` in the lsp-poc repo, sections 1 and 6). Re-verify against HEAD before designing — the tree may have moved.
