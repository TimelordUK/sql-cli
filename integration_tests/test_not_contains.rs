use serde_json::json;
use sql_cli::csv_datasource::CsvApiClient;

fn main() {
    // Create test data with countries
    let test_data = vec![
        json!({"Index": 1, "Country": "Brazil", "City": "São Paulo"}),
        json!({"Index": 2, "Country": "Britain", "City": "London"}),
        json!({"Index": 3, "Country": "Germany", "City": "Berlin"}),
        json!({"Index": 4, "Country": "France", "City": "Paris"}),
        json!({"Index": 5, "Country": "United States", "City": "New York"}),
    ];

    let mut client = CsvApiClient::new();
    client
        .load_from_json(test_data.clone(), "countries")
        .unwrap();

    // Test 1: Countries that contain "Br"
    println!("Test 1 - Countries containing 'Br':");
    let result = client
        .query_csv(r#"SELECT * FROM countries WHERE Country.Contains("Br")"#)
        .unwrap();
    for row in &result.data {
        if let Some(country) = row["Country"].as_str() {
            println!("  {}", country);
        }
    }
    println!("  Count: {}\n", result.data.len());

    // Test 2: Countries that do NOT contain "Br"
    println!("Test 2 - Countries NOT containing 'Br':");
    let result2 = client
        .query_csv(r#"SELECT * FROM countries WHERE NOT Country.Contains("Br")"#)
        .unwrap();
    for row in &result2.data {
        if let Some(country) = row["Country"].as_str() {
            println!("  {}", country);
        }
    }
    println!("  Count: {}\n", result2.data.len());

    // Test 3: Complex condition with NOT
    println!("Test 3 - Countries NOT containing 'Br' AND NOT 'United':");
    let result3 = client.query_csv(r#"SELECT * FROM countries WHERE NOT Country.Contains("Br") AND NOT Country.Contains("United")"#).unwrap();
    for row in &result3.data {
        if let Some(country) = row["Country"].as_str() {
            println!("  {}", country);
        }
    }
    println!("  Count: {}", result3.data.len());
}
