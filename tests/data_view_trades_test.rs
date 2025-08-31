// Test DataView with real trades.json data (100 trades with 15 columns)

use sql_cli::data::data_view::DataView;
use sql_cli::datatable::DataValue;
use sql_cli::datatable_loaders::load_json_to_datatable;
use std::path::PathBuf;
use std::sync::Arc;

fn get_test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("data");
    path.push(filename);
    path
}

fn load_trades_datatable() -> sql_cli::datatable::DataTable {
    let trades_path = get_test_data_path("trades.json");
    load_json_to_datatable(trades_path, "trades").expect("Failed to load trades.json")
}

#[test]
fn test_trades_data_loaded_correctly() {
    let table = load_trades_datatable();
    let view = DataView::new(Arc::new(table));

    // trades.json has 100 rows and 15 columns
    assert_eq!(view.row_count(), 100, "Should have 100 trades");
    assert_eq!(view.column_count(), 15, "Should have 15 columns");

    // Verify the actual column names from trades.json
    let columns = view.column_names();
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"instrumentId".to_string()));
    assert!(columns.contains(&"quantity".to_string()));
    assert!(columns.contains(&"price".to_string()));
    assert!(columns.contains(&"counterparty".to_string()));
    assert!(columns.contains(&"trader".to_string()));
    assert!(columns.contains(&"book".to_string()));
}

#[test]
fn test_hide_columns_on_trades() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    assert_eq!(view.column_count(), 15);

    // Hide some actual columns from trades.json
    view.hide_column_by_name("commission");
    assert_eq!(view.column_count(), 14);

    view.hide_column_by_name("confirmationStatus");
    assert_eq!(view.column_count(), 13);

    // Verify they're hidden
    let columns = view.column_names();
    assert!(!columns.contains(&"commission".to_string()));
    assert!(!columns.contains(&"confirmationStatus".to_string()));

    // Check hidden list
    let hidden = view.get_hidden_column_names();
    assert_eq!(hidden.len(), 2);
    assert!(hidden.contains(&"commission".to_string()));
    assert!(hidden.contains(&"confirmationStatus".to_string()));

    // Unhide all
    view.unhide_all_columns();
    assert_eq!(view.column_count(), 15);
}

#[test]
fn test_pin_columns_on_trades() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Pin some important columns
    view.pin_column_by_name("instrumentId").unwrap();
    view.pin_column_by_name("quantity").unwrap();
    view.pin_column_by_name("price").unwrap();

    // Check they're pinned
    assert_eq!(view.get_pinned_columns().len(), 3);
    let pinned_names = view.get_pinned_column_names();
    assert_eq!(pinned_names, vec!["instrumentId", "quantity", "price"]);

    // Column order should have pinned first
    let columns = view.column_names();
    assert_eq!(columns[0], "instrumentId");
    assert_eq!(columns[1], "quantity");
    assert_eq!(columns[2], "price");

    // Try to hide a pinned column - should fail
    let hidden = view.hide_column_by_name("price");
    assert!(!hidden);
    assert_eq!(view.column_count(), 15); // Still all columns visible
}

#[test]
fn test_filter_trades_by_counterparty() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    assert_eq!(view.row_count(), 100);

    // Filter for a specific counterparty (check what's in the data)
    view.apply_text_filter("MORGAN", false);

    // Should have fewer rows now
    let filtered_count = view.row_count();
    assert!(filtered_count > 0, "Should find some MORGAN trades");
    assert!(filtered_count < 100, "Should filter out non-MORGAN trades");

    // Check visible rows contain MORGAN
    for i in 0..filtered_count.min(5) {
        if let Some(row) = view.get_row(i) {
            let row_text = row
                .values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(row_text.contains("MORGAN"));
        }
    }

    // Clear filter
    view.clear_filter();
    assert_eq!(view.row_count(), 100);
}

#[test]
fn test_sort_trades_by_quantity() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Find quantity column index
    let columns = view.column_names();
    let quantity_idx = columns.iter().position(|c| c == "quantity").unwrap();

    // Sort by quantity descending
    view.apply_sort(quantity_idx, false).unwrap();

    // Check first few rows have high quantities
    let mut last_quantity = f64::MAX;
    for i in 0..5 {
        if let Some(row) = view.get_row(i) {
            if let DataValue::Float(qty) = &row.values[quantity_idx] {
                assert!(*qty <= last_quantity, "Quantities should be descending");
                last_quantity = *qty;
            } else if let DataValue::Integer(qty) = &row.values[quantity_idx] {
                let qty_float = *qty as f64;
                assert!(
                    qty_float <= last_quantity,
                    "Quantities should be descending"
                );
                last_quantity = qty_float;
            }
        }
    }
}

#[test]
fn test_fuzzy_filter_on_counterparty() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Fuzzy search for partial counterparty name
    view.apply_fuzzy_filter("MRGN", true); // Should match "MORGAN" with fuzzy

    let matches = view.row_count();
    assert!(
        matches > 0,
        "Should find some MORGAN trades with fuzzy match"
    );
    assert!(matches < 100, "Should filter out non-MORGAN trades");

    // Try exact match with '
    view.clear_filter();
    view.apply_fuzzy_filter("'MORGAN", false); // Exact substring match

    let exact_matches = view.row_count();
    assert!(
        exact_matches > 0,
        "Should find MORGAN trades with exact match"
    );
}

#[test]
fn test_combined_operations_on_trades() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // 1. Pin key columns
    view.pin_column_by_name("instrumentId").unwrap();
    view.pin_column_by_name("counterparty").unwrap();

    // 2. Hide less important columns
    view.hide_column_by_name("commission");
    view.hide_column_by_name("confirmationStatus");

    // 3. Filter for specific trader
    view.apply_text_filter("Jane", false);

    // 4. Sort by price
    let columns = view.column_names();
    let price_idx = columns.iter().position(|c| c == "price").unwrap();
    view.apply_sort(price_idx, false).unwrap();

    // Verify state
    assert_eq!(view.get_pinned_columns().len(), 2);
    assert_eq!(view.column_count(), 13); // 15 - 2 hidden
    let filtered = view.row_count();
    assert!(filtered > 0, "Should find Jane's trades");
    assert!(filtered < 100, "Should be filtered");

    // Check first row has pinned columns first
    if let Some(row) = view.get_row(0) {
        let columns = view.column_names();
        assert_eq!(columns[0], "instrumentId");
        assert_eq!(columns[1], "counterparty");
    }
}

#[test]
fn test_export_filtered_trades() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Set up a specific view
    view.pin_column_by_name("instrumentId").unwrap();
    view.hide_column_by_name("commission");
    view.apply_text_filter("USD", false); // Filter for USD trades

    // Export to CSV
    let csv = view.to_csv().unwrap();
    let lines: Vec<&str> = csv.lines().collect();

    // Header should start with instrumentId (pinned)
    assert!(lines[0].starts_with("instrumentId,"));

    // Should not contain commission column
    assert!(!lines[0].contains("commission"));

    // Export to JSON
    let json = view.to_json();
    if let Some(array) = json.as_array() {
        assert!(array.len() > 0, "Should have some USD trades");
        assert!(array.len() < 100, "Should be filtered");

        // Check first item
        if let Some(first) = array.first() {
            assert!(first.get("instrumentId").is_some());
            assert!(first.get("commission").is_none()); // Hidden column
        }
    }
}

#[test]
fn test_column_search_on_trades() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Search for columns containing "counter"
    view.search_columns("counter");

    let matches = view.get_matching_columns();
    assert!(matches.len() > 0, "Should find counter columns");

    // Should find counterparty, counterpartyCountry, counterpartyType
    let match_names: Vec<String> = matches.iter().map(|(_, name)| name.clone()).collect();
    assert!(match_names.contains(&"counterparty".to_string()));
    assert!(match_names.contains(&"counterpartyCountry".to_string()));
    assert!(match_names.contains(&"counterpartyType".to_string()));

    // Navigate through matches
    let first_match = view.get_current_column_match();
    assert!(first_match.is_some());

    if matches.len() > 1 {
        let next_match = view.next_column_match();
        assert!(next_match.is_some());
    }

    // Clear search
    view.clear_column_search();
    assert!(!view.has_column_search());
}

#[test]
fn test_trades_data_integrity() {
    let table = load_trades_datatable();
    let view = DataView::new(Arc::new(table));

    // Check first trade has expected structure
    if let Some(first_trade) = view.get_row(0) {
        assert_eq!(first_trade.values.len(), 15);

        // Verify some values are not null
        let columns = view.column_names();
        let id_idx = columns.iter().position(|c| c == "id").unwrap();
        let inst_idx = columns.iter().position(|c| c == "instrumentId").unwrap();

        assert!(!matches!(first_trade.values[id_idx], DataValue::Null));
        assert!(!matches!(first_trade.values[inst_idx], DataValue::Null));
    }

    // Check we can access last trade
    if let Some(last_trade) = view.get_row(99) {
        assert_eq!(last_trade.values.len(), 15);
    }
}

#[test]
fn test_move_columns_with_pinned() {
    let table = load_trades_datatable();
    let mut view = DataView::new(Arc::new(table));

    // Pin first two columns
    view.pin_column_by_name("id").unwrap();
    view.pin_column_by_name("instrumentId").unwrap();

    // Get column order
    let columns = view.column_names();
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "instrumentId");

    // Try to move a regular column
    let book_idx = columns.iter().position(|c| c == "book").unwrap();
    view.move_column_left(book_idx);

    // Pinned columns should still be first
    let columns = view.column_names();
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "instrumentId");
}
