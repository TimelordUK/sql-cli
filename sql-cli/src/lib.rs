pub mod api_client;
pub mod app_paths;
pub mod buffer;
pub mod cache;
pub mod config;
pub mod csv_datasource;
pub mod csv_fixes;
pub mod cursor_aware_parser;
pub mod datatable;
pub mod datatable_buffer;
pub mod datatable_loaders;
pub mod datatable_view;
pub mod dynamic_schema;
pub mod global_state;
pub mod history;
pub mod hybrid_parser;
pub mod input_manager;
pub mod logging;
pub mod modern_input;
pub mod modern_tui;
pub mod parser;
pub mod recursive_parser;
pub mod schema_config;
pub mod sql_highlighter;
pub mod virtual_table;
pub mod where_ast;
pub mod where_parser;

// New refactored modules for enhanced_tui decomposition
pub mod completion_manager;
pub mod cursor_manager;
pub mod data_analyzer;
// pub mod data_manager; // TODO: Fix QueryResponse field access
