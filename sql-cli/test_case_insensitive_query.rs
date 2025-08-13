use sql_cli::csv_datasource::CsvApiClient;
use anyhow::Result;

fn main() -> Result<()> {
    // Test case-sensitive (default = false)
    println!("Testing case-sensitive mode (default):");
    let mut client = CsvApiClient::new();
    client.load_json("test_trades.json", "trades")?;
    
    let result = client.query_csv("SELECT * FROM trades WHERE confirmationStatus = 'pending'")?;
    println!("  Query: confirmationStatus = 'pending'");
    println!("  Results with case-sensitive: {} rows", result.count);
    
    // Test case-insensitive
    println!("\nTesting case-insensitive mode:");
    let mut client = CsvApiClient::new();
    client.set_case_insensitive(true);
    client.load_json("test_trades.json", "trades")?;
    
    let result = client.query_csv("SELECT * FROM trades WHERE confirmationStatus = 'pending'")?;
    println!("  Query: confirmationStatus = 'pending'");
    println!("  Results with case-insensitive: {} rows", result.count);
    println!("  Expected: 3 rows (Pending, pending, PENDING)");
    
    // Show the actual values
    if result.count > 0 {
        println!("\n  Matched rows:");
        for row in &result.data {
            if let Some(status) = row.get("confirmationStatus") {
                if let Some(trader) = row.get("traderId") {
                    println!("    - confirmationStatus: {}, traderId: {}", status, trader);
                }
            }
        }
    }
    
    Ok(())
}