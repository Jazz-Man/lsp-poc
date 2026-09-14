# Cycle 2 — link-model hardening + definition capability

Date: 2026-09-13 · Status: approved by owner (scope + design) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

Cycle 1 (commits 3df84d8…e784cba) landed the link model, the cross-file index, push+pull
diagnostics, and — through the Claude Code dogfood plugin — a second real client. Three
defects were found and triaged to this cycle, plus the next roadmap capability:

1. **Code regions are scanned** (dogfood bug report, 8 false positives on the cycle-1 plan
   document): the off-tree footnote/wikilink scans run over every inline tree, so `[[…]]`/`[^…]`
   shapes inside code spans produce bogus codes 5/6. (Corrected 2026-09-13 during execution:
   tree-sitter-md already emits no inline trees for fenced blocks — the plan document's
   fence-line hits traced to nested-fence misparsing, where inner ``` markers close the
   outer markdown fence and the remainder parses as live Markdown. The model leak is code
   spans; the fence filter below stays as no-op defense for rev bumps.) The upstream
   reference does not have this class of false positive (mdast models code as `Code`
   nodes, not text).
2. **Setext headings are not collected**: `collect_block` matches only `atx_heading`;
   tree-sitter-md 0.5.3 also models `setext_heading` (same `heading_content` field).
   Links targeting setext headings false-positive codes 1/2.
3. **scan_range columns skew on multi-line inline nodes**: for a hit on a continuation
   line, `base.column + (start − line_start)` is only correct when the continuation's
   leading prefix width equals the root's start column — true for blockquote/list
   continuations, wrong for lazy continuations. Bytes are always correct. Untested (the
   deferred multi-line test).

Plus the capability: **definition** (roadmap cycle 2): link / reference link /
footnote reference / wikilink → target `Location`.

## Decisions

- **Code exclusion at the model's roots** (the only workable mechanics): during the
  block walk, collect the ranges of `fenced_code_block` and `indented_code_block`;
  inline trees whose root range falls inside any collected range are skipped entirely
  (both `collect_inline` and `scan_offtree`). Inside a live inline tree, the walk
  collects typed `code_span` ranges and the off-tree scans run only over the
  complement segments. Walk-level avoidance is impossible: `MarkdownTree` does not
  expose the inline-tree → parent mapping (`inline_indices` is private), so exclusion
  must be range-based. Typed link nodes inside code spans do not exist (backticks take
  precedence), so `collect_inline` needs no span handling.
- **The model un-trims where a consumer now exists** (cycle 1 trimmed to fields with
  readers; definition is the reader): `Heading` regains `range`;
  `Definition` becomes `{ label, destination, range }`;
  `FootnoteDefinition` becomes `{ id, range }`. Accessors: `wikilinks()` already
  exists; `headings()`, `definitions()`, and `footnote_definitions()` are added,
  returning the structs. `has_heading_slug`/`has_definition`/
  `has_footnote_definition` stay (diagnostics uses them).
- **Duplicate slugs**: the first heading with the matching slug wins (document order);
  stated once here, applied by definition and later reference cycles alike.
- **Setext**: a `setext_heading` arm in `collect_block` — level from the underline
  child's kind (`setext_h1_underline` → 1, `setext_h2_underline` → 2), content from the
  same `heading_content` field, same `slugify`.
- **Slug semantics upgraded** (owner decision 2026-09-14, superseding cycle 1's
  "not a full slugger — accepted POC divergence" and its `slugify` in
  `src/links/mod.rs`): a dedicated `src/links/slug.rs` module, GitHub-style and
  unicode-preserving — trim ends, lowercase, whitespace runs (`char::is_whitespace`,
  covers NBSP/ideographic) collapse to one dash, keep `char::is_alphanumeric` plus `_`
  and `-`, drop everything else, trim edge dashes. `# Привіт, світ` → `привіт-світ`;
  `Setext Title` → `setext-title` (existing fixtures unaffected). One function serves
  both matching sides (heading collection + fragment resolution). No positional math
  inside — UTF-8 position handling stays where it belongs: the framework converts
  client positions to UTF-8 at the trait boundary and tree-sitter ranges are byte-based;
  `slugify` is a pure string transform and encoding-neutral.
- **scan_range**: for a hit below the inline node's first line (`rows > 0`), the column
  is `start − line_start` without the base column; the first-line branch is unchanged.
  Multi-line fixture lines (including a lazy-continuation) pin both branches.
- **Definition resolution mirrors diagnostics**: `src/definitions/mod.rs` (scope
  pre-declared in `arch-lint.toml`) maps a position to the model item containing it —
  all item ranges are document-absolute, so this is a plain range search over the
  requesting document's `MdIndex`, no tree-sitter at request time — then resolves:
  - `Link` with fragment: own document's first heading with the slug → its range;
    otherwise cross-file through the same `resolve` closure diagnostics uses → the
    target index's heading range. No fragment → `Location{target, 0:0}`.
  - `Reference` → the matching definition's range in the same document.
  - `FootnoteReference` → the matching footnote definition's range.
  - `Wikilink` → like `Link` with wiki candidate resolution.
  - External/empty/unparseable → `None`.
  Response is `GotoDefinitionResponse::Scalar`; no definition for the cursor position →
  `Ok(None)` (absence is not an error).
- **Server wiring, canonical pair**: `definition_provider: Some(OneOf::Left(true))` +
  the `definition` trait method delegating to `src/definitions/`; the `open` closure
  and `files.resolve` are shared with the diagnostics path (wiring stays in
  `server.rs`).
- **Dupes ledger effect**: the `has_definition`/`has_footnote_definition` duplicate
  group dissolves (bodies diverge); its now-stale fingerprint entry is pruned with
  `cargo dupes cleanup` (dry-run first), never left rotting.
- Roadmap stays: cycle 3 = references.

## Changes

- `crates/lsp-poc/src/links/mod.rs` — code-range collection + inline-tree filtering,
  `code_span` complement scanning, setext arm, `scan_range` column fix, un-trimmed
  `Heading`/`Definition`/`FootnoteDefinition`, struct accessors, module-doc update
  (the "ATX only" line is replaced by the setext support; the code-limitation line is
  replaced by the exclusion description).
- `crates/lsp-poc/src/definitions/mod.rs` — new: position → item → `Location`.
- `crates/lsp-poc/src/server.rs` — capability + `definition` method; shared resolver
  closure lifted so diagnostics and definitions use one construction.
- `crates/lsp-poc/src/main.rs` — `mod definitions;` (alphabetical, the structure
  rule's gotcha).
- `crates/lsp-poc/tests/fixtures/*` — links fixture gains a setext heading and a fenced
  block containing decoy links (must stay silent); a definitions fixture with known
  ranges; diagnostics fixtures unchanged.
- `crates/lsp-poc/.dupes-ignore.toml` — prune the dissolved group's entry.

## Verification

- Unit: fixture assertions for setext collection, fence silence (no diagnostics from
  decoy links inside fenced blocks), code-span silence, multi-line `scan_range` ranges
  sliced back to source text, definition routes (each of the four item kinds, own-doc
  and cross-file, miss → `None`).
- Gates: `make fmt-fix && make fmt && make clippy && make test`, then `make battery`;
  `make deny` = ratified red with exactly one wildcard; `make dupes` green with the
  dissolved entry pruned.
- Live (both clients): owner in Zed — cursor on a link, F12 jumps to the target; and
  the dogfood loop — after rebuild + `/reload-plugins`, Claude Code's `goToDefinition`
  on the fixture returns the expected locations ("метод вже так"). Diagnostics on the
  cycle-1 plan document drop to zero false positives from fenced examples.

## Constraints

- Git is read-only for every agent; the owner commits.
- No new dependencies; no new `PocError` variants; never print to stdout in server
  code; hooks and request handlers must not panic.
- Surgical: references/rename/completion stay untouched (later cycles); hover stays
  byte-identical; `arch-lint.toml` scopes untouched (only content lands in the
  pre-declared `definitions` scope).
- `rust-skills` applies to planning and implementation; LSP-first code analysis.
