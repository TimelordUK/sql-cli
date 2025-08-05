#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::csv_datasource::CsvApiClient;

    #[test]
    fn test_comparison_operators() {
        let test_data = vec![
            json!({
                "id": 1,
                "price": 100.5,
                "quantity": 10,
                "status": "active"
            }),
            json!({
                "id": 2,
                "price": 50.0,
                "quantity": 20,
                "status": "pending"
            }),
            json!({
                "id": 3,
                "price": 75.25,
                "quantity": 15,
                "status": "active"
            }),
            json!({
                "id": 4,
                "price": 100.5,
                "quantity": 5,
                "status": "cancelled"
            }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "test_data").unwrap();
        
        // Test != operator
        let result = csv_client.query_csv("SELECT * FROM test_data WHERE status != \"cancelled\"").unwrap();
        assert_eq!(result.data.len(), 3, "Should exclude cancelled status");
        
        // Test >= operator
        let result = csv_client.query_csv("SELECT * FROM test_data WHERE price >= 75.25").unwrap();
        assert_eq!(result.data.len(), 3, "Should return prices >= 75.25");
        
        // Test <= operator
        let result = csv_client.query_csv("SELECT * FROM test_data WHERE quantity <= 10").unwrap();
        assert_eq!(result.data.len(), 2, "Should return quantities <= 10");
        
        // Test < operator
        let result = csv_client.query_csv("SELECT * FROM test_data WHERE price < 75.25").unwrap();
        assert_eq!(result.data.len(), 1, "Should return prices < 75.25");
    }
    
    #[test]
    fn test_not_in_clause() {
        let test_data = vec![
            json!({ "id": 1, "country": "US" }),
            json!({ "id": 2, "country": "FR" }),
            json!({ "id": 3, "country": "JP" }),
            json!({ "id": 4, "country": "UK" }),
            json!({ "id": 5, "country": "DE" }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "countries").unwrap();
        
        let result = csv_client.query_csv("SELECT * FROM countries WHERE country NOT IN (\"US\", \"UK\")").unwrap();
        assert_eq!(result.data.len(), 3, "Should exclude US and UK");
        
        // Verify the results
        for row in &result.data {
            let country = row["country"].as_str().unwrap();
            assert!(country != "US" && country != "UK");
        }
    }
    
    #[test]
    fn test_between_operator() {
        let test_data = vec![
            json!({ "id": 1, "price": 10.0, "date": "2025-01-01" }),
            json!({ "id": 2, "price": 25.0, "date": "2025-01-15" }),
            json!({ "id": 3, "price": 50.0, "date": "2025-02-01" }),
            json!({ "id": 4, "price": 75.0, "date": "2025-02-15" }),
            json!({ "id": 5, "price": 100.0, "date": "2025-03-01" }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "sales").unwrap();
        
        // Test numeric BETWEEN
        let result = csv_client.query_csv("SELECT * FROM sales WHERE price BETWEEN 25 AND 75").unwrap();
        assert_eq!(result.data.len(), 3, "Should return prices between 25 and 75 inclusive");
        
        // Test date BETWEEN
        let result = csv_client.query_csv("SELECT * FROM sales WHERE date BETWEEN \"2025-01-15\" AND \"2025-02-15\"").unwrap();
        assert_eq!(result.data.len(), 3, "Should return dates between Jan 15 and Feb 15");
    }
    
    #[test]
    fn test_null_handling() {
        let test_data = vec![
            json!({ "id": 1, "name": "John", "email": "john@example.com" }),
            json!({ "id": 2, "name": "Jane", "email": null }),
            json!({ "id": 3, "name": "Bob" }), // missing email field
            json!({ "id": 4, "name": "Alice", "email": "alice@example.com" }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "users").unwrap();
        
        // Test IS NULL
        let result = csv_client.query_csv("SELECT * FROM users WHERE email IS NULL").unwrap();
        assert_eq!(result.data.len(), 2, "Should return rows with null or missing email");
        
        // Test IS NOT NULL
        let result = csv_client.query_csv("SELECT * FROM users WHERE email IS NOT NULL").unwrap();
        assert_eq!(result.data.len(), 2, "Should return rows with non-null email");
    }
    
    #[test]
    fn test_like_pattern_matching() {
        let test_data = vec![
            json!({ "id": 1, "name": "John Smith", "email": "john@gmail.com" }),
            json!({ "id": 2, "name": "Jane Doe", "email": "jane@yahoo.com" }),
            json!({ "id": 3, "name": "Bob Johnson", "email": "bob@gmail.com" }),
            json!({ "id": 4, "name": "Alice Brown", "email": "alice@hotmail.com" }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "contacts").unwrap();
        
        // Test LIKE with % wildcard
        let result = csv_client.query_csv("SELECT * FROM contacts WHERE email LIKE \"%@gmail.com\"").unwrap();
        assert_eq!(result.data.len(), 2, "Should return gmail addresses");
        
        // Test LIKE with multiple wildcards
        let result = csv_client.query_csv("SELECT * FROM contacts WHERE name LIKE \"J%\"").unwrap();
        assert_eq!(result.data.len(), 2, "Should return names starting with J");
        
        // Test LIKE with _ wildcard
        let result = csv_client.query_csv("SELECT * FROM contacts WHERE name LIKE \"J_ne%\"").unwrap();
        assert_eq!(result.data.len(), 1, "Should return Jane");
    }
    
    #[test]
    fn test_complex_combined_queries() {
        let test_data = vec![
            json!({
                "id": 1,
                "product": "Laptop",
                "price": 1200.0,
                "quantity": 5,
                "category": "Electronics",
                "discount": null
            }),
            json!({
                "id": 2,
                "product": "Mouse",
                "price": 25.0,
                "quantity": 50,
                "category": "Electronics",
                "discount": 10.0
            }),
            json!({
                "id": 3,
                "product": "Desk",
                "price": 350.0,
                "quantity": 10,
                "category": "Furniture",
                "discount": 15.0
            }),
            json!({
                "id": 4,
                "product": "Chair",
                "price": 200.0,
                "quantity": 0,
                "category": "Furniture",
                "discount": null
            }),
            json!({
                "id": 5,
                "product": "Monitor",
                "price": 400.0,
                "quantity": 8,
                "category": "Electronics",
                "discount": 20.0
            }),
        ];
        
        let mut csv_client = CsvApiClient::new();
        csv_client.load_from_json(test_data.clone(), "inventory").unwrap();
        
        // Test BETWEEN first
        let query = "SELECT * FROM inventory WHERE price BETWEEN 100 AND 500";
        let result = csv_client.query_csv(query).unwrap();
        println!("BETWEEN test: {} results", result.data.len());
        for item in &result.data {
            println!("  {} - Price: {}", item["product"], item["price"]);
        }
        
        // Complex query 1: Electronics with price between 100-500 and quantity > 0
        // Expected: Monitor (400, qty 8) - Mouse is 25 (too low), Laptop is 1200 (too high)
        let query = "SELECT * FROM inventory WHERE category = \"Electronics\" AND price BETWEEN 100 AND 500 AND quantity > 0";
        let result = csv_client.query_csv(query).unwrap();
        println!("\nComplex query results:");
        for item in &result.data {
            println!("  {} - Price: {}, Category: {}, Quantity: {}", 
                item["product"], item["price"], item["category"], item["quantity"]);
        }
        assert_eq!(result.data.len(), 1, "Should return Monitor only");
        
        // Complex query 2: Items with discount or low quantity
        // Laptop: qty=5 (matches), Mouse: discount=10 (matches), Desk: discount=15 (matches),
        // Chair: qty=0 (matches), Monitor: discount=20 (matches) = 5 items total
        let query = "SELECT * FROM inventory WHERE discount IS NOT NULL OR quantity <= 5";
        let result = csv_client.query_csv(query).unwrap();
        assert_eq!(result.data.len(), 5, "Should return 5 items");
        
        // Complex query 3: Pattern matching with other conditions
        let query = "SELECT * FROM inventory WHERE product LIKE \"M%\" AND category = \"Electronics\" AND price < 500";
        let result = csv_client.query_csv(query).unwrap();
        assert_eq!(result.data.len(), 2, "Should return Mouse and Monitor");
    }
}