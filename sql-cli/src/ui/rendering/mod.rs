/// Rendering module for TUI components
/// This module contains all the rendering logic extracted from enhanced_tui.rs

pub mod status_line;
pub mod table_renderer;
pub mod help_renderer;
pub mod debug_renderer;
pub mod history_renderer;

// Re-export commonly used items
pub use status_line::render_status_line;
pub use table_renderer::render_table_with_provider;
pub use help_renderer::{render_help, render_help_two_column};
pub use debug_renderer::{render_debug, render_pretty_query};
pub use history_renderer::{render_history, render_history_list};