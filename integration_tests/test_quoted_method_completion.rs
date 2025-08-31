use sql_cli::recursive_parser::{detect_cursor_context, CursorContext};

fn main() {
    // Test cases for quoted identifier method completion
    let test_cases = vec![
        // Regular identifier with method - use actual string length
        (
            "SELECT * FROM table WHERE LastName.",
            "SELECT * FROM table WHERE LastName.".len(),
        ),
        (
            "SELECT * FROM table WHERE LastName.Con",
            "SELECT * FROM table WHERE LastName.Con".len(),
        ),
        // Quoted identifier with method - the key test cases
        (
            "SELECT * FROM table WHERE \"Last Name\".",
            "SELECT * FROM table WHERE \"Last Name\".".len(),
        ),
        (
            "SELECT * FROM table WHERE \"Last Name\".Con",
            "SELECT * FROM table WHERE \"Last Name\".Con".len(),
        ),
        (
            "SELECT * FROM table WHERE \"Customer Id\".Conta",
            "SELECT * FROM table WHERE \"Customer Id\".Conta".len(),
        ),
    ];

    for (query, cursor_pos) in test_cases {
        let (context, partial) = detect_cursor_context(query, cursor_pos);

        println!("Query: {}", query);
        println!("  Cursor at: {}", cursor_pos);
        println!("  Context: {:?}", context);
        println!("  Partial: {:?}", partial);

        // Check if we properly detect method call context for quoted identifiers
        match &context {
            CursorContext::AfterColumn(col) => {
                println!("  ✓ Detected column: {}", col);
                if query.contains("\"Last Name\"") {
                    assert_eq!(
                        col, "Last Name",
                        "Should extract 'Last Name' without quotes"
                    );
                } else if query.contains("\"Customer Id\"") {
                    assert_eq!(
                        col, "Customer Id",
                        "Should extract 'Customer Id' without quotes"
                    );
                }
            }
            _ => {
                if query.contains('.') && !query.contains("()") {
                    println!("  ✗ Failed to detect method call context!");
                    panic!("Should detect AfterColumn context for: {}", query);
                }
            }
        }
        println!();
    }

    println!("✅ All tests passed!");
}
