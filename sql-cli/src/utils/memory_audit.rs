use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use std::sync::Arc;
use tracing::info;

/// Detailed memory audit for tracking memory usage across components
pub struct MemoryAudit {
    pub component: String,
    pub bytes: usize,
    pub description: String,
}

impl MemoryAudit {
    pub fn new(component: &str, bytes: usize, description: &str) -> Self {
        Self {
            component: component.to_string(),
            bytes,
            description: description.to_string(),
        }
    }

    pub fn mb(&self) -> f64 {
        self.bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Estimate memory usage of a DataTable
pub fn estimate_datatable_memory(table: &DataTable) -> usize {
    let mut total_bytes = 0;

    // Base struct size
    total_bytes += std::mem::size_of::<DataTable>();

    // Column metadata
    total_bytes += table.column_count() * std::mem::size_of::<crate::data::datatable::DataColumn>();
    for col_name in table.column_names() {
        total_bytes += col_name.len();
    }

    // Row data - this is the big one
    let rows = table.row_count();
    let cols = table.column_count();

    // Estimate based on data types
    // Each DataValue enum takes up space for the discriminant + largest variant
    let value_size = std::mem::size_of::<crate::data::datatable::DataValue>();
    total_bytes += rows * cols * value_size;

    // String data (not in enum, heap allocated)
    // Sample first 100 rows to estimate string sizes
    let sample_rows = std::cmp::min(100, rows);
    let mut string_bytes = 0;

    for row_idx in 0..sample_rows {
        if let Some(row) = table.get_row(row_idx) {
            for col_idx in 0..cols {
                if let Some(value) = row.get(col_idx) {
                    use crate::data::datatable::DataValue;
                    match value {
                        DataValue::String(s) => string_bytes += s.len(),
                        DataValue::InternedString(s) => string_bytes += s.len(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Extrapolate string usage
    if sample_rows > 0 {
        let avg_string_per_row = string_bytes / sample_rows;
        total_bytes += avg_string_per_row * rows;
    }

    total_bytes
}

/// Estimate memory usage of a DataView
pub fn estimate_dataview_memory(view: &DataView) -> usize {
    let mut total_bytes = 0;

    // Base struct size
    total_bytes += std::mem::size_of::<DataView>();

    // The view holds an Arc to the DataTable (just a pointer, 8 bytes)
    total_bytes += std::mem::size_of::<Arc<DataTable>>();

    // Visible row indices
    let visible_rows = view.row_count();
    total_bytes += visible_rows * std::mem::size_of::<usize>();

    // Visible column indices
    let visible_cols = view.column_count();
    total_bytes += visible_cols * std::mem::size_of::<usize>();

    // Note: The actual DataTable is NOT counted here as it's shared via Arc

    total_bytes
}

/// Perform a comprehensive memory audit
pub fn perform_memory_audit(
    datatable: Option<&DataTable>,
    original_source: Option<&DataTable>,
    dataview: Option<&DataView>,
) -> Vec<MemoryAudit> {
    let mut audits = Vec::new();

    // Track current process memory
    if let Some(kb) = crate::utils::memory_tracker::get_process_memory_kb() {
        audits.push(MemoryAudit::new(
            "Process Total",
            kb * 1024,
            "Total process memory (RSS)",
        ));
    }

    // Track DataTable memory
    if let Some(dt) = datatable {
        let bytes = estimate_datatable_memory(dt);
        audits.push(MemoryAudit::new(
            "DataTable",
            bytes,
            &format!("{} rows x {} cols", dt.row_count(), dt.column_count()),
        ));
    }

    // Track original source memory (this is the duplication!)
    if let Some(original) = original_source {
        let bytes = estimate_datatable_memory(original);
        audits.push(MemoryAudit::new(
            "Original Source (DUPLICATE!)",
            bytes,
            &format!(
                "{} rows x {} cols",
                original.row_count(),
                original.column_count()
            ),
        ));
    }

    // Track DataView memory (should be small)
    if let Some(view) = dataview {
        let bytes = estimate_dataview_memory(view);
        audits.push(MemoryAudit::new(
            "DataView",
            bytes,
            &format!("{} visible rows", view.row_count()),
        ));
    }

    audits
}

/// Log memory audit results
pub fn log_memory_audit(audits: &[MemoryAudit]) {
    info!("=== MEMORY AUDIT ===");
    let mut total_tracked = 0;

    for audit in audits {
        info!(
            "  {}: {:.2} MB - {}",
            audit.component,
            audit.mb(),
            audit.description
        );
        if !audit.component.contains("Process") {
            total_tracked += audit.bytes;
        }
    }

    info!(
        "  Total Tracked: {:.2} MB",
        total_tracked as f64 / (1024.0 * 1024.0)
    );

    // Check for duplication
    if audits.iter().any(|a| a.component.contains("DUPLICATE")) {
        info!("  ⚠️  WARNING: Memory duplication detected!");
    }
}
