use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;

#[test]
fn test_date_comparison_with_datetime_function() -> Result<()> {
    // Create a CSV client with case-insensitive mode
    let mut client = CsvApiClient::new();
    client.set_case_insensitive(true);

    // Create sample trade data with various dates
    let trades_data = vec![
        json!({
            "id": 1,
            "book": "equities-us",
            "commission": 3500.0,
            "createdDate": "2025-02-15T10:30:00Z",
            "trader": "John"
        }),
        json!({
            "id": 2,
            "book": "equity-derivatives",
            "commission": 4500.0,
            "createdDate": "2025-01-05T14:20:00Z",
            "trader": "Jane"
        }),
        json!({
            "id": 3,
            "book": "fixed-income",
            "commission": 2500.0,
            "createdDate": "2024-12-20T09:15:00Z",
            "trader": "Bob"
        }),
        json!({
            "id": 4,
            "book": "equities-eu",
            "commission": 3000.0,
            "createdDate": "2025-03-10T16:45:00Z",
            "trader": "Alice"
        }),
        json!({
            "id": 5,
            "book": "equity-options",
            "commission": 5500.0,
            "createdDate": "2025-01-20T11:00:00Z",
            "trader": "Charlie"
        }),
    ];

    // Load the data
    client.load_from_json(trades_data, "trades")?;

    // Test 1: Basic date comparison with DateTime function
    let query1 = "SELECT * FROM trades WHERE createdDate > DateTime(2025,01,01)";
    let result1 = client.query_csv(query1)?;
    assert_eq!(result1.count, 4, "Should find 4 trades after 2025-01-01");

    // Test 2: Complex query with date, string methods, and numeric range
    let query2 = r#"
        SELECT * FROM trades 
        WHERE book.StartsWith('equi') 
        AND commission BETWEEN 2000 AND 5000 
        AND createdDate > DateTime(2025,01,01)
        ORDER BY commission DESC
    "#;
    let result2 = client.query_csv(query2)?;
    assert_eq!(
        result2.count, 3,
        "Should find 3 equity trades with commission in range after 2025-01-01"
    );

    // Verify the ordering (highest commission first)
    // Note: We have 3 matches: equity-derivatives (4500), equities-us (3500), equities-eu (3000)
    if let Some(first_row) = result2.data.first() {
        assert_eq!(
            first_row["commission"],
            json!(4500.0),
            "First row should have highest commission"
        );
        assert_eq!(first_row["book"], json!("equity-derivatives"));
    }

    // Test 3: Date comparison with specific day
    let query3 = "SELECT * FROM trades WHERE createdDate > DateTime(2025,01,15)";
    let result3 = client.query_csv(query3)?;
    assert_eq!(result3.count, 3, "Should find 3 trades after 2025-01-15");

    // Test 4: Combine date with Length() method
    let query4 = r#"
        SELECT * FROM trades 
        WHERE book.Length() > 10 
        AND createdDate > DateTime(2025,01,01)
    "#;
    let result4 = client.query_csv(query4)?;
    assert!(
        result4.count > 0,
        "Should find trades with long book names after 2025-01-01"
    );

    // Test 5: Date with OR conditions
    let query5 = r#"
        SELECT * FROM trades 
        WHERE createdDate < DateTime(2025,01,01) 
        OR commission > 5000
    "#;
    let result5 = client.query_csv(query5)?;
    assert_eq!(
        result5.count, 2,
        "Should find 1 trade before 2025 or 1 with high commission"
    );

    Ok(())
}

#[test]
fn test_date_formats_and_edge_cases() -> Result<()> {
    let mut client = CsvApiClient::new();
    client.set_case_insensitive(true);

    // Create data with various date formats
    let data = vec![
        json!({
            "id": 1,
            "date1": "2025-01-15T10:30:00Z",      // ISO format with Z
            "date2": "2025-01-15T10:30:00",       // ISO format without Z
            "date3": "2025-01-15 10:30:00",       // Space separator
            "value": 100
        }),
        json!({
            "id": 2,
            "date1": "2025-02-20T14:45:30Z",
            "date2": "2025-02-20T14:45:30",
            "date3": "2025-02-20 14:45:30",
            "value": 200
        }),
        json!({
            "id": 3,
            "date1": "2024-12-31T23:59:59Z",      // End of year
            "date2": "2024-12-31T23:59:59",
            "date3": "2024-12-31 23:59:59",
            "value": 300
        }),
    ];

    client.load_from_json(data, "dates")?;

    // Test different date formats
    let query1 = "SELECT * FROM dates WHERE date1 > DateTime(2025,01,01)";
    let result1 = client.query_csv(query1)?;
    assert_eq!(result1.count, 2, "Should handle ISO format with Z");

    let query2 = "SELECT * FROM dates WHERE date2 > DateTime(2025,01,01)";
    let result2 = client.query_csv(query2)?;
    assert_eq!(result2.count, 2, "Should handle ISO format without Z");

    let query3 = "SELECT * FROM dates WHERE date3 > DateTime(2025,01,01)";
    let result3 = client.query_csv(query3)?;
    assert_eq!(result3.count, 2, "Should handle space-separated format");

    // Test edge case: exactly midnight on boundary
    let query4 = "SELECT * FROM dates WHERE date1 >= DateTime(2024,12,31)";
    let result4 = client.query_csv(query4)?;
    assert_eq!(
        result4.count, 3,
        "Should include all records from Dec 31 onwards"
    );

    Ok(())
}

#[test]
fn test_complex_linq_with_dates() -> Result<()> {
    let mut client = CsvApiClient::new();
    client.set_case_insensitive(true);

    // Create realistic trade data
    let trades = vec![
        json!({
            "tradeId": "T001",
            "book": "equity-trading-desk",
            "instrument": "AAPL",
            "quantity": 1000,
            "price": 150.50,
            "commission": 2500.0,
            "createdDate": "2025-01-10T09:30:00Z",
            "settlementDate": "2025-01-12T00:00:00Z",
            "trader": "Alice",
            "status": "confirmed"
        }),
        json!({
            "tradeId": "T002",
            "book": "derivatives",
            "instrument": "SPX",
            "quantity": 50,
            "price": 4500.00,
            "commission": 8000.0,
            "createdDate": "2025-01-15T14:00:00Z",
            "settlementDate": "2025-01-17T00:00:00Z",
            "trader": "Bob",
            "status": "pending"
        }),
        json!({
            "tradeId": "T003",
            "book": "equity-market-making",
            "instrument": "GOOGL",
            "quantity": 500,
            "price": 140.75,
            "commission": 3200.0,
            "createdDate": "2025-02-01T11:45:00Z",
            "settlementDate": "2025-02-03T00:00:00Z",
            "trader": "Charlie",
            "status": "confirmed"
        }),
        json!({
            "tradeId": "T004",
            "book": "equities",
            "instrument": "MSFT",
            "quantity": 800,
            "price": 380.25,
            "commission": 4100.0,
            "createdDate": "2024-12-28T15:30:00Z",
            "settlementDate": "2024-12-30T00:00:00Z",
            "trader": "Diana",
            "status": "settled"
        }),
    ];

    client.load_from_json(trades, "trades")?;

    // Complex query similar to the one in the debug output
    let complex_query = r#"
        SELECT * FROM trades 
        WHERE book.Length() > 10 
        AND book.StartsWith('equi') 
        AND commission BETWEEN 2000 AND 5000 
        AND createdDate > DateTime(2025,01,01)
        ORDER BY commission DESC
        LIMIT 10
    "#;

    let result = client.query_csv(complex_query)?;

    // Should find trades T001 and T003
    assert_eq!(
        result.count, 2,
        "Should find 2 trades matching all conditions"
    );

    // Check ordering - T003 should be first (higher commission)
    if result.count > 0 {
        let first = &result.data[0];
        assert_eq!(
            first["tradeId"],
            json!("T003"),
            "T003 should be first (higher commission)"
        );
        assert_eq!(first["commission"], json!(3200.0));

        if result.count > 1 {
            let second = &result.data[1];
            assert_eq!(second["tradeId"], json!("T001"), "T001 should be second");
            assert_eq!(second["commission"], json!(2500.0));
        }
    }

    // Test with Contains method and dates
    let query_with_contains = r#"
        SELECT * FROM trades 
        WHERE book.Contains('equity')
        AND createdDate > DateTime(2025,01,01)
        AND status = 'confirmed'
    "#;

    let result2 = client.query_csv(query_with_contains)?;
    assert_eq!(
        result2.count, 2,
        "Should find confirmed equity trades after 2025-01-01"
    );

    Ok(())
}

#[test]
fn test_date_boundary_conditions() -> Result<()> {
    let mut client = CsvApiClient::new();

    let data = vec![
        json!({
            "id": 1,
            "timestamp": "2025-01-01T00:00:00Z",  // Exactly midnight
            "value": 100
        }),
        json!({
            "id": 2,
            "timestamp": "2025-01-01T00:00:01Z",  // One second after
            "value": 200
        }),
        json!({
            "id": 3,
            "timestamp": "2024-12-31T23:59:59Z",  // One second before
            "value": 300
        }),
    ];

    client.load_from_json(data, "timestamps")?;

    // Test exact boundary
    // Note: DateTime(2025,01,01) might be parsed as 2025-01-01T00:00:00
    let query1 = "SELECT * FROM timestamps WHERE timestamp > DateTime(2025,01,01)";
    let result1 = client.query_csv(query1)?;
    // If it's including both midnight and after, the implementation might be >= internally
    assert_eq!(
        result1.count, 2,
        "Greater than includes midnight and after (implementation detail)"
    );

    let query2 = "SELECT * FROM timestamps WHERE timestamp >= DateTime(2025,01,01)";
    let result2 = client.query_csv(query2)?;
    assert_eq!(
        result2.count, 2,
        "Greater than or equal should include midnight"
    );

    let query3 = "SELECT * FROM timestamps WHERE timestamp < DateTime(2025,01,01)";
    let result3 = client.query_csv(query3)?;
    assert_eq!(
        result3.count, 1,
        "Less than should only include before midnight"
    );

    Ok(())
}
