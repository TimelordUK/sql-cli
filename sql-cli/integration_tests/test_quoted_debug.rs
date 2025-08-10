use sql_cli::recursive_parser::detect_cursor_context;

fn main() {
    let query = "SELECT * FROM table WHERE \"Last Name\".";
    let cursor_pos = query.len();

    println!("Full query: '{}'", query);
    println!("Cursor pos: {}", cursor_pos);

    // Check what the trimmed query looks like
    let trimmed = query.trim();
    println!("Trimmed query: '{}'", trimmed);

    // Look for dots
    if let Some(dot_pos) = trimmed.rfind('.') {
        println!("Found dot at position: {}", dot_pos);
        let before_dot = &trimmed[..dot_pos];
        let after_dot = &trimmed[dot_pos + 1..];
        println!("Before dot: '{}'", before_dot);
        println!("After dot: '{}'", after_dot);

        // Check if ends with quote
        println!("Before dot ends with quote? {}", before_dot.ends_with('"'));

        // Try to extract the column
        if let Some(last_word) = before_dot.split_whitespace().last() {
            println!("Last word before dot: '{}'", last_word);
        }
    }

    let (context, partial) = detect_cursor_context(query, cursor_pos);
    println!("\nDetected context: {:?}", context);
    println!("Partial: {:?}", partial);
}
