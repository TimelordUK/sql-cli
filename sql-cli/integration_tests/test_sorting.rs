use sql_cli::csv_datasource::CsvApiClient;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing numeric sorting...");
    
    let mut client = CsvApiClient::new();
    client.load_json("sample_trades.json", "sample_trades")?;
    
    // Query all records to see initial order
    println!("\nOriginal order:");
    let result = client.query_csv("SELECT * FROM sample_trades")?;
    
    for (i, row) in result.data.iter().enumerate() {
        if let Some(obj) = row.as_object() {
            println!("Row {}: quantity = {}, id = {}", 
                i + 1, 
                obj.get("quantity").unwrap_or(&serde_json::Value::Null),
                obj.get("id").unwrap_or(&serde_json::Value::Null)
            );
        }
    }
    
    println!("\nQuantities in data:");
    for row in &result.data {
        if let Some(obj) = row.as_object() {
            if let Some(quantity) = obj.get("quantity") {
                println!("  {}: {}", quantity, match quantity {
                    serde_json::Value::Number(n) => format!("Number({})", n),
                    serde_json::Value::String(s) => format!("String({})", s),
                    other => format!("Other({:?})", other),
                });
            }
        }
    }
    
    Ok(())
}