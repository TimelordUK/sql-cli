use crate::buffer::{AppMode, EditMode};
use anyhow::Result;
use tracing::debug;

/// Handles the execution of actions triggered by key bindings
pub struct ActionHandler;

impl ActionHandler {
    /// Execute a navigation action
    pub fn handle_navigation(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "next_row" => {
                app.next_row();
                Ok(false)
            }
            "previous_row" => {
                app.previous_row();
                Ok(false)
            }
            "next_column" | "move_column_right" => {
                app.move_column_right();
                Ok(false)
            }
            "previous_column" | "move_column_left" => {
                app.move_column_left();
                Ok(false)
            }
            "page_down" => {
                app.page_down();
                Ok(false)
            }
            "page_up" => {
                app.page_up();
                Ok(false)
            }
            "goto_first_row" => {
                app.goto_first_row();
                Ok(false)
            }
            "goto_last_row" => {
                app.goto_last_row();
                Ok(false)
            }
            "goto_first_column" => {
                app.goto_first_column();
                Ok(false)
            }
            "goto_last_column" => {
                app.goto_last_column();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a mode change action
    pub fn handle_mode_change(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "enter_edit_mode" => {
                app.set_mode(AppMode::Command);
                app.set_edit_mode(EditMode::SingleLine);
                Ok(false)
            }
            "enter_results_mode" => {
                app.set_mode(AppMode::Results);
                Ok(false)
            }
            "exit_results_mode" => {
                app.set_mode(AppMode::Command);
                Ok(false)
            }
            "toggle_help" => {
                if app.get_mode() == AppMode::Help {
                    app.set_mode(AppMode::Command);
                } else {
                    app.set_mode(AppMode::Help);
                }
                Ok(false)
            }
            "toggle_debug" => {
                if app.get_mode() == AppMode::Debug {
                    app.set_mode(AppMode::Command);
                } else {
                    app.set_mode(AppMode::Debug);
                    app.generate_debug_context();
                }
                Ok(false)
            }
            "show_pretty_query" => {
                app.set_mode(AppMode::PrettyQuery);
                app.generate_pretty_query();
                Ok(false)
            }
            "show_history" => {
                app.set_mode(AppMode::History);
                Ok(false)
            }
            "show_cache" => {
                app.set_mode(AppMode::Command);
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a yank/clipboard action
    pub fn handle_yank(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "yank_cell" => {
                app.yank_cell();
                Ok(false)
            }
            "yank_row" => {
                app.yank_row();
                Ok(false)
            }
            "yank_column" => {
                app.yank_column();
                Ok(false)
            }
            "yank_all" => {
                app.yank_all();
                Ok(false)
            }
            "paste" => {
                app.paste_from_clipboard();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute an export action
    pub fn handle_export(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "export_csv" => {
                app.export_to_csv();
                Ok(false)
            }
            "export_json" => {
                app.export_to_json();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a buffer action
    pub fn handle_buffer(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "next_buffer" => {
                app.next_buffer();
                Ok(false)
            }
            "previous_buffer" => {
                app.previous_buffer();
                Ok(false)
            }
            "close_buffer" => app.close_buffer(),
            "new_buffer" => {
                app.new_buffer();
                Ok(false)
            }
            "list_buffers" => {
                app.list_buffers();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a query action
    pub fn handle_query(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "execute_query" => {
                let query = app.get_input_text();
                if !query.trim().is_empty() {
                    app.execute_query(&query)?;
                }
                Ok(false)
            }
            "handle_completion" => {
                app.handle_completion();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a filter/search action
    pub fn handle_filter(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "start_filter" => {
                app.set_mode(AppMode::Filter);
                Ok(false)
            }
            "start_fuzzy_filter" => {
                app.set_mode(AppMode::FuzzyFilter);
                Ok(false)
            }
            "start_search" => {
                app.set_mode(AppMode::Search);
                Ok(false)
            }
            "clear_filter" => {
                app.clear_filter();
                Ok(false)
            }
            "next_match" => {
                app.next_search_match();
                Ok(false)
            }
            "previous_match" => {
                app.previous_search_match();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a sort action
    pub fn handle_sort(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "sort_column" | "sort_column_asc" => {
                let column = app.get_current_column();
                app.sort_by_column(column);
                Ok(false)
            }
            "sort_column_desc" => {
                let column = app.get_current_column();
                app.sort_by_column_desc(column);
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute a column action
    pub fn handle_column(action: &str, app: &mut impl AppContext) -> Result<bool> {
        match action {
            "pin_column" | "toggle_column_pin" => {
                app.toggle_column_pin();
                Ok(false)
            }
            "clear_pins" | "clear_all_pinned_columns" => {
                app.clear_all_pinned_columns();
                Ok(false)
            }
            "calculate_statistics" => {
                app.calculate_column_statistics();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Main action dispatcher
    pub fn execute(action: &str, app: &mut impl AppContext) -> Result<bool> {
        debug!("Executing action: {}", action);

        // Check for quit action first
        if action == "quit" {
            return Ok(true);
        }

        // Try each category of actions
        if Self::handle_navigation(action, app)? {
            return Ok(true);
        }
        if Self::handle_mode_change(action, app)? {
            return Ok(true);
        }
        if Self::handle_yank(action, app)? {
            return Ok(true);
        }
        if Self::handle_export(action, app)? {
            return Ok(true);
        }
        if Self::handle_buffer(action, app)? {
            return Ok(true);
        }
        if Self::handle_query(action, app)? {
            return Ok(true);
        }
        if Self::handle_filter(action, app)? {
            return Ok(true);
        }
        if Self::handle_sort(action, app)? {
            return Ok(true);
        }
        if Self::handle_column(action, app)? {
            return Ok(true);
        }

        // Unknown action
        debug!("Unknown action: {}", action);
        Ok(false)
    }
}

/// Trait for application context that actions can operate on
pub trait AppContext {
    // Navigation
    fn next_row(&mut self);
    fn previous_row(&mut self);
    fn move_column_left(&mut self);
    fn move_column_right(&mut self);
    fn page_down(&mut self);
    fn page_up(&mut self);
    fn goto_first_row(&mut self);
    fn goto_last_row(&mut self);
    fn goto_first_column(&mut self);
    fn goto_last_column(&mut self);
    fn get_current_column(&self) -> usize;

    // Mode management
    fn get_mode(&self) -> AppMode;
    fn set_mode(&mut self, mode: AppMode);
    fn set_edit_mode(&mut self, mode: EditMode);

    // Query operations
    fn get_input_text(&self) -> String;
    fn execute_query(&mut self, query: &str) -> Result<()>;
    fn handle_completion(&mut self);

    // Yank operations
    fn yank_cell(&mut self);
    fn yank_row(&mut self);
    fn yank_column(&mut self);
    fn yank_all(&mut self);
    fn paste_from_clipboard(&mut self);

    // Export operations
    fn export_to_csv(&mut self);
    fn export_to_json(&mut self);

    // Buffer operations
    fn next_buffer(&mut self);
    fn previous_buffer(&mut self);
    fn close_buffer(&mut self) -> Result<bool>;
    fn new_buffer(&mut self);
    fn list_buffers(&mut self);

    // Filter/search operations
    fn clear_filter(&mut self);
    fn next_search_match(&mut self);
    fn previous_search_match(&mut self);

    // Sort operations
    fn sort_by_column(&mut self, column: usize);
    fn sort_by_column_desc(&mut self, column: usize);

    // Column operations
    fn toggle_column_pin(&mut self);
    fn clear_all_pinned_columns(&mut self);
    fn calculate_column_statistics(&mut self);

    // Debug operations
    fn generate_debug_context(&mut self);
    fn generate_pretty_query(&mut self);
}
