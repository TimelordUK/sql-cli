use crate::history::{CommandHistory, HistoryMatch};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::VecDeque;

/// Modern, clean input system without legacy baggage
/// Preserves all the excellent features of the original but with cleaner architecture
pub struct ModernInput {
    /// Current input text
    text: String,

    /// Cursor position (byte offset)
    cursor: usize,

    /// History management
    history: CommandHistory,
    history_navigation: HistoryNavigation,

    /// Current schema context for history search
    schema_columns: Vec<String>,
    data_source: Option<String>,

    /// Input mode
    mode: InputMode,

    /// Search state for fuzzy history search
    search_state: SearchState,

    /// Undo/redo stack
    undo_stack: VecDeque<InputSnapshot>,
    redo_stack: VecDeque<InputSnapshot>,
    max_undo: usize,
}

/// Input modes
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,        // Regular typing
    HistorySearch, // Ctrl+R fuzzy search through history
    HistoryNav,    // Up/Down arrow navigation
}

/// History navigation state
#[derive(Debug, Clone)]
struct HistoryNavigation {
    entries: Vec<String>,
    current_index: Option<usize>,
    original_input: Option<String>,
}

/// Search state for Ctrl+R fuzzy search
#[derive(Debug, Clone)]
struct SearchState {
    query: String,
    matches: Vec<HistoryMatch>,
    selected_index: usize,
    original_input: String,
}

/// Snapshot for undo/redo
#[derive(Debug, Clone)]
struct InputSnapshot {
    text: String,
    cursor: usize,
}

impl ModernInput {
    /// Create a new modern input
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: CommandHistory::default(),
            history_navigation: HistoryNavigation {
                entries: Vec::new(),
                current_index: None,
                original_input: None,
            },
            schema_columns: Vec::new(),
            data_source: None,
            mode: InputMode::Normal,
            search_state: SearchState {
                query: String::new(),
                matches: Vec::new(),
                selected_index: 0,
                original_input: String::new(),
            },
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo: 50,
        }
    }

    /// Create with initial text
    pub fn with_text(text: String) -> Self {
        let cursor = text.len();
        Self {
            text,
            cursor,
            ..Self::new()
        }
    }

    /// Set schema context for better history matching
    pub fn set_schema_context(&mut self, columns: Vec<String>, data_source: Option<String>) {
        self.schema_columns = columns;
        self.data_source = data_source;
    }

    /// Get current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    /// Get current input mode
    pub fn mode(&self) -> &InputMode {
        &self.mode
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.save_snapshot();
        self.text.clear();
        self.cursor = 0;
        self.exit_special_modes();
    }

    /// Set text and cursor to end
    pub fn set_text(&mut self, text: String) {
        self.save_snapshot();
        self.cursor = text.len();
        self.text = text;
        self.exit_special_modes();
    }

    /// Handle key events - returns true if input was consumed
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match &self.mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::HistorySearch => self.handle_history_search_mode(key),
            InputMode::HistoryNav => self.handle_history_nav_mode(key),
        }
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command: String, success: bool, duration_ms: Option<u64>) {
        if let Err(e) = self.history.add_entry_with_schema(
            command,
            success,
            duration_ms,
            self.schema_columns.clone(),
            self.data_source.clone(),
        ) {
            eprintln!("Failed to add to history: {}", e);
        }

        // Update navigation entries
        self.update_navigation_entries();
    }

    /// Handle normal input mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // History search (Ctrl+R)
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.enter_history_search_mode();
                true
            }

            // History navigation (Up/Down)
            KeyCode::Up => {
                self.enter_history_nav_mode();
                self.history_nav_previous();
                true
            }
            KeyCode::Down => {
                if self.mode == InputMode::HistoryNav {
                    self.history_nav_next();
                } else {
                    self.enter_history_nav_mode();
                    self.history_nav_next();
                }
                true
            }

            // Undo/Redo
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.undo();
                true
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.redo();
                true
            }

            // Regular editing
            _ => self.handle_edit_keys(key),
        }
    }

    /// Handle history search mode (Ctrl+R)
    fn handle_history_search_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Continue search
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_next_match();
                true
            }

            // Accept current match
            KeyCode::Enter => {
                self.accept_search_match();
                true
            }

            // Cancel search
            KeyCode::Esc => {
                self.cancel_search();
                true
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.cancel_search();
                true
            }

            // Navigate matches
            KeyCode::Up => {
                self.search_prev_match();
                true
            }
            KeyCode::Down => {
                self.search_next_match();
                true
            }

            // Edit search query
            KeyCode::Char(c) => {
                self.search_add_char(c);
                true
            }
            KeyCode::Backspace => {
                self.search_delete_char();
                true
            }

            _ => false,
        }
    }

    /// Handle history navigation mode (Up/Down arrows)
    fn handle_history_nav_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.history_nav_previous();
                true
            }
            KeyCode::Down => {
                self.history_nav_next();
                true
            }
            KeyCode::Esc => {
                self.exit_special_modes();
                true
            }
            // Any other key exits navigation mode
            _ => {
                self.exit_special_modes();
                self.handle_edit_keys(key)
            }
        }
    }

    /// Handle regular editing keys
    fn handle_edit_keys(&mut self, key: KeyEvent) -> bool {
        self.save_snapshot();

        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.handle_control_char(c)
                } else {
                    self.insert_char(c);
                    true
                }
            }
            KeyCode::Backspace => {
                self.delete_char_backward();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_backward();
                } else {
                    self.move_cursor_left();
                }
                true
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_forward();
                } else {
                    self.move_cursor_right();
                }
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }

    /// Handle Ctrl+key combinations
    fn handle_control_char(&mut self, c: char) -> bool {
        match c {
            'a' => {
                self.move_cursor_home();
                true
            }
            'e' => {
                self.move_cursor_end();
                true
            }
            'f' => {
                self.move_cursor_right();
                true
            }
            'b' => {
                self.move_cursor_left();
                true
            }
            'd' => {
                self.delete_char_forward();
                true
            }
            'h' => {
                self.delete_char_backward();
                true
            }
            'k' => {
                self.delete_to_end_of_line();
                true
            }
            'u' => {
                self.delete_to_start_of_line();
                true
            }
            'w' => {
                self.delete_word_backward();
                true
            }
            'l' => {
                self.clear();
                true
            }
            _ => false,
        }
    }

    // === Text editing operations ===

    fn insert_char(&mut self, c: char) {
        self.text
            .insert(self.char_index_to_byte_index(self.cursor), c);
        self.cursor += 1;
    }

    fn delete_char_backward(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let byte_index = self.char_index_to_byte_index(self.cursor);
            self.text.remove(byte_index);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor < self.text.chars().count() {
            let byte_index = self.char_index_to_byte_index(self.cursor);
            self.text.remove(byte_index);
        }
    }

    fn delete_to_end_of_line(&mut self) {
        let byte_index = self.char_index_to_byte_index(self.cursor);
        self.text.truncate(byte_index);
    }

    fn delete_to_start_of_line(&mut self) {
        let byte_index = self.char_index_to_byte_index(self.cursor);
        self.text.drain(0..byte_index);
        self.cursor = 0;
    }

    fn delete_word_backward(&mut self) {
        let original_cursor = self.cursor;
        self.move_word_backward();
        let start_byte = self.char_index_to_byte_index(self.cursor);
        let end_byte = self.char_index_to_byte_index(original_cursor);
        self.text.drain(start_byte..end_byte);
    }

    // === Cursor movement ===

    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        let char_count = self.text.chars().count();
        if self.cursor < char_count {
            self.cursor += 1;
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor = self.text.chars().count();
    }

    fn move_word_backward(&mut self) {
        while self.cursor > 0
            && self
                .char_at_cursor_minus_1()
                .map_or(false, |c| c.is_whitespace())
        {
            self.cursor -= 1;
        }
        while self.cursor > 0
            && self
                .char_at_cursor_minus_1()
                .map_or(false, |c| !c.is_whitespace())
        {
            self.cursor -= 1;
        }
    }

    fn move_word_forward(&mut self) {
        let char_count = self.text.chars().count();
        while self.cursor < char_count
            && self.char_at_cursor().map_or(false, |c| !c.is_whitespace())
        {
            self.cursor += 1;
        }
        while self.cursor < char_count && self.char_at_cursor().map_or(false, |c| c.is_whitespace())
        {
            self.cursor += 1;
        }
    }

    // === History search mode ===

    fn enter_history_search_mode(&mut self) {
        self.search_state = SearchState {
            query: String::new(),
            matches: Vec::new(),
            selected_index: 0,
            original_input: self.text.clone(),
        };
        self.mode = InputMode::HistorySearch;
        self.update_search_matches();
    }

    fn search_add_char(&mut self, c: char) {
        self.search_state.query.push(c);
        self.update_search_matches();
    }

    fn search_delete_char(&mut self) {
        self.search_state.query.pop();
        self.update_search_matches();
    }

    fn search_next_match(&mut self) {
        if !self.search_state.matches.is_empty() {
            self.search_state.selected_index =
                (self.search_state.selected_index + 1) % self.search_state.matches.len();
            self.update_input_from_search();
        }
    }

    fn search_prev_match(&mut self) {
        if !self.search_state.matches.is_empty() {
            self.search_state.selected_index = if self.search_state.selected_index == 0 {
                self.search_state.matches.len() - 1
            } else {
                self.search_state.selected_index - 1
            };
            self.update_input_from_search();
        }
    }

    fn accept_search_match(&mut self) {
        self.mode = InputMode::Normal;
        self.cursor = self.text.chars().count();
    }

    fn cancel_search(&mut self) {
        self.text = self.search_state.original_input.clone();
        self.cursor = self.text.chars().count();
        self.mode = InputMode::Normal;
    }

    fn update_search_matches(&mut self) {
        self.search_state.matches = self.history.search_with_schema(
            &self.search_state.query,
            &self.schema_columns,
            self.data_source.as_deref(),
        );

        if self.search_state.selected_index >= self.search_state.matches.len() {
            self.search_state.selected_index = 0;
        }

        self.update_input_from_search();
    }

    fn update_input_from_search(&mut self) {
        if let Some(selected_match) = self
            .search_state
            .matches
            .get(self.search_state.selected_index)
        {
            self.text = selected_match.entry.command.clone();
        } else if self.search_state.query.is_empty() {
            self.text = self.search_state.original_input.clone();
        } else {
            // No matches, show original input
            self.text = self.search_state.original_input.clone();
        }
        self.cursor = self.text.chars().count();
    }

    // === History navigation mode ===

    fn enter_history_nav_mode(&mut self) {
        if self.mode != InputMode::HistoryNav {
            self.history_navigation.original_input = Some(self.text.clone());
            self.history_navigation.current_index = None;
            self.mode = InputMode::HistoryNav;
        }
    }

    fn history_nav_previous(&mut self) {
        if self.history_navigation.entries.is_empty() {
            return;
        }

        let new_index = match self.history_navigation.current_index {
            None => Some(self.history_navigation.entries.len() - 1),
            Some(0) => Some(0), // Stay at oldest
            Some(i) => Some(i - 1),
        };

        if let Some(index) = new_index {
            if let Some(entry) = self.history_navigation.entries.get(index) {
                self.text = entry.clone();
                self.cursor = self.text.chars().count();
                self.history_navigation.current_index = Some(index);
            }
        }
    }

    fn history_nav_next(&mut self) {
        match self.history_navigation.current_index {
            None => return, // Not navigating
            Some(i) if i >= self.history_navigation.entries.len() - 1 => {
                // Go back to original input
                if let Some(original) = &self.history_navigation.original_input {
                    self.text = original.clone();
                    self.cursor = self.text.chars().count();
                }
                self.history_navigation.current_index = None;
            }
            Some(i) => {
                if let Some(entry) = self.history_navigation.entries.get(i + 1) {
                    self.text = entry.clone();
                    self.cursor = self.text.chars().count();
                    self.history_navigation.current_index = Some(i + 1);
                }
            }
        }
    }

    fn update_navigation_entries(&mut self) {
        self.history_navigation.entries = self
            .history
            .get_navigation_entries()
            .into_iter()
            .map(|e| e.command)
            .collect();
    }

    // === Mode management ===

    fn exit_special_modes(&mut self) {
        self.mode = InputMode::Normal;
        self.history_navigation.current_index = None;
        self.history_navigation.original_input = None;
    }

    // === Undo/Redo ===

    fn save_snapshot(&mut self) {
        let snapshot = InputSnapshot {
            text: self.text.clone(),
            cursor: self.cursor,
        };

        self.undo_stack.push_back(snapshot);
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.pop_front();
        }

        // Clear redo stack on new action
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop_back() {
            let current = InputSnapshot {
                text: self.text.clone(),
                cursor: self.cursor,
            };
            self.redo_stack.push_back(current);

            self.text = snapshot.text;
            self.cursor = snapshot.cursor;
        }
    }

    fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop_back() {
            let current = InputSnapshot {
                text: self.text.clone(),
                cursor: self.cursor,
            };
            self.undo_stack.push_back(current);

            self.text = snapshot.text;
            self.cursor = snapshot.cursor;
        }
    }

    // === Utility methods ===

    fn char_index_to_byte_index(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map(|(byte_index, _)| byte_index)
            .unwrap_or(self.text.len())
    }

    fn char_at_cursor(&self) -> Option<char> {
        self.text.chars().nth(self.cursor)
    }

    fn char_at_cursor_minus_1(&self) -> Option<char> {
        if self.cursor > 0 {
            self.text.chars().nth(self.cursor - 1)
        } else {
            None
        }
    }

    // === Rendering ===

    /// Create a widget for rendering the input
    pub fn create_widget(&self) -> Paragraph<'_> {
        match &self.mode {
            InputMode::Normal | InputMode::HistoryNav => {
                let title = if self.mode == InputMode::HistoryNav {
                    "Query (History Navigation)"
                } else {
                    "Query"
                };

                Paragraph::new(self.text.as_str())
                    .block(Block::default().borders(Borders::ALL).title(title))
            }
            InputMode::HistorySearch => self.create_search_widget(),
        }
    }

    fn create_search_widget(&self) -> Paragraph<'_> {
        let mut spans = vec![
            Span::styled("(reverse-i-search)`", Style::default().fg(Color::Cyan)),
            Span::styled(&self.search_state.query, Style::default().fg(Color::Yellow)),
            Span::styled("': ", Style::default().fg(Color::Cyan)),
            Span::raw(&self.text),
        ];

        // Show match count
        if !self.search_state.matches.is_empty() {
            let match_info = format!(
                " [{}/{}]",
                self.search_state.selected_index + 1,
                self.search_state.matches.len()
            );
            spans.push(Span::styled(
                match_info,
                Style::default().fg(Color::DarkGray),
            ));
        }

        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("History Search (Ctrl+R: next, Enter: select, Esc: cancel)")
                .style(Style::default().fg(Color::Cyan)),
        )
    }

    /// Get cursor position for rendering
    pub fn visual_cursor_position(&self) -> usize {
        match &self.mode {
            InputMode::Normal | InputMode::HistoryNav => self.cursor,
            InputMode::HistorySearch => {
                // In search mode, cursor is after the search query
                self.search_state.query.chars().count() + 20 // Offset for prompt
            }
        }
    }

    /// Get status information
    pub fn get_status(&self) -> String {
        match &self.mode {
            InputMode::Normal => String::new(),
            InputMode::HistoryNav => {
                if let Some(index) = self.history_navigation.current_index {
                    format!(
                        "History: {}/{}",
                        index + 1,
                        self.history_navigation.entries.len()
                    )
                } else {
                    "History navigation".to_string()
                }
            }
            InputMode::HistorySearch => {
                if self.search_state.matches.is_empty() {
                    format!("Search: '{}' (no matches)", self.search_state.query)
                } else {
                    format!(
                        "Search: '{}' ({}/{})",
                        self.search_state.query,
                        self.search_state.selected_index + 1,
                        self.search_state.matches.len()
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_basic_input() {
        let mut input = ModernInput::new();

        // Type some text
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        input.handle_key_event(key);

        assert_eq!(input.text(), "h");
        assert_eq!(input.cursor_position(), 1);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = ModernInput::with_text("hello world".to_string());

        // Move to beginning
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        input.handle_key_event(key);
        assert_eq!(input.cursor_position(), 0);

        // Move right by word
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);
        input.handle_key_event(key);
        assert_eq!(input.cursor_position(), 6); // After "hello " (includes space)
    }

    #[test]
    fn test_deletion() {
        let mut input = ModernInput::with_text("hello world".to_string());

        // Delete word backward from end
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        input.handle_key_event(key);
        assert_eq!(input.text(), "hello ");
    }

    #[test]
    fn test_history_search_mode() {
        let mut input = ModernInput::new();

        // Enter search mode
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let handled = input.handle_key_event(key);
        assert!(handled);
        assert_eq!(input.mode(), &InputMode::HistorySearch);

        // Cancel search
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let handled = input.handle_key_event(key);
        assert!(handled);
        assert_eq!(input.mode(), &InputMode::Normal);
    }

    #[test]
    fn test_undo_redo() {
        let mut input = ModernInput::new();

        // Type and save snapshot
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        input.handle_key_event(key);

        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        input.handle_key_event(key);

        assert_eq!(input.text(), "ab");

        // Undo
        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        input.handle_key_event(key);

        assert_eq!(input.text(), "a");

        // Redo
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        input.handle_key_event(key);

        assert_eq!(input.text(), "ab");
    }
}
