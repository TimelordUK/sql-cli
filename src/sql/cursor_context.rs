//! Where the cursor is, decided from the token stream (T9).
//!
//! Completion has three questions to answer: *what kind of thing goes here*,
//! *what has the user typed so far*, and *which span does an accepted
//! suggestion replace*. T1 gave the third one a single owner
//! ([`crate::sql::completion_token`]). T2 gave the first one a schema. This
//! module is the first question's owner.
//!
//! # Why it exists
//!
//! Before T9 the answer came from two near-identical string scanners —
//! `analyze_statement` for a query that parsed and `analyze_partial` for one
//! that did not — which between them used `rfind('.')`,
//! `split_whitespace().last()`, `ends_with(" AND")` and
//! `query_upper.contains("SELECT")`. `analyze_partial` even built a token
//! stream first and then ignored it. The consequences were not subtle:
//!
//! - `WHERE region = '` collapsed to `WhereClause` and offered every **column
//!   name** from inside the string literal, because nothing looked at
//!   [`Token::StringLiteral`].
//! - `WHERE "name.common" = ` found no comparison operator at all, because the
//!   column had to satisfy `chars().all(|c| c.is_alphanumeric() || c == '_')`
//!   — which excludes every column that needs quoting.
//! - `WHERE region='x'` found none either, because the operator list was
//!   `[" > ", " < ", " = ", …]` and required the spaces.
//! - `GROUP BY ` and `LIMIT ` fell through to a fallback that suggested
//!   `WHERE`/`ORDER BY` and a table name respectively. The completer knew six
//!   keywords; [`Token::from_keyword`] knows fifty-five.
//!
//! # How it works
//!
//! The text is truncated at the cursor and tokenized once. Context is then a
//! match on the **tail** of the token stream, not a series of backward scans:
//! the last token says what kind of position this is, and the token before it
//! disambiguates a bare word. Keywords are [`Token`] variants, so a keyword the
//! lexer learns the completer learns with it.
//!
//! Spans are byte offsets throughout — see
//! [`Lexer::tokenize_all_with_byte_positions`].

use crate::sql::parser::ast::LogicalOp;
use crate::sql::parser::lexer::{Lexer, Token};

/// What the cursor is sitting in.
///
/// Produced only by [`analyze`]; consumed by
/// [`crate::sql::cursor_aware_parser::CursorAwareParser`].
#[derive(Debug, Clone, PartialEq)]
pub enum CursorContext {
    SelectClause,
    FromClause,
    /// A table name is complete; the next thing is a clause keyword.
    AfterTable,
    WhereClause,
    GroupByClause,
    HavingClause,
    OrderByClause,
    /// A row count is expected — nothing in the schema can help.
    LimitClause,
    AfterColumn(String),
    AfterLogicalOp(LogicalOp),
    /// column name, operator text
    AfterComparisonOp(String, String),
    /// The cursor is inside an unterminated string literal. `column` is what
    /// the literal is being compared against, and is `None` when it is
    /// attached to nothing recognisable — a literal is still not a place for
    /// column names either way. `in_list` distinguishes `IN ('a', '<here>')`
    /// from `= '<here>'`; `value_start` is the byte offset just past the
    /// opening quote, which is the span an accepted value replaces.
    ///
    /// T4 fills this with the column's distinct values. Until then the
    /// completer deliberately offers nothing here — which is still an
    /// improvement on offering all 100 column names inside a string.
    InStringLiteral {
        column: Option<String>,
        in_list: bool,
        value_start: usize,
    },
    /// object, method
    InMethodCall(String, String),
    InExpression,
    Unknown,
}

/// Decide what the cursor is sitting in, and what has been typed so far.
///
/// `cursor_pos` is a **byte** offset. Everything after it is ignored: context
/// is a question about what precedes the cursor, and truncating first is what
/// lets the tail-of-stream match below be so short.
#[must_use]
pub fn analyze(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    let text = safe_slice_to(query, cursor_pos);
    let tokens = Lexer::new(text).tokenize_all_with_byte_positions();

    if tokens.is_empty() {
        return (CursorContext::Unknown, None);
    }

    let last = tokens.len() - 1;
    let (start, stop, token) = (tokens[last].0, tokens[last].1, &tokens[last].2);

    // The cursor is inside an unterminated literal. Checked first, because the
    // text inside quotes may be anything at all — keywords, dots, operators —
    // and none of it means what it would mean outside.
    if let Token::StringLiteral(value) = token {
        if !literal_is_terminated(&text[start..stop]) {
            let (column, in_list) =
                value_target(&tokens, last).map_or((None, false), |(c, l)| (Some(c), l));
            return (
                CursorContext::InStringLiteral {
                    column,
                    in_list,
                    value_start: start + 1,
                },
                non_empty(value),
            );
        }
    }

    // Positions *after* a complete token. Trailing whitespace is irrelevant
    // here — `region = ` and `region =` are the same position.
    if matches!(token, Token::Dot) {
        if let Some(column) = column_chain_ending_at(&tokens, last.checked_sub(1)) {
            return (CursorContext::AfterColumn(column), None);
        }
    }
    if let Some(op) = logical_op(token) {
        return (CursorContext::AfterLogicalOp(op), None);
    }
    if let Some(op) = comparison_op(token) {
        if let Some(column) = column_chain_ending_at(&tokens, last.checked_sub(1)) {
            return (
                CursorContext::AfterComparisonOp(column, op.to_string()),
                None,
            );
        }
    }

    // A token that reaches the cursor with no whitespace between is the word
    // being typed; one that stops short of it is finished. Only here does the
    // distinction matter, and what the word is a partial *of* is decided by
    // the token before it.
    if stop == text.len() {
        let partial = match token {
            // An unterminated quoted identifier (`WHERE "na`) is a column being
            // typed, not a value. The clause decides what to offer; which span
            // it replaces is `find_completion_token`'s business, not ours.
            Token::Identifier(word)
            | Token::NumberLiteral(word)
            | Token::QuotedIdentifier(word) => non_empty(word),
            _ => None,
        };

        if partial.is_some() && !matches!(token, Token::QuotedIdentifier(_)) {
            if let Some(prev) = last.checked_sub(1) {
                let before = &tokens[prev].2;
                if matches!(before, Token::Dot) {
                    if let Some(column) = column_chain_ending_at(&tokens, prev.checked_sub(1)) {
                        return (CursorContext::AfterColumn(column), partial);
                    }
                }
                if let Some(op) = logical_op(before) {
                    return (CursorContext::AfterLogicalOp(op), partial);
                }
                if let Some(op) = comparison_op(before) {
                    if let Some(column) = column_chain_ending_at(&tokens, prev.checked_sub(1)) {
                        return (
                            CursorContext::AfterComparisonOp(column, op.to_string()),
                            partial,
                        );
                    }
                }
            }
        }

        return (clause_context(&tokens), partial);
    }

    (clause_context(&tokens), None)
}

fn logical_op(token: &Token) -> Option<LogicalOp> {
    match token {
        Token::And => Some(LogicalOp::And),
        Token::Or => Some(LogicalOp::Or),
        _ => None,
    }
}

/// The clause the cursor is in, from the last clause-introducing keyword.
///
/// Truncation at the cursor is what makes "last" the right one to take.
fn clause_context(tokens: &[(usize, usize, Token)]) -> CursorContext {
    let Some(idx) = tokens.iter().rposition(|(_, _, t)| is_clause_keyword(t)) else {
        return CursorContext::Unknown;
    };

    match &tokens[idx].2 {
        Token::Select => CursorContext::SelectClause,
        Token::From => {
            // `FROM ` wants a table; `FROM countries ` wants a clause keyword.
            if tokens[idx + 1..]
                .iter()
                .any(|(_, _, t)| matches!(t, Token::Identifier(_) | Token::QuotedIdentifier(_)))
            {
                CursorContext::AfterTable
            } else {
                CursorContext::FromClause
            }
        }
        // ON and QUALIFY are predicate positions: the same columns apply.
        Token::Where | Token::On | Token::Qualify => CursorContext::WhereClause,
        Token::GroupBy => CursorContext::GroupByClause,
        Token::Having => CursorContext::HavingClause,
        Token::OrderBy => CursorContext::OrderByClause,
        Token::Limit | Token::Offset => CursorContext::LimitClause,
        _ => CursorContext::Unknown,
    }
}

fn is_clause_keyword(token: &Token) -> bool {
    matches!(
        token,
        Token::Select
            | Token::From
            | Token::Where
            | Token::GroupBy
            | Token::Having
            | Token::OrderBy
            | Token::Qualify
            | Token::Limit
            | Token::Offset
            | Token::On
    )
}

/// Which column an unterminated literal is being compared against, and whether
/// it sits in an `IN (...)` list.
///
/// Walks back over any values already in the list, so
/// `region IN ('Asia', 'Eu` resolves to `region` just as `region = 'Eu` does.
fn value_target(tokens: &[(usize, usize, Token)], literal: usize) -> Option<(String, bool)> {
    let mut idx = literal.checked_sub(1)?;
    let mut in_list = false;

    loop {
        match &tokens[idx].2 {
            // Earlier entries in the same list.
            Token::Comma | Token::StringLiteral(_) | Token::NumberLiteral(_) => {}
            Token::LeftParen => {
                in_list = true;
                idx = idx.checked_sub(1)?;
                // `IN (` and `NOT IN (` — step past the keywords to the column.
                if matches!(tokens[idx].2, Token::In) {
                    idx = idx.checked_sub(1)?;
                    if matches!(tokens[idx].2, Token::Not) {
                        idx = idx.checked_sub(1)?;
                    }
                }
                break;
            }
            token if comparison_op(token).is_some() => {
                idx = idx.checked_sub(1)?;
                break;
            }
            _ => return None,
        }
        idx = idx.checked_sub(1)?;
    }

    column_chain_ending_at(tokens, Some(idx)).map(|column| (column, in_list))
}

/// The column name ending at `idx`, following `Dot` links backwards.
///
/// `translations.deu.common` is one column in this codebase, so the chain
/// matters; taking only the last segment is how dotted columns became
/// unreachable in the first place (T1). Quotes are stripped, because the
/// schema is keyed on the bare name.
fn column_chain_ending_at(tokens: &[(usize, usize, Token)], idx: Option<usize>) -> Option<String> {
    let mut idx = idx?;
    let mut segments = vec![identifier_text(&tokens[idx].2)?.to_string()];

    while idx >= 2 && matches!(tokens[idx - 1].2, Token::Dot) {
        let Some(text) = identifier_text(&tokens[idx - 2].2) else {
            break;
        };
        segments.push(text.to_string());
        idx -= 2;
    }

    segments.reverse();
    Some(segments.join("."))
}

fn identifier_text(token: &Token) -> Option<&str> {
    match token {
        Token::Identifier(s) | Token::QuotedIdentifier(s) => Some(s),
        _ => None,
    }
}

/// The operator's text, or `None` if the token is not a comparison.
///
/// Replaces a `[" > ", " < ", " = ", …]` scan that required the surrounding
/// spaces, so `region='x'` never matched.
fn comparison_op(token: &Token) -> Option<&'static str> {
    match token {
        Token::Equal => Some("="),
        Token::NotEqual => Some("!="),
        Token::LessThan => Some("<"),
        Token::GreaterThan => Some(">"),
        Token::LessThanOrEqual => Some("<="),
        Token::GreaterThanOrEqual => Some(">="),
        Token::Like => Some("LIKE"),
        Token::ILike => Some("ILIKE"),
        _ => None,
    }
}

/// Whether a literal's source span carries its closing quote.
///
/// The lexer runs an unterminated string to end-of-input and returns its
/// contents with no complaint, which is exactly what completion wants — but it
/// means the token alone cannot say whether the user has closed the quote.
/// Mirrors `read_string`, doubled quotes included, so `'O''` reads as still
/// open.
fn literal_is_terminated(span: &str) -> bool {
    let mut chars = span.chars();
    let Some(quote) = chars.next() else {
        return false;
    };
    let mut chars = chars.peekable();
    while let Some(ch) = chars.next() {
        if ch == quote {
            if chars.peek() == Some(&quote) {
                chars.next();
            } else {
                return true;
            }
        }
    }
    false
}

fn non_empty(word: &str) -> Option<String> {
    (!word.is_empty()).then(|| word.to_string())
}

/// Truncate at `pos`, backing up to a character boundary rather than panicking.
fn safe_slice_to(s: &str, pos: usize) -> &str {
    if pos >= s.len() {
        return s;
    }
    let mut safe = pos;
    while safe > 0 && !s.is_char_boundary(safe) {
        safe -= 1;
    }
    &s[..safe]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(query: &str) -> CursorContext {
        analyze(query, query.len()).0
    }

    fn partial(query: &str) -> Option<String> {
        analyze(query, query.len()).1
    }

    // ---- the cases T9 exists to fix ----

    #[test]
    fn cursor_inside_a_literal_is_a_value_position() {
        assert_eq!(
            ctx("SELECT * FROM t WHERE region = '"),
            CursorContext::InStringLiteral {
                column: Some("region".to_string()),
                in_list: false,
                value_start: 32,
            }
        );
    }

    #[test]
    fn a_partial_value_is_reported_as_the_partial() {
        let (context, partial) = analyze("SELECT * FROM t WHERE region = 'Am", 34);
        assert!(matches!(context, CursorContext::InStringLiteral { .. }));
        assert_eq!(partial.as_deref(), Some("Am"));
    }

    #[test]
    fn in_list_is_distinguished_and_walks_past_existing_values() {
        assert_eq!(
            ctx("SELECT * FROM t WHERE region IN ('Asia', 'Eu"),
            CursorContext::InStringLiteral {
                column: Some("region".to_string()),
                in_list: true,
                value_start: 42,
            }
        );
    }

    #[test]
    fn not_in_still_resolves_to_the_column() {
        // The negation sits between the column and the keyword, so the walk
        // back has to step over it or the list loses its column.
        assert_eq!(
            ctx("SELECT * FROM t WHERE region NOT IN ('"),
            CursorContext::InStringLiteral {
                column: Some("region".to_string()),
                in_list: true,
                value_start: 38,
            }
        );
    }

    #[test]
    fn a_closed_literal_is_not_a_value_position() {
        // The quote is closed, so the cursor is after a complete value.
        assert_eq!(ctx("SELECT * FROM t WHERE region = 'Asia'"), {
            CursorContext::WhereClause
        });
    }

    #[test]
    fn a_doubled_quote_leaves_the_literal_open() {
        // `'O''` is an unterminated literal containing one quote, not a
        // finished one — the same rule `read_string` applies.
        assert!(matches!(
            ctx("SELECT * FROM t WHERE name = 'O''"),
            CursorContext::InStringLiteral { .. }
        ));
    }

    #[test]
    fn keywords_inside_a_literal_do_not_move_the_cursor() {
        assert!(matches!(
            ctx("SELECT * FROM t WHERE region = 'GROUP BY FROM "),
            CursorContext::InStringLiteral { .. }
        ));
    }

    #[test]
    fn a_quoted_column_reaches_the_comparison_operator() {
        // Was `WhereClause`: the old check required
        // `chars().all(alphanumeric || '_')`, which no quoted column passes.
        assert_eq!(
            ctx(r#"SELECT * FROM t WHERE "name.common" = "#),
            CursorContext::AfterComparisonOp("name.common".to_string(), "=".to_string())
        );
    }

    #[test]
    fn an_operator_without_spaces_is_still_an_operator() {
        // Was missed entirely: the old list was `[" = ", " > ", …]`.
        assert_eq!(
            ctx("SELECT * FROM t WHERE region="),
            CursorContext::AfterComparisonOp("region".to_string(), "=".to_string())
        );
    }

    #[test]
    fn clause_keywords_the_old_scanner_did_not_know() {
        assert_eq!(
            ctx("SELECT * FROM t GROUP BY "),
            CursorContext::GroupByClause
        );
        assert_eq!(
            ctx("SELECT * FROM t GROUP BY a HAVING "),
            CursorContext::HavingClause
        );
        // Was `FromClause`, which suggested a table name where a row count goes.
        assert_eq!(ctx("SELECT * FROM t LIMIT "), CursorContext::LimitClause);
        assert_eq!(ctx("SELECT * FROM t QUALIFY "), CursorContext::WhereClause);
    }

    #[test]
    fn a_named_table_moves_past_the_from_clause() {
        assert_eq!(ctx("SELECT * FROM "), CursorContext::FromClause);
        assert_eq!(ctx("SELECT * FROM countries "), CursorContext::AfterTable);
    }

    // ---- behaviour that must not regress ----

    #[test]
    fn clauses_still_resolve() {
        assert_eq!(ctx("SELECT "), CursorContext::SelectClause);
        assert_eq!(ctx("SELECT * FROM t WHERE "), CursorContext::WhereClause);
        assert_eq!(
            ctx("SELECT * FROM t ORDER BY "),
            CursorContext::OrderByClause
        );
    }

    #[test]
    fn logical_operators_still_resolve() {
        assert_eq!(
            ctx("SELECT * FROM t WHERE a = 1 AND "),
            CursorContext::AfterLogicalOp(LogicalOp::And)
        );
        assert_eq!(
            ctx("SELECT * FROM t WHERE a = 1 OR reg"),
            CursorContext::AfterLogicalOp(LogicalOp::Or)
        );
        assert_eq!(
            partial("SELECT * FROM t WHERE a = 1 OR reg").as_deref(),
            Some("reg")
        );
    }

    #[test]
    fn a_dot_is_a_method_position_on_the_whole_dotted_chain() {
        assert_eq!(
            ctx("SELECT * FROM t WHERE name."),
            CursorContext::AfterColumn("name".to_string())
        );
        assert_eq!(
            ctx("SELECT translations.deu."),
            CursorContext::AfterColumn("translations.deu".to_string())
        );
        assert_eq!(
            ctx("SELECT * FROM t WHERE name.Con"),
            CursorContext::AfterColumn("name".to_string())
        );
        assert_eq!(
            partial("SELECT * FROM t WHERE name.Con").as_deref(),
            Some("Con")
        );
    }

    #[test]
    fn an_open_method_call_is_not_a_method_position() {
        // `price.Contains(` — the method is already chosen.
        assert!(!matches!(
            ctx("SELECT * FROM t WHERE price.Contains("),
            CursorContext::AfterColumn(_)
        ));
    }

    #[test]
    fn context_ignores_everything_after_the_cursor() {
        // Cursor in the SELECT list of a query that goes on to have a WHERE.
        let (context, _) = analyze("SELECT  FROM t WHERE region = 'Asia'", 7);
        assert_eq!(context, CursorContext::SelectClause);
    }

    #[test]
    fn trailing_whitespace_means_no_partial() {
        assert_eq!(partial("SELECT * FROM t WHERE region "), None);
        assert_eq!(partial("SELECT * FROM t WHERE reg").as_deref(), Some("reg"));
    }

    // ---- the char/byte trap ----

    #[test]
    fn spans_are_byte_offsets_not_char_indices() {
        // `ö` is two bytes. A char index would land short of the real
        // offset and the spliced value would come out mangled.
        let query = "SELECT * FROM t WHERE city = 'Malmö' AND name = '";
        let CursorContext::InStringLiteral { value_start, .. } = ctx(query) else {
            panic!("expected a value position");
        };
        assert_eq!(value_start, query.len());
        assert!(query.is_char_boundary(value_start));
    }

    #[test]
    fn a_cursor_inside_a_multibyte_character_does_not_panic() {
        let query = "SELECT * FROM t WHERE city = 'Malmö'";
        // Byte 35 is the second byte of `ö`.
        assert!(!query.is_char_boundary(35));
        let _ = analyze(query, 35);
    }

    #[test]
    fn empty_input_is_unknown() {
        assert_eq!(ctx(""), CursorContext::Unknown);
        assert_eq!(ctx("   "), CursorContext::Unknown);
    }
}
