use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::ui::viewport_manager::{RowNavigationResult, ViewportManager};
use std::cell::RefCell;
use tracing::debug;

/// Trait that provides navigation behavior for TUI components
/// This extracts navigation methods from EnhancedTui to reduce coupling
pub trait NavigationBehavior {
    // Required methods - these provide access to TUI internals
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>>;
    fn buffer_mut(&mut self) -> &mut dyn BufferAPI;
    fn buffer(&self) -> &dyn BufferAPI;
    fn state_container(&self) -> &AppStateContainer;
    fn get_row_count(&self) -> usize;

    // Helper method that stays in the trait
    fn apply_row_navigation_result(&mut self, result: RowNavigationResult) {
        // Update Buffer's selected row
        self.buffer_mut()
            .set_selected_row(Some(result.row_position));

        // Update scroll offset if viewport changed
        if result.viewport_changed {
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = result.row_scroll_offset;
            self.buffer_mut().set_scroll_offset(offset);
        }

        // Update AppStateContainer for consistency
        self.state_container().navigation_mut().selected_row = result.row_position;
        if result.viewport_changed {
            self.state_container().navigation_mut().scroll_offset.0 = result.row_scroll_offset;
        }
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
                self.buffer_mut()
                    .set_status_message(format!("Jumped to row {} (centered)", line_number));
            }
        } else {
            self.buffer_mut().set_status_message(format!(
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
}
