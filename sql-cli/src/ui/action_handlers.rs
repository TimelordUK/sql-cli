//! Action handlers using visitor pattern
//!
//! This module implements a visitor pattern for handling different groups of actions,
//! allowing us to break down the massive try_handle_action function into manageable chunks.

use crate::buffer::AppMode;
use crate::ui::actions::{Action, ActionContext, ActionResult, NavigateAction, YankTarget};
use anyhow::Result;

/// Trait for handling groups of related actions
pub trait ActionHandler {
    /// Handle an action if this handler is responsible for it
    fn handle_action(
        &self,
        action: &Action,
        context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>>;

    /// Get the name of this handler (for debugging/logging)
    fn name(&self) -> &'static str;
}

/// Context interface for action handlers to interact with the TUI
/// This abstracts the TUI methods that action handlers need
pub trait ActionHandlerContext {
    // Navigation methods
    fn previous_row(&mut self);
    fn next_row(&mut self);
    fn move_column_left(&mut self);
    fn move_column_right(&mut self);
    fn page_up(&mut self);
    fn page_down(&mut self);
    fn goto_first_row(&mut self);
    fn goto_last_row(&mut self);
    fn goto_first_column(&mut self);
    fn goto_last_column(&mut self);
    fn goto_row(&mut self, row: usize);
    fn goto_column(&mut self, col: usize);

    // Mode and UI state
    fn set_mode(&mut self, mode: AppMode);
    fn get_mode(&self) -> AppMode;
    fn set_status_message(&mut self, message: String);

    // Column operations
    fn toggle_column_pin(&mut self);
    fn hide_current_column(&mut self);
    fn unhide_all_columns(&mut self);
    fn clear_all_pinned_columns(&mut self);

    // Export operations
    fn export_to_csv(&mut self);
    fn export_to_json(&mut self);

    // Yank operations
    fn yank_cell(&mut self);
    fn yank_row(&mut self);
    fn yank_column(&mut self);
    fn yank_all(&mut self);
    fn yank_query(&mut self);
}

/// Handler for navigation actions (Up, Down, Left, Right, PageUp, etc.)
pub struct NavigationActionHandler;

impl ActionHandler for NavigationActionHandler {
    fn handle_action(
        &self,
        action: &Action,
        _context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>> {
        match action {
            Action::Navigate(nav_action) => match nav_action {
                NavigateAction::Up(count) => {
                    for _ in 0..*count {
                        tui.previous_row();
                    }
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::Down(count) => {
                    for _ in 0..*count {
                        tui.next_row();
                    }
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::Left(count) => {
                    for _ in 0..*count {
                        tui.move_column_left();
                    }
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::Right(count) => {
                    for _ in 0..*count {
                        tui.move_column_right();
                    }
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::PageUp => {
                    tui.page_up();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::PageDown => {
                    tui.page_down();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::Home => {
                    tui.goto_first_row();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::End => {
                    tui.goto_last_row();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::FirstColumn => {
                    tui.goto_first_column();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::LastColumn => {
                    tui.goto_last_column();
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::JumpToRow(row) => {
                    tui.goto_row(*row);
                    Some(Ok(ActionResult::Handled))
                }
                NavigateAction::JumpToColumn(col) => {
                    tui.goto_column(*col);
                    Some(Ok(ActionResult::Handled))
                }
            },
            Action::NextColumn => {
                tui.move_column_right();
                Some(Ok(ActionResult::Handled))
            }
            Action::PreviousColumn => {
                tui.move_column_left();
                Some(Ok(ActionResult::Handled))
            }
            _ => None, // Not handled by this handler
        }
    }

    fn name(&self) -> &'static str {
        "Navigation"
    }
}

/// Handler for column operations (pin, hide, sort, etc.)
pub struct ColumnActionHandler;

impl ActionHandler for ColumnActionHandler {
    fn handle_action(
        &self,
        action: &Action,
        _context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>> {
        match action {
            Action::ToggleColumnPin => {
                tui.toggle_column_pin();
                Some(Ok(ActionResult::Handled))
            }
            Action::HideColumn => {
                tui.hide_current_column();
                Some(Ok(ActionResult::Handled))
            }
            Action::UnhideAllColumns => {
                tui.unhide_all_columns();
                Some(Ok(ActionResult::Handled))
            }
            Action::ClearAllPins => {
                tui.clear_all_pinned_columns();
                Some(Ok(ActionResult::Handled))
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "Column"
    }
}

/// Handler for export operations (CSV, JSON)
pub struct ExportActionHandler;

impl ActionHandler for ExportActionHandler {
    fn handle_action(
        &self,
        action: &Action,
        _context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>> {
        match action {
            Action::ExportToCsv => {
                tui.export_to_csv();
                Some(Ok(ActionResult::Handled))
            }
            Action::ExportToJson => {
                tui.export_to_json();
                Some(Ok(ActionResult::Handled))
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "Export"
    }
}

/// Handler for yank operations (cell, row, column, etc.)
pub struct YankActionHandler;

impl ActionHandler for YankActionHandler {
    fn handle_action(
        &self,
        action: &Action,
        _context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>> {
        match action {
            Action::Yank(target) => {
                match target {
                    YankTarget::Cell => tui.yank_cell(),
                    YankTarget::Row => tui.yank_row(),
                    YankTarget::Column => tui.yank_column(),
                    YankTarget::All => tui.yank_all(),
                    YankTarget::Query => tui.yank_query(),
                }
                Some(Ok(ActionResult::Handled))
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "Yank"
    }
}

/// Handler for UI mode and display operations
pub struct UIActionHandler;

impl ActionHandler for UIActionHandler {
    fn handle_action(
        &self,
        action: &Action,
        _context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Option<Result<ActionResult>> {
        match action {
            Action::ShowHelp => {
                tui.set_mode(AppMode::Help);
                tui.set_status_message("Help mode - Press 'q' or Escape to return".to_string());
                Some(Ok(ActionResult::Handled))
            }
            Action::ShowDebugInfo => {
                tui.set_mode(AppMode::Debug);
                tui.set_status_message("Debug mode - Press 'q' or Escape to return".to_string());
                Some(Ok(ActionResult::Handled))
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "UI"
    }
}

/// Main action dispatcher using visitor pattern
pub struct ActionDispatcher {
    handlers: Vec<Box<dyn ActionHandler>>,
}

impl ActionDispatcher {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn ActionHandler>> = vec![
            Box::new(NavigationActionHandler),
            Box::new(ColumnActionHandler),
            Box::new(ExportActionHandler),
            Box::new(YankActionHandler),
            Box::new(UIActionHandler),
        ];

        Self { handlers }
    }

    /// Dispatch an action to the appropriate handler
    pub fn dispatch(
        &self,
        action: &Action,
        context: &ActionContext,
        tui: &mut dyn ActionHandlerContext,
    ) -> Result<ActionResult> {
        for handler in &self.handlers {
            if let Some(result) = handler.handle_action(action, context, tui) {
                return result;
            }
        }

        // No handler found for this action
        Ok(ActionResult::NotHandled)
    }
}

impl Default for ActionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::actions::{Action, NavigateAction};

    // Mock implementation for testing
    struct MockTui {
        pub last_action: String,
        pub mode: AppMode,
        pub status_message: String,
    }

    impl MockTui {
        fn new() -> Self {
            Self {
                last_action: String::new(),
                mode: AppMode::Results,
                status_message: String::new(),
            }
        }
    }

    impl ActionHandlerContext for MockTui {
        fn previous_row(&mut self) {
            self.last_action = "previous_row".to_string();
        }
        fn next_row(&mut self) {
            self.last_action = "next_row".to_string();
        }
        fn move_column_left(&mut self) {
            self.last_action = "move_column_left".to_string();
        }
        fn move_column_right(&mut self) {
            self.last_action = "move_column_right".to_string();
        }
        fn page_up(&mut self) {
            self.last_action = "page_up".to_string();
        }
        fn page_down(&mut self) {
            self.last_action = "page_down".to_string();
        }
        fn goto_first_row(&mut self) {
            self.last_action = "goto_first_row".to_string();
        }
        fn goto_last_row(&mut self) {
            self.last_action = "goto_last_row".to_string();
        }
        fn goto_first_column(&mut self) {
            self.last_action = "goto_first_column".to_string();
        }
        fn goto_last_column(&mut self) {
            self.last_action = "goto_last_column".to_string();
        }
        fn goto_row(&mut self, row: usize) {
            self.last_action = format!("goto_row_{}", row);
        }
        fn goto_column(&mut self, col: usize) {
            self.last_action = format!("goto_column_{}", col);
        }

        fn set_mode(&mut self, mode: AppMode) {
            self.mode = mode;
        }
        fn get_mode(&self) -> AppMode {
            self.mode.clone()
        }
        fn set_status_message(&mut self, message: String) {
            self.status_message = message;
        }

        fn toggle_column_pin(&mut self) {
            self.last_action = "toggle_column_pin".to_string();
        }
        fn hide_current_column(&mut self) {
            self.last_action = "hide_current_column".to_string();
        }
        fn unhide_all_columns(&mut self) {
            self.last_action = "unhide_all_columns".to_string();
        }
        fn clear_all_pinned_columns(&mut self) {
            self.last_action = "clear_all_pinned_columns".to_string();
        }

        fn export_to_csv(&mut self) {
            self.last_action = "export_to_csv".to_string();
        }
        fn export_to_json(&mut self) {
            self.last_action = "export_to_json".to_string();
        }

        fn yank_cell(&mut self) {
            self.last_action = "yank_cell".to_string();
        }
        fn yank_row(&mut self) {
            self.last_action = "yank_row".to_string();
        }
        fn yank_column(&mut self) {
            self.last_action = "yank_column".to_string();
        }
        fn yank_all(&mut self) {
            self.last_action = "yank_all".to_string();
        }
        fn yank_query(&mut self) {
            self.last_action = "yank_query".to_string();
        }
    }

    #[test]
    fn test_navigation_handler() {
        let handler = NavigationActionHandler;
        let mut mock_tui = MockTui::new();
        let context = ActionContext {
            mode: AppMode::Results,
            selection_mode: crate::app_state_container::SelectionMode::Row,
            has_results: true,
            has_filter: false,
            has_search: false,
            row_count: 10,
            column_count: 5,
            current_row: 0,
            current_column: 0,
        };

        let action = Action::Navigate(NavigateAction::Up(1));
        let result = handler.handle_action(&action, &context, &mut mock_tui);

        assert!(result.is_some());
        assert_eq!(mock_tui.last_action, "previous_row");

        match result.unwrap() {
            Ok(ActionResult::Handled) => {}
            _ => panic!("Expected Handled result"),
        }
    }

    #[test]
    fn test_action_dispatcher() {
        let dispatcher = ActionDispatcher::new();
        let mut mock_tui = MockTui::new();
        let context = ActionContext {
            mode: AppMode::Results,
            selection_mode: crate::app_state_container::SelectionMode::Row,
            has_results: true,
            has_filter: false,
            has_search: false,
            row_count: 10,
            column_count: 5,
            current_row: 0,
            current_column: 0,
        };

        // Test navigation action
        let action = Action::Navigate(NavigateAction::Down(2));
        let result = dispatcher
            .dispatch(&action, &context, &mut mock_tui)
            .unwrap();

        assert_eq!(result, ActionResult::Handled);
        assert_eq!(mock_tui.last_action, "next_row"); // Called twice for count=2

        // Test export action
        let action = Action::ExportToCsv;
        let result = dispatcher
            .dispatch(&action, &context, &mut mock_tui)
            .unwrap();

        assert_eq!(result, ActionResult::Handled);
        assert_eq!(mock_tui.last_action, "export_to_csv");
    }
}
