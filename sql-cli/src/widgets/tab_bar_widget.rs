use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

/// Renders a tab bar showing all open buffers
pub struct TabBarWidget {
    /// Current buffer index
    current_index: usize,
    /// List of buffer names  
    buffer_names: Vec<String>,
    /// Whether to show Alt+N shortcuts
    show_shortcuts: bool,
}

impl TabBarWidget {
    pub fn new(current_index: usize, buffer_names: Vec<String>) -> Self {
        Self {
            current_index,
            buffer_names,
            show_shortcuts: true,
        }
    }

    /// Set whether to show Alt+N shortcuts
    pub fn with_shortcuts(mut self, show: bool) -> Self {
        self.show_shortcuts = show;
        self
    }

    /// Render the tab bar
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Don't render if only one buffer
        if self.buffer_names.len() <= 1 {
            return;
        }

        // Create tab titles with optional shortcuts
        let titles: Vec<Line> = self
            .buffer_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let mut spans = vec![];

                // Add shortcut indicator if enabled and within Alt+1-9 range
                if self.show_shortcuts && i < 9 {
                    spans.push(Span::styled(
                        format!("{}:", i + 1),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ));
                }

                // Truncate long buffer names
                let display_name = if name.len() > 20 {
                    format!("{}...", &name[..17])
                } else {
                    name.clone()
                };

                spans.push(Span::raw(display_name));
                Line::from(spans)
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .select(self.current_index)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            )
            .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));

        f.render_widget(tabs, area);
    }

    /// Calculate the height needed for the tab bar
    pub fn height(&self) -> u16 {
        if self.buffer_names.len() <= 1 {
            0 // Don't take space if only one buffer
        } else {
            2 // Tab bar with border
        }
    }
}

/// Helper function to create and render a tab bar in one call
pub fn render_tab_bar(f: &mut Frame, area: Rect, current_index: usize, buffer_names: Vec<String>) {
    let widget = TabBarWidget::new(current_index, buffer_names);
    widget.render(f, area);
}
