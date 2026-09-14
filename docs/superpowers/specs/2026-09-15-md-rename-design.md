# Cycle 4 — rename + prepare_rename capability (+ cycle-3 opener tails)

Date: 2026-09-15 · Status: approved by owner (scope + approach A + rust-skills consultation) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

The server serves hover, push+pull diagnostics, definition, and references. Cycle 4
adds the **rename** capability (roadmap cycle 4): `textDocument/rename` and
`textDocument/prepareRename`, with semantics mirroring the reference implementation
`matkrin/md-lsp`'s rename.rs, adapted to this project's model, plus the owner's
extension: **wikilinks are fully renameable** (upstream lacks them).

Rename routes (cursor on → what gets edited):
- **Heading** → its own text (`content_range` ← `new_name`) + every inbound
  link/wikilink fragment across the candidate documents (cross-file, each resolved in
  its own context) ← `slugify(new_name)`.
- **Reference** (a `[label][ref]`/collapsed/shortcut usage) → its label ← `new_name` +
  its same-document definition's label ← `new_name`.
- **Definition** (`[label]: dest`) → its label ← `new_name` + every same-document
  reference's label ← `new_name`.
- **FootnoteReference** → its id ← `new_name` + its footnote definition's id ←
  `new_name`. **FootnoteDefinition** → its id ← `new_name` + every same-document
  footnote reference's id ← `new_name`.
- **Wikilink with fragment** → its fragment ← `slugify(new_name)`.
- **Wikilink without fragment** (`[[doc]]`) → its target text ← `new_name` (the
  physical file is NOT renamed — file renames are `workspace/willRename`, out of
  scope).

Plus the cycle-3 opener tails: a definitions test pinning the `[text][^1]` text-part
guard (cursor on the text → `None`), and a dated sentence in the cycle-2 spec noting
that the delivered references capability also serves the reverse routes
(reference/footnote-reference → their definitions' locations).

## Decisions

- **Approach A (approved): precision sub-ranges collected at parse time.** The model
  gains the ranges rename needs, computed from already-typed grammar children —
  `api-parse-dont-validate`: ranges are validated at construction, not found by string
  search at rename time (the rejected alternative — byte-scanning inside destinations
  under the cursor — is the fragile-workaround class):
  - `Heading` + `content_range: Range` — the `heading_content` child's range (the
    text without the `#` marker or the setext underline).
  - `Link` + `fragment_range: Option<Range>` — the fragment's range inside the
    destination node (byte offset of `#` within that node's own text; `None` when no
    fragment). Same-node math only — no global scans.
  - `Wikilink` + `fragment_range: Option<Range>` — same rule over the wikilink's
    scanned content.
  - `Definition` + `label_range: Range` — the `link_label` child's range.
  - `FootnoteReference` + `id_range: Range`, `FootnoteDefinition` + `id_range:
    Range` — the id's range from the scan's known offsets.
- **`src/renames/mod.rs`** (scope pre-declared in `arch-lint.toml`):
  ```rust
  pub fn prepare_at(
      index: &links::MdIndex, position: LspPosition,
  ) -> Option<async_language_server::tree_sitter::Range>
  pub fn at_position(
      index: &links::MdIndex, self_url: &Url, position: LspPosition,
      new_name: &str, documents: &[DocumentSnapshot],
      resolve: &dyn Fn(&Target) -> Option<Resolved>,
      resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
  ) -> Option<WorkspaceEdit>
  ```
  `prepare_at` maps the cursor to the renameable sub-range under it (content_range /
  fragment_range / label_range / id_range / the whole link range — via `item_at` and
  the new pub(crate) field accessors); the server slices the document text at that
  range for the `placeholder`. The two resolver seams mirror references.
- **prepare_rename returns `RangeWithPlaceholder`** (verified shape in lsp-types
  0.95.1): placeholder = the item's current text (heading text / fragment / label /
  id) — the editor prefills it.
- **`WorkspaceEdit::changes`** (`HashMap<Url, Vec<TextEdit>>`, the simple map form —
  verified in lsp-types 0.95.1; `document_changes` stays `None`).
  **TextEdits are sorted by descending start position within each file** before
  returning — standard LSP practice so sequential client application does not shift
  later ranges.
- **Duplicate-slug rule stays** (first heading in document order). **Absence is
  `None`/`Ok(None)`**: cursor not on a renameable item, unresolvable target,
  fragment-less file link (file renames are `workspace/willRename`, out of scope),
  empty `new_name` — all answer `None` (absence as value; LSP permits an error, the
  POC standard is softer).
- **Server wiring, canonical pair:** `rename_provider:
  Some(OneOf::Right(RenameOptions { prepare_provider: Some(true), .. }))` + the
  `rename` and `rename_prepare` trait methods, eager sync compute, the same
  resolver/`resolve_from`/snapshot seams as references (lifted into one place if the
  duplication crosses the three-similar-lines bar).
- rust-skills consultation (2026-09-15) confirmed approach A: `api-parse-dont-validate`
  (sub-ranges validated at construction), `pat-exhaustive-enum` (no catch-all in the
  CursorItem match), `own-borrow-over-clone`, `type-unicode-length` (byte offsets
  throughout; the framework converts at the trait boundary), plus the descending-sort
  edit rule and `RangeWithPlaceholder`.
- Roadmap: cycle 5 = completion; 6 = symbols/ToC; 7 = code actions (ToC + Tables);
  8 = hover upgrade; 9 = folding.

## Changes

- `crates/lsp-poc/src/links/mod.rs` — the five sub-range fields (collected in
  `collect_inline`/`collect_setext_heading`/`collect_heading`/`collect_definition`/
  footnote scanners from typed children and known scan offsets); no behavior change to
  existing fields.
- `crates/lsp-poc/src/renames/mod.rs` — new module: `prepare_at`, `at_position`,
  the edit-map builder with per-file descending sort.
- `crates/lsp-poc/src/server.rs` — `rename_provider` capability + `rename`/
  `rename_prepare` methods (eager sync compute, shared seams).
- `crates/lsp-poc/src/main.rs` — `mod renames;` (alphabetical: after `references`).
- `crates/lsp-poc/tests/fixtures/*` — links fixture unchanged if sub-range tests use
  inline parses; renames tests use inline parses + stub resolvers (references-test
  pattern).
- `crates/lsp-poc/src/definitions/mod.rs` — the `[text][^1]` text-part probe test
  (cycle-3 opener tail).
- `docs/superpowers/specs/2026-09-13-md-model-hardening-definition-design.md` — dated
  reverse-route sentence (cycle-3 opener tail).

## Verification

- Unit tests: sub-range collection (each new field slices back to source text);
  prepare ranges (each of the seven item kinds + miss); rename routes (heading own +
  cross-file; reference↔definition symmetry both directions; footnote symmetry both
  directions; wikilink fragment; wikilink target text; fragment-less file link →
  None; web link → None; descending sort order within a file).
- Gates: whole suite; task gates; `make battery` with **zero warning lines**;
  `make dupes` green.
- Live on both clients: Zed — F2 on a heading renames it and rewrites inbound
  fragments across notes; Claude Code dogfood — `prepareRename` then `rename` through
  LSP after rebuild + `/reload-plugins` ("метод вже так"), definition/references
  regression probes.

## Constraints

- Git is read-only for every agent; the owner commits.
- Zero new dependencies; zero new `PocError` variants; no stdout in server code; no
  panics in hooks or handlers.
- Surgical: completion/symbols/code actions untouched (later cycles); hover,
  diagnostics, definition, references byte-identical except the carried test
  additions; arch-lint.toml untouched (`renames` scope pre-declared).
- `rust-skills` applies to planning and implementation; LSP-first analysis; dispatches
  carry the standing model rules (workers `sonnet[1m]`, reviewers `opus[1m]` with own
  battery/dupes runs, findings as actionable fix-tasks, no workarounds).
