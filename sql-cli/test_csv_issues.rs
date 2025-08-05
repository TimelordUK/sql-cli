use anyhow::Result;
use sql_cli::csv_datasource::CsvApiClient;
use sql_cli::cursor_aware_parser::CursorAwareParser;
use sql_cli::parser::{Schema, SqlParser};

fn main() -> Result<()> {
    println!("Testing real-world CSV issues...\n");

    // Test 1: Table name with hyphen
    println!("=== Test 1: Table name with hyphen ===");
    test_table_with_hyphen()?;

    // Test 2: Column names with spaces
    println!("\n=== Test 2: Column names with spaces ===");
    test_columns_with_spaces()?;

    // Test 3: Case sensitivity
    println!("\n=== Test 3: Case sensitivity in column names ===");
    test_case_sensitivity()?;

    Ok(())
}

fn test_table_with_hyphen() -> Result<()> {
    let mut client = CsvApiClient::new();

    // Simulate loading a file with hyphen in name
    println!("Loading CSV as table 'customers-10000'...");
    client.load_csv("test_real_world_issues.csv", "customers-10000")?;

    // Test if we can query it
    match client.query_csv("SELECT * FROM \"customers-10000\" LIMIT 1") {
        Ok(result) => println!(
            "✓ Query with quoted table name works: {} rows",
            result.data.len()
        ),
        Err(e) => println!("✗ Query failed: {}", e),
    }

    // Test tab completion
    let mut parser = CursorAwareParser::new();
    let schema = Schema::new();
    parser.update_single_table(
        "customers-10000".to_string(),
        vec![
            "Customer ID".to_string(),
            "First Name".to_string(),
            "Phone 1".to_string(),
        ],
    );

    let query = "SELECT ";
    let result = parser.get_completions(query, query.len());
    println!("Tab completion after 'SELECT ': {:?}", result.suggestions);

    Ok(())
}

fn test_columns_with_spaces() -> Result<()> {
    let mut client = CsvApiClient::new();
    client.load_csv("test_real_world_issues.csv", "customers")?;

    // Test queries with column names containing spaces
    println!("\nTesting column 'Phone 1' (with space):");

    // Without quotes - should fail
    match client.query_csv("SELECT Phone 1 FROM customers") {
        Ok(_) => println!("✗ Unquoted column with space incorrectly accepted"),
        Err(e) => println!("✓ Unquoted column correctly rejected: {}", e),
    }

    // With quotes - should work
    match client.query_csv("SELECT \"Phone 1\" FROM customers") {
        Ok(result) => println!("✓ Quoted column works: {} rows", result.data.len()),
        Err(e) => println!("✗ Quoted column failed: {}", e),
    }

    // Test ORDER BY with quoted column
    println!("\nTesting ORDER BY with quoted column:");
    match client
        .query_csv("SELECT \"Customer ID\", \"Phone 1\" FROM customers ORDER BY \"Phone 1\"")
    {
        Ok(result) => {
            println!(
                "✓ ORDER BY with quoted column works: {} rows",
                result.data.len()
            );
            // Print first few results to verify sorting
            for (i, row) in result.data.iter().take(3).enumerate() {
                if let Some(obj) = row.as_object() {
                    println!("  {}: Phone 1 = {:?}", i + 1, obj.get("Phone 1"));
                }
            }
        }
        Err(e) => println!("✗ ORDER BY with quoted column failed: {}", e),
    }

    Ok(())
}

fn test_case_sensitivity() -> Result<()> {
    let mut client = CsvApiClient::new();
    client.load_csv("test_real_world_issues.csv", "customers")?;

    // CSV headers are: City (capital C)
    println!("\nTesting case sensitivity:");

    // Lowercase - might not work
    match client.query_csv("SELECT city FROM customers") {
        Ok(result) => println!(
            "✗ Lowercase 'city' incorrectly accepted: {} rows",
            result.data.len()
        ),
        Err(e) => println!("✓ Lowercase 'city' correctly rejected: {}", e),
    }

    // Correct case - should work
    match client.query_csv("SELECT City FROM customers") {
        Ok(result) => println!("✓ Correct case 'City' works: {} rows", result.data.len()),
        Err(e) => println!("✗ Correct case 'City' failed: {}", e),
    }

    // Test ORDER BY with correct case
    match client.query_csv("SELECT City FROM customers ORDER BY City") {
        Ok(result) => {
            println!(
                "✓ ORDER BY with correct case works: {} rows",
                result.data.len()
            );
            for (i, row) in result.data.iter().take(3).enumerate() {
                if let Some(obj) = row.as_object() {
                    println!("  {}: City = {:?}", i + 1, obj.get("City"));
                }
            }
        }
        Err(e) => println!("✗ ORDER BY failed: {}", e),
    }

    // Test tab completion suggestions
    let mut parser = CursorAwareParser::new();
    parser.update_single_table(
        "customers".to_string(),
        vec![
            "Customer ID".to_string(),
            "First Name".to_string(),
            "City".to_string(),
            "Phone 1".to_string(),
        ],
    );

    let query = "SELECT Ci";
    let result = parser.get_completions(query, query.len());
    println!("\nTab completion for 'SELECT Ci': {:?}", result.suggestions);
    println!("Note: Should suggest 'City' not 'city'");

    Ok(())
}
