use serde_json::json;
use sql_cli::data::csv_datasource::CsvApiClient;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_json_preserves_numeric_types() {
    // Create test data with various numeric values that would sort incorrectly as strings
    let test_data = json!([
        {"id": 1, "quantity": 1000, "name": "First"},
        {"id": 2, "quantity": 500, "name": "Second"},
        {"id": 3, "quantity": 750, "name": "Third"},
        {"id": 4, "quantity": 2000, "name": "Fourth"}
    ]);

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", test_data.to_string()).unwrap();

    let mut client = CsvApiClient::new();
    client.load_json(temp_file.path(), "test").unwrap();

    let result = client.query_csv("SELECT * FROM test").unwrap();

    // Verify data types are preserved in JSON
    for row in &result.data {
        if let Some(obj) = row.as_object() {
            if let Some(quantity) = obj.get("quantity") {
                // Should be a Number, not a String
                assert!(
                    quantity.is_number(),
                    "Quantity should be a number: {:?}",
                    quantity
                );
            }
        }
    }

    // Original order should be: 1000, 500, 750, 2000
    let quantities: Vec<i64> = result
        .data
        .iter()
        .filter_map(|row| row.get("quantity")?.as_i64())
        .collect();

    assert_eq!(quantities, vec![1000, 500, 750, 2000]);
    println!("✓ Original JSON preserves numeric types: {:?}", quantities);
}
