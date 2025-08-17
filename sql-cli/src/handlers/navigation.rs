// Navigation key handler
// Handles all movement-related key events in Results mode

use crate::ui::actions::{Action, NavigateAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct NavigationHandler;

impl NavigationHandler {
    pub fn new() -> Self {
        NavigationHandler
    }

    /// Process navigation keys and convert to actions
    /// Returns Some(Action) if key was handled, None otherwise
    pub fn handle_key(&self, key: KeyEvent, mode: &crate::buffer::AppMode) -> Option<Action> {
        use crate::buffer::AppMode;

        // Only handle navigation in Results mode (for now)
        if !matches!(mode, AppMode::Results) {
            return None;
        }

        match key.code {
            // Vim-style navigation
            KeyCode::Char('h') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::Navigate(NavigateAction::Left(1)))
            }
            KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Navigate(NavigateAction::Down(1)))
            }
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Navigate(NavigateAction::Up(1)))
            }
            KeyCode::Char('l') => Some(Action::Navigate(NavigateAction::Right(1))),

            // Arrow keys
            KeyCode::Up if !key.modifiers.contains(KeyModifiers::ALT) => {
                Some(Action::Navigate(NavigateAction::Up(1)))
            }
            KeyCode::Down if !key.modifiers.contains(KeyModifiers::ALT) => {
                Some(Action::Navigate(NavigateAction::Down(1)))
            }
            KeyCode::Left
                if !key.modifiers.contains(KeyModifiers::SHIFT)
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(Action::Navigate(NavigateAction::Left(1)))
            }
            KeyCode::Right
                if !key.modifiers.contains(KeyModifiers::SHIFT)
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(Action::Navigate(NavigateAction::Right(1)))
            }

            // Page navigation
            KeyCode::PageUp => Some(Action::Navigate(NavigateAction::PageUp)),
            KeyCode::PageDown => Some(Action::Navigate(NavigateAction::PageDown)),
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Navigate(NavigateAction::PageDown))
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Navigate(NavigateAction::PageUp))
            }

            // Home/End for vertical navigation
            KeyCode::Home if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::Navigate(NavigateAction::Home))
            }
            KeyCode::End if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::Navigate(NavigateAction::End))
            }

            // g/G for top/bottom
            KeyCode::Char('g') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Navigate(NavigateAction::Home))
            }
            KeyCode::Char('G') => Some(Action::Navigate(NavigateAction::End)),

            // H, M, L for viewport navigation
            KeyCode::Char('H') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::NavigateToViewportTop)
            }
            KeyCode::Char('M') => Some(Action::NavigateToViewportMiddle),
            KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::NavigateToViewportBottom)
            }

            // ^/$ for horizontal navigation
            KeyCode::Char('^') => Some(Action::Navigate(NavigateAction::FirstColumn)),
            KeyCode::Char('$') => Some(Action::Navigate(NavigateAction::LastColumn)),

            // Tab navigation for columns
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::NextColumn)
            }
            KeyCode::BackTab | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::PreviousColumn)
            }

            // Lock modes
            KeyCode::Char('x') => Some(Action::ToggleCursorLock),
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::ToggleViewportLock)
            }

            _ => None,
        }
    }

    /// Check if a key is a navigation key that this handler manages
    pub fn is_navigation_key(&self, key: &KeyEvent, mode: &crate::buffer::AppMode) -> bool {
        self.handle_key(*key, mode).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::AppMode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_vim_navigation() {
        let handler = NavigationHandler::new();
        let mode = AppMode::Results;

        // Test h,j,k,l
        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(h_key, &mode),
            Some(Action::Navigate(NavigateAction::Left(1)))
        );

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(j_key, &mode),
            Some(Action::Navigate(NavigateAction::Down(1)))
        );

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(k_key, &mode),
            Some(Action::Navigate(NavigateAction::Up(1)))
        );

        let l_key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(l_key, &mode),
            Some(Action::Navigate(NavigateAction::Right(1)))
        );
    }

    #[test]
    fn test_arrow_keys() {
        let handler = NavigationHandler::new();
        let mode = AppMode::Results;

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(up_key, &mode),
            Some(Action::Navigate(NavigateAction::Up(1)))
        );

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(down_key, &mode),
            Some(Action::Navigate(NavigateAction::Down(1)))
        );
    }

    #[test]
    fn test_page_navigation() {
        let handler = NavigationHandler::new();
        let mode = AppMode::Results;

        let pageup_key = KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(pageup_key, &mode),
            Some(Action::Navigate(NavigateAction::PageUp))
        );

        let pagedown_key = KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty());
        assert_eq!(
            handler.handle_key(pagedown_key, &mode),
            Some(Action::Navigate(NavigateAction::PageDown))
        );
    }

    #[test]
    fn test_not_in_results_mode() {
        let handler = NavigationHandler::new();
        let mode = AppMode::Command;

        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty());
        assert_eq!(handler.handle_key(h_key, &mode), None);
    }
}
