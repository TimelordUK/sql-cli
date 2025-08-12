use sql_cli::data::data_provider::DataProvider;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};

#[test]
fn test_datatable_as_provider() {
    // Create a DataTable
    let mut table = DataTable::new("test_table");

    // Add columns
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("name").with_type(DataType::String));
    table.add_column(DataColumn::new("active").with_type(DataType::Boolean));

    // Add some data rows
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Alice".to_string()),
            DataValue::Boolean(true),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Bob".to_string()),
            DataValue::Boolean(false),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Charlie".to_string()),
            DataValue::Boolean(true),
        ]))
        .unwrap();

    // Use DataTable through DataProvider trait
    let provider: &dyn DataProvider = &table;

    // Test DataProvider methods
    assert_eq!(provider.get_row_count(), 3);
    assert_eq!(provider.get_column_count(), 3);

    let columns = provider.get_column_names();
    assert_eq!(columns, vec!["id", "name", "active"]);

    // Get first row
    let row = provider.get_row(0).unwrap();
    assert_eq!(row, vec!["1", "Alice", "true"]);

    // Get last row
    let row = provider.get_row(2).unwrap();
    assert_eq!(row, vec!["3", "Charlie", "true"]);

    // Test out of bounds
    assert!(provider.get_row(3).is_none());

    println!("DataTable successfully implements DataProvider!");
}

#[test]
fn test_datatable_with_mixed_types() {
    let mut table = DataTable::new("mixed_data");

    // Add columns
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("value").with_type(DataType::Float));
    table.add_column(DataColumn::new("description").with_type(DataType::String));

    // Add rows with different data types
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Float(99.99),
            DataValue::String("First item".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::Float(123.45),
            DataValue::String("Second item".to_string()),
        ]))
        .unwrap();

    // Access as DataProvider
    let provider: &dyn DataProvider = &table;

    let row = provider.get_row(0).unwrap();
    assert_eq!(row, vec!["1", "99.99", "First item"]);

    let row = provider.get_row(1).unwrap();
    assert_eq!(row, vec!["2", "123.45", "Second item"]);
}
