use sql_cli::sql::recursive_parser::Parser;

fn main() {
    let queries = vec![
        "SELECT * FROM trades WHERE trader.Contains('John')",
        "SELECT * FROM trades WHERE book.StartsWith('EQUITY')",
        "SELECT * FROM trades WHERE name.EndsWith('Option')",
        "SELECT * FROM trades WHERE trader.Contains('John') AND currency = 'USD'",
        "SELECT * FROM trades WHERE name.ToLower() == 'test'",
        "SELECT * FROM trades WHERE String.IsNullOrEmpty(name)",
    ];

    for sql in queries {
        println!("\n=== Parsing: {} ===", sql);
        let mut parser = Parser::new(sql);
        match parser.parse() {
            Ok(stmt) => {
                if let Some(where_clause) = &stmt.where_clause {
                    println!("WHERE clause parsed successfully:");
                    println!("{:#?}", where_clause);
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }
    }
}
