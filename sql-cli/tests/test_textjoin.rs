use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a DataView
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_textjoin");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("first_name"));
    table.add_column(DataColumn::new("last_name"));
    table.add_column(DataColumn::new("age"));
    table.add_column(DataColumn::new("salary"));
    table.add_column(DataColumn::new("department"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("John".to_string()),
            DataValue::String("Doe".to_string()),
            DataValue::Integer(30),
            DataValue::Float(75000.50),
            DataValue::String("Engineering".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Jane".to_string()),
            DataValue::String("Smith".to_string()),
            DataValue::Null,
            DataValue::Float(82000.00),
            DataValue::String("Sales".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Bob".to_string()),
            DataValue::String("".to_string()), // Empty last name
            DataValue::Integer(25),
            DataValue::Float(65000.00),
            DataValue::Null, // Null department
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_textjoin_basic() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r#"SELECT TEXTJOIN(' ', 0, first_name, last_name) as full_name FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    // Check full names
    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John Doe".to_string())
    );
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane Smith".to_string())
    );
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob ".to_string()) // Empty last name included
    );
}

#[test]
fn test_textjoin_ignore_empty() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r#"SELECT TEXTJOIN(' ', 1, first_name, last_name) as full_name FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    // Check full names - empty values should be ignored
    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John Doe".to_string())
    );
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane Smith".to_string())
    );
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob".to_string()) // Empty last name ignored
    );
}

#[test]
fn test_textjoin_with_delimiter() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r#"SELECT TEXTJOIN(', ', 0, first_name, last_name, department) as info FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John, Doe, Engineering".to_string())
    );
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane, Smith, Sales".to_string())
    );
    // Bob has empty last name and null department
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob, , ".to_string())
    );
}

#[test]
fn test_textjoin_with_numbers() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query = r#"SELECT TEXTJOIN(' - ', 1, first_name, id, age) as info FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John - 1 - 30".to_string())
    );
    // Jane's age is null, should be ignored when ignore_empty is true
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane - 2".to_string())
    );
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob - 3 - 25".to_string())
    );
}

#[test]
fn test_textjoin_with_expressions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let query =
        r#"SELECT TEXTJOIN(' | ', 1, first_name, ROUND(salary, 0), department) as info FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John | 75001 | Engineering".to_string())
    );
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane | 82000 | Sales".to_string())
    );
    // Bob's department is null, should be ignored
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob | 65000".to_string())
    );
}

#[test]
fn test_textjoin_many_args() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test with many arguments
    let query = r#"SELECT TEXTJOIN('-', 1, id, first_name, last_name, age, department, 'END') as long_join FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("1-John-Doe-30-Engineering-END".to_string())
    );
    // Jane's age is null
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("2-Jane-Smith-Sales-END".to_string())
    );
    // Bob has empty last name and null department
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("3-Bob-25-END".to_string())
    );
}

#[test]
fn test_textjoin_in_where_clause() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Use TEXTJOIN in WHERE clause
    let query = r#"SELECT id, first_name FROM test WHERE TEXTJOIN(' ', 1, first_name, last_name) = 'John Doe'"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 2);
    assert_eq!(result.row_count(), 1);

    assert_eq!(get_value(&result, 0, 0), DataValue::Integer(1));
    assert_eq!(
        get_value(&result, 0, 1),
        DataValue::String("John".to_string())
    );
}

#[test]
fn test_textjoin_nested_with_functions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Nested with other functions
    let query =
        r#"SELECT TEXTJOIN(' ', 1, first_name, TEXTJOIN('-', 0, 'Age', age)) as nested FROM test"#;
    let result = engine.execute(table.clone(), query).unwrap();

    assert_eq!(result.column_count(), 1);
    assert_eq!(result.row_count(), 3);

    assert_eq!(
        get_value(&result, 0, 0),
        DataValue::String("John Age-30".to_string())
    );
    // Jane's age is null, becomes "Age-"
    assert_eq!(
        get_value(&result, 1, 0),
        DataValue::String("Jane Age-".to_string())
    );
    assert_eq!(
        get_value(&result, 2, 0),
        DataValue::String("Bob Age-25".to_string())
    );
}
