use crate::buffer::{AppMode, BufferAPI, SortState};
use crate::debug_info::DebugInfo;
use crate::hybrid_parser::HybridParser;
use crate::where_parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// A self-contained debug widget that manages its own state and rendering
pub struct DebugWidget {
    /// The debug content to display
    content: String,
    /// Current scroll offset
    scroll_offset: u16,
    /// Maximum scroll position
    max_scroll: u16,
}

impl DebugWidget {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            scroll_offset: 0,
            max_scroll: 0,
        }
    }

    /// Generate and set debug content
    pub fn generate_debug(
        &mut self,
        buffer: &dyn BufferAPI,
        buffer_count: usize,
        buffer_index: usize,
        buffer_names: Vec<String>,
        hybrid_parser: &HybridParser,
        sort_state: &SortState,
        input_text: &str,
        cursor_pos: usize,
        visual_cursor: usize,
        api_url: &str,
    ) {
        // Generate full debug info
        let mut debug_info = DebugInfo::generate_full_debug_simple(
            buffer,
            buffer_count,
            buffer_index,
            buffer_names,
            hybrid_parser,
            sort_state,
            input_text,
            cursor_pos,
            visual_cursor,
            api_url,
        );

        // Add WHERE clause AST if query contains WHERE
        if input_text.to_lowercase().contains(" where ") {
            let where_ast_info = match Self::parse_where_clause_ast(input_text) {
                Ok(ast_str) => ast_str,
                Err(e) => format!(
                    "\n========== WHERE CLAUSE AST ==========\nError parsing WHERE clause: {}\n",
                    e
                ),
            };
            debug_info.push_str(&where_ast_info);
        }

        self.content = debug_info;
        self.scroll_offset = 0;
        self.update_max_scroll();
    }

    /// Generate pretty formatted SQL
    pub fn generate_pretty_sql(&mut self, query: &str) {
        if !query.trim().is_empty() {
            let debug_text = format!(
                "Pretty SQL Query\n{}\n\n{}",
                "=".repeat(50),
                crate::recursive_parser::format_sql_pretty_compact(query, 5).join("\n")
            );
            self.content = debug_text;
            self.scroll_offset = 0;
            self.update_max_scroll();
        }
    }

    /// Generate test case content
    pub fn generate_test_case(&mut self, buffer: &dyn BufferAPI) {
        self.content = DebugInfo::generate_test_case(buffer);
        self.scroll_offset = 0;
        self.update_max_scroll();
    }

    /// Handle key events for the debug widget
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_up(1);
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_down(1);
                false
            }
            KeyCode::PageUp => {
                self.scroll_up(10);
                false
            }
            KeyCode::PageDown => {
                self.scroll_down(10);
                false
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll_to_top();
                false
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll_to_bottom();
                false
            }

            // Exit debug mode
            KeyCode::Esc | KeyCode::Char('q') => true,

            // Ctrl+C to copy debug content to clipboard
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // This would return true to signal the main app to copy content
                // The main app would handle the actual clipboard operation
                true
            }

            _ => false,
        }
    }

    /// Render the debug widget
    pub fn render(&self, f: &mut Frame, area: Rect, mode: AppMode) {
        let visible_height = area.height.saturating_sub(2) as usize;
        let visible_lines = self.get_visible_lines(visible_height);

        let debug_text = Text::from(visible_lines);
        let total_lines = self.content.lines().count();
        let start = self.scroll_offset as usize;
        let end = (start + visible_height).min(total_lines);

        // Check if there's a parse error
        let has_parse_error = self.content.contains("❌ PARSE ERROR ❌");
        let (border_color, title_prefix) = if has_parse_error {
            (Color::Red, "⚠️  Parser Debug Info [PARSE ERROR] ")
        } else {
            (Color::Yellow, "Parser Debug Info ")
        };

        let title = match mode {
            AppMode::Debug => format!(
                "{}- Lines {}-{} of {} (↑↓/jk: scroll, PgUp/PgDn: page, Home/g: top, End/G: bottom, q/Esc: exit)",
                title_prefix,
                start + 1,
                end,
                total_lines
            ),
            AppMode::PrettyQuery => {
                "Pretty SQL Query (F6) - ↑↓ to scroll, Esc/q to close".to_string()
            }
            _ => "Debug Info".to_string(),
        };

        let debug_paragraph = Paragraph::new(debug_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        f.render_widget(debug_paragraph, area);
    }

    /// Get the visible lines based on scroll offset
    pub fn get_visible_lines(&self, height: usize) -> Vec<Line<'static>> {
        let lines: Vec<&str> = self.content.lines().collect();
        let start = self.scroll_offset as usize;
        let end = (start + height).min(lines.len());

        lines[start..end]
            .iter()
            .map(|line| Line::from(line.to_string()))
            .collect()
    }

    /// Scroll up by the specified amount
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by the specified amount
    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll_offset = (self.scroll_offset + amount).min(self.max_scroll);
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll;
    }

    /// Update the maximum scroll position based on content
    fn update_max_scroll(&mut self) {
        let line_count = self.content.lines().count() as u16;
        self.max_scroll = line_count.saturating_sub(10); // Leave some visible lines
    }

    /// Get the current content (for clipboard operations)
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Set custom content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.scroll_offset = 0;
        self.update_max_scroll();
    }

    /// Parse WHERE clause and return AST representation
    fn parse_where_clause_ast(query: &str) -> Result<String, String> {
        // Find WHERE clause in the query
        let lower_query = query.to_lowercase();
        let where_pos = lower_query.find(" where ");

        if let Some(pos) = where_pos {
            let where_start = pos + 7; // Skip " where "
            let where_clause = &query[where_start..];

            // Find the end of WHERE clause (before ORDER BY, GROUP BY, LIMIT, etc.)
            let end_keywords = ["order by", "group by", "limit", "offset", ";"];
            let mut where_end = where_clause.len();

            for keyword in &end_keywords {
                if let Some(keyword_pos) = where_clause.to_lowercase().find(keyword) {
                    where_end = where_end.min(keyword_pos);
                }
            }

            let where_only = where_clause[..where_end].trim();

            match where_parser::WhereParser::parse(where_only) {
                Ok(ast) => {
                    let mut result = String::from("\n========== WHERE CLAUSE AST ==========\n");
                    result.push_str(&format!("Input: {}\n", where_only));
                    result.push_str(&format!("Parsed AST:\n{:#?}\n", ast));
                    Ok(result)
                }
                Err(e) => Err(format!("Failed to parse WHERE clause: {}", e)),
            }
        } else {
            Err("No WHERE clause found in query".to_string())
        }
    }
}

impl Default for DebugWidget {
    fn default() -> Self {
        Self::new()
    }
}
