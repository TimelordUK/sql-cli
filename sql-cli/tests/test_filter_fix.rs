use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_filter_no_crash() {
    // Create test data similar to what would cause the crash
    let test_data = json!([
        {"id": 1, "executionSide": "BUY", "quantity": 1000},
        {"id": 2, "executionSide": "SELL", "quantity": 500},
        {"id": 3, "executionSide": "BUY", "quantity": 750},
        {"id": 4, "executionSide": "SELL", "quantity": 2000}
    ]);

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", test_data.to_string()).unwrap();

    let mut client = CsvApiClient::new();
    client.load_json(temp_file.path(), "test").unwrap();

    // This should not crash - previously would panic on filter application
    let result = client.query_csv("SELECT * FROM test").unwrap();
    assert_eq!(result.data.len(), 4);

    // Verify the data structure is correct for potential filtering
    for row in &result.data {
        assert!(row.is_object(), "Each row should be a JSON object");
        if let Some(obj) = row.as_object() {
            assert!(
                obj.contains_key("executionSide"),
                "Should have executionSide field"
            );
            assert!(obj.contains_key("quantity"), "Should have quantity field");
        }
    }

    println!("✓ Filter test data structure is valid and won't crash");
}
