//! Tab completion for column names that need quoting.
//!
//! `data/countries.csv` has columns like `name.common` and `idd.root`. Those
//! must be quoted in SQL, and the dot in them collides with method-call syntax
//! (`price.Contains('x')`), so they exercise every corner of the completion
//! path: recognising the column, filtering on a partial, and splicing the
//! suggestion in without mangling the quotes.

use sql_cli::sql::cursor_aware_parser::CursorAwareParser;
use sql_cli::ui::utils::text_operations::apply_completion_to_text;

fn parser() -> CursorAwareParser {
    let mut parser = CursorAwareParser::new();
    parser.update_single_table(
        "countries".to_string(),
        vec![
            "name.common".to_string(),
            "name.official".to_string(),
            "tld".to_string(),
            "cca2".to_string(),
            "idd.root".to_string(),
            "capital".to_string(),
            "region".to_string(),
        ],
    );
    parser
}

/// Suggestions for the cursor at the end of `query`.
fn suggest(query: &str) -> Vec<String> {
    parser().get_completions(query, query.len()).suggestions
}

/// Type `query`, press Tab, accept `suggestion`; returns the resulting text.
fn complete(query: &str, suggestion: &str) -> String {
    let result = parser().get_completions(query, query.len());
    assert!(
        result.suggestions.iter().any(|s| s == suggestion),
        "{suggestion:?} was not offered for {query:?}; got {:?}",
        result.suggestions
    );
    apply_completion_to_text(query, query.len(), result.replace_start, suggestion).new_text
}

// ---------------------------------------------------------------------------
// Recognising dotted column names
// ---------------------------------------------------------------------------

#[test]
fn bare_prefix_offers_the_quoted_columns() {
    assert_eq!(
        suggest("SELECT na"),
        vec!["\"name.common\"", "\"name.official\""]
    );
}

#[test]
fn trailing_dot_offers_columns_not_string_methods() {
    // `name` is not a column, so `name.` can only be reaching for `name.common`
    // or `name.official` - offering Contains()/StartsWith() here is a dead end.
    assert_eq!(
        suggest("SELECT name."),
        vec!["\"name.common\"", "\"name.official\""]
    );
}

#[test]
fn partial_after_the_dot_narrows_the_columns() {
    assert_eq!(suggest("SELECT name.com"), vec!["\"name.common\""]);
    assert_eq!(suggest("SELECT name.off"), vec!["\"name.official\""]);
}

#[test]
fn dotted_columns_are_offered_in_where_and_order_by() {
    assert_eq!(
        suggest("SELECT * FROM countries WHERE idd."),
        vec!["\"idd.root\""]
    );
    assert_eq!(
        suggest("SELECT * FROM countries ORDER BY name.off"),
        vec!["\"name.official\""]
    );
}

#[test]
fn method_calls_on_real_columns_still_work() {
    // `capital` is a real column and nothing is named `capital.*`, so the dot
    // means a method call.
    let suggestions = suggest("SELECT * FROM countries WHERE capital.");
    assert!(
        suggestions.contains(&"Contains('')".to_string()),
        "expected string methods, got {suggestions:?}"
    );

    let suggestions = suggest("SELECT * FROM countries WHERE \"name.common\".Con");
    assert_eq!(suggestions, vec!["Contains('')"]);
}

// ---------------------------------------------------------------------------
// Splicing the suggestion in
// ---------------------------------------------------------------------------

#[test]
fn accepting_a_quoted_column_replaces_the_whole_partial() {
    assert_eq!(
        complete("SELECT na", "\"name.common\""),
        "SELECT \"name.common\""
    );
    // The dot is part of the identifier, not a separator to complete after.
    assert_eq!(
        complete("SELECT name.com", "\"name.common\""),
        "SELECT \"name.common\""
    );
    assert_eq!(
        complete("SELECT name.", "\"name.official\""),
        "SELECT \"name.official\""
    );
}

#[test]
fn a_quote_the_user_typed_is_not_duplicated() {
    // Typing the opening quote yourself is the natural way to reach a column
    // that needs quoting; it used to produce `SELECT name.common"`.
    assert_eq!(
        complete("SELECT \"na", "\"name.common\""),
        "SELECT \"name.common\""
    );
    assert_eq!(
        complete("SELECT \"name.com", "\"name.common\""),
        "SELECT \"name.common\""
    );
}

#[test]
fn cycling_replaces_the_previous_suggestion() {
    // Second Tab: the cursor sits after the closing quote of the suggestion
    // just inserted, which must be replaced rather than appended to.
    let first = complete("SELECT na", "\"name.common\"");
    assert_eq!(first, "SELECT \"name.common\"");

    let result = parser().get_completions(&first, first.len());
    let second = apply_completion_to_text(
        &first,
        first.len(),
        result.replace_start,
        "\"name.official\"",
    )
    .new_text;
    assert_eq!(second, "SELECT \"name.official\"");
}

#[test]
fn accepting_a_method_keeps_the_column_it_hangs_off() {
    assert_eq!(
        complete("SELECT * FROM countries WHERE capital.Con", "Contains('')"),
        "SELECT * FROM countries WHERE capital.Contains('')"
    );
    assert_eq!(
        complete(
            "SELECT * FROM countries WHERE \"name.common\".",
            "Contains('')"
        ),
        "SELECT * FROM countries WHERE \"name.common\".Contains('')"
    );
    // The closing quote ends the identifier, so the partial method after the
    // dot is the whole token rather than its last segment.
    assert_eq!(
        complete(
            "SELECT * FROM countries WHERE \"name.common\".Star",
            "StartsWith('')"
        ),
        "SELECT * FROM countries WHERE \"name.common\".StartsWith('')"
    );
}

#[test]
fn completing_after_a_comma_does_not_disturb_earlier_columns() {
    assert_eq!(
        complete("SELECT \"name.common\", ca", "capital"),
        "SELECT \"name.common\", capital"
    );
}

#[test]
fn cursor_lands_inside_the_quotes_of_a_method_argument() {
    let result = parser().get_completions(
        "SELECT * FROM countries WHERE capital.Con",
        "SELECT * FROM countries WHERE capital.Con".len(),
    );
    let query = "SELECT * FROM countries WHERE capital.Con";
    let applied =
        apply_completion_to_text(query, query.len(), result.replace_start, "Contains('')");
    assert_eq!(
        &applied.new_text[applied.new_cursor_position..],
        "')",
        "cursor should sit between the argument quotes"
    );
}
