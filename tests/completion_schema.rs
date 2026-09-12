//! The completer's schema, taken from real data (T2).
//!
//! Before T2 the parser was handed `Vec<String>` and decided every column's
//! type from a hardcoded list of trade-desk names. These tests load
//! `data/countries.csv` through the ordinary loader, take the same snapshot
//! the TUI takes, and check that the suggestions follow from the *data*.
//!
//! Nothing here needs a terminal: the parser is a pure function of
//! `(query, cursor, schema)`, which is exactly the property the snapshot
//! boundary exists to preserve.

use sql_cli::data::datatable::{DataTable, DISTINCT_VALUES_CAP};
use sql_cli::data::datatable_loaders::load_csv_to_datatable;
use sql_cli::sql::cursor_aware_parser::CursorAwareParser;
use sql_cli::sql::parser::{ColumnInfo, ColumnType, TableInfo};
use sql_cli::sql::suggestion::SuggestionKind;
use sql_cli::ui::utils::text_operations::apply_completion_to_text;

fn countries() -> DataTable {
    load_csv_to_datatable("data/countries.csv", "countries").expect("load data/countries.csv")
}

/// The same snapshot `StateCoordinator::schema_snapshot` takes.
fn snapshot(table: &DataTable) -> TableInfo {
    TableInfo::new(
        table.name.clone(),
        table
            .columns
            .iter()
            .map(ColumnInfo::from_data_column)
            .collect(),
    )
    .with_row_count(table.row_count())
}

fn parser_for(table: &DataTable) -> CursorAwareParser {
    let mut parser = CursorAwareParser::new();
    parser.update_single_table_info(snapshot(table));
    parser
}

#[test]
fn snapshot_types_columns_from_the_loaded_data() {
    let table = countries();
    let info = snapshot(&table);

    let column_type = |name: &str| {
        info.find_column(name)
            .unwrap_or_else(|| panic!("no column {name} in countries.csv"))
            .data_type
    };

    // Numbers and text, neither of which appears on any hardcoded list of
    // trade-desk column names, so before T2 both typed as string.
    assert_eq!(column_type("area"), ColumnType::Numeric);
    assert_eq!(column_type("region"), ColumnType::String);
    assert_eq!(column_type("name.common"), ColumnType::String);

    // `unMember` is a 0/1 flag and types as numeric. `independent` is the
    // same shape but has one quoted-empty cell, which the loader stores as
    // `String("")` rather than NULL, so the column merges to `Mixed` and the
    // snapshot reports string. That is upstream type inference, not the
    // completer - recorded here so the difference is visible if it changes.
    assert_eq!(column_type("unMember"), ColumnType::Numeric);
    assert_eq!(column_type("independent"), ColumnType::String);
}

#[test]
fn numeric_columns_no_longer_get_offered_string_only_methods() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE area.";
    let result = parser.get_completions(query, query.len());

    assert!(
        result.insert_texts().contains(&"ToString()".to_string()),
        "area is numeric, expected ToString(): {:?}",
        result.suggestions
    );
    assert!(
        !result.insert_texts().contains(&"Trim()".to_string()),
        "area is numeric, Trim() is meaningless on it: {:?}",
        result.suggestions
    );
}

#[test]
fn string_columns_keep_their_methods() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region.";
    let result = parser.get_completions(query, query.len());

    assert!(result.insert_texts().contains(&"Contains('')".to_string()));
    assert!(result
        .insert_texts()
        .contains(&"StartsWith('')".to_string()));
}

/// The snapshot carries what T4's low-cardinality gate will need, and pins
/// down that the numbers arriving are the real ones, so the gate can be
/// designed against them rather than against a guess.
#[test]
fn snapshot_carries_cardinality_for_the_value_completion_gate() {
    let table = countries();
    let info = snapshot(&table);
    let rows = info.row_count.expect("row count captured");
    assert!(
        rows > 100,
        "expected the full country list, got {rows} rows"
    );

    let cardinality = |name: &str| {
        info.find_column(name)
            .unwrap_or_else(|| panic!("no column {name}"))
            .cardinality
            .unwrap_or_else(|| panic!("no cardinality for {name}"))
    };

    // The two ends of the gate: `region` is worth offering as values,
    // `name.common` is one distinct value per row and never should be.
    let region = cardinality("region");
    assert!(
        (2..=12).contains(&region),
        "region should be low cardinality, got {region}"
    );
    assert!(
        cardinality("independent") <= 3,
        "independent is a 0/1 flag - a handful of distinct values in the whole file"
    );
    assert_eq!(
        cardinality("name.common"),
        rows,
        "every country name is distinct, so the gate must exclude it"
    );
}

// ---------------------------------------------------------------------------
// T3: suggestions are values, not strings.
//
// The inserted text and the shown text are now separate fields, and what the
// user has typed is matched against the shown one. These tests pin the split
// where it is load-bearing: a column that has to be quoted to be inserted,
// but is read, typed and displayed without quotes.
// ---------------------------------------------------------------------------

#[test]
fn a_column_is_inserted_quoted_and_displayed_plain() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT nam";
    let result = parser.get_completions(query, query.len());
    let dotted = result
        .suggestions
        .iter()
        .find(|s| s.label == "name.common")
        .expect("name.common should be offered for `nam`");

    assert_eq!(dotted.insert, "\"name.common\"");
    assert_eq!(dotted.display_text(), "name.common");
    assert_eq!(dotted.kind, SuggestionKind::Column);
}

/// The old filter compared against the inserted text and had to strip the
/// quotes back off to do it. Matching the label is the same rule stated once.
#[test]
fn typing_is_matched_against_the_displayed_name() {
    let table = countries();
    let parser = parser_for(&table);

    for query in ["SELECT nam", "SELECT \"nam"] {
        let result = parser.get_completions(query, query.len());
        assert!(
            result.suggestions.iter().any(|s| s.label == "name.common"),
            "{query:?} should reach name.common, got {:?}",
            result.insert_texts()
        );
    }
}

/// Kinds exist so that T4 can rank values against columns, and so the status
/// line can say what it is offering. Each context contributes its own.
#[test]
fn suggestions_know_what_kind_of_thing_they_are() {
    let table = countries();
    let parser = parser_for(&table);

    let kind_of = |query: &str, label: &str| {
        let result = parser.get_completions(query, query.len());
        result
            .suggestions
            .iter()
            .find(|s| s.label == label)
            .unwrap_or_else(|| {
                panic!(
                    "{query:?} did not offer {label:?}; got {:?}",
                    result.insert_texts()
                )
            })
            .kind
    };

    assert_eq!(kind_of("SELECT ", "region"), SuggestionKind::Column);
    assert_eq!(kind_of("SELECT ", "ROUND("), SuggestionKind::Function);
    assert_eq!(
        kind_of("SELECT * FROM ", "countries"),
        SuggestionKind::Table
    );
    assert_eq!(
        kind_of("SELECT * FROM countries ", "WHERE"),
        SuggestionKind::Keyword
    );
    assert_eq!(
        kind_of("SELECT * FROM countries WHERE region.", "Contains('')"),
        SuggestionKind::Method
    );
    // The empty literal a text comparison offers today is where T4's values
    // will go, so it is already a Value.
    assert_eq!(
        kind_of("SELECT * FROM countries WHERE region = ", "''"),
        SuggestionKind::Value
    );
}

// ---------------------------------------------------------------------------
// T11: the values themselves, captured at load.
//
// `infer_column_types` used to build every column's distinct set and keep
// only its size. It now keeps the values, with row counts, for columns under
// `DISTINCT_VALUES_CAP`, and the snapshot carries them to the completer. T4
// is what offers them; these tests pin that they arrive, and arrive right.
// ---------------------------------------------------------------------------

fn values_of(info: &TableInfo, name: &str) -> Option<Vec<(String, usize)>> {
    info.find_column(name)
        .unwrap_or_else(|| panic!("no column {name}"))
        .distinct_values
        .as_ref()
        .map(|values| values.iter().map(|v| (v.value.clone(), v.count)).collect())
}

fn owned(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
    pairs.iter().map(|(v, c)| ((*v).to_string(), *c)).collect()
}

#[test]
fn low_cardinality_values_reach_the_snapshot_with_their_counts() {
    let table = countries();
    let info = snapshot(&table);

    assert_eq!(
        values_of(&info, "region"),
        Some(owned(&[
            ("Africa", 59),
            ("Americas", 56),
            ("Antarctic", 5),
            ("Asia", 50),
            ("Europe", 53),
            ("Oceania", 27),
        ]))
    );

    // A numeric flag keeps its values unquoted-looking; T4 decides whether
    // they are inserted with quotes, from the column type.
    assert_eq!(
        values_of(&info, "unMember"),
        Some(owned(&[("0", 56), ("1", 194)]))
    );

    // The type-inference wart from T2, visible from the other side: the one
    // quoted-empty cell in `independent` is `String("")`, not NULL, so it is
    // a distinct value in its own right. Pinned so a fix upstream shows here.
    assert_eq!(
        values_of(&info, "independent"),
        Some(owned(&[("", 1), ("0", 55), ("1", 194)]))
    );
}

#[test]
fn high_cardinality_columns_keep_their_count_but_not_their_values() {
    let table = countries();
    let info = snapshot(&table);

    let name = info.find_column("name.common").expect("name.common");
    assert_eq!(name.cardinality, info.row_count);
    assert_eq!(name.distinct_values, None);
}

/// The gate, checked against every column of a real file rather than a
/// fixture: values are retained exactly when the count is within the cap,
/// and when they are, they agree with the count and with the row total.
#[test]
fn values_are_retained_exactly_for_columns_within_the_cap() {
    let table = countries();
    let info = snapshot(&table);
    let rows = info.row_count.expect("row count captured");

    let mut retained = 0;
    for column in &info.columns {
        let cardinality = column.cardinality.expect("loader counts every column");
        match &column.distinct_values {
            Some(values) => {
                retained += 1;
                assert!(cardinality <= DISTINCT_VALUES_CAP, "{}", column.name);
                assert_eq!(values.len(), cardinality, "{}", column.name);
                let counted: usize = values.iter().map(|v| v.count).sum();
                assert!(counted <= rows, "{}", column.name);
            }
            None => assert!(
                cardinality > DISTINCT_VALUES_CAP,
                "{} has {cardinality} values, within the cap, but none were kept",
                column.name
            ),
        }
    }
    assert!(
        retained >= 7,
        "expected region, subregion, the flags and idd.root at least; got {retained}"
    );
}

// ---------------------------------------------------------------------------
// T4: the values are offered between the quotes.
//
// The position T9 identified and left empty, and T11 captured the data for.
// Scope is deliberately the quoted forms only — `= '<tab>'` and
// `IN ('<tab>')` — so there is no quoting or number-formatting decision to
// make: the value goes in exactly as the data reads it.
// ---------------------------------------------------------------------------

/// What the whole workstream has been for: six regions, in a stable order,
/// each labelled with how many rows it covers.
#[test]
fn a_quoted_value_position_offers_the_columns_values() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region = '";
    let result = parser.get_completions(query, query.len());

    let offered: Vec<(String, String)> = result
        .suggestions
        .iter()
        .map(|s| (s.insert.clone(), s.display_text()))
        .collect();

    assert_eq!(
        offered,
        vec![
            ("Africa".to_string(), "Africa (59 rows)".to_string()),
            ("Americas".to_string(), "Americas (56 rows)".to_string()),
            ("Antarctic".to_string(), "Antarctic (5 rows)".to_string()),
            ("Asia".to_string(), "Asia (50 rows)".to_string()),
            ("Europe".to_string(), "Europe (53 rows)".to_string()),
            ("Oceania".to_string(), "Oceania (27 rows)".to_string()),
        ]
    );
}

/// Typing narrows, which is what makes a 100-value column usable.
#[test]
fn typing_inside_the_quotes_narrows_the_values() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region = 'Am";
    let result = parser.get_completions(query, query.len());
    assert_eq!(result.insert_texts(), vec!["Americas".to_string()]);

    // Case-insensitively, and the inserted text keeps the data's own case.
    let query = "SELECT * FROM countries WHERE region = 'af";
    let result = parser.get_completions(query, query.len());
    assert_eq!(result.insert_texts(), vec!["Africa".to_string()]);
}

/// An `IN` list is the same position, so it gets the same values. Excluding
/// the ones already in the list is T5.
#[test]
fn an_in_list_offers_values_too() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region IN ('Asia', '";
    let result = parser.get_completions(query, query.len());
    assert!(result.insert_texts().contains(&"Europe".to_string()));
}

/// The gate, from the other end: a column whose values were not retained
/// offers nothing rather than something misleading.
#[test]
fn a_high_cardinality_column_offers_no_values() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE \"name.common\" = '";
    let result = parser.get_completions(query, query.len());
    assert!(
        result.context.starts_with("InStringLiteral"),
        "got {}",
        result.context
    );
    assert!(
        result.suggestions.is_empty(),
        "name.common has 250 values and none were kept, got {:?}",
        result.insert_texts()
    );
}

/// Accepting a value replaces the text between the quotes and nothing else.
/// `replace_start` has pointed here since T9; this is the first thing to use it.
#[test]
fn accepting_a_value_splices_it_between_the_quotes() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region = 'Am";
    let result = parser.get_completions(query, query.len());
    let accepted = &result.suggestions[0];

    let spliced =
        apply_completion_to_text(query, query.len(), result.replace_start, &accepted.insert);
    assert_eq!(
        spliced.new_text,
        "SELECT * FROM countries WHERE region = 'Americas"
    );
}

/// The blank cell in `independent` is a real value with a real count, but a
/// zero-width suggestion is indistinguishable from Tab doing nothing, so it is
/// not offered. The 0 and 1 around it are.
#[test]
fn an_empty_value_is_not_offered() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE independent = '";
    let result = parser.get_completions(query, query.len());
    assert_eq!(
        result.insert_texts(),
        vec!["0".to_string(), "1".to_string()]
    );
}

// ---------------------------------------------------------------------------
// T9: cursor context from the token stream.
//
// Each case below is a row of the table in `docs/TUI_FEATURES.md`, measured
// against this file before the change. They run through the same
// `get_completions` entry point the TUI calls, so they pin the suggestions the
// user actually sees, not just the context enum.
// ---------------------------------------------------------------------------

/// The headline case. Typing the opening quote used to collapse the context to
/// `WhereClause`, which offered all 76 column names *inside the string
/// literal*; typing a letter then filtered them to nothing.
///
/// T4 has since filled this position with the column's values, so what is
/// pinned here is the part that must stay true either way: a column name is
/// never offered inside the quotes, and everything offered is a Value.
#[test]
fn a_cursor_inside_a_string_literal_is_not_a_column_position() {
    let table = countries();
    let parser = parser_for(&table);

    for query in [
        "SELECT * FROM countries WHERE region = '",
        "SELECT * FROM countries WHERE region = 'Am",
        "SELECT * FROM countries WHERE region IN ('Asia', '",
        // The literal's contents are text, not syntax.
        "SELECT * FROM countries WHERE region = 'GROUP BY ",
    ] {
        let result = parser.get_completions(query, query.len());
        assert!(
            result.context.starts_with("InStringLiteral"),
            "{query:?} should be a value position, got {}",
            result.context
        );
        assert!(
            result
                .suggestions
                .iter()
                .all(|s| s.kind == SuggestionKind::Value),
            "{query:?} should offer values only, got {:?}",
            result.suggestions
        );
    }
}

/// `IN (...)` is distinguished from `=` at the context level, which is what
/// lets T5 iterate a list without T4 having to know about lists.
#[test]
fn an_in_list_is_distinguished_from_a_plain_comparison() {
    let table = countries();
    let parser = parser_for(&table);

    let plain = "SELECT * FROM countries WHERE region = '";
    assert!(parser
        .get_completions(plain, plain.len())
        .context
        .contains("list=false"));

    let list = "SELECT * FROM countries WHERE region IN ('Asia', '";
    assert!(parser
        .get_completions(list, list.len())
        .context
        .contains("list=true"));
}

/// The span an accepted value replaces is the text *between* the quotes.
/// `find_completion_token` returns nothing useful there — a value is not an
/// identifier — so the analyzer reports it, and T4 can splice without
/// re-deriving anything.
#[test]
fn a_value_replaces_the_span_just_past_the_opening_quote() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region = 'Am";
    let result = parser.get_completions(query, query.len());
    assert_eq!(&query[result.replace_start..], "Am");
}

/// Was `WhereClause`: the old check required the column to satisfy
/// `chars().all(|c| c.is_alphanumeric() || c == '_')`, which excludes every
/// column that has to be quoted — i.e. most of this file.
#[test]
fn a_quoted_column_reaches_its_comparison_operator() {
    let table = countries();
    let parser = parser_for(&table);

    let query = r#"SELECT * FROM countries WHERE "name.common" = "#;
    let result = parser.get_completions(query, query.len());
    assert!(
        result.context.starts_with("AfterComparison(name.common"),
        "expected a value position for a quoted column, got {}",
        result.context
    );
    // `name.common` is a string column, so the offer is an empty literal.
    assert_eq!(result.insert_texts(), vec!["''".to_string()]);
}

/// The old operator list was `[" > ", " < ", " = ", …]` — the spaces were part
/// of the pattern, so an operator typed without them was invisible.
#[test]
fn an_operator_without_surrounding_spaces_is_still_an_operator() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries WHERE region=";
    let result = parser.get_completions(query, query.len());
    assert!(
        result.context.starts_with("AfterComparison(region"),
        "got {}",
        result.context
    );
}

/// The completer knew six keywords and guessed at everything else, so clauses
/// the lexer has always understood got confidently wrong answers.
#[test]
fn clauses_the_old_scanner_did_not_know_are_no_longer_guessed() {
    let table = countries();
    let parser = parser_for(&table);

    // Was `AfterTable`, which suggested `WHERE` and `ORDER BY` *after* GROUP BY.
    let group_by = "SELECT region FROM countries GROUP BY ";
    let result = parser.get_completions(group_by, group_by.len());
    assert!(result.context.starts_with("GroupByClause"));
    assert!(
        result.insert_texts().iter().any(|s| s == "region"),
        "GROUP BY should offer columns, got {:?}",
        result.suggestions
    );

    // Was `FromClause`, which suggested the table name where a row count goes.
    let limit = "SELECT * FROM countries LIMIT ";
    let result = parser.get_completions(limit, limit.len());
    assert!(result.context.starts_with("LimitClause"));
    assert!(
        result.suggestions.is_empty(),
        "LIMIT takes a number, got {:?}",
        result.suggestions
    );
}

/// A named table means the next thing is a clause, not another table.
#[test]
fn a_completed_table_name_moves_on_to_clause_keywords() {
    let table = countries();
    let parser = parser_for(&table);

    let query = "SELECT * FROM countries ";
    let result = parser.get_completions(query, query.len());
    assert!(result.insert_texts().iter().any(|s| s == "WHERE"));
    assert!(
        !result.insert_texts().iter().any(|s| s == "countries"),
        "the table is already named, got {:?}",
        result.suggestions
    );
}

/// Column completion in the ordinary positions must be untouched by all of the
/// above — this is the behaviour T1 and T2 established.
#[test]
fn ordinary_column_completion_still_works() {
    let table = countries();
    let parser = parser_for(&table);

    for (query, expected) in [
        ("SELECT * FROM countries WHERE reg", "region"),
        (
            "SELECT * FROM countries WHERE region = 'Asia' AND reg",
            "region",
        ),
        ("SELECT nam", "\"name.common\""),
    ] {
        let result = parser.get_completions(query, query.len());
        assert!(
            result.insert_texts().iter().any(|s| s == expected),
            "{query:?} should still suggest {expected}, got {:?}",
            result.suggestions
        );
    }
}
