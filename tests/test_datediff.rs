use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a `DataView`
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_dates");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("order_date"));
    table.add_column(DataColumn::new("ship_date"));
    table.add_column(DataColumn::new("birth_date"));
    table.add_column(DataColumn::new("last_login"));

    // Add test data with known differences
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("2024-01-15".to_string()),
            DataValue::String("2024-01-16".to_string()), // +1 day
            DataValue::String("1990-01-15".to_string()), // 34 years ago
            DataValue::String("2024-01-10 10:30:00".to_string()), // 5 days before order
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("2024-01-20".to_string()),
            DataValue::String("2024-01-25".to_string()), // +5 days
            DataValue::String("1985-03-10".to_string()), // ~39 years ago
            DataValue::String("2024-01-15 14:45:00".to_string()), // 5 days before order
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("2024-02-01".to_string()),
            DataValue::String("2024-01-22".to_string()), // -10 days (early)
            DataValue::String("2000-06-15".to_string()), // ~24 years ago
            DataValue::String("2024-01-31 09:00:00".to_string()), // 1 day before order
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_datediff_days() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT id, DATEDIFF('day', order_date, ship_date) as days_diff FROM test";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 3);

    // Check day differences
    assert_eq!(get_value(&result, 0, 1), DataValue::Integer(1)); // Row 1: +1 day
    assert_eq!(get_value(&result, 1, 1), DataValue::Integer(5)); // Row 2: +5 days
    assert_eq!(get_value(&result, 2, 1), DataValue::Integer(-10)); // Row 3: -10 days
}

#[test]
fn test_datediff_years() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT id, DATEDIFF('year', birth_date, order_date) as age_years FROM test";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 3);

    // Check approximate ages
    assert_eq!(get_value(&result, 0, 1), DataValue::Integer(34)); // ~34 years
    assert_eq!(get_value(&result, 1, 1), DataValue::Integer(38)); // ~38-39 years
    assert_eq!(get_value(&result, 2, 1), DataValue::Integer(23)); // ~23-24 years
}

#[test]
fn test_datediff_with_datetime() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test hours difference with datetime values
    let query =
        r"SELECT id, DATEDIFF('hour', last_login, order_date) as hours_diff FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 1);

    // Should be approximately 5 days * 24 hours - 10.5 hours = ~109.5 hours
    let hours = match get_value(&result, 0, 1) {
        DataValue::Integer(h) => h,
        _ => panic!("Expected integer result"),
    };
    assert!(
        (108..=112).contains(&hours),
        "Hours difference should be around 110, got {hours}"
    );
}

#[test]
fn test_now_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT NOW() as current_time FROM test LIMIT 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 1);

    // Check that NOW() returns a datetime string
    match get_value(&result, 0, 0) {
        DataValue::DateTime(dt) | DataValue::String(dt) => {
            assert!(dt.contains('-'));
            assert!(dt.contains(':'));
        }
        _ => panic!("NOW() should return a datetime string"),
    }
}

#[test]
fn test_today_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r"SELECT TODAY() as current_date FROM test LIMIT 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 1);

    // Check that TODAY() returns a date string
    match get_value(&result, 0, 0) {
        DataValue::String(dt) => {
            assert!(dt.contains('-'));
            assert!(!dt.contains(':')); // Should not have time component
            assert_eq!(dt.len(), 10); // YYYY-MM-DD format
        }
        _ => panic!("TODAY() should return a date string"),
    }
}

#[test]
fn test_datediff_with_now() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test calculating days from a date to NOW()
    let query =
        r"SELECT id, DATEDIFF('day', order_date, NOW()) as days_since FROM test WHERE id = 1";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 1);

    // The result should be positive (order_date is in the past)
    match get_value(&result, 0, 1) {
        DataValue::Integer(days) => {
            assert!(days > 0, "Days since order should be positive");
            // Rough check - should be more than 200 days from Jan 2024 to Aug 2025
            assert!(days > 200, "Should be more than 200 days from Jan 2024");
        }
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_datediff_in_where_clause() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Find orders shipped more than 3 days after order date
    let query = r"SELECT id, order_date, ship_date FROM test WHERE DATEDIFF('day', order_date, ship_date) > 3";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 3);
    assert_eq!(result.row_count(), 1); // Only row 2 has 5 days difference

    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(2));
}

#[test]
fn test_datediff_negative_values() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Find early shipments (negative date differences)
    let query = r"SELECT id, DATEDIFF('day', order_date, ship_date) as diff FROM test WHERE DATEDIFF('day', order_date, ship_date) < 0";
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 1); // Only row 3 has negative difference

    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(3));
    assert_eq!(get_value(&result, 0, 1), DataValue::Integer(-10));
}
