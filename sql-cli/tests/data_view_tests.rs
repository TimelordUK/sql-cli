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

        assert_eq!(view.row_count(), 4);
        assert_eq!(view.column_count(), 4);
        assert_eq!(view.column_names(), vec!["id", "name", "amount", "active"]);
    }

    #[test]
    fn test_hide_column() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Hide the "amount" column
        view.hide_column_by_name("amount");

        assert_eq!(view.column_count(), 3);
        assert_eq!(view.column_names(), vec!["id", "name", "active"]);
        assert!(view.has_hidden_columns());

        let hidden = view.get_hidden_column_names();
        assert_eq!(hidden, vec!["amount"]);
    }

    #[test]
    fn test_unhide_all_columns() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Hide multiple columns
        view.hide_column_by_name("amount");
        view.hide_column_by_name("active");
        assert_eq!(view.column_count(), 2);

        // Unhide all
        view.unhide_all_columns();
        assert_eq!(view.column_count(), 4);
        assert!(!view.has_hidden_columns());
    }

    #[test]
    fn test_text_filter() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Filter for "Alice"
        view.apply_text_filter("Alice", true);
        assert_eq!(view.row_count(), 2); // Should find 2 rows with Alice

        // Clear filter
        view.clear_filter();
        assert_eq!(view.row_count(), 4); // Should show all rows again
    }

    #[test]
    fn test_text_filter_case_insensitive() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Filter for "alice" (lowercase) with case-insensitive
        view.apply_text_filter("alice", false);
        assert_eq!(view.row_count(), 2);

        // Filter for "alice" (lowercase) with case-sensitive
        view.clear_filter();
        view.apply_text_filter("alice", true);
        assert_eq!(view.row_count(), 0); // Should find nothing (all names are capitalized)
    }

    #[test]
    fn test_fuzzy_filter() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Fuzzy filter for "ale" should match "Alice"
        view.apply_fuzzy_filter("ale", false);
        assert_eq!(view.row_count(), 2);

        // Clear and try exact match with '
        view.clear_filter();
        view.apply_fuzzy_filter("'Alice", false);
        assert_eq!(view.row_count(), 2); // Exact substring match

        // Clear filter
        view.clear_filter();
        assert_eq!(view.row_count(), 4);
    }

    #[test]
    fn test_sort_ascending() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Sort by amount ascending
        view.apply_sort(2, true).unwrap(); // Column 2 is "amount"

        // Check the order by getting rows
        let rows = view.get_visible_rows();
        if let Some(first_row) = view.source().get_row(rows[0]) {
            if let DataValue::Float(amount) = &first_row.values[2] {
                assert_eq!(*amount, 50.00); // Smallest amount first
            }
        }
    }

    #[test]
    fn test_sort_descending() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Sort by amount descending
        view.apply_sort(2, false).unwrap(); // Column 2 is "amount"

        // Check the order by getting rows
        let rows = view.get_visible_rows();
        if let Some(first_row) = view.source().get_row(rows[0]) {
            if let DataValue::Float(amount) = &first_row.values[2] {
                assert_eq!(*amount, 200.75); // Largest amount first
            }
        }
    }

    #[test]
    fn test_sort_then_filter_preserves_sort() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Sort by amount descending
        view.apply_sort(2, false).unwrap();

        // Apply filter for "Alice"
        view.apply_text_filter("Alice", true);
        assert_eq!(view.row_count(), 2);

        // Check that Alice's entries are still sorted by amount
        let rows = view.get_visible_rows();
        if let Some(first_row) = view.source().get_row(rows[0]) {
            if let DataValue::Float(amount) = &first_row.values[2] {
                assert_eq!(*amount, 100.50); // Alice's larger amount first
            }
        }

        // Clear filter - sort should still be there
        view.clear_filter();
        assert_eq!(view.row_count(), 4);

        let all_rows = view.get_visible_rows();
        if let Some(first_row) = view.source().get_row(all_rows[0]) {
            if let DataValue::Float(amount) = &first_row.values[2] {
                assert_eq!(*amount, 200.75); // Largest amount still first
            }
        }
    }

    #[test]
    fn test_move_column_left() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Move "amount" column left
        let moved = view.move_column_left_by_name("amount");
        assert!(moved);

        let columns = view.column_names();
        assert_eq!(columns, vec!["id", "amount", "name", "active"]);
    }

    #[test]
    fn test_move_column_right() {
        let table = load_trades_datatable();
        let mut view = DataView::new(Arc::new(table));

        // Move "id" column right
        let moved = view.move_column_right_by_name("id");
        assert!(moved);

        let columns = view.column_names();
        assert_eq!(columns, vec!["name", "id", "amount", "active"]);
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
