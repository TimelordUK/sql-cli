//! The single source of truth for "does this column name have to be quoted?".
//!
//! Anything that splices a column name back into query text — tab completion,
//! `SELECT *` expansion, the pretty-printer — has to agree on this, or the
//! user ends up with a query the parser cannot read. `data/countries.csv` is
//! the standing example: `name.official`, `idd.root` and 60-odd
//! `translations.*.common` columns are all unparseable bare.
//!
//! The rules mirror [`Lexer::read_identifier`](crate::sql::parser::lexer), which
//! is what actually decides whether a bare word survives: alphanumeric (Unicode
//! included, so `país` is fine) plus `_`, not starting with a digit. Keyword
//! status comes from [`Token::from_keyword`] rather than a second hand-kept
//! list, so a column called `row` or `end` is quoted for as long as the lexer
//! reserves those words and no longer.

use crate::sql::parser::lexer::Token;

/// Check if an identifier must be quoted to survive a round trip through the
/// parser.
#[must_use]
pub fn needs_quoting(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }

    // A leading digit makes the lexer read a number, not an identifier.
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return true;
    }

    // Anything the lexer would stop reading at: '.', ' ', '-', '(', '"', ...
    if !name.chars().all(is_identifier_char) {
        return true;
    }

    // A bare keyword would tokenize as that keyword rather than a column.
    Token::from_keyword(name).is_some()
}

/// Quote an identifier unconditionally, doubling any embedded `"` the way SQL
/// expects (`Has"Quote` -> `"Has""Quote"`).
#[must_use]
pub fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Quote an identifier, but only if it would not parse bare.
#[must_use]
pub fn quote_if_needed(name: &str) -> String {
    if needs_quoting(name) {
        quote_identifier(name)
    } else {
        name.to_string()
    }
}

/// The characters the lexer will accept inside an unquoted identifier.
fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_identifiers_are_left_alone() {
        for name in ["City", "customer_id", "tld", "cca2", "unMember", "x1"] {
            assert!(!needs_quoting(name), "{name} should not need quoting");
            assert_eq!(quote_if_needed(name), name);
        }
    }

    #[test]
    fn dotted_names_need_quoting() {
        // The countries.csv case that started this.
        assert!(needs_quoting("name.official"));
        assert_eq!(quote_if_needed("name.common"), "\"name.common\"");
        assert_eq!(
            quote_if_needed("translations.ara.official"),
            "\"translations.ara.official\""
        );
    }

    #[test]
    fn punctuation_and_spaces_need_quoting() {
        assert!(needs_quoting("Phone 1"));
        assert!(needs_quoting("Customer-ID"));
        assert!(needs_quoting("Price ($)"));
        assert!(needs_quoting("a/b"));
        assert!(needs_quoting("col[0]"));
    }

    #[test]
    fn leading_digit_needs_quoting() {
        assert!(needs_quoting("2024"));
        assert!(needs_quoting("1st_place"));
        assert!(!needs_quoting("q1"));
    }

    #[test]
    fn keywords_need_quoting_case_insensitively() {
        assert!(needs_quoting("order"));
        assert!(needs_quoting("ORDER"));
        assert!(needs_quoting("Row"));
        assert!(needs_quoting("end"));
        // Not reserved by this lexer, so leave it bare.
        assert!(!needs_quoting("status"));
        assert!(!needs_quoting("independent"));
    }

    #[test]
    fn unicode_identifiers_do_not_need_quoting() {
        assert!(!needs_quoting("país"));
        assert!(!needs_quoting("naïve_count"));
    }

    #[test]
    fn embedded_quotes_are_doubled() {
        assert_eq!(quote_if_needed("Has\"Quote"), "\"Has\"\"Quote\"");
    }

    #[test]
    fn empty_name_is_quoted_rather_than_emitted_bare() {
        assert_eq!(quote_if_needed(""), "\"\"");
    }
}
