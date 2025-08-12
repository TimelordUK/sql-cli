// New module structure (gradually moving files here)
pub mod api;
pub mod config;
pub mod core;
pub mod data;
pub mod sql;
pub mod state;
pub mod ui;
pub mod utils;
pub mod widgets;

// Existing flat structure (to be gradually moved to modules above)
pub mod api_client;
// pub mod app_paths; // Moved to utils/
pub mod buffer;
pub mod buffer_handler;
// pub mod cache; // Moved to sql/
pub mod cell_renderer;
// pub mod config; // Moved to config module
// pub mod csv_datasource; // Moved to data/
// pub mod csv_fixes; // Moved to data/
// pub mod cursor_aware_parser; // Moved to sql/
pub mod cursor_operations;
// pub mod data_exporter; // Moved to data/
// pub mod data_provider; // Moved to data/
// pub mod datasource_adapter; // Moved to data/
// pub mod datasource_trait; // Moved to data/
// pub mod datatable; // Moved to data/
// pub mod datatable_buffer; // Moved to data/
// pub mod datatable_converter; // Moved to data/
// pub mod datatable_loaders; // Moved to data/
// pub mod datatable_view; // Moved to data/
// pub mod debouncer; // Moved to utils/
// pub mod debug_info; // Moved to utils/
// pub mod debug_service; // Moved to utils/
// pub mod debug_widget; // Moved to widgets/
// pub mod dual_logging; // Moved to utils/
pub mod dynamic_schema;
// pub mod editor_widget; // Moved to widgets/
pub mod global_state;
// pub mod help_widget; // Moved to widgets/
pub mod history;
pub mod history_protection;
// pub mod history_widget; // Moved to widgets/
// pub mod hybrid_parser; // Moved to sql/
pub mod input_manager;
pub mod key_indicator;
// pub mod logging; // Moved to utils/
// pub mod modern_input; // Removed - experimental
// pub mod modern_tui; // Moved to ui/
// pub mod parser; // Moved to sql/
// pub mod recursive_parser; // Moved to sql/
// pub mod schema_config; // Moved to config/
// pub mod search_modes_widget; // Moved to widgets/
pub mod service_container;
// pub mod sql_highlighter; // Moved to sql/
pub mod state_manager;
// pub mod stats_widget; // Moved to widgets/
pub mod virtual_table;
// pub mod where_ast; // Moved to sql/
// pub mod where_parser; // Moved to sql/
pub mod widget_traits;
pub mod yank_manager;

// New refactored modules for enhanced_tui decomposition
pub mod action_handler;
pub mod app_state_container;
pub mod column_manager;
pub mod completion_manager;
pub mod cursor_manager;
// pub mod data_analyzer; // Moved to data/
pub mod help_text;
pub mod history_manager;
// pub mod key_bindings; // Moved to config/
pub mod key_chord_handler;
// pub mod key_dispatcher; // Moved to ui/
pub mod search_filter;
pub mod text_navigation;
// pub mod tui_renderer; // Moved to ui/
// pub mod tui_state; // Moved to ui/
// pub mod data_manager; // TODO: Fix QueryResponse field access

// Re-export widgets for backward compatibility
pub use widgets::debug_widget;
pub use widgets::editor_widget;
pub use widgets::help_widget;
pub use widgets::history_widget;
pub use widgets::search_modes_widget;
pub use widgets::stats_widget;

// Re-export data modules for backward compatibility
pub use data::csv_datasource;
pub use data::csv_fixes;
pub use data::data_analyzer;
pub use data::data_exporter;
pub use data::data_provider;
pub use data::datasource_adapter;
pub use data::datasource_trait;
pub use data::datatable;
pub use data::datatable_buffer;
pub use data::datatable_converter;
pub use data::datatable_loaders;
pub use data::datatable_view;

// Re-export UI modules for backward compatibility
pub use ui::enhanced_tui;
pub use ui::key_dispatcher;
pub use ui::tui_app;
pub use ui::tui_renderer;
pub use ui::tui_state;

// Re-export SQL modules for backward compatibility
pub use sql::cache;
pub use sql::cursor_aware_parser;
pub use sql::hybrid_parser;
pub use sql::parser;
pub use sql::recursive_parser;
pub use sql::smart_parser;
pub use sql::sql_highlighter;
pub use sql::where_ast;
pub use sql::where_parser;

// Re-export utils modules for backward compatibility
pub use utils::app_paths;
pub use utils::debouncer;
pub use utils::debug_helpers;
pub use utils::debug_info;
pub use utils::debug_service;
pub use utils::dual_logging;
pub use utils::logging;

// Re-export config modules for backward compatibility
pub use config::config as config_module;
pub use config::key_bindings;
pub use config::schema_config;
