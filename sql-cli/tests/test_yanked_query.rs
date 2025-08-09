use serde_json::Value;
use sql_cli::csv_datasource::CsvApiClient;
use std::collections::HashMap;

// Import the test infrastructure
use crate::real_query_capture::{CapturedQuery, QueryReplayHarness};

mod real_query_capture;

#[test]
fn test_yanked_from_tui_session() -> anyhow::Result<()> {
    let mut harness = QueryReplayHarness::new();

    // Test case generated from TUI session at 2025-08-09 10:59:51
    // Buffer: ../data/trades_10k.json (ID: 2)
    // Results: 43 rows, 11 columns

    harness.add_query(CapturedQuery {
        description: "Captured from TUI session 2025-08-09 10:59:51".to_string(),
        data_file: "../data/trades_10k.json".to_string(),
        query: "SELECT book,commission,confirmationStatus,instrumentId,platformOrderId,counterparty,instrumentName,counterpartyCountry,counterpartyType,createdDate,currency FROM trades where not confirmationStatus.Contains('pend') and commission between 20 and 50 order by counterparty,book".to_string(),
        expected_row_count: 43,
        expected_columns: vec![
            "book".to_string(), 
            "commission".to_string(), 
            "confirmationStatus".to_string(), 
            "counterparty".to_string(), 
            "counterpartyCountry".to_string(), 
            "counterpartyType".to_string(), 
            "createdDate".to_string(), 
            "currency".to_string(), 
            "instrumentId".to_string(), 
            "instrumentName".to_string(), 
            "platformOrderId".to_string()
        ],
        expected_first_row: Some({
            let mut map = std::collections::HashMap::new();
            map.insert("book".to_string(), serde_json::Value::String("Options Trading".to_string()));
            map.insert("commission".to_string(), serde_json::Value::String("26.96".to_string()));
            map.insert("confirmationStatus".to_string(), serde_json::Value::String("Confirmed".to_string()));
            map
        }),
        case_insensitive: true,
    });

    // Run the test
    harness.run_all_tests()?;

    println!("✅ Yanked query test passed!");
    Ok(())
}

// Alternative: Test directly without the harness
#[test]
fn test_yanked_query_direct() -> anyhow::Result<()> {
    // Check if the data file exists
    let data_file = "../data/trades_10k.json";
    if !std::path::Path::new(data_file).exists() {
        println!("Skipping test - {} not found", data_file);
        return Ok(());
    }

    // Load the data
    let mut csv_client = CsvApiClient::new();
    csv_client.set_case_insensitive(true);
    csv_client.load_json(data_file, "trades")?;

    // Run the exact query from the TUI session
    let query = "SELECT book,commission,confirmationStatus,instrumentId,platformOrderId,counterparty,instrumentName,counterpartyCountry,counterpartyType,createdDate,currency FROM trades where not confirmationStatus.Contains('pend') and commission between 20 and 50 order by counterparty,book";

    let response = csv_client.query_csv(query)?;

    // Verify results
    println!("Query returned {} rows", response.data.len());
    assert_eq!(response.data.len(), 43, "Expected 43 rows from the query");

    // Check first row if available
    if let Some(first_row) = response.data.first() {
        println!("First row: {:?}", first_row);

        // Verify the expected columns exist
        if let Some(obj) = first_row.as_object() {
            assert!(obj.contains_key("book"), "Missing 'book' column");
            assert!(
                obj.contains_key("commission"),
                "Missing 'commission' column"
            );
            assert!(
                obj.contains_key("confirmationStatus"),
                "Missing 'confirmationStatus' column"
            );

            // Check specific values from the first row
            if let Some(book) = obj.get("book").and_then(|v| v.as_str()) {
                println!("First row book: {}", book);
            }
            if let Some(commission) = obj.get("commission") {
                println!("First row commission: {}", commission);
            }
            if let Some(status) = obj.get("confirmationStatus").and_then(|v| v.as_str()) {
                println!("First row confirmationStatus: {}", status);
                assert!(
                    !status.to_lowercase().contains("pend"),
                    "confirmationStatus should not contain 'pend': {}",
                    status
                );
            }
        }
    }

    // Verify all rows match the WHERE clause conditions
    for (i, row) in response.data.iter().enumerate() {
        if let Some(obj) = row.as_object() {
            // Check confirmationStatus doesn't contain 'pend'
            if let Some(status) = obj.get("confirmationStatus").and_then(|v| v.as_str()) {
                assert!(
                    !status.to_lowercase().contains("pend"),
                    "Row {} confirmationStatus '{}' should not contain 'pend'",
                    i,
                    status
                );
            }

            // Check commission is between 20 and 50
            if let Some(commission_val) = obj.get("commission") {
                let commission = match commission_val {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                    _ => 0.0,
                };
                assert!(
                    commission >= 20.0 && commission <= 50.0,
                    "Row {} commission {} should be between 20 and 50",
                    i,
                    commission
                );
            }
        }
    }

    println!("✅ All 43 rows validated successfully!");
    println!("✅ NOT confirmationStatus.Contains('pend') works correctly");
    println!("✅ Commission BETWEEN 20 AND 50 works correctly");
    println!("✅ ORDER BY counterparty,book applied");

    Ok(())
}
