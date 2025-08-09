use crate::api_client::QueryResponse;
use crate::buffer::SortOrder;
use crate::buffer::{AppMode, BufferAPI};
use crate::config::Config;
use crate::sql_highlighter::SqlHighlighter;
use crate::tui_state::SelectionMode;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};
use regex::Regex;

/// Handles all rendering operations for the TUI
pub struct TuiRenderer {
    config: Config,
    sql_highlighter: SqlHighlighter,
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Context needed for rendering various TUI components
pub struct RenderContext<'a> {
    pub buffer: &'a dyn BufferAPI,
    pub config: &'a Config,
    pub table_state: &'a TableState,
    pub selection_mode: SelectionMode,
    pub last_yanked: Option<(&'a str, &'a str)>,
    pub filter_active: bool,
    pub filter_pattern: &'a str,
    pub filter_regex: Option<&'a Regex>,
    pub sort_column: Option<usize>,
    pub sort_order: SortOrder,
    pub help_scroll: u16,
    pub debug_content: &'a str,
    pub debug_scroll: u16,
    pub history_matches: &'a [HistoryMatch],
    pub history_selected: usize,
    pub jump_input: &'a str,
    pub input_text: &'a str,
    pub cursor_token_pos: (usize, usize),
    pub current_token: Option<&'a str>,
    pub parser_error: Option<&'a str>,
}

#[derive(Clone)]
pub struct HistoryMatch {
    pub entry: HistoryEntry,
    pub score: i64,
    pub indices: Vec<usize>,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub execution_count: usize,
    pub duration_ms: Option<u64>,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
            sql_highlighter: SqlHighlighter::new(),
        }
    }
    /// Render the main status line at the bottom of the screen
    pub fn render_status_line(
        f: &mut Frame,
        area: Rect,
        mode: AppMode,
        buffer: &dyn BufferAPI,
        message: &str,
    ) {
        // This will contain the status line rendering logic
        // Extracted from enhanced_tui.rs render_status_line method

        let status_text = format!(
            "[{}] {} | {}",
            format!("{:?}", mode),
            buffer.get_status_message(),
            message
        );

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(status, area);
    }

    /// Render the results table
    pub fn render_table(
        f: &mut Frame,
        area: Rect,
        results: &QueryResponse,
        _selected_row: Option<usize>,
        _current_column: usize,
        _pinned_columns: &[usize],
    ) {
        // This will contain the table rendering logic
        // Extracted from enhanced_tui.rs render_table_immutable method

        if results.data.is_empty() {
            let empty_msg = Paragraph::new("No results to display")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Results"));
            f.render_widget(empty_msg, area);
            return;
        }

        // Table rendering logic here...
    }

    /// Render the help screen
    pub fn render_help(f: &mut Frame, area: Rect, scroll_offset: u16) {
        let help_text = vec![
            "=== SQL CLI Help ===",
            "",
            "NAVIGATION:",
            "  ↑/↓/←/→ or hjkl - Move cursor",
            "  PgUp/PgDn       - Page up/down",
            "  Home/End        - Go to start/end",
            "  g/G             - Go to first/last row",
            "",
            "MODES:",
            "  i               - Enter edit mode",
            "  Esc             - Exit to command mode",
            "  Enter           - Execute query",
            "  Tab             - Autocomplete",
            "",
            "OPERATIONS:",
            "  /               - Search",
            "  Ctrl+F          - Filter",
            "  s/S             - Sort ascending/descending",
            "  y/Y             - Yank cell/row",
            "  Ctrl+E          - Export to CSV",
            "  Ctrl+J          - Export to JSON",
            "",
            "FUNCTION KEYS:",
            "  F1              - This help",
            "  F5              - Debug mode",
            "  F6              - Pretty query",
            "  F7              - History",
            "  F8              - Cache",
            "  F9              - Statistics",
            "",
            "Press q or Esc to close help",
        ];

        let visible_height = area.height.saturating_sub(2) as usize;
        let start = scroll_offset as usize;
        let end = (start + visible_height).min(help_text.len());

        let visible_text: Vec<Line> = help_text[start..end]
            .iter()
            .map(|&line| Line::from(line))
            .collect();

        let help_widget = Paragraph::new(visible_text)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Help - Lines {}-{} of {}",
                start + 1,
                end,
                help_text.len()
            )))
            .style(Style::default().fg(Color::White));

        f.render_widget(help_widget, area);
    }

    /// Render the debug view
    pub fn render_debug(f: &mut Frame, area: Rect, debug_content: &str, scroll_offset: u16) {
        let visible_height = area.height.saturating_sub(2) as usize;
        let lines: Vec<&str> = debug_content.lines().collect();
        let total_lines = lines.len();
        let start = scroll_offset as usize;
        let end = (start + visible_height).min(total_lines);

        let visible_lines: Vec<Line> = lines[start..end]
            .iter()
            .map(|&line| Line::from(line.to_string()))
            .collect();

        let debug_text = Text::from(visible_lines);
        let has_error = debug_content.contains("ERROR");

        let (border_color, title) = if has_error {
            (Color::Red, "Debug Info [ERROR]")
        } else {
            (Color::Yellow, "Debug Info")
        };

        let debug_widget = Paragraph::new(debug_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "{} - Lines {}-{} of {}",
                        title,
                        start + 1,
                        end,
                        total_lines
                    ))
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(debug_widget, area);
    }

    /// Render the history view
    pub fn render_history(
        f: &mut Frame,
        area: Rect,
        history_items: &[String],
        selected_index: usize,
    ) {
        let items: Vec<ListItem> = history_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(item.as_str()).style(style)
            })
            .collect();

        let history_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command History"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_widget(history_list, area);
    }

    /// Render the cache view
    pub fn render_cache(
        f: &mut Frame,
        area: Rect,
        cache_entries: &[String],
        selected_index: usize,
    ) {
        let items: Vec<ListItem> = cache_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(entry.as_str()).style(style)
            })
            .collect();

        let cache_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Query Cache"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_widget(cache_list, area);
    }

    /// Render column statistics
    pub fn render_column_stats(f: &mut Frame, area: Rect, stats: &[(String, String)]) {
        let rows: Vec<Row> = stats
            .iter()
            .map(|(name, value)| {
                Row::new(vec![Cell::from(name.as_str()), Cell::from(value.as_str())])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Column Statistics"),
        )
        .style(Style::default());

        f.render_widget(table, area);
    }
}
