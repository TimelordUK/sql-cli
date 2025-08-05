use sql_cli::csv_datasource::CsvApiClient;

fn main() -> anyhow::Result<()> {
    let mut csv_client = CsvApiClient::new();

    // Load the CSV file
    csv_client.load_csv(
        "/home/me/dev/sql-cli/BusinessCrimeBoroughLevel.csv",
        "crime_data",
    )?;

    println!("Testing LINQ-style methods on CSV data\n");

    // Test queries demonstrating different methods
    let queries = vec![
        // .Contains() method
        (
            "SELECT * FROM crime_data WHERE Borough.Contains('Aviation')",
            "Contains: Boroughs containing 'Aviation'"
        ),

        // .StartsWith() method
        (
            "SELECT * FROM crime_data WHERE Borough.StartsWith('B')",
            "StartsWith: Boroughs starting with 'B'"
        ),

        // .EndsWith() method
        (
            "SELECT * FROM crime_data WHERE \"Minor Class Description\".EndsWith('Dwelling')",
            "EndsWith: Crimes ending with 'Dwelling'"
        ),

        // .Length() method with comparison
        (
            "SELECT * FROM crime_data WHERE Borough.Length() > 10",
            "Length > 10: Boroughs with names longer than 10 chars"
        ),

        // .Length() method with exact match
        (
            "SELECT * FROM crime_data WHERE Borough.Length() = 5",
            "Length = 5: Boroughs with exactly 5 character names"
        ),

        // Complex query with multiple methods
        (
            "SELECT * FROM crime_data WHERE Borough.StartsWith('B') AND \"Major Class Description\".Contains('Criminal') AND Borough.Length() < 20",
            "Complex: Starts with B, contains Criminal, length < 20"
        ),
    ];

    for (query, description) in queries {
        println!("=== {} ===", description);
        println!("Query: {}", query);

        match csv_client.query_csv(query) {
            Ok(result) => {
                println!("Results: {} rows", result.count);

                // Show first few unique values
                if result.count > 0 {
                    let mut unique_boroughs = std::collections::HashSet::new();
                    let mut unique_descriptions = std::collections::HashSet::new();

                    for row in result.data.iter().take(100) {
                        if let Some(obj) = row.as_object() {
                            if let Some(borough) = obj.get("Borough").and_then(|v| v.as_str()) {
                                unique_boroughs.insert(borough);
                            }
                            if let Some(desc) =
                                obj.get("Major Class Description").and_then(|v| v.as_str())
                            {
                                unique_descriptions.insert(desc);
                            }
                        }
                    }

                    println!(
                        "Sample Boroughs: {:?}",
                        unique_boroughs.iter().take(5).collect::<Vec<_>>()
                    );
                    println!(
                        "Sample Crime Types: {:?}",
                        unique_descriptions.iter().take(3).collect::<Vec<_>>()
                    );
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        println!();
    }

    Ok(())
}
