use crate::buffer::{BufferAPI, BufferManager, EditMode};
use crate::cursor_manager::CursorManager;
use crate::ui::text_operations::{self, CursorMovementResult, TextOperationResult};

/// Trait that provides input operation behavior for TUI components
/// This uses pure text manipulation functions and applies results to TUI state
pub trait InputBehavior {
    // Required methods - these provide access to TUI internals
    fn buffer_manager(&mut self) -> &mut BufferManager;
    fn cursor_manager(&mut self) -> &mut CursorManager;
    fn set_input_text_with_cursor(&mut self, text: String, cursor: usize);

    // Helper method to get current input state
    fn get_current_input(&mut self) -> (String, usize) {
        if let Some(buffer) = self.buffer_manager().current() {
            (buffer.get_input_text(), buffer.get_input_cursor_position())
        } else {
            (String::new(), 0)
        }
    }

    // Helper method to apply text operation results
    fn apply_text_result(&mut self, result: TextOperationResult) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            // Update buffer text and cursor
            buffer.set_input_text(result.new_text.clone());
            buffer.set_input_cursor_position(result.new_cursor_position);

            // Add killed text to kill ring if present
            if let Some(killed) = result.killed_text {
                buffer.set_kill_ring(killed);
            }

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                self.set_input_text_with_cursor(result.new_text, result.new_cursor_position);
                self.cursor_manager()
                    .set_position(result.new_cursor_position);
            }
        }
    }

    // Helper method to apply cursor movement results
    fn apply_cursor_result(&mut self, result: CursorMovementResult) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.set_input_cursor_position(result.new_position);

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                self.set_input_text_with_cursor(text, result.new_position);
                self.cursor_manager().set_position(result.new_position);
            }
        }
    }

    // ========== Text Manipulation Operations ==========

    /// Kill text from cursor to end of line (Ctrl+K)
    fn kill_line(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::kill_line(&text, cursor);
        self.apply_text_result(result);
    }

    /// Kill text from beginning of line to cursor (Ctrl+U)
    fn kill_line_backward(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::kill_line_backward(&text, cursor);
        self.apply_text_result(result);
    }

    /// Delete word backward from cursor (Ctrl+W)
    fn delete_word_backward(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::delete_word_backward(&text, cursor);
        self.apply_text_result(result);
    }

    /// Delete word forward from cursor (Alt+D)
    fn delete_word_forward(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::delete_word_forward(&text, cursor);
        self.apply_text_result(result);
    }

    // ========== Cursor Movement Operations ==========

    /// Move cursor backward one word (Ctrl+Left or Alt+B)
    fn move_cursor_word_backward(&mut self) {
        let (text, cursor) = self.get_current_input();
        let result = text_operations::move_word_backward(&text, cursor);
        self.apply_cursor_result(result);
    }

    /// Move cursor forward one word (Ctrl+Right or Alt+F)
    fn move_cursor_word_forward(&mut self) {
        let (text, cursor) = self.get_current_input();
        let result = text_operations::move_word_forward(&text, cursor);
        self.apply_cursor_result(result);
    }

    /// Jump to previous SQL token (Alt+[)
    fn jump_to_prev_token(&mut self) {
        let (text, cursor) = self.get_current_input();
        let result = text_operations::jump_to_prev_token(&text, cursor);
        self.apply_cursor_result(result);
    }

    /// Jump to next SQL token (Alt+])
    fn jump_to_next_token(&mut self) {
        let (text, cursor) = self.get_current_input();
        let result = text_operations::jump_to_next_token(&text, cursor);
        self.apply_cursor_result(result);
    }

    // ========== Basic Text Operations ==========

    /// Insert a character at the cursor position
    fn insert_char(&mut self, ch: char) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::insert_char(&text, cursor, ch);
        self.apply_text_result(result);
    }

    /// Delete character at cursor position (Delete key)
    fn delete_char(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::delete_char(&text, cursor);
        self.apply_text_result(result);
    }

    /// Delete character before cursor (Backspace)
    fn backspace(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let (text, cursor) = self.get_current_input();
        let result = text_operations::backspace(&text, cursor);
        self.apply_text_result(result);
    }

    /// Clear all input text
    fn clear_input(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.save_state_for_undo();
        }

        let result = text_operations::clear_text();
        self.apply_text_result(result);
    }
}
