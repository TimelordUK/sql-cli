use crate::buffer::{BufferAPI, ColumnStatistics, ColumnType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// A self-contained widget for displaying column statistics
pub struct StatsWidget {
    /// Whether the widget should handle its own key events
    handle_keys: bool,
}

impl StatsWidget {
    pub fn new() -> Self {
        Self { handle_keys: true }
    }

    /// Handle key input when the stats widget is active
    /// Returns true if the app should exit, false otherwise
    pub fn handle_key(&mut self, key: KeyEvent) -> StatsAction {
        if !self.handle_keys {
            return StatsAction::PassThrough;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                StatsAction::Quit
            }
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('S') => StatsAction::Close,
            _ => StatsAction::Continue,
        }
    }

    /// Render the statistics widget
    pub fn render(&self, f: &mut Frame, area: Rect, buffer: &dyn BufferAPI) {
        if let Some(stats) = buffer.get_column_stats() {
            let lines = self.build_stats_lines(stats);

            let stats_paragraph = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Column Statistics - {} (S to close)",
                    stats.column_name
                )))
                .wrap(Wrap { trim: false });

            f.render_widget(stats_paragraph, area);
        } else {
            let error = Paragraph::new("No statistics available")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Column Statistics"),
                )
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
        }
    }

    /// Build the lines of text for the statistics display
    fn build_stats_lines(&self, stats: &ColumnStatistics) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(format!("Column Statistics: {}", stats.column_name)).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from(format!("Type: {:?}", stats.column_type))
                .style(Style::default().fg(Color::Yellow)),
            Line::from(format!("Total Rows: {}", stats.total_count)),
            Line::from(format!("Unique Values: {}", stats.unique_count)),
            Line::from(format!("Null/Empty Count: {}", stats.null_count)),
            Line::from(""),
        ];

        // Add numeric statistics if available
        if matches!(stats.column_type, ColumnType::Numeric | ColumnType::Mixed) {
            lines.push(
                Line::from("Numeric Statistics:").style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            );

            if let Some(min) = stats.min {
                lines.push(Line::from(format!("  Min: {:.2}", min)));
            }
            if let Some(max) = stats.max {
                lines.push(Line::from(format!("  Max: {:.2}", max)));
            }
            if let Some(mean) = stats.mean {
                lines.push(Line::from(format!("  Mean: {:.2}", mean)));
            }
            if let Some(median) = stats.median {
                lines.push(Line::from(format!("  Median: {:.2}", median)));
            }
            if let Some(sum) = stats.sum {
                lines.push(Line::from(format!("  Sum: {:.2}", sum)));
            }
            lines.push(Line::from(""));
        }

        // Add frequency distribution if available
        if let Some(ref freq_map) = stats.frequency_map {
            lines.push(
                Line::from("Frequency Distribution:").style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            );

            // Sort by frequency (descending) and take top 20
            let mut freq_vec: Vec<(&String, &usize)> = freq_map.iter().collect();
            freq_vec.sort_by(|a, b| b.1.cmp(a.1));

            let max_count = freq_vec.first().map(|(_, c)| **c).unwrap_or(1);

            for (value, count) in freq_vec.iter().take(20) {
                let bar_width = ((**count as f64 / max_count as f64) * 30.0) as usize;
                let bar = "█".repeat(bar_width);
                let display_value = if value.len() > 30 {
                    format!("{}...", &value[..27])
                } else {
                    value.to_string()
                };
                lines.push(Line::from(format!(
                    "  {:30} {} ({})",
                    display_value, bar, count
                )));
            }

            if freq_vec.len() > 20 {
                lines.push(
                    Line::from(format!(
                        "  ... and {} more unique values",
                        freq_vec.len() - 20
                    ))
                    .style(Style::default().fg(Color::DarkGray)),
                );
            }
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("Press S or Esc to return to results")
                .style(Style::default().fg(Color::DarkGray)),
        );

        lines
    }
}

/// Actions that can be returned from handling keys
#[derive(Debug, Clone, PartialEq)]
pub enum StatsAction {
    /// Continue showing stats
    Continue,
    /// Close the stats view
    Close,
    /// Quit the application
    Quit,
    /// Pass the key through to the main handler
    PassThrough,
}

impl Default for StatsWidget {
    fn default() -> Self {
        Self::new()
    }
}
