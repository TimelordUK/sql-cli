use sql_cli::recursive_parser::{detect_cursor_context, CursorContext};

fn main() {
    // Test cases for method completion inside parentheses
    let test_cases = vec![
        // Without parentheses - baseline
        (
            "SELECT * FROM table WHERE Country.",
            "SELECT * FROM table WHERE Country.".len(),
        ),
        (
            "SELECT * FROM table WHERE Country.Con",
            "SELECT * FROM table WHERE Country.Con".len(),
        ),
        // With parentheses - the key test cases
        (
            "SELECT * FROM table WHERE (Country.",
            "SELECT * FROM table WHERE (Country.".len(),
        ),
        (
            "SELECT * FROM table WHERE (Country.Con",
            "SELECT * FROM table WHERE (Country.Con".len(),
        ),
        // Nested parentheses
        (
            "SELECT * FROM table WHERE ((Country.",
            "SELECT * FROM table WHERE ((Country.".len(),
        ),
        // Mixed with AND/OR
        (
            "SELECT * FROM table WHERE (Country.Con",
            "SELECT * FROM table WHERE (Country.Con".len(),
        ),
        (
            "SELECT * FROM table WHERE Age > 10 AND (Country.",
            "SELECT * FROM table WHERE Age > 10 AND (Country.".len(),
        ),
    ];

    for (query, cursor_pos) in test_cases {
        let (context, partial) = detect_cursor_context(query, cursor_pos);

        println!("Query: {}", query);
        println!("  Cursor at: {}", cursor_pos);
        println!("  Context: {:?}", context);
        println!("  Partial: {:?}", partial);

        // Check if we properly detect method call context even with parentheses
        match &context {
            CursorContext::AfterColumn(col) => {
                println!("  ✓ Detected column: {}", col);
                assert_eq!(
                    col, "Country",
                    "Should extract 'Country' without parentheses"
                );
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
