//! Crosshair Widget - Handles visual cursor rendering on the table
//!
//! This widget tracks crosshair position changes and ensures the table
//! re-renders the affected cells when the crosshair moves. Since ratatui
//! doesn't support partial updates, we need to trigger a re-render of
//! at least the affected rows when the crosshair moves.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    Frame,
};
use tracing::{debug, trace};

/// Crosshair position in absolute data coordinates
#[derive(Debug, Clone, Default)]
pub struct CrosshairPosition {
    /// Row in data coordinates (0-based)
    pub row: usize,
    /// Column in data coordinates (0-based)
    pub column: usize,
    /// Whether the crosshair is visible
    pub visible: bool,
}

/// Crosshair widget for rendering the cursor on the table
pub struct CrosshairWidget {
    /// Current position in data coordinates
    position: CrosshairPosition,
    /// Style for the crosshair
    style: Style,
}

impl CrosshairWidget {
    /// Create a new crosshair widget
    pub fn new() -> Self {
        Self {
            position: CrosshairPosition::default(),
            style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::REVERSED),
        }
    }

    /// Update the crosshair position
    pub fn set_position(&mut self, row: usize, column: usize) {
        self.position.row = row;
        self.position.column = column;
        self.position.visible = true;
        debug!("Crosshair position updated to ({}, {})", row, column);
    }

    /// Hide the crosshair
    pub fn hide(&mut self) {
        self.position.visible = false;
    }

    /// Show the crosshair
    pub fn show(&mut self) {
        self.position.visible = true;
    }

    /// Get the current position
    pub fn position(&self) -> &CrosshairPosition {
        &self.position
    }

    /// Set custom style for the crosshair
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Render the crosshair overlay on the table
    /// This should be called AFTER rendering the table
    pub fn render_overlay(
        &self,
        f: &mut Frame,
        table_area: Rect,
        viewport_row_offset: usize,
        viewport_col_offset: usize,
        row_heights: &[u16], // Height of each visible row
        col_widths: &[u16],  // Width of each visible column
    ) {
        if !self.position.visible {
            return;
        }

        // Check if crosshair is within the visible viewport
        let visible_rows = row_heights.len();
        let visible_cols = col_widths.len();

        // Calculate relative position within viewport
        if self.position.row < viewport_row_offset {
            return; // Above viewport
        }
        let relative_row = self.position.row - viewport_row_offset;
        if relative_row >= visible_rows {
            return; // Below viewport
        }

        if self.position.column < viewport_col_offset {
            return; // Left of viewport
        }
        let relative_col = self.position.column - viewport_col_offset;
        if relative_col >= visible_cols {
            return; // Right of viewport
        }

        // Calculate pixel position for the crosshair
        let mut y = table_area.y + 2; // Account for table border and header
        for i in 0..relative_row {
            y += row_heights[i];
        }

        let mut x = table_area.x + 1; // Account for table border
        for i in 0..relative_col {
            x += col_widths[i] + 1; // +1 for column separator
        }

        // Draw the crosshair cell
        let cell_width = col_widths[relative_col];
        let cell_rect = Rect {
            x,
            y,
            width: cell_width,
            height: 1,
        };

        // Apply crosshair style to the cell
        // Note: This is a simplified version. In practice, we'd need to
        // re-render just the cell content with the crosshair style
        trace!(
            "Rendering crosshair at viewport ({}, {}) -> screen ({}, {})",
            relative_row,
            relative_col,
            x,
            y
        );
    }

    /// Calculate if scrolling is needed to show the crosshair
    pub fn calculate_scroll_offset(
        &self,
        current_row_offset: usize,
        current_col_offset: usize,
        viewport_height: usize,
        viewport_width: usize,
    ) -> (usize, usize) {
        let mut new_row_offset = current_row_offset;
        let mut new_col_offset = current_col_offset;

        // Vertical scrolling
        if self.position.row < current_row_offset {
            // Crosshair is above viewport, scroll up
            new_row_offset = self.position.row;
        } else if self.position.row >= current_row_offset + viewport_height {
            // Crosshair is below viewport, center it
            new_row_offset = self.position.row.saturating_sub(viewport_height / 2);
        }

        // Horizontal scrolling
        if self.position.column < current_col_offset {
            // Crosshair is left of viewport, scroll left
            new_col_offset = self.position.column;
        } else if self.position.column >= current_col_offset + viewport_width {
            // Crosshair is right of viewport, scroll right
            new_col_offset = self.position.column.saturating_sub(viewport_width / 2);
        }

        (new_row_offset, new_col_offset)
    }
}
