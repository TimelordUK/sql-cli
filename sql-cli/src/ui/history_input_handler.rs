//! History input handler operations
//!
//! This module contains the logic for handling Ctrl+R history search functionality,
//! extracted from the monolithic TUI to improve maintainability and testability.

use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, BufferAPI};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

/// Context for history input operations
/// Provides the minimal interface needed for history search operations
pub struct HistoryInputContext<'a> {
    pub state_container: &'a Arc<AppStateContainer>,
    pub buffer: &'a mut dyn BufferAPI,
}

/// Result of processing a history input key event
#[derive(Debug, Clone, PartialEq)]
pub enum HistoryInputResult {
    /// Continue in history mode
    Continue,
    /// Exit the application (Ctrl+C)
    Exit,
    /// Switch back to command mode, optionally with input text and cursor position
    SwitchToCommand(Option<(String, usize)>),
}

/// Handle a key event in history search mode
/// Returns the result of processing the key event
pub fn handle_history_input(
    ctx: &mut HistoryInputContext,
    key: KeyEvent,
) -> HistoryInputResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            HistoryInputResult::Exit
        }
        KeyCode::Esc => {
            // Cancel history search and restore original input
            let original_input = ctx.state_container.cancel_history_search();
            ctx.buffer.set_mode(AppMode::Command);
            ctx.buffer.set_status_message("History search cancelled".to_string());
            HistoryInputResult::SwitchToCommand(Some((original_input, 0)))
        }
        KeyCode::Enter => {
            // Accept the selected history command
            if let Some(command) = ctx.state_container.accept_history_search() {
                ctx.buffer.set_mode(AppMode::Command);
                ctx.buffer.set_status_message(
                    "Command loaded from history (cursor at start)".to_string(),
                );
                // Return command with cursor at the beginning for better visibility
                HistoryInputResult::SwitchToCommand(Some((command, 0)))
            } else {
                HistoryInputResult::Continue
            }
        }
        KeyCode::Up => {
            ctx.state_container.history_search_previous();
            HistoryInputResult::Continue
        }
        KeyCode::Down => {
            ctx.state_container.history_search_next();
            HistoryInputResult::Continue
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+R cycles through matches
            ctx.state_container.history_search_next();
            HistoryInputResult::Continue
        }
        KeyCode::Backspace => {
            ctx.state_container.history_search_backspace();
            HistoryInputResult::Continue
        }
        KeyCode::Char(c) => {
            ctx.state_container.history_search_add_char(c);
            HistoryInputResult::Continue
        }
        _ => HistoryInputResult::Continue,
    }
}

/// Update history matches with schema context
/// This is a separate function that the TUI can call when needed
pub fn should_update_history_matches(result: &HistoryInputResult) -> bool {
    match result {
        HistoryInputResult::Continue => true,
        _ => false,
    }
}

/// Check if the key event would cause a history search update
pub fn key_updates_search(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Backspace | KeyCode::Char(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state_container::AppStateContainer;
    use crate::buffer::{AppMode, Buffer};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_context() -> (AppStateContainer, Buffer) {
        let state_container = AppStateContainer::new();
        let buffer = Buffer::new();
        (state_container, buffer)
    }

    #[test]
    fn test_ctrl_c_exits() {
        let (state_container, mut buffer) = create_test_context();
        let state_arc = Arc::new(state_container);
        
        let mut ctx = HistoryInputContext {
            state_container: &state_arc,
            buffer: &mut buffer,
        };

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let result = handle_history_input(&mut ctx, key);
        
        assert_eq!(result, HistoryInputResult::Exit);
    }

    #[test]
    fn test_esc_cancels_search() {
        let (state_container, mut buffer) = create_test_context();
        let state_arc = Arc::new(state_container);
        
        let mut ctx = HistoryInputContext {
            state_container: &state_arc,
            buffer: &mut buffer,
        };

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = handle_history_input(&mut ctx, key);
        
        // Should switch to command mode with original input
        match result {
            HistoryInputResult::SwitchToCommand(input) => {
                assert!(input.is_some());
                assert_eq!(ctx.buffer.get_mode(), AppMode::Command);
            }
            _ => panic!("Expected SwitchToCommand result"),
        }
    }

    #[test]
    fn test_up_down_navigation() {
        let (state_container, mut buffer) = create_test_context();
        let state_arc = Arc::new(state_container);
        
        let mut ctx = HistoryInputContext {
            state_container: &state_arc,
            buffer: &mut buffer,
        };

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let result = handle_history_input(&mut ctx, up_key);
        assert_eq!(result, HistoryInputResult::Continue);

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let result = handle_history_input(&mut ctx, down_key);
        assert_eq!(result, HistoryInputResult::Continue);
    }

    #[test]
    fn test_ctrl_r_navigation() {
        let (state_container, mut buffer) = create_test_context();
        let state_arc = Arc::new(state_container);
        
        let mut ctx = HistoryInputContext {
            state_container: &state_arc,
            buffer: &mut buffer,
        };

        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let result = handle_history_input(&mut ctx, key);
        assert_eq!(result, HistoryInputResult::Continue);
    }

    #[test]
    fn test_character_input() {
        let (state_container, mut buffer) = create_test_context();
        let state_arc = Arc::new(state_container);
        
        let mut ctx = HistoryInputContext {
            state_container: &state_arc,
            buffer: &mut buffer,
        };

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        let result = handle_history_input(&mut ctx, key);
        assert_eq!(result, HistoryInputResult::Continue);
    }

    #[test]
    fn test_key_updates_search() {
        let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(key_updates_search(backspace_key));
        
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(key_updates_search(char_key));
        
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!key_updates_search(up_key));
    }

    #[test]
    fn test_should_update_history_matches() {
        assert!(should_update_history_matches(&HistoryInputResult::Continue));
        assert!(!should_update_history_matches(&HistoryInputResult::Exit));
        assert!(!should_update_history_matches(&HistoryInputResult::SwitchToCommand(None)));
    }
}