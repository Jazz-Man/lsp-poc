//! Pure Markdown link model: what a document contains and what its links
//! point at, parsed with tree-sitter's block + inline grammars.
//!
//! Headings, link reference definitions, and inline/reference links come
//! from typed grammar nodes. Footnotes and wikilinks have no nodes in
//! tree-sitter-md, so they are handled off-tree: definitions are `^…`
//! link-reference-definitions (with a paragraph fallback), references and
//! wikilinks are byte scans over inline text. Code regions are excluded:
//! fenced and indented code blocks suppress their inline trees entirely,
//! and off-tree scans skip `code_span` windows — example links in
//! documentation produce no shapes. Headings come in both ATX (`#`) and
//! setext (underlined) forms.

mod slug;

use async_language_server::lsp_types::Position as LspPosition;
use async_language_server::tree_sitter::{Node, Point, Range};
use async_language_server::tree_sitter_utils::ts_range_contains_lsp_position;
use tree_sitter_md::MarkdownParser;

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

/// A heading: its anchor slug, depth, and document range (first match by
/// slug wins for definition routing).
#[derive(Debug)]
pub struct Heading {
    pub slug: String,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "heading depth is consumed from cycle 6 (symbols/ToC) onward"
        )
    )]
    pub level: u8,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "heading content is consumed from cycle 4 (rename) onward"
        )
    )]
    pub content_range: Range,
    pub range: Range,
}

/// A link reference definition: `[label]: destination`.
#[derive(Debug)]
pub struct Definition {
    pub label: String,
    #[expect(
        dead_code,
        reason = "definition destinations ride along for cycle-5 completion"
    )]
    pub destination: String,
    pub range: Range,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "label ranges are consumed from cycle 4 (rename) onward"
        )
    )]
    pub label_range: Range,
}

/// A footnote definition `[^id]: text`.
#[derive(Debug)]
pub struct FootnoteDefinition {
    pub id: String,
    pub range: Range,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "footnote id ranges are consumed from cycle 4 (rename) onward"
        )
    )]
    pub id_range: Range,
}

/// An inline link `[text](destination)` (images are not collected in cycle 1).
#[derive(Debug)]
pub struct Link {
    pub destination: String,
    pub range: Range,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "link fragments are consumed from cycle 4 (rename) onward"
        )
    )]
    pub fragment_range: Option<Range>,
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
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "footnote id ranges are consumed from cycle 4 (rename) onward"
        )
    )]
    pub id_range: Range,
}

/// A wikilink `[[target]]` (Obsidian-style; not standard Markdown).
#[derive(Debug)]
pub struct Wikilink {
    pub target: String,
    pub range: Range,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "wikilink fragments are consumed from cycle 4 (rename) onward"
        )
    )]
    pub fragment_range: Option<Range>,
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

/// The model item whose range contains a cursor position.
#[derive(Debug)]
pub enum CursorItem<'a> {
    /// A heading.
    Heading(&'a Heading),
    /// An inline link.
    Link(&'a Link),
    /// A reference link's label.
    Reference(&'a Reference),
    /// A link reference definition.
    Definition(&'a Definition),
    /// A footnote reference.
    FootnoteReference(&'a FootnoteReference),
    /// A footnote definition.
    FootnoteDefinition(&'a FootnoteDefinition),
    /// A wikilink.
    Wikilink(&'a Wikilink),
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
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    #[must_use]
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    #[must_use]
    pub fn footnote_definitions(&self) -> &[FootnoteDefinition] {
        &self.footnote_definitions
    }

    #[must_use]
    pub fn has_heading_slug(&self, slug: &str) -> bool {
        self.headings.iter().any(|heading| heading.slug == slug)
    }

    #[must_use]
    pub fn has_definition(&self, label: &str) -> bool {
        self.definitions.iter().any(|known| known.label == label)
    }

    #[must_use]
    pub fn has_footnote_definition(&self, id: &str) -> bool {
        self.footnote_definitions.iter().any(|known| known.id == id)
    }

    /// First heading whose slug matches, in document order.
    #[must_use]
    pub fn heading_by_slug(&self, slug: &str) -> Option<&Heading> {
        self.headings.iter().find(|heading| heading.slug == slug)
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
    let mut code = Vec::new();
    collect_block(tree.block_tree().root_node(), text, &mut index, &mut code);
    for inline in tree.inline_trees() {
        let root = inline.root_node();
        if code.iter().any(|range| covers(*range, root.range())) {
            continue;
        }
        let mut spans = Vec::new();
        collect_inline(root, text, &mut index, &mut spans);
        scan_offtree(root, text, &mut index, &mut spans);
    }
    Some(index)
}

/// `true` when `inner` lies fully inside `outer` (byte comparison — both
/// ranges come from the same parse of the same text).
fn covers(outer: Range, inner: Range) -> bool {
    inner.start_byte >= outer.start_byte && inner.end_byte <= outer.end_byte
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

pub use slug::slugify;

/// `CommonMark` label identity: trim, lowercase, collapse whitespace runs.
fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn collect_block(node: Node, text: &str, index: &mut MdIndex, code: &mut Vec<Range>) {
    match node.kind() {
        "atx_heading" | "setext_heading" => {
            let mut cursor = node.walk();
            if let Some(level) = node
                .children(&mut cursor)
                .find(|child| {
                    child.kind().starts_with("atx_h") || child.kind().starts_with("setext_h")
                })
                .and_then(|marker| heading_level(&marker))
            {
                push_heading(node, text, level, index);
            }
        }
        "link_reference_definition" => collect_definition(node, text, index),
        "paragraph" => collect_footnote_definition_from_paragraph(node, text, index),
        "fenced_code_block" | "indented_code_block" => code.push(node.range()),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_block(child, text, index, code);
    }
}

/// Pushes a heading (ATX or setext — the node carries the same
/// `heading_content` field) into the index.
fn push_heading(heading: Node, text: &str, level: u8, index: &mut MdIndex) {
    let Some(content) = heading.child_by_field_name("heading_content") else {
        return;
    };
    let Ok(raw) = content.utf8_text(text.as_bytes()) else {
        return;
    };
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
        content_range: content.range(),
        range: heading.range(),
    });
}

/// Level from the marker/underline child: `atx_h1_marker` → 1,
/// `setext_h2_underline` → 2.
fn heading_level(marker: &Node) -> Option<u8> {
    let kind = marker.kind();
    let digit = kind
        .strip_prefix("atx_h")
        .and_then(|rest| rest.strip_suffix("_marker"))
        .or_else(|| {
            kind.strip_prefix("setext_h")
                .and_then(|rest| rest.strip_suffix("_underline"))
        })?;
    digit.parse().ok()
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
    let destination = definition
        .children(&mut cursor)
        .find(|child| child.kind() == "link_destination")
        .and_then(|child| clean_destination(child, text));
    if let Some(id) = label.strip_prefix('^') {
        if !id.is_empty() {
            // The label's inner bytes still carry the caret (`^id`); the
            // rename replaces the bare id, so the range starts past it.
            let Some(mut id_range) = inner_range(&label_node, text) else {
                return;
            };
            id_range.start_byte += 1;
            id_range.start_point.column += 1;
            index.footnote_definitions.push(FootnoteDefinition {
                id: id.to_owned(),
                range: definition.range(),
                id_range,
            });
        }
        return;
    }
    let Some(label_range) = inner_range(&label_node, text) else {
        return;
    };
    index.definitions.push(Definition {
        label,
        destination: destination.unwrap_or_default(),
        range: definition.range(),
        label_range,
    });
}

/// The bytes INSIDE a `[label]` node — brackets stripped — as a
/// document-absolute range (the rename replaces bare label text).
/// Labels are single-line (the `link_label` grammar node), so the end
/// point's column is the start column plus the inner length.
fn inner_range(label_node: &Node, text: &str) -> Option<Range> {
    let raw = label_node.utf8_text(text.as_bytes()).ok()?;
    let leading = raw.len() - raw.trim_start_matches('[').len();
    let trailing = raw.len() - raw.trim_end_matches(']').len();
    let start = label_node.start_byte() + leading;
    let inner_len = label_node.end_byte() - trailing - start;
    let start_col = label_node.start_position().column + leading;
    Some(Range {
        start_byte: start,
        end_byte: start + inner_len,
        start_point: Point {
            row: label_node.start_position().row,
            column: start_col,
        },
        end_point: Point {
            row: label_node.start_position().row,
            column: start_col + inner_len,
        },
    })
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
    index.footnote_definitions.push(FootnoteDefinition {
        id: normalize_label(id),
        range: paragraph.range(),
        id_range: scan_range(
            (first.start_byte(), first.start_position()),
            raw,
            2,
            id.len(),
        ),
    });
}

fn collect_inline(root: Node, text: &str, index: &mut MdIndex, spans: &mut Vec<Range>) {
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
            let fragment_range = fragment_range_in(&destination, text);
            index.links.push(Link {
                destination: cleaned,
                range: node.range(),
                fragment_range,
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
        "code_span" => spans.push(node.range()),
        // Images are not diagnosed in cycle 1 (the reference skips them too),
        // and nothing else in the inline grammar is collected either.
        _ => {}
    });
}

/// Scans an inline tree's root text for footnote references and wikilinks
/// — the shapes the grammars do not model.
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex, spans: &mut [Range]) {
    let Ok(source) = root.utf8_text(text.as_bytes()) else {
        return;
    };
    let root_base = (root.start_byte(), root.start_position());
    // Spans carry absolute document bytes and `complement` keeps that
    // space; the scans below index the root's own text, so each window is
    // brought back into that root-relative space before slicing.
    let text = (root_base.0, root_base.0 + source.len());
    for (start, end) in complement(spans, text) {
        let (start, end) = (start - root_base.0, end - root_base.0);
        let slice = &source[start..end];
        let before = &source[..start];
        let rows = before.matches('\n').count();
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let column = if rows == 0 {
            root_base.1.column + start
        } else {
            start - line_start
        };
        let base = (
            root_base.0 + start,
            Point {
                row: root_base.1.row + rows,
                column,
            },
        );
        scan_footnote_references(slice, base, index);
        scan_wikilinks(slice, base, index);
    }
}

/// Maximal windows of `text` (absolute byte bounds) not covered by any
/// span, in document order. Sorts `spans` in place as a side effect.
fn complement(spans: &mut [Range], text: (usize, usize)) -> Vec<(usize, usize)> {
    spans.sort_unstable_by_key(|range| (range.start_byte, range.end_byte));
    let mut segments = Vec::new();
    let mut at = text.0;
    for range in spans.iter() {
        let (start, end) = (range.start_byte, range.end_byte);
        if start > at {
            segments.push((at, start));
        }
        at = at.max(end);
    }
    if at < text.1 {
        segments.push((at, text.1));
    }
    segments
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
            id_range: scan_range(base, source, start + 2, relative),
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
        let fragment_range = content.find('#').map(|hash_offset| {
            scan_range(
                base,
                source,
                start + 2 + hash_offset + 1,
                relative - hash_offset - 1,
            )
        });
        index.wikilinks.push(Wikilink {
            target: content.trim().to_owned(),
            range: scan_range(base, source, start, relative + 4),
            fragment_range,
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
    let column = if rows == 0 {
        base.1.column + start
    } else {
        // A continuation line's prefix width is not the node's start
        // column (lazy continuations carry none) — the column is the
        // offset within its own line.
        start - line_start
    };
    let start_point = Point {
        row: base.1.row + rows,
        column,
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

/// The `#fragment` range inside a destination node's own text, in
/// document-absolute coordinates (`None` when the destination has no
/// fragment).
fn fragment_range_in(node: &Node, text: &str) -> Option<Range> {
    let node_text = node.utf8_text(text.as_bytes()).ok()?;
    let hash_offset = node_text.find('#')?;
    Some(scan_range(
        (node.start_byte(), node.start_position()),
        node_text,
        hash_offset + 1,
        node_text.len() - hash_offset - 1,
    ))
}

fn walk(node: Node, visit: &mut dyn FnMut(Node)) {
    visit(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, visit);
    }
}

/// The document-start range, for file targets without a fragment.
#[must_use]
pub fn zero_range() -> Range {
    Range {
        start_byte: 0,
        end_byte: 0,
        start_point: Point { row: 0, column: 0 },
        end_point: Point { row: 0, column: 0 },
    }
}

/// The model item whose range contains `position`, searched in a fixed
/// order: links, footnote references, references, definitions, wikilinks,
/// footnote definitions, headings. Footnote references precede references
/// so a `[text][^1]` cursor routes to the footnote side, as the cycle-2
/// fall-through did; wikilinks precede footnote definitions so the more
/// specific item wins inside a footnote paragraph.
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
    if let Some(reference) = index
        .references
        .iter()
        .find(|reference| at(reference.range))
    {
        return Some(CursorItem::Reference(reference));
    }
    if let Some(definition) = index.definitions.iter().find(|def| at(def.range)) {
        return Some(CursorItem::Definition(definition));
    }
    if let Some(wikilink) = index.wikilinks.iter().find(|wikilink| at(wikilink.range)) {
        return Some(CursorItem::Wikilink(wikilink));
    }
    if let Some(definition) = index
        .footnote_definitions
        .iter()
        .find(|definition| at(definition.range))
    {
        return Some(CursorItem::FootnoteDefinition(definition));
    }
    if let Some(heading) = index.headings.iter().find(|heading| at(heading.range)) {
        return Some(CursorItem::Heading(heading));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        CursorItem, LspPosition, MarkdownParser, MdIndex, Range, Target, build, item_at,
        parse_destination, parse_wiki_target, slugify,
    };

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

    /// The item under `position`, flattened to its variant name so tests
    /// can assert the kind with flat comparisons.
    fn kind_at(index: &MdIndex, position: LspPosition) -> Option<&'static str> {
        match item_at(index, position) {
            Some(CursorItem::Heading(_)) => Some("Heading"),
            Some(CursorItem::Link(_)) => Some("Link"),
            Some(CursorItem::Reference(_)) => Some("Reference"),
            Some(CursorItem::Definition(_)) => Some("Definition"),
            Some(CursorItem::FootnoteReference(_)) => Some("FootnoteReference"),
            Some(CursorItem::FootnoteDefinition(_)) => Some("FootnoteDefinition"),
            Some(CursorItem::Wikilink(_)) => Some("Wikilink"),
            None => None,
        }
    }

    #[test]
    fn fixture_collects_headings_definitions_and_links() {
        let index = fixture();
        assert_eq!(index.headings.len(), 3);
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
                .any(|link| link.destination == "guide.md"),
        );
    }

    #[test]
    fn fixture_collects_references_footnotes_and_wikilinks() {
        let index = fixture();
        let mut labels: Vec<&str> = index
            .references
            .iter()
            .map(|reference| reference.label.as_str())
            .collect();
        labels.sort_unstable();
        assert_eq!(labels, vec!["collapsed", "ref", "shortcut"]);
        assert!(index.has_footnote_definition("1"));
        assert_eq!(index.footnote_references.len(), 2);
        assert_eq!(index.footnote_references[0].id, "1");
        assert_eq!(index.footnote_references[1].id, "note");
        assert!(index.has_footnote_definition("note"));
        assert!(!index.has_footnote_definition("Note"));
        let targets: Vec<&str> = index
            .wikilinks
            .iter()
            .map(|wikilink| wikilink.target.as_str())
            .collect();
        assert_eq!(targets, vec!["Other", "folder/note#Section", "LazyWiki"]);
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
            Some(Target::Fragment("top".to_owned())),
        );
        assert_eq!(
            parse_destination("guide.md"),
            Some(Target::Doc {
                path: "guide.md".to_owned(),
                fragment: None,
            }),
        );
        assert_eq!(
            parse_destination("a/b.md#section"),
            Some(Target::Doc {
                path: "a/b.md".to_owned(),
                fragment: Some("section".to_owned()),
            }),
        );
        assert_eq!(
            parse_destination("guide.md#"),
            Some(Target::Doc {
                path: "guide.md".to_owned(),
                fragment: None,
            }),
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
                fragment: None,
            }),
        );
        assert_eq!(
            parse_wiki_target("folder/note#Section"),
            Some(Target::Wiki {
                path: "folder/note".to_owned(),
                fragment: Some("Section".to_owned()),
            }),
        );
        assert_eq!(parse_wiki_target("#Section"), None);
    }

    #[test]
    fn slugify_matches_the_reference_normalization() {
        assert_eq!(slugify("Heading One"), "heading-one");
        assert_eq!(slugify("  Mixed CASE "), "mixed-case");
    }

    #[test]
    fn code_regions_produce_no_shapes() {
        let index = fixture();
        // Only the real links, wikilinks, and footnotes — no decoy.md, WikiDecoy,
        // SpanDecoy, ^99, or ^77.
        assert_eq!(index.links.len(), 2);
        assert!(
            !index
                .links
                .iter()
                .any(|link| link.destination == "decoy.md"),
        );
        let wikis: Vec<&str> = index
            .wikilinks
            .iter()
            .map(|wikilink| wikilink.target.as_str())
            .collect();
        assert!(!wikis.contains(&"WikiDecoy"));
        assert!(!wikis.contains(&"SpanDecoy"));
        let footnotes: Vec<&str> = index
            .footnote_references
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        assert!(!footnotes.contains(&"99"));
        assert!(!footnotes.contains(&"77"));
    }

    #[test]
    fn setext_headings_are_collected() {
        let index = fixture();
        assert_eq!(index.headings.len(), 3);
        let setext = index
            .headings
            .iter()
            .find(|heading| heading.slug == "setext-title")
            .expect("setext heading collected");
        assert_eq!(setext.level, 1);
        let setext_two = index
            .headings
            .iter()
            .find(|heading| heading.slug == "setext-two")
            .expect("level-2 setext heading collected");
        assert_eq!(setext_two.level, 2);
        assert!(index.has_heading_slug("setext-title"));
    }

    #[test]
    fn scanned_points_match_their_source_lines() {
        let text = fixture_text();
        let index = fixture();
        let lazy = index
            .wikilinks
            .iter()
            .find(|wikilink| wikilink.target == "LazyWiki")
            .expect("lazy-continuation wikilink collected");
        let slice = &text[lazy.range.start_byte..lazy.range.end_byte];
        assert_eq!(slice, "[[LazyWiki]]");
        let line_start = text[..lazy.range.start_byte]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        assert_eq!(
            lazy.range.start_point.row,
            text[..lazy.range.start_byte].matches('\n').count(),
        );
        assert_eq!(
            lazy.range.start_point.column,
            lazy.range.start_byte - line_start,
            "continuation-line column is the offset within its own line",
        );
    }

    #[test]
    fn item_at_locates_each_kind_and_misses_plain_text() {
        let index = fixture();
        // The needle's first byte, read back from the fixture text as a
        // UTF-8 (line, byte column) position so the test survives edits.
        let at = |needle: &str| {
            let text = fixture_text();
            let offset = text.find(needle).expect("needle present");
            let line = text[..offset].matches('\n').count();
            let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
            LspPosition {
                line: u32::try_from(line).expect("line fits"),
                character: u32::try_from(offset - line_start).expect("column fits"),
            }
        };

        assert_eq!(kind_at(&index, at("# Top")), Some("Heading"));
        assert_eq!(kind_at(&index, at("[docs](guide.md)")), Some("Link"));
        assert_eq!(kind_at(&index, at("[^1] here")), Some("FootnoteReference"));
        assert_eq!(kind_at(&index, at("[collapsed][]")), Some("Reference"));
        assert_eq!(kind_at(&index, at("[ref]: https")), Some("Definition"));
        assert_eq!(
            kind_at(&index, at("[^1]: footnote")),
            Some("FootnoteDefinition"),
        );
        assert_eq!(kind_at(&index, at("[[Other]]")), Some("Wikilink"));
        // `See` opens a plain sentence — nothing reference- or
        // definition-worthy under it.
        assert_eq!(kind_at(&index, at("See")), None);
    }

    #[test]
    fn rename_sub_ranges_slice_back_to_their_source_text() {
        let text = fixture_text();
        let index = fixture();
        let slice = |range: Range| &text[range.start_byte..range.end_byte];

        let heading = &index.headings[0];
        assert_eq!(slice(heading.content_range), "Top");

        let summary = index
            .links
            .iter()
            .find(|link| link.destination == "#top")
            .expect("fragment link collected");
        assert_eq!(slice(summary.fragment_range.expect("fragment")), "top");

        let other = index
            .wikilinks
            .iter()
            .find(|wikilink| wikilink.target == "folder/note#Section")
            .expect("wikilink collected");
        assert_eq!(slice(other.fragment_range.expect("fragment")), "Section");

        let reference = &index.references[0];
        assert_eq!(slice(reference.range), "[label][ref]");

        let definition = &index.definitions[0];
        assert_eq!(slice(definition.label_range), "ref");

        let footnote = &index.footnote_references[0];
        assert_eq!(slice(footnote.id_range), "1");

        let footnote_definition = &index.footnote_definitions[0];
        assert_eq!(slice(footnote_definition.id_range), "1");

        // `[^b]: x` has single-token content, so it routes through the
        // definition branch; its id range must exclude the caret.
        assert!(index.has_footnote_definition("b"));
        let caret_definition = index
            .footnote_definitions
            .iter()
            .find(|definition| definition.id == "b")
            .expect("caret footnote definition collected");
        assert_eq!(slice(caret_definition.id_range), "b");
    }
}
