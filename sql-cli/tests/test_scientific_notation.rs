use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing scientific notation in numeric comparisons");
    println!("{}", "=".repeat(60));

    // Test data with scientific notation strings
    let scientific_data = json!([
        {
            "id": 1,
            "name": "Small value",
            "value": "1e-4",     // 0.0001
            "amount": "1.5e3"    // 1500
        },
        {
            "id": 2,
            "name": "Medium value",
            "value": "5E-2",     // 0.05
            "amount": "2.5E+3"   // 2500
        },
        {
            "id": 3,
            "name": "Large value",
            "value": "3.14159",  // Regular decimal
            "amount": "1e6"      // 1000000
        },
        {
            "id": 4,
            "name": "Tiny value",
            "value": "2.718e-10", // Very small number
            "amount": "9.8E1"     // 98
        },
        {
            "id": 5,
            "name": "Invalid",
            "value": "NaN",       // Not a number
            "amount": "infinity"  // Infinity
        }
    ]);

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("scientific_test.json");

    // Write JSON data to file
    fs::write(&temp_file, scientific_data.to_string())?;

    // Create CSV client and load the JSON file
    let mut client = CsvApiClient::new();
    client.load_json(&temp_file, "scientific")?;

    // Test queries
    let test_queries = vec![
        ("All records", "SELECT * FROM scientific"),
        (
            "value > 0.001 (values larger than 1e-3)",
            "SELECT * FROM scientific WHERE value > 0.001",
        ),
        (
            "value < 0.1 (values less than 0.1)",
            "SELECT * FROM scientific WHERE value < 0.1",
        ),
        (
            "value BETWEEN 0.00001 AND 1",
            "SELECT * FROM scientific WHERE value BETWEEN 0.00001 AND 1",
        ),
        (
            "amount > 1000 (amounts larger than 1000)",
            "SELECT * FROM scientific WHERE amount > 1000",
        ),
        (
            "amount = 1500 (exact match with 1.5e3)",
            "SELECT * FROM scientific WHERE amount = 1500",
        ),
        (
            "amount < 100 (amounts less than 100)",
            "SELECT * FROM scientific WHERE amount < 100",
        ),
        (
            "Complex: value < 0.01 AND amount > 100",
            "SELECT * FROM scientific WHERE value < 0.01 AND amount > 100",
        ),
    ];

    for (description, query) in test_queries {
        println!("\n{}", "-".repeat(60));
        println!("Query: {}", description);
        println!("SQL: {}", query);

        match client.query_csv(query) {
            Ok(result) => {
                println!("Result count: {} rows", result.data.len());

                // Show values from results
                for row in result.data.iter() {
                    if let Some(obj) = row.as_object() {
                        let id = obj.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let value = obj
                            .get("value")
                            .map(|v| format!("{}", v))
                            .unwrap_or_default();
                        let amount = obj
                            .get("amount")
                            .map(|v| format!("{}", v))
                            .unwrap_or_default();

                        println!(
                            "  Row {}: {} - value={}, amount={}",
                            id, name, value, amount
                        );
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // Clean up
    let _ = fs::remove_file(&temp_file);

    Ok(())
}
