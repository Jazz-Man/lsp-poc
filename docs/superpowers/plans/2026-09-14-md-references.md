# Cycle 3 — References Capability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve `textDocument/references`: a heading answers with every link/wikilink that lands on it across candidate documents (cross-file, each link resolved in its own document's context); a link reference definition or footnote definition answers with its usages; a link or wikilink answers with its target (web destinations excluded by the parsers) — per `docs/superpowers/specs/2026-09-14-md-references-design.md`.

**Architecture:** `src/links/` gains `CursorItem` + `item_at` (single containment search; **footnote references search before references** — preserves today's fall-through outcome for `[text][^1]`), and `heading_by_slug` becomes an `MdIndex` method (definitions and references share it — avoids a dupes group). `src/definitions/` switches its routing to `item_at` (behavior-preserving) and gains the 2 missing route-cell tests. New `src/references/` computes the four routes over a `DocumentSnapshot` list, with two resolver seams: the bound `resolve` for the requesting context, `resolve_from(url, target)` for per-candidate contexts. `workspace::Index::snapshot()` exposes the cache; server.rs merges open documents + snapshot and wires the capability pair.

**Tech Stack:** unchanged — Rust 2024 stable, `async-language-server` v0.10.0, `tree-sitter-md` 0.5.3, `cargo nextest`, Makefile battery. **Zero new dependencies.**

## Global Constraints

- Git is read-only for every agent: NO git-write commands. The owner commits. (No commit steps.)
- Zero new dependencies; zero new `PocError` variants; no stdout in server code; no panics in hooks/handlers; no `.unwrap()`/`.expect()` outside tests.
- Battery discipline (owner bar): `make battery` must finish with **zero `^warning` lines** — no new single-letter closure params, no ungrouped imports, no missing trailing commas, no unicode ellipsis in comments, no doc links to nonexistent paths.
- Positions are UTF-8 at the trait boundary; tree-sitter → LSP ranges only via `ts_range_to_lsp_range`.
- Surgical: hover byte-identical; diagnostics untouched; arch-lint.toml untouched (`references` scope pre-declared); rename/completion untouched.
- LESSON (cycles 1–2): brief code is untested by construction — minimal mechanical fixes documented, never BLOCK for brief compile errors; BLOCK only for genuine ambiguity or a mandated anchor not matching the tree.
- Dupes ledger: fingerprint-keyed; restructures that change bodies may stale entries — `cargo dupes cleanup --dry-run` first, prune only stale.
- Dispatch rules (owner directive): workers `sonnet[1m]`, reviewers `opus[1m]`; reviewers run `make battery`/`make dupes` themselves; findings are actionable fix-tasks; `rust-skills` + LSP verification mandatory; no workarounds.
- Every task ends with `make fmt-fix && make fmt && make clippy && make test` green ("the task gates").

### Behavioral invariant to preserve (from cycle-2 review + tree)

`at_position`'s current order is links → guarded references → footnote references → wikilinks, with fall-through: a cursor on the `[^1]` inside `[text][^1]` skips the guarded reference arm (label starts with `^`) and lands on the footnote route. A single-item `item_at` preserves this ONLY if `footnote_references` is searched BEFORE `references` — mandated in Task 1. Do not "fix" the order.

---

### Task 1: `CursorItem`/`item_at` — definitions switches, route-cell tests

**Files:**
- Modify: `crates/lsp-poc/src/links/mod.rs` (imports, `CursorItem`, `item_at`, `MdIndex::heading_by_slug`)
- Modify: `crates/lsp-poc/src/definitions/mod.rs` (routing via `item_at`; +2 route-cell tests)

**Interfaces:**
- Consumes: existing model accessors; `ts_range_contains_lsp_position`.
- Produces (Tasks 2–3 rely on these):
  - `pub enum CursorItem<'a> { Heading(&'a Heading), Link(&'a Link), Reference(&'a Reference), Definition(&'a Definition), FootnoteReference(&'a FootnoteReference), FootnoteDefinition(&'a FootnoteDefinition), Wikilink(&'a Wikilink) }`
  - `pub fn item_at(index: &MdIndex, position: LspPosition) -> Option<CursorItem<'_>>`
  - `impl MdIndex { pub fn heading_by_slug(&self, slug: &str) -> Option<&Heading> }` (definitions' private fn moves here)
- definitions keeps its public signature; the 2 new tests pin the wikilink route and the no-fragment arm.

- [ ] **Step 1: `CursorItem` + `item_at` + `MdIndex::heading_by_slug` in `src/links/mod.rs`**

Extend the imports at the top:

```rust
use async_language_server::lsp_types::Position as LspPosition;
use async_language_server::tree_sitter_utils::ts_range_contains_lsp_position;
```

Add after the `Target` enum:

```rust
/// The model item whose range contains a cursor position.
#[derive(Debug)]
pub enum CursorItem<'a> {
    Heading(&'a Heading),
    Link(&'a Link),
    Reference(&'a Reference),
    Definition(&'a Definition),
    FootnoteReference(&'a FootnoteReference),
    FootnoteDefinition(&'a FootnoteDefinition),
    Wikilink(&'a Wikilink),
}
```

Add to the `impl MdIndex` block (after `has_footnote_definition`):

```rust
    /// First heading whose slug matches, in document order.
    #[must_use]
    pub fn heading_by_slug(&self, slug: &str) -> Option<&Heading> {
        self.headings.iter().find(|heading| heading.slug == slug)
    }
```

Add the locator free function (before the tests):

```rust
/// The model item whose range contains `position`, searched in a fixed
/// order: links, footnote references, references, definitions, footnote
/// definitions, wikilinks, headings. Footnote references precede
/// references so a `[text][^1]` cursor routes to the footnote side, as
/// the cycle-2 fall-through did.
#[must_use]
pub fn item_at(index: &MdIndex, position: LspPosition) -> Option<CursorItem<'_>> {
    let at = |range: Range| ts_range_contains_lsp_position(range, position);
    if let Some(link) = index.links.iter().find(|link| at(link.range)) {
        return Some(CursorItem::Link(link));
    }
    if let Some(footnote) = index
        .footnote_references
        .iter()
        .find(|footnote| at(footnote.range))
    {
        return Some(CursorItem::FootnoteReference(footnote));
    }
    if let Some(reference) = index.references.iter().find(|reference| at(reference.range)) {
        return Some(CursorItem::Reference(reference));
    }
    if let Some(definition) = index.definitions.iter().find(|def| at(def.range)) {
        return Some(CursorItem::Definition(definition));
    }
    if let Some(definition) = index
        .footnote_definitions
        .iter()
        .find(|definition| at(definition.range))
    {
        return Some(CursorItem::FootnoteDefinition(definition));
    }
    if let Some(wikilink) = index.wikilinks.iter().find(|wikilink| at(wikilink.range)) {
        return Some(CursorItem::Wikilink(wikilink));
    }
    if let Some(heading) = index.headings.iter().find(|heading| at(heading.range)) {
        return Some(CursorItem::Heading(heading));
    }
    None
}
```

- [ ] **Step 2b: Add the `item_at` unit test to `mod tests` in `src/links/mod.rs`**

```rust
    #[test]
    fn item_at_locates_each_kind_and_misses_plain_text() {
        let index = fixture();
        let at = |line: usize, column: usize| {
            LspPosition {
                line: u32::try_from(line).expect("line fits"),
                character: u32::try_from(column).expect("column fits"),
            }
        };
        // Byte offsets are read back from the fixture text so the test
        // survives fixture edits.
        let byte_col = |needle: &str| {
            fixture_text().find(needle).expect("needle present")
        };

        let top_line = 0;
        assert!(matches!(
            item_at(&fixture(), at(top_line, 2)),
            Some(CursorItem::Heading(_))
        ));
        let link_at = byte_col("[docs](guide.md)");
        assert!(matches!(
            item_at(&fixture(), at(2, link_at)),
            Some(CursorItem::Link(_))
        ));
        let footnote_at = byte_col("[^1] here");
        assert!(matches!(
            item_at(&fixture(), at(2, footnote_at)),
            Some(CursorItem::FootnoteReference(_))
        ));
        let definition_at = byte_col("[ref]: https");
        assert!(matches!(
            item_at(&fixture(), at(2, definition_at)),
            Some(CursorItem::Definition(_))
        ));
        let wiki_at = byte_col("[[Other]]");
        assert!(matches!(
            item_at(&fixture(), at(2, wiki_at)),
            Some(CursorItem::Wikilink(_))
        ));
        // Line 2 column 0 is `See ` — plain text, nothing reference- or
        // definition-worthy.
        assert!(item_at(&fixture(), at(2, 0)).is_none());
    }
```

**Implementer note — the sketch above guesses coordinates; make them real.** Compute
each needle's `(line, column)` from `fixture_text()` (find the needle, count `\n`
before it for the line, subtract the line start for the column) with a small local
helper, and assert the variant for each. The fixtures contain:
`# Top` (Heading), `[docs](guide.md)` (Link), `[^1]` (FootnoteReference),
`[ref]: https://example.com` (Definition), `[[Other]]` (Wikilink), plus plain text
and — on the references line — `[collapsed][]` (Reference). Cover all seven kinds if
the fixture supports them, else extend the fixture. The miss case must be a genuine
plain-text position.

- [ ] **Step 2: Switch `src/definitions/mod.rs` routing to `item_at`; move `heading_by_slug`**

Replace `at_position`'s body (keep the signature and doc):

```rust
    match item_at(index, position) {
        Some(CursorItem::Link(link)) => {
            let target = parse_destination(&link.destination)?;
            route(index, self_url, &target, resolve)
        }
        // Footnote and wikilink shapes surface as reference labels; the
        // guard sends them to their own arms.
        Some(CursorItem::Reference(reference))
            if !reference.label.starts_with(['^', '[']) =>
        {
            let definition = index
                .definitions()
                .iter()
                .find(|definition| definition.label == reference.label)?;
            Some(scalar(self_url, definition.range))
        }
        Some(CursorItem::FootnoteReference(footnote)) => {
            let definition = index
                .footnote_definitions()
                .iter()
                .find(|definition| definition.id == footnote.id)?;
            Some(scalar(self_url, definition.range))
        }
        Some(CursorItem::Wikilink(wikilink)) => {
            let target = parse_wiki_target(&wikilink.target)?;
            let Target::Wiki { .. } = &target else {
                return None;
            };
            route(index, self_url, &target, resolve)
        }
        // Headings, definitions, and footnote definitions have no
        // definition of their own.
        _ => None,
    }
```

Delete the private `heading_by_slug` fn; its two call sites (`route`'s two arms) call
`index.heading_by_slug(...)` / `target_index.heading_by_slug(...)` instead. Update the
imports: add `item_at` and `CursorItem` to the `crate::links` use:
`use crate::links::{self, CursorItem, Target, item_at, parse_destination, parse_wiki_target, slugify};`

- [ ] **Step 3: Add the two route-cell tests (coverage adds — they must PASS on landing)**

Append to `mod tests` in `src/definitions/mod.rs`:

```rust
    #[test]
    fn wikilink_route_lands_on_the_target_document() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("see [[other#target]]\n");
        let location = location_of(&index, &url, 0, 6).expect("wikilink resolves");
        assert_eq!(
            location.uri.as_str(),
            "file:///target.md",
            "the stub resolver's document",
        );
        assert_eq!(location.range.start.line, 0, "the # Target heading");
    }

    #[test]
    fn fragment_less_file_target_lands_on_the_first_heading() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("see [f](other.md)\n");
        let location = location_of(&index, &url, 0, 8).expect("file target resolves");
        assert_eq!(
            location.uri.as_str(),
            "file:///target.md",
            "the stub resolver's document",
        );
        assert_eq!(location.range.start.line, 0, "its first heading");
    }
```

- [ ] **Step 4: Run everything**

Run: `cargo nextest run -p lsp-poc`
Expected: all pass (definitions suite still 6 tests = 4 + 2 new; links suite unchanged).
Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 2: `src/references/` + `workspace::Index::snapshot()`

**Files:**
- Modify: `crates/lsp-poc/src/workspace/mod.rs` (`Index::snapshot()`)
- Create: `crates/lsp-poc/src/references/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (`mod references;` — alphabetical: after `mod definitions;`, before `mod server;`)

**Interfaces:**
- Consumes: Task 1's `item_at`/`CursorItem`; `MdIndex::heading_by_slug`; `workspace::{Index, Resolved}`.
- Produces (Task 3 relies on these):
  - workspace: `impl Index { pub fn snapshot(&self) -> Vec<(Url, Arc<links::MdIndex>)> }` (cache clone, sorted by URL)
  - references: `pub struct DocumentSnapshot { pub url: Url, pub index: Arc<links::MdIndex> }` and
    `pub fn at_position(index: &links::MdIndex, self_url: &Url, position: LspPosition, documents: &[DocumentSnapshot], resolve: &dyn Fn(&Target) -> Option<Resolved>, resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>) -> Option<Vec<Location>>`

- [ ] **Step 1: `Index::snapshot()` in `src/workspace/mod.rs`**

Add to the `impl Index` block (after `reset_root`):

```rust
    /// Clones the cached (URL, index) pairs, sorted by URL. The snapshot
    /// covers previously resolved workspace files; open documents are the
    /// caller's merge.
    #[must_use]
    pub fn snapshot(&self) -> Vec<(Url, Arc<links::MdIndex>)> {
        let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let mut pairs: Vec<_> = cache
            .iter()
            .map(|entry| (entry.key().clone(), Arc::clone(&entry.index)))
            .collect();
        pairs.sort_unstable_by(|left, right| left.0.as_str().cmp(right.0.as_str()));
        pairs
    }
```

- [ ] **Step 2: Write the failing references tests**

Create `crates/lsp-poc/src/references/mod.rs` with the module doc, imports, and this
test module (implementation in Step 4):

```rust
//! The `textDocument/references` capability: who points at what.
//!
//! A heading answers with every link/wikilink that lands on it across
//! the candidate documents, each link resolved in its own document's
//! context. A link reference definition or footnote definition answers
//! with its same-document usages. A link or wikilink answers with its
//! target — web and unparseable destinations are excluded by the target
//! parsers. Absence is `None`.

use std::sync::Arc;

use async_language_server::lsp_types::{Location, Position as LspPosition, Url};
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{self, CursorItem, Target, item_at, parse_destination, parse_wiki_target, slugify};
use crate::workspace::Resolved;

/// One candidate document for heading-reference scans.
#[derive(Debug)]
pub struct DocumentSnapshot {
    pub url: Url,
    pub index: Arc<links::MdIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    fn parse(text: &str) -> Arc<MdIndex> {
        let mut parser = MarkdownParser::default();
        Arc::new(links::build(&mut parser, text).expect("input parses"))
    }

    fn self_doc() -> (Url, Arc<MdIndex>) {
        (
            Url::parse("file:///doc.md").expect("url parses"),
            parse("# Top\n\n[summary](#top)\n"),
        )
    }

    fn references_of(
        index: &MdIndex,
        self_url: &Url,
        line: u32,
        character: u32,
        documents: &[DocumentSnapshot],
    ) -> Option<Vec<Location>> {
        let resolve = |_: &Target| {
            Some(Resolved::Found {
                url: Url::parse("file:///target.md").expect("url parses"),
                index: parse("# Target\n"),
            })
        };
        let resolve_from = |candidate_url: &Url, _: &Target| {
            Some(Resolved::Found {
                url: candidate_url.clone(),
                index: parse("# Top\n"),
            })
        };
        at_position(
            index,
            self_url,
            LspPosition { line, character },
            documents,
            &resolve,
            &resolve_from,
        )
    }

    #[test]
    fn heading_references_collect_own_and_cross_file_links() {
        let (self_url, index) = self_doc();
        let other_url = Url::parse("file:///other.md").expect("url parses");
        let documents = vec![
            DocumentSnapshot {
                url: self_url.clone(),
                index: Arc::clone(&index),
            },
            DocumentSnapshot {
                url: other_url.clone(),
                index: parse("[x](doc.md#top) and [[doc#Top]]\n"),
            },
        ];
        // Cursor on the `# Top` heading (line 0).
        let locations =
            references_of(&index, &self_url, 0, 2, &documents).expect("references exist");
        assert_eq!(locations.len(), 3, "own link + 2 cross-file links");
        assert!(
            locations
                .iter()
                .any(|location| location.uri == self_url
                    && location.range.start.line == 2),
            "the own-document link is included",
        );
        assert!(
            locations
                .iter()
                .all(|location| location.uri != other_url
                    || location.range.start.line == 0),
            "cross-file locations point at the links' own lines",
        );
    }

    #[test]
    fn definition_and_footnote_definition_list_their_usages() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[label][ref]\n\n[ref]: a.md\n\nfoot[^1]\n\n[^1]: text\n");
        // Cursor on the `[ref]: a.md` definition (line 2).
        let references =
            references_of(&index, &url, 2, 2, &documents_empty()).expect("usages exist");
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].range.start.line, 0);
        // Cursor on the `[^1]: text` footnote definition (line 6).
        let footnote_refs =
            references_of(&index, &url, 6, 2, &documents_empty()).expect("usages exist");
        assert_eq!(footnote_refs.len(), 1);
        assert_eq!(footnote_refs[0].range.start.line, 4);
    }

    #[test]
    fn link_references_answer_with_their_target() {
        let (self_url, index) = self_doc();
        // Cursor inside `[summary](#top)` — answers with the heading.
        let locations = references_of(&index, &self_url, 2, 3, &documents_empty())
            .expect("the target resolves");
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].uri, self_url);
        assert_eq!(locations[0].range.start.line, 0);
    }

    #[test]
    fn web_link_has_no_references_here() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[site](https://example.com)\n");
        assert!(
            references_of(&index, &url, 0, 8, &documents_empty()).is_none(),
            "web destinations belong to the editor's open-link, not to us",
        );
    }

    #[test]
    fn cursor_on_plain_text_answers_none() {
        let (self_url, index) = self_doc();
        assert!(
            references_of(&index, &self_url, 4, 0, &documents_empty()).is_none(),
        );
    }

    fn documents_empty() -> Vec<DocumentSnapshot> {
        Vec::new()
    }
}
```

- [ ] **Step 3: Add `mod references;` and run the tests to verify they fail**

In `crates/lsp-poc/src/main.rs`, add `mod references;` between `mod definitions;` and
`mod server;`. Run: `cargo nextest run -p lsp-poc references::`
Expected: compile error — `at_position`, `DocumentSnapshot` not found.

- [ ] **Step 4: Implement `at_position`**

Add below `DocumentSnapshot` (before `#[cfg(test)]`):

```rust
/// Answers the references for the item under `position`, or `None` when
/// nothing reference-worthy is under it or no reference exists. `resolve`
/// answers targets in the requesting document's context; `resolve_from`
/// answers them in a candidate document's context.
pub fn at_position(
    index: &links::MdIndex,
    self_url: &Url,
    position: LspPosition,
    documents: &[DocumentSnapshot],
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
    resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
) -> Option<Vec<Location>> {
    let locations = match item_at(index, position)? {
        CursorItem::Heading(heading) => {
            heading_references(heading, self_url, documents, resolve_from)
        }
        CursorItem::Link(link) => {
            let target = parse_destination(&link.destination)?;
            target_location(index, self_url, &target, resolve)
        }
        CursorItem::Reference(reference) => {
            let definition = index
                .definitions()
                .iter()
                .find(|definition| definition.label == reference.label)?;
            vec![location(self_url, definition.range)]
        }
        CursorItem::Definition(definition) => index
            .references()
            .iter()
            .filter(|reference| reference.label == definition.label)
            .map(|reference| location(self_url, reference.range))
            .collect(),
        CursorItem::FootnoteReference(footnote) => {
            let definition = index
                .footnote_definitions()
                .iter()
                .find(|definition| definition.id == footnote.id)?;
            vec![location(self_url, definition.range)]
        }
        CursorItem::FootnoteDefinition(definition) => index
            .footnote_references()
            .iter()
            .filter(|footnote| footnote.id == definition.id)
            .map(|footnote| location(self_url, footnote.range))
            .collect(),
        CursorItem::Wikilink(wikilink) => {
            let target = parse_wiki_target(&wikilink.target)?;
            target_location(index, self_url, &target, resolve)
        }
    };
    (!locations.is_empty()).then_some(locations)
}

/// Every link/wikilink across the candidate documents whose target lands
/// on `heading` in the requesting document. Each candidate's links
/// resolve in that candidate's own context — a `#slug` link in document B
/// points at B's heading, not at ours.
fn heading_references(
    heading: &links::Heading,
    self_url: &Url,
    documents: &[DocumentSnapshot],
    resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
) -> Vec<Location> {
    let mut locations = Vec::new();
    for document in documents {
        for link in document.index.links() {
            let Some(target) = parse_destination(&link.destination) else {
                continue;
            };
            if lands_on(heading, self_url, &document.url, &target, resolve_from) {
                locations.push(location(&document.url, link.range));
            }
        }
        for wikilink in document.index.wikilinks() {
            let Some(target) = parse_wiki_target(&wikilink.target) else {
                continue;
            };
            if lands_on(heading, self_url, &document.url, &target, resolve_from) {
                locations.push(location(&document.url, wikilink.range));
            }
        }
    }
    locations
}

/// `true` when `target`, resolved in the candidate document's context,
/// lands on `heading` in the requesting document. Fragment-less file
/// targets reference a file, not a heading — not counted.
fn lands_on(
    heading: &links::Heading,
    self_url: &Url,
    candidate_url: &Url,
    target: &Target,
    resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
) -> bool {
    match target {
        Target::Fragment(fragment) => {
            candidate_url == self_url && slugify(fragment) == heading.slug
        }
        Target::Doc { fragment, .. } | Target::Wiki { fragment, .. } => {
            let Some(Resolved::Found { url, .. }) = resolve_from(candidate_url, target) else {
                return false;
            };
            url == self_url
                && fragment
                    .as_deref()
                    .is_some_and(|fragment| slugify(fragment) == heading.slug)
        }
    }
}

/// A link/wikilink's target as a one-element answer: fragments land on
/// the own document's matching heading; file targets on the resolved
/// document's matching heading, or its first heading, or its start.
fn target_location(
    index: &links::MdIndex,
    self_url: &Url,
    target: &Target,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Vec<Location> {
    match target {
        Target::Fragment(fragment) => index
            .heading_by_slug(&slugify(fragment))
            .map(|heading| location(self_url, heading.range))
            .into_iter()
            .collect(),
        Target::Doc { fragment, .. } | Target::Wiki { fragment, .. } => {
            let Some(Resolved::Found { url, index: target_index }) = resolve(target) else {
                return Vec::new();
            };
            let range = match fragment {
                Some(fragment) => target_index
                    .heading_by_slug(&slugify(fragment))
                    .map(|heading| heading.range)?,
                None => target_index
                    .headings()
                    .first()
                    .map_or_else(zero_range, |heading| heading.range),
            };
            vec![location(&url, range)]
        }
    }
}

fn location(url: &Url, range: async_language_server::tree_sitter::Range) -> Location {
    Location {
        uri: url.clone(),
        range: ts_range_to_lsp_range(range),
    }
}

/// The document-start range, for file targets without a fragment.
fn zero_range() -> async_language_server::tree_sitter::Range {
    async_language_server::tree_sitter::Range {
        start_byte: 0,
        end_byte: 0,
        start_point: async_language_server::tree_sitter::Point { row: 0, column: 0 },
        end_point: async_language_server::tree_sitter::Point { row: 0, column: 0 },
    }
}
```

Notes for the implementer:
- `zero_range()` is deliberately duplicated from `definitions/mod.rs` (private there;
  the sibling rule forbids importing it) — two small private fns with different
  return consumers; do NOT create a shared module for two 7-line helpers.
- `Index::snapshot()` pairs are `(Url, Arc<MdIndex>)`; the server constructs
  `DocumentSnapshot`s — workspace never imports references-layer types.
- If `definitions`' tests break: they must not — Task 1 switched them to `item_at`
  with behavior-preserving arms; investigate before touching anything.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc references::`
Expected: 5 passed.
Run: `cargo nextest run -p lsp-poc`
Expected: full suite green.

- [ ] **Step 6: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 3: Server wiring + docs tails

**Files:**
- Modify: `crates/lsp-poc/src/server.rs` (capability, `references` method, snapshot helper)
- Modify: `CLAUDE.md` (capability line)
- Modify: `docs/superpowers/specs/2026-09-13-md-model-hardening-definition-design.md` (fixture-wording note)

**Interfaces:**
- Consumes: Task 2's `references::{at_position, DocumentSnapshot}` and `Index::snapshot()`; Task 1's `item_at` (via definitions).
- Produces: `textDocument/references` served end-to-end.

- [ ] **Step 1: Capability + method + snapshot helper in `src/server.rs`**

Capability (after `definition_provider`):

```rust
            references_provider: Some(OneOf::Left(true)),
```

Imports: add `ReferenceParams`, `References` is not needed (method name is
`references`), plus `crate::references::{self, DocumentSnapshot}` — extend the
existing use lists alphabetically.

Snapshot helper (below `resolver`):

```rust
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
```

Trait method (next to `definition`; eager compute, same shape):

```rust
    fn references(
        &self,
        state: ServerState,
        params: ReferenceParams,
    ) -> impl Future<Output = ServerResult<Option<Vec<async_lsp_locations::Location>>>> + Send
```

— concretely, with real types:

```rust
    fn references(
        &self,
        state: ServerState,
        params: ReferenceParams,
    ) -> impl Future<Output = ServerResult<Option<Vec<async_language_server::lsp_types::Location>>>> + Send
    {
        ready(self.references_at(&state, &params))
    }
```

and below the trait impl:

```rust
    /// Answers one `textDocument/references` request. Absence is a normal
    /// `None`: no document, no parse, nothing reference-worthy under the
    /// position, or zero matches.
    fn references_at(
        &self,
        state: &ServerState,
        params: &ReferenceParams,
    ) -> ServerResult<Option<Vec<async_language_server::lsp_types::Location>>> {
        let url = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(doc) = state.document(&url) else {
            return Ok(None);
        };
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no references for {url}");
            return Ok(None);
        };
        let documents = self.document_snapshots(state);
        let resolve = self.resolver(state, &url);
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
            &url,
            position,
            &documents,
            &resolve,
            &resolve_from,
        ))
    }
```

`ReferenceParams` must be added to the `lsp_types` import list.

- [ ] **Step 2: CLAUDE.md capability line**

old:
```
  - `server_capabilities()` declares what the server supports (currently hover and diagnostics)
```
new:
```
  - `server_capabilities()` declares what the server supports (currently hover, diagnostics, definition, and references)
```

- [ ] **Step 3: Cycle-2 spec wording note**

In `docs/superpowers/specs/2026-09-13-md-model-hardening-definition-design.md`, in the
`crates/lsp-poc/tests/fixtures/*` bullet, append one sentence (dated):

```
(Delivered 2026-09-14 as inline `parse()` strings in the definitions module tests
rather than a fixture file — the cases were single-line micro-inputs.)
```

- [ ] **Step 4: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test && make dupes`
Expected: exit 0 across all five. There is no unit-test leg for the hooks (framework
`#[cfg(test)]` only) — live verification is Task 4.

---

### Task 4: Battery, live verification on both clients, whole-cycle final review

**Files:**
- No file changes (verification; controller-run).

- [ ] **Step 1: Battery + deny + build**

Run: `make battery` → exit 0 AND **zero `^warning` lines** (the cycle-2 discipline).
Run: `make deny` → exit 2 with exactly the one ratified wildcard.
Run: `make build-poc` → fresh binary for both live clients.

- [ ] **Step 2: Live verification**

Owner: in Zed, References (shift-F12) on a heading in a note — inbound links across
notes appear. Claude Code: `/reload-plugins`, then `findReferences` on the fixture
heading (`# Top` in `links-fixture.md`) — "метод вже так": the own-document link is
reported; on `[summary](#top)` definition still answers 1:1 (regression check).

- [ ] **Step 3: Whole-cycle final review**

Dispatch the final reviewer (`opus[1m]`, rust-skills + LSP, runs battery itself) with
the whole-cycle package (BASE = the cycle-3 start commit) and the ledger's open-minors
list; triage its findings; close the cycle.
