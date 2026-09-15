//! The `textDocument/rename` capability: a cursor on a renameable item
//! becomes a per-file edit map, sorted by descending start position.
//!
//! Routes mirror references: a heading renames its text plus every
//! inbound fragment across the candidate documents (each resolved in its
//! own context); reference/definition and footnote-ref/footnote-def
//! pairs rename together; a link/wikilink renames its fragment, or its
//! target text when the wikilink has no fragment. Fragment-less file
//! links and web destinations answer `None`.

use std::collections::HashMap;
use std::sync::Arc;

use async_language_server::lsp_types::{Position as LspPosition, TextEdit, Url, WorkspaceEdit};
use async_language_server::tree_sitter::Range;
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{
    self, CursorItem, Target, item_at, parse_destination, parse_wiki_target, slugify,
};
use crate::workspace::Resolved;

/// One candidate document for cross-file heading renames.
#[derive(Debug)]
pub struct DocumentSnapshot {
    pub url: Url,
    pub index: Arc<links::MdIndex>,
}

/// The renameable sub-range under `position`, or `None` when nothing
/// renameable is under it. The sub-range is what `textDocument/rename`
/// replaces; the server slices the document text at it for the
/// prepare-rename placeholder.
#[must_use]
pub fn prepare_at(index: &links::MdIndex, position: LspPosition) -> Option<Range> {
    match item_at(index, position)? {
        CursorItem::Heading(heading) => Some(heading.content_range),
        CursorItem::Link(link) => {
            // Only the shape the rename route accepts — a document target
            // carrying a fragment — offers a rename: same-document
            // fragments, fragment-less file links, and web destinations
            // decline with it.
            let Some(Target::Doc {
                fragment: Some(_), ..
            }) = parse_destination(&link.destination)
            else {
                return None;
            };
            Some(link.fragment_range?)
        }
        CursorItem::Reference(reference) => Some(reference.label_range),
        CursorItem::Definition(definition) => Some(definition.label_range),
        CursorItem::FootnoteReference(footnote) => Some(footnote.id_range),
        CursorItem::FootnoteDefinition(definition) => Some(definition.id_range),
        CursorItem::Wikilink(wikilink) => {
            let Some(Target::Wiki { fragment, .. }) = parse_wiki_target(&wikilink.target) else {
                return None;
            };
            // The same arms the rename route matches: a fragment renames
            // the heading it points to, the fragment-less wikilink edits
            // its inner target text, and a parse-visible fragment the
            // scan could not range edits nothing.
            match (wikilink.fragment_range, fragment) {
                (Some(fragment_range), Some(_)) => Some(fragment_range),
                (_, None) => Some(wikilink.target_range),
                (None, Some(_)) => None,
            }
        }
    }
}

/// Computes the rename edit map for the item under `position`, or `None`
/// when nothing renameable is under it or the new name is empty. Edits
/// within each file are sorted by descending start position.
#[expect(
    clippy::too_many_arguments,
    reason = "the arity is the references sibling's shape plus the new name; task 3 consumes it as written"
)]
#[must_use]
pub fn at_position(
    index: &links::MdIndex,
    self_url: &Url,
    position: LspPosition,
    new_name: &str,
    documents: &[DocumentSnapshot],
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
    resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
) -> Option<WorkspaceEdit> {
    if new_name.is_empty() {
        return None;
    }
    let context = Context {
        documents,
        resolve,
        resolve_from,
        new_name,
    };
    let mut edits: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    match item_at(index, position)? {
        CursorItem::Heading(heading) => {
            heading_rename_edits(heading, self_url, &context, &mut edits);
        }
        CursorItem::Reference(reference) => {
            push_edit(&mut edits, self_url, reference.label_range, new_name);
            let definition = index
                .definitions()
                .iter()
                .find(|definition| definition.label == reference.label)?;
            push_edit(&mut edits, self_url, definition.label_range, new_name);
        }
        CursorItem::Definition(definition) => {
            push_edit(&mut edits, self_url, definition.label_range, new_name);
            for reference in index.references() {
                if reference.label == definition.label {
                    push_edit(&mut edits, self_url, reference.label_range, new_name);
                }
            }
        }
        CursorItem::FootnoteReference(footnote) => {
            push_edit(&mut edits, self_url, footnote.id_range, new_name);
            let definition = index
                .footnote_definitions()
                .iter()
                .find(|definition| definition.id == footnote.id)?;
            push_edit(&mut edits, self_url, definition.id_range, new_name);
        }
        CursorItem::FootnoteDefinition(definition) => {
            push_edit(&mut edits, self_url, definition.id_range, new_name);
            for footnote in index.footnote_references() {
                if footnote.id == definition.id {
                    push_edit(&mut edits, self_url, footnote.id_range, new_name);
                }
            }
        }
        CursorItem::Link(link) => {
            // Renaming a link with a fragment renames the heading it
            // points to (plus every inbound fragment) — renaming only the
            // fragment would break the link. Fragment-less file links and
            // same-document fragments answer None: file renames are
            // `workspace/willRename`, out of scope.
            let target = parse_destination(&link.destination)?;
            delegate_edits(&target, &context, &mut edits)?;
        }
        CursorItem::Wikilink(wikilink) => {
            let target = parse_wiki_target(&wikilink.target)?;
            let Target::Wiki { fragment, .. } = &target else {
                return None;
            };
            match (wikilink.fragment_range, fragment.as_deref()) {
                // A wikilink with a fragment renames the heading it points
                // to (plus every inbound fragment), like a link.
                (Some(_), Some(_)) => {
                    delegate_edits(&target, &context, &mut edits)?;
                }
                // A fragment-less wikilink edits its own target text (the
                // physical file is not renamed) — the inner bytes, so the
                // brackets stay and the link survives.
                (_, None) => push_edit(&mut edits, self_url, wikilink.target_range, new_name),
                // The parse sees a fragment the scan could not range —
                // nothing to edit.
                (None, Some(_)) => return None,
            }
        }
    }
    Some(into_workspace_edit(edits))
}

/// What the rename routes share: the candidate documents, both target
/// resolvers, and the new name.
struct Context<'a> {
    documents: &'a [DocumentSnapshot],
    resolve: &'a dyn Fn(&Target) -> Option<Resolved>,
    resolve_from: &'a dyn Fn(&Url, &Target) -> Option<Resolved>,
    new_name: &'a str,
}

/// Renames the heading `target` points at (plus every inbound fragment),
/// the delegate route links and wikilinks share. `None` when the target
/// carries no fragment or does not resolve to one of its document's
/// headings.
fn delegate_edits(
    target: &Target,
    context: &Context,
    edits: &mut HashMap<Url, Vec<TextEdit>>,
) -> Option<()> {
    let fragment = match target {
        Target::Doc {
            fragment: Some(fragment),
            ..
        }
        | Target::Wiki {
            fragment: Some(fragment),
            ..
        } => fragment,
        Target::Fragment(_)
        | Target::Doc { fragment: None, .. }
        | Target::Wiki { fragment: None, .. } => return None,
    };
    let Some(Resolved::Found {
        url: target_url,
        index: target_index,
    }) = (context.resolve)(target)
    else {
        return None;
    };
    let target_heading = target_index.heading_by_slug(&slugify(fragment))?;
    heading_rename_edits(target_heading, &target_url, context, edits);
    Some(())
}

/// The finished edit map, each file's edits sorted by descending start
/// position.
fn into_workspace_edit(mut edits: HashMap<Url, Vec<TextEdit>>) -> WorkspaceEdit {
    for edits_in_file in edits.values_mut() {
        edits_in_file.sort_unstable_by(|left, right| {
            right
                .range
                .start
                .line
                .cmp(&left.range.start.line)
                .then(right.range.start.character.cmp(&left.range.start.character))
        });
    }
    WorkspaceEdit {
        changes: Some(edits),
        ..WorkspaceEdit::default()
    }
}

/// The heading-rename edit set: the heading's own text plus every
/// link/wikilink fragment that lands on it across the candidate
/// documents, each resolved in its own document's context.
fn heading_rename_edits(
    heading: &links::Heading,
    heading_url: &Url,
    context: &Context,
    edits: &mut HashMap<Url, Vec<TextEdit>>,
) {
    push_edit(edits, heading_url, heading.content_range, context.new_name);
    let slug = heading.slug.as_str();
    for document in context.documents {
        for link in document.index.links() {
            let Some(target) = parse_destination(&link.destination) else {
                continue;
            };
            if lands_on(
                slug,
                heading_url,
                &document.url,
                &target,
                context.resolve_from,
            ) {
                let Some(fragment_range) = link.fragment_range else {
                    continue;
                };
                push_edit(
                    edits,
                    &document.url,
                    fragment_range,
                    slugify(context.new_name),
                );
            }
        }
        for wikilink in document.index.wikilinks() {
            let Some(target) = parse_wiki_target(&wikilink.target) else {
                continue;
            };
            if lands_on(
                slug,
                heading_url,
                &document.url,
                &target,
                context.resolve_from,
            ) {
                let Some(fragment_range) = wikilink.fragment_range else {
                    continue;
                };
                push_edit(
                    edits,
                    &document.url,
                    fragment_range,
                    slugify(context.new_name),
                );
            }
        }
    }
}

/// Registers one edit for `url`, converting the model range to LSP
/// coordinates.
fn push_edit(
    edits: &mut HashMap<Url, Vec<TextEdit>>,
    url: &Url,
    range: Range,
    new_text: impl Into<String>,
) {
    edits.entry(url.clone()).or_default().push(TextEdit {
        range: ts_range_to_lsp_range(range),
        new_text: new_text.into(),
    });
}

/// `true` when `target`, resolved in the candidate document's context,
/// lands on the heading named `slug` in the requesting document.
fn lands_on(
    slug: &str,
    self_url: &Url,
    candidate_url: &Url,
    target: &Target,
    resolve_from: &dyn Fn(&Url, &Target) -> Option<Resolved>,
) -> bool {
    match target {
        Target::Fragment(fragment) => candidate_url == self_url && slugify(fragment) == slug,
        Target::Doc { fragment, .. } | Target::Wiki { fragment, .. } => {
            let Some(Resolved::Found { url, .. }) = resolve_from(candidate_url, target) else {
                return false;
            };
            url == *self_url
                && fragment
                    .as_deref()
                    .is_some_and(|fragment| slugify(fragment) == slug)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentSnapshot, HashMap, LspPosition, Resolved, Target, TextEdit, Url, at_position,
        links, prepare_at,
    };
    use std::sync::Arc;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    /// Column of the cursor inside the `[label][ref]` reference usage
    /// (line 2 of [`self_doc`]'s text).
    const REFERENCE_CURSOR_COLUMN: u32 = 20;

    fn parse(text: &str) -> Arc<MdIndex> {
        let mut parser = MarkdownParser::default();
        Arc::new(links::build(&mut parser, text).expect("input parses"))
    }

    fn self_doc() -> (Url, Arc<MdIndex>) {
        (
            Url::parse("file:///doc.md").expect("url parses"),
            parse("# Top\n\n[summary](#top) [label][ref]\n\n[ref]: a.md\n\nfoot[^1]\n\n[^1]: t\n"),
        )
    }

    fn resolver_stub(_target: &Target) -> Option<Resolved> {
        Some(Resolved::Found {
            url: Url::parse("file:///target.md").expect("url parses"),
            index: parse("# Target\n"),
        })
    }

    fn edits_of(
        index: &MdIndex,
        self_url: &Url,
        line: u32,
        character: u32,
        new_name: &str,
        documents: &[DocumentSnapshot],
    ) -> Option<HashMap<Url, Vec<TextEdit>>> {
        let resolve = resolver_stub;
        // Candidate links resolve back onto the requesting document —
        // that is what `lands_on` matches against.
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
            new_name,
            documents,
            &resolve,
            &resolve_from,
        )
        .and_then(|edit| edit.changes)
    }

    #[test]
    fn heading_rename_edits_text_and_inbound_fragments() {
        let (self_url, index) = self_doc();
        let other_url = Url::parse("file:///other.md").expect("url parses");
        let documents = vec![
            DocumentSnapshot {
                url: self_url.clone(),
                index: Arc::clone(&index),
            },
            DocumentSnapshot {
                url: other_url.clone(),
                index: parse("[x](doc.md#top)\n"),
            },
        ];
        let edits =
            edits_of(&index, &self_url, 0, 2, "Renamed", &documents).expect("heading rename works");
        // The heading text and both inbound fragments.
        assert_eq!(edits[&self_url].len(), 2);
        assert_eq!(edits[&other_url].len(), 1);
        assert_eq!(edits[&other_url][0].new_text, "renamed");
    }

    #[test]
    fn reference_and_definition_rename_in_pairs() {
        let (self_url, index) = self_doc();
        // Cursor inside `[label][ref]` (columns 17–29 of line 2) — on the
        // reference usage, not on the leading inline link.
        let edits = edits_of(
            &index,
            &self_url,
            2,
            REFERENCE_CURSOR_COLUMN,
            "renamed-ref",
            &documents_empty(),
        )
        .expect("reference rename resolves");
        assert_eq!(edits.len(), 1);
        // The usage label and the definition label: two edits, same file.
        assert_eq!(edits[&self_url].len(), 2);
        assert!(
            edits[&self_url]
                .iter()
                .all(|edit| edit.new_text == "renamed-ref"),
        );
    }

    #[test]
    fn footnote_rename_edits_reference_and_definition() {
        let (self_url, index) = self_doc();
        // Cursor inside `[^1]` (columns 4–8 of line 6) — col 6 is the id.
        let edits = edits_of(&index, &self_url, 6, 6, "2", &documents_empty())
            .expect("footnote rename resolves");
        assert_eq!(edits[&self_url].len(), 2);
    }

    #[test]
    fn wikilink_with_fragment_renames_the_target_heading() {
        // The wikilink resolves to a target document whose `Target`
        // heading exists: the rename applies THERE (the delegate route).
        let index = parse("see [[doc#Target]]\n");
        let edits = edits_of(&index, &self_doc().0, 0, 8, "Renamed", &documents_empty())
            .expect("wikilink fragment rename resolves");
        assert_eq!(edits.len(), 1);
        let target_key = Url::parse("file:///target.md").expect("url parses");
        assert_eq!(edits[&target_key].len(), 1);
        assert_eq!(edits[&target_key][0].new_text, "Renamed");
    }

    #[test]
    fn wikilink_without_fragment_edits_its_own_target_text() {
        let (self_url, _) = self_doc();
        let text = "see [[My Note]] here\n";
        let index = parse(text);
        let bracket = text.find("[[My Note]]").expect("wikilink present");
        let edits = edits_of(&index, &self_url, 0, 8, "Renamed Note", &documents_empty())
            .expect("wikilink target rename resolves");
        assert_eq!(edits[&self_url].len(), 1);
        let edit = &edits[&self_url][0];
        assert_eq!(edit.new_text, "Renamed Note");
        // The inner target text is replaced; the `[[`/`]]` brackets stay.
        assert_eq!(
            edit.range.start.character,
            u32::try_from(bracket + 2).expect("offset fits"),
        );
        assert_eq!(
            edit.range.end.character,
            u32::try_from(bracket + 2 + "My Note".len()).expect("offset fits"),
        );
    }

    #[test]
    fn link_fragment_rename_renames_the_target_heading() {
        // The link resolves to a target document whose `Target` heading
        // exists: the rename applies THERE (the delegate route).
        let index = parse("[f](other.md#Target)\n");
        let edits = edits_of(&index, &self_doc().0, 0, 8, "Renamed", &documents_empty())
            .expect("link fragment rename resolves");
        assert_eq!(edits.len(), 1);
        let target_key = Url::parse("file:///target.md").expect("url parses");
        assert_eq!(edits[&target_key].len(), 1);
        assert_eq!(edits[&target_key][0].new_text, "Renamed");
    }

    #[test]
    fn fragment_less_file_link_answers_none() {
        let (self_url, _) = self_doc();
        let index = parse("see [f](other.md)\n");
        assert!(
            edits_of(&index, &self_url, 0, 8, "x", &documents_empty()).is_none(),
            "file renames are workspace/willRename, out of scope",
        );
    }

    #[test]
    fn edits_are_sorted_by_descending_start_within_a_file() {
        let (self_url, index) = self_doc();
        let edits = edits_of(
            &index,
            &self_url,
            2,
            REFERENCE_CURSOR_COLUMN,
            "renamed-ref",
            &documents_empty(),
        )
        .expect("reference rename resolves");
        let starts: Vec<u32> = edits[&self_url]
            .iter()
            .map(|edit| edit.range.start.line)
            .collect();
        let mut sorted = starts.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(starts, sorted, "descending start order");
    }

    #[test]
    fn prepare_at_answers_the_renameable_sub_range() {
        let index = parse("# Top\n\nsome prose\n\n[summary](doc.md#top)\n");
        // Heading: the content range, not the `#` marker.
        let heading = prepare_at(
            &index,
            LspPosition {
                line: 0,
                character: 2,
            },
        )
        .expect("heading sub-range");
        assert_eq!(heading.start_point.row, 0);
        assert_eq!(heading.start_point.column, 2);
        assert_eq!(heading.end_point.column, 5);
        // Link: the fragment range inside the destination. The cursor sits
        // on the fragment text (columns 17–19); the same-document fragment
        // shape declines instead — see
        // `prepare_at_declines_shapes_the_rename_route_declines`.
        let fragment = prepare_at(
            &index,
            LspPosition {
                line: 4,
                character: 19,
            },
        )
        .expect("fragment sub-range");
        assert_eq!(fragment.start_point.column, 17);
        assert_eq!(fragment.end_point.column, 20);
        // Plain text: nothing renameable.
        assert!(
            prepare_at(
                &index,
                LspPosition {
                    line: 2,
                    character: 1,
                },
            )
            .is_none(),
        );
    }

    #[test]
    fn prepare_at_answers_wikilink_ranges_coherently_with_the_rename_edit() {
        let text = "see [[My Note]] and [[doc#Target]]\n";
        let index = parse(text);
        // Fragment-less wikilink: the prepare range is the inner target
        // text — exactly what the rename edit replaces; the brackets stay.
        let target = prepare_at(
            &index,
            LspPosition {
                line: 0,
                character: 8,
            },
        )
        .expect("target sub-range");
        assert_eq!(&text[target.start_byte..target.end_byte], "My Note");
        // With a fragment: the fragment bytes.
        let fragment = prepare_at(
            &index,
            LspPosition {
                line: 0,
                character: 24,
            },
        )
        .expect("fragment sub-range");
        assert_eq!(&text[fragment.start_byte..fragment.end_byte], "Target");
    }

    #[test]
    fn prepare_at_declines_shapes_the_rename_route_declines() {
        let index = parse("[x](#top)\n\n[site](https://example.com#s)\n\nsee [[#frag]]\n");
        // Same-document fragment link: `parse_destination` answers
        // `Target::Fragment`, which the delegate route cannot rename.
        assert!(
            prepare_at(
                &index,
                LspPosition {
                    line: 0,
                    character: 4,
                },
            )
            .is_none(),
            "same-document fragment links are not renameable",
        );
        // Web destination with a fragment: `parse_destination` answers
        // None outright.
        assert!(
            prepare_at(
                &index,
                LspPosition {
                    line: 2,
                    character: 20,
                },
            )
            .is_none(),
            "web destinations are not renameable",
        );
        // Same-document wikilink: `parse_wiki_target` answers None for
        // the empty path.
        assert!(
            prepare_at(
                &index,
                LspPosition {
                    line: 4,
                    character: 8,
                },
            )
            .is_none(),
            "same-document wikilinks are not renameable",
        );
    }

    fn documents_empty() -> Vec<DocumentSnapshot> {
        Vec::new()
    }
}
