//! Debug context trait for extracting debug functionality from the main TUI
//!
//! This module provides a trait that encapsulates all debug-related operations,
//! allowing them to be organized separately from the main TUI logic.

use crate::app_state_container::AppStateContainer;
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
    fn get_state_container(&self) -> &AppStateContainer;
    fn get_state_container_mut(&mut self) -> &mut AppStateContainer;

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
        self.get_debug_widget_mut().set_content(debug_info.clone());

        // Copy to clipboard
        match self
            .get_state_container_mut()
            .write_to_clipboard(&debug_info)
        {
            Ok(_) => {
                let status_msg = format!(
                    "DEBUG INFO copied to clipboard ({} chars)!",
                    debug_info.len()
                );
                self.buffer_mut().set_status_message(status_msg);
            }
            Err(e) => {
                let status_msg = format!("Clipboard error: {}", e);
                self.buffer_mut().set_status_message(status_msg);
            }
        }
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

    // DataView state can be a default implementation now that we have buffer_manager
    fn debug_generate_dataview_state(&self) -> String {
        let mut debug_info = String::new();
        if let Some(buffer) = self.get_buffer_manager().current() {
            if let Some(dataview) = buffer.get_dataview() {
                debug_info.push_str("\n========== DATAVIEW STATE ==========\n");

                // Add the detailed column mapping info
                debug_info.push_str(&dataview.get_column_debug_info());
                debug_info.push_str("\n");

                // Show visible columns in order with both indices
                let visible_columns = dataview.column_names();
                let column_mappings = dataview.get_column_index_mapping();
                debug_info.push_str(&format!(
                    "Visible Columns ({}) with Index Mapping:\n",
                    visible_columns.len()
                ));
                for (visible_idx, col_name, datatable_idx) in &column_mappings {
                    debug_info.push_str(&format!(
                        "  V[{:3}] → DT[{:3}] : {}\n",
                        visible_idx, datatable_idx, col_name
                    ));
                }

                // Show row information
                debug_info.push_str(&format!("\nVisible Rows: {}\n", dataview.row_count()));

                // Show internal visible_columns array (source column indices)
                debug_info.push_str("\n--- Internal State ---\n");

                // Get the visible_columns indices from DataView
                let visible_indices = dataview.get_visible_column_indices();
                debug_info.push_str(&format!("visible_columns array: {:?}\n", visible_indices));

                // Show pinned columns
                let pinned_names = dataview.get_pinned_column_names();
                if !pinned_names.is_empty() {
                    debug_info.push_str(&format!("Pinned Columns ({}):\n", pinned_names.len()));
                    for (idx, name) in pinned_names.iter().enumerate() {
                        // Find source index for this pinned column
                        let source_idx = dataview.source().get_column_index(name).unwrap_or(999);
                        debug_info.push_str(&format!(
                            "  [{}] {} (source_idx: {})\n",
                            idx, name, source_idx
                        ));
                    }
                } else {
                    debug_info.push_str("Pinned Columns: None\n");
                }

                // Show sort state
                let sort_state = dataview.get_sort_state();
                match sort_state.order {
                    crate::data::data_view::SortOrder::None => {
                        debug_info.push_str("Sort State: None\n");
                    }
                    crate::data::data_view::SortOrder::Ascending => {
                        if let Some(col_idx) = sort_state.column {
                            let col_name = visible_columns
                                .get(col_idx)
                                .map(|s| s.as_str())
                                .unwrap_or("unknown");
                            debug_info.push_str(&format!(
                                "Sort State: Ascending on column '{}' (idx: {})\n",
                                col_name, col_idx
                            ));
                        }
                    }
                    crate::data::data_view::SortOrder::Descending => {
                        if let Some(col_idx) = sort_state.column {
                            let col_name = visible_columns
                                .get(col_idx)
                                .map(|s| s.as_str())
                                .unwrap_or("unknown");
                            debug_info.push_str(&format!(
                                "Sort State: Descending on column '{}' (idx: {})\n",
                                col_name, col_idx
                            ));
                        }
                    }
                }
            }
        }
        debug_info
    }

    // DataTable schema can be a default implementation now that we have buffer_manager
    fn debug_generate_datatable_schema(&self) -> String {
        let mut debug_info = String::new();
        if let Some(buffer) = self.get_buffer_manager().current() {
            if let Some(dataview) = buffer.get_dataview() {
                let datatable = dataview.source();
                debug_info.push_str("\n========== DATATABLE SCHEMA ==========\n");
                debug_info.push_str(&datatable.get_schema_summary());
            }
        }
        debug_info
    }

    // Rendering methods with default implementations
    fn render_debug(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.get_debug_widget().render(f, area, AppMode::Debug);
    }

    fn render_pretty_query(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.get_debug_widget()
            .render(f, area, AppMode::PrettyQuery);
    }

    // Navigation state debug - can be default implementation using state_container
    fn debug_generate_navigation_state(&self) -> String {
        let mut debug_info = String::new();
        debug_info.push_str("\n========== NAVIGATION DEBUG ==========\n");
        let current_column = self.get_state_container().get_current_column();
        let scroll_offset = self.get_state_container().get_scroll_offset();
        let nav_state = self.get_state_container().navigation();

        debug_info.push_str(&format!("Buffer Column Position: {}\n", current_column));
        debug_info.push_str(&format!(
            "Buffer Scroll Offset: row={}, col={}\n",
            scroll_offset.0, scroll_offset.1
        ));
        debug_info.push_str(&format!(
            "NavigationState Column: {}\n",
            nav_state.selected_column
        ));
        debug_info.push_str(&format!(
            "NavigationState Row: {:?}\n",
            nav_state.selected_row
        ));
        debug_info.push_str(&format!(
            "NavigationState Scroll Offset: row={}, col={}\n",
            nav_state.scroll_offset.0, nav_state.scroll_offset.1
        ));

        // Show if synchronization is correct
        if current_column != nav_state.selected_column {
            debug_info.push_str(&format!(
                "⚠️  WARNING: Column mismatch! Buffer={}, Nav={}\n",
                current_column, nav_state.selected_column
            ));
        }
        if scroll_offset.1 != nav_state.scroll_offset.1 {
            debug_info.push_str(&format!(
                "⚠️  WARNING: Scroll column mismatch! Buffer={}, Nav={}\n",
                scroll_offset.1, nav_state.scroll_offset.1
            ));
        }

        debug_info.push_str("\n--- Navigation Flow ---\n");
        debug_info.push_str(
            "(Enable RUST_LOG=sql_cli::ui::viewport_manager=debug,navigation=debug to see flow)\n",
        );

        // Show pinned column info for navigation context
        if let Some(dataview) = self.get_state_container().get_buffer_dataview() {
            let pinned_count = dataview.get_pinned_columns().len();
            let pinned_names = dataview.get_pinned_column_names();
            debug_info.push_str(&format!("Pinned Column Count: {}\n", pinned_count));
            if !pinned_names.is_empty() {
                debug_info.push_str(&format!("Pinned Column Names: {:?}\n", pinned_names));
            }
            debug_info.push_str(&format!("First Scrollable Column: {}\n", pinned_count));

            // Show if current column is in pinned or scrollable area
            if current_column < pinned_count {
                debug_info.push_str(&format!(
                    "Current Position: PINNED area (column {})\n",
                    current_column
                ));
            } else {
                debug_info.push_str(&format!(
                    "Current Position: SCROLLABLE area (column {}, scrollable index {})\n",
                    current_column,
                    current_column - pinned_count
                ));
            }

            // Show display column order
            let display_columns = dataview.get_display_columns();
            debug_info.push_str(&format!("\n--- COLUMN ORDERING ---\n"));
            debug_info.push_str(&format!(
                "Display column order (first 10): {:?}\n",
                &display_columns[..display_columns.len().min(10)]
            ));
            if display_columns.len() > 10 {
                debug_info.push_str(&format!(
                    "... and {} more columns\n",
                    display_columns.len() - 10
                ));
            }

            // Find current column in display order
            if let Some(display_idx) = display_columns
                .iter()
                .position(|&idx| idx == current_column)
            {
                debug_info.push_str(&format!(
                    "Current column {} is at display index {}/{}\n",
                    current_column,
                    display_idx,
                    display_columns.len()
                ));

                // Show what happens on next move
                if display_idx + 1 < display_columns.len() {
                    let next_col = display_columns[display_idx + 1];
                    debug_info.push_str(&format!(
                        "Next 'l' press should move to column {} (display index {})\n",
                        next_col,
                        display_idx + 1
                    ));
                } else {
                    debug_info.push_str("Next 'l' press should wrap to first column\n");
                }
            } else {
                debug_info.push_str(&format!(
                    "WARNING: Current column {} not found in display order!\n",
                    current_column
                ));
            }
        }
        debug_info.push_str("==========================================\n");
        debug_info
    }

    // Methods that need to be implemented by the TUI (need access to TUI fields)
    fn debug_generate_parser_info(&self, query: &str) -> String;
    fn debug_generate_column_search_state(&self) -> String;
    fn debug_generate_trace_logs(&self) -> String;
    fn debug_generate_state_logs(&self) -> String;
}
