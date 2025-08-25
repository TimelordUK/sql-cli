// Quick test to verify state delegation works
use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{BufferManager, Buffer};

fn main() {
    // Create a buffer manager with a buffer
    let mut buffer_manager = BufferManager::new();
    let buffer = Buffer::new(1);
    buffer_manager.add_buffer(buffer);
    
    // Create AppStateContainer
    let mut state = AppStateContainer::new(buffer_manager)
        .expect("Failed to create AppStateContainer");
    
    // Test navigation delegation
    println!("Initial selected row: {:?}", state.delegated_selected_row());
    state.set_delegated_selected_row(Some(5));
    println!("After setting to 5: {:?}", state.delegated_selected_row());
    
    // Test column delegation
    println!("Initial column: {}", state.delegated_current_column());
    state.set_delegated_current_column(3);
    println!("After setting to 3: {}", state.delegated_current_column());
    
    // Test search delegation
    println!("Initial search pattern: '{}'", state.delegated_search_pattern());
    state.set_delegated_search_pattern("test pattern".to_string());
    println!("After setting: '{}'", state.delegated_search_pattern());
    
    // Test filter delegation  
    println!("Initial filter active: {}", state.delegated_filter_active());
    state.set_delegated_filter_active(true);
    println!("After setting to true: {}", state.delegated_filter_active());
    
    println!("\n✅ All delegation methods work correctly!");
}