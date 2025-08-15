// Maps keyboard input to actions
// This will gradually replace direct key handling in TUI

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

use crate::buffer::AppMode;
use crate::ui::actions::{Action, ActionContext, NavigateAction, YankTarget};

/// Maps keyboard input to actions based on context
pub struct KeyMapper {
    /// Static mappings that don't depend on mode
    global_mappings: HashMap<(KeyCode, KeyModifiers), Action>,

    /// Mode-specific mappings
    mode_mappings: HashMap<AppMode, HashMap<(KeyCode, KeyModifiers), Action>>,

    /// Vim-style count buffer for motions
    count_buffer: String,
}

impl KeyMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            global_mappings: HashMap::new(),
            mode_mappings: HashMap::new(),
            count_buffer: String::new(),
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

        // Tab to switch to Command mode
        mappings.insert((Tab, Mod::NONE), Action::SwitchMode(AppMode::Command));

        // Pinning
        mappings.insert((Char('p'), Mod::NONE), Action::ToggleColumnPin);

        // Sorting
        mappings.insert((Char('s'), Mod::NONE), Action::Sort(None));

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

        // Tab to switch back to Results mode (if results exist)
        mappings.insert((Tab, Mod::NONE), Action::SwitchMode(AppMode::Results));

        // Clear line
        mappings.insert((Char('u'), Mod::CONTROL), Action::ClearLine);

        // Undo/Redo
        mappings.insert((Char('z'), Mod::CONTROL), Action::Undo);
        mappings.insert((Char('y'), Mod::CONTROL), Action::Redo);

        self.mode_mappings.insert(AppMode::Command, mappings);
    }

    /// Map a key event to an action based on current context
    pub fn map_key(&mut self, key: KeyEvent, context: &ActionContext) -> Option<Action> {
        // Handle vim-style counts (e.g., "5j" for moving down 5 lines)
        if context.mode == AppMode::Results {
            if let KeyCode::Char(c) = key.code {
                if c.is_ascii_digit() && key.modifiers.is_empty() {
                    self.count_buffer.push(c);
                    return None; // Collecting count, no action yet
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

    /// Clear any pending state (like count buffer)
    pub fn clear_pending(&mut self) {
        self.count_buffer.clear();
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
}
