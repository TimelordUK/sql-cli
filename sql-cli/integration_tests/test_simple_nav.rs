// Simple test to verify navigation works
#[cfg(test)]
mod tests {
    use crate::ui::enhanced_tui::*;
    use crate::buffer::Buffer;
    
    #[test]
    fn test_navigation_basics() {
        // Create a simple app with data
        let mut app = EnhancedTuiApp::new("http://test", None).unwrap();
        
        // Load some test data
        let mut buffer = Buffer::new(1);
        buffer.set_mode(AppMode::Results);
        
        // Check we can get row count
        let count = app.get_row_count();
        println!("Row count: {}", count);
        
        // Try to navigate
        app.next_row();
        println!("Navigated to next row");
        
        app.previous_row();
        println!("Navigated to previous row");
    }
}