use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, BufferAPI};
use crate::ui::viewport_manager::{ColumnOperationResult, NavigationResult, ViewportManager};
use std::cell::RefCell;
use std::sync::Arc;

/// Trait that provides column operation behavior for TUI components
/// This extracts column operation methods from EnhancedTui to reduce coupling
pub trait ColumnBehavior {
    // Required methods - these provide access to TUI internals
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>>;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn buffer(&self) -> &dyn BufferAPI;
    fn state_container(&self) -> &Arc<AppStateContainer>;

    // Mode checking - will be implemented by TUI to use shadow state
    fn is_in_results_mode(&self) -> bool {
        // Default implementation for compatibility
        self.buffer().get_mode() == AppMode::Results
    }

    // Helper method to apply column navigation results
    fn apply_column_navigation_result(&mut self, result: NavigationResult, direction: &str) {
        // Get the visual position from ViewportManager after navigation
        let visual_position = {
            let viewport_borrow = self.viewport_manager().borrow();
            viewport_borrow
                .as_ref()
                .map(|vm| vm.get_crosshair_col())
                .unwrap_or(0)
        };

        // Update Buffer's current column
        self.buffer_mut().set_current_column(visual_position);

        // Update navigation state
        self.state_container().navigation_mut().selected_column = visual_position;

        // Update scroll offset if viewport changed
        if result.viewport_changed {
            let mut offset = self.buffer().get_scroll_offset();
            offset.1 = result.scroll_offset;
            self.buffer_mut().set_scroll_offset(offset);

            // Also update the navigation state scroll offset
            self.state_container().navigation_mut().scroll_offset.1 = result.scroll_offset;
        }

        // Set status message based on direction
        let message = match direction {
            "first" => "Moved to first column".to_string(),
            "last" => "Moved to last column".to_string(),
            _ => format!("Moved to column {}", visual_position),
        };
        self.buffer_mut().set_status_message(message);
    }

    // Helper method that stays in the trait
    fn apply_column_operation_result(&mut self, result: ColumnOperationResult) {
        if !result.success {
            if !result.description.is_empty() {
                self.buffer_mut().set_status_message(result.description);
            }
            return;
        }

        // Sync DataView if updated
        if let Some(dataview) = result.updated_dataview {
            self.buffer_mut().set_dataview(Some(dataview));
        }

        // Update navigation state if column position changed
        if let Some(new_col) = result.new_column_position {
            // Update navigation state
            self.state_container().navigation_mut().selected_column = new_col;

            // Update scroll offset if viewport changed
            if let Some(viewport) = result.new_viewport {
                let pinned_count = self
                    .buffer()
                    .get_dataview()
                    .as_ref()
                    .map(|dv| dv.get_pinned_columns().len())
                    .unwrap_or(0);
                self.state_container().navigation_mut().scroll_offset.1 =
                    viewport.start.saturating_sub(pinned_count);
            }

            self.buffer_mut().set_current_column(new_col);
        }

        // Set status message
        self.buffer_mut().set_status_message(result.description);
    }

    // ========== Column Operation Methods ==========

    /// Hide the currently selected column
    fn hide_current_column(&mut self) {
        if !self.is_in_results_mode() {
            return;
        }

        let result = self
            .viewport_manager()
            .borrow_mut()
            .as_mut()
            .map(|vm| vm.hide_current_column_with_result())
            .unwrap_or_else(|| ColumnOperationResult::failure("No viewport manager"));

        self.apply_column_operation_result(result);
    }

    /// Unhide all columns
    fn unhide_all_columns(&mut self) {
        let result = self
            .viewport_manager()
            .borrow_mut()
            .as_mut()
            .map(|vm| vm.unhide_all_columns_with_result())
            .unwrap_or_else(|| ColumnOperationResult::failure("No viewport manager"));

        self.apply_column_operation_result(result);
    }

    /// Move the current column left in the view
    fn move_current_column_left(&mut self) {
        if !self.is_in_results_mode() {
            return;
        }

        let result = self
            .viewport_manager()
            .borrow_mut()
            .as_mut()
            .map(|vm| vm.reorder_column_left_with_result())
            .unwrap_or_else(|| ColumnOperationResult::failure("No viewport manager"));

        self.apply_column_operation_result(result);
    }

    /// Move the current column right in the view
    fn move_current_column_right(&mut self) {
        if !self.is_in_results_mode() {
            return;
        }

        let result = self
            .viewport_manager()
            .borrow_mut()
            .as_mut()
            .map(|vm| vm.reorder_column_right_with_result())
            .unwrap_or_else(|| ColumnOperationResult::failure("No viewport manager"));

        self.apply_column_operation_result(result);
    }

    /// Navigate to the column on the left
    fn move_column_left(&mut self) {
        // Get navigation result from ViewportManager
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            let vm = viewport_borrow
                .as_mut()
                .expect("ViewportManager must exist for navigation");
            let current_visual = vm.get_crosshair_col();
            vm.navigate_column_left(current_visual)
        };

        self.apply_column_navigation_result(nav_result, "left");
    }

    /// Navigate to the column on the right
    fn move_column_right(&mut self) {
        // Get navigation result from ViewportManager
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            let vm = viewport_borrow
                .as_mut()
                .expect("ViewportManager must exist for navigation");
            let current_visual = vm.get_crosshair_col();
            vm.navigate_column_right(current_visual)
        };

        self.apply_column_navigation_result(nav_result, "right");
    }

    /// Navigate to the first column
    fn goto_first_column(&mut self) {
        // Get navigation result from ViewportManager
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow
                .as_mut()
                .expect("ViewportManager must exist for navigation")
                .navigate_to_first_column()
        };

        // Note: goto_first/last_column don't need cursor_manager updates
        self.apply_column_navigation_result(nav_result, "first");
    }

    /// Navigate to the last column
    fn goto_last_column(&mut self) {
        // Get navigation result from ViewportManager
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow
                .as_mut()
                .expect("ViewportManager must exist for navigation")
                .navigate_to_last_column()
        };

        // Note: goto_first/last_column don't need cursor_manager updates
        self.apply_column_navigation_result(nav_result, "last");
    }
}
