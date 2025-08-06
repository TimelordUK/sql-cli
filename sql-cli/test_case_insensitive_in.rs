use serde_json::json;
use sql_cli::csv_datasource::CsvDataSource;

fn main() {
    println!("Testing case-insensitive IN clause:");
    println!();

    // Create test data
    let data = vec![
        json!({"Country": "Ghana", "City": "Accra"}),
        json!({"Country": "Kenya", "City": "Nairobi"}),
        json!({"Country": "NIGERIA", "City": "Lagos"}),
        json!({"Country": "South Africa", "City": "Cape Town"}),
    ];

    let datasource = CsvDataSource::new(data.clone());

    // Test 1: Case-sensitive IN (should match exact case only)
    println!("Test 1: Case-sensitive IN with 'ghana' (lowercase)");
    let where_clause = "Country IN ('ghana', 'kenya')";

    let results = datasource
        .filter_results(data.clone(), where_clause)
        .unwrap();
    println!("Results (case-sensitive): {} rows", results.len());
    for row in &results {
        println!("  - {}", row["Country"]);
    }
    println!();

    // Test 2: Case-insensitive IN (should match regardless of case)
    println!("Test 2: Case-insensitive IN with 'ghana' (lowercase)");

    let results = datasource
        .filter_results_with_options(data.clone(), where_clause, true)
        .unwrap();
    println!("Results (case-insensitive): {} rows", results.len());
    for row in &results {
        println!("  - {}", row["Country"]);
    }
    println!();

    // Test 3: Mixed case in IN list
    println!("Test 3: Case-insensitive IN with mixed case list");
    let where_clause = "Country IN ('GHANA', 'kenya', 'NiGeRiA')";
    let results = datasource
        .filter_results_with_options(data.clone(), where_clause, true)
        .unwrap();
    println!("Results (case-insensitive): {} rows", results.len());
    for row in &results {
        println!("  - {}", row["Country"]);
    }
    println!();

    // Test 4: NOT IN case-insensitive
    println!("Test 4: Case-insensitive NOT IN");
    let where_clause = "Country NOT IN ('ghana', 'kenya')";
    let results = datasource
        .filter_results_with_options(data.clone(), where_clause, true)
        .unwrap();
    println!("Results (case-insensitive NOT IN): {} rows", results.len());
    for row in &results {
        println!("  - {}", row["Country"]);
    }
}
