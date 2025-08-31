use crate::history::{CommandHistory, HistoryMatch};
use crate::widget_traits::DebugInfoProvider;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, Paragraph, Wrap},
    Frame,
};

/// Manages the state for history search and display
#[derive(Clone)]
pub struct HistoryState {
    pub search_query: String,
    pub matches: Vec<HistoryMatch>,
    pub selected_index: usize,
}

/// A self-contained widget for command history
pub struct HistoryWidget {
    command_history: CommandHistory,
    state: HistoryState,
    fuzzy_matcher: SkimMatcherV2,
}

impl HistoryWidget {
    pub fn new(command_history: CommandHistory) -> Self {
        Self {
            command_history,
            state: HistoryState {
                search_query: String::new(),
                matches: Vec::new(),
                selected_index: 0,
            },
            fuzzy_matcher: SkimMatcherV2::default(),
        }
    }

    /// Initialize history mode - load all history entries
    pub fn initialize(&mut self) {
        self.state.search_query.clear();
        self.state.matches = self
            .command_history
            .get_all()
            .iter()
            .cloned()
            .map(|entry| HistoryMatch {
                entry,
                indices: Vec::new(),
                score: 0,
            })
            .collect();
        self.state.selected_index = 0;
    }

    /// Update search query and filter matches
    pub fn update_search(&mut self, query: String) {
        self.state.search_query = query;

        if self.state.search_query.is_empty() {
            // Show all history when no search
            self.state.matches = self
                .command_history
                .get_all()
                .iter()
                .cloned()
                .map(|entry| HistoryMatch {
                    entry,
                    indices: Vec::new(),
                    score: 0,
                })
                .collect();
        } else {
            // Fuzzy search through history
            let mut matches: Vec<HistoryMatch> = self
                .command_history
                .get_all()
                .iter()
                .cloned()
                .filter_map(|entry| {
                    self.fuzzy_matcher
                        .fuzzy_indices(&entry.command, &self.state.search_query)
                        .map(|(score, indices)| HistoryMatch {
                            entry,
                            indices,
                            score,
                        })
                })
                .collect();

            // Sort by score (highest first)
            matches.sort_by(|a, b| b.score.cmp(&a.score));
            self.state.matches = matches;
        }

        self.state.selected_index = 0;
    }

    /// Handle key input for history mode
    pub fn handle_key(&mut self, key: KeyEvent) -> HistoryAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                HistoryAction::Quit
            }
            KeyCode::Esc => HistoryAction::Exit,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.selected_index > 0 {
                    self.state.selected_index -= 1;
                }
                HistoryAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.selected_index < self.state.matches.len().saturating_sub(1) {
                    self.state.selected_index += 1;
                }
                HistoryAction::None
            }
            KeyCode::PageUp => {
                self.state.selected_index = self.state.selected_index.saturating_sub(10);
                HistoryAction::None
            }
            KeyCode::PageDown => {
                let max_index = self.state.matches.len().saturating_sub(1);
                self.state.selected_index = (self.state.selected_index + 10).min(max_index);
                HistoryAction::None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.selected_index = 0;
                HistoryAction::None
            }
            KeyCode::End | KeyCode::Char('G') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.state.selected_index = self.state.matches.len().saturating_sub(1);
                HistoryAction::None
            }
            KeyCode::Enter => {
                if let Some(selected_match) = self.state.matches.get(self.state.selected_index) {
                    HistoryAction::ExecuteCommand(selected_match.entry.command.clone())
                } else {
                    HistoryAction::None
                }
            }
            KeyCode::Tab => {
                if let Some(selected_match) = self.state.matches.get(self.state.selected_index) {
                    HistoryAction::UseCommand(selected_match.entry.command.clone())
                } else {
                    HistoryAction::None
                }
            }
            // Delete functionality disabled - CommandHistory doesn't support deletion yet
            // KeyCode::Delete | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            //     HistoryAction::None
            // }
            KeyCode::Char('/') => HistoryAction::StartSearch,
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                self.update_search(self.state.search_query.clone());
                HistoryAction::None
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
                self.update_search(self.state.search_query.clone());
                HistoryAction::None
            }
            _ => HistoryAction::None,
        }
    }

    /// Render the history widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.state.matches.is_empty() {
            self.render_empty_state(f, area);
            return;
        }

        // Split the area to show selected command details
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // History list
                Constraint::Percentage(50), // Selected command preview
            ])
            .split(area);

        self.render_history_list(f, chunks[0]);
        self.render_selected_command_preview(f, chunks[1]);
    }

    fn render_empty_state(&self, f: &mut Frame, area: Rect) {
        let message = if self.state.search_query.is_empty() {
            "No command history found.\nExecute some queries to build history."
        } else {
            "No matches found for your search.\nTry a different search term."
        };

        let placeholder = Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command History"),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, area);
    }

    fn render_history_list(&self, f: &mut Frame, area: Rect) {
        let history_items: Vec<Line> = self
            .state
            .matches
            .iter()
            .enumerate()
            .map(|(i, history_match)| {
                let entry = &history_match.entry;
                let is_selected = i == self.state.selected_index;

                let success_indicator = if entry.success { "✓" } else { "✗" };
                let time_ago = self.format_time_ago(&entry.timestamp);

                // Use more space for the command, less for metadata
                let terminal_width = area.width as usize;
                let metadata_space = 15;
                let available_for_command = terminal_width.saturating_sub(metadata_space).max(50);

                let command_text = if entry.command.len() > available_for_command {
                    format!(
                        "{}…",
                        &entry.command[..available_for_command.saturating_sub(1)]
                    )
                } else {
                    entry.command.clone()
                };

                let line_text = format!(
                    "{} {} {} {}x {}",
                    if is_selected { "►" } else { " " },
                    command_text,
                    success_indicator,
                    entry.execution_count,
                    time_ago
                );

                let mut style = Style::default();
                if is_selected {
                    style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                }
                if !entry.success {
                    style = style.fg(Color::Red);
                }

                // Highlight matching characters
                if !history_match.indices.is_empty() && is_selected {
                    style = style.fg(Color::Yellow);
                }

                Line::from(vec![Span::styled(line_text, style)])
            })
            .collect();

        let title = if self.state.search_query.is_empty() {
            "Command History (↑/↓ navigate, Enter to execute, Tab to edit, / to search)"
        } else {
            "History Search (Esc to clear search)"
        };

        let history_list = List::new(history_items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White));

        f.render_widget(history_list, area);
    }

    fn render_selected_command_preview(&self, f: &mut Frame, area: Rect) {
        if let Some(selected_match) = self.state.matches.get(self.state.selected_index) {
            let entry = &selected_match.entry;

            let metadata = vec![
                format!("Executed: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S")),
                format!("Run count: {}", entry.execution_count),
                format!(
                    "Status: {}",
                    if entry.success { "Success" } else { "Failed" }
                ),
                format!("Duration: {}ms", entry.duration_ms.unwrap_or(0)),
            ];

            let content = format!("{}\n\n{}", metadata.join("\n"), entry.command);

            let preview = Paragraph::new(content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Command Details"),
                )
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(preview, area);
        }
    }

    fn format_time_ago(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let elapsed = chrono::Utc::now() - *timestamp;
        if elapsed.num_days() > 0 {
            format!("{}d", elapsed.num_days())
        } else if elapsed.num_hours() > 0 {
            format!("{}h", elapsed.num_hours())
        } else if elapsed.num_minutes() > 0 {
            format!("{}m", elapsed.num_minutes())
        } else {
            "now".to_string()
        }
    }

    /// Get the current state (for persistence/restoration)
    pub fn get_state(&self) -> &HistoryState {
        &self.state
    }

    /// Restore state (for mode switching)
    pub fn set_state(&mut self, state: HistoryState) {
        self.state = state;
    }

    /// Get selected command if any
    pub fn get_selected_command(&self) -> Option<String> {
        self.state
            .matches
            .get(self.state.selected_index)
            .map(|m| m.entry.command.clone())
    }
}

/// Actions that can result from history widget interaction
#[derive(Debug, Clone)]
pub enum HistoryAction {
    None,
    Exit,
    Quit,
    ExecuteCommand(String),
    UseCommand(String),
    StartSearch,
}

impl DebugInfoProvider for HistoryWidget {
    fn debug_info(&self) -> String {
        let mut info = String::from("=== HISTORY WIDGET ===\n");
        info.push_str(&format!("Search Query: '{}'\n", self.state.search_query));
        info.push_str(&format!("Total Matches: {}\n", self.state.matches.len()));
        info.push_str(&format!("Selected Index: {}\n", self.state.selected_index));

        if !self.state.matches.is_empty() && self.state.selected_index < self.state.matches.len() {
            info.push_str(&format!("\nCurrent Selection:\n"));
            let current = &self.state.matches[self.state.selected_index];
            info.push_str(&format!(
                "  Command: '{}'\n",
                if current.entry.command.len() > 50 {
                    format!("{}...", &current.entry.command[..50])
                } else {
                    current.entry.command.clone()
                }
            ));
            info.push_str(&format!("  Score: {:?}\n", current.score));
        }

        info.push_str(&format!("\nHistory Stats:\n"));
        info.push_str(&format!(
            "  Total Entries: {}\n",
            self.command_history.get_all().len()
        ));

        info
    }

    fn debug_summary(&self) -> String {
        format!(
            "HistoryWidget: {} matches, idx={}",
            self.state.matches.len(),
            self.state.selected_index
        )
    }
}
