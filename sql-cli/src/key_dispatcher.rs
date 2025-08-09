use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Represents a key binding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }

    pub fn with_ctrl(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::CONTROL,
        }
    }

    pub fn with_alt(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::ALT,
        }
    }

    pub fn with_shift(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::SHIFT,
        }
    }

    pub fn from_event(event: &KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Simple key dispatcher that maps keys to action names
pub struct KeyDispatcher {
    // Mode-specific key maps
    command_map: HashMap<KeyBinding, String>,
    results_map: HashMap<KeyBinding, String>,
    search_map: HashMap<KeyBinding, String>,
    filter_map: HashMap<KeyBinding, String>,
    help_map: HashMap<KeyBinding, String>,
    debug_map: HashMap<KeyBinding, String>,
}

impl KeyDispatcher {
    pub fn new() -> Self {
        let mut dispatcher = Self {
            command_map: HashMap::new(),
            results_map: HashMap::new(),
            search_map: HashMap::new(),
            filter_map: HashMap::new(),
            help_map: HashMap::new(),
            debug_map: HashMap::new(),
        };
        dispatcher.setup_default_bindings();
        dispatcher
    }

    fn setup_default_bindings(&mut self) {
        // Command mode bindings
        self.setup_command_bindings();

        // Results mode bindings
        self.setup_results_bindings();

        // Search/Filter mode bindings
        self.setup_search_bindings();
        self.setup_filter_bindings();

        // Help/Debug mode bindings
        self.setup_help_bindings();
        self.setup_debug_bindings();
    }

    fn setup_command_bindings(&mut self) {
        // Basic navigation and editing
        self.command_map
            .insert(KeyBinding::new(KeyCode::Enter), "execute_query".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::Tab), "handle_completion".into());
        self.command_map.insert(
            KeyBinding::new(KeyCode::Backspace),
            "delete_char_backward".into(),
        );
        self.command_map.insert(
            KeyBinding::new(KeyCode::Delete),
            "delete_char_forward".into(),
        );
        self.command_map
            .insert(KeyBinding::new(KeyCode::Left), "move_cursor_left".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::Right), "move_cursor_right".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::Home), "move_to_line_start".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::End), "move_to_line_end".into());

        // Control combinations
        self.command_map
            .insert(KeyBinding::with_ctrl(KeyCode::Char('c')), "quit".into());
        self.command_map
            .insert(KeyBinding::with_ctrl(KeyCode::Char('d')), "quit".into());
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('x')),
            "expand_asterisk".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('r')),
            "search_history".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('p')),
            "previous_history".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('n')),
            "next_history".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('a')),
            "move_to_line_start".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('e')),
            "move_to_line_end".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('w')),
            "delete_word_backward".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('k')),
            "kill_line".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('u')),
            "kill_line_backward".into(),
        );
        self.command_map
            .insert(KeyBinding::with_ctrl(KeyCode::Char('y')), "yank".into());
        self.command_map
            .insert(KeyBinding::with_ctrl(KeyCode::Char('z')), "undo".into());
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('v')),
            "paste_from_clipboard".into(),
        );
        self.command_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('6')),
            "quick_switch_buffer".into(),
        );

        // Alt+number for buffer switching (1-9)
        for i in 1..=9 {
            let digit_char = char::from_digit(i, 10).unwrap();
            self.command_map.insert(
                KeyBinding::with_alt(KeyCode::Char(digit_char)),
                format!("switch_to_buffer_{}", i),
            );
        }

        // Alt combinations
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('d')),
            "delete_word_forward".into(),
        );
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('b')),
            "list_buffers".into(), // Changed from move_word_backward to list_buffers
        );
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('f')),
            "move_word_forward".into(),
        );
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('n')),
            "new_buffer".into(),
        );
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('w')),
            "close_buffer".into(),
        );
        self.command_map
            .insert(KeyBinding::with_alt(KeyCode::Tab), "next_buffer".into());
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char('[')),
            "jump_to_prev_token".into(),
        );
        self.command_map.insert(
            KeyBinding::with_alt(KeyCode::Char(']')),
            "jump_to_next_token".into(),
        );

        // Function keys
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(1)), "toggle_help".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(5)), "toggle_debug".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(6)), "show_pretty_query".into());
        self.command_map.insert(
            KeyBinding::new(KeyCode::F(8)),
            "toggle_case_insensitive".into(),
        );
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(9)), "kill_line".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(10)), "kill_line_backward".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(11)), "previous_buffer".into());
        self.command_map
            .insert(KeyBinding::new(KeyCode::F(12)), "next_buffer".into());

        // Navigation to results
        self.command_map
            .insert(KeyBinding::new(KeyCode::Down), "enter_results_mode".into());
        self.command_map.insert(
            KeyBinding::new(KeyCode::PageDown),
            "enter_results_mode".into(),
        );
    }

    fn setup_results_bindings(&mut self) {
        // Exit and navigation
        self.results_map
            .insert(KeyBinding::new(KeyCode::Esc), "exit_results_mode".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Up), "exit_results_mode".into());
        self.results_map
            .insert(KeyBinding::with_ctrl(KeyCode::Char('c')), "quit".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('q')), "quit".into());

        // Row navigation
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('j')), "next_row".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Down), "next_row".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('k')), "previous_row".into());

        // Column navigation
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('h')),
            "move_column_left".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Left), "move_column_left".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('l')),
            "move_column_right".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Right), "move_column_right".into());

        // Jump navigation
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('g')), "goto_first_row".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('G')), "goto_last_row".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('^')),
            "goto_first_column".into(),
        );
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('0')),
            "goto_first_column".into(),
        );
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('$')),
            "goto_last_column".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::PageUp), "page_up".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::PageDown), "page_down".into());

        // Features
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('/')), "start_search".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('\\')),
            "start_column_search".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('F')), "start_filter".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('f')),
            "start_fuzzy_filter".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('s')), "sort_by_column".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('S')),
            "show_column_stats".into(),
        );
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('n')),
            "next_search_match".into(),
        );
        self.results_map.insert(
            KeyBinding::with_shift(KeyCode::Char('N')),
            "previous_search_match".into(),
        );

        // Display options
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('C')),
            "toggle_compact_mode".into(),
        );
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('N')),
            "toggle_row_numbers".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char(':')), "jump_to_row".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('p')), "pin_column".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('P')), "clear_pins".into());

        // Selection and clipboard
        self.results_map.insert(
            KeyBinding::new(KeyCode::Char('v')),
            "toggle_selection_mode".into(),
        );
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('y')), "handle_yank".into()); // Will check selection mode

        // Export
        self.results_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('e')),
            "export_to_csv".into(),
        );
        self.results_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('j')),
            "export_to_json".into(),
        );

        // Debug/Help
        self.results_map
            .insert(KeyBinding::new(KeyCode::F(1)), "toggle_help".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::Char('?')), "toggle_help".into());
        self.results_map
            .insert(KeyBinding::new(KeyCode::F(5)), "toggle_debug".into());
        self.results_map.insert(
            KeyBinding::new(KeyCode::F(8)),
            "toggle_case_insensitive".into(),
        );
    }

    fn setup_search_bindings(&mut self) {
        self.search_map
            .insert(KeyBinding::new(KeyCode::Enter), "apply_search".into());
        self.search_map
            .insert(KeyBinding::new(KeyCode::Esc), "cancel_search".into());
        self.search_map.insert(
            KeyBinding::new(KeyCode::Backspace),
            "delete_char_backward".into(),
        );
        self.search_map.insert(
            KeyBinding::new(KeyCode::Delete),
            "delete_char_forward".into(),
        );
        self.search_map
            .insert(KeyBinding::new(KeyCode::Left), "move_cursor_left".into());
        self.search_map
            .insert(KeyBinding::new(KeyCode::Right), "move_cursor_right".into());
        self.search_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('u')),
            "clear_input".into(),
        );
    }

    fn setup_filter_bindings(&mut self) {
        self.filter_map
            .insert(KeyBinding::new(KeyCode::Enter), "apply_filter".into());
        self.filter_map
            .insert(KeyBinding::new(KeyCode::Esc), "cancel_filter".into());
        self.filter_map.insert(
            KeyBinding::new(KeyCode::Backspace),
            "delete_char_backward".into(),
        );
        self.filter_map.insert(
            KeyBinding::new(KeyCode::Delete),
            "delete_char_forward".into(),
        );
        self.filter_map
            .insert(KeyBinding::new(KeyCode::Left), "move_cursor_left".into());
        self.filter_map
            .insert(KeyBinding::new(KeyCode::Right), "move_cursor_right".into());
        self.filter_map.insert(
            KeyBinding::with_ctrl(KeyCode::Char('u')),
            "clear_input".into(),
        );
    }

    fn setup_help_bindings(&mut self) {
        self.help_map
            .insert(KeyBinding::new(KeyCode::Esc), "exit_help".into());
        self.help_map
            .insert(KeyBinding::new(KeyCode::Char('q')), "exit_help".into());
        self.help_map
            .insert(KeyBinding::new(KeyCode::Down), "scroll_help_down".into());
        self.help_map
            .insert(KeyBinding::new(KeyCode::Up), "scroll_help_up".into());
        self.help_map
            .insert(KeyBinding::new(KeyCode::PageDown), "help_page_down".into());
        self.help_map
            .insert(KeyBinding::new(KeyCode::PageUp), "help_page_up".into());
    }

    fn setup_debug_bindings(&mut self) {
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Esc), "exit_debug".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Enter), "exit_debug".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Char('q')), "exit_debug".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Down), "scroll_debug_down".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Up), "scroll_debug_up".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::PageDown), "debug_page_down".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::PageUp), "debug_page_up".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::Home), "debug_go_to_top".into());
        self.debug_map
            .insert(KeyBinding::new(KeyCode::End), "debug_go_to_bottom".into());
        // Vim-style navigation
        self.debug_map.insert(
            KeyBinding::new(KeyCode::Char('j')),
            "scroll_debug_down".into(),
        );
        self.debug_map.insert(
            KeyBinding::new(KeyCode::Char('k')),
            "scroll_debug_up".into(),
        );
        self.debug_map.insert(
            KeyBinding::new(KeyCode::Char('g')),
            "debug_go_to_top".into(),
        );
        self.debug_map.insert(
            KeyBinding::new(KeyCode::Char('G')),
            "debug_go_to_bottom".into(),
        );
    }

    /// Get action for a key in command mode
    pub fn get_command_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.command_map.get(&binding).map(|s| s.as_str())
    }

    /// Get action for a key in results mode
    pub fn get_results_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.results_map.get(&binding).map(|s| s.as_str())
    }

    /// Get action for a key in search mode
    pub fn get_search_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.search_map.get(&binding).map(|s| s.as_str())
    }

    /// Get action for a key in filter mode
    pub fn get_filter_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.filter_map.get(&binding).map(|s| s.as_str())
    }

    /// Get action for a key in help mode
    pub fn get_help_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.help_map.get(&binding).map(|s| s.as_str())
    }

    /// Get action for a key in debug mode
    pub fn get_debug_action(&self, key: &KeyEvent) -> Option<&str> {
        let binding = KeyBinding::from_event(key);
        self.debug_map.get(&binding).map(|s| s.as_str())
    }

    /// Load custom bindings from config (future feature)
    pub fn load_custom_bindings(&mut self, mode: &str, bindings: HashMap<String, String>) {
        let map = match mode {
            "command" => &mut self.command_map,
            "results" => &mut self.results_map,
            "search" => &mut self.search_map,
            "filter" => &mut self.filter_map,
            "help" => &mut self.help_map,
            "debug" => &mut self.debug_map,
            _ => return,
        };

        // Parse and add custom bindings
        // Format: "Ctrl+X" -> "expand_asterisk"
        // This would parse the key string and create the binding
    }
}
