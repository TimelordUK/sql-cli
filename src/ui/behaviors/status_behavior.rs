// Status and error message handling behavior
// Manages all status bar updates and error display

use anyhow::Result;

/// Trait for managing status and error messages in the UI
pub trait StatusBehavior {
    // Required methods - provide access to TUI internals
    fn buffer_mut(&mut self) -> &mut dyn crate::buffer::BufferAPI;

    /// Set a status message
    fn set_status(&mut self, message: impl Into<String>) {
        let msg = message.into();
        tracing::debug!("Status: {}", msg);
        self.buffer_mut().set_status_message(msg);
    }

    /// Set an error message with context
    fn set_error(&mut self, context: &str, error: impl std::fmt::Display) {
        let msg = format!("{}: {}", context, error);
        tracing::error!("Error status: {}", msg);
        self.buffer_mut().set_status_message(msg);
    }

    /// Set a success message
    fn set_success(&mut self, message: impl Into<String>) {
        let msg = message.into();
        tracing::info!("Success: {}", msg);
        self.buffer_mut().set_status_message(msg);
    }

    /// Clear the current status message
    fn clear_status(&mut self) {
        self.buffer_mut().set_status_message(String::new());
    }

    /// Set a temporary status message that auto-clears after a duration
    fn set_temporary_status(&mut self, message: impl Into<String>, _duration_ms: u64) {
        // For now, just set the status - timer functionality can be added later
        self.set_status(message);
    }

    /// Format and set a query execution status
    fn set_query_status(&mut self, rows: usize, columns: usize, elapsed_ms: u64) {
        self.set_status(format!(
            "Query executed: {} rows, {} columns ({} ms)",
            rows, columns, elapsed_ms
        ));
    }

    /// Set a search status with match count
    fn set_search_status(&mut self, pattern: &str, current: usize, total: usize) {
        if total == 0 {
            self.set_status(format!("/{} - no matches", pattern));
        } else {
            self.set_status(format!("Match {}/{} for '{}'", current, total, pattern));
        }
    }

    /// Set a filter status
    fn set_filter_status(&mut self, active: bool, matches: usize) {
        if active {
            self.set_status(format!("Filter active: {} matches", matches));
        } else {
            self.set_status("Filter cleared");
        }
    }

    /// Set a column operation status
    fn set_column_status(&mut self, operation: &str, column_name: &str) {
        self.set_status(format!("{}: {}", operation, column_name));
    }

    /// Set a navigation status
    fn set_navigation_status(&mut self, description: &str) {
        self.set_status(description);
    }

    /// Set a mode change status
    fn set_mode_status(&mut self, new_mode: &str) {
        self.set_status(format!("{} mode", new_mode));
    }

    /// Set a yank operation status
    fn set_yank_status(&mut self, target: &str, size: usize) {
        self.set_status(format!("Yanked {} ({} items)", target, size));
    }

    /// Set a chord mode status
    fn set_chord_status(&mut self, chord: &str, completions: &[String]) {
        if completions.is_empty() {
            self.set_status(format!("Chord: {} (no completions)", chord));
        } else {
            let completion_str = completions.join(", ");
            self.set_status(format!("Chord: {} - available: {}", chord, completion_str));
        }
    }

    /// Set a history search status
    fn set_history_status(&mut self, matches: usize) {
        self.set_status(format!("History search: {} matches", matches));
    }

    /// Handle status from a result type
    fn handle_result_status<T>(
        &mut self,
        result: Result<T>,
        success_msg: &str,
        error_context: &str,
    ) -> Option<T> {
        match result {
            Ok(value) => {
                self.set_success(success_msg);
                Some(value)
            }
            Err(e) => {
                self.set_error(error_context, e);
                None
            }
        }
    }
}
