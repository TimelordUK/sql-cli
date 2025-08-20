//! Data layer for DataTable/DataView architecture
//!
//! This module provides the data abstraction layer that separates
//! data storage from presentation.

pub mod adapters;
pub mod converters;

// Core data modules
pub mod data_provider;
pub mod data_view;
pub mod datatable;
pub mod type_inference;

pub mod datatable_buffer;
pub mod datatable_converter;
pub mod datatable_loaders;
pub mod datatable_view;

// Data source modules
pub mod advanced_csv_loader;
pub mod csv_datasource;
pub mod csv_fixes;
pub mod data_analyzer;
pub mod data_exporter;
pub mod datasource_adapter;
pub mod datasource_trait;
pub mod direct_csv_loader;

// Query execution
pub mod query_engine;
pub mod query_executor;
pub mod recursive_where_evaluator;
pub mod simple_where;
pub mod where_clause_converter;
pub mod where_evaluator;

// Test modules
#[cfg(test)]
mod test_type_coercion_datetime;
