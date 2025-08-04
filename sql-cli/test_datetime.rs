use sql_cli::cursor_aware_parser::CursorAwareParser;

fn main() {
    let parser = CursorAwareParser::new();
    
    // Test cases
    let test_cases = vec![
        ("SELECT * FROM trade_deal WHERE createdDate > ", "createdDate >"),
        ("SELECT * FROM trade_deal WHERE createdDate > Date", "createdDate > Date"),
        ("SELECT * FROM trade_deal WHERE tradeDate >= ", "tradeDate >="),
    ];
    
    for (query, desc) in test_cases {
        let result = parser.get_completions(query, query.len());
        println!("\nTest: {}", desc);
        println!("Query: '{}'", query);
        println!("Context: {}", result.context);
        println!("Suggestions: {:?}", result.suggestions);
    }
}
