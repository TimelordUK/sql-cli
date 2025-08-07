use sql_cli::recursive_parser::{Lexer, Parser, Token};

fn main() {
    println!("Testing LIMIT parsing");
    println!("{}", "=".repeat(50));

    let test_queries = vec![
        "SELECT * FROM trades_10k LIMIT 100",
        "SELECT * FROM trades_10k limit 100",
        "select * from trades_10k limit 100",
    ];

    for query in test_queries {
        println!("\nQuery: '{}'", query);

        // Test lexer
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all();

        println!("Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  [{}] {:?}", i, token);
        }

        // Test parser
        let mut parser = Parser::new(query);
        match parser.parse() {
            Ok(stmt) => {
                println!("Parsed successfully!");
                println!("  columns: {:?}", stmt.columns);
                println!("  from_table: {:?}", stmt.from_table);
                println!("  limit: {:?}", stmt.limit);
                println!("  offset: {:?}", stmt.offset);
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

        println!("{}", "-".repeat(40));
    }
}
