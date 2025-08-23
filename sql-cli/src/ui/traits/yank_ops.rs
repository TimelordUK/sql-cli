use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::handlers::YankHandler;
use crate::ui::actions::{Action, YankTarget};
use std::sync::Arc;

/// Trait that provides yank operation behavior for TUI components
/// This extracts yank operations from EnhancedTui to reduce coupling
pub trait YankBehavior {
    // Required methods - these provide access to TUI internals
    fn buffer(&self) -> &dyn BufferAPI;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn state_container(&self) -> &Arc<AppStateContainer>;
    fn set_status_message(&mut self, message: String);
    fn set_error_status(&mut self, prefix: &str, error: anyhow::Error);

    // ========== Yank Operation Methods ==========

    /// Yank the currently selected cell
    fn yank_cell(&mut self) {
        let action = Action::Yank(YankTarget::Cell);
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_action(&action, buffer, self.state_container());

        match result {
            Ok(Some(message)) => {
                self.set_status_message(message);
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error_status("Failed to yank cell", e);
            }
        }
    }

    /// Yank the currently selected row
    fn yank_row(&mut self) {
        let action = Action::Yank(YankTarget::Row);
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_action(&action, buffer, self.state_container());

        match result {
            Ok(Some(message)) => {
                self.set_status_message(message);
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error_status("Failed to yank row", e);
            }
        }
    }

    /// Yank the currently selected column
    fn yank_column(&mut self) {
        let action = Action::Yank(YankTarget::Column);
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_action(&action, buffer, self.state_container());

        match result {
            Ok(Some(message)) => {
                self.set_status_message(message);
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error_status("Failed to yank column", e);
            }
        }
    }

    /// Yank all visible data
    fn yank_all(&mut self) {
        let action = Action::Yank(YankTarget::All);
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_action(&action, buffer, self.state_container());

        match result {
            Ok(Some(message)) => {
                self.set_status_message(message);
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error_status("Failed to yank all", e);
            }
        }
    }

    /// Yank the current query
    fn yank_query(&mut self) {
        let action = Action::Yank(YankTarget::Query);
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_action(&action, buffer, self.state_container());

        match result {
            Ok(Some(message)) => {
                self.set_status_message(message);
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error_status("Failed to yank query", e);
            }
        }
    }

    /// Yank current query and results as a complete test case (Ctrl+T in debug mode)
    fn yank_as_test_case(&mut self) {
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_as_test_case(buffer, self.state_container());

        match result {
            Ok(message) => {
                self.set_status_message(message);
            }
            Err(e) => {
                self.set_error_status("Failed to copy test case", e);
            }
        }
    }

    /// Yank debug dump with context for manual test creation (Shift+Y in debug mode)
    fn yank_debug_with_context(&mut self) {
        let buffer = self.buffer();
        let result = YankHandler::handle_yank_debug_context(buffer, self.state_container());

        match result {
            Ok(message) => {
                self.set_status_message(message);
            }
            Err(e) => {
                self.set_error_status("Failed to copy debug context", e);
            }
        }
    }
}
