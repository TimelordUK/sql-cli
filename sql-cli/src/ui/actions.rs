// Action system for UI operations
// This will gradually replace direct key handling in TUI

use crate::app_state_container::SelectionMode;
use crate::buffer::AppMode;
use anyhow::Result;

/// Where to position the cursor when switching to Command mode
#[derive(Debug, Clone, PartialEq)]
pub enum CursorPosition {
    /// Keep cursor at current position
    Current,
    /// Move cursor to end of input
    End,
    /// Move cursor after a specific SQL clause
    AfterClause(SqlClause),
}

/// SQL clauses that can be targeted for cursor positioning
#[derive(Debug, Clone, PartialEq)]
pub enum SqlClause {
    Select,
    From,
    Where,
    OrderBy,
    GroupBy,
    Having,
    Limit,
}

/// All possible actions that can be triggered in the UI
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Navigation actions
    Navigate(NavigateAction),

    // Mode switching
    SwitchMode(AppMode),
    SwitchModeWithCursor(AppMode, CursorPosition),
    ToggleSelectionMode,
    ExitCurrentMode,

    // Editing actions
    InsertChar(char),
    Backspace,
    Delete,
    ClearLine,
    Undo,
    Redo,

    // Cursor movement
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    MoveCursorWordLeft,
    MoveCursorWordRight,

    // Text deletion
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToLineStart,
    DeleteToLineEnd,

    // Clipboard/Yank operations
    Yank(YankTarget),
    Paste,

    // Column operations
    ToggleColumnPin,
    HideColumn,
    UnhideAllColumns,
    MoveColumnLeft,
    MoveColumnRight,
    ClearAllPins,

    // Data operations
    Sort(Option<usize>), // None = current column
    StartFilter,
    StartFuzzyFilter,
    ApplyFilter(String),
    ClearFilter,

    // Search operations
    StartSearch,
    StartColumnSearch,
    NextMatch,
    PreviousMatch,

    // Query operations
    ExecuteQuery,
    LoadFromHistory(usize),

    // View operations
    RefreshView,
    ShowHelp,
    ShowDebugInfo,
    ToggleRowNumbers,
    ToggleCompactMode,
    StartJumpToRow,

    // Application control
    Quit,
    ForceQuit,
}

/// Navigation actions with optional counts for vim-style motions
#[derive(Debug, Clone, PartialEq)]
pub enum NavigateAction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    PageUp,
    PageDown,
    Home,
    End,
    FirstColumn,
    LastColumn,
    JumpToRow(usize),
    JumpToColumn(usize),
}

/// Targets for yank operations
#[derive(Debug, Clone, PartialEq)]
pub enum YankTarget {
    Cell,
    Row,
    Column,
    All,
    Query,
}

/// Context needed to determine action availability and behavior
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub mode: AppMode,
    pub selection_mode: SelectionMode,
    pub has_results: bool,
    pub has_filter: bool,
    pub has_search: bool,
    pub row_count: usize,
    pub column_count: usize,
    pub current_row: usize,
    pub current_column: usize,
}

/// Result of handling an action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    /// Action was handled successfully
    Handled,
    /// Action was not applicable in current context
    NotHandled,
    /// Action should cause application exit
    Exit,
    /// Action requires mode switch
    SwitchMode(AppMode),
    /// Action failed with error
    Error(String),
}

/// Trait for components that can handle actions
pub trait ActionHandler {
    /// Check if this handler can process the given action in the current context
    fn can_handle(&self, action: &Action, context: &ActionContext) -> bool;

    /// Handle the action, returning the result
    fn handle(&mut self, action: Action, context: &ActionContext) -> Result<ActionResult>;
}

/// Default implementation for checking action availability
pub fn can_perform_action(action: &Action, context: &ActionContext) -> bool {
    match action {
        // Navigation is usually available in Results mode
        Action::Navigate(_) => context.mode == AppMode::Results && context.has_results,

        // Mode switching depends on current mode
        Action::ToggleSelectionMode => context.mode == AppMode::Results,
        Action::ExitCurrentMode => context.mode != AppMode::Command,

        // Yank operations need results
        Action::Yank(_) => context.has_results,

        // Filter operations
        Action::StartFilter | Action::StartFuzzyFilter => {
            context.mode == AppMode::Results || context.mode == AppMode::Command
        }
        Action::ClearFilter => context.has_filter,

        // Search operations
        Action::NextMatch | Action::PreviousMatch => context.has_search,

        // Most others depend on specific contexts
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_action_creation() {
        let action = Action::Navigate(NavigateAction::Down(5));
        assert_eq!(action, Action::Navigate(NavigateAction::Down(5)));
    }

    #[test]
    fn test_action_context() {
        let context = ActionContext {
            mode: AppMode::Results,
            selection_mode: SelectionMode::Row,
            has_results: true,
            has_filter: false,
            has_search: false,
            row_count: 100,
            column_count: 10,
            current_row: 0,
            current_column: 0,
        };

        // Navigation should be available in Results mode with results
        assert!(can_perform_action(
            &Action::Navigate(NavigateAction::Down(1)),
            &context
        ));

        // Filter clearing should not be available without active filter
        assert!(!can_perform_action(&Action::ClearFilter, &context));
    }
}
