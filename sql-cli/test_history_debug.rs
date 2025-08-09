use sql_cli::history::CommandHistory;

fn main() {
    println!("Testing history functionality...");
    
    match CommandHistory::new() {
        Ok(mut history) => {
            println!("✓ CommandHistory created successfully");
            
            // Check existing entries
            let all_entries = history.get_all();
            println!("Found {} existing history entries", all_entries.len());
            
            // Try adding a new entry
            if let Err(e) = history.add_entry("SELECT * FROM test".to_string(), true, Some(100)) {
                println!("✗ Failed to add entry: {}", e);
            } else {
                println!("✓ Successfully added test entry");
            }
            
            // Check if it was added
            let new_count = history.get_all().len();
            println!("History now has {} entries", new_count);
            
            // Test search
            let matches = history.search("SELECT");
            println!("Found {} matches for 'SELECT'", matches.len());
            
            // Show first few entries
            for (i, entry) in history.get_all().iter().take(5).enumerate() {
                println!("  [{}] {}", i, entry.command);
            }
        }
        Err(e) => {
            println!("✗ Failed to create CommandHistory: {}", e);
        }
    }
}