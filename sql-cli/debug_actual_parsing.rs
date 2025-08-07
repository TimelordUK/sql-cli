use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;
use sql_cli::recursive_parser::Parser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing actual query execution with LIMIT");

    // Test data - same as trades
    let test_data = json!([
        {"id": 1, "commission": "100.0", "counterparty": "Bank A"},
        {"id": 2, "commission": "200.0", "counterparty": "Bank B"},
        {"id": 3, "commission": "300.0", "counterparty": "Bank C"},
        {"id": 4, "commission": "400.0", "counterparty": "Bank D"},
        {"id": 5, "commission": "500.0", "counterparty": "Bank E"},
        {"id": 6, "commission": "600.0", "counterparty": "Bank F"},
        {"id": 7, "commission": "700.0", "counterparty": "Bank G"},
        {"id": 8, "commission": "800.0", "counterparty": "Bank H"},
        {"id": 9, "commission": "900.0", "counterparty": "Bank I"},
        {"id": 10, "commission": "1000.0", "counterparty": "Bank J"}
    ]);

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("debug_limit.json");

    fs::write(&temp_file, test_data.to_string())?;

    // Create CSV client and load the JSON file
    let mut client = CsvApiClient::new();
    client.load_json(&temp_file, "test_data")?;

    let query = "SELECT * FROM test_data LIMIT 3";
    println!("\nQuery: {}", query);

    // Test the parser directly first
    println!("\n1. Testing parser directly:");
    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => {
            println!("  Parsed successfully!");
            println!("  limit: {:?}", stmt.limit);
            println!("  offset: {:?}", stmt.offset);
        }
        Err(e) => {
            println!("  Parse error: {}", e);
        }
    }

    // Test actual query execution
    println!("\n2. Testing query execution:");
    match client.query_csv(query) {
        Ok(result) => {
            println!("  Query succeeded!");
            println!("  Result count: {} rows", result.data.len());

            for row in result.data.iter() {
                if let Some(obj) = row.as_object() {
                    let id = obj.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let commission = obj
                        .get("commission")
                        .map(|v| format!("{}", v))
                        .unwrap_or_default();
                    println!("    Row {}: commission={}", id, commission);
                }
            }
        }
        Err(e) => {
            println!("  Query error: {}", e);
        }
    }

    // Test a query that might trigger fallback
    let complex_query = "SELECT * FROM test_data WHERE commission > '200' LIMIT 2";
    println!("\nQuery with WHERE: {}", complex_query);

    println!("\n3. Testing parser with WHERE:");
    let mut parser2 = Parser::new(complex_query);
    match parser2.parse() {
        Ok(stmt) => {
            println!("  Parsed successfully!");
            println!("  limit: {:?}", stmt.limit);
            println!("  where_clause: {:?}", stmt.where_clause.is_some());
        }
        Err(e) => {
            println!("  Parse error: {}", e);
        }
    }

    println!("\n4. Testing query execution with WHERE:");
    match client.query_csv(complex_query) {
        Ok(result) => {
            println!("  Query succeeded!");
            println!("  Result count: {} rows", result.data.len());

            for row in result.data.iter() {
                if let Some(obj) = row.as_object() {
                    let id = obj.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let commission = obj
                        .get("commission")
                        .map(|v| format!("{}", v))
                        .unwrap_or_default();
                    println!("    Row {}: commission={}", id, commission);
                }
            }
        }
        Err(e) => {
            println!("  Query error: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_file(&temp_file);

    Ok(())
}
