// Simple test to verify action system is working
#[cfg(test)]
mod tests {
    use sql_cli::ui::key_mapper::KeyMapper;
    use sql_cli::ui::actions::{Action, ActionContext, NavigateAction};
    use sql_cli::buffer::AppMode;
    use sql_cli::app_state_container::SelectionMode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_editing_actions() {
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

        // Test backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::Backspace));

        // Test cursor movement
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorLeft));

        // Test Ctrl+A (home)
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::MoveCursorHome));

        // Test Ctrl+U (clear line)
        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let action = mapper.map_key(key, &context);
        assert_eq!(action, Some(Action::ClearLine));

        println!("✓ All editing action mappings work correctly");
    }
}
