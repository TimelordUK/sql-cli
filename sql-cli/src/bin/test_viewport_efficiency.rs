// Test program to verify viewport efficiency improvements
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::ui::viewport_manager::ViewportManager;
use std::sync::Arc;

fn main() {
    // Create a test table with many columns of varying widths
    let mut table = DataTable::new("test");

    // Add 60 columns with different widths
    for i in 0..60 {
        let col_name = if i == 51 {
            "longColumnName51".to_string()
        } else if i == 52 {
            "veryLongColumnName52".to_string()
        } else if i == 53 {
            "extremelyLongColumnName53".to_string()
        } else if i == 54 {
            "shortCol54".to_string()
        } else if i == 55 {
            "mediumColumnName55".to_string()
        } else if i == 56 {
            "anotherLongColumnName56".to_string()
        } else if i == 0 {
            "id".to_string()
        } else {
            format!("col{}", i)
        };
        table.add_column(DataColumn::new(&col_name));
    }

    // Add some sample data
    for row_id in 1..=5 {
        let mut values = vec![DataValue::Integer(row_id as i64)];
        for col in 1..60 {
            if col >= 51 && col <= 56 {
                values.push(DataValue::String(format!("LongValue{}", col)));
            } else {
                values.push(DataValue::String(format!("A{}", col)));
            }
        }
        table.add_row(DataRow::new(values)).unwrap();
    }

    let dataview = Arc::new(DataView::new(Arc::new(table)));
    let mut viewport_manager = ViewportManager::new(dataview);

    // Test different terminal widths
    let test_widths = vec![80, 120, 160, 200];

    println!("Testing Viewport Efficiency Optimization");
    println!("=========================================\n");

    for width in test_widths {
        viewport_manager.set_viewport(0, 0, width, 24);

        // Test regular column layout
        let visible_indices = viewport_manager.calculate_visible_column_indices(width);
        let efficiency = viewport_manager.calculate_efficiency_metrics(width);

        println!("Terminal width: {}w", width);
        println!("  Visible columns: {}", visible_indices.len());
        println!("  Used width: {}w", efficiency.used_width);
        println!("  Wasted space: {}w", efficiency.wasted_space);
        println!("  Efficiency: {}%", efficiency.efficiency_percent);

        // Test optimal offset for last column
        let optimal_scrollable_offset =
            viewport_manager.calculate_optimal_offset_for_last_column(width);

        // Convert scrollable offset back to absolute for update_column_viewport
        let pinned_count = 0; // No pinned columns in our test
        let optimal_absolute_offset = optimal_scrollable_offset + pinned_count;

        println!(
            "  Optimal offset for last column: {} (scrollable), {} (absolute)",
            optimal_scrollable_offset, optimal_absolute_offset
        );

        // Set viewport to optimal offset and recalculate
        viewport_manager.update_column_viewport(optimal_absolute_offset, width);
        let last_col_indices = viewport_manager.calculate_visible_column_indices(width);
        let last_col_efficiency = viewport_manager.calculate_efficiency_metrics(width);

        println!("  With last column optimization:");
        println!("    Visible columns: {}", last_col_indices.len());
        println!(
            "    Efficiency: {}%",
            last_col_efficiency.efficiency_percent
        );
        println!(
            "    Last column visible: {}",
            if last_col_indices.contains(&59) {
                "YES"
            } else {
                "NO"
            }
        );
        println!();
    }
}
