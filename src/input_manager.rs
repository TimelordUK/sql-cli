use crossterm::event::{Event, KeyEvent};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Unified interface for managing input widgets (Input and TextArea)
/// This trait abstracts the differences between single-line and multi-line input
/// allowing the Buffer system to work with either transparently
pub trait InputManager: Send + Sync {
    /// Get the current text content
    fn get_text(&self) -> String;

    /// Set the text content
    fn set_text(&mut self, text: String);

    /// Get the current cursor position (char offset from start)
    fn get_cursor_position(&self) -> usize;

    /// Set the cursor position (char offset from start)
    fn set_cursor_position(&mut self, position: usize);

    /// Handle a key event
    fn handle_key_event(&mut self, event: KeyEvent) -> bool;

    /// Clear the content
    fn clear(&mut self);

    /// Check if content is empty
    fn is_empty(&self) -> bool;

    /// Get the visual cursor position for rendering (row, col)
    fn get_visual_cursor(&self) -> (u16, u16);

    /// Check if this is a multi-line input
    fn is_multiline(&self) -> bool;

    /// Get line count (1 for single-line)
    fn line_count(&self) -> usize;

    /// Get a specific line of text (0-indexed)
    fn get_line(&self, index: usize) -> Option<String>;

    /// Clone the input manager (for undo/redo)
    fn clone_box(&self) -> Box<dyn InputManager>;

    // --- History Navigation ---

    /// Set the history entries for navigation
    fn set_history(&mut self, history: Vec<String>);

    /// Navigate to previous history entry (returns true if navigation occurred)
    fn history_previous(&mut self) -> bool;

    /// Navigate to next history entry (returns true if navigation occurred)
    fn history_next(&mut self) -> bool;

    /// Get current history index (None if not navigating history)
    fn get_history_index(&self) -> Option<usize>;

    /// Reset history navigation (go back to user input)
    fn reset_history_position(&mut self);
}

/// Single-line input manager wrapping tui_input::Input
pub struct SingleLineInput {
    input: Input,
    history: Vec<String>,
    history_index: Option<usize>,
    temp_storage: Option<String>, // Store user input when navigating history
}

impl SingleLineInput {
    pub fn new(text: String) -> Self {
        let input = Input::new(text.clone()).with_cursor(text.len());
        Self {
            input,
            history: Vec::new(),
            history_index: None,
            temp_storage: None,
        }
    }

    pub fn from_input(input: Input) -> Self {
        Self {
            input,
            history: Vec::new(),
            history_index: None,
            temp_storage: None,
        }
    }

    pub fn as_input(&self) -> &Input {
        &self.input
    }

    pub fn as_input_mut(&mut self) -> &mut Input {
        &mut self.input
    }
}

impl InputManager for SingleLineInput {
    fn get_text(&self) -> String {
        self.input.value().to_string()
    }

    fn set_text(&mut self, text: String) {
        let cursor_pos = text.len();
        self.input = Input::new(text).with_cursor(cursor_pos);
    }

    fn get_cursor_position(&self) -> usize {
        self.input.visual_cursor()
    }

    fn set_cursor_position(&mut self, position: usize) {
        let text = self.input.value().to_string();
        self.input = Input::new(text).with_cursor(position);
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        // Convert KeyEvent to Event for tui_input
        let crossterm_event = Event::Key(event);
        self.input.handle_event(&crossterm_event);
        true
    }

    fn clear(&mut self) {
        self.input = Input::default();
    }

    fn is_empty(&self) -> bool {
        self.input.value().is_empty()
    }

    fn get_visual_cursor(&self) -> (u16, u16) {
        (0, self.input.visual_cursor() as u16)
    }

    fn is_multiline(&self) -> bool {
        false
    }

    fn line_count(&self) -> usize {
        1
    }

    fn get_line(&self, index: usize) -> Option<String> {
        if index == 0 {
            Some(self.get_text())
        } else {
            None
        }
    }

    fn clone_box(&self) -> Box<dyn InputManager> {
        Box::new(SingleLineInput {
            input: self.input.clone(),
            history: self.history.clone(),
            history_index: self.history_index,
            temp_storage: self.temp_storage.clone(),
        })
    }

    // --- History Navigation ---

    fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
        self.history_index = None;
        self.temp_storage = None;
    }

    fn history_previous(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }

        match self.history_index {
            None => {
                // First time navigating, save current input
                self.temp_storage = Some(self.input.value().to_string());
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return false, // Already at oldest
            Some(idx) => {
                self.history_index = Some(idx - 1);
            }
        }

        // Update input with history entry
        if let Some(idx) = self.history_index {
            if let Some(entry) = self.history.get(idx) {
                let len = entry.len();
                self.input = Input::new(entry.clone()).with_cursor(len);
                return true;
            }
        }
        false
    }

    fn history_next(&mut self) -> bool {
        match self.history_index {
            None => false, // Not navigating history
            Some(idx) => {
                if idx >= self.history.len() - 1 {
                    // Going back to user input
                    if let Some(temp) = &self.temp_storage {
                        let len = temp.len();
                        self.input = Input::new(temp.clone()).with_cursor(len);
                    }
                    self.history_index = None;
                    self.temp_storage = None;
                    true
                } else {
                    self.history_index = Some(idx + 1);
                    if let Some(entry) = self.history.get(idx + 1) {
                        let len = entry.len();
                        self.input = Input::new(entry.clone()).with_cursor(len);
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

    fn get_history_index(&self) -> Option<usize> {
        self.history_index
    }

    fn reset_history_position(&mut self) {
        self.history_index = None;
        self.temp_storage = None;
    }
}

/// Factory methods for creating InputManager instances
pub fn create_single_line(text: String) -> Box<dyn InputManager> {
    Box::new(SingleLineInput::new(text))
}

pub fn create_from_input(input: Input) -> Box<dyn InputManager> {
    Box::new(SingleLineInput::from_input(input))
}
