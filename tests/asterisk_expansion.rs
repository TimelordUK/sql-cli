//! `SELECT *` expansion (Ctrl+X / Alt+X) must emit column names the parser can
//! read back.
//!
//! `data/countries.csv` is the case that exposed this: `name.common`,
//! `idd.root` and 60-odd `translations.*.common` columns all have to be quoted.
//! Expansion used to join the raw names, so pressing Alt+X on that file
//! produced `SELECT name.common, name.official, ...` — a query the parser
//! reads as method calls on a `name` column. Tab completion already quoted
//! correctly (T1), which is exactly the drift this pins down: both go through
//! `sql::identifier::quote_if_needed` now.

use sql_cli::buffer::{Buffer, BufferAPI};
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::hybrid_parser::HybridParser;
use std::sync::Arc;

/// The awkward end of countries.csv, plus a couple of names that are fine bare.
const COLUMNS: &[&str] = &[
    "name.common",
    "name.official",
    "tld",
    "cca2",
    "idd.root",
    "unMember",
];

fn table() -> DataTable {
    let mut table = DataTable::new("countries");
    for name in COLUMNS {
        table.add_column(DataColumn::new(*name));
    }
    table
        .add_row(DataRow::new(
            COLUMNS
                .iter()
                .map(|c| DataValue::String((*c).to_string()))
                .collect(),
        ))
        .expect("row matches column count");
    table
}

fn parser() -> HybridParser {
    let mut parser = HybridParser::new();
    parser.update_single_table(
        "countries".to_string(),
        COLUMNS.iter().map(|c| (*c).to_string()).collect(),
    );
    parser
}

fn buffer_with_query(query: &str) -> Buffer {
    let mut buffer = Buffer::new(1);
    buffer.set_input_text(query.to_string());
    buffer.set_dataview(Some(DataView::new(Arc::new(table()))));
    buffer
}

const EXPANDED: &str = "\"name.common\", \"name.official\", tld, cca2, \"idd.root\", unMember";

#[test]
fn schema_expansion_quotes_dotted_names() {
    let mut buffer = buffer_with_query("SELECT * FROM countries");
    assert!(buffer.expand_asterisk(&parser()));
    assert_eq!(
        buffer.get_input_text(),
        format!("SELECT {EXPANDED} FROM countries")
    );
}

#[test]
fn visible_expansion_quotes_dotted_names() {
    let mut buffer = buffer_with_query("SELECT * FROM countries");
    assert!(buffer.expand_asterisk_visible());
    assert_eq!(
        buffer.get_input_text(),
        format!("SELECT {EXPANDED} FROM countries")
    );
}

#[test]
fn visible_expansion_follows_hidden_columns() {
    let mut buffer = buffer_with_query("SELECT * FROM countries");
    let mut view = DataView::new(Arc::new(table()));
    view.hide_column_by_name("tld");
    view.hide_column_by_name("cca2");
    buffer.set_dataview(Some(view));

    assert!(buffer.expand_asterisk_visible());
    assert_eq!(
        buffer.get_input_text(),
        "SELECT \"name.common\", \"name.official\", \"idd.root\", unMember FROM countries"
    );
}

#[test]
fn expansion_leaves_the_rest_of_the_query_alone() {
    let mut buffer = buffer_with_query("SELECT * FROM countries WHERE cca2 = 'GB' ORDER BY tld");
    assert!(buffer.expand_asterisk_visible());
    assert_eq!(
        buffer.get_input_text(),
        format!("SELECT {EXPANDED} FROM countries WHERE cca2 = 'GB' ORDER BY tld")
    );
}

#[test]
fn expansion_quotes_names_that_collide_with_keywords() {
    let mut table = DataTable::new("t");
    for name in ["order", "Row", "Price ($)", "1st", "region"] {
        table.add_column(DataColumn::new(name));
    }
    let mut buffer = Buffer::new(1);
    buffer.set_input_text("SELECT * FROM t".to_string());
    buffer.set_dataview(Some(DataView::new(Arc::new(table))));

    assert!(buffer.expand_asterisk_visible());
    assert_eq!(
        buffer.get_input_text(),
        "SELECT \"order\", \"Row\", \"Price ($)\", \"1st\", region FROM t"
    );
}
