// Test TableWidgetManager functionality
use sql_cli::ui::table_widget_manager::TableWidgetManager;
use sql_cli::data::data_view::DataView;
use std::sync::Arc;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("sql_cli::ui::table_widget_manager=debug")
        .init();

    println!("Testing TableWidgetManager...\n");

    // Create a manager
    let mut manager = TableWidgetManager::new();
    
    // Create a simple DataView
    let dataview = DataView::new_test(vec![
        vec!["id", "book", "product"],
        vec!["1", "Fixed Income", "Corporate"],
        vec!["2", "Commodities", "Energy"],
        vec!["3", "Equities", "Tech"],
        vec!["4", "Forex", "EUR/USD"],
        vec!["5", "Derivatives", "Options"],
        vec!["6", "Fixed Income", "emerging"],
    ]);
    
    manager.set_dataview(Arc::new(dataview));
    
    // Test navigation
    println!("Initial needs_render: {}", manager.needs_render());
    
    // Navigate to search match
    manager.navigate_to_search_match(6, 2);
    println!("After search navigation needs_render: {}", manager.needs_render());
    
    // Mark as rendered
    manager.rendered();
    println!("After rendered() needs_render: {}", manager.needs_render());
    
    // Navigate with debounced search
    manager.on_debounced_search(3, 1);
    println!("After debounced search needs_render: {}", manager.needs_render());
    
    println!("\nTest complete!");
}
