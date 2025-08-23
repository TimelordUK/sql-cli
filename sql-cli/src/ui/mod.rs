//! User interface layer
//!
//! This module contains the main TUI application and related UI components.

pub mod action_handlers;
pub mod actions;
pub mod behaviors;
pub mod cell_renderer;
pub mod column_utils;
pub mod data_export_operations;
pub mod enhanced_tui;
pub mod enhanced_tui_debug;
pub mod enhanced_tui_debug_integration;
pub mod enhanced_tui_helpers;
pub mod history_input_handler;
pub mod input_handlers;
pub mod key_chord_handler;
pub mod key_dispatcher;
pub mod key_indicator;
pub mod key_mapper;
pub mod key_sequence_renderer;
pub mod query_engine_integration;
pub mod scroll_utils;
pub mod search_operations;
pub mod simple_operations;
pub mod table_render_context;
pub mod table_renderer;
pub mod text_operations;
pub mod text_utils;
pub mod traits;
pub mod tui_app;
pub mod tui_renderer;
pub mod tui_state;
pub mod ui_layout_utils;
pub mod viewport_manager;
pub mod vim_search_manager;
