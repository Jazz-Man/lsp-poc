# Cycle 3 — references capability (+ cycle-2 opener tails)

Date: 2026-09-14 · Status: approved by owner (scope + approach A) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

The server serves hover, push+pull diagnostics, and definition. Cycle 3 adds the
**references** capability (roadmap cycle 3) with semantics mirroring the reference
implementation `matkrin/md-lsp`, adapted to this project's model:

- references on a **heading** → every link and wikilink pointing at that heading,
  across the whole workspace (cross-file, mirroring definition's reach);
- references on a **link reference definition** → the link references using it
  (same document — link references are a same-document construct);
- references on a **footnote definition** → the footnote references using it
  (same document);
- **extension over upstream (owner decision 2026-09-14):** references on a **link**
  (inline link or wikilink) → the target it points to (the heading's range, or the
  target file's location). **Web targets are excluded**: `parse_destination` already
  returns `None` for `://` and `mailto:` destinations, and editors handle open-link
  themselves — the filter falls out of the existing parser.

Plus the cycle-2 final-review opener tails, folded into this cycle's plan:
two missing definition route-cell tests (wikilink route, no-fragment arm),
the stale CLAUDE.md capability line, and a spec-text note for the cycle-2
"definitions fixture" wording.

## Decisions

- **Approach A (approved): shared cursor-item extraction in `links`.**
  `src/links/mod.rs` gains `CursorItem` (enum over `Heading`/`Link`/`Reference`/
  `FootnoteReference`/`Wikilink`, borrowed) and
  `item_at(index, position) -> Option<CursorItem>` (range containment via
  `ts_range_contains_lsp_position`, collection-order search). `src/definitions/`
  switches to it — the "what is under the cursor" containment search is written once,
  not twice; definitions' routing stays its own.
- **`src/references/mod.rs`** (scope pre-declared in `arch-lint.toml`):
  ```rust
  pub struct DocumentSnapshot { pub url: Url, pub index: Arc<links::MdIndex> }
  pub fn at_position(
      index: &links::MdIndex,
      self_url: &Url,
      position: LspPosition,
      documents: &[DocumentSnapshot],
      resolve: &dyn Fn(&Target) -> Option<Resolved>,
  ) -> Option<Vec<Location>>
  ```
- **Heading references are cross-file and context-correct:** a link in document B
  pointing at `#slug` refers to B's own heading, not the requesting document's — so
  each candidate document's links resolve with that document's own URL as the base
  (`workspace::Index::resolve` is already parameterized by `self_url`; the bound
  resolver closure from definition is used only for the Link/Wikilink routes). A
  candidate link counts when its resolution lands on the requesting document and a
  heading whose slug matches the requesting heading's slug; the link's own range is
  the `Location`. The requesting document is part of the candidate snapshot (a link
  within A to A's heading counts).
- **Document snapshot:** `workspace::Index` gains `snapshot() -> Vec<DocumentSnapshot>`
  (clone of the stamped cache); the server merges in open documents from
  `state.documents()` (own parse each — the established POC cost, same ledger line as
  `version: None`). Honest scope: this is "open ∪ previously resolved", not a full
  disk walk on every request.
- **Definition/footnote-definition references are same-document** per CommonMark.
- **Wiring, canonical pair:** `references_provider: Some(OneOf::Left(true))` + the
  `references` trait method with eager sync compute (hover/document_diagnostics
  precedent). Absence — no item, no target, zero matches — is `None`/`Ok(None)`
  (absence is not an error). Web/unparseable destinations never reach a route.
- **Duplicate-slug rule stays**: first heading with the matching slug in document
  order wins (cycle-2 spec), applied by the heading-references matcher.
- **Cycle-2 opener tails ride this plan:** two definition route-cell tests
  (wikilink route; no-fragment arm — first heading, zero range only when the target
  has no headings) in `src/definitions/mod.rs`; CLAUDE.md capability line updated to
  "hover, diagnostics, definition, and references"; the cycle-2 spec's "definitions
  fixture with known ranges" wording amended (delivered as inline `parse()` strings).
- Roadmap: cycle 4 = rename (+prepare); 5 = completion; 6 = symbols/ToC; 7 = code
  actions (ToC + Tables); 8 = hover upgrade; 9 = folding.

## Changes

- `crates/lsp-poc/src/links/mod.rs` — `CursorItem` + `item_at`; no behavior change to
  existing items.
- `crates/lsp-poc/src/definitions/mod.rs` — switch to `item_at`; +2 route-cell tests.
- `crates/lsp-poc/src/references/mod.rs` — new module: `DocumentSnapshot`,
  `at_position` with the four routes.
- `crates/lsp-poc/src/workspace/mod.rs` — `Index::snapshot() -> Vec<DocumentSnapshot>`
  (note: `DocumentSnapshot` lives in `references` and is constructed by the server —
  `snapshot()` returns `(Url, Arc<MdIndex>)` pairs to keep workspace free of
  references-layer types).
- `crates/lsp-poc/src/server.rs` — capability + `references` method + document-snapshot
  helper (merge of open documents and index cache, own parse each).
- `crates/lsp-poc/src/main.rs` — `mod references;` (alphabetical: after definitions,
  before server).
- `CLAUDE.md:23` — capability line; cycle-2 spec wording note.
- Tests: `item_at` (five kinds + miss); references routes (own-doc link, cross-file
  relative link from another document, cross-file wikilink with fragment, non-match,
  definition→refs, footnote-def→refs, link→fragment target, link→file target,
  web link → None, empty → None); +2 definition route-cell tests.

## Verification

- Unit tests as listed; the whole crate suite; task gates; `make battery` with **zero
  warning lines** (cycle-2 discipline); `make dupes` green.
- Live on both clients: Zed — References on a heading shows all inbound links across
  notes; Claude Code dogfood — `findReferences` on the fixture heading after rebuild +
  `/reload-plugins` ("метод вже так"), plus the definition regressions from cycle 2.

## Constraints

- Git is read-only for every agent; the owner commits.
- Zero new dependencies; zero new `PocError` variants; no stdout in server code; no
  panics in hooks or handlers.
- Surgical: rename/completion untouched (later cycles); hover byte-identical;
  arch-lint.toml untouched (`references` scope pre-declared).
- `rust-skills` applies to planning and implementation; LSP-first analysis; dispatches
  carry the standing model rules (workers `sonnet[1m]`, reviewers `opus[1m]`,
  reviewers run battery themselves, findings are actionable fix-tasks).
