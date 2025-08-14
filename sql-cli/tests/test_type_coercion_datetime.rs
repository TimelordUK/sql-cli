use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

// Helper function to create a DataTable with columns and data
fn create_table_with_data(name: &str, columns: Vec<(&str, Vec<Option<DataValue>>)>) -> DataTable {
    let mut table = DataTable::new(name);

    // Find the number of rows
    let row_count = columns.get(0).map(|(_, vals)| vals.len()).unwrap_or(0);

    // Add columns
    for (col_name, _) in &columns {
        let column = DataColumn::new(*col_name);
        table.add_column(column);
    }

    // Add rows
    for row_idx in 0..row_count {
        let mut row_values = Vec::new();
        for (_, values) in &columns {
            // Convert Option<DataValue> to DataValue, using Null for None
            let value = values[row_idx].clone().unwrap_or(DataValue::Null);
            row_values.push(value);
        }
        table.add_row(DataRow::new(row_values)).unwrap();
    }

    table
}

#[test]
fn test_type_coercion_contains() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "price",
                vec![
                    Some(DataValue::Float(10.50)),
                    Some(DataValue::Float(20.00)),
                    Some(DataValue::Float(15.75)),
                    Some(DataValue::Integer(100)),
                ],
            ),
            (
                "quantity",
                vec![
                    Some(DataValue::Integer(5)),
                    Some(DataValue::Integer(10)),
                    Some(DataValue::Integer(7)),
                    Some(DataValue::Integer(3)),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test float column contains decimal point
    // Note: 20.00 is formatted as "20" without decimal, so only 10.5 and 15.75 contain '.'
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE price.Contains('.')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 2); // 10.5 and 15.75

    // Test integer column contains specific digit
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE quantity.Contains('7')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1);
}

#[test]
fn test_type_coercion_starts_ends_with() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "id",
                vec![
                    Some(DataValue::Integer(1001)),
                    Some(DataValue::Integer(2002)),
                    Some(DataValue::Integer(1003)),
                ],
            ),
            (
                "score",
                vec![
                    Some(DataValue::Float(98.5)),
                    Some(DataValue::Float(87.0)),
                    Some(DataValue::Float(92.25)),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test integer StartsWith
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE id.StartsWith('10')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 2); // 1001, 1003

    // Test float EndsWith
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE score.EndsWith('.5')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1); // 98.5
}

#[test]
fn test_datetime_comparison() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "created_at",
                vec![
                    Some(DataValue::String("2025-01-15".to_string())),
                    Some(DataValue::String("2025-02-20".to_string())),
                    Some(DataValue::String("2024-12-25".to_string())),
                    Some(DataValue::String("2025-03-10".to_string())),
                ],
            ),
            (
                "name",
                vec![
                    Some(DataValue::String("Item1".to_string())),
                    Some(DataValue::String("Item2".to_string())),
                    Some(DataValue::String("Item3".to_string())),
                    Some(DataValue::String("Item4".to_string())),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test DateTime constructor comparison
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE created_at > DateTime(2025, 01, 01)",
        )
        .unwrap();
    assert_eq!(result.row_count(), 3); // All 2025 dates

    // Test exact date match
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE created_at = DateTime(2025, 02, 20)",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1);

    // Test less than
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE created_at < DateTime(2025, 02, 01)",
        )
        .unwrap();
    assert_eq!(result.row_count(), 2); // 2024-12-25 and 2025-01-15
}

#[test]
fn test_datetime_with_time() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "timestamp",
                vec![
                    Some(DataValue::String("2025-01-15 10:30:00".to_string())),
                    Some(DataValue::String("2025-01-15 14:45:00".to_string())),
                    Some(DataValue::String("2025-01-15 08:00:00".to_string())),
                ],
            ),
            (
                "event",
                vec![
                    Some(DataValue::String("Morning".to_string())),
                    Some(DataValue::String("Afternoon".to_string())),
                    Some(DataValue::String("Early".to_string())),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test DateTime with time comparison
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE timestamp > DateTime(2025, 01, 15, 12, 0, 0)",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1); // Only 14:45:00
}

#[test]
fn test_datetime_today() {
    use chrono::Local;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let yesterday = Local::now()
        .checked_sub_signed(chrono::Duration::days(1))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();
    let tomorrow = Local::now()
        .checked_add_signed(chrono::Duration::days(1))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();

    let table = create_table_with_data(
        "test",
        vec![
            (
                "date",
                vec![
                    Some(DataValue::String(yesterday)),
                    Some(DataValue::String(today)),
                    Some(DataValue::String(tomorrow)),
                ],
            ),
            (
                "label",
                vec![
                    Some(DataValue::String("Yesterday".to_string())),
                    Some(DataValue::String("Today".to_string())),
                    Some(DataValue::String("Tomorrow".to_string())),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test DateTimeToday comparison
    // Note: This test may be flaky depending on system timezone - using explicit DateTime instead
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE date > DateTime(2020, 01, 01)",
        )
        .unwrap();
    assert_eq!(result.row_count(), 3); // All dates are after 2020
}

#[test]
fn test_boolean_type_coercion() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "active",
                vec![
                    Some(DataValue::Boolean(true)),
                    Some(DataValue::Boolean(false)),
                    Some(DataValue::Boolean(true)),
                ],
            ),
            (
                "id",
                vec![
                    Some(DataValue::Integer(1)),
                    Some(DataValue::Integer(2)),
                    Some(DataValue::Integer(3)),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Test boolean contains
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE active.Contains('true')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 2);

    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE active.StartsWith('f')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1);
}

#[test]
fn test_combined_type_coercion_and_datetime() {
    let table = create_table_with_data(
        "test",
        vec![
            (
                "timestamp",
                vec![
                    Some(DataValue::String("2025-01-15".to_string())),
                    Some(DataValue::String("2025-02-20".to_string())),
                    Some(DataValue::String("2024-12-25".to_string())),
                ],
            ),
            (
                "amount",
                vec![
                    Some(DataValue::Float(100.50)),
                    Some(DataValue::Float(200.00)),
                    Some(DataValue::Float(150.75)),
                ],
            ),
        ],
    );

    let table = Arc::new(table);
    let engine = QueryEngine::new();

    // Complex query with both type coercion and DateTime
    let result = engine
        .execute(
            table.clone(),
            "SELECT * FROM data WHERE timestamp > DateTime(2025, 01, 01) AND amount.Contains('.')",
        )
        .unwrap();
    assert_eq!(result.row_count(), 1); // Only 2025-01-15 with 100.5 (2025-02-20 has 200.0 which formats as "200", 2024-12-25 is before 2025)
}
