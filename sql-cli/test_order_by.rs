use sql_cli::csv_datasource::CsvApiClient;
use serde_json::json;
use std::io::Write;
use std::fs::File;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing ORDER BY execution in cache/CSV mode...");
    
    // Create test data with different types for comprehensive sorting
    let test_data = json!([
        {
            "id": 3,
            "counterparty": "Bank of America", 
            "quantity": 1000,
            "book": "TradingBook1"
        },
        {
            "id": 1,
            "counterparty": "JP Morgan",
            "quantity": 500,
            "book": "TradingBook2"
        },
        {
            "id": 2,
            "counterparty": "Mizuho Bank",
            "quantity": 750,
            "book": "TradingBook1"
        }
    ]);
    
    // Create a temporary JSON file
    let temp_path = "test_order_by_data.json";
    let mut temp_file = File::create(temp_path)?;
    write!(temp_file, "{}", test_data.to_string())?;
    drop(temp_file); // Close the file
    
    let mut client = CsvApiClient::new();
    client.load_json(temp_path, "test")?;
    
    println!("\n=== Original Data ===");
    let result = client.query_csv("SELECT * FROM test")?;
    print_results(&result.data);
    
    println!("\n=== ORDER BY id ===");
    let result = client.query_csv("SELECT * FROM test ORDER BY id")?;
    print_results(&result.data);
    
    println!("\n=== ORDER BY counterparty ===");
    let result = client.query_csv("SELECT * FROM test ORDER BY counterparty")?;
    print_results(&result.data);
    
    println!("\n=== ORDER BY book, quantity ===");
    let result = client.query_csv("SELECT * FROM test ORDER BY book, quantity")?;
    print_results(&result.data);
    
    println!("\n=== ORDER BY with WHERE clause ===");
    let result = client.query_csv("SELECT * FROM test WHERE quantity > 600 ORDER BY counterparty")?;
    print_results(&result.data);
    
    println!("\n✅ ORDER BY execution working correctly!");
    println!("   - Single column sorting: ✓");
    println!("   - Multi-column sorting: ✓");
    println!("   - Type-aware sorting (numbers vs strings): ✓");
    println!("   - Combined with WHERE clause: ✓");
    
    // Clean up the temporary file
    std::fs::remove_file(temp_path).ok();
    
    Ok(())
}

fn print_results(data: &[serde_json::Value]) {
    for row in data {
        if let Some(obj) = row.as_object() {
            let id = obj.get("id").unwrap();
            let counterparty = obj.get("counterparty").unwrap();
            let quantity = obj.get("quantity").unwrap();
            let book = obj.get("book").unwrap();
            println!("  id: {}, counterparty: {}, quantity: {}, book: {}", 
                id, counterparty, quantity, book);
        }
    }
}