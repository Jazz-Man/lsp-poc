# Markdown LSP Feature Set — Design

**Date:** 2026-08-28
**Grounding:** `docs/superpowers/research/2026-08-28-md-lsp-research.md` (6 sections; every architectural claim below cites it as §N.M)
**Status:** approved section-by-section by the owner on 2026-08-28

## Goal

Implement every `vendors/md-lsp` feature **except formatting** (the owner uses external linters/formatters) on the lsp-poc server: hover, gotoDefinition, references, documentSymbol, workspaceSymbol, code actions (TOC), completion, rename/prepareRename, diagnostics — as a Markdown LSP built on the owner's fork of `async-language-server` and `tree-sitter-md`.

The server targets the **LSP standard, not any specific client**. Questions about client behavior (what an editor sends, how it surfaces results) are out of scope by owner's decision — the binary must work with any LSP-capable editor. The Zed extension is just a launcher.

## Owner decisions (2026-08-28 brainstorm)

1. **Symbols via fork extension.** `documentSymbol`/`workspaceSymbol` have no `Server` trait methods (§1) and the Router is sealed inside `serve()` (§6.4) — unreachable without framework changes. The owner will extend their fork (`/Users/vasilsokolik/www/async-language-server`, currently at `d7795c4` = the pinned `v0.0.1`) as a **separate project with its own pipeline**; lsp-poc consumes the new release afterwards. A requirements handoff document accompanies this spec: `2026-08-28-als-symbols-requirements-handoff.md`.
2. **Workspace index when needed.** Cross-file semantics get their index at the moment the first cross-file feature (definition, phase 3) requires it — no upfront infrastructure.
3. **Both GFM extras in scope:** wikilinks **and** footnotes, via an off-tree layer over the tree-sitter-md representation (spike phase 0 determines how thin that layer can be).
4. **Diagnostics: syntax + full link validation**, pull model (`document_diagnostics`); the push channel exists (§6.5) but has no framework trigger and is not used.

## Architecture

**The tree is the core; everything else is a layer around it.** The framework's document store stays untouched: one matcher, one grammar `tree_sitter_md::LANGUAGE` (block). Inline content and off-tree extras (footnotes, wikilinks) never enter the store — they are pure functions over text/tree, called per request (§6.7 strategy (a)).

**Module layout** (per the structure rule: thin trait methods, logic in modules; every `mod` line lands in `main.rs` in the same change):

```
crates/lsp-poc/src/
├── server.rs          — the single Server impl: advertise + dispatch + .or_else() chains (zap pattern, §2)
├── error.rs           — PocError (created here, finally, per error-handling.md)
├── hovers/            — phase 1
├── code_actions/      — phase 2 (TOC)
├── links/             — phase 3: normalize (slug/label) · inline (side-parse) · offtree (footnotes+wikilinks)
├── definitions/       — phase 3
├── references/        — phase 4
├── renames/           — phase 5
├── completions/       — phase 6
├── diagnostics/       — phase 7
├── symbols/           — phase 8 (after the fork release)
└── workspace/         — phase 3: WorkspaceIndex (server-owned state, §6.3)
```

**Testable signatures — the load-bearing design fact:** `Document` has no public constructor (only getters; fork `src/document.rs`), so unit tests cannot build one. Feature functions therefore **never** take `&Document`. Every feature function takes only what a test can construct:

```rust
fn some_feature(text: &str, tree: &Tree, pos: Position) -> Option<T>
```

The trait method in `server.rs` is a thin wrapper: doc → `text_contents()` + tree → call the feature fn → wrap the result. This also keeps features framework-agnostic.

**Zap conventions adopted** (§2: transferable ~1:1): per-capability free functions with rename-on-re-export in each `mod.rs`; `.or_else()` chains in `server.rs`; let-else + `tracing::debug!` + `Ok(None)` for every absence; `server_info()` from `env!("CARGO_PKG_*")`. **Not** adopted: `.expect()` (forbidden by workspace lints), zap's byte-offset position arithmetic (ASCII-only bug; we use only `tree_sitter_utils` conversions), zap's untyped errors.

## Components

- **`links/normalize`** — pure functions: heading → slug, label → normalized label (reference matching). Input `&str`, output `String`. The single module through which **all** label/anchor matching flows — no per-feature forks of this logic (research risk #10). md-lsp's baseline semantics: lowercase + space→dash (§3, `ast.rs:107-145`).
- **`links/inline`** — `parse_inline(text) -> Option<Tree>`: `Parser` + `INLINE_LANGUAGE` over the text of one target `inline` node (§6.7 strategy (a); the parser feature stays enabled for its included-ranges reference logic).
- **`links/offtree`** — footnotes and wikilinks. Working hypothesis (spike phase 0 decides): `[^ref]:` parses as `link_reference_definition` with label `^ref`, reducing footnotes to a classification over reference nodes; wikilinks need a regex scan layer with computed positions, its items merged into every link-consuming feature (trees are immutable — no synthetic nodes, §3 non_portable #4).
- **`workspace/`** — `WorkspaceIndex`: `Arc<DashMap<Url, IndexedFile>>` field on `PocLanguageServer` (sanctioned pattern §6.3; lsp-poc is the first user — Rust-legal, not framework-exemplified). Own scan (walk + `*.md` + read + parse) — the framework's scan is gated on an advertised `diagnostic_provider` (§6.6). Refresh: lazy initial scan + mtime check of the target file at resolve time. Keyed by `Url` + mtime, never by document `version` (its staleness holes, §6.2, do not apply to disk files).
- **`error.rs`** — `PocError` (below).

## Data flow (canonical: cross-file definition)

```
request → snapshot doc → block tree → node at position
  → if inline node → parse_inline(paragraph text) → link node
  → normalize(url/fragment) → WorkspaceIndex lookup (mtime check, rescan if stale)
  → Location → response (version re-check / CONTENT_MODIFIED is the framework's, §6.2)
```

Hover and TOC follow the same path without index or inline: `text + block tree + pos` → result.

## Error handling

- `PocError` starts minimal, per the restraint rule: `IndexRead { path: PathBuf, source: std::io::Error }` — the one real failure computable now (index file read). New variants land only when their phase introduces a genuinely unrecoverable failure.
- **Absence is not an error:** missing document/node/target/heading → `Ok(None)` throughout (let-else + debug log). A side-parse returning `None` is absence.
- **No `Document::query`:** diagnostics finds `ERROR`/`MISSING` nodes by **walking the tree**, not `doc.query(...)` — that API silently swallows query-compile failures as `None` (§1). We compile no queries; therefore no `QueryCompile` variant.
- Boundary: `impl From<PocError> for ServerError` with one `tracing::error!` at the edge; `?` converts inside handlers. `anyhow` stays at the CLI edge only.

## Testing

Tests live in `crates/lsp-poc` (never the extension crate). Unit tests per feature function — the `(text, tree, pos)` signatures make them trivial:

- Test helpers `parse(text) -> Tree` (block) and `parse_inline(text) -> Tree`: tests build exactly what `server.rs` passes — `Parser` + `LANGUAGE`/`INLINE_LANGUAGE` directly, no framework.
- Per phase: hover-outline (headings → outline with current marked), TOC insert/replace between markers, `normalize` (table-driven), `offtree` (footnote/wikilink with positions), definition/references/rename (fixtures → expected `Location`/`WorkspaceEdit`), diagnostics (text → expected diagnostics), completion modes.
- **The phase-0 spike becomes a pinned test:** the assertion about how tree-sitter-md actually represents `[^ref]:` and `[[wiki]]` is fixed as a test; the offtree layer is built against it.
- Phase 7 optionally adds an E2E through the fork's `oneshot::workspace_diagnostics` batch runner over fixture files.
- Live verification in the editor remains each phase's success criterion (product rule) — a complement to the unit tests, not a replacement.

## Roadmap

Each phase is its own plan → subagent implementation → review → live verification cycle (product rule: one capability at a time).

| # | Phase | Delivers | Success criterion |
|---|-------|----------|-------------------|
| 0 | Representation spike | Pinned node-shape test for `[^ref]:` and `[[wiki]]`; final offtree design | test committed |
| 1 | Hover | Heading → document outline + raw-node fallback re-fenced ```markdown (fixes server.rs:72) | unit tests + live hover |
| 2 | TOC code action | Create/update between `<!--toc:start-->`/`<!--toc:end-->` markers | tests + live code action |
| 3 | Definition + link stack | `links/` (normalize, inline, offtree) + **WorkspaceIndex**; same-doc refs → cross-file | goto-def works both scopes |
| 4 | References | Inbound links to headings/definitions (workspace-wide) | full reference list |
| 5 | Rename | ref↔def pairs same-doc; heading rename rewrites inbound links (wikilink vs normal) | correct `WorkspaceEdit` |
| 6 | Completion | Trigger chars `[` `^` `(` (not md-lsp's unhandled `#` `\|`); cursor-stickiness pre-pass (zap); all md-lsp modes | completions in every mode |
| 7 | Diagnostics | ERROR/MISSING walk + link validation, pull model | expected diagnostics on fixtures |
| 8 | Symbols | Bump fork pin → `symbols/`: documentSymbol + workspaceSymbol | outline + symbol search |

Phase 8 detail: md-lsp returns a flat heading list; tree-sitter-md has `section` nodes, so hierarchical `DocumentSymbol` with `children` is an improvement candidate — decided in phase 8's plan (parity is the default).

## Parallel track: the fork

Runs after this brainstorm, in parallel with phases 1–7; does not block them.

1. This spec ships with `2026-08-28-als-symbols-requirements-handoff.md` — requirements + research facts (file:line citations into the fork's sources) for `documentSymbol`/`workspaceSymbol` support.
2. The owner loads the handoff into `/Users/vasilsokolik/www/async-language-server` (separate Zed window) and runs that project's full brainstorming cycle there (rust skill + LSP available).
3. Claude reviews the implementation in that folder and provides fixes.
4. Owner commits, pushes, cuts a release (new tag).
5. lsp-poc bumps the dependency pin; phase 8 proceeds.

## Out of scope

- **Formatting** (`document_format`/`document_range_format`): the owner uses external tools; md-lsp's implementation is dprint, excluded from the start.
- **Client-specific behavior** of any kind (owner's standing decision).
- Framework changes inside lsp-poc: everything fork-related happens in the fork's project.

## Standing constraints

Workspace lints (`unwrap_used`/`expect_used`/`dbg_macro` forbidden), typed `PocError`, stderr-only logging, git read-only for agents (the owner commits), tests in `crates/lsp-poc`, rust skills + `lsp-code-analysis` + no-workarounds on every planning and implementation pass.
