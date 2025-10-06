//! Data layer for DataTable/DataView architecture
//!
//! This module provides the data abstraction layer that separates
//! data storage from presentation.

pub mod adapters;
pub mod converters;

// Core data modules
pub mod computed_view;
pub mod data_provider;
pub mod data_view;
pub mod datatable;
pub mod datavalue_compare;
pub mod type_inference;
pub mod value_comparisons;
pub mod value_parsing;

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
pub mod stream_loader;

// Query execution
pub mod arithmetic_evaluator;
pub mod evaluation_context;
pub mod group_by_expressions;
pub mod hash_join;
pub mod query_engine;
pub mod query_executor;
pub mod recursive_where_evaluator;
pub mod row_expanders; // Row expansion system (UNNEST, etc.)
pub mod simple_where;
pub mod subquery_executor;
pub mod temp_table_registry; // Temporary tables for script execution
pub mod unit_converter;
pub mod virtual_table_generator;
pub mod where_clause_converter;
pub mod where_evaluator;

// Test modules
#[cfg(test)]
mod group_by_test;
#[cfg(test)]
mod test_type_coercion_datetime;
