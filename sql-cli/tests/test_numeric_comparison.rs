use serde_json::json;
use sql_cli::data::csv_datasource::CsvApiClient;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing numeric comparison with commission field");
    println!("{}", "=".repeat(60));

    // Test Case 1: Commission as numeric values (how it should work)
    let numeric_data = json!([
        {
            "id": 1,
            "counterparty": "Bank A",
            "commission": 500.0,
            "amount": 10000
        },
        {
            "id": 2,
            "counterparty": "Bank B",
            "commission": 1500.0,
            "amount": 20000
        },
        {
            "id": 3,
            "counterparty": "Bank C",
            "commission": 2500.0,
            "amount": 30000
        }
    ]);

    // Test Case 2: Commission as string values (like in trades_10k.json)
    let string_data = json!([
        {
            "id": 1,
            "counterparty": "Bank A",
            "commission": "500.0",
            "amount": 10000
        },
        {
            "id": 2,
            "counterparty": "Bank B",
            "commission": "1500.0",
            "amount": 20000
        },
        {
            "id": 3,
            "counterparty": "Bank C",
            "commission": "2500.0",
            "amount": 30000
        }
    ]);

    // Test with numeric commission values
    println!("\n1. Testing with NUMERIC commission values:");
    println!("{}", "-".repeat(40));
    test_commission_queries(numeric_data, "numeric_test")?;

    // Test with string commission values
    println!("\n2. Testing with STRING commission values (like trades_10k.json):");
    println!("{}", "-".repeat(40));
    test_commission_queries(string_data, "string_test")?;

    Ok(())
}

fn test_commission_queries(
    data: serde_json::Value,
    table_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create temp file with .json extension
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("{}.json", table_name));

    // Write JSON data to file
    fs::write(&temp_file, data.to_string())?;

    // Create CSV client and load the JSON file
    let mut client = CsvApiClient::new();
    client.load_json(&temp_file, table_name)?;

    // Test queries
    let test_queries = vec![
        ("All records", format!("SELECT * FROM {}", table_name)),
        (
            "commission > 1000",
            format!("SELECT * FROM {} WHERE commission > 1000", table_name),
        ),
        (
            "commission > 1000.0",
            format!("SELECT * FROM {} WHERE commission > 1000.0", table_name),
        ),
        (
            "commission < 1000",
            format!("SELECT * FROM {} WHERE commission < 1000", table_name),
        ),
        (
            "commission BETWEEN 1000 AND 2000",
            format!(
                "SELECT * FROM {} WHERE commission BETWEEN 1000 AND 2000",
                table_name
            ),
        ),
        (
            "commission = 1500",
            format!("SELECT * FROM {} WHERE commission = 1500", table_name),
        ),
        (
            "commission = '1500.0' (string literal)",
            format!("SELECT * FROM {} WHERE commission = '1500.0'", table_name),
        ),
    ];

    for (description, query) in test_queries {
        println!("\n  Query: {}", description);
        println!("  SQL: {}", query);

        match client.query_csv(&query) {
            Ok(result) => {
                println!("  Result count: {} rows", result.data.len());

                // Show commission values from results
                if !result.data.is_empty() {
                    print!("  Commission values: ");
                    for (i, row) in result.data.iter().enumerate() {
                        if let Some(commission) = row.get("commission") {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{}", commission);
                        }
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
    }

    // Additional type inspection
    println!("\n  Type inspection:");
    let result = client.query_csv(&format!("SELECT * FROM {} LIMIT 1", table_name))?;
    if let Some(first_row) = result.data.first() {
        if let Some(commission) = first_row.get("commission") {
            println!("  Commission value type: {}", value_type_name(commission));
            println!("  Commission raw value: {:?}", commission);
        }
    }

    Ok(())
}

fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
