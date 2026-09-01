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

use sql_cli::data::datatable::DataTable;
use sql_cli::data::datatable_loaders::load_csv_to_datatable;
use sql_cli::sql::cursor_aware_parser::CursorAwareParser;
use sql_cli::sql::parser::{ColumnInfo, ColumnType, TableInfo};

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
        result.suggestions.contains(&"ToString()".to_string()),
        "area is numeric, expected ToString(): {:?}",
        result.suggestions
    );
    assert!(
        !result.suggestions.contains(&"Trim()".to_string()),
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

    assert!(result.suggestions.contains(&"Contains('')".to_string()));
    assert!(result.suggestions.contains(&"StartsWith('')".to_string()));
}

/// The snapshot carries what T4's low-cardinality gate will need. It is not
/// used yet - this pins down that the numbers arriving are the real ones, so
/// the gate can be designed against them rather than against a guess.
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
