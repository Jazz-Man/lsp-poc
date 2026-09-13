//! Broken-link diagnostics, codes 1–6 (see the cycle-1 spec).
//!
//! Invalid link syntax (the reference's code 0) is consciously not
//! reproduced: with a typed parse, malformed links are simply not links.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "model lands before its consumers; wiring lands in cycle-1 task 4"
    )
)]

use async_language_server::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use async_language_server::tree_sitter::Range;
use async_language_server::tree_sitter_utils::ts_range_to_lsp_range;

use crate::links::{self, Target, parse_destination, parse_wiki_target, slugify};
use crate::workspace::Resolved;

/// `Diagnostic::source` value for every diagnostic this server publishes.
pub const SOURCE: &str = "lsp-poc";

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
    check_links(index, resolve, &mut diagnostics);
    check_wikilinks(index, resolve, &mut diagnostics);
    check_references(index, &mut diagnostics);
    check_footnotes(index, &mut diagnostics);
    diagnostics
}

/// Codes 1–3: inline links (fragments, files, and their headings).
fn check_links(
    index: &links::MdIndex,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
    diagnostics: &mut Vec<Diagnostic>,
) {
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
                        diagnostics,
                        &target_index,
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
}

/// Code 6 (and 2): wikilinks against the workspace.
fn check_wikilinks(
    index: &links::MdIndex,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
    diagnostics: &mut Vec<Diagnostic>,
) {
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
                    diagnostics,
                    &target_index,
                    fragment.as_deref(),
                    wikilink.range,
                    &format!("wikilink to non-existent heading `{{}}` in `{path}`"),
                );
            }
        }
    }
}

/// Code 4: reference labels without a definition.
fn check_references(index: &links::MdIndex, diagnostics: &mut Vec<Diagnostic>) {
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
}

/// Code 5: footnote references without a definition.
fn check_footnotes(index: &links::MdIndex, diagnostics: &mut Vec<Diagnostic>) {
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
    diagnostics.push(broken(2, range, template.replace("{}", fragment)));
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
                ref other => panic!("unexpected code: {other:?}"),
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
