use sql_cli::csv_datasource::CsvApiClient;

fn main() -> anyhow::Result<()> {
    let mut csv_client = CsvApiClient::new();

    // Load the CSV file
    csv_client.load_csv(
        "/home/me/dev/sql-cli/BusinessCrimeBoroughLevel.csv",
        "crime_data",
    )?;

    // Get schema
    if let Some(schema) = csv_client.get_schema() {
        println!("Schema loaded:");
        for (table, columns) in schema {
            println!("Table: {}", table);
            println!("Columns: {:?}", columns);
        }
    }

    // Test a simple query
    let result = csv_client.query_csv("SELECT * FROM crime_data")?;
    println!("\nQuery returned {} rows", result.count);

    // Show first few rows
    for (i, row) in result.data.iter().take(5).enumerate() {
        println!("\nRow {}:", i + 1);
        if let Some(obj) = row.as_object() {
            for (key, value) in obj {
                println!("  {}: {}", key, value);
            }
        }
    }

    // Test a filtered query
    let filtered = csv_client.query_csv("SELECT * FROM crime_data WHERE Borough = 'Bronx'")?;
    println!(
        "\n\nFiltered query (Borough = 'Bronx') returned {} rows",
        filtered.count
    );

    // Test a Contains query
    let contains = csv_client.query_csv("SELECT * FROM crime_data WHERE Borough.Contains('Br')")?;
    println!(
        "\nContains query (Borough contains 'Br') returned {} rows",
        contains.count
    );

    Ok(())
}
