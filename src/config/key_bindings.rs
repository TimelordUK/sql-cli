use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Type alias for action callbacks
pub type ActionCallback = Box<dyn Fn(&mut dyn ActionHandler) -> bool>;

/// Trait that the TUI must implement to handle actions
pub trait ActionHandler {
    // Command mode actions
    fn execute_query(&mut self) -> bool;
    fn toggle_multiline(&mut self) -> bool;
    fn expand_select_star(&mut self) -> bool;
    fn search_history(&mut self) -> bool;
    fn previous_history(&mut self) -> bool;
    fn next_history(&mut self) -> bool;
    fn autocomplete(&mut self) -> bool;
    fn toggle_help(&mut self) -> bool;
    fn toggle_debug(&mut self) -> bool;
    fn toggle_case_insensitive(&mut self) -> bool;
    fn enter_results_mode(&mut self) -> bool;
    fn exit_app(&mut self) -> bool;

    // Buffer management
    fn next_buffer(&mut self) -> bool;
    fn previous_buffer(&mut self) -> bool;
    fn quick_switch_buffer(&mut self) -> bool;
    fn new_buffer(&mut self) -> bool;
    fn close_buffer(&mut self) -> bool;
    fn list_buffers(&mut self) -> bool;

    // Text editing
    fn move_cursor_left(&mut self) -> bool;
    fn move_cursor_right(&mut self) -> bool;
    fn move_cursor_up(&mut self) -> bool;
    fn move_cursor_down(&mut self) -> bool;
    fn move_to_line_start(&mut self) -> bool;
    fn move_to_line_end(&mut self) -> bool;
    fn move_word_backward(&mut self) -> bool;
    fn move_word_forward(&mut self) -> bool;
    fn delete_char_backward(&mut self) -> bool;
    fn delete_char_forward(&mut self) -> bool;
    fn delete_word_backward(&mut self) -> bool;
    fn delete_word_forward(&mut self) -> bool;
    fn kill_line_forward(&mut self) -> bool;
    fn kill_line_backward(&mut self) -> bool;
    fn yank(&mut self) -> bool;
    fn undo(&mut self) -> bool;
    fn redo(&mut self) -> bool;
    fn insert_char(&mut self, c: char) -> bool;

    // Results mode actions
    fn move_row_up(&mut self) -> bool;
    fn move_row_down(&mut self) -> bool;
    fn move_column_left(&mut self) -> bool;
    fn move_column_right(&mut self) -> bool;
    fn page_up(&mut self) -> bool;
    fn page_down(&mut self) -> bool;
    fn go_to_first_row(&mut self) -> bool;
    fn go_to_last_row(&mut self) -> bool;
    fn go_to_first_column(&mut self) -> bool;
    fn go_to_last_column(&mut self) -> bool;
    fn toggle_compact_mode(&mut self) -> bool;
    fn toggle_row_numbers(&mut self) -> bool;
    fn jump_to_row(&mut self) -> bool;
    fn pin_column(&mut self) -> bool;
    fn clear_pins(&mut self) -> bool;
    fn start_search(&mut self) -> bool;
    fn start_column_search(&mut self) -> bool;
    fn start_filter(&mut self) -> bool;
    fn start_fuzzy_filter(&mut self) -> bool;
    fn next_search_result(&mut self) -> bool;
    fn previous_search_result(&mut self) -> bool;
    fn sort_by_column(&mut self) -> bool;
    fn show_column_stats(&mut self) -> bool;
    fn toggle_selection_mode(&mut self) -> bool;
    fn yank_selection(&mut self) -> bool;
    fn export_csv(&mut self) -> bool;
    fn export_json(&mut self) -> bool;
    fn back_to_command(&mut self) -> bool;

    // Search/Filter mode actions
    fn apply_search(&mut self) -> bool;
    fn cancel_search(&mut self) -> bool;
}

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

    pub fn with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn from_event(event: &KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Manages key bindings for different modes
pub struct KeyBindingManager {
    command_bindings: HashMap<KeyBinding, String>,
    results_bindings: HashMap<KeyBinding, String>,
    search_bindings: HashMap<KeyBinding, String>,
}

impl KeyBindingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            command_bindings: HashMap::new(),
            results_bindings: HashMap::new(),
            search_bindings: HashMap::new(),
        };
        manager.setup_default_bindings();
        manager
    }

    fn setup_default_bindings(&mut self) {
        // Command mode bindings
        self.command_bindings
            .insert(KeyBinding::new(KeyCode::Enter), "execute_query".to_string());
        self.command_bindings
            .insert(KeyBinding::new(KeyCode::Tab), "autocomplete".to_string());
        self.command_bindings
            .insert(KeyBinding::new(KeyCode::F(1)), "toggle_help".to_string());
        self.command_bindings.insert(
            KeyBinding::new(KeyCode::Char('?')),
            "toggle_help".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::new(KeyCode::F(3)),
            "toggle_multiline".to_string(),
        );
        self.command_bindings
            .insert(KeyBinding::new(KeyCode::F(5)), "toggle_debug".to_string());
        self.command_bindings.insert(
            KeyBinding::new(KeyCode::F(8)),
            "toggle_case_insensitive".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::new(KeyCode::Down),
            "enter_results_mode".to_string(),
        );

        // Ctrl combinations
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('c')),
            "exit_app".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('d')),
            "exit_app".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('x')),
            "expand_select_star".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('r')),
            "search_history".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('p')),
            "previous_history".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('n')),
            "next_history".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('a')),
            "move_to_line_start".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('e')),
            "move_to_line_end".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('w')),
            "delete_word_backward".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('y')),
            "yank".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('z')),
            "undo".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('6')),
            "quick_switch_buffer".to_string(),
        );

        // Alt combinations
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Char('d')),
            "delete_word_forward".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Char('n')),
            "new_buffer".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Char('w')),
            "close_buffer".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Char('b')),
            "list_buffers".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Up),
            "previous_history".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_alt(KeyCode::Down),
            "next_history".to_string(),
        );

        // Function keys for buffers
        self.command_bindings.insert(
            KeyBinding::new(KeyCode::F(11)),
            "previous_buffer".to_string(),
        );
        self.command_bindings
            .insert(KeyBinding::new(KeyCode::F(12)), "next_buffer".to_string());
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::PageUp),
            "previous_buffer".to_string(),
        );
        self.command_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::PageDown),
            "next_buffer".to_string(),
        );

        // Results mode bindings
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('j')),
            "move_row_down".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('k')),
            "move_row_up".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('h')),
            "move_column_left".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('l')),
            "move_column_right".to_string(),
        );
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::Down), "move_row_down".to_string());
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::Up), "move_row_up".to_string());
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Left),
            "move_column_left".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Right),
            "move_column_right".to_string(),
        );
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::PageDown), "page_down".to_string());
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::PageUp), "page_up".to_string());
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('g')),
            "go_to_first_row".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('G')),
            "go_to_last_row".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('0')),
            "go_to_first_column".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('^')),
            "go_to_first_column".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('$')),
            "go_to_last_column".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('C')),
            "toggle_compact_mode".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('N')),
            "toggle_row_numbers".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char(':')),
            "jump_to_row".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('p')),
            "pin_column".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('P')),
            "clear_pins".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('/')),
            "start_search".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('\\')),
            "start_column_search".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('F')),
            "start_filter".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('f')),
            "start_fuzzy_filter".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('n')),
            "next_search_result".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('N')),
            "previous_search_result".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('s')),
            "sort_by_column".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('S')),
            "show_column_stats".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('v')),
            "toggle_selection_mode".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::new(KeyCode::Char('y')),
            "yank_selection".to_string(),
        );
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::Char('q')), "exit_app".to_string());
        self.results_bindings
            .insert(KeyBinding::new(KeyCode::Esc), "back_to_command".to_string());
        self.results_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('e')),
            "export_csv".to_string(),
        );
        self.results_bindings.insert(
            KeyBinding::with_ctrl(KeyCode::Char('j')),
            "export_json".to_string(),
        );

        // Search/Filter mode bindings
        self.search_bindings
            .insert(KeyBinding::new(KeyCode::Enter), "apply_search".to_string());
        self.search_bindings
            .insert(KeyBinding::new(KeyCode::Esc), "cancel_search".to_string());
    }

    /// Get the action for a key in command mode
    pub fn get_command_action(&self, key: &KeyEvent) -> Option<&String> {
        let binding = KeyBinding::from_event(key);
        self.command_bindings.get(&binding)
    }

    /// Get the action for a key in results mode
    pub fn get_results_action(&self, key: &KeyEvent) -> Option<&String> {
        let binding = KeyBinding::from_event(key);
        self.results_bindings.get(&binding)
    }

    /// Get the action for a key in search mode
    pub fn get_search_action(&self, key: &KeyEvent) -> Option<&String> {
        let binding = KeyBinding::from_event(key);
        self.search_bindings.get(&binding)
    }

    /// Customize a binding
    pub fn set_binding(&mut self, mode: &str, binding: KeyBinding, action: String) {
        match mode {
            "command" => {
                self.command_bindings.insert(binding, action);
            }
            "results" => {
                self.results_bindings.insert(binding, action);
            }
            "search" => {
                self.search_bindings.insert(binding, action);
            }
            _ => {}
        }
    }

    /// Remove a binding
    pub fn remove_binding(&mut self, mode: &str, binding: &KeyBinding) {
        match mode {
            "command" => {
                self.command_bindings.remove(binding);
            }
            "results" => {
                self.results_bindings.remove(binding);
            }
            "search" => {
                self.search_bindings.remove(binding);
            }
            _ => {}
        }
    }

    /// Get all bindings for a mode (for help display)
    pub fn get_bindings(&self, mode: &str) -> Vec<(KeyBinding, String)> {
        let map = match mode {
            "command" => &self.command_bindings,
            "results" => &self.results_bindings,
            "search" => &self.search_bindings,
            _ => return Vec::new(),
        };

        let mut bindings: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        bindings.sort_by_key(|(k, _)| format!("{:?}", k));
        bindings
    }
}
