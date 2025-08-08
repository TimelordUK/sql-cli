use crate::datatable::DataTable;
use crate::datatable_view::{DataTableView, SortOrder, ViewMode};
use crate::modern_input::{InputMode, ModernInput};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};
use std::io;

/// The main modern TUI application - stripped down and focused
pub struct ModernTui {
    /// The current view we're displaying
    view: DataTableView,

    /// Modern input system
    input: ModernInput,

    /// Terminal state
    should_quit: bool,

    /// Current mode
    mode: TuiMode,

    /// Viewport dimensions (for virtualization)
    viewport_height: u16,
    viewport_width: u16,
}

/// TUI application modes
#[derive(Debug, Clone, PartialEq)]
enum TuiMode {
    Query,   // User is typing/editing queries
    Results, // User is viewing/navigating results
}

impl ModernTui {
    /// Create a new modern TUI with a DataTable
    pub fn new(table: DataTable) -> Self {
        let mut input = ModernInput::new();

        // Set schema context for better history matching
        let columns: Vec<String> = table.column_names();
        let table_name = table.name.clone();
        input.set_schema_context(columns, Some(table_name.clone()));

        // Set initial query like enhanced TUI does
        input.set_text(format!("SELECT * FROM {}", table_name));

        Self {
            view: DataTableView::new(table),
            input,
            should_quit: false,
            mode: TuiMode::Results, // Start in Results mode since we have data
            viewport_height: 20,
            viewport_width: 80,
        }
    }

    /// Main run loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            // Draw the UI
            terminal.draw(|f| {
                // Update viewport before drawing
                let size = f.area();
                self.view.update_viewport(size.width, size.height);
                self.draw(f)
            })?;

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key) {
                        break;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        // Global quit keys
        if matches!(key.code, KeyCode::Char('q')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return true;
        }

        // Mode switching
        match key.code {
            KeyCode::Tab => {
                self.toggle_mode();
                return false;
            }
            KeyCode::Esc => {
                // Always go back to query mode on Esc
                self.mode = TuiMode::Query;
                self.view.exit_special_mode();
                return false;
            }
            _ => {}
        }

        // Handle based on current mode
        match self.mode {
            TuiMode::Query => self.handle_query_mode(key),
            TuiMode::Results => self.handle_results_mode(key),
        }
    }

    /// Toggle between query and results mode
    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            TuiMode::Query => TuiMode::Results,
            TuiMode::Results => TuiMode::Query,
        };
    }

    /// Handle keys in query mode
    fn handle_query_mode(&mut self, key: KeyEvent) -> bool {
        // Let input handle most keys
        if self.input.handle_key_event(key) {
            return false;
        }

        // Handle keys not handled by input
        match key.code {
            KeyCode::Enter => {
                // Execute query (for now just switch to results)
                let query = self.input.text().to_string();
                if !query.trim().is_empty() {
                    // TODO: Actually execute the query and update the view
                    // For now, just add to history and switch modes
                    self.input.add_to_history(query, true, None);
                    self.mode = TuiMode::Results;
                }
                false
            }
            _ => false,
        }
    }

    /// Handle keys in results mode
    fn handle_results_mode(&mut self, key: KeyEvent) -> bool {
        // Handle view-specific keys first
        match self.view.mode() {
            ViewMode::Normal => self.handle_results_normal_mode(key),
            ViewMode::Filtering => {
                self.view.handle_filter_input(key);
                false
            }
            ViewMode::Searching => {
                self.view.handle_search_input(key);
                false
            }
            ViewMode::Sorting => {
                // TODO: Implement sort column selection
                self.view.exit_special_mode();
                false
            }
        }
    }

    /// Handle keys in results normal mode
    fn handle_results_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Navigation
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => {
                self.view.handle_navigation(key);
            }

            // Enter filter mode
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.view.enter_filter_mode();
            }

            // Enter search mode
            KeyCode::Char('/') => {
                self.view.enter_search_mode();
            }

            // Search navigation
            KeyCode::Char('n') if key.modifiers.is_empty() => {
                self.view.next_search_match();
            }
            KeyCode::Char('N') if key.modifiers.is_empty() => {
                self.view.prev_search_match();
            }

            // Sort by current column
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                // Sort ascending by current column
                self.view
                    .apply_sort(self.get_selected_column(), SortOrder::Ascending);
            }
            KeyCode::Char('S') if key.modifiers.is_empty() => {
                // Sort descending by current column
                self.view
                    .apply_sort(self.get_selected_column(), SortOrder::Descending);
            }

            // Clear filters/search/sort
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.view.clear_filter();
                self.view.clear_search();
                self.view.clear_sort();
            }

            // Clear filter only
            KeyCode::Char('F') if key.modifiers.is_empty() => {
                self.view.clear_filter();
            }

            // Quit
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                self.should_quit = true;
                return true;
            }

            _ => {}
        }

        false
    }

    /// Draw the UI
    fn draw(&mut self, f: &mut Frame) {
        let size = f.area();
        self.viewport_height = size.height.saturating_sub(6); // Reserve space for input and status
        self.viewport_width = size.width;

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Query input area
                Constraint::Min(5),    // Main table area
                Constraint::Length(3), // Status area
                Constraint::Length(1), // Help line
            ])
            .split(size);

        // Draw query input - highlight if active
        let input_widget = if self.mode == TuiMode::Query {
            self.input.create_widget().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Query")
                    .style(Style::default().fg(Color::Yellow)),
            )
        } else {
            self.input.create_widget().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Query")
                    .style(Style::default().fg(Color::DarkGray)),
            )
        };
        f.render_widget(input_widget, chunks[0]);

        // Draw the main table - highlight if active
        let table_widget = if self.mode == TuiMode::Results {
            self.view.create_table_widget().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Data")
                    .style(Style::default()),
            )
        } else {
            self.view.create_table_widget().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Data")
                    .style(Style::default().fg(Color::DarkGray)),
            )
        };
        f.render_widget(table_widget, chunks[1]);

        // Draw status information
        let mut status_lines = Vec::new();

        // Current mode
        let mode_text = match self.mode {
            TuiMode::Query => "Query Mode",
            TuiMode::Results => "Results Mode",
        };
        status_lines.push(format!("Mode: {}", mode_text));

        // Input status (history search, etc.)
        let input_status = self.input.get_status();
        if !input_status.is_empty() {
            status_lines.push(input_status);
        }

        // View status (filters, search, etc.)
        let view_status = self.view.get_status_info();
        if !view_status.is_empty() {
            status_lines.push(view_status);
        }

        let status_text = status_lines.join(" | ");
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .alignment(Alignment::Left);
        f.render_widget(status, chunks[2]);

        // Draw help line
        let help_text = self.get_help_text();
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);
        f.render_widget(help, chunks[3]);

        // Draw input overlay if view is in special mode
        if let Some(input_widget) = self.view.create_input_widget() {
            self.draw_input_overlay(f, input_widget);
        }

        // Draw selected cell value if in results mode and available
        if self.mode == TuiMode::Results {
            if let Some(value) = self.view.get_selected_value() {
                self.draw_cell_inspector(f, value);
            }
        }
    }

    /// Draw an input overlay for filter/search modes
    fn draw_input_overlay(&self, f: &mut Frame, widget: Paragraph) {
        let size = f.area();

        // Create a centered popup area
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ])
            .split(size)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];

        // Clear the area and render the widget
        f.render_widget(Clear, popup_area);
        f.render_widget(widget, popup_area);
    }

    /// Draw cell value inspector (small overlay showing full cell content)
    fn draw_cell_inspector(&self, f: &mut Frame, value: &crate::datatable::DataValue) {
        let size = f.area();

        // Only show if the value is long enough to be worth inspecting
        let value_str = value.to_string();
        if value_str.len() <= 20 {
            return;
        }

        // Create inspector area in bottom right
        let inspector_width = 40.min(size.width / 3);
        let inspector_height = 5.min(size.height / 4);

        let inspector_area = ratatui::layout::Rect {
            x: size.width.saturating_sub(inspector_width + 1),
            y: size.height.saturating_sub(inspector_height + 1),
            width: inspector_width,
            height: inspector_height,
        };

        let inspector = Paragraph::new(value_str)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Cell Value")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(Clear, inspector_area);
        f.render_widget(inspector, inspector_area);
    }

    /// Get help text based on current mode
    fn get_help_text(&self) -> String {
        match (&self.mode, &self.view.mode(), &self.input.mode()) {
            // Query mode help
            (TuiMode::Query, _, InputMode::Normal) => {
                "Query Mode: Type SQL | ↑↓: History | Ctrl+R: Search History | Enter: Execute | Tab: Switch to Results | Ctrl+Q: Quit".to_string()
            }
            (TuiMode::Query, _, InputMode::HistorySearch) => {
                "History Search: Ctrl+R: Next | Enter: Select | Esc: Cancel | Type to search...".to_string()
            }
            (TuiMode::Query, _, InputMode::HistoryNav) => {
                "History Navigation: ↑↓: Navigate | Enter: Select | Esc: Cancel | Any key: Edit".to_string()
            }

            // Results mode help
            (TuiMode::Results, ViewMode::Normal, _) => {
                "Results Mode: ↑↓←→: Navigate | f: Filter | /: Search | s/S: Sort | Tab: Switch to Query | Esc: Query Mode".to_string()
            }
            (TuiMode::Results, ViewMode::Filtering, _) => {
                "Filter Mode: Enter: Apply | Esc: Cancel | Type to filter...".to_string()
            }
            (TuiMode::Results, ViewMode::Searching, _) => {
                "Search Mode: Enter: Apply | Esc: Cancel | n/N: Next/Prev | Type to search...".to_string()
            }
            (TuiMode::Results, ViewMode::Sorting, _) => {
                "Sort Mode: Select column | Esc: Cancel".to_string()
            }
        }
    }

    /// Get the currently selected column index
    fn get_selected_column(&self) -> usize {
        self.view.get_selected_column()
    }
}

/// Create and run the modern TUI with a DataTable
pub fn run_modern_tui(table: DataTable) -> io::Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let mut app = ModernTui::new(table);
    let result = app.run(&mut terminal);

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatable::{DataColumn, DataRow, DataType, DataValue};

    fn create_test_table() -> DataTable {
        let mut table = DataTable::new("test");

        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("name").with_type(DataType::String));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Alice".to_string()),
            ]))
            .unwrap();

        table
    }

    #[test]
    fn test_modern_tui_creation() {
        let table = create_test_table();
        let tui = ModernTui::new(table);

        assert!(!tui.should_quit);
        assert_eq!(tui.view.visible_row_count(), 1);
    }

    #[test]
    fn test_key_handling() {
        let table = create_test_table();
        let mut tui = ModernTui::new(table);

        // Test quit key
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let should_quit = tui.handle_key_event(quit_key);
        assert!(should_quit);
        assert!(tui.should_quit);
    }

    #[test]
    fn test_filter_mode() {
        let table = create_test_table();
        let mut tui = ModernTui::new(table);

        // Enter filter mode
        let filter_key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        tui.handle_key_event(filter_key);

        assert_eq!(tui.view.mode(), ViewMode::Filtering);
    }
}
