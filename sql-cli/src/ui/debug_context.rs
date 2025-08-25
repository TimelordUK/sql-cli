//! Debug context trait for extracting debug functionality from the main TUI
//!
//! This module provides a trait that encapsulates all debug-related operations,
//! allowing them to be organized separately from the main TUI logic.

use crate::buffer::{AppMode, Buffer, BufferAPI, BufferManager};
use crate::ui::shadow_state::ShadowStateManager;
use crate::ui::viewport_manager::ViewportManager;
use crate::widgets::debug_widget::DebugWidget;
use std::cell::RefCell;

/// Context trait for debug-related functionality
/// This extracts debug operations into a cohesive interface
pub trait DebugContext {
    // Core accessors needed by debug operations
    fn buffer(&self) -> &dyn BufferAPI;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn get_debug_widget(&self) -> &DebugWidget;
    fn get_debug_widget_mut(&mut self) -> &mut DebugWidget;
    fn get_shadow_state(&self) -> &RefCell<ShadowStateManager>;

    // Additional accessors for viewport and buffer management
    fn get_buffer_manager(&self) -> &BufferManager;
    fn get_viewport_manager(&self) -> &RefCell<Option<ViewportManager>>;

    // Additional accessors for the full debug implementation
    fn get_navigation_timings(&self) -> &Vec<String>;
    fn get_render_timings(&self) -> &Vec<String>;
    fn debug_current_buffer(&mut self);
    fn get_input_cursor(&self) -> usize;
    fn get_visual_cursor(&self) -> (usize, usize);
    fn get_input_text(&self) -> String;

    // The complete toggle_debug_mode implementation from EnhancedTuiApp
    fn toggle_debug_mode(&mut self) {
        // Check if we're exiting debug mode
        if self.buffer().get_mode() == AppMode::Debug {
            // Exit debug mode - use a helper to avoid borrow issues
            self.set_mode_via_shadow_state(AppMode::Command, "debug_toggle_exit");
            return;
        }

        // Entering debug mode - collect all the data we need
        let (
            previous_mode,
            last_query,
            input_text,
            selected_row,
            current_column,
            results_count,
            filtered_count,
        ) = self.collect_current_state();

        // Switch to debug mode - use a helper to avoid borrow issues
        self.set_mode_via_shadow_state(AppMode::Debug, "debug_toggle_enter");

        // Generate full debug information
        self.debug_current_buffer();
        let cursor_pos = self.get_input_cursor();
        let visual_cursor = self.get_visual_cursor().1;
        let query = self.get_input_text();

        // Use the appropriate query for parser debug based on mode
        let query_for_parser = if previous_mode == AppMode::Results && !last_query.is_empty() {
            last_query.clone()
        } else if !query.is_empty() {
            query.clone()
        } else if !last_query.is_empty() {
            last_query.clone()
        } else {
            query.clone()
        };

        // Generate debug info using helper methods
        let mut debug_info = self.debug_generate_parser_info(&query_for_parser);

        // Add comprehensive buffer state
        debug_info.push_str(&self.debug_generate_buffer_state(
            previous_mode,
            &last_query,
            &input_text,
            cursor_pos,
            visual_cursor,
        ));

        // Add results state if in Results mode
        debug_info.push_str(&self.debug_generate_results_state(
            results_count,
            filtered_count,
            selected_row,
            current_column,
        ));

        // Add DataTable schema and DataView state
        debug_info.push_str(&self.debug_generate_datatable_schema());
        debug_info.push_str(&self.debug_generate_dataview_state());

        // Add memory tracking history
        debug_info.push_str(&self.debug_generate_memory_info());

        // Add navigation timing statistics
        debug_info.push_str(&self.format_navigation_timing());

        // Add render timing statistics
        debug_info.push_str(&self.format_render_timing());

        // Add viewport and navigation information
        debug_info.push_str(&self.debug_generate_viewport_state());
        debug_info.push_str(&self.debug_generate_navigation_state());

        // Add buffer manager state info
        debug_info.push_str(&self.format_buffer_manager_state());

        // Add viewport efficiency metrics
        debug_info.push_str(&self.debug_generate_viewport_efficiency());

        // Add key chord handler debug info
        debug_info.push_str(&self.debug_generate_key_chord_info());

        // Add search modes widget debug info
        debug_info.push_str(&self.debug_generate_search_modes_info());

        // Add column search state if active
        debug_info.push_str(&self.debug_generate_column_search_state());

        // Add trace logs from ring buffer
        debug_info.push_str(&self.debug_generate_trace_logs());

        // Add DebugService logs (our StateManager logs!)
        debug_info.push_str(&self.debug_generate_state_logs());

        // Add AppStateContainer debug dump if available
        debug_info.push_str(&self.debug_generate_state_container_info());

        // Add Shadow State debug info
        debug_info.push_str("\n========== SHADOW STATE MANAGER ==========\n");
        debug_info.push_str(&self.get_shadow_state().borrow().debug_info());
        debug_info.push_str("\n==========================================\n");

        // Store the debug info in the widget
        self.get_debug_widget_mut().set_content(debug_info);
    }

    // Required helper methods that implementations must provide
    fn get_buffer_mut_if_available(&mut self) -> Option<&mut Buffer>;
    fn set_mode_via_shadow_state(&mut self, mode: AppMode, trigger: &str);
    fn collect_current_state(
        &self,
    ) -> (AppMode, String, String, Option<usize>, usize, usize, usize);
    fn format_buffer_manager_state(&self) -> String;
    fn debug_generate_viewport_efficiency(&self) -> String;
    fn debug_generate_key_chord_info(&self) -> String;
    fn debug_generate_search_modes_info(&self) -> String;
    fn debug_generate_state_container_info(&self) -> String;

    // Helper methods with default implementations
    fn format_navigation_timing(&self) -> String {
        let mut result = String::from("\n========== NAVIGATION TIMING ==========\n");
        let timings = self.get_navigation_timings();
        if !timings.is_empty() {
            result.push_str(&format!("Last {} navigation timings:\n", timings.len()));
            for timing in timings {
                result.push_str(&format!("  {}\n", timing));
            }
            // Calculate average
            let total_ms: f64 = timings
                .iter()
                .filter_map(|s| self.debug_extract_timing(s))
                .sum();
            if timings.len() > 0 {
                let avg_ms = total_ms / timings.len() as f64;
                result.push_str(&format!("Average navigation time: {:.3}ms\n", avg_ms));
            }
        } else {
            result.push_str("No navigation timing data yet (press j/k to navigate)\n");
        }
        result
    }

    fn format_render_timing(&self) -> String {
        let mut result = String::from("\n========== RENDER TIMING ==========\n");
        let timings = self.get_render_timings();
        if !timings.is_empty() {
            result.push_str(&format!("Last {} render timings:\n", timings.len()));
            for timing in timings {
                result.push_str(&format!("  {}\n", timing));
            }
            // Calculate average
            let total_ms: f64 = timings
                .iter()
                .filter_map(|s| self.debug_extract_timing(s))
                .sum();
            if timings.len() > 0 {
                let avg_ms = total_ms / timings.len() as f64;
                result.push_str(&format!("Average render time: {:.3}ms\n", avg_ms));
            }
        } else {
            result.push_str("No render timing data yet\n");
        }
        result
    }

    fn debug_extract_timing(&self, s: &str) -> Option<f64> {
        // Extract timing value from a string like "Navigation: 1.234ms"
        if let Some(ms_pos) = s.find("ms") {
            let start = s[..ms_pos].rfind(' ').map(|p| p + 1).unwrap_or(0);
            s[start..ms_pos].parse().ok()
        } else {
            None
        }
    }

    // Collect all debug information (simplified version for the trait default)
    fn collect_debug_info(&self) -> String;

    // Debug generation methods with default implementations where possible

    // Simple methods that don't need TUI state - can be default implementations
    fn debug_generate_memory_info(&self) -> String {
        format!(
            "\n========== MEMORY USAGE ==========\n\
            Current Memory: {} MB\n{}",
            crate::utils::memory_tracker::get_memory_mb(),
            crate::utils::memory_tracker::format_memory_history()
        )
    }

    // Note: debug_extract_timing is already defined above

    // Simple formatting methods that can have default implementations
    fn debug_generate_buffer_state(
        &self,
        mode: AppMode,
        last_query: &str,
        input_text: &str,
        cursor_pos: usize,
        visual_cursor: usize,
    ) -> String {
        format!(
            "\n========== BUFFER STATE ==========\n\
            Current Mode: {:?}\n\
            Last Executed Query: '{}'\n\
            Input Text: '{}'\n\
            Input Cursor: {}\n\
            Visual Cursor: {}\n",
            mode, last_query, input_text, cursor_pos, visual_cursor
        )
    }

    fn debug_generate_results_state(
        &self,
        results_count: usize,
        filtered_count: usize,
        selected_row: Option<usize>,
        current_column: usize,
    ) -> String {
        format!(
            "\n========== RESULTS STATE ==========\n\
            Total Results: {}\n\
            Filtered Results: {}\n\
            Selected Row: {:?}\n\
            Current Column: {}\n",
            results_count, filtered_count, selected_row, current_column
        )
    }

    // Viewport state can be a default implementation now that we have buffer_manager and viewport_manager
    fn debug_generate_viewport_state(&self) -> String {
        let mut debug_info = String::new();
        if let Some(buffer) = self.get_buffer_manager().current() {
            debug_info.push_str("\n========== VIEWPORT STATE ==========\n");
            let (scroll_row, scroll_col) = buffer.get_scroll_offset();
            debug_info.push_str(&format!(
                "Scroll Offset: row={}, col={}\n",
                scroll_row, scroll_col
            ));
            debug_info.push_str(&format!(
                "Current Column: {}\n",
                buffer.get_current_column()
            ));
            debug_info.push_str(&format!("Selected Row: {:?}\n", buffer.get_selected_row()));
            debug_info.push_str(&format!("Viewport Lock: {}\n", buffer.is_viewport_lock()));
            if let Some(lock_row) = buffer.get_viewport_lock_row() {
                debug_info.push_str(&format!("Viewport Lock Row: {}\n", lock_row));
            }

            // Add ViewportManager crosshair position
            if let Some(ref viewport_manager) = *self.get_viewport_manager().borrow() {
                let visual_row = viewport_manager.get_crosshair_row();
                let visual_col = viewport_manager.get_crosshair_col();
                debug_info.push_str(&format!(
                    "ViewportManager Crosshair (visual): row={}, col={}\n",
                    visual_row, visual_col
                ));

                // Also show viewport-relative position
                if let Some((viewport_row, viewport_col)) =
                    viewport_manager.get_crosshair_viewport_position()
                {
                    debug_info.push_str(&format!(
                        "Crosshair in viewport (relative): row={}, col={}\n",
                        viewport_row, viewport_col
                    ));
                }
            }

            // Show visible area calculation
            if let Some(dataview) = buffer.get_dataview() {
                let total_rows = dataview.row_count();
                let total_cols = dataview.column_count();
                let visible_rows = buffer.get_last_visible_rows();
                debug_info.push_str(&format!("\nVisible Area:\n"));
                debug_info.push_str(&format!(
                    "  Total Data: {} rows × {} columns\n",
                    total_rows, total_cols
                ));
                debug_info.push_str(&format!("  Visible Rows in Terminal: {}\n", visible_rows));

                // Calculate what section is being viewed
                if total_rows > 0 && visible_rows > 0 {
                    let start_row = scroll_row.min(total_rows.saturating_sub(1));
                    let end_row = (scroll_row + visible_rows).min(total_rows);
                    let percent_start = (start_row as f64 / total_rows as f64 * 100.0) as u32;
                    let percent_end = (end_row as f64 / total_rows as f64 * 100.0) as u32;
                    debug_info.push_str(&format!(
                        "  Viewing rows {}-{} ({}%-{}% of data)\n",
                        start_row + 1,
                        end_row,
                        percent_start,
                        percent_end
                    ));
                }

                if total_cols > 0 {
                    let visible_cols_estimate = 10; // Estimate based on typical column widths
                    let start_col = scroll_col.min(total_cols.saturating_sub(1));
                    let end_col = (scroll_col + visible_cols_estimate).min(total_cols);
                    debug_info.push_str(&format!(
                        "  Viewing columns {}-{} of {}\n",
                        start_col + 1,
                        end_col,
                        total_cols
                    ));
                }
            }
        }
        debug_info
    }

    // Methods that need to be implemented by the TUI (need access to TUI fields)
    fn debug_generate_parser_info(&self, query: &str) -> String;
    fn debug_generate_datatable_schema(&self) -> String;
    fn debug_generate_dataview_state(&self) -> String;
    fn debug_generate_navigation_state(&self) -> String;
    fn debug_generate_column_search_state(&self) -> String;
    fn debug_generate_trace_logs(&self) -> String;
    fn debug_generate_state_logs(&self) -> String;
}
