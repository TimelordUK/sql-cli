//! Table Widget Manager - Centralized table state and rendering
//!
//! This manager owns the table widget state and ensures all updates
//! go through a single interface, properly triggering re-renders.

use crate::data::data_view::DataView;
use crate::ui::render_state::RenderState;
use crate::ui::viewport_manager::ViewportManager;
use std::sync::Arc;
use tracing::{debug, info, trace};

/// Position in the table (row, column)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TablePosition {
    pub row: usize,
    pub column: usize,
}

/// Table widget manager that owns all table-related state
pub struct TableWidgetManager {
    /// Current cursor/crosshair position
    position: TablePosition,
    /// Previous position (for clearing old crosshair)
    previous_position: Option<TablePosition>,
    /// Viewport manager for column/row visibility
    viewport_manager: Option<ViewportManager>,
    /// Render state tracker
    render_state: RenderState,
    /// Current data view
    dataview: Option<Arc<DataView>>,
    /// Scroll offset (row, column)
    scroll_offset: (usize, usize),
}

impl TableWidgetManager {
    /// Create a new table widget manager
    pub fn new() -> Self {
        Self {
            position: TablePosition { row: 0, column: 0 },
            previous_position: None,
            viewport_manager: None,
            render_state: RenderState::new(),
            dataview: None,
            scroll_offset: (0, 0),
        }
    }

    /// Set the data view for the table
    pub fn set_dataview(&mut self, dataview: Arc<DataView>) {
        debug!("TableWidgetManager: Setting new dataview");
        self.dataview = Some(dataview.clone());

        // Update viewport manager with new dataview
        if let Some(ref mut vm) = self.viewport_manager {
            vm.set_dataview(dataview);
        } else {
            self.viewport_manager = Some(ViewportManager::new(dataview));
        }

        self.render_state.on_data_change();
    }

    /// Navigate to a specific position
    pub fn navigate_to(&mut self, row: usize, column: usize) {
        let old_pos = self.position;
        info!(
            "TableWidgetManager: Navigate from ({}, {}) to ({}, {})",
            old_pos.row, old_pos.column, row, column
        );

        // Store previous position for clearing
        if self.position.row != row || self.position.column != column {
            self.previous_position = Some(self.position);
            self.position = TablePosition { row, column };

            info!("TableWidgetManager: Position changed, marking dirty for re-render");

            // Update viewport manager crosshair
            if let Some(ref mut vm) = self.viewport_manager {
                vm.set_crosshair(row, column);
                info!(
                    "TableWidgetManager: Updated ViewportManager crosshair to ({}, {})",
                    row, column
                );

                // Calculate if we need to scroll
                let viewport_height = 79; // TODO: Get from actual terminal size
                let viewport_width = 100; // TODO: Get from actual terminal size

                // Update scroll offset if needed
                let new_row_offset = if row < self.scroll_offset.0 {
                    info!(
                        "TableWidgetManager: Row {} is above viewport, scrolling up",
                        row
                    );
                    row // Scroll up
                } else if row >= self.scroll_offset.0 + viewport_height {
                    let centered = row.saturating_sub(viewport_height / 2);
                    info!(
                        "TableWidgetManager: Row {} is below viewport, centering at {}",
                        row, centered
                    );
                    centered // Center it
                } else {
                    trace!(
                        "TableWidgetManager: Row {} is visible in current viewport",
                        row
                    );
                    self.scroll_offset.0 // Keep current
                };

                if new_row_offset != self.scroll_offset.0 {
                    info!(
                        "TableWidgetManager: Changing scroll offset from {} to {}",
                        self.scroll_offset.0, new_row_offset
                    );
                    self.scroll_offset.0 = new_row_offset;
                    vm.set_viewport(
                        new_row_offset,
                        self.scroll_offset.1,
                        viewport_width as u16,
                        viewport_height as u16,
                    );
                }
            }

            // Mark for re-render
            self.render_state.on_navigation_change();
            info!("TableWidgetManager: State marked dirty, will trigger re-render");
        } else {
            trace!("TableWidgetManager: Position unchanged, no re-render needed");
        }
    }

    /// Move cursor by relative amount
    pub fn move_cursor(&mut self, row_delta: isize, col_delta: isize) {
        let new_row = (self.position.row as isize + row_delta).max(0) as usize;
        let new_col = (self.position.column as isize + col_delta).max(0) as usize;

        // Clamp to data bounds
        if let Some(ref dv) = self.dataview {
            let max_row = dv.row_count().saturating_sub(1);
            let max_col = dv.column_count().saturating_sub(1);
            let clamped_row = new_row.min(max_row);
            let clamped_col = new_col.min(max_col);

            self.navigate_to(clamped_row, clamped_col);
        }
    }

    /// Handle search result navigation
    pub fn navigate_to_search_match(&mut self, row: usize, column: usize) {
        info!(
            "TableWidgetManager: Navigate to search match at ({}, {})",
            row, column
        );

        // Force immediate render for search results
        self.navigate_to(row, column);
        self.render_state.on_search_update();
    }

    /// Check if render is needed
    pub fn needs_render(&self) -> bool {
        self.render_state.needs_render()
    }

    /// Mark that render has completed
    pub fn rendered(&mut self) {
        self.render_state.rendered();
        // Clear previous position after successful render
        self.previous_position = None;
    }

    /// Get current position
    pub fn position(&self) -> TablePosition {
        self.position
    }

    /// Get previous position (for clearing)
    pub fn previous_position(&self) -> Option<TablePosition> {
        self.previous_position
    }

    /// Force a re-render
    pub fn force_render(&mut self) {
        debug!("TableWidgetManager: Forcing render");
        self.render_state.force_render();
    }

    /// Set high-frequency mode for responsive updates
    pub fn set_high_frequency_mode(&mut self, enabled: bool) {
        self.render_state.set_high_frequency_mode(enabled);
    }

    /// Get the viewport manager
    pub fn viewport_manager(&self) -> Option<&ViewportManager> {
        self.viewport_manager.as_ref()
    }

    /// Get mutable viewport manager
    pub fn viewport_manager_mut(&mut self) -> Option<&mut ViewportManager> {
        self.viewport_manager.as_mut()
    }

    /// Handle debounced search action
    pub fn on_debounced_search(&mut self, row: usize, column: usize) {
        info!(
            "TableWidgetManager: Debounced search navigating to ({}, {})",
            row, column
        );
        self.navigate_to(row, column);
        self.render_state.on_debounced_action();
    }

    /// Get render state for debugging
    pub fn render_state(&self) -> &RenderState {
        &self.render_state
    }

    /// Update scroll offset
    pub fn set_scroll_offset(&mut self, row_offset: usize, col_offset: usize) {
        if self.scroll_offset != (row_offset, col_offset) {
            debug!(
                "TableWidgetManager: Scroll offset changed to ({}, {})",
                row_offset, col_offset
            );
            self.scroll_offset = (row_offset, col_offset);
            self.render_state.on_navigation_change();
        }
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> (usize, usize) {
        self.scroll_offset
    }

    /// Check and perform render if needed
    /// Returns true if render was performed
    pub fn check_and_render<F>(&mut self, mut render_fn: F) -> bool
    where
        F: FnMut(&TablePosition, &RenderState),
    {
        if self.needs_render() {
            info!("═══════════════════════════════════════════════════════");
            info!("TableWidgetManager: RENDERING TABLE");
            info!(
                "  Crosshair position: ({}, {})",
                self.position.row, self.position.column
            );
            info!(
                "  Scroll offset: ({}, {})",
                self.scroll_offset.0, self.scroll_offset.1
            );
            info!("  Render reason: {:?}", self.render_state.dirty_reason());
            if let Some(prev) = self.previous_position {
                info!("  Previous position: ({}, {})", prev.row, prev.column);
            }
            info!("═══════════════════════════════════════════════════════");

            // Call the actual render function
            render_fn(&self.position, &self.render_state);

            // Mark as rendered
            self.rendered();

            info!("TableWidgetManager: Render complete");
            true
        } else {
            trace!("TableWidgetManager: No render needed (not dirty or throttled)");
            false
        }
    }
}
