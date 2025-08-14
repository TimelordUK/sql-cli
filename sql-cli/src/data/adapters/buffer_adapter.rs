// Simplified BufferAdapter - just delegates to DataView
// This is a temporary migration adapter until we remove DataProvider completely

use crate::buffer::BufferAPI;
use crate::data::data_provider::DataProvider;
use std::fmt::Debug;

/// Minimal adapter that just uses DataView for everything
pub struct BufferAdapter<'a> {
    buffer: &'a (dyn BufferAPI + Send + Sync),
}

impl<'a> BufferAdapter<'a> {
    pub fn new(buffer: &'a (dyn BufferAPI + Send + Sync)) -> Self {
        Self { buffer }
    }
}

impl<'a> Debug for BufferAdapter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferAdapter").finish()
    }
}

impl<'a> DataProvider for BufferAdapter<'a> {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        // Use DataView if available
        if let Some(dataview) = self.buffer.get_dataview() {
            if let Some(row) = dataview.get_row(index) {
                return Some(row.values.iter().map(|v| v.to_string()).collect());
            }
        }

        // Fallback to DataTable if no DataView
        if let Some(datatable) = self.buffer.get_datatable() {
            if let Some(row) = datatable.get_row_as_strings(index) {
                return Some(row);
            }
        }

        None
    }

    fn get_column_names(&self) -> Vec<String> {
        // Use DataView if available (respects hidden columns)
        if let Some(dataview) = self.buffer.get_dataview() {
            return dataview.column_names();
        }

        // Fallback to DataTable
        if let Some(datatable) = self.buffer.get_datatable() {
            return datatable.column_names();
        }

        Vec::new()
    }

    fn get_row_count(&self) -> usize {
        // Use DataView if available (respects filtering)
        if let Some(dataview) = self.buffer.get_dataview() {
            return dataview.row_count();
        }

        // Fallback to DataTable
        if let Some(datatable) = self.buffer.get_datatable() {
            return datatable.row_count();
        }

        0
    }

    fn get_column_count(&self) -> usize {
        // Use DataView if available (respects hidden columns)
        if let Some(dataview) = self.buffer.get_dataview() {
            return dataview.column_count();
        }

        // Fallback to DataTable
        if let Some(datatable) = self.buffer.get_datatable() {
            return datatable.column_count();
        }

        0
    }
}
