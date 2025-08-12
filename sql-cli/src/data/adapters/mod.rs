//! Adapters for existing data sources
//!
//! These adapters implement the DataProvider trait for existing data sources,
//! allowing gradual migration to the new architecture.

pub mod buffer_adapter;
pub mod csv_client_adapter;

pub use buffer_adapter::BufferAdapter;
pub use csv_client_adapter::CsvClientAdapter;
