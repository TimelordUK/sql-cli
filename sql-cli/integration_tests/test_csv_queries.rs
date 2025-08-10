use sql_cli::csv_datasource::CsvApiClient;

fn main() -> anyhow::Result<()> {
    let mut csv_client = CsvApiClient::new();

    // Load the CSV file
    csv_client.load_csv(
        "/home/me/dev/sql-cli/BusinessCrimeBoroughLevel.csv",
        "crime_data",
    )?;

    println!("CSV file loaded successfully!\n");

    // Example queries with column names that have spaces
    let queries = vec![
        // Select all data
        ("SELECT * FROM crime_data", "All data"),

        // Filter by Borough (no spaces in column name)
        ("SELECT * FROM crime_data WHERE Borough = 'Bronx'", "Filter by Bronx"),

        // Filter using column with spaces - quoted
        ("SELECT * FROM crime_data WHERE \"Major Class Description\" = 'Burglary'", "Burglary crimes"),

        // Using Contains with quoted column names
        ("SELECT * FROM crime_data WHERE \"Major Class Description\".Contains('Criminal')", "Criminal related"),

        // Select specific columns with spaces
        ("SELECT Borough, \"Major Class Description\", \"Minor Class Description\" FROM crime_data", "Specific columns"),

        // Complex query with multiple conditions
        ("SELECT * FROM crime_data WHERE Borough.Contains('Aviation') AND \"Major Class Description\".Contains('Damage')", "Aviation damage crimes"),
    ];

    for (query, description) in queries {
        println!("Query: {}", query);
        println!("Description: {}", description);

        match csv_client.query_csv(query) {
            Ok(result) => {
                println!("Results: {} rows", result.count);

                // Show first row as example
                if let Some(first_row) = result.data.first() {
                    if let Some(obj) = first_row.as_object() {
                        println!("First row sample:");
                        for (key, value) in obj.iter().take(5) {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        println!("\n---\n");
    }

    Ok(())
}
