use sql_cli::csv_datasource::CsvApiClient;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing column auto-sizing with sample data...");
    
    // Create test data that demonstrates the column width issue
    let test_data = json!([
        {
            "id": 1,                           // Very short
            "platformOrderId": "ORDER-2024-001",  // Long header, medium content
            "quantity": 1000,                   // Medium
            "status": "Completed",             // Medium
            "counterparty": "Bank of America", // Long content
            "commission": 75.25                // Medium
        },
        {
            "id": 22,                          // Still short
            "platformOrderId": "ORDER-2024-002",
            "quantity": 500,                   // Short content
            "status": "Pending",               // Short content
            "counterparty": "JP Morgan",       // Medium content
            "commission": 100.0
        },
        {
            "id": 333,                         // Medium
            "platformOrderId": "ORDER-2024-003",
            "quantity": 750,
            "status": "Completed",
            "counterparty": "Mizuho Bank",
            "commission": 87.5
        }
    ]);
    
    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file, "{}", test_data.to_string())?;
    
    let mut client = CsvApiClient::new();
    client.load_json(temp_file.path(), "test")?;
    
    let result = client.query_csv("SELECT * FROM test")?;
    
    println!("\nData loaded successfully!");
    println!("Rows: {}", result.data.len());
    
    // Analyze the data to show what the optimal column widths should be
    println!("\nColumn width analysis:");
    if let Some(first_row) = result.data.first() {
        if let Some(obj) = first_row.as_object() {
            for (header, _) in obj {
                let header_len = header.len();
                let mut max_content_len = 0;
                
                for row in &result.data {
                    if let Some(obj) = row.as_object() {
                        if let Some(value) = obj.get(header) {
                            let content_len = match value {
                                serde_json::Value::String(s) => s.len(),
                                serde_json::Value::Number(n) => n.to_string().len(),
                                serde_json::Value::Bool(b) => b.to_string().len(),
                                serde_json::Value::Null => 4,
                                _ => value.to_string().len(),
                            };
                            max_content_len = max_content_len.max(content_len);
                        }
                    }
                }
                
                let optimal_width = (header_len.max(max_content_len) + 2).max(4).min(50);
                println!("  {}: header={}, max_content={}, optimal={}", 
                    header, header_len, max_content_len, optimal_width);
            }
        }
    }
    
    println!("\n✅ This shows how much space we can save with intelligent column sizing!");
    println!("   - Short columns like 'id' should be ~5 chars instead of 15");
    println!("   - Long columns like 'counterparty' can be properly sized to ~17 chars");
    println!("   - This allows more columns to fit on screen simultaneously");
    
    Ok(())
}