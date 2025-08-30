use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::ui::viewport_manager::ViewportManager;
use std::sync::Arc;

/// Helper function to create a test DataView with specified dimensions
fn create_test_dataview(rows: usize, cols: usize) -> DataView {
    let mut table = DataTable::new("test_table".to_string());

    // Add columns
    for i in 0..cols {
        let col_name = format!("col_{}", i);
        table.add_column(DataColumn::new(col_name));
    }

    // Add rows
    for row in 0..rows {
        let mut row_data = vec![];
        for col in 0..cols {
            row_data.push(DataValue::String(format!("r{}c{}", row, col)));
        }
        table.add_row(DataRow::new(row_data)).unwrap();
    }

    DataView::new(Arc::new(table))
}

/// Helper function to create a test DataView with varied column widths
fn create_test_dataview_with_varied_widths() -> DataView {
    let mut table = DataTable::new("test_table".to_string());

    // Add columns with different content widths
    table.add_column(DataColumn::new("id")); // Short
    table.add_column(DataColumn::new("long_column_name_for_testing")); // Long header
    table.add_column(DataColumn::new("medium")); // Medium
    table.add_column(DataColumn::new("x")); // Very short

    // Add rows with varied content
    table
        .add_row(DataRow::new(vec![
            DataValue::String("1".to_string()),
            DataValue::String("This is a very long content that should be truncated".to_string()),
            DataValue::String("Medium text".to_string()),
            DataValue::String("y".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("999".to_string()),
            DataValue::String("Short".to_string()),
            DataValue::String("Another medium".to_string()),
            DataValue::String("z".to_string()),
        ]))
        .unwrap();

    for i in 2..20 {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(i.to_string()),
                DataValue::String(format!("Content {}", i)),
                DataValue::String(format!("Med {}", i)),
                DataValue::String("a".to_string()),
            ]))
            .unwrap();
    }

    DataView::new(Arc::new(table))
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
#[ignore = "Pinned column viewport ordering not yet fully implemented"]
fn test_horizontal_scrolling_with_pinned_columns() {
    let dataview = create_test_dataview(100, 20);
    let mut viewport = ViewportManager::new(Arc::new(dataview.clone()));

    // Pin first 2 columns
    viewport.pin_column(0);
    viewport.pin_column(1);

    // Move crosshair to column 10
    viewport.set_crosshair_column(10);

    // Pinned columns should always be visible
    let visible_cols = viewport.get_visible_columns();

    // First visible columns should be the pinned ones
    assert!(
        visible_cols.iter().any(|c| c == "col_0"),
        "Column col_0 should be pinned and visible"
    );
    assert!(
        visible_cols.iter().any(|c| c == "col_1"),
        "Column col_1 should be pinned and visible"
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

    // Hide columns 2, 3, 4 (in reverse order to avoid index shifting issues)
    viewport.hide_column(4);
    viewport.hide_column(3);
    viewport.hide_column(2);

    // Get visible columns
    let visible_cols = viewport.get_visible_columns();

    // Hidden columns should not be in visible list
    assert!(
        !visible_cols.contains(&"col_2".to_string()),
        "Column col_2 should be hidden"
    );
    assert!(
        !visible_cols.contains(&"col_3".to_string()),
        "Column col_3 should be hidden"
    );
    assert!(
        !visible_cols.contains(&"col_4".to_string()),
        "Column col_4 should be hidden"
    );

    // Other columns should be visible
    assert!(
        visible_cols.contains(&"col_0".to_string()),
        "Column col_0 should be visible"
    );
    assert!(
        visible_cols.contains(&"col_1".to_string()),
        "Column col_1 should be visible"
    );
    assert!(
        visible_cols.contains(&"col_5".to_string()),
        "Column col_5 should be visible"
    );
}

#[test]
fn test_column_width_calculations() {
    let dataview = create_test_dataview_with_varied_widths();
    let mut viewport = ViewportManager::new(Arc::new(dataview.clone()));

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
    viewport.page_down();
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert!(v_scroll > 0, "Should scroll down");

    // Page down again
    let prev_scroll = v_scroll;
    viewport.page_down();
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert!(v_scroll > prev_scroll, "Should scroll down more");

    // Page up
    let prev_scroll = v_scroll;
    viewport.page_up();
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert!(v_scroll < prev_scroll, "Should scroll up");

    // Page up to top
    viewport.page_up();
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert!(v_scroll <= prev_scroll, "Should continue scrolling up");
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
    viewport.page_up();
    let (v_scroll, _) = viewport.get_scroll_offset();
    assert_eq!(v_scroll, 0, "Should not scroll below 0");
}

#[test]
#[ignore = "Pinned column viewport ordering not yet fully implemented"]
fn test_visible_columns_with_mixed_state() {
    let dataview = create_test_dataview(100, 15);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Setup: Pin columns 0 and 1, hide columns 3 and 4 (in reverse order)
    viewport.pin_column(0);
    viewport.pin_column(1);
    viewport.hide_column(4);
    viewport.hide_column(3);

    // Scroll horizontally
    viewport.set_crosshair_column(10);

    let visible_cols = viewport.get_visible_columns();

    // Pinned columns should always be first
    assert_eq!(
        visible_cols[0], "col_0",
        "First visible should be pinned column col_0"
    );
    assert_eq!(
        visible_cols[1], "col_1",
        "Second visible should be pinned column col_1"
    );

    // Hidden columns should never appear
    assert!(
        !visible_cols.contains(&"col_3".to_string()),
        "Hidden column col_3 should not be visible"
    );
    assert!(
        !visible_cols.contains(&"col_4".to_string()),
        "Hidden column col_4 should not be visible"
    );

    // Column 10 (crosshair) should be visible
    assert!(
        visible_cols.contains(&"col_10".to_string()),
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

    let visible_cols = viewport.get_visible_columns();

    // The reordered column should be visible at the start
    assert!(
        visible_cols.contains(&"col_5".to_string()),
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

    let visible_cols = viewport.get_visible_columns();
    assert!(
        visible_cols.contains(&"col_30".to_string()),
        "Column col_30 should be visible after ensure_visible"
    );

    // Ensure column 0 is visible (scroll back)
    viewport.ensure_column_visible(0, terminal_width);

    let visible_cols = viewport.get_visible_columns();
    assert!(
        visible_cols.contains(&"col_0".to_string()),
        "Column col_0 should be visible after ensure_visible"
    );
}

#[test]
#[ignore = "Pinned column viewport ordering not yet fully implemented"]
fn test_complex_navigation_scenario() {
    let dataview = create_test_dataview(500, 30);
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Simulate complex user navigation
    // 1. Pin first two columns
    viewport.pin_column(0);
    viewport.pin_column(1);

    // 2. Hide some columns (in reverse order to avoid index shifting)
    viewport.hide_column(7);
    viewport.hide_column(6);
    viewport.hide_column(5);

    // 3. Navigate to middle of data
    viewport.set_crosshair_row(250);
    viewport.set_crosshair_column(15);

    // 4. Page down several times
    for _ in 0..3 {
        viewport.page_down();
    }

    // 5. Move crosshair far right
    viewport.set_crosshair_column(25);

    // Verify state is consistent
    let (row, col) = viewport.get_crosshair_position();
    assert_eq!(col, 25, "Crosshair column should be at 25");
    assert!(row >= 250, "Crosshair row should have moved down from 250");

    let visible_cols = viewport.get_visible_columns();
    assert!(
        visible_cols.contains(&"col_0".to_string()),
        "Pinned column col_0 should still be visible"
    );
    assert!(
        visible_cols.contains(&"col_1".to_string()),
        "Pinned column col_1 should still be visible"
    );
    assert!(
        !visible_cols.contains(&"col_5".to_string()),
        "Hidden column col_5 should not be visible"
    );
    assert!(
        !visible_cols.contains(&"col_6".to_string()),
        "Hidden column col_6 should not be visible"
    );
    assert!(
        !visible_cols.contains(&"col_7".to_string()),
        "Hidden column col_7 should not be visible"
    );
}
