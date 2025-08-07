#[cfg(test)]
mod tests {
    use crate::api_client::{QueryInfo, QueryResponse};
    use crate::buffer::{AppMode, Buffer, BufferAPI, SortOrder};
    use serde_json::json;

    #[test]
    fn test_buffer_basic_operations() {
        let mut buffer = Buffer::new(1);

        // Test mode operations
        assert_eq!(buffer.get_mode(), AppMode::Command);
        buffer.set_mode(AppMode::Results);
        assert_eq!(buffer.get_mode(), AppMode::Results);

        // Test status message
        buffer.set_status_message("Test status".to_string());
        assert_eq!(buffer.get_status_message(), "Test status");

        // Test query operations
        buffer.set_query("SELECT * FROM test".to_string());
        assert_eq!(buffer.get_query(), "SELECT * FROM test");
        assert_eq!(buffer.get_input_cursor(), 18); // Should be at end

        // Test column operations
        buffer.set_current_column(5);
        assert_eq!(buffer.get_current_column(), 5);
    }

    #[test]
    fn test_buffer_filtering() {
        let mut buffer = Buffer::new(1);

        // Set up filter
        buffer.set_filter_pattern("test pattern".to_string());
        assert_eq!(buffer.get_filter_pattern(), "test pattern");

        buffer.set_filter_active(true);
        assert!(buffer.is_filter_active());

        // Clear filters
        buffer.clear_filters();
        assert!(!buffer.is_filter_active());
        assert_eq!(buffer.get_filter_pattern(), "");
    }

    #[test]
    fn test_buffer_pinned_columns() {
        let mut buffer = Buffer::new(1);

        // Add pinned columns
        buffer.add_pinned_column(2);
        buffer.add_pinned_column(5);
        buffer.add_pinned_column(1);

        // Should be sorted
        assert_eq!(buffer.get_pinned_columns(), &vec![1, 2, 5]);

        // Remove a column
        buffer.remove_pinned_column(2);
        assert_eq!(buffer.get_pinned_columns(), &vec![1, 5]);

        // Clear all
        buffer.clear_pinned_columns();
        assert_eq!(buffer.get_pinned_columns(), &Vec::<usize>::new());
    }

    #[test]
    fn test_buffer_results() {
        let mut buffer = Buffer::new(1);

        // Create mock results
        let results = QueryResponse {
            data: vec![
                json!({"id": 1, "name": "Alice"}),
                json!({"id": 2, "name": "Bob"}),
            ],
            count: 2,
            query: QueryInfo {
                select: vec!["id".to_string(), "name".to_string()],
                where_clause: None,
                order_by: None,
            },
            source: None,
            table: None,
            cached: None,
        };

        buffer.set_results(Some(results));
        assert!(buffer.get_results().is_some());
        assert_eq!(buffer.get_row_count(), 2);
        assert_eq!(buffer.get_column_count(), 2);

        // Test with no results
        buffer.set_results(None);
        assert!(buffer.get_results().is_none());
        assert_eq!(buffer.get_row_count(), 0);
        assert_eq!(buffer.get_column_count(), 0);
    }

    #[test]
    fn test_buffer_navigation() {
        let mut buffer = Buffer::new(1);

        // Test row selection
        buffer.set_selected_row(Some(5));
        assert_eq!(buffer.get_selected_row(), Some(5));

        buffer.set_selected_row(None);
        assert_eq!(buffer.get_selected_row(), None);

        // Test scroll offset
        buffer.set_scroll_offset((10, 20));
        assert_eq!(buffer.get_scroll_offset(), (10, 20));
    }

    #[test]
    fn test_buffer_sorting() {
        let mut buffer = Buffer::new(1);

        // Set sort column and order
        buffer.set_sort_column(Some(3));
        buffer.set_sort_order(SortOrder::Ascending);

        assert_eq!(buffer.get_sort_column(), Some(3));
        assert_eq!(buffer.get_sort_order(), SortOrder::Ascending);

        // Change to descending
        buffer.set_sort_order(SortOrder::Descending);
        assert_eq!(buffer.get_sort_order(), SortOrder::Descending);

        // Clear sort column
        buffer.set_sort_column(None);
        assert_eq!(buffer.get_sort_column(), None);
    }

    #[test]
    fn test_buffer_metadata() {
        let mut buffer = Buffer::new(1);

        // Test name
        assert_eq!(buffer.get_name(), "[Buffer 1]");

        // Test modified flag
        assert!(!buffer.is_modified());
        buffer.set_modified(true);
        assert!(buffer.is_modified());

        // Test CSV mode
        assert!(!buffer.is_csv_mode());
        assert_eq!(buffer.get_table_name(), "");
    }

    #[test]
    fn test_buffer_display_options() {
        let mut buffer = Buffer::new(1);

        // Test compact mode
        assert!(!buffer.is_compact_mode());
        buffer.set_compact_mode(true);
        assert!(buffer.is_compact_mode());

        // Test row numbers
        assert!(!buffer.is_show_row_numbers());
        buffer.set_show_row_numbers(true);
        assert!(buffer.is_show_row_numbers());
    }

    #[test]
    fn test_buffer_search() {
        let mut buffer = Buffer::new(1);

        // Set search pattern
        buffer.set_search_pattern("find me".to_string());
        assert_eq!(buffer.get_search_pattern(), "find me");

        // Set matches
        let matches = vec![(0, 5), (2, 10), (5, 3)];
        buffer.set_search_matches(matches.clone());
        assert_eq!(buffer.get_search_matches(), matches);

        // Set current match
        buffer.set_current_match(Some((2, 10)));
        assert_eq!(buffer.get_current_match(), Some((2, 10)));
    }

    #[test]
    fn test_buffer_input_operations() {
        let mut buffer = Buffer::new(1);

        // Set input value
        buffer.set_input_value("SELECT * FROM users".to_string());
        assert_eq!(buffer.get_input_value(), "SELECT * FROM users");
        assert_eq!(buffer.get_input_cursor(), 19); // Should be at end

        // Move cursor
        buffer.set_input_cursor(7);
        assert_eq!(buffer.get_input_cursor(), 7);

        // Change value preserves cursor position if valid
        buffer.set_input_value("SELECT id FROM users".to_string());
        assert_eq!(buffer.get_input_value(), "SELECT id FROM users");
    }
}
