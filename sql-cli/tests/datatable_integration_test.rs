use sql_cli::datatable::{DataTable, DataType};
use sql_cli::datatable_loaders::load_json_to_datatable;
use std::path::PathBuf;

fn get_test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up one directory from sql-cli to root
    path.push("data");
    path.push(filename);
    path
}

#[test]
fn test_load_real_trades_json() {
    let trades_path = get_test_data_path("trades.json");

    println!("\n=== Loading trades.json ===");
    println!("Path: {:?}", trades_path);

    let table = load_json_to_datatable(trades_path, "trades").expect("Failed to load trades.json");

    // Print comprehensive debug information
    println!("\n{}", debug_dump_table(&table));

    // Basic assertions
    assert_eq!(table.name, "trades");
    assert!(table.row_count() > 0, "Should have loaded some trades");

    // Check for expected columns
    let column_names = table.column_names();
    assert!(
        column_names.contains(&"ticker".to_string()),
        "Should have ticker column"
    );
    assert!(
        column_names.contains(&"price".to_string()),
        "Should have price column"
    );
    assert!(
        column_names.contains(&"quantity".to_string()),
        "Should have quantity column"
    );
    assert!(
        column_names.contains(&"side".to_string()),
        "Should have side column"
    );
}

#[test]
fn test_load_trades_and_inspect_types() {
    let trades_path = get_test_data_path("trades.json");
    let table = load_json_to_datatable(trades_path, "trades").expect("Failed to load trades.json");

    // Check specific column types
    if let Some(price_col) = table.get_column("price") {
        println!("Price column type: {:?}", price_col.data_type);
        assert!(
            matches!(price_col.data_type, DataType::Float | DataType::Integer),
            "Price should be numeric"
        );
    }

    if let Some(ticker_col) = table.get_column("ticker") {
        println!("Ticker column type: {:?}", ticker_col.data_type);
        assert_eq!(
            ticker_col.data_type,
            DataType::String,
            "Ticker should be string"
        );
    }

    if let Some(quantity_col) = table.get_column("quantity") {
        println!("Quantity column type: {:?}", quantity_col.data_type);
        assert_eq!(
            quantity_col.data_type,
            DataType::Integer,
            "Quantity should be integer"
        );
    }
}

/// Create a nice debug dump of a DataTable for F5 debugger display
pub fn debug_dump_table(table: &DataTable) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "╔═══════════════════════════════════════════════════════╗\n"
    ));
    output.push_str(&format!("║ DataTable: {:^41} ║\n", table.name));
    output.push_str(&format!(
        "╠═══════════════════════════════════════════════════════╣\n"
    ));

    // Summary stats
    output.push_str(&format!(
        "║ Rows: {:6} | Columns: {:3} | Memory: ~{:6} bytes ║\n",
        table.row_count(),
        table.column_count(),
        table.get_stats().memory_size
    ));

    // Metadata if any
    if !table.metadata.is_empty() {
        output.push_str(&format!(
            "╠═══════════════════════════════════════════════════════╣\n"
        ));
        output.push_str(&format!(
            "║ Metadata:                                             ║\n"
        ));
        for (key, value) in &table.metadata {
            let truncated_value = if value.len() > 35 {
                format!("{}...", &value[..32])
            } else {
                value.clone()
            };
            output.push_str(&format!(
                "║   {:15} : {:35} ║\n",
                truncate_string(key, 15),
                truncated_value
            ));
        }
    }

    // Column details
    output.push_str(&format!(
        "╠═══════════════════════════════════════════════════════╣\n"
    ));
    output.push_str(&format!(
        "║ Columns:                                              ║\n"
    ));
    output.push_str(&format!(
        "╟───────────────────┬──────────┬─────────┬──────┬──────╢\n"
    ));
    output.push_str(&format!(
        "║ Name              │ Type     │ Nullable│ Nulls│Unique║\n"
    ));
    output.push_str(&format!(
        "╟───────────────────┼──────────┼─────────┼──────┼──────╢\n"
    ));

    for column in &table.columns {
        let type_str = format!("{:?}", column.data_type);
        let unique_str = column
            .unique_values
            .map(|u| format!("{:5}", u))
            .unwrap_or_else(|| "  ?  ".to_string());

        output.push_str(&format!(
            "║ {:17} │ {:8} │ {:7} │ {:4} │{}║\n",
            truncate_string(&column.name, 17),
            truncate_string(&type_str, 8),
            if column.nullable { "Yes" } else { "No " },
            column.null_count,
            unique_str
        ));
    }

    output.push_str(&format!(
        "╟───────────────────┴──────────┴─────────┴──────┴──────╢\n"
    ));

    // Sample data (first 5 rows)
    let sample_rows = 5.min(table.row_count());
    if sample_rows > 0 {
        output.push_str(&format!(
            "║ Sample Data (first {} rows):                          ║\n",
            sample_rows
        ));
        output.push_str(&format!(
            "╟───────────────────────────────────────────────────────╢\n"
        ));

        // Column headers for sample data
        let mut header_line = String::from("║ ");
        for (i, col) in table.columns.iter().enumerate() {
            if i < 4 {
                // Show first 4 columns
                header_line.push_str(&format!("{:12} ", truncate_string(&col.name, 12)));
            }
        }
        if table.columns.len() > 4 {
            header_line.push_str("...");
        }
        while header_line.len() < 56 {
            header_line.push(' ');
        }
        header_line.push_str("║\n");
        output.push_str(&header_line);

        output.push_str(&format!(
            "╟───────────────────────────────────────────────────────╢\n"
        ));

        // Sample rows
        for row_idx in 0..sample_rows {
            let mut row_line = String::from("║ ");
            for col_idx in 0..4.min(table.columns.len()) {
                if let Some(value) = table.get_value(row_idx, col_idx) {
                    row_line.push_str(&format!("{:12} ", truncate_string(&value.to_string(), 12)));
                }
            }
            if table.columns.len() > 4 {
                row_line.push_str("...");
            }
            while row_line.len() < 56 {
                row_line.push(' ');
            }
            row_line.push_str("║\n");
            output.push_str(&row_line);
        }
    }

    output.push_str(&format!(
        "╚═══════════════════════════════════════════════════════╝\n"
    ));

    output
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[test]
fn test_debug_dump_display() {
    // Create a small test table
    use sql_cli::datatable::{DataColumn, DataRow, DataValue};

    let mut table = DataTable::new("test_table");

    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("name").with_type(DataType::String));
    table.add_column(DataColumn::new("price").with_type(DataType::Float));

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Widget".to_string()),
            DataValue::Float(9.99),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Gadget".to_string()),
            DataValue::Float(19.99),
        ]))
        .unwrap();

    let dump = debug_dump_table(&table);
    println!("{}", dump);

    assert!(dump.contains("test_table"));
    assert!(dump.contains("Rows:      2"));
    assert!(dump.contains("Widget"));
}
