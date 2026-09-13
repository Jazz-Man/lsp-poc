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
- No `as` casts for narrowing conversions (clippy pedantic denies them); tree-sitter `Range`/`Point` are `usize`-based and `ts_range_to_lsp_range` handles the `u32` edge.
- Surgical: the existing `hover` stays byte-identical; the Zed extension crate is untouched; `arch-lint.toml` scopes stay untouched (only the header comment is refreshed in Task 5).
- Rust work follows the `rust-skills` skill rules (err-\*, own-\*, pat-let-else, api-must-use, obs-structured-fields, test-cfg-test-module).
- Test documents are real files under `crates/lsp-poc/tests/fixtures/` (owner request, 2026-09-13) so later cycles reuse them; single-line micro-cases stay inline. Tests read them via `env!("CARGO_MANIFEST_DIR")` — each read site carries the `NoSyncIo` allow comment.
- Every task ends with `make fmt-fix && make fmt && make clippy && make test` green (called "the task gates").

### Model facts the tasks rely on (verified against tree-sitter-md 0.5.3 during planning)

- `MarkdownParser::default()` + `parse(text: &[u8], old_tree: Option<&MarkdownTree>) -> Option<MarkdownTree>`; `MarkdownTree::block_tree()` and `inline_trees()`. Inline trees are parsed via `set_included_ranges` with **absolute document coordinates** — every node's `range()` is a document range, no offset math.
- The grammars have **no footnote and no wikilink nodes**. Two consequences the model handles:
  - `[^1]: footnote text` is valid CommonMark link-reference-definition syntax, so the block grammar may parse it as `link_reference_definition` with label `^1` — footnote definitions are therefore detected in **both** places (definition nodes with `^`-labels, and paragraph-first-inline as fallback). Exactly one of the two applies per occurrence, so no duplicates.
  - `[^1]` and `[[Other]]` may surface inside `shortcut_link`/`collapsed_reference_link` nodes; a guard in `collect_inline` keeps those shapes out of `references` (they belong to the off-tree scan).
- Off-tree scanning runs over each inline tree's root text; wikilink ranges are rebuilt with byte-accurate `scan_range` (columns are bytes, tree-sitter's convention).

---

### Task 1: `src/links/` — pure Markdown link model

**Files:**
- Create: `crates/lsp-poc/tests/fixtures/links-fixture.md`
- Create: `crates/lsp-poc/src/links/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod links;` between `mod hovers;` and `mod server;`)

**Interfaces:**
- Consumes: `tree_sitter_md::MarkdownParser` (crate dep, `parser` feature); `async_language_server::tree_sitter::{Node, Point, Range}`.
- Produces (Tasks 2–4 rely on these exact names):
  - `MdIndex` (`Debug + Default`), built by `pub fn build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex>`
  - accessors: `links(&self) -> &[Link]`, `references(&self) -> &[Reference]`, `footnote_references(&self) -> &[FootnoteReference]`, `wikilinks(&self) -> &[Wikilink]`, `has_heading_slug(&self, slug: &str) -> bool`, `has_definition(&self, label: &str) -> bool`, `has_footnote_definition(&self, id: &str) -> bool`
  - types: `Heading { slug: String, level: u8 }`, `Link { destination: String, range: Range }`, `Reference { label: String, range: Range }`, `FootnoteReference { id: String, range: Range }`, `Wikilink { target: String, range: Range }` (all `Debug`; ranges are tree-sitter `Range`s in document coordinates). Definitions and footnote definitions are stored as `Vec<String>` (normalized labels / ids) — the struct-per-item shapes return in later cycles when consumers need their ranges.
  - `#[derive(Debug, PartialEq, Eq)] pub enum Target { Fragment(String), Doc { path: String, fragment: Option<String> }, Wiki { path: String, fragment: Option<String> } }`
  - `pub fn parse_destination(destination: &str) -> Option<Target>` (None = external/empty/degenerate), `pub fn parse_wiki_target(target: &str) -> Option<Target>` (None for `[[#frag]]` — same-document wikilinks are not diagnosed in cycle 1), `pub fn slugify(heading: &str) -> String` (lowercase + spaces → `-`, the reference's normalization)

- [ ] **Step 1: Create the fixture file and the module skeleton**

Create `crates/lsp-poc/tests/fixtures/links-fixture.md` with exactly this content (byte-exact — the range assertions slice back into it; trailing newline included):

```markdown
# Top

See [docs](guide.md) and [summary](#top).

Ref [label][ref] and [collapsed][] and [shortcut].

A footnote[^1] here.

[^1]: footnote text

Wiki [[Other]] and [[folder/note#Section]].

[ref]: https://example.com
```

Then create `crates/lsp-poc/src/links/mod.rs` with the module doc, imports, and types:

```rust
//! Pure Markdown link model: what a document contains and what its links
//! point at, parsed with tree-sitter's block + inline grammars.
//!
//! Headings, link reference definitions, and inline/reference links come
//! from typed grammar nodes. Footnotes and wikilinks have no nodes in
//! tree-sitter-md, so they are handled off-tree: definitions are `^…`
//! link-reference-definitions (with a paragraph fallback), references and
//! wikilinks are byte scans over inline text. Known limitation: those
//! scans also match shapes inside code spans — the grammars give no
//! cheaper boundary; refine when a cycle needs it.

use async_language_server::tree_sitter::{Node, Point, Range};
use tree_sitter_md::{MarkdownParser, MarkdownTree};

/// Everything the diagnostics need to know about one parsed document.
#[derive(Debug, Default)]
pub struct MdIndex {
    headings: Vec<Heading>,
    definitions: Vec<String>,
    links: Vec<Link>,
    references: Vec<Reference>,
    footnote_references: Vec<FootnoteReference>,
    footnote_definitions: Vec<String>,
    wikilinks: Vec<Wikilink>,
}

/// A heading, reduced to what matching needs: its anchor slug and depth.
#[derive(Debug)]
pub struct Heading {
    pub slug: String,
    pub level: u8,
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

- [ ] **Step 2: Write the failing tests (characterization + unit)**

Append to `src/links/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Absolute path of a shared fixture document.
    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn fixture_text() -> String {
        // arch-lint: allow(no-sync-io) reason="unit tests read their fixture document synchronously"
        std::fs::read_to_string(fixture_path("links-fixture.md"))
            .expect("fixture file exists")
    }

    fn fixture() -> MdIndex {
        let mut parser = MarkdownParser::default();
        build(&mut parser, &fixture_text()).expect("fixture parses")
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
        // `[label][ref]` contributes its LOOKUP label ("ref"), not its
        // display text — diagnostics code 4 is defined on the lookup key.
        // (Corrected 2026-09-13 during Task 1 execution: the original
        // expectation ["collapsed", "label", "shortcut"] contradicted
        // Task 3's valid-links semantics; full_reference_link's
        // link_label child is the lookup bracket.)
        assert_eq!(labels, vec!["collapsed", "ref", "shortcut"]);
        assert!(index.has_footnote_definition("1"));
        assert_eq!(index.footnote_references.len(), 1);
        assert_eq!(index.footnote_references[0].id, "1");
        let targets: Vec<&str> =
            index.wikilinks.iter().map(|w| w.target.as_str()).collect();
        assert_eq!(targets, vec!["Other", "folder/note#Section"]);
    }

    #[test]
    fn scanned_ranges_slice_back_to_their_source_text() {
        let text = fixture_text();
        let index = fixture();
        let range = index.footnote_references[0].range;
        assert_eq!(&text[range.start_byte..range.end_byte], "[^1]");
        let wiki_range = index.wikilinks[1].range;
        assert_eq!(
            &text[wiki_range.start_byte..wiki_range.end_byte],
            "[[folder/note#Section]]",
        );
    }

    #[test]
    fn parse_destination_classifies_targets() {
        assert_eq!(
            parse_destination("#top"),
            Some(Target::Fragment("top".to_owned()))
        );
        assert_eq!(
            parse_destination("guide.md"),
            Some(Target::Doc { path: "guide.md".to_owned(), fragment: None })
        );
        assert_eq!(
            parse_destination("a/b.md#section"),
            Some(Target::Doc {
                path: "a/b.md".to_owned(),
                fragment: Some("section".to_owned()),
            })
        );
        assert_eq!(
            parse_destination("guide.md#"),
            Some(Target::Doc { path: "guide.md".to_owned(), fragment: None })
        );
        assert_eq!(parse_destination("https://example.com"), None);
        assert_eq!(parse_destination("mailto:x@y.z"), None);
        assert_eq!(parse_destination("#"), None);
        assert_eq!(parse_destination(""), None);
    }

    #[test]
    fn parse_wiki_target_splits_path_and_fragment() {
        assert_eq!(
            parse_wiki_target("Other"),
            Some(Target::Wiki { path: "Other".to_owned(), fragment: None })
        );
        assert_eq!(
            parse_wiki_target("folder/note#Section"),
            Some(Target::Wiki {
                path: "folder/note".to_owned(),
                fragment: Some("Section".to_owned()),
            })
        );
        assert_eq!(parse_wiki_target("#Section"), None);
    }

    #[test]
    fn slugify_matches_the_reference_normalization() {
        assert_eq!(slugify("Heading One"), "heading-one");
        assert_eq!(slugify("  Mixed CASE "), "mixed-case");
    }
}
```

**Characterization note (pre-authorized adjustment):** the exact shapes tree-sitter-md 0.5.3 produces for `[^1]` and `[[Other]]` inside inline content are pinned by `fixture_collects_references_footnotes_and_wikilinks`. If that run shows an extra `references` entry (`^1`, `other`, `[[other]]`, …), the fix is the guard in Step 4's collapsed/shortcut arm — strengthen the guard, never the expectation: footnote/wikilink syntax must not produce `Reference` entries, and normal `[collapsed]`/`[shortcut]` links must keep producing them.

- [ ] **Step 3: Add `mod links;` and run the tests to verify they fail**

In `crates/lsp-poc/src/main.rs` add between `mod hovers;` and `mod server;`:

```rust
mod links;
```

Run: `cargo nextest run -p lsp-poc links::`
Expected: compile error — `build`, `parse_destination`, `parse_wiki_target`, `slugify` not found in `links`.

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
    pub fn wikilinks(&self) -> &[Wikilink] {
        &self.wikilinks
    }

    #[must_use]
    pub fn has_heading_slug(&self, slug: &str) -> bool {
        self.headings.iter().any(|heading| heading.slug == slug)
    }

    #[must_use]
    pub fn has_definition(&self, label: &str) -> bool {
        self.definitions.iter().any(|known| known == label)
    }

    #[must_use]
    pub fn has_footnote_definition(&self, id: &str) -> bool {
        self.footnote_definitions.iter().any(|known| known == id)
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
/// (empty, `scheme://…`, `mailto:`, or a degenerate `#`/`path#` shape).
#[must_use]
pub fn parse_destination(destination: &str) -> Option<Target> {
    let trimmed = destination.trim();
    if trimmed.is_empty() || trimmed.contains("://") || trimmed.starts_with("mailto:") {
        return None;
    }
    match trimmed.split_once('#') {
        Some(("", fragment)) if !fragment.is_empty() => {
            Some(Target::Fragment(fragment.to_owned()))
        }
        Some((path, fragment)) => {
            let fragment = (!fragment.is_empty()).then(|| fragment.to_owned());
            (!path.is_empty()).then(|| Target::Doc { path: path.to_owned(), fragment })
        }
        None => Some(Target::Doc { path: trimmed.to_owned(), fragment: None }),
    }
}

/// Splits a wikilink's content into a [`Target::Wiki`]. `None` for
/// `[[#fragment]]` — same-document wikilinks are not diagnosed in cycle 1.
#[must_use]
pub fn parse_wiki_target(target: &str) -> Option<Target> {
    let (path, fragment) = match target.split_once('#') {
        Some((path, fragment)) => {
            (path, (!fragment.is_empty()).then(|| fragment.to_owned()))
        }
        None => (target, None),
    };
    (!path.is_empty()).then(|| Target::Wiki { path: path.to_owned(), fragment })
}

/// GitHub-style anchor: lowercase, spaces → dashes (the reference's
/// normalization, not a full slugger — accepted POC divergence).
#[must_use]
pub fn slugify(heading: &str) -> String {
    heading.trim().to_lowercase().replace(' ', "-")
}

/// CommonMark label identity: trim, lowercase, collapse whitespace runs.
fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn collect_block(node: Node, text: &str, index: &mut MdIndex) {
    match node.kind() {
        "atx_heading" => collect_heading(node, text, index),
        "link_reference_definition" => collect_definition(node, text, index),
        "paragraph" => collect_footnote_definition_from_paragraph(node, text, index),
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
    index.headings.push(Heading { slug: slugify(raw), level });
}

fn heading_level(marker: &Node) -> Option<u8> {
    marker
        .kind()
        .strip_prefix("atx_h")?
        .strip_suffix("_marker")?
        .parse()
        .ok()
}

/// A link reference definition. One whose label starts with `^` is a
/// footnote definition (`[^id]: text` is valid CommonMark definition
/// syntax and this grammar has no footnote nodes) — recorded on the
/// footnote side instead.
fn collect_definition(definition: Node, text: &str, index: &mut MdIndex) {
    let mut cursor = definition.walk();
    let Some(label_node) = definition
        .children(&mut cursor)
        .find(|child| child.kind() == "link_label")
    else {
        return;
    };
    let Some(raw) = bracketed_text(label_node, text) else {
        return;
    };
    let label = normalize_label(&raw);
    if let Some(id) = label.strip_prefix('^') {
        if !id.is_empty() {
            index.footnote_definitions.push(id.to_owned());
        }
        return;
    }
    index.definitions.push(label);
}

/// Fallback footnote-definition detection for the case the block grammar
/// parsed `[^id]: …` as a plain paragraph: its first inline starts with
/// `[^id]:`. Exactly one of the two detectors applies per occurrence.
fn collect_footnote_definition_from_paragraph(paragraph: Node, text: &str, index: &mut MdIndex) {
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
    index.footnote_definitions.push(id.to_owned());
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
            index
                .links
                .push(Link { destination: cleaned, range: node.range() });
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
            // `[[wikilink]]` and `[^footnote]` can parse as these node
            // kinds too; keep them out of references — the off-tree scan
            // owns both shapes. The guard covers both observed layouts:
            // the node spanning the outer brackets, or starting one byte
            // after a `[`/`^`.
            let start = node.range().start_byte;
            let prev = if start == 0 { b' ' } else { text.as_bytes()[start - 1] };
            let Ok(node_raw) = node.utf8_text(text.as_bytes()) else {
                return;
            };
            if node_raw.starts_with("[[")
                || node_raw.starts_with("[^")
                || prev == b'['
                || prev == b'^'
            {
                return;
            }
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

/// Scans an inline tree's root text for footnote references and wikilinks
/// — the shapes the grammars do not model.
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex) {
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
        let after_bracket = &source[start + 2 + relative + 1..];
        let is_definition_marker = after_bracket.starts_with(':');
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
        Point {
            row: start_point.row,
            column: start_point.column + len,
        }
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

/// `[label]` text without its brackets (falls back to the raw text when
/// the brackets are not part of the node).
fn bracketed_text(node: Node, text: &str) -> Option<String> {
    let raw = node.utf8_text(text.as_bytes()).ok()?;
    let stripped = raw.strip_prefix('[').and_then(|rest| rest.strip_suffix(']'));
    Some(stripped.unwrap_or(raw).trim().to_owned())
}

/// A link destination without `<>` wrapping.
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

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc links::`
Expected: 6 passed. If `fixture_collects_references_footnotes_and_wikilinks` fails on an extra or missing reference, apply the pre-authorized guard adjustment from Step 2 and re-run.

- [ ] **Step 6: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 2: `src/workspace/` — cross-file index and target resolution

**Files:**
- Create: `crates/lsp-poc/src/workspace/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod workspace;` after `mod tracing;`)

**Interfaces:**
- Consumes: Task 1's `links::{build, parse_destination, MdIndex, Target}` (exact names above); `async_language_server::lsp_types::Url`.
- Produces (Tasks 3–4 rely on these):
  - `#[derive(Debug)] pub enum Resolved { Found(Arc<links::MdIndex>), Missing }` (no `PartialEq` — tests pattern-match)
  - `pub struct Index` with `Index::new()`, `pub fn resolve(&self, open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>, self_url: &Url, target: &Target) -> Option<Resolved>` (`None` = "not applicable", only for `Target::Fragment`), `pub fn reset_root(&self)`

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

use crate::links;

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
    use crate::links::{Target, parse_destination};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lsp-poc-ws-{name}-{}", std::process::id()));
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
        let open_index = parse("# Open only\n");
        let url_for_open = doc_url.clone();
        let open = move |url: &Url| (url == &url_for_open).then(|| open_index.clone());

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
    fn wikilinks_miss_without_a_git_root() {
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

- [ ] **Step 2: Add `mod workspace;` and run the tests to verify they fail**

In `crates/lsp-poc/src/main.rs` add after `mod tracing;`:

```rust
mod workspace;
```

Run: `cargo nextest run -p lsp-poc workspace::`
Expected: compile error — `Index`, `Resolved::Found`, `resolve`, `reset_root` not defined.

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

    fn load(
        &self,
        open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>,
        candidates: &[PathBuf],
    ) -> Resolved {
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
            .insert(
                url.clone(),
                Entry { stamp: Some(stamp), index: Arc::clone(&index) },
            );
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
- Create: `crates/lsp-poc/tests/fixtures/broken-links.md`
- Create: `crates/lsp-poc/tests/fixtures/valid-links.md`
- Create: `crates/lsp-poc/src/diagnostics/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (add `mod diagnostics;` as the first module line, alphabetical)

**Interfaces:**
- Consumes: Task 1's `links::{MdIndex, Target, parse_destination, parse_wiki_target, slugify}` + accessors; Task 2's `Resolved`; `async_language_server::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString}`; `async_language_server::tree_sitter::Range`; `async_language_server::tree_sitter_utils::ts_range_to_lsp_range`.
- Produces (Task 4 relies on this):
  - `pub const SOURCE: &str = "lsp-poc";`
  - `pub fn compute(index: &links::MdIndex, resolve: &dyn Fn(&Target) -> Option<Resolved>) -> Vec<Diagnostic>`
  - Codes: `1` fragment heading missing · `2` heading missing in an existing file (links and wikilinks) · `3` file missing · `4` reference label without definition · `5` footnote reference without definition · `6` wikilink target missing. All `DiagnosticSeverity::ERROR`, `source: "lsp-poc"`.

- [ ] **Step 1: Create the fixture files and write the failing tests**

Create `crates/lsp-poc/tests/fixtures/broken-links.md` (byte-exact, trailing newline):

```markdown
# Top

[x](#ghost)

[y][missing]

file [f](nowhere.md#nope)

footnote [^2] here

wiki [[ghost-wiki]]
```

Create `crates/lsp-poc/tests/fixtures/valid-links.md` (byte-exact, trailing newline):

```markdown
# Top

[ok](#top) [ok][ref] foot[^1] wiki[[Top]]

[^1]: text

[ref]: a.md
```

Then create `crates/lsp-poc/src/diagnostics/mod.rs` with module doc, imports, and the test module (implementation in Step 3):

```rust
//! Broken-link diagnostics, codes 1–6 (see the cycle-1 spec).
//!
//! Invalid link syntax (the reference's code 0) is consciously not
//! reproduced: with a typed parse, malformed links are simply not links.

use async_language_server::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use async_language_server::tree_sitter::Range;
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{self, parse_destination, parse_wiki_target, slugify, Target};
use crate::workspace::Resolved;

/// `Diagnostic::source` value for every diagnostic this server publishes.
pub const SOURCE: &str = "lsp-poc";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    fn fixture_text(name: &str) -> String {
        // arch-lint: allow(no-sync-io) reason="unit tests read their fixture documents synchronously"
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
        )
        .expect("fixture file exists")
    }

    fn parse(text: &str) -> MdIndex {
        let mut parser = MarkdownParser::default();
        links::build(&mut parser, text).expect("input parses")
    }

    fn fixture(name: &str) -> MdIndex {
        parse(&fixture_text(name))
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

    #[test]
    fn broken_fixture_produces_all_five_shapes() {
        let index = fixture("broken-links.md");
        let everything_missing = |_: &Target| Some(Resolved::Missing);
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
        let resolve = |_: &Target| Some(Resolved::Found(Arc::new(parse("# Other\n"))));
        let diagnostics = compute(&index, &resolve);
        assert_eq!(codes(&diagnostics), vec![2]);
        assert!(diagnostics[0].message.contains("real.md"));
    }

    #[test]
    fn valid_targets_stay_silent() {
        let index = fixture("valid-links.md");
        let resolve = |_: &Target| Some(Resolved::Found(Arc::new(parse("# Top\n"))));
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

- [ ] **Step 2: Add `mod diagnostics;` and run the tests to verify they fail**

In `crates/lsp-poc/src/main.rs` add as the first module line (alphabetical):

```rust
mod diagnostics;
```

Run: `cargo nextest run -p lsp-poc diagnostics::`
Expected: compile error — `compute` not defined in `diagnostics`.

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
                    check_target_heading(
                        &mut diagnostics,
                        target_index,
                        fragment.as_deref(),
                        link.range,
                        &format!("link to non-existent heading `{{}}` in `{path}`"),
                    );
                }
            },
            // parse_destination never produces Wiki — the wikilink loop
            // below owns those. The arm exists for match exhaustiveness.
            Target::Wiki { .. } => {}
        }
    }

    for wikilink in index.wikilinks() {
        let Some(target) = parse_wiki_target(&wikilink.target) else {
            continue;
        };
        let Target::Wiki { path, fragment } = &target else {
            continue;
        };
        match resolve(&target) {
            None => {}
            Some(Resolved::Missing) => diagnostics.push(broken(
                6,
                wikilink.range,
                format!("wikilink to non-existent target `{path}`"),
            )),
            Some(Resolved::Found(target_index)) => {
                check_target_heading(
                    &mut diagnostics,
                    target_index,
                    fragment.as_deref(),
                    wikilink.range,
                    &format!("wikilink to non-existent heading `{{}}` in `{path}`"),
                );
            }
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
                format!("link reference to non-existent definition `{}`", reference.label),
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

/// Appends code 2 when a resolved target's fragment names no heading.
fn check_target_heading(
    diagnostics: &mut Vec<Diagnostic>,
    target_index: &links::MdIndex,
    fragment: Option<&str>,
    range: async_language_server::tree_sitter::Range,
    template: &str,
) {
    let Some(fragment) = fragment else {
        return;
    };
    if target_index.has_heading_slug(&slugify(fragment)) {
        return;
    }
    diagnostics.push(broken(
        2,
        range,
        template.replace("{}", fragment),
    ));
}

fn broken(code: i32, range: Range, message: String) -> Diagnostic {
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

**Implementer latitude on the fragment check:** the `check_target_heading` helper above may be replaced by the direct inline form in both loops — no behavioral difference, pick whichever reads better in place. The direct form for the link loop is:

```rust
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
```
(and the analogous `wikilink to non-existent heading …` in the wiki loop). Pick one form — whichever reads better in place — and keep it identical in both loops.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p lsp-poc diagnostics::`
Expected: 4 passed. If `broken_fixture_produces_all_five_shapes` reports an extra code 4 for `[^2]` or the wiki shape, the Task 1 guard or this task's `references` filter missed the shape — extend the filter (labels starting with `^`/`[`), never the expectation.

- [ ] **Step 5: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 4: Server wiring — capabilities, publish hooks, pull parity

**Files:**
- Modify: `crates/lsp-poc/src/server.rs` (struct fields, capabilities, hooks, `document_diagnostics`, private helpers)
- Modify: `crates/lsp-poc/src/main.rs` (verify the module list reads exactly: `mod diagnostics; mod hovers; mod links; mod server; mod tracing; mod workspace;`)

**Interfaces:**
- Consumes: Tasks 1–3 exact names: `links::{self, Target}`, `workspace::Index`, `diagnostics::compute`.
- Produces: a binary that publishes `textDocument/publishDiagnostics` on `did_open`/`did_change` and answers `textDocument/diagnostic`. Zed shows red underlines (end-to-end criterion, owner-verified).

- [ ] **Step 1: Rewrite `crates/lsp-poc/src/server.rs`**

Full new content (the `hover` fn stays byte-identical at the bottom; only imports, the struct, `new`, `server_capabilities`, and the trait additions are new):

```rust
use std::future::{ready, Future};
use std::sync::{Arc, Mutex, PoisonError};

use async_language_server::lsp_types::notification::PublishDiagnostics;
use async_language_server::lsp_types::{
    ClientCapabilities, Diagnostic, DiagnosticServerCapabilities, DiagnosticOptions,
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
- `did_change_workspace_folders` matches the trait's exact signature (sync, `&ServerState`, no return).
- If clippy flags anything about the `open` closure's inference, annotate it as `let open = |open_url: &Url| -> Option<Arc<links::MdIndex>> { … };`.

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
- Modify: `CLAUDE.md` (capability line, feature-modules line)
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

The sentence beginning "Scopes for modules that the feature" ends with:
```
# spec has not landed yet (error, links, workspace, most capabilities) are
# declared up front so the layering guards the structure as it grows.
```
Replace that tail with:
```
# spec has not landed yet (error and the remaining capability scopes) are
# declared up front so the layering guards the structure as it grows;
# links, workspace, and diagnostics landed with the 2026-09-13 cycle-1
# spec (docs/superpowers/specs/2026-09-13-md-capabilities-cycle1-diagnostics-design.md).
```
Match the exact wrapped lines in the file before editing — if the wrap differs, adapt `old_string` to the file, not the other way around.

- [ ] **Step 6: Verify the docs**

Run: `grep -rn "currently only hover\|currently: hover)\|or cross-file analysis" CLAUDE.md .claude/rules/ && exit 1 || exit 0`
Expected: exit 0 (no matches).
Run: `make test`
Expected: exit 0 (the arch-lint test re-reads the edited file).

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
Expected changed set: created `crates/lsp-poc/tests/fixtures/{links-fixture,broken-links,valid-links}.md`, `crates/lsp-poc/src/links/mod.rs`, `crates/lsp-poc/src/workspace/mod.rs`, `crates/lsp-poc/src/diagnostics/mod.rs`; modified `crates/lsp-poc/src/main.rs`, `crates/lsp-poc/src/server.rs`, `CLAUDE.md`, `.claude/rules/product.md`, `crates/lsp-poc/arch-lint.toml` — 11 paths, nothing else (no `Cargo.lock` delta: zero dependency changes).

- [ ] **Step 4: Hand off to the owner**

Report: status, gate outputs, the changed set. Then the owner: commits; `cargo build`; opens a `.md` in Zed and breaks a link (`[x](#nope)`) — red underline appears; fixes it — disappears. Wikilink check: `[[ghost]]` on a line → underline; `[[README]]` with `README.md` at the repo root → clean. Footnote check: `[^x]` without a definition → underline; with `[^x]: text` → clean. End-to-end success criterion from the spec.
