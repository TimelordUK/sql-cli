use crate::debug::debug_trace::{DebugSection, DebugSectionBuilder, DebugTrace, Priority};
use crate::ui::viewport_manager::ViewportManager;
use std::sync::Arc;

/// Debug trace implementation for ViewportManager
pub struct ViewportDebugProvider {
    viewport_manager: Arc<ViewportManager>,
}

impl ViewportDebugProvider {
    pub fn new(viewport_manager: Arc<ViewportManager>) -> Self {
        Self { viewport_manager }
    }
}

impl DebugTrace for ViewportDebugProvider {
    fn name(&self) -> &str {
        "ViewportManager"
    }

    fn debug_sections(&self) -> Vec<DebugSection> {
        let mut builder = DebugSectionBuilder::new();

        // Main viewport state section
        builder.add_section("VIEWPORT STATE", "", Priority::VIEWPORT);

        // Get crosshair position
        let crosshair_row = self.viewport_manager.get_crosshair_row();
        let crosshair_col = self.viewport_manager.get_crosshair_col();
        builder.add_field(
            "Crosshair Position",
            format!("row={}, col={}", crosshair_row, crosshair_col),
        );

        // Get viewport range
        let viewport_range = self.viewport_manager.get_viewport_range();
        builder.add_field(
            "Viewport Range",
            format!("{}-{}", viewport_range.start, viewport_range.end),
        );

        // Get viewport rows
        let viewport_rows = self.viewport_manager.get_viewport_rows();
        builder.add_field(
            "Viewport Rows",
            format!("{}-{}", viewport_rows.start, viewport_rows.end),
        );

        // Packing mode
        builder.add_field(
            "Packing Mode",
            format!("{:?}", self.viewport_manager.get_packing_mode()),
        );

        // Lock states
        builder.add_field("Cursor Locked", self.viewport_manager.is_cursor_locked());
        builder.add_field(
            "Viewport Locked",
            self.viewport_manager.is_viewport_locked(),
        );

        // Note: Pinned columns info would go here if ViewportManager supported it
        // Currently ViewportManager doesn't expose pinned column information

        // Note: Column width information requires mutable access
        // which we can't get through the Arc<ViewportManager>
        // This information could be added if ViewportManager provided
        // a non-mutable accessor for cached column widths

        builder.build()
    }

    fn debug_summary(&self) -> Option<String> {
        let viewport_rows = self.viewport_manager.get_viewport_rows();
        let viewport_cols = self.viewport_manager.get_viewport_range();
        let crosshair_row = self.viewport_manager.get_crosshair_row();
        let crosshair_col = self.viewport_manager.get_crosshair_col();
        Some(format!(
            "Viewport: {} rows x {} cols, Crosshair: ({}, {})",
            viewport_rows.end - viewport_rows.start,
            viewport_cols.end - viewport_cols.start,
            crosshair_row,
            crosshair_col
        ))
    }

    fn is_active(&self) -> bool {
        true // ViewportManager is always active when present
    }
}
