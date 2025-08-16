use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::VecDeque;
use std::time::Instant;

/// A visual indicator that shows recent key presses with fade effect
pub struct KeyPressIndicator {
    /// Recent key presses with timestamps
    key_history: VecDeque<(String, Instant)>,
    /// Maximum number of keys to show
    max_keys: usize,
    /// How long before a key starts fading (milliseconds)
    fade_start_ms: u64,
    /// How long the fade takes (milliseconds)
    fade_duration_ms: u64,
    /// Whether the indicator is enabled
    pub enabled: bool,
}

impl KeyPressIndicator {
    pub fn new() -> Self {
        Self {
            key_history: VecDeque::with_capacity(10),
            max_keys: 10, // Allow up to 10 keys but fade will naturally limit display
            fade_start_ms: 500,
            fade_duration_ms: 1500,
            enabled: true, // Enable by default for better debugging
        }
    }

    /// Enable or disable the indicator
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.key_history.clear();
        }
    }

    /// Record a key press
    pub fn record_key(&mut self, key: String) {
        if !self.enabled {
            return;
        }

        // Add new key
        self.key_history.push_back((key, Instant::now()));

        // Remove old keys if we exceed capacity
        while self.key_history.len() > self.max_keys {
            self.key_history.pop_front();
        }

        // Remove keys that have fully faded (after fade_start + fade_duration)
        let fade_complete = self.fade_start_ms + self.fade_duration_ms;
        self.key_history
            .retain(|(_, time)| time.elapsed().as_millis() < fade_complete as u128);
    }

    /// Render the indicator
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.enabled || self.key_history.is_empty() {
            return;
        }

        let mut spans = Vec::new();

        for (i, (key, time)) in self.key_history.iter().enumerate() {
            let elapsed_ms = time.elapsed().as_millis() as u64;

            // Calculate opacity (0.0 to 1.0)
            let opacity = if elapsed_ms < self.fade_start_ms {
                1.0
            } else if elapsed_ms < self.fade_start_ms + self.fade_duration_ms {
                let fade_progress =
                    (elapsed_ms - self.fade_start_ms) as f32 / self.fade_duration_ms as f32;
                1.0 - fade_progress
            } else {
                0.0
            };

            if opacity > 0.0 {
                // Convert opacity to color intensity
                let color = self.opacity_to_color(opacity);

                // Add separator if not first
                if i > 0 {
                    spans.push(Span::styled(" → ", Style::default().fg(Color::DarkGray)));
                }

                // Add the key with fading color
                spans.push(Span::styled(
                    key.clone(),
                    Style::default().fg(color).add_modifier(Modifier::ITALIC),
                ));
            }
        }

        if !spans.is_empty() {
            let paragraph = Paragraph::new(Line::from(spans)).block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default()),
            );
            frame.render_widget(paragraph, area);
        }
    }

    /// Convert opacity (0.0 to 1.0) to a color
    fn opacity_to_color(&self, opacity: f32) -> Color {
        // Fade from bright cyan to dark gray
        if opacity > 0.7 {
            Color::Cyan
        } else if opacity > 0.4 {
            Color::Gray
        } else {
            Color::DarkGray
        }
    }

    /// Create a formatted string representation for debugging
    pub fn to_string(&self) -> String {
        if !self.enabled || self.key_history.is_empty() {
            return String::new();
        }

        self.key_history
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

/// Format a key event for display
pub fn format_key_for_display(key: &crossterm::event::KeyEvent) -> String {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mut parts = Vec::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    // Add the key itself
    let key_str = match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                c.to_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "⌫".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => "?".to_string(),
    };

    if !parts.is_empty() {
        format!("{}-{}", parts.join("+"), key_str)
    } else {
        key_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_indicator() {
        let mut indicator = KeyPressIndicator::new();
        indicator.set_enabled(true);

        indicator.record_key("j".to_string());
        indicator.record_key("k".to_string());
        indicator.record_key("Enter".to_string());

        let display = indicator.to_string();
        assert!(display.contains("j"));
        assert!(display.contains("k"));
        assert!(display.contains("Enter"));
    }

    #[test]
    fn test_key_formatting() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(format_key_for_display(&key), "Ctrl-C");

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(format_key_for_display(&key), "↑");
    }
}
