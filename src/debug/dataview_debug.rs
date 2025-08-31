use crate::data::data_view::{DataView, SortOrder};
use crate::debug::debug_trace::{DebugSection, DebugSectionBuilder, DebugTrace, Priority};
use std::sync::Arc;

/// Debug trace implementation for DataView
pub struct DataViewDebugProvider {
    dataview: Arc<DataView>,
}

impl DataViewDebugProvider {
    pub fn new(dataview: Arc<DataView>) -> Self {
        Self { dataview }
    }
}

impl DebugTrace for DataViewDebugProvider {
    fn name(&self) -> &str {
        "DataView"
    }

    fn debug_sections(&self) -> Vec<DebugSection> {
        let mut builder = DebugSectionBuilder::new();

        // Main DataView state section
        builder.add_section("DATAVIEW STATE", "", Priority::DATAVIEW);

        // Basic counts
        builder.add_field("Visible Rows", self.dataview.row_count());
        builder.add_field("Visible Columns", self.dataview.column_count());
        builder.add_field("Source Rows", self.dataview.source().row_count());
        builder.add_field("Source Columns", self.dataview.source().column_count());

        // Column mapping information
        builder.add_line("");
        builder.add_line(&self.dataview.get_column_debug_info());

        // Show visible columns in order
        let visible_columns = self.dataview.column_names();
        let column_mappings = self.dataview.get_column_index_mapping();
        builder.add_line(format!(
            "\nVisible Columns ({}) with Index Mapping:",
            visible_columns.len()
        ));
        for (visible_idx, col_name, datatable_idx) in &column_mappings {
            builder.add_line(format!(
                "  V[{:3}] → DT[{:3}] : {}",
                visible_idx, datatable_idx, col_name
            ));
        }

        // Show internal state
        builder.add_line("\n--- Internal State ---");
        let visible_indices = self.dataview.get_visible_column_indices();
        builder.add_line(format!("visible_columns array: {:?}", visible_indices));

        // Pinned columns
        let pinned_names = self.dataview.get_pinned_column_names();
        if !pinned_names.is_empty() {
            builder.add_line(format!("\nPinned Columns ({}):", pinned_names.len()));
            for (idx, name) in pinned_names.iter().enumerate() {
                let source_idx = self.dataview.source().get_column_index(name).unwrap_or(999);
                builder.add_line(format!("  [{}] {} (source_idx: {})", idx, name, source_idx));
            }
        } else {
            builder.add_line("Pinned Columns: None");
        }

        // Sort state
        let sort_state = self.dataview.get_sort_state();
        match sort_state.order {
            SortOrder::None => {
                builder.add_line("Sort State: None");
            }
            SortOrder::Ascending => {
                if let Some(col_idx) = sort_state.column {
                    let col_name = visible_columns
                        .get(col_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    builder.add_line(format!(
                        "Sort State: Column {} ('{}') Ascending ↑",
                        col_idx, col_name
                    ));
                } else {
                    builder.add_line("Sort State: Ascending (no column)");
                }
            }
            SortOrder::Descending => {
                if let Some(col_idx) = sort_state.column {
                    let col_name = visible_columns
                        .get(col_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    builder.add_line(format!(
                        "Sort State: Column {} ('{}') Descending ↓",
                        col_idx, col_name
                    ));
                } else {
                    builder.add_line("Sort State: Descending (no column)");
                }
            }
        }

        // Filter state
        if let Some(filter) = self.dataview.get_filter_pattern() {
            builder.add_line(format!("\nText Filter Active: '{}'", filter));
        }
        if let Some(fuzzy) = self.dataview.get_fuzzy_filter_pattern() {
            builder.add_line(format!("Fuzzy Filter Active: '{}'", fuzzy));
        }
        if self.dataview.has_column_search() {
            if let Some(pattern) = self.dataview.column_search_pattern() {
                builder.add_line(format!("Column Search Active: '{}'", pattern));
                let matches = self.dataview.get_matching_columns();
                builder.add_line(format!("  {} matches found", matches.len()));
            }
        }

        // Column order changes
        let original_columns = self.dataview.source().column_names();
        if visible_columns != original_columns {
            builder.add_line("\nColumn Order Changed: YES");

            // Show hidden columns
            let hidden: Vec<String> = original_columns
                .iter()
                .filter(|col| !visible_columns.contains(col))
                .cloned()
                .collect();
            if !hidden.is_empty() {
                builder.add_line(format!("Hidden Columns ({}):", hidden.len()));
                for col in hidden {
                    builder.add_line(format!("  - {}", col));
                }
            }
        } else {
            builder.add_line("\nColumn Order Changed: NO");
        }

        builder.build()
    }

    fn debug_summary(&self) -> Option<String> {
        let rows = self.dataview.row_count();
        let cols = self.dataview.column_count();
        let pinned = self.dataview.get_pinned_columns().len();
        let filtered = self.dataview.has_filter();

        let mut summary = format!("{}x{} view", rows, cols);
        if pinned > 0 {
            summary.push_str(&format!(", {} pinned", pinned));
        }
        if filtered {
            summary.push_str(", filtered");
        }

        Some(summary)
    }

    fn is_active(&self) -> bool {
        true
    }
}
