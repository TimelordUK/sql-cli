#[cfg(test)]
mod tests {
    use crate::csv_datasource::{CsvApiClient, CsvDataSource};
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_json_file_loading() {
        // Create test JSON data - array of flat objects like API would return
        let test_data = json!([
            {
                "id": 1,
                "platformOrderId": "ORDER-001",
                "tradeDate": "2024-01-15",
                "counterparty": "Bank of America",
                "counterpartyCountry": "US",
                "quantity": 1000,
                "price": 150.50,
                "commission": 75.25,
                "status": "Completed"
            },
            {
                "id": 2,
                "platformOrderId": "ORDER-002",
                "tradeDate": "2024-01-16",
                "counterparty": "JP Morgan",
                "counterpartyCountry": "US",
                "quantity": 500,
                "price": 200.00,
                "commission": 100.00,
                "status": "Pending"
            },
            {
                "id": 3,
                "platformOrderId": "ORDER-003",
                "tradeDate": "2024-01-17",
                "counterparty": "Mizuho Bank",
                "counterpartyCountry": "JP",
                "quantity": 750,
                "price": 175.75,
                "commission": 87.50,
                "status": "Completed"
            }
        ]);
        
        // Write to temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", test_data.to_string()).unwrap();
        
        // Load JSON file
        let datasource = CsvDataSource::load_from_json_file(temp_file.path(), "trades").unwrap();
        
        // Verify headers were extracted
        let headers = datasource.get_headers();
        assert!(headers.contains(&"id".to_string()));
        assert!(headers.contains(&"platformOrderId".to_string()));
        assert!(headers.contains(&"counterparty".to_string()));
        assert!(headers.contains(&"commission".to_string()));
        
        // Verify data loaded correctly
        assert_eq!(datasource.get_row_count(), 3);
        assert_eq!(datasource.get_table_name(), "trades");
    }
    
    #[test]
    fn test_json_queries_with_where_clause() {
        let test_data = json!([
            {
                "id": 1,
                "counterparty": "Bank of America",
                "counterpartyCountry": "US",
                "commission": 75.25
            },
            {
                "id": 2,
                "counterparty": "JP Morgan",
                "counterpartyCountry": "US", 
                "commission": 100.00
            },
            {
                "id": 3,
                "counterparty": "Mizuho Bank",
                "counterpartyCountry": "JP",
                "commission": 87.50
            },
            {
                "id": 4,
                "counterparty": "BNP Paribas",
                "counterpartyCountry": "FR",
                "commission": 45.00
            }
        ]);
        
        // Write to temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", test_data.to_string()).unwrap();
        
        // Test with CsvApiClient
        let mut client = CsvApiClient::new();
        client.load_json(temp_file.path(), "trades").unwrap();
        
        // Test 1: Simple WHERE clause
        let result = client.query_csv("SELECT * FROM trades WHERE commission > 80").unwrap();
        assert_eq!(result.data.len(), 2);
        
        // Test 2: LINQ-style Contains
        let result = client.query_csv("SELECT * FROM trades WHERE counterparty.Contains(\"Bank\")").unwrap();
        assert_eq!(result.data.len(), 2); // Bank of America and Mizuho Bank
        
        // Test 3: IN clause
        let result = client.query_csv("SELECT * FROM trades WHERE counterpartyCountry IN (\"JP\", \"FR\")").unwrap();
        assert_eq!(result.data.len(), 2);
        
        // Test 4: Complex AND conditions
        let result = client.query_csv("SELECT * FROM trades WHERE commission > 50 AND counterpartyCountry = \"US\"").unwrap();
        assert_eq!(result.data.len(), 2);
    }
    
    #[test]
    fn test_json_and_csv_produce_same_cache() {
        // Create identical data in JSON format
        let json_data = json!([
            {"id": 1, "name": "Alice", "age": 30, "city": "New York"},
            {"id": 2, "name": "Bob", "age": 25, "city": "London"},
            {"id": 3, "name": "Charlie", "age": 35, "city": "Tokyo"}
        ]);
        
        // Write JSON file
        let mut json_file = NamedTempFile::new().unwrap();
        write!(json_file, "{}", json_data.to_string()).unwrap();
        
        // Write CSV file with same data
        let mut csv_file = NamedTempFile::new().unwrap();
        writeln!(csv_file, "id,name,age,city").unwrap();
        writeln!(csv_file, "1,Alice,30,New York").unwrap();
        writeln!(csv_file, "2,Bob,25,London").unwrap();
        writeln!(csv_file, "3,Charlie,35,Tokyo").unwrap();
        
        // Load both files
        let mut json_client = CsvApiClient::new();
        json_client.load_json(json_file.path(), "people").unwrap();
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_csv(csv_file.path(), "people").unwrap();
        
        // Run same query on both
        let json_result = json_client.query_csv("SELECT * FROM people WHERE age > 28").unwrap();
        let csv_result = csv_client.query_csv("SELECT * FROM people WHERE age > 28").unwrap();
        
        // Results should be identical
        assert_eq!(json_result.data.len(), csv_result.data.len());
        assert_eq!(json_result.data.len(), 2); // Alice and Charlie
        
        // Check schemas are the same
        let json_schema = json_client.get_schema().unwrap();
        let csv_schema = csv_client.get_schema().unwrap();
        
        assert_eq!(json_schema.get("people").unwrap().len(), csv_schema.get("people").unwrap().len());
    }
    
    #[test]
    fn test_json_validation() {
        // Test empty array
        let empty_data = json!([]);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", empty_data.to_string()).unwrap();
        
        let result = CsvDataSource::load_from_json_file(temp_file.path(), "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no data"));
        
        // Test non-object array
        let invalid_data = json!([1, 2, 3]);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_data.to_string()).unwrap();
        
        let result = CsvDataSource::load_from_json_file(temp_file.path(), "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be objects"));
        
        // Test mixed types (should work but second record validation should catch)
        let mixed_data = json!([
            {"id": 1, "name": "Alice"},
            "not an object"
        ]);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", mixed_data.to_string()).unwrap();
        
        let result = CsvDataSource::load_from_json_file(temp_file.path(), "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Record 1 is not an object"));
    }
}