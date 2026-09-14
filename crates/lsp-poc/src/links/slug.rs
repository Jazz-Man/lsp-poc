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
    fn slugify_matches_github_style_slugs() {
        let cases = [
            ("Top", "top"),
            ("Setext Title", "setext-title"),
            ("Heading One", "heading-one"),
            ("Привіт, світ", "привіт-світ"),
            ("  НОТАТКИ і ідеї ", "нотатки-і-ідеї"),
            ("Hello, World!", "hello-world"),
            ("A  B", "a-b"),
            ("a\t\nb", "a-b"),
            ("-- x --", "x"),
            ("snake_case heading", "snake_case-heading"),
            ("   ", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(slugify(input), expected, "input: {input:?}");
        }
    }
}
