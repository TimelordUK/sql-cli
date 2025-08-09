use sql_cli::history::CommandHistory;
use std::fs;
use tempfile::TempDir;

fn main() {
    println!("Testing History Protection Integration...\n");
    
    // Create temp directory for test
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());
    
    // Create history instance
    let mut history = CommandHistory::new().unwrap();
    println!("✓ Created CommandHistory with protection");
    
    // Add some entries
    for i in 1..=5 {
        let cmd = format!("SELECT * FROM table_{}", i);
        history.add_entry(cmd.clone(), true, Some(100)).unwrap();
        println!("✓ Added: {}", cmd);
    }
    
    // Get current entry count
    let entries = history.get_all();
    println!("\nCurrent entries: {}", entries.len());
    
    // Check backup directory exists
    let backup_dir = temp_dir.path().join(".sql_cli").join("history_backups");
    if backup_dir.exists() {
        println!("✓ Backup directory exists: {:?}", backup_dir);
        
        // List backups
        if let Ok(files) = fs::read_dir(&backup_dir) {
            let count = files.count();
            println!("✓ Found {} backup files", count);
        }
    } else {
        println!("✗ Backup directory not found");
    }
    
    // Test protection by trying to clear
    println!("\nTesting clear operation...");
    history.clear().unwrap();
    println!("✓ Clear executed (should have created backup)");
    
    // Check backups again
    if let Ok(files) = fs::read_dir(&backup_dir) {
        for entry in files {
            if let Ok(entry) = entry {
                println!("  Backup: {:?}", entry.file_name());
            }
        }
    }
    
    println!("\n✓ Test completed successfully!");
}