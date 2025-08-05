use sql_cli::cursor_aware_parser::CursorAwareParser;
use sql_cli::parser::{ParseState, Schema, SqlParser};

fn main() {
    println!("Testing ORDER BY tab completion with explicit column selection...");

    // Create a test schema
    let mut schema = Schema::new();
    schema.set_single_table(
        "trade_deal".to_string(),
        vec![
            "id".to_string(),
            "price".to_string(),
            "counterparty".to_string(),
            "quantity".to_string(),
            "status".to_string(),
            "tradeDate".to_string(),
        ],
    );

    // Test with parser.rs (old parser)
    println!("\n=== Testing parser.rs (SqlParser) ===");
    test_sql_parser(schema.clone());

    // Test with cursor_aware_parser.rs
    println!("\n=== Testing cursor_aware_parser.rs (CursorAwareParser) ===");
    test_cursor_aware_parser(schema);
}

fn test_sql_parser(schema: Schema) {
    let mut parser = SqlParser::new();

    // Test 1: SELECT * - should suggest all columns
    println!("\n1. Testing 'SELECT * FROM trade_deal ORDER BY '");
    let context = parser.get_completion_context("SELECT * FROM trade_deal ORDER BY ");
    let suggestions = context.get_suggestions(&schema);
    println!("   Selected columns: {:?}", context.selected_columns);
    println!("   Suggestions: {:?}", suggestions);

    // Test 2: SELECT specific columns - should only suggest those columns
    println!("\n2. Testing 'SELECT id, price FROM trade_deal ORDER BY '");
    let context = parser.get_completion_context("SELECT id, price FROM trade_deal ORDER BY ");
    let suggestions = context.get_suggestions(&schema);
    println!("   Selected columns: {:?}", context.selected_columns);
    println!("   Suggestions: {:?}", suggestions);

    // Test 3: SELECT specific columns with WHERE - should only suggest selected columns
    println!("\n3. Testing 'SELECT counterparty, quantity FROM trade_deal WHERE status = \"active\" ORDER BY '");
    let context = parser.get_completion_context(
        "SELECT counterparty, quantity FROM trade_deal WHERE status = \"active\" ORDER BY ",
    );
    let suggestions = context.get_suggestions(&schema);
    println!("   Selected columns: {:?}", context.selected_columns);
    println!("   Suggestions: {:?}", suggestions);
}

fn test_cursor_aware_parser(schema: Schema) {
    let mut parser = CursorAwareParser::new();
    parser.set_schema(schema);

    // Test 1: SELECT * - should suggest all columns
    println!("\n1. Testing 'SELECT * FROM trade_deal ORDER BY '");
    let query = "SELECT * FROM trade_deal ORDER BY ";
    let result = parser.get_completions(query, query.len());
    println!("   Context: {}", result.context);
    println!("   Suggestions: {:?}", result.suggestions);

    // Test 2: SELECT specific columns - should only suggest those columns
    println!("\n2. Testing 'SELECT id, price FROM trade_deal ORDER BY '");
    let query = "SELECT id, price FROM trade_deal ORDER BY ";
    let result = parser.get_completions(query, query.len());
    println!("   Context: {}", result.context);
    println!("   Suggestions: {:?}", result.suggestions);

    // Test 3: SELECT specific columns with WHERE - should only suggest selected columns
    println!("\n3. Testing 'SELECT counterparty, quantity FROM trade_deal WHERE status = \"active\" ORDER BY '");
    let query = "SELECT counterparty, quantity FROM trade_deal WHERE status = \"active\" ORDER BY ";
    let result = parser.get_completions(query, query.len());
    println!("   Context: {}", result.context);
    println!("   Suggestions: {:?}", result.suggestions);
}
