use crate::buffer::AppMode;
use crate::debouncer::Debouncer;
use crate::widget_traits::DebugInfoProvider;
use crossterm::event::{Event, KeyCode, KeyEvent};
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use regex::Regex;
use tui_input::{backend::crossterm::EventHandler, Input};

/// Represents the different search/filter modes
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Search,
    Filter,
    FuzzyFilter,
    ColumnSearch,
}

impl SearchMode {
    pub fn to_app_mode(&self) -> AppMode {
        match self {
            SearchMode::Search => AppMode::Search,
            SearchMode::Filter => AppMode::Filter,
            SearchMode::FuzzyFilter => AppMode::FuzzyFilter,
            SearchMode::ColumnSearch => AppMode::ColumnSearch,
        }
    }

    pub fn from_app_mode(mode: &AppMode) -> Option<Self> {
        match mode {
            AppMode::Search => Some(SearchMode::Search),
            AppMode::Filter => Some(SearchMode::Filter),
            AppMode::FuzzyFilter => Some(SearchMode::FuzzyFilter),
            AppMode::ColumnSearch => Some(SearchMode::ColumnSearch),
            _ => None,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            SearchMode::Search => "Search Pattern",
            SearchMode::Filter => "Filter Pattern",
            SearchMode::FuzzyFilter => "Fuzzy Filter",
            SearchMode::ColumnSearch => "Column Search",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            SearchMode::Search => Style::default().fg(Color::Yellow),
            SearchMode::Filter => Style::default().fg(Color::Cyan),
            SearchMode::FuzzyFilter => Style::default().fg(Color::Magenta),
            SearchMode::ColumnSearch => Style::default().fg(Color::Green),
        }
    }
}

/// State for search/filter operations
pub struct SearchModesState {
    pub mode: SearchMode,
    pub input: Input,
    pub fuzzy_matcher: SkimMatcherV2,
    pub regex: Option<Regex>,
    pub matching_columns: Vec<(usize, String)>,
    pub current_match_index: usize,
    pub saved_sql_text: String,
    pub saved_cursor_position: usize,
}

impl Clone for SearchModesState {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode.clone(),
            input: self.input.clone(),
            fuzzy_matcher: SkimMatcherV2::default(), // Create new matcher
            regex: self.regex.clone(),
            matching_columns: self.matching_columns.clone(),
            current_match_index: self.current_match_index,
            saved_sql_text: self.saved_sql_text.clone(),
            saved_cursor_position: self.saved_cursor_position,
        }
    }
}

impl SearchModesState {
    pub fn new(mode: SearchMode) -> Self {
        Self {
            mode,
            input: Input::default(),
            fuzzy_matcher: SkimMatcherV2::default(),
            regex: None,
            matching_columns: Vec::new(),
            current_match_index: 0,
            saved_sql_text: String::new(),
            saved_cursor_position: 0,
        }
    }

    pub fn reset(&mut self) {
        self.input.reset();
        self.regex = None;
        self.matching_columns.clear();
        self.current_match_index = 0;
    }

    pub fn get_pattern(&self) -> String {
        self.input.value().to_string()
    }
}

/// Actions that can be returned from the search modes widget
#[derive(Debug, Clone)]
pub enum SearchModesAction {
    Continue,
    Apply(SearchMode, String),
    Cancel,
    NextMatch,
    PreviousMatch,
    PassThrough,
    InputChanged(SearchMode, String), // Signal that input changed (for debouncing)
    ExecuteDebounced(SearchMode, String), // Execute the debounced action
}

/// A widget for handling all search/filter modes
pub struct SearchModesWidget {
    state: Option<SearchModesState>,
    debouncer: Debouncer,
    last_applied_pattern: Option<String>,
}

impl SearchModesWidget {
    pub fn new() -> Self {
        Self {
            state: None,
            debouncer: Debouncer::new(500), // 500ms debounce delay
            last_applied_pattern: None,
        }
    }

    /// Initialize the widget for a specific search mode
    pub fn enter_mode(&mut self, mode: SearchMode, current_sql: String, cursor_pos: usize) {
        let mut state = SearchModesState::new(mode);
        state.saved_sql_text = current_sql;
        state.saved_cursor_position = cursor_pos;
        self.state = Some(state);
        self.last_applied_pattern = None; // Reset when entering a new mode
    }

    /// Exit the current mode and return saved state
    pub fn exit_mode(&mut self) -> Option<(String, usize)> {
        self.debouncer.reset();
        self.last_applied_pattern = None; // Reset when exiting
        self.state
            .take()
            .map(|s| (s.saved_sql_text, s.saved_cursor_position))
    }

    /// Check if widget is active
    pub fn is_active(&self) -> bool {
        self.state.is_some()
    }

    /// Get the current mode if active
    pub fn current_mode(&self) -> Option<SearchMode> {
        self.state.as_ref().map(|s| s.mode.clone())
    }

    /// Get the current pattern
    pub fn get_pattern(&self) -> String {
        self.state
            .as_ref()
            .map(|s| s.get_pattern())
            .unwrap_or_default()
    }

    /// Get cursor position for rendering
    pub fn get_cursor_position(&self) -> usize {
        self.state.as_ref().map(|s| s.input.cursor()).unwrap_or(0)
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> SearchModesAction {
        let Some(state) = &mut self.state else {
            return SearchModesAction::PassThrough;
        };

        match key.code {
            KeyCode::Esc => SearchModesAction::Cancel,
            KeyCode::Enter => {
                let pattern = state.get_pattern();
                if !pattern.is_empty() {
                    SearchModesAction::Apply(state.mode.clone(), pattern)
                } else {
                    SearchModesAction::Cancel
                }
            }
            KeyCode::Tab => {
                if state.mode == SearchMode::ColumnSearch {
                    SearchModesAction::NextMatch
                } else {
                    state.input.handle_event(&Event::Key(key));
                    SearchModesAction::Continue
                }
            }
            KeyCode::BackTab => {
                if state.mode == SearchMode::ColumnSearch {
                    SearchModesAction::PreviousMatch
                } else {
                    SearchModesAction::Continue
                }
            }
            _ => {
                let old_pattern = state.get_pattern();
                state.input.handle_event(&Event::Key(key));
                let new_pattern = state.get_pattern();

                // If the pattern changed, trigger debouncing
                if old_pattern != new_pattern {
                    self.debouncer.trigger();
                    SearchModesAction::InputChanged(state.mode.clone(), new_pattern)
                } else {
                    SearchModesAction::Continue
                }
            }
        }
    }

    /// Check if debounced action should execute
    pub fn check_debounce(&mut self) -> Option<SearchModesAction> {
        if self.debouncer.should_execute() {
            if let Some(state) = &self.state {
                let pattern = state.get_pattern();

                // Check if pattern is different from last applied
                let should_apply = match &self.last_applied_pattern {
                    Some(last) => last != &pattern,
                    None => true, // First pattern always applies
                };

                if should_apply {
                    // For fuzzy filter, we need to apply even empty patterns to clear
                    if !pattern.is_empty() || state.mode == SearchMode::FuzzyFilter {
                        self.last_applied_pattern = Some(pattern.clone());
                        return Some(SearchModesAction::ExecuteDebounced(
                            state.mode.clone(),
                            pattern,
                        ));
                    }
                }
            }
        }
        None
    }

    /// Render the search input field
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let Some(state) = &self.state else {
            return;
        };

        let input_text = state.get_pattern();
        let mut title = state.mode.title().to_string();

        // Add debounce indicator to title with color coding
        if self.debouncer.is_pending() {
            if let Some(remaining) = self.debouncer.time_remaining() {
                let ms = remaining.as_millis();
                if ms > 0 {
                    // Add visual indicator with countdown
                    if ms > 300 {
                        title.push_str(&format!(" [⏱ {}ms]", ms));
                    } else if ms > 100 {
                        title.push_str(&format!(" [⚡ {}ms]", ms));
                    } else {
                        title.push_str(&format!(" [🔥 {}ms]", ms));
                    }
                } else {
                    title.push_str(" [⏳ applying...]");
                }
            }
        }

        let style = state.mode.style();

        let input_widget = Paragraph::new(input_text.as_str()).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        );

        f.render_widget(input_widget, area);

        // Set cursor position
        f.set_cursor_position((area.x + state.input.cursor() as u16 + 1, area.y + 1));
    }

    /// Render inline hint in status bar
    pub fn render_hint(&self) -> Line<'static> {
        if self.state.is_some() {
            Line::from(vec![
                Span::raw("Enter"),
                Span::styled(":Apply", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("Esc"),
                Span::styled(":Cancel", Style::default().fg(Color::Red)),
            ])
        } else {
            Line::from("")
        }
    }
}

impl DebugInfoProvider for SearchModesWidget {
    fn debug_info(&self) -> String {
        let mut info = String::from("=== SEARCH MODES WIDGET ===\n");

        // Add debouncer state
        info.push_str(&format!("Debouncer: "));
        if self.debouncer.is_pending() {
            if let Some(remaining) = self.debouncer.time_remaining() {
                info.push_str(&format!(
                    "PENDING ({}ms remaining)\n",
                    remaining.as_millis()
                ));
            } else {
                info.push_str("PENDING\n");
            }
        } else {
            info.push_str("IDLE\n");
        }
        info.push_str("\n");

        if let Some(state) = &self.state {
            info.push_str(&format!("State: ACTIVE\n"));
            info.push_str(&format!("Mode: {:?}\n", state.mode));
            info.push_str(&format!("Current Pattern: '{}'\n", state.get_pattern()));
            info.push_str(&format!("Pattern Length: {}\n", state.input.value().len()));
            info.push_str(&format!("Cursor Position: {}\n", state.input.cursor()));
            info.push_str("\n");

            info.push_str("Saved SQL State:\n");
            info.push_str(&format!(
                "  Text: '{}'\n",
                if state.saved_sql_text.len() > 50 {
                    format!(
                        "{}... ({} chars)",
                        &state.saved_sql_text[..50],
                        state.saved_sql_text.len()
                    )
                } else {
                    state.saved_sql_text.clone()
                }
            ));
            info.push_str(&format!("  Cursor: {}\n", state.saved_cursor_position));
            info.push_str(&format!("  SQL Length: {}\n", state.saved_sql_text.len()));

            if state.mode == SearchMode::ColumnSearch {
                info.push_str("\nColumn Search State:\n");
                info.push_str(&format!(
                    "  Matching Columns: {} found\n",
                    state.matching_columns.len()
                ));
                if !state.matching_columns.is_empty() {
                    info.push_str(&format!(
                        "  Current Match Index: {}\n",
                        state.current_match_index
                    ));
                    for (i, (idx, name)) in state.matching_columns.iter().take(5).enumerate() {
                        info.push_str(&format!(
                            "    [{}] Column {}: '{}'\n",
                            if i == state.current_match_index {
                                "*"
                            } else {
                                " "
                            },
                            idx,
                            name
                        ));
                    }
                    if state.matching_columns.len() > 5 {
                        info.push_str(&format!(
                            "    ... and {} more\n",
                            state.matching_columns.len() - 5
                        ));
                    }
                }
            }

            if state.mode == SearchMode::FuzzyFilter {
                info.push_str("\nFuzzy Filter State:\n");
                info.push_str(&format!("  Matcher: SkimMatcherV2 (ready)\n"));
            }

            if state.regex.is_some() {
                info.push_str("\nRegex State:\n");
                info.push_str(&format!("  Compiled: Yes\n"));
            }
        } else {
            info.push_str("State: INACTIVE\n");
            info.push_str("No active search mode\n");
        }

        info
    }
}
