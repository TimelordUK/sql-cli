use sql_cli::csv_datasource::CsvApiClient;
use serde_json::json;
use std::time::Instant;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing performance with large dataset...");
    
    // Create a large dataset (10k rows)
    let cities = ["New York", "London", "Tokyo", "Berlin", "Paris"];
    let mut data = Vec::new();
    for i in 0..10000 {
        data.push(json!({
            "id": i,
            "name": format!("Customer {}", i),
            "email": format!("customer{}@example.com", i),
            "city": cities[i % 5],
            "amount": (i * 123) % 10000,
            "active": i % 2 == 0,
        }));
    }
    
    let mut client = CsvApiClient::new();
    
    println!("Loading {} rows...", data.len());
    let start = Instant::now();
    client.load_from_json(data, "customers")?;
    println!("Load time: {:?}", start.elapsed());
    
    // Test query performance
    println!("\nTesting query performance...");
    
    // SELECT *
    let start = Instant::now();
    let result = client.query_csv("SELECT * FROM customers")?;
    println!("SELECT * time: {:?}, rows: {}", start.elapsed(), result.data.len());
    
    // SELECT with WHERE
    let start = Instant::now();
    let result = client.query_csv("SELECT * FROM customers WHERE city = \"Tokyo\"")?;
    println!("SELECT with WHERE time: {:?}, rows: {}", start.elapsed(), result.data.len());
    
    // SELECT with ORDER BY
    let start = Instant::now();
    let result = client.query_csv("SELECT * FROM customers ORDER BY amount")?;
    println!("SELECT with ORDER BY time: {:?}, rows: {}", start.elapsed(), result.data.len());
    
    // Complex query
    let start = Instant::now();
    let result = client.query_csv("SELECT id, name, amount FROM customers WHERE active = true ORDER BY amount")?;
    println!("Complex query time: {:?}, rows: {}", start.elapsed(), result.data.len());
    
    println!("\n✅ Performance test complete");
    println!("\nNote: The TUI performance issue with Shift+G is likely due to:");
    println!("1. Ratatui re-rendering the entire table widget");
    println!("2. JSON serialization/deserialization overhead");
    println!("3. String formatting for each visible cell");
    println!("\nVirtual scrolling is already implemented, but jumping to last row");
    println!("might trigger expensive state updates.");
    
    Ok(())
}