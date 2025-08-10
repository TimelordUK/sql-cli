use anyhow::Result;
use sql_cli::csv_datasource::CsvApiClient;
use std::path::Path;

fn main() -> Result<()> {
    // Use the actual small-customer.csv file (note the hyphen)
    let file_path = "../data/small-customer.csv";

    if !Path::new(file_path).exists() {
        eprintln!("Error: {} not found", file_path);
        eprintln!("Make sure you're running from the sql-cli directory");
        std::process::exit(1);
    }

    println!("Using test CSV at: {}", file_path);

    // Test case-sensitive (default)
    println!("\n=== Case-Sensitive Tests (default) ===");

    let mut csv_client = CsvApiClient::new();
    csv_client.load_csv(&file_path, "small_customer")?;

    // Search for 'panama' - should NOT match 'Panama' (if there's Panama in the data)
    let query = "SELECT * FROM small_customer WHERE Country.Contains('panama')";
    let result = csv_client.query_csv(query)?;
    println!("Query: {}", query);
    println!("Results: {} rows", result.count);
    if result.count > 0 {
        println!("  Found matches (unexpected in case-sensitive mode):");
        for row in &result.data[..result.count.min(3)] {
            if let Some(country) = row.get("Country") {
                println!("    Country: {}", country);
            }
        }
    }

    // Search for exact case match
    let query = "SELECT * FROM small_customer WHERE Country.Contains('Panama')";
    let result = csv_client.query_csv(query)?;
    println!("Query: {}", query);
    println!("Results: {} rows", result.count);
    let panama_count = result.count;
    if result.count > 0 {
        for row in &result.data[..result.count.min(3)] {
            if let Some(country) = row.get("Country") {
                println!("    Country: {}", country);
            }
        }
    }

    // Test case-insensitive
    println!("\n=== Case-INsensitive Tests ===");

    csv_client.set_case_insensitive(true);

    // Search for 'panama' - should match 'Panama'
    let query = "SELECT * FROM small_customer WHERE Country.Contains('panama')";
    let result = csv_client.query_csv(query)?;
    println!("\nQuery: {}", query);
    println!("Results: {} rows", result.count);
    if result.count > 0 {
        println!("  Found matches (case-insensitive):");
        for row in &result.data[..result.count.min(3)] {
            if let Some(country) = row.get("Country") {
                println!("    Country: {}", country);
            }
        }
    }

    // The case-insensitive should find at least as many as case-sensitive
    if panama_count > 0 {
        assert!(
            result.count >= panama_count,
            "Case-insensitive should find at least {} rows but found {}",
            panama_count,
            result.count
        );
    }

    // Test with a name field
    let query = "SELECT * FROM small_customer WHERE \"First Name\".Contains('john')";
    let result = csv_client.query_csv(query)?;
    println!("\nQuery: {}", query);
    println!("Results: {} rows (case-insensitive)", result.count);
    if result.count > 0 {
        for row in &result.data[..result.count.min(3)] {
            if let Some(name) = row.get("First Name") {
                println!("    First Name: {}", name);
            }
        }
    }

    println!("\n✅ All tests passed!");

    Ok(())
}
