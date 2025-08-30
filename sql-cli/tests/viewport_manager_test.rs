use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::DataTable;
use sql_cli::ui::viewport_manager::ViewportManager;
use std::sync::Arc;

/// Helper function to create a test DataView with specified dimensions
fn create_test_dataview(rows: usize, cols: usize) -> DataView {
    let mut table = DataTable::new("test_table".to_string());

    // Add columns
    for i in 0..cols {
        let col_name = format!("col_{}", i);
        table.add_column(col_name);
    }

    // Add rows
    for row in 0..rows {
        let mut row_data = vec![];
        for col in 0..cols {
            row_data.push(Some(format!("r{}c{}", row, col)));
        }
        table.add_row(row_data);
    }

    DataView::from_source(Arc::new(table))
}

/// Helper function to create a test DataView with varied column widths
fn create_test_dataview_with_varied_widths() -> DataView {
    let mut table = DataTable::new("test_table".to_string());

    // Add columns with different content widths
    table.add_column("id".to_string()); // Short
    table.add_column("long_column_name_for_testing".to_string()); // Long header
    table.add_column("medium".to_string()); // Medium
    table.add_column("x".to_string()); // Very short

    // Add rows with varied content
    table.add_row(vec![
        Some("1".to_string()),
        Some("This is a very long content that should be truncated".to_string()),
        Some("Medium text".to_string()),
        Some("y".to_string()),
    ]);

    table.add_row(vec![
        Some("999".to_string()),
        Some("Short".to_string()),
        Some("Another medium".to_string()),
        Some("z".to_string()),
    ]);

    for i in 2..20 {
        table.add_row(vec![
            Some(i.to_string()),
            Some(format!("Content {}", i)),
            Some(format!("Med {}", i)),
            Some("a".to_string()),
        ]);
    }

    DataView::from_source(Arc::new(table))
}

#[test]
fn test_viewport_creation() {
    let dataview = create_test_dataview(100, 10);
    let viewport = ViewportManager::new(Arc::new(dataview));

    // Check initial state
    assert_eq!(viewport.get_scroll_offset(), (0, 0));
    assert_eq!(viewport.get_crosshair_position(), (0, 0));
    assert!(!viewport.is_viewport_locked());
}

#[test]
fn test_horizontal_scrolling_basic() {
    let dataview = create_test_dataview(100, 20);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Set terminal width that can't show all columns
    let terminal_width = 80;

    // Move crosshair to the right
    viewport.set_crosshair_column(5);
    assert_eq!(viewport.get_crosshair_position().1, 5);

    // Move further right - should trigger horizontal scroll
    viewport.set_crosshair_column(15);
    assert_eq!(viewport.get_crosshair_position().1, 15);

    // Check that horizontal scroll offset has changed
    let (_, h_scroll) = viewport.get_scroll_offset();
    assert!(h_scroll > 0, "Horizontal scroll should have moved");
}

#[test]
fn test_horizontal_scrolling_with_pinned_columns() {
    let dataview = create_test_dataview(100, 20);
    let mut viewport = ViewportManager::new(Arc::new(dataview.clone()));

    // Pin first 2 columns
    viewport.pin_column(0);
    viewport.pin_column(1);

    // Move crosshair to column 10
    viewport.set_crosshair_column(10);

    // Pinned columns should always be visible
    let visible_cols = viewport.get_visible_columns(120); // Terminal width

    // First visible columns should be the pinned ones
    assert!(
        visible_cols.iter().any(|c| c == &0),
        "Column 0 should be pinned and visible"
    );
    assert!(
        visible_cols.iter().any(|c| c == &1),
        "Column 1 should be pinned and visible"
    );
}

#[test]
fn test_vertical_scrolling() {
    let dataview = create_test_dataview(1000, 10);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Set viewport height
    let viewport_height = 20;

    // Initial position
    assert_eq!(viewport.get_scroll_offset().0, 0);

    // Move to row 50
    viewport.set_crosshair_row(50);
    assert_eq!(viewport.get_crosshair_position().0, 50);

    // Scroll offset should have changed to keep row 50 visible
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert!(v_scroll > 0, "Vertical scroll should have moved");
    assert!(v_scroll <= 50, "Scroll should not exceed target row");
}

#[test]
fn test_hidden_columns() {
    let dataview = create_test_dataview(100, 10);
    let mut viewport = ViewportManager::new(Arc::new(dataview.clone()));

    // Hide columns 2, 3, 4
    viewport.hide_column(2);
    viewport.hide_column(3);
    viewport.hide_column(4);

    // Get visible columns
    let visible_cols = viewport.get_visible_columns(200);

    // Hidden columns should not be in visible list
    assert!(!visible_cols.contains(&2), "Column 2 should be hidden");
    assert!(!visible_cols.contains(&3), "Column 3 should be hidden");
    assert!(!visible_cols.contains(&4), "Column 4 should be hidden");

    // Other columns should be visible
    assert!(visible_cols.contains(&0), "Column 0 should be visible");
    assert!(visible_cols.contains(&1), "Column 1 should be visible");
    assert!(visible_cols.contains(&5), "Column 5 should be visible");
}

#[test]
fn test_column_width_calculations() {
    let dataview = create_test_dataview_with_varied_widths();
    let viewport = ViewportManager::new(Arc::new(dataview.clone()));

    // Get column widths
    let widths = viewport.calculate_column_widths(200);

    // Check that all columns have reasonable widths
    for (col_idx, width) in widths.iter().enumerate() {
        assert!(*width > 0, "Column {} should have positive width", col_idx);
        assert!(*width <= 50, "Column {} width should be capped", col_idx);
    }
}

#[test]
fn test_crosshair_movement_updates_scroll() {
    let dataview = create_test_dataview(100, 30);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Set small viewport
    let terminal_width = 80;
    let viewport_height = 20;

    // Move crosshair to bottom-right
    viewport.set_crosshair_row(50);
    viewport.set_crosshair_column(25);

    let (v_scroll, h_scroll) = viewport.get_scroll_offset();
    assert!(v_scroll > 0, "Should have scrolled vertically");
    assert!(h_scroll > 0, "Should have scrolled horizontally");

    // Move back to origin
    viewport.set_crosshair_row(0);
    viewport.set_crosshair_column(0);

    let (v_scroll, h_scroll) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, 0, "Should have scrolled back to top");
    assert_eq!(h_scroll, 0, "Should have scrolled back to left");
}

#[test]
fn test_viewport_lock() {
    let dataview = create_test_dataview(100, 20);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Set initial position
    viewport.set_crosshair_row(10);
    viewport.set_crosshair_column(5);
    let initial_scroll = viewport.get_scroll_offset();

    // Lock viewport
    viewport.lock_viewport();
    assert!(viewport.is_viewport_locked());

    // Try to move crosshair - scroll should not change
    viewport.set_crosshair_row(50);
    viewport.set_crosshair_column(15);

    let locked_scroll = viewport.get_scroll_offset();
    assert_eq!(
        locked_scroll, initial_scroll,
        "Scroll should not change when locked"
    );

    // Unlock viewport
    viewport.unlock_viewport();
    assert!(!viewport.is_viewport_locked());
}

#[test]
fn test_page_up_down() {
    let dataview = create_test_dataview(1000, 10);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    let page_size = 20;

    // Page down
    viewport.page_down(page_size);
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, page_size, "Should scroll down one page");

    // Page down again
    viewport.page_down(page_size);
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, page_size * 2, "Should scroll down two pages");

    // Page up
    viewport.page_up(page_size);
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, page_size, "Should scroll up one page");

    // Page up to top
    viewport.page_up(page_size);
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, 0, "Should be back at top");
}

#[test]
fn test_boundary_conditions() {
    let dataview = create_test_dataview(10, 5);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Try to scroll beyond boundaries
    viewport.set_crosshair_row(100); // Beyond row count
    let (row, _) = viewport.get_crosshair_position();
    assert_eq!(row, 9, "Should be clamped to last row");

    viewport.set_crosshair_column(100); // Beyond column count
    let (_, col) = viewport.get_crosshair_position();
    assert_eq!(col, 4, "Should be clamped to last column");

    // Try negative scroll (should be prevented internally)
    viewport.set_crosshair_row(0);
    viewport.page_up(100);
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, 0, "Should not scroll below 0");
}

#[test]
fn test_visible_columns_with_mixed_state() {
    let dataview = create_test_dataview(100, 15);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Setup: Pin columns 0 and 1, hide columns 3 and 4
    viewport.pin_column(0);
    viewport.pin_column(1);
    viewport.hide_column(3);
    viewport.hide_column(4);

    // Scroll horizontally
    viewport.set_crosshair_column(10);

    let visible_cols = viewport.get_visible_columns(150);

    // Pinned columns should always be first
    assert_eq!(
        visible_cols[0], 0,
        "First visible should be pinned column 0"
    );
    assert_eq!(
        visible_cols[1], 1,
        "Second visible should be pinned column 1"
    );

    // Hidden columns should never appear
    assert!(
        !visible_cols.contains(&3),
        "Hidden column 3 should not be visible"
    );
    assert!(
        !visible_cols.contains(&4),
        "Hidden column 4 should not be visible"
    );

    // Column 10 (crosshair) should be visible
    assert!(
        visible_cols.contains(&10),
        "Crosshair column should be visible"
    );
}

#[test]
fn test_column_reordering_with_scroll() {
    let dataview = create_test_dataview(100, 20);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Move column 5 to position 0
    viewport.reorder_column(5, 0);

    // Set crosshair to what was originally column 5 (now at position 0)
    viewport.set_crosshair_column(0);

    let visible_cols = viewport.get_visible_columns(120);

    // The reordered column should be visible at the start
    assert!(
        visible_cols.contains(&5),
        "Reordered column should be visible"
    );
}

#[test]
fn test_ensure_column_visible() {
    let dataview = create_test_dataview(100, 50);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Terminal width that can only show ~10 columns
    let terminal_width = 100;

    // Ensure column 30 is visible
    viewport.ensure_column_visible(30, terminal_width);

    let visible_cols = viewport.get_visible_columns(terminal_width);
    assert!(
        visible_cols.contains(&30),
        "Column 30 should be visible after ensure_visible"
    );

    // Ensure column 0 is visible (scroll back)
    viewport.ensure_column_visible(0, terminal_width);

    let visible_cols = viewport.get_visible_columns(terminal_width);
    assert!(
        visible_cols.contains(&0),
        "Column 0 should be visible after ensure_visible"
    );
}

#[test]
fn test_complex_navigation_scenario() {
    let dataview = create_test_dataview(500, 30);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Simulate complex user navigation
    // 1. Pin first two columns
    viewport.pin_column(0);
    viewport.pin_column(1);

    // 2. Hide some columns
    viewport.hide_column(5);
    viewport.hide_column(6);
    viewport.hide_column(7);

    // 3. Navigate to middle of data
    viewport.set_crosshair_row(250);
    viewport.set_crosshair_column(15);

    // 4. Page down several times
    for _ in 0..3 {
        viewport.page_down(20);
    }

    // 5. Move crosshair far right
    viewport.set_crosshair_column(25);

    // Verify state is consistent
    let (row, col) = viewport.get_crosshair_position();
    assert_eq!(col, 25, "Crosshair column should be at 25");
    assert!(row >= 250, "Crosshair row should have moved down from 250");

    let visible_cols = viewport.get_visible_columns(120);
    assert!(
        visible_cols.contains(&0),
        "Pinned column 0 should still be visible"
    );
    assert!(
        visible_cols.contains(&1),
        "Pinned column 1 should still be visible"
    );
    assert!(
        !visible_cols.contains(&5),
        "Hidden column 5 should not be visible"
    );
    assert!(
        !visible_cols.contains(&6),
        "Hidden column 6 should not be visible"
    );
    assert!(
        !visible_cols.contains(&7),
        "Hidden column 7 should not be visible"
    );
}
