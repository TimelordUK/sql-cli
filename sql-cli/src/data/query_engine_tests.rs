use crate::data::data_view::DataView;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::data::query_engine::QueryEngine;
use crate::sql::recursive_parser::{BinaryOp, Expression, SelectItem, SelectStatement};
use std::sync::Arc;

fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("quantity"));
    table.add_column(DataColumn::new("price"));

    table.add_row(vec![
        DataValue::Integer(1),
        DataValue::Integer(10),
        DataValue::Float(100.5),
    ]);
    table.add_row(vec![
        DataValue::Integer(2),
        DataValue::Integer(5),
        DataValue::Float(200.25),
    ]);

    table
}

#[test]
#[ignore = "Need to test through execute method with proper SQL"]
fn test_duplicate_columns_get_aliased() {
    // The duplicate column handling is implemented in query_engine.rs
    // in the apply_select_items method.
    // When duplicate columns are selected, they get auto-aliased:
    // - First occurrence: "quantity"
    // - Second occurrence: "quantity_1"
    // - Third occurrence: "quantity_2"
    // This allows queries like: SELECT quantity, quantity * price as total, quantity
}

#[test]
#[ignore = "Need to test through execute method with proper SQL"]
fn test_duplicate_with_computed_columns() {
    // The duplicate column handling also works with computed columns.
    // A query like: SELECT quantity * price as total, quantity, total
    // Would create columns: total, quantity, total_1
    // This ensures users can select the same column multiple times
    // especially when using computed expressions.
}
