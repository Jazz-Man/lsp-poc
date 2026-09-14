//! The `textDocument/definition` capability: what is under the cursor,
//! and where does it point.
//!
//! The position is matched against the requesting document's model items
//! (all ranges are document-absolute), then the item's target resolves to
//! a `Location`. External or unresolved targets answer `None` — absence
//! is not an error.

use async_language_server::lsp_types::{
    GotoDefinitionResponse, Location, Position as LspPosition, Url,
};
use async_language_server::tree_sitter_utils::{
    ts_range_contains_lsp_position, ts_range_to_lsp_range,
};

use crate::links::{self, Target, parse_destination, parse_wiki_target, slugify};
use crate::workspace::Resolved;

/// Answers the definition target for the item under `position`, or `None`
/// when nothing definition-worthy is under it or the target does not
/// resolve. First matching item in collection order wins.
#[must_use]
pub fn at_position(
    index: &links::MdIndex,
    self_url: &Url,
    position: LspPosition,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Option<GotoDefinitionResponse> {
    let at = |range: async_language_server::tree_sitter::Range| {
        ts_range_contains_lsp_position(range, position)
    };

    if let Some(link) = index.links().iter().find(|link| at(link.range)) {
        let target = parse_destination(&link.destination)?;
        return route(index, self_url, &target, resolve);
    }
    // Footnote and wikilink shapes surface as reference labels; their own
    // arms below own them.
    if let Some(reference) = index
        .references()
        .iter()
        .find(|reference| at(reference.range))
        && !reference.label.starts_with(['^', '['])
    {
        let definition = index
            .definitions()
            .iter()
            .find(|definition| definition.label == reference.label)?;
        return Some(scalar(self_url, definition.range));
    }
    if let Some(footnote) = index
        .footnote_references()
        .iter()
        .find(|footnote| at(footnote.range))
    {
        let definition = index
            .footnote_definitions()
            .iter()
            .find(|definition| definition.id == footnote.id)?;
        return Some(scalar(self_url, definition.range));
    }
    if let Some(wikilink) = index.wikilinks().iter().find(|wikilink| at(wikilink.range)) {
        let target = parse_wiki_target(&wikilink.target)?;
        let Target::Wiki { .. } = &target else {
            return None;
        };
        return route(index, self_url, &target, resolve);
    }
    None
}

/// Routes a parsed target to its `Location`: own-document fragments land
/// on the first matching heading; file and wikilink targets land on the
/// resolved document's matching heading — or, with no fragment, its first
/// heading, or the document start when it has no headings.
fn route(
    index: &links::MdIndex,
    self_url: &Url,
    target: &Target,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Option<GotoDefinitionResponse> {
    match target {
        Target::Fragment(fragment) => {
            let heading = heading_by_slug(index, &slugify(fragment))?;
            Some(scalar(self_url, heading.range))
        }
        Target::Doc { fragment, .. } | Target::Wiki { fragment, .. } => {
            let Some(Resolved::Found {
                url,
                index: target_index,
            }) = resolve(target)
            else {
                return None;
            };
            let range = match fragment {
                Some(fragment) => heading_by_slug(&target_index, &slugify(fragment))?.range,
                None => target_index
                    .headings()
                    .first()
                    .map_or_else(zero_range, |heading| heading.range),
            };
            Some(scalar(&url, range))
        }
    }
}

/// First heading whose slug matches, in document order.
fn heading_by_slug<'a>(index: &'a links::MdIndex, slug: &str) -> Option<&'a links::Heading> {
    index.headings().iter().find(|heading| heading.slug == slug)
}

fn scalar(url: &Url, range: async_language_server::tree_sitter::Range) -> GotoDefinitionResponse {
    GotoDefinitionResponse::Scalar(Location {
        uri: url.clone(),
        range: ts_range_to_lsp_range(range),
    })
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

#[cfg(test)]
mod tests {
    use super::{
        GotoDefinitionResponse, Location, LspPosition, Resolved, Target, Url, at_position, links,
    };
    use std::sync::Arc;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    fn parse(text: &str) -> MdIndex {
        let mut parser = MarkdownParser::default();
        links::build(&mut parser, text).expect("input parses")
    }

    fn location_of(index: &MdIndex, url: &Url, line: u32, character: u32) -> Option<Location> {
        let resolve = |_: &Target| {
            Some(Resolved::Found {
                url: Url::parse("file:///target.md").expect("url parses"),
                index: Arc::new(parse("# Target\n")),
            })
        };
        match at_position(index, url, LspPosition { line, character }, &resolve) {
            Some(GotoDefinitionResponse::Scalar(location)) => Some(location),
            Some(other) => panic!("unexpected response shape: {other:?}"),
            None => None,
        }
    }

    #[test]
    fn fragment_link_routes_to_the_own_document_heading() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("# Top\n\n[summary](#top)\n");
        // Cursor inside `[summary](#top)` — the link's range covers it.
        let location = location_of(&index, &url, 2, 3).expect("definition resolves");
        assert_eq!(location.uri, url);
        // The `# Top` heading's range starts at the document's first byte.
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
    }

    #[test]
    fn cross_file_link_routes_to_the_target_document() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        // The fragment must name a heading the stubbed target document has.
        let index = parse("file [f](other.md#target)\n");
        let location = location_of(&index, &url, 0, 8).expect("definition resolves");
        assert_eq!(
            location.uri,
            Url::parse("file:///target.md").expect("url parses"),
        );
        assert_eq!(location.range.start.line, 0);
    }

    #[test]
    fn reference_and_footnote_routes_land_on_their_definitions() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[label][ref]\n\n[ref]: https://example.com\n\nfoot[^1]\n\n[^1]: text\n");
        let reference = location_of(&index, &url, 0, 2).expect("reference resolves");
        assert_eq!(reference.uri, url);
        // Cursor inside `[^1]` — the reference spans columns 4–8 of its line.
        let footnote = location_of(&index, &url, 4, 5).expect("footnote resolves");
        assert_eq!(footnote.uri, url);
        // Both land on their own definitions: the `[ref]:` line and the
        // `[^1]:` line (definition nodes include the trailing newline, so
        // a character-only spread is not a meaningful non-degeneracy test).
        assert_eq!(reference.range.start.line, 2);
        assert_eq!(footnote.range.start.line, 6);
    }

    #[test]
    fn unresolved_targets_answer_none() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[f](nowhere.md)\n");
        let resolve = |_: &Target| Some(Resolved::Missing);
        assert!(
            at_position(
                &index,
                &url,
                LspPosition {
                    line: 0,
                    character: 3
                },
                &resolve,
            )
            .is_none(),
        );
        // Cursor on the link itself with a None-resolver: the target
        // parses, but route's unresolved arm answers None.
        let no_resolve = |_: &Target| -> Option<Resolved> { None };
        assert!(
            at_position(
                &index,
                &url,
                LspPosition {
                    line: 0,
                    character: 3
                },
                &no_resolve,
            )
            .is_none(),
        );
        // Plain text under the cursor is not definition-worthy either.
        assert!(
            at_position(
                &index,
                &url,
                LspPosition {
                    line: 5,
                    character: 0
                },
                &no_resolve,
            )
            .is_none(),
        );
    }
}
