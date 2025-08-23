use crate::buffer::{BufferAPI, BufferManager, EditMode};
use crate::buffer_handler::BufferHandler;
use crate::config::config::Config;
use crate::cursor_manager::CursorManager;
use tracing::info;

/// Trait that provides buffer management behavior for TUI components
/// This extracts buffer operations from EnhancedTui to reduce coupling
pub trait BufferManagementBehavior {
    // Required methods - these provide access to TUI internals
    fn buffer_manager(&mut self) -> &mut BufferManager;
    fn buffer_handler(&mut self) -> &mut BufferHandler;
    fn buffer(&self) -> &dyn BufferAPI;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn config(&self) -> &Config;
    fn cursor_manager(&mut self) -> &mut CursorManager;
    fn set_input_text_with_cursor(&mut self, text: String, cursor: usize);

    // ========== Buffer Management Operations ==========

    /// Create a new buffer with current config settings
    fn new_buffer(&mut self) {
        let buffer_count = self.buffer_manager().all_buffers().len();
        let mut new_buffer = crate::buffer::Buffer::new(buffer_count + 1);

        // Apply config settings to the new buffer
        let config = self.config();
        new_buffer.set_compact_mode(config.display.compact_mode);
        new_buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        new_buffer.set_show_row_numbers(config.display.show_row_numbers);

        info!(target: "buffer", 
              "Creating new buffer with config: compact_mode={}, case_insensitive={}, show_row_numbers={}",
              config.display.compact_mode,
              config.behavior.case_insensitive_default,
              config.display.show_row_numbers);

        let index = self.buffer_manager().add_buffer(new_buffer);
        self.buffer_mut()
            .set_status_message(format!("Created new buffer #{}", index + 1));
    }

    /// Switch to the next buffer
    fn next_buffer(&mut self) -> String {
        self.buffer_handler().next_buffer(self.buffer_manager())
    }

    /// Switch to the previous buffer
    fn previous_buffer(&mut self) -> String {
        self.buffer_handler().previous_buffer(self.buffer_manager())
    }

    /// Quick switch to the last used buffer
    fn quick_switch_buffer(&mut self) -> String {
        self.buffer_handler().quick_switch(self.buffer_manager())
    }

    /// Close the current buffer
    fn close_buffer(&mut self) -> (bool, String) {
        self.buffer_handler().close_buffer(self.buffer_manager())
    }

    /// Switch to a specific buffer by index
    fn switch_to_buffer(&mut self, index: usize) -> String {
        self.buffer_handler()
            .switch_to_buffer(self.buffer_manager(), index)
    }

    /// Yank (copy) from the current buffer
    fn yank(&mut self) {
        if let Some(buffer) = self.buffer_manager().current_mut() {
            buffer.yank();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager().set_position(cursor);
            }
        }
    }

    /// Get the total number of buffers
    fn buffer_count(&self) -> usize {
        self.buffer_manager().all_buffers().len()
    }

    /// Get the current buffer index
    fn current_buffer_index(&self) -> usize {
        self.buffer_manager().current_index()
    }
}
