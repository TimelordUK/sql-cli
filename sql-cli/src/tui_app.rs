use crate::api_client::{ApiClient, QueryResponse};
use crate::parser::SqlParser;
use crate::cursor_aware_parser::CursorAwareParser;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use serde_json::Value;
use std::io;
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Clone, PartialEq)]
enum AppMode {
    Command,
    Results,
}

#[derive(Clone)]
pub struct TuiApp {
    api_client: ApiClient,
    input: Input,
    mode: AppMode,
    results: Option<QueryResponse>,
    table_state: TableState,
    show_help: bool,
    status_message: String,
    sql_parser: SqlParser,
    cursor_parser: CursorAwareParser,
}

impl TuiApp {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            mode: AppMode::Command,
            results: None,
            table_state: TableState::default(),
            show_help: false,
            status_message: "Ready - Type SQL query and press Enter (Enhanced parser)".to_string(),
            sql_parser: SqlParser::new(),
            cursor_parser: CursorAwareParser::new(),
        }
    }
    
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        if self.show_help {
                            self.show_help = false;
                        } else if self.mode == AppMode::Results {
                            self.mode = AppMode::Command;
                        } else {
                            break; // Exit app
                        }
                    }
                    KeyCode::F(1) => {
                        self.show_help = !self.show_help;
                    }
                    KeyCode::Enter => {
                        if self.mode == AppMode::Command && !self.input.value().trim().is_empty() {
                            self.execute_query();
                        }
                    }
                    KeyCode::Tab => {
                        if self.mode == AppMode::Command {
                            self.handle_tab_completion();
                        }
                    }
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        if self.mode == AppMode::Results {
                            self.handle_navigation(key.code);
                        } else if key.code == KeyCode::Up || key.code == KeyCode::Down {
                            // Could add command history here
                        } else {
                            // Handle cursor movement in input
                            self.input.handle_event(&Event::Key(key));
                        }
                    }
                    KeyCode::PageUp | KeyCode::PageDown => {
                        if self.mode == AppMode::Results {
                            self.handle_navigation(key.code);
                        }
                    }
                    _ => {
                        if self.mode == AppMode::Command {
                            self.input.handle_event(&Event::Key(key));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn execute_query(&mut self) {
        let query = self.input.value().trim();
        self.status_message = format!("Executing: {}", query);
        
        match self.api_client.query_trades(query) {
            Ok(response) => {
                self.results = Some(response);
                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
                self.status_message = format!("Query executed successfully - {} rows", 
                    self.results.as_ref().unwrap().data.len());
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }
    
    fn handle_tab_completion(&mut self) {
        // Basic completion - could be enhanced with proper parsing
        let input_text = self.input.value().to_string();
        let suggestions = self.get_completions(&input_text);
        
        if !suggestions.is_empty() {
            // For now, just complete the first suggestion
            // In a full implementation, you'd show a popup with options
            let suggestion = &suggestions[0];
            let words: Vec<&str> = input_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                if suggestion.to_lowercase().starts_with(&last_word.to_lowercase()) {
                    let new_input = format!("{}{} ", input_text.trim_end_matches(last_word), suggestion);
                    self.input = Input::from(new_input);
                    // Move cursor to end
                    while self.input.cursor() < self.input.value().len() {
                        self.input.handle_event(&Event::Key(crossterm::event::KeyEvent::new(
                            KeyCode::Right, KeyModifiers::NONE
                        )));
                    }
                }
            }
        }
    }
    
    fn get_completions(&mut self, input: &str) -> Vec<String> {
        let cursor_pos = self.input.cursor(); // Get actual cursor position
        let result = self.cursor_parser.get_completions(input, cursor_pos);
        result.suggestions
    }
    
    fn handle_navigation(&mut self, key: KeyCode) {
        if let Some(results) = &self.results {
            let num_rows = results.data.len();
            if num_rows == 0 { return; }
            
            let current = self.table_state.selected().unwrap_or(0);
            let new_selection = match key {
                KeyCode::Up => {
                    if current > 0 { current - 1 } else { num_rows - 1 }
                }
                KeyCode::Down => {
                    if current < num_rows - 1 { current + 1 } else { 0 }
                }
                KeyCode::PageUp => {
                    if current > 10 { current - 10 } else { 0 }
                }
                KeyCode::PageDown => {
                    let new = current + 10;
                    if new < num_rows { new } else { num_rows - 1 }
                }
                _ => current,
            };
            self.table_state.select(Some(new_selection));
        }
    }
    
    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command input
                Constraint::Min(5),    // Results area
                Constraint::Length(1), // Status bar
            ])
            .split(f.area());
        
        // Command input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("SQL Command");
        
        let input_style = if self.mode == AppMode::Command {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let input_paragraph = Paragraph::new(self.input.value())
            .block(input_block)
            .style(input_style);
        f.render_widget(input_paragraph, chunks[0]);
        
        // Set cursor position when in command mode
        if self.mode == AppMode::Command {
            f.set_cursor_position((
                chunks[0].x + self.input.visual_cursor() as u16 + 1,
                chunks[0].y + 1,
            ));
        }
        
        // Results area
        if let Some(results) = &self.results {
            self.render_results(f, chunks[1], results);
        } else {
            let help_text = if self.mode == AppMode::Command {
                vec![
                    Line::from("Enter your SQL query above and press Enter to execute"),
                    Line::from(""),
                    Line::from("Examples:"),
                    Line::from("  SELECT * FROM trade_deal"),
                    Line::from("  SELECT dealId, price FROM trade_deal WHERE price > 100"),
                    Line::from("  SELECT * FROM trade_deal WHERE ticker = 'AAPL'"),
                    Line::from(""),
                    Line::from("Controls:"),
                    Line::from("  Tab    - Auto-complete"),
                    Line::from("  F1     - Toggle help"),
                    Line::from("  Esc    - Exit"),
                ]
            } else {
                vec![Line::from("No results to display")]
            };
            
            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(help_paragraph, chunks[1]);
        }
        
        // Status bar
        let status_line = Line::from(vec![
            Span::styled(&self.status_message, Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled(
                match self.mode {
                    AppMode::Command => "CMD",
                    AppMode::Results => "VIEW",
                },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | F1=Help | Esc=Back/Exit"),
        ]);
        
        let status = Paragraph::new(status_line)
            .style(Style::default().bg(Color::DarkGray));
        f.render_widget(status, chunks[2]);
        
        // Help popup if active
        if self.show_help {
            self.render_help_popup(f);
        }
    }
    
    fn render_results(&self, f: &mut Frame, area: Rect, results: &QueryResponse) {
        let data = &results.data;
        let select_fields = &results.query.select;
        
        if data.is_empty() {
            let no_data = Paragraph::new("No data returned")
                .block(Block::default().borders(Borders::ALL).title("Results"));
            f.render_widget(no_data, area);
            return;
        }
        
        // Get headers from first record or from select fields
        let headers: Vec<String> = if select_fields.contains(&"*".to_string()) {
            if let Some(first) = data.first() {
                if let Some(obj) = first.as_object() {
                    obj.keys().map(|k| k.clone()).collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            select_fields.clone()
        };
        
        // Create table header
        let header_cells: Vec<ratatui::widgets::Cell> = headers
            .iter()
            .map(|h| ratatui::widgets::Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow)))
            .collect();
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        
        // Create table rows
        let rows: Vec<Row> = data.iter().map(|record| {
            let cells: Vec<ratatui::widgets::Cell> = headers.iter().map(|field| {
                if let Some(obj) = record.as_object() {
                    match obj.get(field) {
                        Some(Value::String(s)) => ratatui::widgets::Cell::from(s.as_str()),
                        Some(Value::Number(n)) => ratatui::widgets::Cell::from(n.to_string()),
                        Some(Value::Bool(b)) => ratatui::widgets::Cell::from(b.to_string()),
                        Some(Value::Null) => ratatui::widgets::Cell::from("NULL").style(Style::default().fg(Color::Gray)),
                        Some(v) => ratatui::widgets::Cell::from(v.to_string()),
                        None => ratatui::widgets::Cell::from(""),
                    }
                } else {
                    ratatui::widgets::Cell::from("")
                }
            }).collect();
            Row::new(cells).height(1)
        }).collect();
        
        // Calculate column widths
        let num_cols = headers.len();
        let col_width = if num_cols > 0 {
            (area.width.saturating_sub(2)) / num_cols as u16
        } else {
            10
        };
        
        let widths: Vec<Constraint> = (0..num_cols)
            .map(|_| Constraint::Length(col_width))
            .collect();
        
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Results ({} rows) - Use ↑↓ to navigate, Esc to return to command",
                data.len()
            )))
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(table, area, &mut self.table_state.clone());
    }
    
    fn render_help_popup(&self, f: &mut Frame) {
        let area = centered_rect(80, 60, f.area());
        f.render_widget(Clear, area);
        
        let help_text = vec![
            Line::from(vec![Span::styled("SQL CLI Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("Command Mode:"),
            Line::from("  Enter     - Execute query"),
            Line::from("  Tab       - Auto-complete"),
            Line::from("  Esc       - Exit application"),
            Line::from(""),
            Line::from("Results Mode:"),
            Line::from("  ↑↓        - Navigate rows"),
            Line::from("  Page Up/Down - Navigate pages"),
            Line::from("  Esc       - Return to command mode"),
            Line::from(""),
            Line::from("Global:"),
            Line::from("  F1        - Toggle this help"),
            Line::from(""),
            Line::from("Example Queries:"),
            Line::from("  SELECT * FROM trade_deal"),
            Line::from("  SELECT dealId, price FROM trade_deal WHERE price > 100"),
            Line::from("  SELECT * FROM trade_deal WHERE ticker = 'AAPL'"),
            Line::from("  SELECT * FROM trade_deal WHERE counterparty.Contains('Goldman')"),
            Line::from("  SELECT * FROM trade_deal ORDER BY price DESC"),
        ];
        
        let help_popup = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(help_popup, area);
    }
}

// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn run_tui_app() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Get API URL from environment or use default
    let api_url = std::env::var("TRADE_API_URL")
        .unwrap_or_else(|_| "http://localhost:5000".to_string());
    
    // Create and run app
    let mut app = TuiApp::new(&api_url);
    let res = app.run(&mut terminal);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}