use serde_json::json;
use sql_cli::cache::QueryCache;

fn main() -> anyhow::Result<()> {
    // Create a cache instance
    let mut cache =
        QueryCache::new().map_err(|e| anyhow::anyhow!("Failed to create cache: {}", e))?;

    // Create some sample trade data
    let sample_data = vec![
        json!({
            "dealId": "DEAL001",
            "tradeId": "TRD001",
            "instrumentName": "AAPL",
            "counterparty": "Bank of America",
            "quantity": 100,
            "price": 150.50,
            "tradeDate": "2024-01-15",
            "currency": "USD"
        }),
        json!({
            "dealId": "DEAL002",
            "tradeId": "TRD002",
            "instrumentName": "GOOGL",
            "counterparty": "JP Morgan",
            "quantity": 50,
            "price": 2800.00,
            "tradeDate": "2024-01-16",
            "currency": "USD"
        }),
        json!({
            "dealId": "DEAL003",
            "tradeId": "TRD003",
            "instrumentName": "MSFT",
            "counterparty": "Goldman Sachs",
            "quantity": 75,
            "price": 380.25,
            "tradeDate": "2024-01-17",
            "currency": "USD"
        }),
    ];

    // Save query to cache
    let query1 = "SELECT * FROM trade_deal WHERE tradeDate > '2024-01-01'";
    let cache_id1 = cache
        .save_query(
            query1,
            &sample_data,
            Some("January 2024 trades".to_string()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to save query: {}", e))?;
    println!("✓ Saved query to cache with ID: {}", cache_id1);

    // Save another query
    let bank_data = vec![sample_data[0].clone()];
    let query2 = "SELECT * FROM trade_deal WHERE counterparty.Contains('Bank')";
    let cache_id2 = cache
        .save_query(query2, &bank_data, Some("Bank trades".to_string()))
        .map_err(|e| anyhow::anyhow!("Failed to save bank query: {}", e))?;
    println!("✓ Saved bank query to cache with ID: {}", cache_id2);

    // List cached queries
    println!("\n📁 Cached Queries:");
    for query in cache.list_cached_queries() {
        println!(
            "  [{}] {} - {} rows - {}",
            query.id,
            query.description.as_ref().unwrap_or(&query.query_text),
            query.row_count,
            query.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    // Get cache stats
    let stats = cache.get_cache_stats();
    println!("\n📊 Cache Statistics:");
    println!("  Total queries: {}", stats.total_queries);
    println!("  Total rows: {}", stats.total_rows);
    println!("  Cache size: {}", stats.format_size());

    // Load a cached query
    println!("\n🔄 Loading cache ID {}...", cache_id1);
    let (loaded_query, loaded_data) = cache
        .load_query(cache_id1)
        .map_err(|e| anyhow::anyhow!("Failed to load query: {}", e))?;
    println!("  Query: {}", loaded_query);
    println!("  Rows: {}", loaded_data.len());

    // Now demonstrate using CSV client with cached data
    use sql_cli::csv_datasource::CsvApiClient;
    let mut csv_client = CsvApiClient::new();
    csv_client.load_from_json(loaded_data, "cached_data")?;

    // Query the cached data with LINQ methods
    println!("\n🔍 Querying cached data with LINQ methods:");

    let result =
        csv_client.query_csv("SELECT * FROM cached_data WHERE instrumentName.StartsWith('A')")?;
    println!("  instrumentName.StartsWith('A'): {} rows", result.count);

    let result =
        csv_client.query_csv("SELECT * FROM cached_data WHERE counterparty.Length() > 10")?;
    println!("  counterparty.Length() > 10: {} rows", result.count);

    let result = csv_client.query_csv("SELECT * FROM cached_data WHERE price > 200")?;
    println!("  price > 200: {} rows", result.count);

    println!("\n✅ Cache test complete!");
    println!("\nUsage in TUI:");
    println!("  :cache save          - Save current query results");
    println!("  :cache list          - List all cached queries (or press F7)");
    println!("  :cache load <id>     - Load cached data by ID");
    println!("  :cache clear         - Exit cache mode");

    Ok(())
}
