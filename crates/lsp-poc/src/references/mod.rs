//! The `textDocument/references` capability: who points at what.
//!
//! A heading answers with every link/wikilink that lands on it across
//! the candidate documents, each link resolved in its own document's
//! context. A link reference definition or footnote definition answers
//! with its same-document usages. A link or wikilink answers with its
//! target — web and unparseable destinations are excluded by the target
//! parsers. Absence is `None`.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the capability lands before its consumers; server wiring lands in cycle-3 task 3"
    )
)]

use std::sync::Arc;

use async_language_server::lsp_types::{Location, Position as LspPosition, Url};
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{
    self, CursorItem, Target, item_at, parse_destination, parse_wiki_target, slugify, zero_range,
};
use crate::workspace::Resolved;

/// One candidate document for heading-reference scans.
#[derive(Debug)]
pub struct DocumentSnapshot {
    pub url: Url,
    pub index: Arc<links::MdIndex>,
}

/// Answers the references for the item under `position`, or `None` when
/// nothing reference-worthy is under it or no reference exists. `resolve`
/// answers targets in the requesting document's context; `resolve_from`
/// answers them in a candidate document's context.
#[must_use]
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
            url == *self_url
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
            let Some(Resolved::Found {
                url,
                index: target_index,
            }) = resolve(target)
            else {
                return Vec::new();
            };
            let range = match fragment {
                Some(fragment) => {
                    let Some(heading) = target_index.heading_by_slug(&slugify(fragment)) else {
                        return Vec::new();
                    };
                    heading.range
                }
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

#[cfg(test)]
mod tests {
    use super::{
        DocumentSnapshot, Location, LspPosition, Resolved, Target, Url, at_position, links,
    };
    use std::sync::Arc;
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
        let resolve_from = |_: &Url, _: &Target| {
            Some(Resolved::Found {
                url: self_url.clone(),
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
                .any(|location| location.uri == self_url && location.range.start.line == 2),
            "the own-document link is included",
        );
        assert!(
            locations
                .iter()
                .all(|location| location.uri != other_url || location.range.start.line == 0),
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
        assert!(references_of(&index, &self_url, 4, 0, &documents_empty()).is_none());
    }

    fn documents_empty() -> Vec<DocumentSnapshot> {
        Vec::new()
    }
}
