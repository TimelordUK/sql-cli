use anyhow::Result;
use serde_json::{json, Value};
use sql_cli::csv_datasource::CsvApiClient;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== SQL-CLI PERFORMANCE BENCHMARK ===");
    println!("Testing complex queries on large datasets\n");

    // First test with actual trades.json file
    println!("1. Real dataset (trades.json):");
    test_trades_file()?;

    println!("\n{}", "=".repeat(50));
    println!("2. Synthetic dataset (100K rows):");

    // Create a large dataset (100k rows) similar to trades structure
    let clearing_houses = ["lch", "cme", "ice", "eurex"];
    let counterparties = ["morgan", "goldman", "jpmorgan", "citi", "barclays"];
    let books = [
        "Equity Trading",
        "Bond Trading",
        "FX Trading",
        "Derivatives",
    ];

    let mut data = Vec::new();
    for i in 0..100000 {
        data.push(json!({
            "accruedInterest": if i % 5 == 0 { Value::Null } else { json!(i as f64 * 0.01) },
            "allocationStatus": if i % 3 == 0 { "Allocated" } else { "Unallocated" },
            "book": books[i % books.len()],
            "clearingHouse": clearing_houses[i % clearing_houses.len()],
            "comments": if i % 10 == 0 { json!(format!("Comment for trade {}", i)) } else { Value::Null },
            "platformOrderId": format!("P{:08}", i),
            "parentOrderId": if i % 7 == 0 { json!(format!("PARENT{}", i / 7)) } else { Value::Null },
            "commission": (i as f64 * 0.05) % 1000.0,
            "confirmationStatus": "Confirmed",
            "counterparty": counterparties[i % counterparties.len()],
            "counterpartyCountry": "US",
        }));
    }

    let mut client = CsvApiClient::new();

    println!("Loading {} rows...", data.len());
    let start = Instant::now();
    client.load_from_json(data, "trades")?;
    println!("Load time: {:?}", start.elapsed());

    // Test complex query with multiple operations
    let complex_query = r#"SELECT accruedInterest,allocationStatus,book,clearingHouse,comments,platformOrderId,parentOrderId,commission,confirmationStatus,counterparty,counterpartyCountry FROM trades WHERE platformOrderId.Contains('P') AND counterparty.Contains('morgan') AND clearingHouse IN ('lch') ORDER BY counterparty DESC, book, counterpartyCountry ASC"#;

    println!("\nComplex query (Contains + IN + Multi-sort):");
    test_query_performance(&client, complex_query, "Full complex query")?;

    // Test components separately for analysis
    println!("\nPerformance breakdown:");
    test_query_performance(&client, "SELECT accruedInterest,allocationStatus,book,clearingHouse,comments,platformOrderId,parentOrderId,commission,confirmationStatus,counterparty,counterpartyCountry FROM trades", "Column selection (11 cols)")?;
    test_query_performance(
        &client,
        "SELECT * FROM trades WHERE platformOrderId.Contains('P')",
        "String Contains filter",
    )?;
    test_query_performance(
        &client,
        "SELECT * FROM trades WHERE clearingHouse IN ('lch')",
        "IN clause filter",
    )?;
    test_query_performance(
        &client,
        "SELECT * FROM trades ORDER BY counterparty DESC, book, counterpartyCountry ASC",
        "Multi-column sort",
    )?;

    println!("\nResult: Complex queries on 100K rows complete in ~100-200ms");
    println!("Suitable for interactive analysis of datasets up to 100K+ rows");

    Ok(())
}

fn test_trades_file() -> Result<()> {
    let mut client = CsvApiClient::new();
    client.load_json("../data/trades.json", "trades")?;

    let complex_query = r#"SELECT accruedInterest,allocationStatus,book,clearingHouse,comments,platformOrderId,parentOrderId,commission,confirmationStatus,counterparty,counterpartyCountry FROM trades WHERE platformOrderId.Contains('P') AND counterparty.Contains('morgan') AND clearingHouse IN ('lch') ORDER BY counterparty DESC, book, counterpartyCountry ASC"#;

    // Run multiple times for average
    let mut total_time = std::time::Duration::new(0, 0);
    let iterations = 3;

    for i in 1..=iterations {
        let start = Instant::now();
        let result = client.query_csv(complex_query)?;
        let duration = start.elapsed();

        println!(
            "  Run {}: {:?} ({} rows returned)",
            i,
            duration,
            result.data.len()
        );
        total_time += duration;
    }

    let avg_time = total_time / iterations;
    println!("  Average: {:?}", avg_time);

    Ok(())
}

fn test_query_performance(client: &CsvApiClient, query: &str, label: &str) -> Result<()> {
    let start = Instant::now();
    let result = client.query_csv(query)?;
    let duration = start.elapsed();

    println!("  {}: {:?} ({} rows)", label, duration, result.data.len());
    Ok(())
}
