use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::ui::viewport_manager::ViewportManager;
use std::sync::Arc;

fn main() {
    // Create a simple table
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("Name"));
    table.add_column(DataColumn::new("Age"));
    table.add_column(DataColumn::new("City"));
    table.add_column(DataColumn::new("Comments"));
    table.add_column(DataColumn::new("Status"));

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Alice".to_string()),
            DataValue::Integer(30),
            DataValue::String("NYC".to_string()),
            DataValue::String("Long comment".to_string()),
            DataValue::String("Active".to_string()),
        ]))
        .unwrap();

    let mut dataview = DataView::new(Arc::new(table));

    // Hide the "Comments" column (index 3)
    println!("Before hiding: {} columns", dataview.column_count());
    dataview.hide_column(3);
    println!("After hiding: {} columns", dataview.column_count());

    // Create viewport
    let mut viewport = ViewportManager::new(Arc::new(dataview));

    // Get visual display for row 0
    let row_indices = vec![0];

    let (headers, data, widths) = viewport.get_visual_display(100, &row_indices);

    println!("\nHeaders: {:?}", headers);
    println!("Data: {:?}", data);
    println!("\nAlignment check:");
    for (i, header) in headers.iter().enumerate() {
        if let Some(row) = data.first() {
            if let Some(value) = row.get(i) {
                println!("  {} -> {}", header, value);
            }
        }
    }
}
