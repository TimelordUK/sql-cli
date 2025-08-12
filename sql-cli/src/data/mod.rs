//! Data layer for DataTable/DataView architecture
//!
//! This module provides the data abstraction layer that separates
//! data storage from presentation.

pub mod adapters;
pub mod converters;

// These will be moved here:
// - data_provider.rs → provider.rs
// - datatable.rs → table.rs
// - datatable_view.rs → view.rs
// - datatable_buffer.rs
// - datatable_converter.rs
// - datatable_loaders.rs
