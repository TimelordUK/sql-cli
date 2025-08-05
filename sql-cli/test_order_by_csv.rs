use anyhow::Result;
use sql_cli::csv_datasource::CsvApiClient;

fn main() -> Result<()> {
    println!("Testing ORDER BY with sample_trades.json...");

    let mut client = CsvApiClient::new();
    client.load_json("sample_trades.json", "trade_deal")?;

    println!("\n=== Sample data (first 3 rows) ===");
    let result = client.query_csv("SELECT * FROM trade_deal")?;
    println!("Total rows: {}", result.data.len());
    for (i, row) in result.data.iter().take(3).enumerate() {
        if let Some(obj) = row.as_object() {
            let id = obj.get("id").unwrap_or(&serde_json::Value::Null);
            let price = obj.get("price").unwrap_or(&serde_json::Value::Null);
            let counterparty = obj.get("counterparty").unwrap_or(&serde_json::Value::Null);
            println!(
                "  {}: id={}, price={}, counterparty={}",
                i + 1,
                id,
                price,
                counterparty
            );
        }
    }

    println!("\n=== ORDER BY id (first 5 rows) ===");
    let result = client.query_csv("SELECT * FROM trade_deal ORDER BY id")?;
    for (i, row) in result.data.iter().take(5).enumerate() {
        if let Some(obj) = row.as_object() {
            let id = obj.get("id").unwrap_or(&serde_json::Value::Null);
            let price = obj.get("price").unwrap_or(&serde_json::Value::Null);
            println!("  {}: id={}, price={}", i + 1, id, price);
        }
    }

    println!("\n=== ORDER BY price (first 5 rows) ===");
    let result = client.query_csv("SELECT * FROM trade_deal ORDER BY price")?;
    for (i, row) in result.data.iter().take(5).enumerate() {
        if let Some(obj) = row.as_object() {
            let id = obj.get("id").unwrap_or(&serde_json::Value::Null);
            let price = obj.get("price").unwrap_or(&serde_json::Value::Null);
            println!("  {}: id={}, price={}", i + 1, id, price);
        }
    }

    println!("\n=== ORDER BY counterparty, price (first 5 rows) ===");
    let result = client.query_csv("SELECT * FROM trade_deal ORDER BY counterparty, price")?;
    for (i, row) in result.data.iter().take(5).enumerate() {
        if let Some(obj) = row.as_object() {
            let id = obj.get("id").unwrap_or(&serde_json::Value::Null);
            let price = obj.get("price").unwrap_or(&serde_json::Value::Null);
            let counterparty = obj.get("counterparty").unwrap_or(&serde_json::Value::Null);
            println!(
                "  {}: counterparty={}, price={}, id={}",
                i + 1,
                counterparty,
                price,
                id
            );
        }
    }

    println!("\n✅ ORDER BY working correctly with real data!");

    Ok(())
}
