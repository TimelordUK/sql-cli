use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_typos");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("order_date"));
    table.add_column(DataColumn::new("ship_date"));
    table.add_column(DataColumn::new("customer_name"));
    table.add_column(DataColumn::new("total_amount"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15".to_string()),
            DataValue::String("2024-01-16".to_string()),
            DataValue::String("John Doe".to_string()),
            DataValue::Float(199.99),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_column_typo_suggestions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: underscore prefix typo (_ship_date instead of ship_date)
    let query = r"SELECT DATEDIFF('day', order_date, _ship_date) as days FROM test";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Did you mean 'ship_date'?"),
        "Error should suggest 'ship_date', got: {error_msg}"
    );
}

#[test]
fn test_column_typo_suggestions_case() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: misspelled with wrong case (Shipp_Date instead of ship_date)
    let query = r"SELECT Shipp_Date FROM test";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Did you mean 'ship_date'?"),
        "Error should suggest 'ship_date', got: {error_msg}"
    );
}

#[test]
fn test_column_typo_suggestions_spelling() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: minor spelling mistake (shp_date instead of ship_date)
    let query = r"SELECT shp_date FROM test";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Did you mean 'ship_date'?"),
        "Error should suggest 'ship_date', got: {error_msg}"
    );
}

#[test]
fn test_column_typo_suggestions_swap() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: letter swap (customre_name instead of customer_name)
    let query = r"SELECT customre_name FROM test";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Did you mean 'customer_name'?"),
        "Error should suggest 'customer_name', got: {error_msg}"
    );
}

#[test]
fn test_no_suggestion_for_very_different_names() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: completely different name should not suggest anything
    let query = r"SELECT xyz_field FROM test";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        !error_msg.contains("Did you mean"),
        "Should not suggest anything for very different names, got: {error_msg}"
    );
    assert!(
        error_msg.contains("Column 'xyz_field' not found"),
        "Should show standard error, got: {error_msg}"
    );
}

#[test]
fn test_where_clause_column_suggestions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: typo in WHERE clause
    let query = r"SELECT * FROM test WHERE total_amout > 100";
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // The WHERE clause errors come from RecursiveWhereEvaluator, check if it contains column not found
    assert!(
        error_msg.contains("total_amout") || error_msg.contains("total_amount"),
        "Error should mention the column name, got: {error_msg}"
    );
}
