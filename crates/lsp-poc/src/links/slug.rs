//! GitHub-style anchor slugs, unicode-preserving.
//!
//! Trim, lowercase, whitespace runs collapse to one dash, keep unicode
//! alphanumeric plus `_`/`-`, drop the rest, trim edge dashes. Pure
//! string transform — no positional math; UTF-8 position handling stays
//! at the framework boundary.

/// GitHub-style anchor: trim, lowercase, whitespace runs collapse to one
/// dash, keep unicode alphanumeric plus `_`/`-`, drop the rest, trim edge
/// dashes. One function serves both matching sides — heading collection
/// and fragment resolution.
#[must_use]
pub fn slugify(heading: &str) -> String {
    let mut slug = String::with_capacity(heading.len());
    let mut pending_dash = false;
    for character in heading.trim().chars() {
        if character.is_whitespace() {
            pending_dash = !slug.is_empty();
            continue;
        }
        if !(character.is_alphanumeric() || character == '_' || character == '-') {
            continue;
        }
        if pending_dash {
            slug.push('-');
            pending_dash = false;
        }
        slug.extend(character.to_lowercase());
    }
    slug.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn existing_fixture_slugs_are_unchanged() {
        assert_eq!(slugify("Top"), "top");
        assert_eq!(slugify("Setext Title"), "setext-title");
        assert_eq!(slugify("Heading One"), "heading-one");
    }

    #[test]
    fn unicode_headings_keep_their_letters() {
        assert_eq!(slugify("Привіт, світ"), "привіт-світ");
        assert_eq!(slugify("  НОТАТКИ і ідеї "), "нотатки-і-ідеї");
    }

    #[test]
    fn punctuation_drops_and_whitespace_runs_collapse() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("A  B"), "a-b");
        assert_eq!(slugify("a\t\nb"), "a-b");
    }

    #[test]
    fn edge_dashes_trim_and_underscores_survive() {
        assert_eq!(slugify("-- x --"), "x");
        assert_eq!(slugify("snake_case heading"), "snake_case-heading");
        assert_eq!(slugify("   "), "");
    }
}
