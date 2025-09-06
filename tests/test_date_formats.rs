use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a `DataView`
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_formats");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("iso_date"));
    table.add_column(DataColumn::new("us_date"));
    table.add_column(DataColumn::new("eu_date"));
    table.add_column(DataColumn::new("excel_date"));
    table.add_column(DataColumn::new("long_date"));

    // Add test data with different date formats, all representing the same date
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15".to_string()), // ISO format
            DataValue::String("01/15/2024".to_string()), // US format
            DataValue::String("15/01/2024".to_string()), // European format
            DataValue::String("15-Jan-2024".to_string()), // Excel format
            DataValue::String("January 15, 2024".to_string()), // Long format
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("2024-03-10".to_string()),
            DataValue::String("03/10/2024".to_string()),
            DataValue::String("10/03/2024".to_string()),
            DataValue::String("10-Mar-2024".to_string()),
            DataValue::String("March 10, 2024".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("2024-12-25".to_string()),
            DataValue::String("12/25/2024".to_string()),
            DataValue::String("25/12/2024".to_string()),
            DataValue::String("25-Dec-2024".to_string()),
            DataValue::String("December 25, 2024".to_string()),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_iso_format() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT DATEDIFF('day', '2024-01-01', iso_date) as days FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(14)); // Jan 15 - Jan 1 = 14 days
}

#[test]
fn test_us_format() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT DATEDIFF('day', '01/01/2024', us_date) as days FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(14)); // Jan 15 - Jan 1 = 14 days
}

#[test]
fn test_european_format() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT DATEDIFF('day', '01/01/2024', eu_date) as days FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(14)); // Jan 15 - Jan 1 = 14 days
}

#[test]
fn test_excel_format() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT DATEDIFF('day', '01-Jan-2024', excel_date) as days FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(14)); // Jan 15 - Jan 1 = 14 days
}

#[test]
fn test_long_format() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query =
        r"SELECT DATEDIFF('day', 'January 1, 2024', long_date) as days FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(14)); // Jan 15 - Jan 1 = 14 days
}

#[test]
fn test_mixed_formats() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Mix different formats in one query
    let query = r"SELECT DATEDIFF('day', us_date, excel_date) as days FROM test WHERE id = 3";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(0)); // Same date, different formats
}

#[test]
fn test_cross_format_comparison() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Compare ISO with US format
    let query = r"SELECT id, DATEDIFF('day', iso_date, us_date) as diff FROM test";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 3);
    // All should be 0 since they represent the same dates
    assert_eq!(get_value(&result, 0, 1), DataValue::Integer(0));
    assert_eq!(get_value(&result, 1, 1), DataValue::Integer(0));
    assert_eq!(get_value(&result, 2, 1), DataValue::Integer(0));
}

#[test]
fn test_datetime_with_different_formats() {
    let mut table = DataTable::new("test_datetime");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("datetime1"));
    table.add_column(DataColumn::new("datetime2"));

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15 10:30:00".to_string()), // ISO datetime
            DataValue::String("01/15/2024 14:30:00".to_string()), // US datetime
        ]))
        .unwrap();

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    let query = r"SELECT DATEDIFF('hour', datetime1, datetime2) as hours_diff FROM test";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(4)); // 4 hours difference
}
