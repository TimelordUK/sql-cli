use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;

fn test_where_with_order_by() {
    // Create test data
    let test_data = vec![
        json!({"Index": 1, "Name": "Product A", "Availability": "in_stock", "Price": 20.0}),
        json!({"Index": 2, "Name": "Product B", "Availability": "pre_order", "Price": 15.0}),
        json!({"Index": 3, "Name": "Product C", "Availability": "out_of_stock", "Price": 25.0}),
        json!({"Index": 4, "Name": "Product D", "Availability": "in_stock", "Price": 10.0}),
        json!({"Index": 5, "Name": "Product E", "Availability": "out_of_stock", "Price": 30.0}),
    ];

    // Test 1: WHERE clause alone
    let mut client = CsvApiClient::new();
    client
        .load_from_json(test_data.clone(), "products")
        .unwrap();

    let result = client
        .query_csv(r#"SELECT * FROM products WHERE Availability NOT IN ("pre_order")"#)
        .unwrap();
    println!("Test 1 - WHERE only: {} rows", result.count);
    assert_eq!(result.count, 4); // Should exclude the pre_order item

    // Test 2: ORDER BY clause alone
    let mut client2 = CsvApiClient::new();
    client2
        .load_from_json(test_data.clone(), "products")
        .unwrap();

    let result2 = client2
        .query_csv(r#"SELECT * FROM products ORDER BY Availability"#)
        .unwrap();
    println!("Test 2 - ORDER BY only: {} rows", result2.count);

    // Check ordering
    let first_availability = result2.data[0]["Availability"].as_str().unwrap();
    let last_availability = result2.data[4]["Availability"].as_str().unwrap();
    println!(
        "  First: {}, Last: {}",
        first_availability, last_availability
    );
    assert!(first_availability <= last_availability);

    // Test 3: WHERE + ORDER BY together (the problematic case)
    let mut client3 = CsvApiClient::new();
    client3
        .load_from_json(test_data.clone(), "products")
        .unwrap();

    let query =
        r#"SELECT * FROM products WHERE Availability NOT IN ("pre_order") ORDER BY Availability"#;
    println!("Test 3 Query: {}", query);
    let result3 = client3.query_csv(query).unwrap();
    println!("Test 3 - WHERE + ORDER BY: {} rows", result3.count);
    assert_eq!(result3.count, 4); // Should still exclude the pre_order item

    // Check that results are ordered
    let availabilities: Vec<String> = result3
        .data
        .iter()
        .map(|row| row["Availability"].as_str().unwrap().to_string())
        .collect();

    println!("  Availabilities in order: {:?}", availabilities);

    // Verify ordering
    for i in 1..availabilities.len() {
        assert!(
            availabilities[i - 1] <= availabilities[i],
            "Results not ordered: {} > {}",
            availabilities[i - 1],
            availabilities[i]
        );
    }

    // Test 4: ORDER BY with price to verify numeric sorting works with WHERE
    let mut client4 = CsvApiClient::new();
    client4
        .load_from_json(test_data.clone(), "products")
        .unwrap();

    let result4 = client4
        .query_csv(
            r#"SELECT * FROM products WHERE Availability NOT IN ("pre_order") ORDER BY Price"#,
        )
        .unwrap();
    let prices: Vec<f64> = result4
        .data
        .iter()
        .map(|row| row["Price"].as_f64().unwrap())
        .collect();

    println!("Test 4 - WHERE + ORDER BY Price: {:?}", prices);

    // Verify numeric ordering
    for i in 1..prices.len() {
        assert!(
            prices[i - 1] <= prices[i],
            "Prices not ordered: {} > {}",
            prices[i - 1],
            prices[i]
        );
    }
}

fn main() {
    test_where_with_order_by();
    println!("All tests passed!");
}
