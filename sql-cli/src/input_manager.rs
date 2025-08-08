use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_textarea::{CursorMove, TextArea};

/// Unified interface for managing input widgets (Input and TextArea)
/// This trait abstracts the differences between single-line and multi-line input
/// allowing the Buffer system to work with either transparently
pub trait InputManager: Send {
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
}

/// Single-line input manager wrapping tui_input::Input
pub struct SingleLineInput {
    input: Input,
}

impl SingleLineInput {
    pub fn new(text: String) -> Self {
        let input = Input::new(text.clone()).with_cursor(text.len());
        Self { input }
    }

    pub fn from_input(input: Input) -> Self {
        Self { input }
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
        })
    }
}

/// Multi-line input manager wrapping tui_textarea::TextArea
pub struct MultiLineInput {
    textarea: TextArea<'static>,
}

impl MultiLineInput {
    pub fn new(text: String) -> Self {
        let mut textarea = TextArea::new(vec![text.clone()]);
        // Move cursor to end
        textarea.move_cursor(CursorMove::End);
        Self { textarea }
    }

    pub fn from_lines(lines: Vec<String>) -> Self {
        let mut textarea = TextArea::new(lines);
        textarea.move_cursor(CursorMove::End);
        Self { textarea }
    }

    pub fn from_textarea(textarea: TextArea<'static>) -> Self {
        Self { textarea }
    }

    pub fn as_textarea(&self) -> &TextArea<'static> {
        &self.textarea
    }

    pub fn as_textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.textarea
    }
}

impl InputManager for MultiLineInput {
    fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    fn set_text(&mut self, text: String) {
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        self.textarea = TextArea::new(lines);
        self.textarea.move_cursor(CursorMove::End);
    }

    fn get_cursor_position(&self) -> usize {
        // Calculate absolute position from row/col
        let (row, col) = self.textarea.cursor();
        let mut pos = 0;
        for (i, line) in self.textarea.lines().iter().enumerate() {
            if i < row {
                pos += line.len() + 1; // +1 for newline
            } else if i == row {
                pos += col;
                break;
            }
        }
        pos
    }

    fn set_cursor_position(&mut self, position: usize) {
        // Convert absolute position to row/col
        let mut current_pos = 0;
        for (row, line) in self.textarea.lines().iter().enumerate() {
            let line_len = line.len();
            if current_pos + line_len >= position {
                let col = position - current_pos;
                self.textarea
                    .move_cursor(CursorMove::Jump(row as u16, col as u16));
                return;
            }
            current_pos += line_len + 1; // +1 for newline
        }
        // If position is beyond text, move to end
        self.textarea.move_cursor(CursorMove::End);
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Handle control keys specially
                    match c {
                        'a' => self.textarea.move_cursor(CursorMove::Head),
                        'e' => self.textarea.move_cursor(CursorMove::End),
                        'f' => self.textarea.move_cursor(CursorMove::Forward),
                        'b' => self.textarea.move_cursor(CursorMove::Back),
                        'n' => self.textarea.move_cursor(CursorMove::Down),
                        'p' => self.textarea.move_cursor(CursorMove::Up),
                        'd' => {
                            self.textarea.delete_char();
                        }
                        'k' => {
                            self.textarea.delete_line_by_end();
                        }
                        'u' => {
                            self.textarea.delete_line_by_head();
                        }
                        'w' => {
                            self.textarea.delete_word();
                        }
                        'h' => {
                            self.textarea.delete_char();
                        }
                        _ => return false,
                    }
                } else {
                    self.textarea.insert_char(c);
                }
            }
            KeyCode::Backspace => {
                self.textarea.delete_char();
            }
            KeyCode::Delete => {
                self.textarea.delete_char();
            }
            KeyCode::Enter => {
                self.textarea.insert_newline();
            }
            KeyCode::Tab => {
                self.textarea.insert_tab();
            }
            KeyCode::Left => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.textarea.move_cursor(CursorMove::WordBack);
                } else {
                    self.textarea.move_cursor(CursorMove::Back);
                }
            }
            KeyCode::Right => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.textarea.move_cursor(CursorMove::WordForward);
                } else {
                    self.textarea.move_cursor(CursorMove::Forward);
                }
            }
            KeyCode::Up => {
                self.textarea.move_cursor(CursorMove::Up);
            }
            KeyCode::Down => {
                self.textarea.move_cursor(CursorMove::Down);
            }
            KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
            }
            KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
            }
            KeyCode::PageUp => {
                // Move up by screen height (approximate)
                for _ in 0..20 {
                    self.textarea.move_cursor(CursorMove::Up);
                }
            }
            KeyCode::PageDown => {
                // Move down by screen height (approximate)
                for _ in 0..20 {
                    self.textarea.move_cursor(CursorMove::Down);
                }
            }
            _ => return false,
        }
        true
    }

    fn clear(&mut self) {
        self.textarea = TextArea::default();
    }

    fn is_empty(&self) -> bool {
        self.textarea.lines().len() == 1 && self.textarea.lines()[0].is_empty()
    }

    fn get_visual_cursor(&self) -> (u16, u16) {
        let (row, col) = self.textarea.cursor();
        (row as u16, col as u16)
    }

    fn is_multiline(&self) -> bool {
        true
    }

    fn line_count(&self) -> usize {
        self.textarea.lines().len()
    }

    fn get_line(&self, index: usize) -> Option<String> {
        self.textarea.lines().get(index).cloned()
    }

    fn clone_box(&self) -> Box<dyn InputManager> {
        Box::new(MultiLineInput {
            textarea: self.textarea.clone(),
        })
    }
}

/// Factory methods for creating InputManager instances
pub fn create_single_line(text: String) -> Box<dyn InputManager> {
    Box::new(SingleLineInput::new(text))
}

pub fn create_multi_line(text: String) -> Box<dyn InputManager> {
    Box::new(MultiLineInput::new(text))
}

pub fn create_from_input(input: Input) -> Box<dyn InputManager> {
    Box::new(SingleLineInput::from_input(input))
}

pub fn create_from_textarea(textarea: TextArea<'static>) -> Box<dyn InputManager> {
    Box::new(MultiLineInput::from_textarea(textarea))
}
