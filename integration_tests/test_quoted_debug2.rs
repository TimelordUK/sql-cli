fn main() {
    let before_dot = "SELECT * FROM table WHERE \"Last Name\"";
    
    println!("Testing: '{}'", before_dot);
    println!("Ends with quote? {}", before_dot.ends_with('"'));
    
    let bytes = before_dot.as_bytes();
    let mut pos = before_dot.len() - 1; // Position of closing quote
    let mut found_start = None;
    
    println!("Starting at position {} (char: '{}')", pos, before_dot.chars().nth(pos).unwrap());
    
    // Skip the closing quote and search backwards
    if pos > 0 {
        pos -= 1;
        println!("Searching backwards from position {}", pos);
        
        while pos > 0 {
            let ch = bytes[pos];
            println!("  Position {}: '{}' (byte: {})", pos, ch as char, ch);
            
            if bytes[pos] == b'"' {
                // Check if it's not an escaped quote
                if pos == 0 || bytes[pos - 1] != b'\\' {
                    found_start = Some(pos);
                    println!("  Found opening quote at position {}", pos);
                    break;
                }
            }
            pos -= 1;
        }
        
        // Check position 0 separately
        if found_start.is_none() && bytes[0] == b'"' {
            found_start = Some(0);
            println!("Found opening quote at position 0");
        }
    }
    
    if let Some(start) = found_start {
        let extracted = &before_dot[start..];
        println!("\nExtracted: '{}'", extracted);
    } else {
        println!("\nNo opening quote found!");
    }
}