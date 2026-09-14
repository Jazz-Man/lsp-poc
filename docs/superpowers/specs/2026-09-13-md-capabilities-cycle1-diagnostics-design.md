# Markdown capabilities — roadmap and cycle 1 (link model + diagnostics)

Date: 2026-09-13 · Status: approved by owner (design + structure A) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

The server currently serves a single capability (hover). The owner wants to grow it toward
the feature set of the reference tree `vendors/md-lsp` (upstream `matkrin/md-lsp`, ~3000
lines, read in full during design): hover, diagnostics, definition, completion, links,
references, rename, symbols, code actions (table of contents, plus **table column
editing** which upstream only lists as a TODO), formatting (later dropped — see
Decisions).

The reference is cross-file by design (it indexes every `.md` in the workspace), sync
raw-`lsp_server` loop, `markdown`-crate mdast + hand-written `TraverseNode` (551 lines)
+ regex wikilink post-processing. None of that carries over: this project is built on
`async-language-server` v0.10.0 and tree-sitter, and the owner wants all document
scanning on tree-sitter.

**Framework facts verified against the pinned rev `v0.10.0` (`935d564`) during design:**

- The `Server` trait already declares every needed request method: `definition`,
  `references`, `rename` + `rename_prepare`, `completion`, `code_action`,
  `document_symbol`, `symbol` (workspace), `document_diagnostics`, `link`, plus
  notification hooks `did_open`/`did_change`/`did_change_watched_files`/
  `did_change_workspace_folders` (called after the internal handler, so documents are
  already updated when the hook runs).
- The framework has a workspace index: with `ServerOptions::with_workspace_diagnostics`
  (default `Enabled`) it walks workspace roots (`ignore`-based walker), parses matching
  files with the server's document matchers, and stores them as `DocumentOrigin::Workspace`
  in the same document store. But population is **lazy — triggered by a client
  `workspace/diagnostic` pull** — and `workspace_roots()` is `pub(crate)`: the server
  cannot see the roots through the public API.
- `ServerState::client()` is public: the server can push notifications
  (`publishDiagnostics`) itself. There is no built-in push path.
- `tree-sitter-md` 0.5.3 (direct dep, `parser` feature) ships **two grammars**:
  `LANGUAGE` (block) and `INLINE_LANGUAGE` (inline). The framework's document store
  parses only the block tree, so inline links are invisible there. The crate's
  `MarkdownParser` parses both and exposes `block_tree()` / `inline_trees()`.
  Typed inline nodes: `inline_link`, `full_reference_link`, `collapsed_reference_link`,
  `shortcut_link`. **No footnote and no wikilink nodes exist in either grammar.**

**Owner decisions (2026-09-13):**

1. **Cross-file from the start** — workspace-wide analysis is in scope; the product
   rule's cross-file ban is superseded (rule edit goes into the cycle-1 plan).
2. **Wikilinks included** from the start.
3. **Formatting dropped** from the roadmap entirely (owner reversed an earlier
   inclusion). A thought about a future shared framework/library for multiple LSP
   servers is recorded as a future consideration, not designed now.
4. **Structure A**: one roadmap + detailed cycle-1 spec (this document); every later
   capability gets its own small spec → plan → live-verification cycle.
5. Standing: the `rust-skills` skill is invoked during planning, implementation, and
   review of all Rust code.

## Decisions

- **Own document model on `MarkdownParser`**: new module `crates/lsp-poc/src/md/`
  parses document text with `tree_sitter_md::MarkdownParser` (block + inline trees).
  The framework's block-only tree stays for the existing hover; link analysis uses the
  model. No upstream rev bump is required for anything in cycle 1.
- **Push diagnostics as the primary channel**: hooks `did_open`/`did_change` compute
  diagnostics and publish via `state.client().notify(publishDiagnostics)`. Pull parity:
  implement `document_diagnostics` from the same compute function and advertise
  `diagnostic_provider` with `workspace_diagnostics: false` (the framework's
  workspace-pull walk stays off; Zed's pull-diagnostics support is unverified, push is
  known-good). Advertised `diagnostic_provider` also switches on framework workspace
  folder tracking, which feeds `did_change_workspace_folders`.
- **Cross-file resolution without upstream changes**: link targets resolve against the
  requesting document's own `MdIndex` first, then against `state.documents()` snapshots,
  then by reading the target file from disk into a small server-side cache keyed by
  `Url → (FileStamp, Arc<MdIndex>)`. Normal links resolve relative to the document's
  directory; wikilinks resolve against the workspace root, found by walking up to the
  nearest `.git` directory (cached per root; Zed worktrees are git repos). Root
  tracking also updates on `did_change_workspace_folders`.
- **Heading normalization**: `lowercase` + spaces → `-` (the reference's matching).
  Not a full GitHub slugger (no punctuation stripping) — accepted POC divergence, noted
  in diagnostics messages only by behavior, not in text.
- **Diagnostic codes**: the reference's 6 minus code 0, plus a wikilink code:
  - `1` — `#heading` not found in the same document
  - `2` — `file.md#heading`: file exists, heading not found
  - `3` — target file does not exist
  - `4` — `[text][ref]` without a matching link reference definition
  - `5` — `[^ref]` without a matching footnote definition
  - `6` — `[[wikilink]]` that does not resolve (new; upstream has no such code)
  - Code 0 ("invalid link syntax") is consciously dropped: with a typed parse,
    malformed links simply do not parse as links; recreating the reference's
    regex-suspicion heuristic adds noise, not signal. `[[target|alias]]` is out of
    scope (upstream lacks it too).
- **Error handling**: absence is not an error — a missing target file is diagnostic
  code 3; an unreadable file is `tracing::warn` + skip; `MarkdownParser::parse`
  returning `None` is warn + empty publication for that round. No new `PocError`
  variants are expected; the only `ServerResult` boundary is `document_diagnostics`.

## Cycle 1 changes

Module names follow the scopes already declared in `arch-lint.toml` (the layering was
ratified up front in an earlier round): the pure link model lives in `src/links/`, the
cross-file index in `src/workspace/`, the diagnostic computation in `src/diagnostics/`.
(Corrected 2026-09-13: the design discussion named these `src/md/mod.rs` +
`src/md/resolve.rs` + `src/diagnostics.rs`; the declared scopes win — no new scopes are
needed, only a header-comment refresh.)

- `crates/lsp-poc/src/links/mod.rs` — pure model types (`MdIndex`, heading, link,
  link-reference-definition, footnote-ref/def, wikilink), `build(parser, text)`,
  target parsing (`Target`), `slugify`; footnote/wikilink shapes are an "offtree" scan
  over inline text (the grammars have no nodes for them). Known limitation, documented
  in the module: shapes inside code spans match too.
- `crates/lsp-poc/src/workspace/mod.rs` — cross-file index: `Index` owning its own
  `MarkdownParser` (so server and index never fight over one Mutex), a
  `Url → (FileStamp, Arc<MdIndex>)` cache, git-root discovery (`root_for`/`reset_root`),
  and `resolve(open, self_url, target) -> Option<Resolved>` where `open` is a closure
  the server wires to `state.document(...)` — workspace stays below `server` per the
  declared deny rule.
- `crates/lsp-poc/src/diagnostics/mod.rs` — `compute(index, resolve)` producing the
  codes above; converts tree-sitter ranges via `tree_sitter_utils`.
- `crates/lsp-poc/src/server.rs` — capability additions (`diagnostic_provider` with
  `workspace_diagnostics: false`), hooks `did_open`/`did_change` (publish) and
  `did_change_workspace_folders` (root reset), `document_diagnostics` (pull parity);
  `PocLanguageServer` gains its own parser Mutex and the `workspace::Index`; the
  `Debug, Clone` derives are dropped (no framework user needs them, the new fields
  aren't `Clone`). (Corrected 2026-09-13 during execution: `serve` at the pinned rev
  bounds `S: Server + Clone` — `Clone` was restored via `Arc<Mutex<MarkdownParser>>` +
  `Arc<Index>` fields; `Debug` stays dropped.)
- `crates/lsp-poc/src/main.rs` — `mod diagnostics; mod links; mod workspace;` declared
  in the same change (structure rule's module gotcha).
- Docs: `CLAUDE.md` (capability + feature-module lines), `product.md` (cross-file
  sentence superseded by owner decision, "currently: hover" line), and the
  `arch-lint.toml` header comment (links/workspace/diagnostics have landed).
- `crates/lsp-poc/arch-lint.toml` scopes themselves stay untouched — all three are
  already declared; `NoSyncIo` allow comments with reasons go on each synchronous
  filesystem call (notification hooks are synchronous by protocol necessity).

## Roadmap (each cycle: own spec → plan → live Zed verification)

2. **definition** — link / link-ref / footnote-ref / wikilink → target location.
3. **references** — heading → links to it; definition → link-refs; footnote-def → refs.
4. **rename** (+ `rename_prepare`) — the reference's five scenarios.
5. **completion** — triggers `](`, `[[`, `[^`, `[` (headings/files/definitions/footnotes).
6. **symbols** — `document_symbol` (headings) + `symbol` (workspace headings).
7. **code actions** — create/update ToC (`<!--toc:start-->`/`<!--toc:end-->` markers)
   and **table column actions** (insert left/right, delete — our own, upstream TODO).
8. **hover upgrade** — heading outline, link target preview, ref/footnote definitions.

**Future considerations (parked, not designed):** workspace-pull diagnostics flag and
framework walker reuse; upstream `workspace_roots()` accessor + deliberate rev bump;
a shared framework/library for the owner's future multiple LSP servers; `documentLink`
capability (nearly free once resolution exists); upstream TODOs (unused
definition/footnote-definition warnings, wikilink → canonical link replacement).

**Out of scope (overall):** formatting (dropped by the owner), changes to the pinned
upstream rev, changes to the Zed extension crate, changes to the existing hover.

## Verification

- `make battery` green; arch-lint test validates the new scopes.
- Unit tests on the pure model: fixture document → headings/definitions/links/
  footnotes/wikilinks collected with correct ranges; resolution cases (heading
  hit/miss, file hit/miss, wikilink with and without `.md` suffix, footnote hit/miss);
  diagnostics tests (codes + ranges).
- Planning-time verifications — done during planning (2026-09-13): inline trees parse
  through `set_included_ranges` with **absolute document coordinates**, so no offset
  composition is needed; `async_language_server::testing` is `#[cfg(test)] pub(crate)`,
  so no integration leg exists — pure-function tests + the live Zed test it is.
- **End-to-end (owner)**: `cargo build`, open a `.md` in Zed, break a link — red
  underline appears; fix it — disappears.

## Constraints

- Git is read-only for every agent; the owner commits.
- Surgical: nothing outside the named files; the existing hover stays as is.
- `rust-skills` is invoked during planning and implementation; LSP-first code analysis
  with documented fallbacks; never print to stdout in server code.
