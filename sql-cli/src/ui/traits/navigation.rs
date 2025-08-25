use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, BufferAPI};
use crate::ui::viewport_manager::{RowNavigationResult, ViewportManager};
use std::cell::RefCell;
// Arc import removed - no longer needed

/// Trait that provides navigation behavior for TUI components
/// This extracts navigation methods from EnhancedTui to reduce coupling
pub trait NavigationBehavior {
    // Required methods - these provide access to TUI internals
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>>;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn buffer(&self) -> &dyn BufferAPI;
    fn state_container(&self) -> &AppStateContainer;
    fn state_container_mut(&mut self) -> &mut AppStateContainer; // Added for mutable access
    fn get_row_count(&self) -> usize;

    // Helper method that stays in the trait
    fn apply_row_navigation_result(&mut self, result: RowNavigationResult) {
        // Use centralized sync method
        self.sync_row_state(result.row_position);

        // Update scroll offset if viewport changed
        if result.viewport_changed {
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = result.row_scroll_offset;
            self.buffer_mut().set_scroll_offset(offset);
            self.state_container().navigation_mut().scroll_offset.0 = result.row_scroll_offset;
        }
    }

    /// Centralized method to sync row state across all components
    /// This ensures Buffer, AppStateContainer, and any other row tracking stays in sync
    fn sync_row_state(&mut self, row: usize) {
        // 1. Update Buffer's selected row
        self.buffer_mut().set_selected_row(Some(row));

        // 2. Update AppStateContainer navigation state
        self.state_container().navigation_mut().selected_row = row;

        // 3. Also update via set_table_selected_row for consistency
        // This ensures any internal bookkeeping in AppStateContainer is maintained
        self.state_container().set_table_selected_row(Some(row));
    }

    // ========== Row Navigation Methods ==========

    fn next_row(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.navigate_row_down())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn previous_row(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.navigate_row_up())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn goto_first_row(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let nav_result = {
                let mut viewport_borrow = self.viewport_manager().borrow_mut();
                viewport_borrow
                    .as_mut()
                    .map(|vm| vm.navigate_to_first_row(total_rows))
            };

            if let Some(nav_result) = nav_result {
                self.apply_row_navigation_result(nav_result);
            }
        }
    }

    fn goto_last_row(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let nav_result = {
                let mut viewport_borrow = self.viewport_manager().borrow_mut();
                viewport_borrow
                    .as_mut()
                    .map(|vm| vm.navigate_to_last_row(total_rows))
            };

            if let Some(nav_result) = nav_result {
                self.apply_row_navigation_result(nav_result);
            }
        }
    }

    fn page_down(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.page_down())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn page_up(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.page_up())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn half_page_down(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.half_page_down())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn half_page_up(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow.as_mut().map(|vm| vm.half_page_up())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    fn goto_line(&mut self, line_number: usize) {
        let total_rows = self.get_row_count();
        if line_number > 0 && line_number <= total_rows {
            let target_row = line_number - 1; // Convert to 0-indexed
            let nav_result = {
                let mut viewport_borrow = self.viewport_manager().borrow_mut();
                viewport_borrow.as_mut().map(|vm| vm.goto_line(target_row))
            };

            if let Some(nav_result) = nav_result {
                self.apply_row_navigation_result(nav_result);
                self.state_container_mut()
                    .set_status_message(format!("Jumped to row {} (centered)", line_number));
            }
        } else {
            self.state_container_mut().set_status_message(format!(
                "Row {} out of range (max: {})",
                line_number, total_rows
            ));
        }
    }

    /// Navigate to the top of the current viewport
    fn goto_viewport_top(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow
                .as_mut()
                .map(|vm| vm.navigate_to_viewport_top())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    /// Navigate to the middle of the current viewport
    fn goto_viewport_middle(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow
                .as_mut()
                .map(|vm| vm.navigate_to_viewport_middle())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    /// Navigate to the bottom of the current viewport
    fn goto_viewport_bottom(&mut self) {
        let nav_result = {
            let mut viewport_borrow = self.viewport_manager().borrow_mut();
            viewport_borrow
                .as_mut()
                .map(|vm| vm.navigate_to_viewport_bottom())
        };

        if let Some(nav_result) = nav_result {
            self.apply_row_navigation_result(nav_result);
        }
    }

    /// Complete jump-to-row operation (called on Enter key)
    fn complete_jump_to_row(&mut self, input: &str) {
        if let Ok(row_num) = input.parse::<usize>() {
            self.goto_line(row_num);
        } else {
            self.state_container_mut()
                .set_status_message("Invalid row number".to_string());
        }

        self.state_container_mut().set_mode(AppMode::Results);

        // Clear jump-to-row state
        let jump_state = self.state_container_mut().jump_to_row_mut();
        jump_state.input.clear();
        jump_state.is_active = false;
    }
}
