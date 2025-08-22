use crate::buffer::{AppMode, BufferAPI};
use crate::debug::debug_trace::{DebugSection, DebugSectionBuilder, DebugTrace, Priority};
use std::sync::Arc;

/// Debug trace implementation for Buffer
pub struct BufferDebugProvider {
    buffer: Arc<dyn BufferAPI>,
}

impl BufferDebugProvider {
    pub fn new(buffer: Arc<dyn BufferAPI>) -> Self {
        Self { buffer }
    }
}

impl DebugTrace for BufferDebugProvider {
    fn name(&self) -> &str {
        "Buffer"
    }

    fn debug_sections(&self) -> Vec<DebugSection> {
        let mut builder = DebugSectionBuilder::new();

        // Main buffer state section
        builder.add_section("BUFFER STATE", "", Priority::BUFFER);

        // Basic buffer info
        builder.add_field("Buffer Name", self.buffer.get_name());
        builder.add_field("Current Mode", format!("{:?}", self.buffer.get_mode()));

        // Input state
        let input_text = self.buffer.get_input_text();
        builder.add_field(
            "Input Text",
            if input_text.is_empty() {
                "(empty)".to_string()
            } else {
                format!("'{}' ({} chars)", input_text, input_text.len())
            },
        );

        // Navigation state
        if let Some(selected_row) = self.buffer.get_selected_row() {
            builder.add_field("Selected Row", selected_row);
        } else {
            builder.add_field("Selected Row", "None");
        }
        builder.add_field("Current Column", self.buffer.get_current_column());

        // Query information
        let last_query = self.buffer.get_last_query();
        if !last_query.is_empty() {
            builder.add_field(
                "Last Query",
                format!(
                    "'{}' ({} chars)",
                    if last_query.len() > 50 {
                        format!("{}...", &last_query[..50])
                    } else {
                        last_query.clone()
                    },
                    last_query.len()
                ),
            );
        }

        // Results information
        if let Some(dataview) = self.buffer.get_dataview() {
            builder.add_field("Has DataView", "Yes");
            builder.add_field("Result Rows", dataview.row_count());
            builder.add_field("Result Columns", dataview.column_count());
        } else {
            builder.add_field("Has DataView", "No");
        }

        // Scroll state
        let (scroll_row, scroll_col) = self.buffer.get_scroll_offset();
        builder.add_field(
            "Scroll Offset",
            format!("row={}, col={}", scroll_row, scroll_col),
        );

        // Viewport lock state
        builder.add_field(
            "Viewport Lock",
            if self.buffer.is_viewport_lock() {
                if let Some(lock_row) = self.buffer.get_viewport_lock_row() {
                    format!("Locked at row {}", lock_row)
                } else {
                    "Locked (no specific row)".to_string()
                }
            } else {
                "Unlocked".to_string()
            },
        );

        // Filter state
        let filter_pattern = self.buffer.get_filter_pattern();
        if !filter_pattern.is_empty() {
            builder.add_field("Filter Pattern", format!("'{}'", filter_pattern));
            builder.add_field("Filter Active", self.buffer.is_filter_active());
        }

        // Status message
        let status_msg = self.buffer.get_status_message();
        if !status_msg.is_empty() {
            builder.add_field("Status Message", status_msg);
        }

        builder.build()
    }

    fn debug_summary(&self) -> Option<String> {
        let mode = self.buffer.get_mode();
        let has_data = self.buffer.get_dataview().is_some();
        let input_len = self.buffer.get_input_text().len();

        Some(format!(
            "Mode: {:?}, Input: {} chars, Data: {}",
            mode,
            input_len,
            if has_data { "Yes" } else { "No" }
        ))
    }

    fn is_active(&self) -> bool {
        true
    }
}

/// Debug provider for multiple buffers (BufferManager)
pub struct BufferManagerDebugProvider {
    buffers: Vec<Arc<dyn BufferAPI>>,
    current_index: usize,
}

impl BufferManagerDebugProvider {
    pub fn new(buffers: Vec<Arc<dyn BufferAPI>>, current_index: usize) -> Self {
        Self {
            buffers,
            current_index,
        }
    }
}

impl DebugTrace for BufferManagerDebugProvider {
    fn name(&self) -> &str {
        "BufferManager"
    }

    fn debug_sections(&self) -> Vec<DebugSection> {
        let mut builder = DebugSectionBuilder::new();

        builder.add_section("BUFFER MANAGER", "", Priority::BUFFER - 10);

        builder.add_field("Total Buffers", self.buffers.len());
        builder.add_field("Current Buffer Index", self.current_index);

        if let Some(current) = self.buffers.get(self.current_index) {
            builder.add_field("Current Buffer Name", current.get_name());
        }

        builder.add_line("");
        builder.add_line("All Buffers:");
        for (idx, buffer) in self.buffers.iter().enumerate() {
            let marker = if idx == self.current_index {
                " * "
            } else {
                "   "
            };
            let mode = buffer.get_mode();
            let has_data = buffer.get_dataview().is_some();
            builder.add_line(format!(
                "{} [{}] {} - Mode: {:?}, Data: {}",
                marker,
                idx,
                buffer.get_name(),
                mode,
                if has_data { "Yes" } else { "No" }
            ));
        }

        builder.build()
    }

    fn debug_summary(&self) -> Option<String> {
        Some(format!(
            "{} buffers, current: #{} ({})",
            self.buffers.len(),
            self.current_index,
            self.buffers
                .get(self.current_index)
                .map(|b| b.get_name())
                .unwrap_or_else(|| "none".to_string())
        ))
    }
}
