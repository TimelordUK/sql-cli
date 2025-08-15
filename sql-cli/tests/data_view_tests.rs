#[cfg(test)]
mod tests {
    use sql_cli::data::data_view::DataView;
    use sql_cli::datatable::{DataTable, DataValue};
    use sql_cli::datatable_loaders::load_json_to_datatable;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn get_test_data_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // Go up one directory from sql-cli to root
        path.push("data");
        path.push(filename);
        path
    }

    /// Helper function to load trades.json into a DataTable
    fn load_trades_datatable() -> DataTable {
        let trades_path = get_test_data_path("trades.json");

        println!("\n=== Loading trades.json ===");
        println!("Path: {:?}", trades_path);

        let table =
            load_json_to_datatable(trades_path, "trades").expect("Failed to load trades.json");
        // Print comprehensive debug information
        println!("{}", table.pretty_print());
        table
    }

    #[test]
    fn test_new_dataview_shows_all_data() {
        let table = load_trades_datatable();
        let view = DataView::new(Arc::new(table.clone()));

        // trades.json has 100 rows and 53 columns
        assert_eq!(view.row_count(), 100);
        assert_eq!(view.column_count(), 53);

        // Check that first few column names are present
        let columns = view.column_names();
        assert!(columns.contains(&"traderId".to_string()));
        assert!(columns.contains(&"instrumentName".to_string()));
        assert!(columns.contains(&"quantity".to_string()));
    }

    #[test]
    fn test_hide_column() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Start with 53 columns
        assert_eq!(view.column_count(), 53);

        // Hide the "quantity" column
        view.hide_column_by_name("quantity");

        assert_eq!(view.column_count(), 52);
        assert!(view.has_hidden_columns());

        let columns = view.column_names();
        assert!(!columns.contains(&"quantity".to_string()));

        let hidden = view.get_hidden_column_names();
        assert!(hidden.contains(&"quantity".to_string()));
    }

    #[test]
    fn test_unhide_all_columns() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Hide multiple columns
        view.hide_column_by_name("quantity");
        view.hide_column_by_name("price");
        assert_eq!(view.column_count(), 51);

        // Unhide all
        view.unhide_all_columns();
        assert_eq!(view.column_count(), 53);
        assert!(!view.has_hidden_columns());
    }

    #[test]
    fn test_text_filter() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Filter for "Alice"
        view.apply_text_filter("Alice", true);
        assert!(view.row_count() > 0); // Should find some rows with Alice
        let count_with_alice = view.row_count();

        // Clear filter
        view.clear_filter();
        assert_eq!(view.row_count(), 100); // Should show all rows again
        assert!(count_with_alice < 100); // Filter should have reduced rows
    }

    #[test]
    fn test_text_filter_case_insensitive() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Filter for "alice" (lowercase) with case-insensitive
        view.apply_text_filter("alice", false);
        let insensitive_count = view.row_count();
        assert!(insensitive_count > 0);

        // Filter for "alice" (lowercase) with case-sensitive
        view.clear_filter();
        view.apply_text_filter("alice", true);
        assert_eq!(view.row_count(), 0); // Should find nothing (all names are capitalized)
    }

    #[test]
    fn test_fuzzy_filter() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Fuzzy filter for "John" should match "John Smith"
        view.apply_fuzzy_filter("John", false);
        let john_count = view.row_count();
        assert!(john_count > 0);

        // Clear and try exact match with '
        view.clear_filter();
        view.apply_fuzzy_filter("'Williams", false);
        let williams_count = view.row_count();
        assert!(williams_count > 0); // Exact substring match

        // Clear filter
        view.clear_filter();
        assert_eq!(view.row_count(), 100);
    }

    #[test]
    fn test_sort_ascending() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Get the column index for traderId (should be 0 or close to it)
        let columns = view.column_names();
        let trader_id_idx = columns.iter().position(|c| c == "traderId").unwrap();

        // Sort by traderId ascending
        view.apply_sort(trader_id_idx, true).unwrap();

        // Just verify sorting was applied without error
        assert_eq!(view.row_count(), 100);
        assert_eq!(view.get_sort_state().column, Some(trader_id_idx));
    }

    #[test]
    fn test_sort_descending() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Get the column index for traderId
        let columns = view.column_names();
        let trader_id_idx = columns.iter().position(|c| c == "traderId").unwrap();

        // Sort by traderId descending
        view.apply_sort(trader_id_idx, false).unwrap();

        // Just verify sorting was applied without error
        assert_eq!(view.row_count(), 100);
        assert_eq!(view.get_sort_state().column, Some(trader_id_idx));
    }

    #[test]
    fn test_sort_then_filter_preserves_sort() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Get column index for traderId
        let columns = view.column_names();
        let trader_id_idx = columns.iter().position(|c| c == "traderId").unwrap();

        // Sort by traderId descending
        view.apply_sort(trader_id_idx, false).unwrap();
        assert_eq!(view.get_sort_state().column, Some(trader_id_idx));

        // Apply filter for "Williams"
        view.apply_text_filter("Williams", true);
        let filtered_count = view.row_count();
        assert!(filtered_count > 0);
        assert!(filtered_count < 100);

        // Verify sort state is preserved
        assert_eq!(view.get_sort_state().column, Some(trader_id_idx));

        // Clear filter - sort should still be there
        view.clear_filter();
        assert_eq!(view.row_count(), 100);
        assert_eq!(view.get_sort_state().column, Some(trader_id_idx));
    }

    #[test]
    fn test_move_column_left() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Get the initial column order
        let initial_columns = view.column_names();

        // Find a column that's not first
        if initial_columns.len() > 1 {
            let second_col = initial_columns[1].clone();

            // Move second column left
            let moved = view.move_column_left_by_name(&second_col);
            assert!(moved);

            let new_columns = view.column_names();
            // Second column should now be first
            assert_eq!(new_columns[0], second_col);
        }
    }

    #[test]
    fn test_move_column_right() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Get the initial column order
        let initial_columns = view.column_names();

        // Move first column right
        if initial_columns.len() > 1 {
            let first_col = initial_columns[0].clone();

            let moved = view.move_column_right_by_name(&first_col);
            assert!(moved);

            let new_columns = view.column_names();
            // First column should now be second
            assert_eq!(new_columns[1], first_col);
        }
    }

    #[test]
    fn test_with_trades_json() {
        // This test requires the trades_100.json file to exist
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Basic checks
        assert_eq!(view.row_count(), 100);
        assert!(view.column_count() > 0);

        // Try filtering
        view.apply_text_filter("USD", false);
        assert!(view.row_count() > 0); // Should find some USD trades
        assert!(view.row_count() < 100); // But not all trades

        // Clear and try fuzzy filter
        view.clear_filter();
        view.apply_fuzzy_filter("buy", false);
        let fuzzy_count = view.row_count();
        assert!(fuzzy_count > 0);

        // Try sorting
        view.clear_filter();
        if let Some(amount_col) = view.source().get_column_index("amount") {
            view.apply_sort(amount_col, false).unwrap();
            // After sort, should still have all rows
            assert_eq!(view.row_count(), 100);
        }
    }
}
