// Minimal lib for testing DataView without TUI
// This allows us to test DataView in isolation while TUI is being refactored

pub mod data {
    pub mod datatable;
    pub mod data_view;
    pub mod data_provider;
}

// Re-export commonly used types at the crate root for convenience
pub use data::datatable::{DataTable, DataRow, DataColumn, DataValue, DataType};
pub use data::data_view::DataView;