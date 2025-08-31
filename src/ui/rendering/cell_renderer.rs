use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};

use crate::config::config::CellSelectionStyle;

/// Different visual styles for rendering selected cells
#[derive(Debug, Clone)]
pub enum CellRenderMode {
    /// Traditional underline style
    Underline,
    /// Full block/inverse video style
    Block,
    /// Border around the cell
    Border,
    /// Just corners of the cell
    Corners,
    /// Subtle highlight (just color change)
    Subtle,
}

impl From<&str> for CellRenderMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "block" => CellRenderMode::Block,
            "border" => CellRenderMode::Border,
            "corners" => CellRenderMode::Corners,
            "subtle" => CellRenderMode::Subtle,
            _ => CellRenderMode::Underline,
        }
    }
}

/// Renders a cell with the configured selection style
pub struct CellRenderer {
    style_config: CellSelectionStyle,
}

impl CellRenderer {
    pub fn new(style_config: CellSelectionStyle) -> Self {
        Self { style_config }
    }

    /// Create a style for a selected cell based on configuration
    pub fn get_selected_style(&self) -> Style {
        let mut style = Style::default();

        // Parse foreground color
        style = style.fg(self.parse_color(&self.style_config.foreground));

        // Apply background if configured
        if self.style_config.use_background {
            style = style.bg(self.parse_color(&self.style_config.background));
        }

        // Apply modifiers based on mode
        let mode = CellRenderMode::from(self.style_config.mode.as_str());
        match mode {
            CellRenderMode::Underline => {
                if self.style_config.underline {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
            }
            CellRenderMode::Block => {
                // Inverse video effect - swap foreground and background
                style = style.add_modifier(Modifier::REVERSED);
            }
            CellRenderMode::Border | CellRenderMode::Corners => {
                // Use dark gray background (like column highlight) with bright foreground
                // This is easier on the eyes than cyan background
                style = style
                    .bg(Color::DarkGray) // Same as column mode background
                    .fg(self.parse_color(&self.style_config.foreground)) // Use configured color
                    .add_modifier(Modifier::BOLD);
            }
            CellRenderMode::Subtle => {
                // Just use the color, no additional modifiers
            }
        }

        // Apply bold if configured
        if self.style_config.bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        style
    }

    /// Render a cell value with optional border/corner decorations
    pub fn render_cell_value(&self, value: &str, is_selected: bool, width: usize) -> String {
        if !is_selected {
            return value.to_string();
        }

        let mode = CellRenderMode::from(self.style_config.mode.as_str());

        match mode {
            CellRenderMode::Border => self.render_with_border(value, width),
            CellRenderMode::Corners => self.render_with_corners(value, width),
            _ => value.to_string(),
        }
    }

    /// Render value with full border
    fn render_with_border(&self, value: &str, width: usize) -> String {
        let chars = match self.style_config.border_style.as_str() {
            "double" => ('═', '║', '╔', '╗', '╚', '╝'),
            "rounded" => ('─', '│', '╭', '╮', '╰', '╯'),
            "thick" => ('━', '┃', '┏', '┓', '┗', '┛'),
            _ => ('─', '│', '┌', '┐', '└', '┘'), // single
        };

        // For inline cell rendering, we can't really do multi-line borders
        // So we'll just add subtle markers
        format!("{}{}{}", chars.2, value, chars.3)
    }

    /// Render value with just corner markers
    fn render_with_corners(&self, value: &str, width: usize) -> String {
        let corners: Vec<char> = self.style_config.corner_chars.chars().collect();
        if corners.len() >= 4 {
            // Add subtle corner markers inline
            format!("{}{}{}", corners[0], value, corners[1])
        } else {
            value.to_string()
        }
    }

    /// Parse color string to Ratatui Color
    fn parse_color(&self, color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "dark_gray" | "dark_grey" => Color::DarkGray,
            "light_red" | "bright_red" => Color::LightRed,
            "light_green" | "bright_green" => Color::LightGreen,
            "light_yellow" | "bright_yellow" => Color::LightYellow,
            "light_blue" | "bright_blue" => Color::LightBlue,
            "light_magenta" | "bright_magenta" => Color::LightMagenta,
            "light_cyan" | "bright_cyan" => Color::LightCyan,
            "white" => Color::White,
            "orange" => Color::Rgb(255, 165, 0),
            "purple" => Color::Rgb(128, 0, 128),
            "teal" => Color::Rgb(0, 128, 128),
            "pink" => Color::Rgb(255, 192, 203),
            _ => Color::Yellow, // Default
        }
    }

    /// Get a preview of all available styles for configuration UI
    pub fn get_style_previews() -> Vec<(&'static str, &'static str)> {
        vec![
            ("underline", "Classic underline style"),
            ("block", "Inverse/block selection"),
            ("border", "Full border around cell"),
            ("corners", "Corner markers only"),
            ("subtle", "Just color highlight"),
        ]
    }
}

/// Helper to create bordered cell for special rendering
pub fn create_bordered_cell<'a>(content: &str, border_style: &str) -> Block<'a> {
    let block = Block::default().borders(Borders::ALL);

    match border_style {
        "double" => block.border_type(ratatui::widgets::BorderType::Double),
        "rounded" => block.border_type(ratatui::widgets::BorderType::Rounded),
        "thick" => block.border_type(ratatui::widgets::BorderType::Thick),
        _ => block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_render_modes() {
        let config = CellSelectionStyle::default();
        let renderer = CellRenderer::new(config);

        let style = renderer.get_selected_style();
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_color_parsing() {
        let mut config = CellSelectionStyle::default();
        config.foreground = "orange".to_string();
        let renderer = CellRenderer::new(config);

        let style = renderer.get_selected_style();
        // Orange should be RGB(255, 165, 0)
        assert_eq!(style.fg, Some(Color::Rgb(255, 165, 0)));
    }
}
