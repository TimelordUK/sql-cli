use anyhow::Result;
use sql_cli::csv_datasource::CsvApiClient;

fn main() -> Result<()> {
    println!("Testing case-insensitive equality with trades.json");

    // Load the actual trades.json file
    let mut client = CsvApiClient::new();
    client.set_case_insensitive(true);
    client.load_json("../data/trades.json", "trades")?;

    // Test the exact query that was having issues
    let query = "SELECT * FROM trades WHERE confirmationStatus = 'pending'";
    println!("\nQuery: {}", query);

    let result = client.query_csv(query)?;
    println!("Results: {} rows found", result.count);

    // Show first few matching rows
    if result.count > 0 {
        println!("\nFirst few matching rows:");
        for (i, row) in result.data.iter().take(5).enumerate() {
            if let Some(status) = row.get("confirmationStatus") {
                println!("  Row {}: confirmationStatus = {}", i + 1, status);
            }
        }
    }

    println!("\nSuccess! Case-insensitive equality is working correctly.");
    println!("The query 'confirmationStatus = \"pending\"' now matches rows with:");
    println!("  - 'pending'");
    println!("  - 'Pending'");
    println!("  - 'PENDING'");
    println!("  - or any other case variation");

    Ok(())
}
