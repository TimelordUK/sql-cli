use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;

fn main() {
    // Create test data with null values
    let test_data = vec![
        json!({"Index": 1, "Name": "John", "Age": 25, "City": "London", "Phone": "+44-123"}),
        json!({"Index": 2, "Name": "Jane", "Age": null, "City": "Paris", "Phone": null}),
        json!({"Index": 3, "Name": "Bob", "Age": 30, "City": null, "Phone": "+1-555"}),
        json!({"Index": 4, "Name": "Alice", "Age": null, "City": null, "Phone": null}),
        json!({"Index": 5, "Name": "Charlie", "Age": 28, "City": "Berlin", "Phone": "+49-789"}),
    ];

    let mut client = CsvApiClient::new();
    client.load_from_json(test_data.clone(), "people").unwrap();
    
    println!("=== NULL Handling Tests ===\n");
    
    // Test 1: IS NULL
    println!("Test 1 - People with NULL Age (using IS NULL):");
    let result = client.query_csv(r#"SELECT Name, Age, City FROM people WHERE Age IS NULL"#).unwrap();
    for row in &result.data {
        println!("  {} - Age: {:?}, City: {:?}", 
            row["Name"].as_str().unwrap_or("?"),
            row["Age"],
            row["City"].as_str().unwrap_or("null"));
    }
    println!("  Count: {}\n", result.data.len());
    
    // Test 2: IS NOT NULL
    println!("Test 2 - People with non-NULL Age (using IS NOT NULL):");
    let result2 = client.query_csv(r#"SELECT Name, Age, City FROM people WHERE Age IS NOT NULL"#).unwrap();
    for row in &result2.data {
        println!("  {} - Age: {}, City: {:?}", 
            row["Name"].as_str().unwrap_or("?"),
            row["Age"].as_i64().unwrap_or(0),
            row["City"].as_str().unwrap_or("null"));
    }
    println!("  Count: {}\n", result2.data.len());
    
    // Test 3: Multiple NULL checks
    println!("Test 3 - People with NULL City OR NULL Phone:");
    let result3 = client.query_csv(r#"SELECT Name, City, Phone FROM people WHERE City IS NULL OR Phone IS NULL"#).unwrap();
    for row in &result3.data {
        println!("  {} - City: {:?}, Phone: {:?}", 
            row["Name"].as_str().unwrap_or("?"),
            row["City"].as_str().unwrap_or("null"),
            row["Phone"].as_str().unwrap_or("null"));
    }
    println!("  Count: {}\n", result3.data.len());
    
    // Test 4: Complex condition with NULL
    println!("Test 4 - People with non-NULL Age AND non-NULL City:");
    let result4 = client.query_csv(r#"SELECT Name, Age, City FROM people WHERE Age IS NOT NULL AND City IS NOT NULL"#).unwrap();
    for row in &result4.data {
        println!("  {} - Age: {}, City: {}", 
            row["Name"].as_str().unwrap_or("?"),
            row["Age"].as_i64().unwrap_or(0),
            row["City"].as_str().unwrap_or("?"));
    }
    println!("  Count: {}\n", result4.data.len());
    
    // Test 5: Combining NULL check with other conditions
    println!("Test 5 - People with NULL Phone AND Age > 25:");
    let result5 = client.query_csv(r#"SELECT Name, Age, Phone FROM people WHERE Phone IS NULL AND Age > 25"#).unwrap();
    for row in &result5.data {
        println!("  {} - Age: {}, Phone: {:?}", 
            row["Name"].as_str().unwrap_or("?"),
            row["Age"].as_i64().unwrap_or(0),
            row["Phone"]);
    }
    println!("  Count: {}\n", result5.data.len());

    // Test with actual CSV to show empty field handling
    println!("=== CSV Empty Field Test ===");
    let csv_content = "Name,Age,City,Phone\nJohn,25,London,+44-123\nJane,,Paris,\nBob,30,,+1-555\nAlice,,,\n";
    std::fs::write("test_nulls.csv", csv_content).unwrap();
    
    let mut csv_client = CsvApiClient::new();
    csv_client.load_csv("test_nulls.csv", "people_csv").unwrap();
    
    println!("\nTest 6 - CSV with empty fields (should be NULL):");
    let result6 = csv_client.query_csv(r#"SELECT * FROM people_csv WHERE Age IS NULL OR City IS NULL"#).unwrap();
    for row in &result6.data {
        println!("  {} - Age: {:?}, City: {:?}, Phone: {:?}", 
            row["Name"].as_str().unwrap_or("?"),
            row["Age"],
            row["City"],
            row["Phone"]);
    }
    println!("  Count: {}", result6.data.len());
    
    // Clean up
    std::fs::remove_file("test_nulls.csv").ok();
}