//! Unified debounced input widget for all search/filter modes
//!
//! This widget handles text input with automatic debouncing to prevent
//! expensive operations (like searching through 20k rows) on every keystroke.

use crate::utils::debouncer::Debouncer;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Result of handling a key in the debounced input
#[derive(Debug, Clone)]
pub enum DebouncedInputAction {
    /// Continue typing, no action needed yet
    Continue,
    /// Input changed, may trigger debounced action
    InputChanged(String),
    /// Debounced period elapsed, execute the action now
    ExecuteDebounced(String),
    /// User pressed Enter to confirm
    Confirm(String),
    /// User pressed Esc to cancel
    Cancel,
    /// Pass the key through to parent handler
    PassThrough,
}

/// Configuration for the debounced input
#[derive(Debug, Clone)]
pub struct DebouncedInputConfig {
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Title for the input box
    pub title: String,
    /// Color style for the input
    pub style: Style,
    /// Whether to show debounce indicator
    pub show_debounce_indicator: bool,
}

impl Default for DebouncedInputConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 300,
            title: "Search".to_string(),
            style: Style::default().fg(Color::Yellow),
            show_debounce_indicator: true,
        }
    }
}

/// A reusable debounced input widget
pub struct DebouncedInput {
    /// The underlying tui_input
    input: Input,
    /// Debouncer for the input
    debouncer: Debouncer,
    /// Last pattern that was executed
    last_executed_pattern: Option<String>,
    /// Configuration
    config: DebouncedInputConfig,
    /// Whether the widget is active
    active: bool,
}

impl DebouncedInput {
    /// Create a new debounced input with default config
    pub fn new() -> Self {
        Self::with_config(DebouncedInputConfig::default())
    }

    /// Create a new debounced input with custom config
    pub fn with_config(config: DebouncedInputConfig) -> Self {
        Self {
            input: Input::default(),
            debouncer: Debouncer::new(config.debounce_ms),
            last_executed_pattern: None,
            config,
            active: false,
        }
    }

    /// Activate the input widget
    pub fn activate(&mut self) {
        self.active = true;
        self.input.reset();
        self.debouncer.reset();
        self.last_executed_pattern = None;
    }

    /// Deactivate the input widget
    pub fn deactivate(&mut self) {
        self.active = false;
        self.debouncer.reset();
    }

    /// Check if the widget is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current input value
    pub fn value(&self) -> &str {
        self.input.value()
    }

    /// Set the input value (useful for restoring state)
    pub fn set_value(&mut self, value: String) {
        self.input = Input::default().with_value(value);
    }

    /// Get the cursor position
    pub fn cursor(&self) -> usize {
        self.input.cursor()
    }

    /// Update configuration
    pub fn set_config(&mut self, config: DebouncedInputConfig) {
        self.debouncer = Debouncer::new(config.debounce_ms);
        self.config = config;
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) -> DebouncedInputAction {
        if !self.active {
            return DebouncedInputAction::PassThrough;
        }

        match key.code {
            KeyCode::Esc => {
                self.deactivate();
                DebouncedInputAction::Cancel
            }
            KeyCode::Enter => {
                let pattern = self.input.value().to_string();
                self.deactivate();
                DebouncedInputAction::Confirm(pattern)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Allow Ctrl-C to exit
                DebouncedInputAction::PassThrough
            }
            _ => {
                // Let tui_input handle the key (char input, backspace, arrows, etc.)
                self.input.handle_event(&crossterm::event::Event::Key(key));
                let current_pattern = self.input.value().to_string();

                // Check if pattern actually changed
                if self.last_executed_pattern.as_ref() != Some(&current_pattern) {
                    self.debouncer.trigger();
                    DebouncedInputAction::InputChanged(current_pattern)
                } else {
                    DebouncedInputAction::Continue
                }
            }
        }
    }

    /// Check if the debounced action should execute
    /// This should be called periodically (e.g., in the main event loop)
    pub fn check_debounce(&mut self) -> Option<String> {
        if self.debouncer.should_execute() {
            let pattern = self.input.value().to_string();
            // Only execute if pattern changed since last execution
            if self.last_executed_pattern.as_ref() != Some(&pattern) {
                self.last_executed_pattern = Some(pattern.clone());
                Some(pattern)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Render the input widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let title = if self.config.show_debounce_indicator && self.debouncer.is_pending() {
            format!("{} (typing...)", self.config.title)
        } else {
            self.config.title.clone()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(self.config.style);

        let input_widget = Paragraph::new(self.input.value())
            .block(block)
            .style(self.config.style);

        f.render_widget(input_widget, area);

        // Set cursor position if active
        if self.active {
            f.set_cursor_position((area.x + self.input.cursor() as u16 + 1, area.y + 1));
        }
    }

    /// Create a custom title with mode indicator
    pub fn set_title(&mut self, title: String) {
        self.config.title = title;
    }

    /// Update the style
    pub fn set_style(&mut self, style: Style) {
        self.config.style = style;
    }
}

/// Builder pattern for DebouncedInput configuration
pub struct DebouncedInputBuilder {
    config: DebouncedInputConfig,
}

impl DebouncedInputBuilder {
    pub fn new() -> Self {
        Self {
            config: DebouncedInputConfig::default(),
        }
    }

    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.config.style = style;
        self
    }

    pub fn show_indicator(mut self, show: bool) -> Self {
        self.config.show_debounce_indicator = show;
        self
    }

    pub fn build(self) -> DebouncedInput {
        DebouncedInput::with_config(self.config)
    }
}
