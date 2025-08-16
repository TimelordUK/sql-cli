// Maps keyboard input to actions
// This will gradually replace direct key handling in TUI

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

use crate::buffer::AppMode;
use crate::ui::actions::{
    Action, ActionContext, CursorPosition, NavigateAction, SqlClause, YankTarget,
};

/// Maps keyboard input to actions based on context
pub struct KeyMapper {
    /// Static mappings that don't depend on mode
    global_mappings: HashMap<(KeyCode, KeyModifiers), Action>,

    /// Mode-specific mappings
    mode_mappings: HashMap<AppMode, HashMap<(KeyCode, KeyModifiers), Action>>,

    /// Vim-style count buffer for motions
    count_buffer: String,

    /// Buffer for multi-character vim commands (e.g., 'wa', 'oa')
    vim_command_buffer: String,
}

impl KeyMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            global_mappings: HashMap::new(),
            mode_mappings: HashMap::new(),
            count_buffer: String::new(),
            vim_command_buffer: String::new(),
        };

        mapper.init_global_mappings();
        mapper.init_mode_mappings();
        mapper
    }

    /// Initialize mappings that work regardless of mode
    fn init_global_mappings(&mut self) {
        use KeyCode::*;
        use KeyModifiers as Mod;

        // Help is always available
        self.global_mappings
            .insert((F(1), Mod::NONE), Action::ShowHelp);

        // Debug info
        self.global_mappings
            .insert((F(5), Mod::NONE), Action::ShowDebugInfo);

        // Force quit
        self.global_mappings
            .insert((Char('c'), Mod::CONTROL), Action::ForceQuit);
        self.global_mappings
            .insert((Char('C'), Mod::CONTROL), Action::ForceQuit);
    }

    /// Initialize mode-specific mappings
    fn init_mode_mappings(&mut self) {
        self.init_results_mappings();
        self.init_command_mappings();
        // Add other modes as we migrate them
    }

    /// Initialize Results mode mappings
    fn init_results_mappings(&mut self) {
        use crate::buffer::AppMode;
        use KeyCode::*;
        use KeyModifiers as Mod;

        let mut mappings = HashMap::new();

        // Basic navigation (will be extracted in Phase 2)
        mappings.insert((Up, Mod::NONE), Action::Navigate(NavigateAction::Up(1)));
        mappings.insert((Down, Mod::NONE), Action::Navigate(NavigateAction::Down(1)));
        mappings.insert((Left, Mod::NONE), Action::Navigate(NavigateAction::Left(1)));
        mappings.insert(
            (Right, Mod::NONE),
            Action::Navigate(NavigateAction::Right(1)),
        );

        mappings.insert(
            (PageUp, Mod::NONE),
            Action::Navigate(NavigateAction::PageUp),
        );
        mappings.insert(
            (PageDown, Mod::NONE),
            Action::Navigate(NavigateAction::PageDown),
        );
        mappings.insert((Home, Mod::NONE), Action::Navigate(NavigateAction::Home));
        mappings.insert((End, Mod::NONE), Action::Navigate(NavigateAction::End));

        // Vim navigation
        mappings.insert(
            (Char('h'), Mod::NONE),
            Action::Navigate(NavigateAction::Left(1)),
        );
        mappings.insert(
            (Char('j'), Mod::NONE),
            Action::Navigate(NavigateAction::Down(1)),
        );
        mappings.insert(
            (Char('k'), Mod::NONE),
            Action::Navigate(NavigateAction::Up(1)),
        );
        mappings.insert(
            (Char('l'), Mod::NONE),
            Action::Navigate(NavigateAction::Right(1)),
        );

        // Selection mode toggle
        mappings.insert((Char('v'), Mod::NONE), Action::ToggleSelectionMode);

        // Mode switching
        mappings.insert((Esc, Mod::NONE), Action::ExitCurrentMode);
        mappings.insert((Char('q'), Mod::NONE), Action::Quit);

        // F2 to switch to Command mode
        mappings.insert((F(2), Mod::NONE), Action::SwitchMode(AppMode::Command));

        // Vim-style 'i' for insert/input mode (switch to Command at current position)
        mappings.insert(
            (Char('i'), Mod::NONE),
            Action::SwitchModeWithCursor(AppMode::Command, CursorPosition::Current),
        );

        // Vim-style 'a' for append mode (switch to Command at end)
        mappings.insert(
            (Char('a'), Mod::NONE),
            Action::SwitchModeWithCursor(AppMode::Command, CursorPosition::End),
        );

        // Column operations
        mappings.insert((Char('p'), Mod::NONE), Action::ToggleColumnPin);
        mappings.insert((Char('-'), Mod::NONE), Action::HideColumn); // '-' to hide column
        mappings.insert(
            (Char('H'), Mod::CONTROL | Mod::SHIFT),
            Action::UnhideAllColumns,
        );
        mappings.insert((Char('+'), Mod::NONE), Action::UnhideAllColumns); // '+' to unhide all
        mappings.insert((Char('='), Mod::NONE), Action::UnhideAllColumns); // '=' to unhide all (easier than shift+= for +)
                                                                           // Handle both lowercase and uppercase 'e' for hide empty columns
        mappings.insert((Char('e'), Mod::NONE), Action::HideEmptyColumns);
        mappings.insert((Char('E'), Mod::SHIFT), Action::HideEmptyColumns);
        mappings.insert((Left, Mod::SHIFT), Action::MoveColumnLeft);
        mappings.insert((Right, Mod::SHIFT), Action::MoveColumnRight);
        // Also support < and > characters for column movement (more intuitive)
        mappings.insert((Char('<'), Mod::NONE), Action::MoveColumnLeft);
        mappings.insert((Char('>'), Mod::NONE), Action::MoveColumnRight);
        mappings.insert((Char('/'), Mod::NONE), Action::StartColumnSearch);

        // Sorting
        mappings.insert((Char('s'), Mod::NONE), Action::Sort(None));

        // View toggles
        mappings.insert((Char('N'), Mod::NONE), Action::ToggleRowNumbers);
        mappings.insert((Char('C'), Mod::NONE), Action::ToggleCompactMode);

        // Jump to row
        mappings.insert((Char(':'), Mod::NONE), Action::StartJumpToRow);

        self.mode_mappings.insert(AppMode::Results, mappings);
    }

    /// Initialize Command mode mappings
    fn init_command_mappings(&mut self) {
        use crate::buffer::AppMode;
        use KeyCode::*;
        use KeyModifiers as Mod;

        let mut mappings = HashMap::new();

        // Execute query
        mappings.insert((Enter, Mod::NONE), Action::ExecuteQuery);

        // F2 to switch back to Results mode (if results exist)
        mappings.insert((F(2), Mod::NONE), Action::SwitchMode(AppMode::Results));

        // Clear line
        mappings.insert((Char('u'), Mod::CONTROL), Action::ClearLine);

        // Undo/Redo
        mappings.insert((Char('z'), Mod::CONTROL), Action::Undo);
        mappings.insert((Char('y'), Mod::CONTROL), Action::Redo);

        // Cursor movement
        mappings.insert((Left, Mod::NONE), Action::MoveCursorLeft);
        mappings.insert((Right, Mod::NONE), Action::MoveCursorRight);
        mappings.insert((Home, Mod::NONE), Action::MoveCursorHome);
        mappings.insert((End, Mod::NONE), Action::MoveCursorEnd);
        mappings.insert((Char('a'), Mod::CONTROL), Action::MoveCursorHome);
        mappings.insert((Char('e'), Mod::CONTROL), Action::MoveCursorEnd);
        mappings.insert((Left, Mod::CONTROL), Action::MoveCursorWordLeft);
        mappings.insert((Right, Mod::CONTROL), Action::MoveCursorWordRight);
        mappings.insert((Char('b'), Mod::ALT), Action::MoveCursorWordLeft);
        mappings.insert((Char('f'), Mod::ALT), Action::MoveCursorWordRight);

        // Text editing
        mappings.insert((Backspace, Mod::NONE), Action::Backspace);
        mappings.insert((Delete, Mod::NONE), Action::Delete);
        mappings.insert((Char('w'), Mod::CONTROL), Action::DeleteWordBackward);
        mappings.insert((Char('d'), Mod::ALT), Action::DeleteWordForward);
        mappings.insert((Char('k'), Mod::CONTROL), Action::DeleteToLineEnd);
        mappings.insert((F(9), Mod::NONE), Action::DeleteToLineEnd); // F9 alternative
        mappings.insert((F(10), Mod::NONE), Action::DeleteToLineStart); // F10 alternative

        // Clipboard operations
        mappings.insert((Char('v'), Mod::CONTROL), Action::Paste);

        self.mode_mappings.insert(AppMode::Command, mappings);
    }

    /// Map a key event to an action based on current context
    pub fn map_key(&mut self, key: KeyEvent, context: &ActionContext) -> Option<Action> {
        // Handle vim-style counts and commands in Results mode
        if context.mode == AppMode::Results {
            if let KeyCode::Char(c) = key.code {
                if key.modifiers.is_empty() {
                    // Check if we're building a vim command
                    if !self.vim_command_buffer.is_empty() {
                        // We have a pending command, check for valid combinations
                        let command = format!("{}{}", self.vim_command_buffer, c);
                        let action = match command.as_str() {
                            "wa" => {
                                // Append after WHERE clause
                                self.vim_command_buffer.clear();
                                Some(Action::SwitchModeWithCursor(
                                    AppMode::Command,
                                    CursorPosition::AfterClause(SqlClause::Where),
                                ))
                            }
                            "oa" => {
                                // Append after ORDER BY clause
                                self.vim_command_buffer.clear();
                                Some(Action::SwitchModeWithCursor(
                                    AppMode::Command,
                                    CursorPosition::AfterClause(SqlClause::OrderBy),
                                ))
                            }
                            "sa" => {
                                // Append after SELECT clause
                                self.vim_command_buffer.clear();
                                Some(Action::SwitchModeWithCursor(
                                    AppMode::Command,
                                    CursorPosition::AfterClause(SqlClause::Select),
                                ))
                            }
                            "ga" => {
                                // Append after GROUP BY clause
                                self.vim_command_buffer.clear();
                                Some(Action::SwitchModeWithCursor(
                                    AppMode::Command,
                                    CursorPosition::AfterClause(SqlClause::GroupBy),
                                ))
                            }
                            _ => {
                                // Invalid command, clear buffer
                                self.vim_command_buffer.clear();
                                None
                            }
                        };

                        if action.is_some() {
                            return action;
                        }
                    }

                    // Check if this starts a vim command sequence
                    if matches!(c, 'w' | 'o' | 's' | 'g') {
                        self.vim_command_buffer.push(c);
                        return None; // Collecting command, no action yet
                    }

                    // Check for digits (vim counts)
                    if c.is_ascii_digit() {
                        self.count_buffer.push(c);
                        return None; // Collecting count, no action yet
                    }
                }
            }
        }

        // Check for action with count
        let action = self.map_key_internal(key, context);

        // Apply count if we have one
        if !self.count_buffer.is_empty() {
            if let Some(mut action) = action {
                if let Ok(count) = self.count_buffer.parse::<usize>() {
                    action = self.apply_count_to_action(action, count);
                }
                self.count_buffer.clear();
                return Some(action);
            }
            // If no action, clear count buffer
            self.count_buffer.clear();
        }

        action
    }

    /// Internal key mapping without count handling
    fn map_key_internal(&self, key: KeyEvent, context: &ActionContext) -> Option<Action> {
        let key_combo = (key.code, key.modifiers);

        // Check global mappings first
        if let Some(action) = self.global_mappings.get(&key_combo) {
            return Some(action.clone());
        }

        // Check mode-specific mappings
        if let Some(mode_mappings) = self.mode_mappings.get(&context.mode) {
            if let Some(action) = mode_mappings.get(&key_combo) {
                return Some(action.clone());
            }
        }

        // Handle regular character input in Command mode
        if context.mode == AppMode::Command {
            if let KeyCode::Char(c) = key.code {
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    // Regular character input
                    return Some(Action::InsertChar(c));
                }
            }
        }

        // No mapping found
        None
    }

    /// Apply a count to an action (for vim-style motions)
    fn apply_count_to_action(&self, action: Action, count: usize) -> Action {
        match action {
            Action::Navigate(NavigateAction::Up(_)) => Action::Navigate(NavigateAction::Up(count)),
            Action::Navigate(NavigateAction::Down(_)) => {
                Action::Navigate(NavigateAction::Down(count))
            }
            Action::Navigate(NavigateAction::Left(_)) => {
                Action::Navigate(NavigateAction::Left(count))
            }
            Action::Navigate(NavigateAction::Right(_)) => {
                Action::Navigate(NavigateAction::Right(count))
            }
            // Other actions don't support counts yet
            _ => action,
        }
    }

    /// Clear any pending state (like count buffer and vim command buffer)
    pub fn clear_pending(&mut self) {
        self.count_buffer.clear();
        self.vim_command_buffer.clear();
    }

    /// Check if we're collecting a count
    pub fn is_collecting_count(&self) -> bool {
        !self.count_buffer.is_empty()
    }

    /// Get the current count buffer for display
    pub fn get_count_buffer(&self) -> &str {
        &self.count_buffer
    }
}

impl Default for KeyMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state_container::SelectionMode;

    #[test]
    fn test_basic_navigation_mapping() {
        let mut mapper = KeyMapper::new();
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

        // Test arrow down
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Navigate(NavigateAction::Down(1))));

        // Test vim j
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Navigate(NavigateAction::Down(1))));
    }

    #[test]
    fn test_vim_count_motion() {
        let mut mapper = KeyMapper::new();
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

        // Type "5"
        let key = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, None); // No action yet, collecting count
        assert_eq!(mapper.get_count_buffer(), "5");

        // Type "j"
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Navigate(NavigateAction::Down(5))));
        assert_eq!(mapper.get_count_buffer(), ""); // Buffer cleared
    }

    #[test]
    fn test_global_mapping_override() {
        let mut mapper = KeyMapper::new();
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

        // F1 should work in any mode
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::ShowHelp));
    }

    #[test]
    fn test_command_mode_editing_actions() {
        let mut mapper = KeyMapper::new();
        let context = ActionContext {
            mode: AppMode::Command,
            selection_mode: SelectionMode::Row,
            has_results: false,
            has_filter: false,
            has_search: false,
            row_count: 0,
            column_count: 0,
            current_row: 0,
            current_column: 0,
        };

        // Test character input
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::InsertChar('a')));

        // Test uppercase character
        let key = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::InsertChar('A')));

        // Test backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Backspace));

        // Test delete
        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Delete));

        // Test cursor movement - left
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorLeft));

        // Test cursor movement - right
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorRight));

        // Test Ctrl+A (home)
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorHome));

        // Test Ctrl+E (end)
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorEnd));

        // Test Ctrl+U (clear line)
        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::ClearLine));

        // Test Ctrl+W (delete word backward)
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::DeleteWordBackward));

        // Test Ctrl+Z (undo)
        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Undo));

        // Test Enter (execute query)
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::ExecuteQuery));
    }

    #[test]
    fn test_vim_style_append_modes() {
        let mut mapper = KeyMapper::new();
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

        // Test 'i' for insert at current
        let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(
            action,
            Some(Action::SwitchModeWithCursor(
                AppMode::Command,
                CursorPosition::Current
            ))
        );

        // Test 'a' for append at end
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(
            action,
            Some(Action::SwitchModeWithCursor(
                AppMode::Command,
                CursorPosition::End
            ))
        );

        // Test 'wa' for append after WHERE
        let key_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        let action_w = mapper.map_key(key_w, &context);
        assert_eq!(action_w, None); // 'w' starts collecting command

        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action_wa = mapper.map_key(key_a, &context);
        assert_eq!(
            action_wa,
            Some(Action::SwitchModeWithCursor(
                AppMode::Command,
                CursorPosition::AfterClause(SqlClause::Where)
            ))
        );

        // Reset mapper for next test
        mapper.clear_pending();

        // Test 'oa' for append after ORDER BY
        let key_o = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
        let action_o = mapper.map_key(key_o, &context);
        assert_eq!(action_o, None); // 'o' starts collecting command

        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action_oa = mapper.map_key(key_a, &context);
        assert_eq!(
            action_oa,
            Some(Action::SwitchModeWithCursor(
                AppMode::Command,
                CursorPosition::AfterClause(SqlClause::OrderBy)
            ))
        );
    }
}
