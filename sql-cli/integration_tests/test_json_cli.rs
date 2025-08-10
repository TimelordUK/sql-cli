use anyhow::Result;
use sql_cli::csv_datasource::CsvApiClient;

fn main() -> Result<()> {
    println!("Testing JSON file loading and sorting...");

    let mut client = CsvApiClient::new();
    client.load_json("sample_trades.json", "sample_trades")?;

    // Get schema
    if let Some(schema) = client.get_schema() {
        println!("\nSchema loaded successfully!");
        for (table, columns) in schema {
            println!("Table: {}", table);
            println!("Columns: {}", columns.join(", "));
        }
    }

    // Test original data order and types
    println!("\nOriginal data order and types:");
    let result = client.query_csv("SELECT * FROM sample_trades")?;

    for (i, row) in result.data.iter().enumerate() {
        if let Some(obj) = row.as_object() {
            let quantity = obj.get("quantity").unwrap_or(&serde_json::Value::Null);
            let id = obj.get("id").unwrap_or(&serde_json::Value::Null);
            println!(
                "Row {}: id={}, quantity={} (type: {})",
                i + 1,
                id,
                quantity,
                match quantity {
                    serde_json::Value::Number(_) => "Number",
                    serde_json::Value::String(_) => "String",
                    _ => "Other",
                }
            );
        }
    }

    // Test a query
    println!("\nTesting query: SELECT * FROM sample_trades WHERE commission > 80");
    let result = client.query_csv("SELECT * FROM sample_trades WHERE commission > 80")?;

    println!("Results: {} rows", result.data.len());
    for (i, row) in result.data.iter().enumerate() {
        if let Some(obj) = row.as_object() {
            println!("\nRow {}:", i + 1);
            println!(
                "  ID: {}",
                obj.get("id").unwrap_or(&serde_json::Value::Null)
            );
            println!(
                "  Counterparty: {}",
                obj.get("counterparty").unwrap_or(&serde_json::Value::Null)
            );
            println!(
                "  Commission: {}",
                obj.get("commission").unwrap_or(&serde_json::Value::Null)
            );
        }
    }

    // Test LINQ-style query
    println!(
        "\n\nTesting LINQ query: SELECT * FROM sample_trades WHERE counterparty.Contains(\"Bank\")"
    );
    let result2 =
        client.query_csv("SELECT * FROM sample_trades WHERE counterparty.Contains(\"Bank\")")?;
    println!("Results: {} rows", result2.data.len());

    // Test case-insensitive queries
    println!("\n\nTesting case-insensitive queries:");
    println!("Query: SELECT * FROM sample_trades WHERE executionSide.ToLower() = \"buy\"");
    let result3 =
        client.query_csv("SELECT * FROM sample_trades WHERE executionSide.ToLower() = \"buy\"")?;
    println!(
        "Results: {} rows (should match 'BUY' entries)",
        result3.data.len()
    );

    println!("\nQuery: SELECT * FROM sample_trades WHERE status.ToUpper() = \"COMPLETED\"");
    let result4 =
        client.query_csv("SELECT * FROM sample_trades WHERE status.ToUpper() = \"COMPLETED\"")?;
    println!(
        "Results: {} rows (should match 'Completed' entries)",
        result4.data.len()
    );

    Ok(())
}
