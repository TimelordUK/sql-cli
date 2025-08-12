#[cfg(test)]
mod datatable_conversion_tests {
    use serde_json::json;
    use sql_cli::data::csv_datasource::CsvApiClient;
    use sql_cli::data::datasource_adapter::CsvDataSourceAdapter;
    use sql_cli::data::datasource_trait::DataSource;
    use sql_cli::data::datatable_converter::DataTableConverter;

    #[test]
    fn test_json_to_datatable_conversion() {
        // Create test JSON data
        let json_data = vec![
            json!({
                "name": "Alice",
                "age": 30,
                "salary": 75000.50,
                "active": true
            }),
            json!({
                "name": "Bob",
                "age": 25,
                "salary": 65000.00,
                "active": false
            }),
            json!({
                "name": "Charlie",
                "age": 35,
                "salary": 85000.75,
                "active": true
            }),
        ];

        // Convert to DataTable
        let table = DataTableConverter::from_json_values(&json_data, "test_table")
            .expect("Failed to convert JSON to DataTable");

        // Verify structure
        assert_eq!(table.name, "test_table");
        assert_eq!(table.column_count(), 4);
        assert_eq!(table.row_count(), 3);

        // Verify columns
        let name_col = table.get_column("name").expect("name column should exist");
        assert_eq!(name_col.name, "name");

        let age_col = table.get_column("age").expect("age column should exist");
        assert_eq!(age_col.name, "age");

        // Debug print for verification
        println!("\n=== DataTable Debug Output ===");
        DataTableConverter::debug_print(&table);
    }

    #[test]
    fn test_csv_to_datatable_conversion() {
        // Create test CSV data
        let headers = vec![
            "id".to_string(),
            "product".to_string(),
            "price".to_string(),
            "in_stock".to_string(),
        ];

        let rows = vec![
            vec![
                "1".to_string(),
                "Laptop".to_string(),
                "999.99".to_string(),
                "true".to_string(),
            ],
            vec![
                "2".to_string(),
                "Mouse".to_string(),
                "29.99".to_string(),
                "true".to_string(),
            ],
            vec![
                "3".to_string(),
                "Keyboard".to_string(),
                "79.99".to_string(),
                "false".to_string(),
            ],
        ];

        // Convert to DataTable
        let table = DataTableConverter::from_csv_data(headers, rows, "products")
            .expect("Failed to convert CSV to DataTable");

        // Verify structure
        assert_eq!(table.name, "products");
        assert_eq!(table.column_count(), 4);
        assert_eq!(table.row_count(), 3);

        // Debug print
        println!("\n=== CSV DataTable Debug Output ===");
        DataTableConverter::debug_print(&table);
    }

    #[test]
    fn test_datasource_trait_with_adapter() {
        // Create a CSV client with test data
        let mut client = CsvApiClient::new();

        // Load some test JSON data into the client
        let json_data = vec![
            json!({"id": 1, "name": "Item1", "value": 100}),
            json!({"id": 2, "name": "Item2", "value": 200}),
            json!({"id": 3, "name": "Item3", "value": 300}),
        ];

        client
            .load_from_json(json_data, "test_data")
            .expect("Failed to load JSON data");

        // Wrap in adapter
        let adapter = CsvDataSourceAdapter::new(client);

        // Use through trait
        let data_source: Box<dyn DataSource> = Box::new(adapter);

        // Test querying through the trait
        let response = data_source
            .query("SELECT * FROM test_data")
            .expect("Query should succeed");

        assert_eq!(response.count, 3);
        assert_eq!(response.columns.len(), 3);
        assert!(response.columns.contains(&"id".to_string()));
        assert!(response.columns.contains(&"name".to_string()));
        assert!(response.columns.contains(&"value".to_string()));

        // Test schema access
        let schema = data_source.get_schema();
        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert!(schema.contains_key("test_data"));

        println!("\n=== DataSource Trait Test Output ===");
        println!("Query returned {} rows", response.count);
        println!("Columns: {:?}", response.columns);
        println!("Table name: {}", response.table_name);
    }
}
