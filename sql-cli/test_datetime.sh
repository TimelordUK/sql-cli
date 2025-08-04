#!/bin/bash

# Test DateTime completion
echo "Testing DateTime completion after date column comparison..."

# Create a test program to check the parser
cat > test_datetime.rs << 'EOF'
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
EOF

# Compile and run the test
rustc --edition 2021 -L target/debug/deps test_datetime.rs \
    --extern sql_cli=target/debug/libsql_cli.rlib \
    -o test_datetime

if [ -f test_datetime ]; then
    ./test_datetime
    rm test_datetime test_datetime.rs
else
    echo "Failed to compile test"
fi