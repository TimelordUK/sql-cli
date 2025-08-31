use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a DataView
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_dateadd");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("base_date"));
    table.add_column(DataColumn::new("event_name"));

    // Add test data with various base dates
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15".to_string()),
            DataValue::String("Start Date".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("2024-02-29".to_string()), // Leap year date
            DataValue::String("Leap Day".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("2024-01-31".to_string()), // Month boundary
            DataValue::String("End of January".to_string()),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_dateadd_days() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 10 days to 2024-01-15, should be 2024-01-25
    let query = r#"SELECT DATEADD('day', 10, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    assert!(matches!(new_date, DataValue::DateTime(_)));
    if let DataValue::DateTime(dt) = new_date {
        assert!(dt.starts_with("2024-01-25"));
    }
}

#[test]
fn test_dateadd_negative_days() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Subtract 5 days from 2024-01-15, should be 2024-01-10
    let query = r#"SELECT DATEADD('day', -5, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        assert!(dt.starts_with("2024-01-10"));
    }
}

#[test]
fn test_dateadd_months() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 1 month to 2024-01-31, should be 2024-02-29 (leap year)
    let query = r#"SELECT DATEADD('month', 1, base_date) as new_date FROM test WHERE id = 3"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        // Should handle month-end overflow gracefully
        assert!(dt.starts_with("2024-02-"));
    }
}

#[test]
fn test_dateadd_years() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 1 year to 2024-02-29 (leap day), should be 2025-02-28
    let query = r#"SELECT DATEADD('year', 1, base_date) as new_date FROM test WHERE id = 2"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        // Should handle leap year to non-leap year
        assert!(dt.starts_with("2025-02-28"));
    }
}

#[test]
fn test_dateadd_hours() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 12 hours to a date
    let query = r#"SELECT DATEADD('hour', 12, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        assert!(dt.contains("12:00:00"));
    }
}

#[test]
fn test_dateadd_minutes() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 90 minutes (1.5 hours) to a date
    let query = r#"SELECT DATEADD('minute', 90, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        assert!(dt.contains("01:30:00"));
    }
}

#[test]
fn test_dateadd_seconds() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Add 3661 seconds (1 hour, 1 minute, 1 second)
    let query = r#"SELECT DATEADD('second', 3661, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_date = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_date {
        assert!(dt.contains("01:01:01"));
    }
}

#[test]
fn test_dateadd_with_datetime_input() {
    let mut table = DataTable::new("test_datetime");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("start_time"));

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15 14:30:00".to_string()),
        ]))
        .unwrap();

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Add 2 hours to datetime
    let query = r#"SELECT DATEADD('hour', 2, start_time) as new_time FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);
    let new_time = get_value(&result, 0, 0);
    if let DataValue::DateTime(dt) = new_time {
        assert!(dt.contains("16:30:00"));
    }
}

#[test]
fn test_dateadd_invalid_unit() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Use invalid unit
    let query = r#"SELECT DATEADD('week', 1, base_date) as new_date FROM test WHERE id = 1"#;
    let result = engine.execute(table.clone(), query);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unknown DATEADD unit"));
}

#[test]
fn test_dateadd_different_date_formats() {
    let mut table = DataTable::new("test_formats");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("us_date"));
    table.add_column(DataColumn::new("eu_date"));
    table.add_column(DataColumn::new("excel_date"));

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("01/15/2024".to_string()), // US format
            DataValue::String("15/01/2024".to_string()), // EU format
            DataValue::String("15-Jan-2024".to_string()), // Excel format
        ]))
        .unwrap();

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Add 1 day to each format
    let query = r#"
        SELECT 
            DATEADD('day', 1, us_date) as us_plus_1,
            DATEADD('day', 1, eu_date) as eu_plus_1,
            DATEADD('day', 1, excel_date) as excel_plus_1
        FROM test WHERE id = 1
    "#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.row_count(), 1);

    // All should result in January 16, 2024
    for col in 0..3 {
        let new_date = get_value(&result, 0, col);
        if let DataValue::DateTime(dt) = new_date {
            assert!(dt.starts_with("2024-01-16"));
        }
    }
}
