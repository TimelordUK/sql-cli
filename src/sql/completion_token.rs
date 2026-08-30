//! Identifier token scanning for tab completion.
//!
//! Completion has two halves that must agree: the parser decides *what* to
//! suggest, and the editor decides *which span of text* the suggestion
//! replaces. Historically each half scanned backwards from the cursor with its
//! own rules, and they disagreed whenever quotes or dots were involved -
//! cycling through `"name.common"` produced `"name.commonname.official"`, and
//! completing `name.com` produced `name."name.common"`.
//!
//! This module is the single scanner both halves use.

/// The identifier-ish text immediately before the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionToken {
    /// Byte offset in the query where the token starts. A suggestion replaces
    /// `query[start..cursor_pos]`.
    pub start: usize,
    /// The raw text of the token, quotes included.
    pub text: String,
    /// True when the token is (or opens) a double-quoted identifier.
    pub is_quoted: bool,
}

impl CompletionToken {
    /// The token with any surrounding quotes removed, for matching against
    /// schema column names. Doubled `""` inside a quoted identifier collapses
    /// back to a single `"`.
    #[must_use]
    pub fn unquoted(&self) -> String {
        if !self.is_quoted {
            return self.text.clone();
        }
        let inner = self
            .text
            .strip_prefix('"')
            .unwrap_or(&self.text)
            .strip_suffix('"')
            .unwrap_or_else(|| self.text.strip_prefix('"').unwrap_or(&self.text));
        inner.replace("\"\"", "\"")
    }

    /// The segment after the last dot, e.g. `Con` for `name.Con`. Used when the
    /// dotted token turns out to be a method call rather than a column name.
    #[must_use]
    pub fn last_segment(&self) -> Option<(usize, String)> {
        let text = &self.text;
        text.rfind('.')
            .map(|dot| (self.start + dot + 1, text[dot + 1..].to_string()))
    }
}

/// A character that can appear unquoted inside a column reference.
fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '.'
}

/// Scan backwards from `cursor_pos` for the identifier the user is completing.
///
/// Handles three shapes:
/// * an open quoted identifier - `SELECT "name.com|` - the token starts at the
///   opening quote and runs to the cursor, spaces and dots included;
/// * a closed quoted identifier the cursor sits just after - `SELECT
///   "name.common"|` - the token is the whole quoted identifier, so a second
///   Tab replaces it rather than appending to it;
/// * a bare identifier - `SELECT name.com|` - dots are part of the token,
///   because `name.common` is one column name, not a method call on `name`.
///
/// Returns `None` when the cursor is not adjacent to an identifier (e.g. after
/// a space or comma), which means the suggestion is inserted rather than
/// replacing anything.
#[must_use]
pub fn find_completion_token(query: &str, cursor_pos: usize) -> Option<CompletionToken> {
    let cursor_pos = cursor_pos.min(query.len());
    if cursor_pos == 0 || !query.is_char_boundary(cursor_pos) {
        return None;
    }
    let prefix = &query[..cursor_pos];

    // Walk the prefix tracking quote state so we know whether the cursor is
    // inside a quoted identifier, and where the current/last one started.
    let mut in_quote = false;
    let mut quote_start = 0usize;
    let mut last_closed: Option<(usize, usize)> = None; // (start, end_exclusive)
    let bytes = prefix.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if in_quote {
                // `""` inside a quoted identifier is an escaped quote.
                if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    i += 2;
                    continue;
                }
                in_quote = false;
                last_closed = Some((quote_start, i + 1));
            } else {
                in_quote = true;
                quote_start = i;
            }
        }
        i += 1;
    }

    if in_quote {
        return Some(CompletionToken {
            start: quote_start,
            text: prefix[quote_start..].to_string(),
            is_quoted: true,
        });
    }

    // Cursor immediately after a closing quote: treat the whole quoted
    // identifier as the token so cycling replaces it.
    if let Some((start, end)) = last_closed {
        if end == cursor_pos {
            return Some(CompletionToken {
                start,
                text: prefix[start..].to_string(),
                is_quoted: true,
            });
        }
    }

    // Bare identifier: scan back over identifier chars.
    let mut start = cursor_pos;
    for (idx, ch) in prefix.char_indices().rev() {
        if is_identifier_char(ch) {
            start = idx;
        } else {
            break;
        }
    }
    if start == cursor_pos {
        return None;
    }

    // A leading run of dots is punctuation, not part of the name.
    let text = &prefix[start..];
    let trimmed = text.trim_start_matches('.');
    if trimmed.is_empty() {
        return None;
    }
    let start = start + (text.len() - trimmed.len());

    Some(CompletionToken {
        start,
        text: prefix[start..].to_string(),
        is_quoted: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(query: &str) -> Option<CompletionToken> {
        find_completion_token(query, query.len())
    }

    #[test]
    fn bare_identifier() {
        let t = tok("SELECT na").unwrap();
        assert_eq!(t.start, 7);
        assert_eq!(t.text, "na");
        assert!(!t.is_quoted);
    }

    #[test]
    fn dotted_identifier_is_one_token() {
        let t = tok("SELECT name.com").unwrap();
        assert_eq!(t.start, 7);
        assert_eq!(t.text, "name.com");
        assert_eq!(t.last_segment(), Some((12, "com".to_string())));
    }

    #[test]
    fn trailing_dot_is_kept() {
        let t = tok("SELECT name.").unwrap();
        assert_eq!(t.text, "name.");
        assert_eq!(t.last_segment(), Some((12, String::new())));
    }

    #[test]
    fn open_quote_starts_the_token() {
        let t = tok("SELECT \"name.com").unwrap();
        assert_eq!(t.start, 7);
        assert_eq!(t.text, "\"name.com");
        assert!(t.is_quoted);
        assert_eq!(t.unquoted(), "name.com");
    }

    #[test]
    fn open_quote_spans_spaces() {
        let t = tok("SELECT * FROM t WHERE \"Customer Id").unwrap();
        assert_eq!(t.text, "\"Customer Id");
        assert_eq!(t.unquoted(), "Customer Id");
    }

    #[test]
    fn closed_quote_before_cursor_is_the_token() {
        let t = tok("SELECT \"name.common\"").unwrap();
        assert_eq!(t.start, 7);
        assert_eq!(t.text, "\"name.common\"");
        assert!(t.is_quoted);
        assert_eq!(t.unquoted(), "name.common");
    }

    #[test]
    fn closed_quote_further_back_is_not_the_token() {
        assert_eq!(tok("SELECT \"name.common\", "), None);
        let t = tok("SELECT \"name.common\", ca").unwrap();
        assert_eq!(t.text, "ca");
        assert!(!t.is_quoted);
    }

    #[test]
    fn no_token_after_whitespace() {
        assert_eq!(tok("SELECT "), None);
        assert_eq!(tok(""), None);
    }

    #[test]
    fn escaped_quotes_inside_identifier() {
        let t = tok("SELECT \"od\"\"d").unwrap();
        assert!(t.is_quoted);
        assert_eq!(t.unquoted(), "od\"d");
    }
}
