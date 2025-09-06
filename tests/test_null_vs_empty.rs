use sql_cli::data::csv_datasource::CsvDataSource;
use sql_cli::data::datatable::DataValue;

#[test]
fn test_null_vs_empty_string() {
    // Create a test CSV file
    let csv_content = r#"id,name,value,notes
1,Alice,,empty_field
2,Bob,"",quoted_empty
3,,100,null_name
4,"",200,quoted_empty_name
5,Charlie,,"double_null""#;

    // Write to temp file
    let temp_file = "/tmp/test_null_empty.csv";
    std::fs::write(temp_file, csv_content).unwrap();

    // Load the CSV
    let datasource = CsvDataSource::load_from_file(temp_file, "test").unwrap();

    // Convert to DataTable for easier testing
    let data_table = datasource.to_datatable();

    // Get the rows
    let rows = &data_table.rows;

    // Test row 1: Alice with unquoted empty value field -> should be NULL
    let row1 = &rows[0];
    assert_eq!(
        row1.values[2],
        DataValue::Null,
        "Row 1 value should be NULL (unquoted empty)"
    );

    // Test row 2: Bob with quoted empty name -> should be empty string
    let row2 = &rows[1];
    assert_eq!(row2.values[1], DataValue::String("Bob".to_string()));
    assert_eq!(
        row2.values[2],
        DataValue::String("".to_string()),
        "Row 2 value should be empty string (quoted empty)"
    );

    // Test row 3: unquoted empty name -> should be NULL
    let row3 = &rows[2];
    assert_eq!(
        row3.values[1],
        DataValue::Null,
        "Row 3 name should be NULL (unquoted empty)"
    );

    // Test row 4: quoted empty name -> should be empty string
    let row4 = &rows[3];
    assert_eq!(
        row4.values[1],
        DataValue::String("".to_string()),
        "Row 4 name should be empty string (quoted empty)"
    );

    // Test row 5: Charlie with unquoted empty value -> should be NULL
    let row5 = &rows[4];
    assert_eq!(
        row5.values[2],
        DataValue::Null,
        "Row 5 value should be NULL (unquoted empty)"
    );

    // Clean up
    std::fs::remove_file(temp_file).ok();
}
