use crate::buffer::{Buffer, BufferAPI, BufferManager};
use crate::config::config::Config;
use tracing::{debug, info};

/// Handles all buffer-related operations
pub struct BufferHandler {
    /// Stack for tracking buffer history (for quick switching)
    buffer_history: Vec<usize>,
    /// Maximum number of buffers to keep in history
    max_history: usize,
}

impl BufferHandler {
    pub fn new() -> Self {
        Self {
            buffer_history: vec![0], // Start with buffer 0
            max_history: 10,
        }
    }

    /// Switch to next buffer
    pub fn next_buffer(&mut self, manager: &mut BufferManager) -> String {
        let prev_index = manager.current_index();
        manager.next_buffer();
        let index = manager.current_index();
        let total = manager.all_buffers().len();

        self.update_history(index);
        debug!(target: "buffer", "Switched from buffer {} to {} (total: {})", prev_index + 1, index + 1, total);

        format!("Switched to buffer {}/{}", index + 1, total)
    }

    /// Switch to previous buffer
    pub fn previous_buffer(&mut self, manager: &mut BufferManager) -> String {
        let prev_index = manager.current_index();
        manager.prev_buffer();
        let index = manager.current_index();
        let total = manager.all_buffers().len();

        self.update_history(index);
        debug!(target: "buffer", "Switched from buffer {} to {} (total: {})", prev_index + 1, index + 1, total);

        format!("Switched to buffer {}/{}", index + 1, total)
    }

    /// Quick switch between last two buffers (like vim's Ctrl+6)
    pub fn quick_switch(&mut self, manager: &mut BufferManager) -> String {
        if self.buffer_history.len() >= 2 {
            // Get the previous buffer from history
            let prev_buffer = self.buffer_history[self.buffer_history.len() - 2];
            self.switch_to_buffer(manager, prev_buffer)
        } else {
            // If no history, just switch to next
            self.next_buffer(manager)
        }
    }

    /// Switch to specific buffer by index (0-based)
    pub fn switch_to_buffer(&mut self, manager: &mut BufferManager, index: usize) -> String {
        let total = manager.all_buffers().len();
        if index >= total {
            return format!(
                "Buffer {} does not exist (have {} buffers)",
                index + 1,
                total
            );
        }

        let prev_index = manager.current_index();
        if prev_index == index {
            return format!("Already on buffer {}/{}", index + 1, total);
        }

        // Set the buffer directly
        if index == 0 {
            while manager.current_index() != 0 {
                manager.prev_buffer();
            }
        } else {
            while manager.current_index() != index {
                if manager.current_index() < index {
                    manager.next_buffer();
                } else {
                    manager.prev_buffer();
                }
            }
        }

        self.update_history(index);
        debug!(target: "buffer", "Switched from buffer {} to {} (total: {})", prev_index + 1, index + 1, total);

        format!("Switched to buffer {}/{}", index + 1, total)
    }

    /// Create a new buffer
    pub fn new_buffer(&mut self, manager: &mut BufferManager, config: &Config) -> String {
        let buffer_id = manager.all_buffers().len() + 1;
        let mut new_buffer = Buffer::new(buffer_id);

        // Apply config settings to the new buffer
        new_buffer.set_compact_mode(config.display.compact_mode);
        new_buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        new_buffer.set_show_row_numbers(config.display.show_row_numbers);

        info!(target: "buffer", "Creating new buffer with config: compact_mode={}, case_insensitive={}, show_row_numbers={}",
              config.display.compact_mode,
              config.behavior.case_insensitive_default,
              config.display.show_row_numbers);

        manager.add_buffer(new_buffer);
        let index = manager.current_index();
        let total = manager.all_buffers().len();

        self.update_history(index);

        format!("Created new buffer {}/{}", index + 1, total)
    }

    /// Close current buffer
    pub fn close_buffer(&mut self, manager: &mut BufferManager) -> (bool, String) {
        if manager.all_buffers().len() == 1 {
            return (false, "Cannot close the last buffer".to_string());
        }

        if manager.close_current() {
            let index = manager.current_index();
            let total = manager.all_buffers().len();

            // Remove closed buffer from history
            self.buffer_history.retain(|&idx| idx < total);

            // Update indices in history (shift down if needed)
            for idx in &mut self.buffer_history {
                if *idx > index {
                    *idx -= 1;
                }
            }

            self.update_history(index);

            (
                true,
                format!("Buffer closed. Now at buffer {}/{}", index + 1, total),
            )
        } else {
            (false, "Failed to close buffer".to_string())
        }
    }

    /// List all buffers with their status
    pub fn list_buffers(&self, manager: &BufferManager) -> Vec<String> {
        let current_index = manager.current_index();
        let mut buffer_list = Vec::new();

        for (i, buffer) in manager.all_buffers().iter().enumerate() {
            let marker = if i == current_index { "▶" } else { " " };

            // Get buffer info
            let has_results = buffer.has_datatable();
            let query = buffer.get_query();
            let query_preview = if !query.is_empty() {
                if query.len() > 30 {
                    format!("{}...", &query[..27])
                } else {
                    query.clone()
                }
            } else {
                "Empty".to_string()
            };

            let status = if has_results { "●" } else { "○" };

            buffer_list.push(format!(
                "{} [{}] Buffer {}: {} {}",
                marker,
                status,
                i + 1,
                query_preview,
                if i < 9 {
                    format!("(Alt+{})", i + 1)
                } else {
                    String::new()
                }
            ));
        }

        // Add history at the bottom
        if !self.buffer_history.is_empty() {
            buffer_list.push(format!(
                "  History: {}",
                self.buffer_history
                    .iter()
                    .rev()
                    .take(5)
                    .map(|idx| format!("{}", idx + 1))
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
        }

        buffer_list
    }

    /// Update buffer history
    fn update_history(&mut self, index: usize) {
        // Remove if already in history
        self.buffer_history.retain(|&idx| idx != index);

        // Add to end
        self.buffer_history.push(index);

        // Trim to max size
        if self.buffer_history.len() > self.max_history {
            self.buffer_history.remove(0);
        }
    }

    /// Get buffer history
    pub fn get_history(&self) -> &[usize] {
        &self.buffer_history
    }

    /// Clear buffer history
    pub fn clear_history(&mut self) {
        self.buffer_history.clear();
        self.buffer_history.push(0);
    }
}
