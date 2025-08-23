use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, BufferAPI};
use crate::ui::viewport_manager::{ColumnOperationResult, ViewportManager};
use std::cell::RefCell;

/// Trait that provides column operation behavior for TUI components
/// This extracts column operation methods from EnhancedTui to reduce coupling
pub trait ColumnBehavior {
    // Required methods - these provide access to TUI internals
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>>;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn buffer(&self) -> &dyn BufferAPI;
    fn state_container(&self) -> &AppStateContainer;

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
        if self.buffer().get_mode() != AppMode::Results {
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
        if self.buffer().get_mode() != AppMode::Results {
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
        if self.buffer().get_mode() != AppMode::Results {
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
}
