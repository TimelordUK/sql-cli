// Test to compare existing state methods vs new delegation methods
use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{BufferManager, Buffer, BufferAPI};

fn main() {
    // Create a buffer manager with a buffer
    let mut buffer_manager = BufferManager::new();
    let buffer = Buffer::new(1);
    buffer_manager.add_buffer(buffer);
    
    // Create AppStateContainer
    let mut state = AppStateContainer::new(buffer_manager)
        .expect("Failed to create AppStateContainer");
    
    // Test existing methods vs what should be delegated
    println!("=== Testing State Duplication ===\n");
    
    // Set values using existing methods
    state.set_table_selected_row(Some(10));
    state.set_current_column(5);
    
    // Check if values are in sync
    println!("AppStateContainer selected_row: {:?}", state.get_selected_row());
    println!("Buffer selected_row (via current_buffer): {:?}", 
        state.current_buffer().and_then(|b| b.get_selected_row()));
    
    println!("\nAppStateContainer current_column: {}", state.get_current_column());
    println!("Buffer current_column (via current_buffer): {}", 
        state.current_buffer().map(|b| b.get_current_column()).unwrap_or(0));
    
    // Now set via buffer directly and see if AppStateContainer reflects it
    if let Some(buffer) = state.current_buffer_mut() {
        buffer.set_selected_row(Some(20));
        buffer.set_current_column(8);
    }
    
    println!("\n--- After setting via Buffer directly ---");
    println!("AppStateContainer selected_row: {:?}", state.get_selected_row());
    println!("Buffer selected_row: {:?}", 
        state.current_buffer().and_then(|b| b.get_selected_row()));
    
    println!("\nAppStateContainer current_column: {}", state.get_current_column());
    println!("Buffer current_column: {}", 
        state.current_buffer().map(|b| b.get_current_column()).unwrap_or(0));
    
    println!("\n⚠️  If values differ, we have state duplication!");
}