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

#![expect(
    dead_code,
    reason = "model lands before its consumers; wiring lands in cycle-1 task 4"
)]

use async_language_server::tree_sitter::{Node, Point, Range};
use tree_sitter_md::MarkdownParser;

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
        Some(("", fragment)) if !fragment.is_empty() => Some(Target::Fragment(fragment.to_owned())),
        Some((path, fragment)) => {
            let fragment = (!fragment.is_empty()).then(|| fragment.to_owned());
            (!path.is_empty()).then(|| Target::Doc {
                path: path.to_owned(),
                fragment,
            })
        }
        None => Some(Target::Doc {
            path: trimmed.to_owned(),
            fragment: None,
        }),
    }
}

/// Splits a wikilink's content into a [`Target::Wiki`]. `None` for
/// `[[#fragment]]` — same-document wikilinks are not diagnosed in cycle 1.
#[must_use]
pub fn parse_wiki_target(target: &str) -> Option<Target> {
    let (path, fragment) = match target.split_once('#') {
        Some((path, fragment)) => (path, (!fragment.is_empty()).then(|| fragment.to_owned())),
        None => (target, None),
    };
    (!path.is_empty()).then(|| Target::Wiki {
        path: path.to_owned(),
        fragment,
    })
}

/// GitHub-style anchor: lowercase, spaces → dashes (the reference's
/// normalization, not a full slugger — accepted POC divergence).
#[must_use]
pub fn slugify(heading: &str) -> String {
    heading.trim().to_lowercase().replace(' ', "-")
}

/// `CommonMark` label identity: trim, lowercase, collapse whitespace runs.
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
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
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

/// A link reference definition. One whose label starts with `^` is a
/// footnote definition (`[^id]: text` is valid `CommonMark` definition
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
    index.footnote_definitions.push(normalize_label(id));
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
            index.links.push(Link {
                destination: cleaned,
                range: node.range(),
            });
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
            let prev = if start == 0 {
                b' '
            } else {
                text.as_bytes()[start - 1]
            };
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
        // Images are not diagnosed in cycle 1 (the reference skips them too),
        // and nothing else in the inline grammar is collected either.
        _ => {}
    });
}

/// Scans an inline tree's root text for footnote references and wikilinks
/// — the shapes the grammars do not model.
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex) {
    let Ok(source) = root.utf8_text(text.as_bytes()) else {
        return;
    };
    let base = (root.start_byte(), root.start_position());
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
            id: normalize_label(id),
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
    let stripped = raw
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'));
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
        std::fs::read_to_string(fixture_path("links-fixture.md")).expect("fixture file exists")
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
        assert!(
            index
                .links
                .iter()
                .any(|link| link.destination == "guide.md")
        );
    }

    #[test]
    fn fixture_collects_references_footnotes_and_wikilinks() {
        let index = fixture();
        let mut labels: Vec<&str> = index.references.iter().map(|r| r.label.as_str()).collect();
        labels.sort_unstable();
        assert_eq!(labels, vec!["collapsed", "ref", "shortcut"]);
        assert!(index.has_footnote_definition("1"));
        assert_eq!(index.footnote_references.len(), 2);
        assert_eq!(index.footnote_references[0].id, "1");
        assert_eq!(index.footnote_references[1].id, "note");
        assert!(index.has_footnote_definition("note"));
        assert!(!index.has_footnote_definition("Note"));
        let targets: Vec<&str> = index.wikilinks.iter().map(|w| w.target.as_str()).collect();
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
            Some(Target::Doc {
                path: "guide.md".to_owned(),
                fragment: None
            })
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
            Some(Target::Doc {
                path: "guide.md".to_owned(),
                fragment: None
            })
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
            Some(Target::Wiki {
                path: "Other".to_owned(),
                fragment: None
            })
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
