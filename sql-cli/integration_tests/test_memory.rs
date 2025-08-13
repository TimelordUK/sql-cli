use serde_json::json;

fn main() {
    // Test memory usage of serde_json::Value
    let json_val = json!({
        "id": 12345,
        "symbol": "AAPL",
        "price": 150.25,
        "quantity": 100,
        "timestamp": "2024-01-15T10:30:00Z",
        "side": "BUY",
        "exchange": "NASDAQ"
    });
    
    println!("Size of serde_json::Value: {} bytes", std::mem::size_of_val(&json_val));
    
    // String version
    let str_vec = vec![
        "12345".to_string(),
        "AAPL".to_string(), 
        "150.25".to_string(),
        "100".to_string(),
        "2024-01-15T10:30:00Z".to_string(),
        "BUY".to_string(),
        "NASDAQ".to_string()
    ];
    
    println!("Size of Vec<String>: {} bytes", std::mem::size_of_val(&str_vec));
    
    // Actual string content size
    let json_str = serde_json::to_string(&json_val).unwrap();
    println!("JSON string length: {} bytes", json_str.len());
    
    let total_str_len: usize = str_vec.iter().map(|s| s.len()).sum();
    println!("Total string content: {} bytes", total_str_len);
}
