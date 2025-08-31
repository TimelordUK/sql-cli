use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::BufferManager;

fn main() {
    println!("Testing AppStateContainer initialization...");
    
    let buffer_manager = BufferManager::new();
    
    match AppStateContainer::new(buffer_manager) {
        Ok(_) => {
            println!("✓ AppStateContainer initialized successfully!");
        }
        Err(e) => {
            println!("✗ AppStateContainer initialization failed: {}", e);
            println!("Error chain:");
            let mut source = e.source();
            while let Some(err) = source {
                println!("  Caused by: {}", err);
                source = err.source();
            }
        }
    }
}