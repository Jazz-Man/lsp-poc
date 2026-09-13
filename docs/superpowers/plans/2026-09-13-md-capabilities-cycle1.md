# Markdown Cycle 1 — Link Model + Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parse Markdown documents into a tree-sitter link model, resolve link targets across the workspace, and publish broken-link diagnostics (codes 1–6) to Zed — cycle 1 of the roadmap in `docs/superpowers/specs/2026-09-13-md-capabilities-cycle1-diagnostics-design.md`.

**Architecture:** Three new modules under scopes already declared in `arch-lint.toml`: `src/links/` (pure model: headings, definitions, links, references, footnotes, wikilinks, parsed via `tree_sitter_md::MarkdownParser` — block + inline grammars), `src/workspace/` (cross-file index: on-demand disk reads with mtime+size stamp cache, git-root discovery, `resolve()`), `src/diagnostics/` (`compute()` mapping model + resolution to LSP diagnostics). `server.rs` wires: capabilities, `did_open`/`did_change` publish hooks, `document_diagnostics` pull parity.

**Tech Stack:** Rust edition 2024, stable, `async-language-server` v0.10.0 (`Server` trait, `ServerState`, `lsp_types`/`tree_sitter`/`tree_sitter_utils` re-exports), `tree-sitter-md` 0.5.3 with `parser` feature (`MarkdownParser` — already a dependency, **no new deps**), `tracing`, `cargo nextest`, Makefile battery.

## Global Constraints

- Git is read-only for every agent: NO `git add`, `git commit`, `git rm`, or any git-write command. The owner commits. (This plan has no commit steps by design.)
- Zero new dependencies; zero new `PocError` variants; no `error.rs` is created (absence of a target is a diagnostic, not an error — spec decision).
- Never print to stdout (LSP transport). Logging = `tracing` macros to stderr only; include the error value when warning (`tracing::warn!(%error, ...)`) — never swallow a fallible call silently.
- Mutex poisoning recovers honestly: `lock().unwrap_or_else(PoisonError::into_inner)` — no `.ok()?` swallows on locks.
- Sync `std::fs` calls in `src/` trip the arch-lint `NoSyncIo` rule: each site needs `// arch-lint: allow(no-sync-io) reason="..."` (the framework's own code does exactly this).
- Notification hooks are synchronous by protocol necessity ("may not await and must not panic") — no `.await`, no `panic!`/`unwrap`/`expect` in hooks; `unwrap`/`expect` exist only in `#[cfg(test)]` code (clippy gates are test-aware via `clippy.toml`).
- Positions are UTF-8 at the trait boundary; convert tree-sitter ranges only via `async_language_server::tree_sitter_utils::ts_range_to_lsp_range`.
- No `as` casts for narrowing conversions (clippy pedantic denies them); use `TryFrom`/`usize` arithmetic (tree-sitter `Range`/`Point` are `usize`-based; `ts_range_to_lsp_range` handles the `u32` edge).
- Surgical: the existing `hover` stays byte-identical; the Zed extension crate is untouched; `arch-lint.toml` scopes stay untouched (only the header comment is refreshed in Task 5).
- Rust work follows the `rust-skills` skill rules (err-\*, own-\*, pat-let-else, api-must-use, obs-structured-fields, test-cfg-test-module).
- Every task ends with `make fmt-fix && make fmt && make clippy && make test` green (called "the task gates").

---

### Task 1: `src/links/` — pure Markdown link model

**Files:**
- Create: `crates/lsp-poc/src/links/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod links;`)

**Interfaces:**
- Consumes: `tree_sitter_md::MarkdownParser` (crate dep, `parser` feature); `async_language_server::tree_sitter::{Node, Point, Range}`.
- Produces (Tasks 2–4 rely on these exact names):
  - `MdIndex` (`Debug + Default`), built by `pub fn build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex>`
  - accessors: `links(&self) -> &[Link]`, `references(&self) -> &[Reference]`, `footnote_references(&self) -> &[FootnoteReference]`, `wikilinks(&self) -> &[Wikilink]`, `has_heading_slug(&self, slug: &str) -> bool`, `has_definition(&self, label: &str) -> bool`, `has_footnote_definition(&self, id: &str) -> bool`
  - types: `Heading { slug: String, level: u8, text: String, range: Range }`, `Definition { label: String, destination: String, range: Range }`, `Link { destination: String, range: Range }`, `Reference { label: String, range: Range }`, `FootnoteReference { id: String, range: Range }`, `FootnoteDefinition { id: String, range: Range }`, `Wikilink { target: String, range: Range }` (all `Debug`; `range` is a tree-sitter `Range` in document coordinates)
  - `#[derive(Debug, PartialEq, Eq)] pub enum Target { Fragment(String), Doc { path: String, fragment: Option<String> }, Wiki { path: String, fragment: Option<String> } }`
  - `pub fn parse_destination(destination: &str) -> Option<Target>` (None = external/empty), `pub fn slugify(heading: &str) -> String` (lowercase + spaces → `-`, the reference's normalization)

- [ ] **Step 1: Create `crates/lsp-poc/src/links/mod.rs` with types and a failing characterization test**

Create the file with the module doc, imports, type definitions, and a `#[cfg(test)] mod tests` containing `FIXTURE` and the tests from Step 3 (they fail to compile — `build` etc. do not exist yet). Skeleton:

```rust
//! Pure Markdown link model: what a document contains and what its links
//! point at, parsed with tree-sitter's block + inline grammars.
//!
//! Headings, link reference definitions, and inline/reference links come
//! from typed grammar nodes. Footnotes and wikilinks have no nodes in
//! tree-sitter-md, so they are scanned off-tree over inline text. Known
//! limitation: that scan also matches shapes inside code spans — the
//! grammars give no cheaper boundary; refine when a cycle needs it.

use async_language_server::tree_sitter::{Node, Point, Range};
use tree_sitter_md::{MarkdownParser, MarkdownTree};

/// Everything the diagnostics need to know about one parsed document.
#[derive(Debug, Default)]
pub struct MdIndex {
    headings: Vec<Heading>,
    definitions: Vec<Definition>,
    links: Vec<Link>,
    references: Vec<Reference>,
    footnote_references: Vec<FootnoteReference>,
    footnote_definitions: Vec<FootnoteDefinition>,
    wikilinks: Vec<Wikilink>,
}

/// A GitHub-style heading anchor: lowercase, spaces folded to dashes.
#[derive(Debug)]
pub struct Heading {
    pub slug: String,
    pub level: u8,
    pub text: String,
    pub range: Range,
}

/// A link reference definition: `[label]: destination`.
#[derive(Debug)]
pub struct Definition {
    pub label: String,
    pub destination: String,
    pub range: Range,
}

/// An inline link `[text](destination)` (images are not collected in cycle 1).
#[derive(Debug)]
pub struct Link {
    pub destination: String,
    pub range: Range,
}

/// A full, collapsed, or shortcut reference link's label.
#[derive(Debug)]
pub struct Reference {
    pub label: String,
    pub range: Range,
}

/// A footnote reference `[^id]`.
#[derive(Debug)]
pub struct FootnoteReference {
    pub id: String,
    pub range: Range,
}

/// A footnote definition `[^id]: text` (first inline of a paragraph).
#[derive(Debug)]
pub struct FootnoteDefinition {
    pub id: String,
    pub range: Range,
}

/// A wikilink `[[target]]` (Obsidian-style; not standard Markdown).
#[derive(Debug)]
pub struct Wikilink {
    pub target: String,
    pub range: Range,
}

/// Where a parsed destination points.
#[derive(Debug, PartialEq, Eq)]
pub enum Target {
    /// `#fragment` — the same document.
    Fragment(String),
    /// `path` or `path#fragment` — another file.
    Doc {
        path: String,
        fragment: Option<String>,
    },
    /// `[[path]]` or `[[path#fragment]]`.
    Wiki {
        path: String,
        fragment: Option<String>,
    },
}
```

(The test module and the functions below are added in Steps 1 and 4; the tests must not compile until the functions exist — that is the failing state.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p lsp-poc links::`
Expected: compile error (`build`, `parse_destination`, `slugify` not found) — add `mod links;` to `main.rs` first (after `mod hovers;`, alphabetical):

```rust
mod diagnostics;
mod hovers;
mod links;
mod server;
mod tracing;
mod workspace;
```

Wait — `mod diagnostics;` and `mod workspace;` do not exist until Tasks 2–3; in THIS task only add the `mod links;` line between `mod hovers;` and `mod server;`. The final ordering above is reached in Task 3.

Expected after adding `mod links;`: compile error about missing `build`/`parse_destination`/`slugify` in `links`.

- [ ] **Step 3: Write the failing tests (characterization + unit)**

Append to `src/links/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = concat!(
        "# Top\n",
        "\n",
        "See [docs](guide.md) and [summary](#top).\n",
        "\n",
        "Ref [label][ref] and [collapsed][] and [shortcut].\n",
        "\n",
        "A footnote[^1] here.\n",
        "\n",
        "[^1]: footnote text\n",
        "\n",
        "Wiki [[Other]] and [[folder/note#Section]].\n",
        "\n",
        "[ref]: https://example.com\n",
    );

    fn fixture() -> MdIndex {
        let mut parser = MarkdownParser::default();
        build(&mut parser, FIXTURE).expect("fixture parses")
    }

    #[test]
    fn fixture_collects_headings_definitions_and_links() {
        let index = fixture();
        assert_eq!(index.headings.len(), 1);
        assert_eq!(index.headings[0].slug, "top");
        assert_eq!(index.headings[0].level, 1);
        assert!(index.has_heading_slug("top"));
        assert!(!index.has_heading_slug("ghost"));
        assert!(index.has_definition("ref"));
        assert!(!index.has_definition("missing"));
        assert_eq!(index.links.len(), 2);
        assert!(index
            .links
            .iter()
            .any(|link| link.destination == "guide.md"));
    }

    #[test]
    fn fixture_collects_references_footnotes_and_wikilinks() {
        let index = fixture();
        let mut labels: Vec<&str> =
            index.references.iter().map(|r| r.label.as_str()).collect();
        labels.sort_unstable();
        assert_eq!(labels, vec!["collapsed", "label", "shortcut"]);
        assert!(index.has_footnote_definition("1"));
        assert_eq!(index.footnote_references.len(), 1);
        assert_eq!(index.footnote_references[0].id, "1");
        let targets: Vec<&str> =
            index.wikilinks.iter().map(|w| w.target.as_str()).collect();
        assert_eq!(targets, vec!["Other", "folder/note#Section"]);
    }

    #[test]
    fn scanned_ranges_slice_back_to_their_source_text() {
        let index = fixture();
        let range = index.footnote_references[0].range;
        assert_eq!(&FIXTURE[range.start_byte..range.end_byte], "[^1]");
        assert_eq!(
            &FIXTURE[index.wikilinks[1].range.start_byte..index.wikilinks[1].range.end_byte],
            "[[folder/note#Section]]",
        );
    }

    #[test]
    fn parse_destination_classifies_targets() {
        assert_eq!(parse_destination("#top"), Some(Target::Fragment("top".to_owned())));
        assert_eq!(
            parse_destination("guide.md"),
            Some(Target::Doc { path: "guide.md".to_owned(), fragment: None }),
        );
        assert_eq!(
            parse_destination("a/b.md#section"),
            Some(Target::Doc {
                path: "a/b.md".to_owned(),
                fragment: Some("section".to_owned()),
            }),
        );
        assert_eq!(parse_destination("https://example.com"), None);
        assert_eq!(parse_destination("mailto:x@y.z"), None);
        assert_eq!(parse_destination(""), None);
    }

    #[test]
    fn slugify_matches_the_reference_normalization() {
        assert_eq!(slugify("Heading One"), "heading-one");
        assert_eq!(slugify("  Mixed CASE "), "mixed-case");
    }
}
```

**Characterization note (pre-authorized adjustment):** the exact shapes tree-sitter-md 0.5.3 produces for `[^1]` and `[[Other]]` inside inline content are pinned by these assertions. If the run shows extra `references` entries (e.g. `^1` or `[[other]]`), the fix is the guard in Step 4's `collect_inline` (skip link-text starting with `[` or `^`) — adjust the guard, never the expectation: footnote/wikilink syntax must not produce `Reference` entries.

- [ ] **Step 4: Implement the model**

Add below the types (before `#[cfg(test)]`):

```rust
impl MdIndex {
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.references
    }

    #[must_use]
    pub fn footnote_references(&self) -> &[FootnoteReference] {
        &self.footnote_references
    }

    #[must_use]
    pub fn has_heading_slug(&self, slug: &str) -> bool {
        self.headings.iter().any(|heading| heading.slug == slug)
    }

    #[must_use]
    pub fn has_definition(&self, label: &str) -> bool {
        self.definitions
            .iter()
            .any(|definition| definition.label == label)
    }

    #[must_use]
    pub fn has_footnote_definition(&self, id: &str) -> bool {
        self.footnote_definitions
            .iter()
            .any(|definition| definition.id == id)
    }
}

/// Parses `text` into an [`MdIndex`] with both grammars.
///
/// Returns `None` when tree-sitter reports a timeout or cancellation —
/// callers treat that as "nothing to say this round".
#[must_use]
pub fn build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex> {
    let tree = parser.parse(text.as_bytes(), None)?;
    let mut index = MdIndex::default();
    collect_block(tree.block_tree().root_node(), text, &mut index);
    for inline in tree.inline_trees() {
        collect_inline(inline.root_node(), text, &mut index);
        scan_offtree(inline.root_node(), text, &mut index);
    }
    Some(index)
}

/// Classifies a link destination; `None` means "nothing to check"
/// (empty, `scheme://…`, `mailto:`).
#[must_use]
pub fn parse_destination(destination: &str) -> Option<Target> {
    let trimmed = destination.trim();
    if trimmed.is_empty() || trimmed.contains("://") || trimmed.starts_with("mailto:") {
        return None;
    }
    match trimmed.split_once('#') {
        Some(("", "")) | None => {
            let path = trimmed.strip_suffix('#').unwrap_or(trimmed);
            Some(Target::Doc { path: path.to_owned(), fragment: None })
        }
        Some(("", fragment)) => Some(Target::Fragment(fragment.to_owned())),
        Some((path, "")) => Some(Target::Doc { path: path.to_owned(), fragment: None }),
        Some((path, fragment)) => Some(Target::Doc {
            path: path.to_owned(),
            fragment: Some(fragment.to_owned()),
        }),
    }
}

/// GitHub-style anchor: lowercase, spaces → dashes (the reference's
/// normalization, not a full slugger — accepted POC divergence).
#[must_use]
pub fn slugify(heading: &str) -> String {
    heading.trim().to_lowercase().replace(' ', "-")
}

/// CommonMark label identity: trim, lowercase, collapse whitespace runs.
fn normalize_label(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn collect_block(node: Node, text: &str, index: &mut MdIndex) {
    match node.kind() {
        "atx_heading" => collect_heading(node, text, index),
        "link_reference_definition" => collect_definition(node, text, index),
        "paragraph" => collect_footnote_definition(node, text, index),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_block(child, text, index);
    }
}

fn collect_heading(heading: Node, text: &str, index: &mut MdIndex) {
    let Some(content) = heading.child_by_field_name("heading_content") else {
        return;
    };
    let mut cursor = heading.walk();
    let Some(marker) = heading
        .children(&mut cursor)
        .find(|child| child.kind().starts_with("atx_h"))
    else {
        return;
    };
    let Some(level) = heading_level(&marker) else {
        return;
    };
    let Ok(raw) = content.utf8_text(text.as_bytes()) else {
        return;
    };
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
        text: raw.trim().to_owned(),
        range: heading.range(),
    });
}

fn heading_level(marker: &Node) -> Option<u8> {
    marker
        .kind()
        .strip_prefix("atx_h")?
        .strip_suffix("_marker")?
        .parse()
        .ok()
}

fn collect_definition(definition: Node, text: &str, index: &mut MdIndex) {
    let mut label = None;
    let mut destination = None;
    let mut cursor = definition.walk();
    for child in definition.children(&mut cursor) {
        match child.kind() {
            "link_label" => label = bracketed_text(child, text),
            "link_destination" => destination = clean_destination(child, text),
            _ => {}
        }
    }
    let (Some(label), Some(destination)) = (label, destination) else {
        return;
    };
    index.definitions.push(Definition {
        label: normalize_label(&label),
        destination,
        range: definition.range(),
    });
}

/// A paragraph whose first inline starts with `[^id]:` defines a footnote.
fn collect_footnote_definition(paragraph: Node, text: &str, index: &mut MdIndex) {
    let mut cursor = paragraph.walk();
    let Some(first) = paragraph
        .children(&mut cursor)
        .find(|child| child.kind() == "inline")
    else {
        return;
    };
    let Ok(raw) = first.utf8_text(text.as_bytes()) else {
        return;
    };
    let Some(rest) = raw.strip_prefix("[^") else {
        return;
    };
    let Some((id, after)) = rest.split_once(']') else {
        return;
    };
    if !after.starts_with(':') || id.is_empty() {
        return;
    }
    index.footnote_definitions.push(FootnoteDefinition {
        id: id.to_owned(),
        range: paragraph.range(),
    });
}

fn collect_inline(root: Node, text: &str, index: &mut MdIndex) {
    walk(root, &mut |node| match node.kind() {
        "inline_link" => {
            let mut cursor = node.walk();
            let Some(destination) = node
                .children(&mut cursor)
                .find(|child| child.kind() == "link_destination")
            else {
                return;
            };
            let Some(cleaned) = clean_destination(destination, text) else {
                return;
            };
            index.links.push(Link { destination: cleaned, range: node.range() });
        }
        "full_reference_link" => {
            let mut cursor = node.walk();
            let Some(label) = node
                .children(&mut cursor)
                .find(|child| child.kind() == "link_label")
            else {
                return;
            };
            let Some(raw) = bracketed_text(label, text) else {
                return;
            };
            index.references.push(Reference {
                label: normalize_label(&raw),
                range: node.range(),
            });
        }
        "collapsed_reference_link" | "shortcut_link" => {
            let mut cursor = node.walk();
            let Some(link_text) = node
                .children(&mut cursor)
                .find(|child| child.kind() == "link_text")
            else {
                return;
            };
            let Ok(raw) = link_text.utf8_text(text.as_bytes()) else {
                return;
            };
            // Wikilink and footnote shapes parse as these node kinds too;
            // they belong to the off-tree scan, not to reference links.
            if raw.starts_with('[') || raw.starts_with('^') {
                return;
            }
            index.references.push(Reference {
                label: normalize_label(raw),
                range: node.range(),
            });
        }
        // Images are not diagnosed in cycle 1 (the reference skips them too).
        "image" => {}
        _ => {}
    });
}

/// Scans raw inline text for footnote references and wikilinks — shapes
/// the grammars do not model.
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex) {
    if root.kind() != "inline" {
        return;
    }
    let Ok(source) = root.utf8_text(text.as_bytes()) else {
        return;
    };
    let base = (root.start_byte(), root.start_point());
    scan_footnote_references(source, base, index);
    scan_wikilinks(source, base, index);
}

fn scan_footnote_references(source: &str, base: (usize, Point), index: &mut MdIndex) {
    let mut at = 0;
    while let Some(offset) = source[at..].find("[^") {
        let start = at + offset;
        let Some(relative) = source[start + 2..].find(']') else {
            break;
        };
        let id = &source[start + 2..start + 2 + relative];
        let followed_by_colon = source[start + 2 + relative..].strip_prefix(']');
        let is_definition_marker =
            followed_by_colon.is_some_and(|rest| rest.starts_with(':'));
        at = start + 2 + relative + 1;
        if id.is_empty() || id.contains('[') || is_definition_marker {
            continue;
        }
        index.footnote_references.push(FootnoteReference {
            id: id.to_owned(),
            range: scan_range(base, source, start, relative + 3),
        });
    }
}

fn scan_wikilinks(source: &str, base: (usize, Point), index: &mut MdIndex) {
    let mut at = 0;
    while let Some(offset) = source[at..].find("[[") {
        let start = at + offset;
        let Some(relative) = source[start + 2..].find("]]") else {
            break;
        };
        let content = &source[start + 2..start + 2 + relative];
        at = start + 2;
        if content.is_empty() || content.contains('[') {
            continue;
        }
        index.wikilinks.push(Wikilink {
            target: content.trim().to_owned(),
            range: scan_range(base, source, start, relative + 4),
        });
    }
}

/// Builds a tree-sitter `Range` for a byte window inside an inline node's
/// text, composing the node's absolute base byte and point. Columns are
/// bytes, matching tree-sitter's convention.
fn scan_range(base: (usize, Point), source: &str, start: usize, len: usize) -> Range {
    let before = &source[..start];
    let rows = before.matches('\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let start_point = Point {
        row: base.1.row + rows,
        column: base.1.column + (start - line_start),
    };
    let inside = &source[start..start + len];
    let extra_rows = inside.matches('\n').count();
    let end_point = if extra_rows == 0 {
        Point { row: start_point.row, column: start_point.column + len }
    } else {
        let last_line = inside.rfind('\n').map_or(0, |i| i + 1);
        Point {
            row: start_point.row + extra_rows,
            column: inside.len() - last_line,
        }
    };
    Range {
        start_byte: base.0 + start,
        end_byte: base.0 + start + len,
        start_point,
        end_point,
    }
}

fn bracketed_text(node: Node, text: &str) -> Option<String> {
    let raw = node.utf8_text(text.as_bytes()).ok()?;
    let stripped = raw.strip_prefix('[').and_then(|rest| rest.strip_suffix(']'));
    Some(stripped.unwrap_or(raw).trim().to_owned())
}

fn clean_destination(node: Node, text: &str) -> Option<String> {
    let raw = node.utf8_text(text.as_bytes()).ok()?;
    let trimmed = raw.trim();
    let unwrapped = trimmed
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'));
    Some(unwrapped.unwrap_or(trimmed).to_owned())
}

fn walk(node: Node, visit: &mut dyn FnMut(Node)) {
    visit(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, visit);
    }
}
```

Note: `MdIndex::wikilinks` accessor is not needed by diagnostics in cycle 1 but keep fields private and add `wikilinks()` alongside the others for symmetry? — NO (YAGNI): only the accessors listed in **Interfaces**. If clippy `must_use`-candidates fires on `build`/`parse_destination`/`slugify`/accessors, the `#[must_use]` attributes above already satisfy it.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc links::`
Expected: 5 passed. If a characterization assertion fails, apply the pre-authorized guard adjustment from Step 3 and re-run.

- [ ] **Step 6: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 2: `src/workspace/` — cross-file index and target resolution

**Files:**
- Create: `crates/lsp-poc/src/workspace/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod workspace;` after `mod tracing;`)

**Interfaces:**
- Consumes: Task 1's `links::{build, MdIndex, Target}` (exact names above); `async_language_server::lsp_types::Url`.
- Produces (Tasks 3–4 rely on these):
  - `#[derive(Debug)] pub enum Resolved { Found(Arc<MdIndex>), Missing }` (no `PartialEq` — `Arc<MdIndex>` compares by pointer semantics; tests pattern-match)
  - `pub struct Index` — `Index::new()`, `pub fn resolve(&self, open: &dyn Fn(&Url) -> Option<Arc<MdIndex>>, self_url: &Url, target: &Target) -> Option<Resolved>` (None = "not applicable", only for `Target::Fragment`), `pub fn reset_root(&self)`

- [ ] **Step 1: Write the failing tests**

Create `crates/lsp-poc/src/workspace/mod.rs` with module doc, imports, and this test module (implementation comes in Step 3 — compile fails first):

```rust
//! Cross-file index: resolves link targets to workspace files.
//!
//! Targets resolve against open documents first (the closure the server
//! wires to its document store), then against the disk through a
//! mtime+size-stamped cache. The workspace root for wikilinks is the
//! nearest ancestor holding `.git` (Zed worktrees are git repos),
//! discovered once and reset by `reset_root` on folder changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use async_language_server::lsp_types::Url;
use tree_sitter_md::MarkdownParser;

use crate::links::{self, Target};

/// (modification time, size in bytes) — any doubt re-reads.
type FileStamp = (SystemTime, u64);

#[derive(Debug)]
struct Entry {
    stamp: Option<FileStamp>,
    index: Arc<links::MdIndex>,
}

/// What a resolved target turned out to be.
#[derive(Debug)]
pub enum Resolved {
    /// The target file exists (or is open); its index is attached.
    Found(Arc<links::MdIndex>),
    /// No candidate path exists.
    Missing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lsp-poc-ws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir creates");
        dir
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent dir creates");
        }
        std::fs::write(path, contents).expect("fixture writes");
    }

    fn parse(text: &str) -> Arc<links::MdIndex> {
        let mut parser = MarkdownParser::default();
        Arc::new(links::build(&mut parser, text).expect("fixture parses"))
    }

    #[test]
    fn resolve_follows_doc_relative_and_wiki_paths() {
        let root = temp_dir("resolve");
        std::fs::create_dir_all(root.join(".git")).expect(".git dir creates");
        write(&root.join("a/doc.md"), "# Doc\n");
        write(&root.join("a/b.md"), "# B\n");
        write(&root.join("c.md"), "# C\n");

        let index = Index::new();
        let doc_url = Url::from_file_path(root.join("a/doc.md")).expect("doc url");
        let no_open = |_: &Url| None;

        let relative = index.resolve(
            &no_open,
            &doc_url,
            &parse_destination("b.md").expect("target"),
        );
        assert!(matches!(relative, Some(Resolved::Found(_))));

        assert!(matches!(
            index.resolve(&no_open, &doc_url, &parse_destination("nope.md").expect("target")),
            Some(Resolved::Missing),
        ));
        assert!(
            index
                .resolve(&no_open, &doc_url, &parse_destination("#doc").expect("target"))
                .is_none(),
            "fragments are the caller's job",
        );
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki { path: "c".to_owned(), fragment: None },
            ),
            Some(Resolved::Found(_)),
        ));
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki { path: "ghost".to_owned(), fragment: None },
            ),
            Some(Resolved::Missing),
        ));

        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }

    #[test]
    fn resolve_prefers_open_documents_over_disk() {
        let root = temp_dir("open-preference");
        std::fs::create_dir_all(root.join(".git")).expect(".git dir creates");
        write(&root.join("doc.md"), "# Disk\n");
        let doc_url = Url::from_file_path(root.join("doc.md")).expect("doc url");
        let open_text = "# Open only\n";
        let open_index = parse(open_text);
        let url_for_open = doc_url.clone();
        let open = move |url: &Url| {
            (url == &url_for_open).then(|| open_index.clone())
        };

        let index = Index::new();
        let Some(Resolved::Found(found)) = index.resolve(
            &open,
            &doc_url,
            &parse_destination("doc.md#open-only").expect("target"),
        ) else {
            panic!("the open document must resolve");
        };
        assert!(found.has_heading_slug("open-only"));

        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }

    #[test]
    fn resolve_returns_none_without_a_git_root_for_wikilinks() {
        let root = temp_dir("no-root");
        write(&root.join("doc.md"), "# Doc\n");
        let doc_url = Url::from_file_path(root.join("doc.md")).expect("doc url");
        let index = Index::new();
        let no_open = |_: &Url| None;
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki { path: "x".to_owned(), fragment: None },
            ),
            Some(Resolved::Missing),
        ));
        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }
}
```

Also add `use crate::links::parse_destination;` to the test module's `use super::*` scope — include it as `use super::*;` plus `use crate::links::parse_destination;` at the top of `mod tests`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p lsp-poc workspace::` (after adding `mod workspace;` to `main.rs`, after `mod tracing;`)
Expected: compile error — `Index`, `Resolved`, `resolve`, `reset_root` not defined.

- [ ] **Step 3: Implement `Index`**

Add below `Resolved` (before `#[cfg(test)]`):

```rust
/// Cache of parsed workspace files plus the discovered workspace root.
///
/// The index owns its own parser so the server's parser and the index's
/// never contend for one Mutex.
#[derive(Debug)]
pub struct Index {
    parser: Mutex<MarkdownParser>,
    cache: Mutex<HashMap<Url, Entry>>,
    root: Mutex<Option<PathBuf>>,
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parser: Mutex::new(MarkdownParser::default()),
            cache: Mutex::new(HashMap::new()),
            root: Mutex::new(None),
        }
    }

    /// Forgets the discovered workspace root (folder changes).
    pub fn reset_root(&self) {
        *self.root.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// Resolves `target` to a file. Returns `None` only for
    /// [`Target::Fragment`] — same-document targets are the caller's job.
    /// Open documents (via `open`) win over the disk.
    pub fn resolve(
        &self,
        open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>,
        self_url: &Url,
        target: &Target,
    ) -> Option<Resolved> {
        match target {
            Target::Fragment(_) => None,
            Target::Doc { path, .. } => {
                Some(self.load(open, &self.doc_candidates(self_url, path)))
            }
            Target::Wiki { path, .. } => {
                Some(self.load(open, &self.wiki_candidates(self_url, path)))
            }
        }
    }

    fn load(&self, open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>, candidates: &[PathBuf]) -> Resolved {
        for path in candidates {
            let Ok(url) = Url::from_file_path(path) else {
                continue;
            };
            if let Some(index) = open(&url) {
                return Resolved::Found(index);
            }
            if let Some(index) = self.load_from_disk(&url, path) {
                return Resolved::Found(index);
            }
        }
        Resolved::Missing
    }

    fn load_from_disk(&self, url: &Url, path: &Path) -> Option<Arc<links::MdIndex>> {
        let stamp = file_stamp(path)?;
        {
            let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(entry) = cache.get(url) {
                if entry.stamp == Some(stamp) {
                    return Some(Arc::clone(&entry.index));
                }
            }
        }
        // arch-lint: allow(no-sync-io) reason="link targets load on demand inside synchronous notification hooks, which the protocol keeps sync"
        let text = std::fs::read_to_string(path).ok()?;
        let index = {
            let mut parser = self.parser.lock().unwrap_or_else(PoisonError::into_inner);
            links::build(&mut parser, &text)?
        };
        let index = Arc::new(index);
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(url.clone(), Entry { stamp: Some(stamp), index: Arc::clone(&index) });
        Some(index)
    }

    fn doc_candidates(&self, self_url: &Url, path: &str) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(self_path) = self_url.to_file_path() {
            if let Some(dir) = self_path.parent() {
                candidates.push(dir.join(path));
            }
        }
        if let Some(root) = self.root_for(self_url) {
            candidates.push(root.join(path));
        }
        candidates
    }

    fn wiki_candidates(&self, self_url: &Url, path: &str) -> Vec<PathBuf> {
        let Some(root) = self.root_for(self_url) else {
            return Vec::new();
        };
        let base = root.join(path);
        if path.ends_with(".md") {
            vec![base]
        } else {
            vec![base, root.join(format!("{path}.md"))]
        }
    }

    fn root_for(&self, self_url: &Url) -> Option<PathBuf> {
        {
            let root = self.root.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(root) = *root {
                return Some(root);
            }
        }
        let discovered = git_root(&self_url.to_file_path().ok()?)?;
        *self.root.lock().unwrap_or_else(PoisonError::into_inner) = Some(discovered.clone());
        Some(discovered)
    }
}

/// Nearest ancestor directory holding `.git` (a dir or a worktree file).
fn git_root(from: &Path) -> Option<PathBuf> {
    from.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    // arch-lint: allow(no-sync-io) reason="stamp probes ride the same synchronous hook context as the reads they gate"
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc workspace::`
Expected: 3 passed.

- [ ] **Step 5: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 3: `src/diagnostics/` — broken-link diagnostics (codes 1–6)

**Files:**
- Create: `crates/lsp-poc/src/diagnostics/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod diagnostics;` as the first module, alphabetical)

**Interfaces:**
- Consumes: Task 1's `links::{MdIndex, Target, parse_destination, slugify}` + accessor methods; Task 2's `Resolved`; `async_language_server::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString}`; `async_language_server::tree_sitter_utils::ts_range_to_lsp_range`.
- Produces (Task 4 relies on this):
  - `pub const SOURCE: &str = "lsp-poc";`
  - `pub fn compute(index: &links::MdIndex, resolve: &dyn Fn(&Target) -> Option<Resolved>) -> Vec<Diagnostic>`
  - Codes: `1` fragment heading missing · `2` heading missing in an existing file · `3` file missing · `4` reference label without definition · `5` footnote reference without definition · `6` wikilink target missing. All `DiagnosticSeverity::ERROR`, `source: "lsp-poc"`.

- [ ] **Step 1: Write the failing tests**

Create `crates/lsp-poc/src/diagnostics/mod.rs` with module doc, imports, and the test module (implementation in Step 3):

```rust
//! Broken-link diagnostics, codes 1–6 (see the cycle-1 spec).
//!
//! Invalid link syntax (the reference's code 0) is consciously not
//! reproduced: with a typed parse, malformed links are simply not links.

use async_language_server::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{self, Target};
use crate::workspace::Resolved;

/// `Diagnostic::source` value for every diagnostic this server publishes.
pub const SOURCE: &str = "lsp-poc";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    fn parse(text: &str) -> MdIndex {
        let mut parser = MarkdownParser::default();
        links::build(&mut parser, text).expect("fixture parses")
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<i32> {
        let mut codes: Vec<i32> = diagnostics
            .iter()
            .map(|diagnostic| match diagnostic.code {
                Some(NumberOrString::Number(code)) => code,
                other => panic!("unexpected code: {other:?}"),
            })
            .collect();
        codes.sort_unstable();
        codes
    }

    const BROKEN: &str = concat!(
        "# Top\n",
        "\n",
        "[x](#ghost)\n",
        "\n",
        "[y][missing]\n",
        "\n",
        "file [f](nowhere.md#nope)\n",
        "\n",
        "footnote [^2] here\n",
        "\n",
        "wiki [[ghost-wiki]]\n",
    );

    #[test]
    fn broken_fixture_produces_all_five_shapes() {
        let index = parse(BROKEN);
        let everything_missing =
            |_: &Target| Some(Resolved::Missing);
        let diagnostics = compute(&index, &everything_missing);
        assert_eq!(codes(&diagnostics), vec![1, 3, 4, 5, 6]);
        for diagnostic in &diagnostics {
            assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
            assert_eq!(diagnostic.source.as_deref(), Some(SOURCE));
        }
    }

    #[test]
    fn heading_in_existing_file_is_code_two() {
        let index = parse("[f](real.md#nope)\n");
        let resolve = |_: &Target| {
            Some(Resolved::Found(Arc::new(parse("# Other\n"))))
        };
        let diagnostics = compute(&index, &resolve);
        assert_eq!(codes(&diagnostics), vec![2]);
        assert!(diagnostics[0].message.contains("real.md"));
    }

    #[test]
    fn valid_targets_stay_silent() {
        let index = parse(
            "# Top\n\n[ok](#top) [ok][ref] foot[^1] wiki[[Top]]\n\n[^1]: text\n\n[ref]: a.md\n",
        );
        let resolve = |_: &Target| {
            Some(Resolved::Found(Arc::new(parse("# Top\n"))))
        };
        let diagnostics = compute(&index, &resolve);
        assert!(diagnostics.is_empty(), "got: {diagnostics:?}");
    }

    #[test]
    fn footnote_shaped_references_do_not_double_report() {
        // `[^3]` without a definition must yield code 5 exactly once,
        // never code 4 for the same shape.
        let index = parse("footnote [^3] here\n");
        let everything_missing = |_: &Target| Some(Resolved::Missing);
        let diagnostics = compute(&index, &everything_missing);
        assert_eq!(codes(&diagnostics), vec![5]);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p lsp-poc diagnostics::` (after adding `mod diagnostics;` as the first `mod` line in `main.rs`)
Expected: compile error — `compute`, `SOURCE` not defined.

- [ ] **Step 3: Implement `compute`**

Add below `SOURCE` (before `#[cfg(test)]`):

```rust
/// Computes every broken-link diagnostic for one parsed document.
///
/// `resolve` answers cross-file targets (`Target::Doc`/`Target::Wiki`);
/// `None` means "not applicable" and silences the target. Same-document
/// fragments never reach `resolve`.
pub fn compute(
    index: &links::MdIndex,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for link in index.links() {
        let Some(target) = parse_destination(&link.destination) else {
            continue;
        };
        match &target {
            Target::Fragment(fragment) => {
                if !index.has_heading_slug(&slugify(fragment)) {
                    diagnostics.push(broken(
                        1,
                        link.range,
                        format!("link to non-existent heading `{fragment}`"),
                    ));
                }
            }
            Target::Doc { path, fragment } => match resolve(&target) {
                None => {}
                Some(Resolved::Missing) => diagnostics.push(broken(
                    3,
                    link.range,
                    format!("link to non-existent file `{path}`"),
                )),
                Some(Resolved::Found(target_index)) => {
                    if let Some(fragment) = fragment {
                        if !target_index.has_heading_slug(&slugify(fragment)) {
                            diagnostics.push(broken(
                                2,
                                link.range,
                                format!("link to non-existent heading `{fragment}` in `{path}`"),
                            ));
                        }
                    }
                }
            },
            Target::Wiki { path, fragment } => match resolve(&target) {
                None => {}
                Some(Resolved::Missing) => diagnostics.push(broken(
                    6,
                    link.range,
                    format!("wikilink to non-existent target `{path}`"),
                )),
                Some(Resolved::Found(target_index)) => {
                    if let Some(fragment) = fragment {
                        if !target_index.has_heading_slug(&slugify(fragment)) {
                            diagnostics.push(broken(
                                2,
                                link.range,
                                format!(
                                    "wikilink to non-existent heading `{fragment}` in `{path}`"
                                ),
                            ));
                        }
                    }
                }
            },
        }
    }

    for reference in index.references() {
        // Footnote and wikilink shapes may surface as reference links;
        // the footnote scan owns `^…`, wikilinks never have definitions.
        if reference.label.starts_with('^') || reference.label.starts_with('[') {
            continue;
        }
        if !index.has_definition(&reference.label) {
            diagnostics.push(broken(
                4,
                reference.range,
                format!(
                    "link reference to non-existent definition `{}`",
                    reference.label
                ),
            ));
        }
    }

    for footnote in index.footnote_references() {
        if !index.has_footnote_definition(&footnote.id) {
            diagnostics.push(broken(
                5,
                footnote.range,
                format!(
                    "footnote reference to non-existent definition `{}`",
                    footnote.id
                ),
            ));
        }
    }

    diagnostics
}

fn broken(code: i32, range: async_language_server::tree_sitter::Range, message: String) -> Diagnostic {
    Diagnostic {
        range: ts_range_to_lsp_range(range),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::Number(code)),
        code_description: None,
        source: Some(SOURCE.to_owned()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}
```

The `use` list at the top must also bring in `slugify` and `parse_destination`:
change `use crate::links::{self, Target};` to
`use crate::links::{self, parse_destination, slugify, Target};`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc diagnostics::`
Expected: 4 passed. If `broken_fixture_produces_all_five_shapes` reports an extra code 4 for `[^2]`, the Task 1 guard or this task's `references` filter needs the shape it missed — extend the filter (labels starting with `^`/`[`), never the expectation.

- [ ] **Step 5: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 4: Server wiring — capabilities, publish hooks, pull parity

**Files:**
- Modify: `crates/lsp-poc/src/server.rs` (struct fields, capabilities, hooks, `document_diagnostics`, private helpers)
- Modify: `crates/lsp-poc/src/main.rs` (verify the module list reads: `mod diagnostics; mod hovers; mod links; mod server; mod tracing; mod workspace;`)

**Interfaces:**
- Consumes: Tasks 1–3 exact names: `links::MdIndex`, `workspace::{Index, Resolved}`, `diagnostics::{compute, SOURCE}` (SOURCE not used in server — skip), `Target`.
- Produces: a binary that publishes `textDocument/publishDiagnostics` on `did_open`/`did_change` and answers `textDocument/diagnostic`. Zed shows red underlines (end-to-end criterion, owner-verified).

- [ ] **Step 1: Rewrite `crates/lsp-poc/src/server.rs`**

Full new content (the `hover` fn and its helpers stay byte-identical; only imports, the struct, `new`, `server_capabilities`, and the trait additions are new):

```rust
use std::future::{ready, Future};
use std::sync::{Arc, Mutex, PoisonError};

use async_language_server::lsp_types::notification::PublishDiagnostics;
use async_language_server::lsp_types::{
    ClientCapabilities, Diagnostic, DiagnosticServerCapabilities, DiagnosticOptions,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidChangeWorkspaceFoldersParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
    DocumentDiagnosticReportResult, FullDocumentDiagnosticReport, Hover, HoverContents,
    HoverParams, HoverProviderCapability, MarkupContent, MarkupKind, PublishDiagnosticsParams,
    RelatedFullDocumentDiagnosticReport, ServerCapabilities, ServerInfo, Url,
};
use async_language_server::server::{DocumentMatcher, Server, ServerResult, ServerState};
use async_language_server::tree_sitter_utils::{
    ts_range_contains_lsp_position, ts_range_to_lsp_range,
};
use tree_sitter_md::MarkdownParser;

use crate::diagnostics;
use crate::links::{self, Target};
use crate::workspace::{self, Index, Resolved};

pub struct PocLanguageServer {
    parser: Mutex<MarkdownParser>,
    files: Index,
}

impl PocLanguageServer {
    pub fn new() -> Self {
        Self {
            parser: Mutex::new(MarkdownParser::default()),
            files: Index::new(),
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
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                DiagnosticOptions {
                    workspace_diagnostics: false,
                    ..DiagnosticOptions::default()
                },
            )),
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
```

Notes:
- The old `#[derive(Debug, Clone)]` on the struct is dropped — the new fields are not `Clone`, and nothing in the framework requires either trait of the server.
- The `workspace::{self, ...}` import: `workspace` itself may be unused after this task (only `Index`/`Resolved` are named). If `rustfmt`/clippy flags the bare `workspace`, drop it and keep `use crate::workspace::{Index, Resolved};`.
- `did_change_workspace_folders` matches the trait's exact signature (sync, `&ServerState`, no return).

- [ ] **Step 2: Verify the module list in `main.rs`**

`crates/lsp-poc/src/main.rs` must read, in this order:

```rust
mod diagnostics;
mod hovers;
mod links;
mod server;
mod tracing;
mod workspace;
```

- [ ] **Step 3: Verify the wiring compiles clean**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four. There is no unit-test leg for the hooks: `ServerState` is only constructible inside the framework's own `#[cfg(test)]` — hook behavior is verified by the battery (types/signatures) plus the owner's live Zed test (Task 6).

---

### Task 5: Docs — the tree changed shape

**Files:**
- Modify: `CLAUDE.md` (lines 23, 24-25 region)
- Modify: `.claude/rules/product.md` (boundary paragraph, "Target Use Cases" line)
- Modify: `crates/lsp-poc/arch-lint.toml` (header comment only — no scope/rule edits)

**Interfaces:**
- Consumes: the landed Tasks 1–4 (module names, capabilities).
- Produces: documentation matching the tree; nothing downstream depends on this task.

- [ ] **Step 1: CLAUDE.md — capabilities line**

old:
```
  - `server_capabilities()` declares what the server supports (currently only hover)
```
new:
```
  - `server_capabilities()` declares what the server supports (currently hover and diagnostics)
```

- [ ] **Step 2: CLAUDE.md — feature-modules sentence**

old:
```
- Feature modules: `hovers/` is an empty placeholder — hover logic lives in `server.rs` itself. `src/utils.rs` is an undeclared PHP-era leftover (a module file on disk is dead until declared in `main.rs`).
```
new:
```
- Feature modules: `links/` (pure link model), `workspace/` (cross-file index), and `diagnostics/` (broken-link diagnostics) hang off `server.rs`; hover logic still lives in `server.rs` itself. `src/utils.rs` is an undeclared PHP-era leftover (a module file on disk is dead until declared in `main.rs`).
```

- [ ] **Step 3: product.md — the boundary paragraph**

old:
```
Keep Markdown as the boundary. A capability that needs another language's grammar or cross-file analysis is out of scope for this project.
```
new:
```
Keep Markdown as the boundary. A capability that needs another language's grammar is out of scope for this project; cross-file Markdown-to-Markdown analysis (workspace links) is in scope by owner decision (2026-09-13, cycle-1 spec).
```

- [ ] **Step 4: product.md — target use cases line**

old:
```
- Live experiments against real Markdown files open in Zed (currently: hover).
```
new:
```
- Live experiments against real Markdown files open in Zed (currently: hover, diagnostics).
```

- [ ] **Step 5: arch-lint.toml — header comment refresh (comment only)**

old (the sentence inside the `# --- Layer rules` comment block):
```
# spec has not landed yet (error, links, workspace, most capabilities) are
# declared up front so the layering guards the structure as it grows.
```
new:
```
# spec has not landed yet (error and the remaining capability scopes) are
# declared up front so the layering guards the structure as it grows;
# links, workspace, and diagnostics landed with the 2026-09-13 cycle-1
# spec (docs/superpowers/specs/2026-09-13-md-capabilities-cycle1-diagnostics-design.md).
```
Match the exact wrapped lines in the file before editing — the old text above is the tail of the sentence beginning "Scopes for modules that the feature". If the wrapped lines differ, adapt `old_string` to the file, not the other way around.

- [ ] **Step 6: Verify the docs**

Run: `grep -rn "currently only hover\|currently: hover)\|or cross-file analysis" CLAUDE.md .claude/rules/ && exit 1 || exit 0`
Expected: exit 0 (no matches).
Run: `make test`
Expected: exit 0 (the arch-lint test re-reads the header comment file).

---

### Task 6: Full verification + handoff

**Files:**
- No file changes. Verification only.

**Interfaces:**
- Consumes: Tasks 1–5 complete.
- Produces: the evidence the owner reads before committing, and the live-test instructions.

- [ ] **Step 1: Battery**

Run: `make battery`
Expected: exit 0 across `fmt`, `clippy`, `doc`, `test`, `dylint`.

- [ ] **Step 2: Deny and dupes (report, do not edit)**

Run: `make deny ; make dupes`
Expected: deny exits 2 with exactly the one owner-ratified `wildcard` error on the git dep (no new findings — this plan adds zero dependencies); dupes 0 groups. Any delta is a finding to report, not to fix silently.

- [ ] **Step 3: Changed-set report for the owner**

Run: `git status --porcelain && git diff --stat`
Expected changed set: created `crates/lsp-poc/src/links/mod.rs`, `crates/lsp-poc/src/workspace/mod.rs`, `crates/lsp-poc/src/diagnostics/mod.rs`; modified `crates/lsp-poc/src/main.rs`, `crates/lsp-poc/src/server.rs`, `CLAUDE.md`, `.claude/rules/product.md`, `crates/lsp-poc/arch-lint.toml` — 8 paths, nothing else (no `Cargo.lock` delta: zero dependency changes).

- [ ] **Step 4: Hand off to the owner**

Report: status, gate outputs, the changed set. Then the owner: commits; `cargo build`; opens a `.md` in Zed and breaks a link (`[x](#nope)`) — red underline appears; fixes it — disappears. Wikilink check: `[[ghost]]` on a line → underline; `[[README]]` with `README.md` at the repo root → clean. End-to-end success criterion from the spec.
