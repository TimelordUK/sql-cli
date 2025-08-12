use crate::debug_service::DebugProvider;
use crate::help_text::HelpText;
use crate::service_container::ServiceContainer;
use crate::widget_traits::DebugInfoProvider;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Actions that can be returned from the help widget
#[derive(Debug, Clone)]
pub enum HelpAction {
    None,
    Exit,
    ShowDebug,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Home,
    End,
    Search(String),
}

/// State for the help widget
#[derive(Debug, Clone)]
pub struct HelpState {
    /// Current scroll offset
    pub scroll_offset: u16,

    /// Maximum scroll position
    pub max_scroll: u16,

    /// Search query within help
    pub search_query: String,

    /// Whether search mode is active
    pub search_active: bool,

    /// Current search match index
    pub search_match_index: usize,

    /// All search match positions
    pub search_matches: Vec<usize>,

    /// Selected help section
    pub selected_section: HelpSection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HelpSection {
    General,
    Commands,
    Navigation,
    Search,
    Advanced,
    Debug,
}

impl Default for HelpState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            max_scroll: 0,
            search_query: String::new(),
            search_active: false,
            search_match_index: 0,
            search_matches: Vec::new(),
            selected_section: HelpSection::General,
        }
    }
}

/// Help widget that manages its own state and rendering
pub struct HelpWidget {
    state: HelpState,
    services: Option<ServiceContainer>,
}

impl HelpWidget {
    pub fn new() -> Self {
        Self {
            state: HelpState::default(),
            services: None,
        }
    }

    /// Set the service container for debug capabilities
    pub fn set_services(&mut self, services: ServiceContainer) {
        self.services = Some(services);
        self.log_debug("HelpWidget initialized with services");
    }

    /// Log a debug message if services are available
    fn log_debug(&self, message: &str) {
        if let Some(ref services) = self.services {
            services
                .debug_service
                .info("HelpWidget", message.to_string());
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> HelpAction {
        self.log_debug(&format!("Handling key: {:?}", key));

        // F5 should exit help and show debug - let main app handle it
        if key.code == KeyCode::F(5) {
            self.log_debug("F5 pressed - exiting help to show debug");
            return HelpAction::Exit;
        }

        // Handle search mode
        if self.state.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.state.search_active = false;
                    self.state.search_query.clear();
                    self.state.search_matches.clear();
                    self.log_debug("Search mode exited");
                    return HelpAction::None;
                }
                KeyCode::Enter => {
                    self.perform_search();
                    return HelpAction::None;
                }
                KeyCode::Char(c) => {
                    self.state.search_query.push(c);
                    return HelpAction::None;
                }
                KeyCode::Backspace => {
                    self.state.search_query.pop();
                    return HelpAction::None;
                }
                _ => return HelpAction::None,
            }
        }

        // Normal mode key handling
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.log_debug("Exit requested");
                HelpAction::Exit
            }
            KeyCode::F(1) => {
                self.log_debug("Help exit via F1");
                HelpAction::Exit
            }
            KeyCode::Char('/') => {
                self.state.search_active = true;
                self.log_debug("Search mode activated");
                HelpAction::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
                HelpAction::ScrollDown
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
                HelpAction::ScrollUp
            }
            KeyCode::Char('G') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.scroll_to_end();
                HelpAction::End
            }
            KeyCode::Char('g') => {
                self.scroll_to_home();
                HelpAction::Home
            }
            KeyCode::PageDown | KeyCode::Char(' ') => {
                self.page_down();
                HelpAction::PageDown
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.page_up();
                HelpAction::PageUp
            }
            KeyCode::Home => {
                self.scroll_to_home();
                HelpAction::Home
            }
            KeyCode::End => {
                self.scroll_to_end();
                HelpAction::End
            }
            // Section navigation with number keys
            KeyCode::Char('1') => {
                self.state.selected_section = HelpSection::General;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to General section");
                HelpAction::None
            }
            KeyCode::Char('2') => {
                self.state.selected_section = HelpSection::Commands;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to Commands section");
                HelpAction::None
            }
            KeyCode::Char('3') => {
                self.state.selected_section = HelpSection::Navigation;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to Navigation section");
                HelpAction::None
            }
            KeyCode::Char('4') => {
                self.state.selected_section = HelpSection::Search;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to Search section");
                HelpAction::None
            }
            KeyCode::Char('5') => {
                self.state.selected_section = HelpSection::Advanced;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to Advanced section");
                HelpAction::None
            }
            KeyCode::Char('6') => {
                self.state.selected_section = HelpSection::Debug;
                self.state.scroll_offset = 0;
                self.log_debug("Switched to Debug section");
                HelpAction::None
            }
            _ => HelpAction::None,
        }
    }

    /// Perform search within help content
    fn perform_search(&mut self) {
        self.log_debug(&format!("Searching for: {}", self.state.search_query));
        // TODO: Implement actual search logic
        self.state.search_matches.clear();
    }

    /// Scroll helpers
    fn scroll_up(&mut self) {
        if self.state.scroll_offset > 0 {
            self.state.scroll_offset = self.state.scroll_offset.saturating_sub(1);
        }
    }

    fn scroll_down(&mut self) {
        if self.state.scroll_offset < self.state.max_scroll {
            self.state.scroll_offset = self.state.scroll_offset.saturating_add(1);
        }
    }

    fn page_up(&mut self) {
        self.state.scroll_offset = self.state.scroll_offset.saturating_sub(10);
    }

    fn page_down(&mut self) {
        self.state.scroll_offset = (self.state.scroll_offset + 10).min(self.state.max_scroll);
    }

    fn scroll_to_home(&mut self) {
        self.state.scroll_offset = 0;
    }

    fn scroll_to_end(&mut self) {
        self.state.scroll_offset = self.state.max_scroll;
    }

    /// Render the help widget
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Log that we're rendering help
        self.log_debug(&format!(
            "Rendering help - section: {:?}, scroll: {}/{}",
            self.state.selected_section, self.state.scroll_offset, self.state.max_scroll
        ));

        // Simple rendering - no split screen
        self.render_help_content(f, area);
    }

    /// Render the main help content
    fn render_help_content(&mut self, f: &mut Frame, area: Rect) {
        // Create layout with header
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with sections
                Constraint::Min(0),    // Content
                Constraint::Length(2), // Status/search bar
            ])
            .split(area);

        // Render section tabs
        self.render_section_tabs(f, chunks[0]);

        // Render content based on selected section - now with two columns for some sections
        match self.state.selected_section {
            HelpSection::General => {
                // General section uses two columns
                self.render_two_column_content(f, chunks[1]);
            }
            _ => {
                // Other sections use single column for now
                self.render_single_column_content(f, chunks[1]);
            }
        }

        // Render status/search bar
        self.render_status_bar(f, chunks[2]);
    }

    /// Render content in two columns
    fn render_two_column_content(&mut self, f: &mut Frame, area: Rect) {
        // Split into two columns
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Get the two-column content from HelpText
        let left_content = HelpText::left_column();
        let right_content = HelpText::right_column();

        // Calculate visible area for scrolling
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        let max_lines = left_content.len().max(right_content.len());
        self.state.max_scroll = max_lines.saturating_sub(visible_height) as u16;

        // Apply scroll offset
        let scroll_offset = self.state.scroll_offset as usize;

        // Get visible portions with scrolling
        let left_visible: Vec<Line> = left_content
            .into_iter()
            .skip(scroll_offset)
            .take(visible_height)
            .collect();

        let right_visible: Vec<Line> = right_content
            .into_iter()
            .skip(scroll_offset)
            .take(visible_height)
            .collect();

        // Create scroll indicator in title
        let scroll_indicator = if max_lines > visible_height {
            format!(
                " ({}/{})",
                scroll_offset + 1,
                max_lines.saturating_sub(visible_height) + 1
            )
        } else {
            String::new()
        };

        // Convert Vec<Line> to Text for proper rendering
        let left_text = Text::from(left_visible);
        let right_text = Text::from(right_visible);

        // Render left column
        let left_paragraph = Paragraph::new(left_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Commands & Editing{}", scroll_indicator)),
            )
            .style(Style::default());

        // Render right column
        let right_paragraph = Paragraph::new(right_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Navigation & Features"),
            )
            .style(Style::default());

        f.render_widget(left_paragraph, chunks[0]);
        f.render_widget(right_paragraph, chunks[1]);
    }

    /// Render content in single column (for other tabs)
    fn render_single_column_content(&mut self, f: &mut Frame, area: Rect) {
        let content = self.get_section_content();

        // Calculate max scroll
        let visible_height = area.height.saturating_sub(2) as usize;
        let content_height = content.lines().count();
        self.state.max_scroll = content_height.saturating_sub(visible_height) as u16;

        // Create the paragraph with scrolling
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.get_section_title()),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.state.scroll_offset, 0));

        f.render_widget(paragraph, area);
    }

    /// Render section tabs
    fn render_section_tabs(&self, f: &mut Frame, area: Rect) {
        let sections = vec![
            ("1:General", HelpSection::General),
            ("2:Commands", HelpSection::Commands),
            ("3:Navigation", HelpSection::Navigation),
            ("4:Search", HelpSection::Search),
            ("5:Advanced", HelpSection::Advanced),
            ("6:Debug", HelpSection::Debug),
        ];

        let mut spans = Vec::new();
        for (i, (label, section)) in sections.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }

            let style = if *section == self.state.selected_section {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            spans.push(Span::styled(*label, style));
        }

        let tabs = Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help Sections"),
        );

        f.render_widget(tabs, area);
    }

    /// Get content for the selected section
    fn get_section_content(&self) -> String {
        match self.state.selected_section {
            HelpSection::General => {
                // Convert Vec<Line> to String
                HelpText::left_column()
                    .iter()
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            HelpSection::Commands => {
                // Convert Vec<Line> to String
                HelpText::right_column()
                    .iter()
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            HelpSection::Navigation => self.get_navigation_help(),
            HelpSection::Search => self.get_search_help(),
            HelpSection::Advanced => self.get_advanced_help(),
            HelpSection::Debug => self.get_debug_help(),
        }
    }

    /// Get section title
    fn get_section_title(&self) -> &str {
        match self.state.selected_section {
            HelpSection::General => "General Help",
            HelpSection::Commands => "Command Reference",
            HelpSection::Navigation => "Navigation",
            HelpSection::Search => "Search & Filter",
            HelpSection::Advanced => "Advanced Features",
            HelpSection::Debug => "Debug Information",
        }
    }

    fn get_navigation_help(&self) -> String {
        r#"NAVIGATION HELP

Within Results:
  ↑/↓         - Move between rows
  ←/→         - Scroll columns horizontally
  Home/End    - Jump to first/last row
  PgUp/PgDn   - Page up/down
  g           - Go to first row
  G           - Go to last row
  [number]g   - Go to row number
  
Column Navigation:
  Tab         - Next column
  Shift+Tab   - Previous column
  [number]    - Jump to column by number
  \           - Search for column by name
  
Selection Modes:
  v           - Toggle between row/cell selection
  V           - Select entire column
  Ctrl+A      - Select all
  
Viewport Control:
  Ctrl+L      - Lock/unlock viewport
  z           - Center current row
  zt          - Current row to top
  zb          - Current row to bottom"#
            .to_string()
    }

    fn get_search_help(&self) -> String {
        r#"SEARCH & FILTER HELP

Search Modes:
  /           - Search forward in results
  ?           - Search backward in results
  n           - Next search match
  N           - Previous search match
  *           - Search for word under cursor
  
Filter Modes:
  F           - Filter rows (case-sensitive)
  Shift+F     - Filter rows (case-insensitive)
  f           - Fuzzy filter
  Ctrl+F      - Clear all filters
  
Column Search:
  \           - Search for column by name
  Tab         - Next matching column
  Shift+Tab   - Previous matching column
  Enter       - Jump to column
  
Search Within Help:
  /           - Search in help text
  n           - Next match
  N           - Previous match
  Esc         - Exit search mode"#
            .to_string()
    }

    fn get_advanced_help(&self) -> String {
        r#"ADVANCED FEATURES

Query Management:
  Ctrl+S      - Save query to file
  Ctrl+O      - Open query from file
  Ctrl+R      - Query history
  Tab         - Auto-complete
  
Export Options:
  Ctrl+E, C   - Export to CSV
  Ctrl+E, J   - Export to JSON
  Ctrl+E, M   - Export to Markdown
  Ctrl+E, H   - Export to HTML
  
Cache Management:
  F7          - Show cache list
  Ctrl+K      - Clear cache
  :cache list - List cached results
  :cache clear - Clear all cache
  
Buffer Management:
  Ctrl+N      - New buffer
  Ctrl+Tab    - Next buffer
  Ctrl+Shift+Tab - Previous buffer
  :ls         - List all buffers
  :b [n]      - Switch to buffer n"#
            .to_string()
    }

    fn get_debug_help(&self) -> String {
        let mut help = String::from(
            r#"DEBUG FEATURES

Debug Keys:
  F5          - Toggle debug overlay (in help)
  F5          - Show full debug view (from main)
  Ctrl+D      - Dump state to clipboard
  
Debug Commands:
  :debug on   - Enable debug logging
  :debug off  - Disable debug logging
  :debug clear - Clear debug log
  :debug save  - Save debug log to file
  
Debug Information Available:
  - Application state
  - Mode transitions
  - SQL parser state
  - Buffer contents
  - Widget states
  - Performance metrics
  - Error logs
  
"#,
        );

        // Add current debug status if services available
        if let Some(ref services) = self.services {
            help.push_str(&format!(
                "\nDebug Status: {}\n",
                if services.debug_service.is_enabled() {
                    "ENABLED"
                } else {
                    "DISABLED"
                }
            ));

            let entries = services.debug_service.get_recent_entries(5);
            if !entries.is_empty() {
                help.push_str("\nRecent Debug Entries:\n");
                for entry in entries {
                    help.push_str(&format!(
                        "  [{} ms] {} - {}\n",
                        &entry.timestamp[9..], // Just show milliseconds part
                        entry.component,
                        entry.message
                    ));
                }
            }
        }

        help
    }

    /// Render status bar
    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        if self.state.search_active {
            spans.push(Span::styled("Search: ", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw(&self.state.search_query));
            spans.push(Span::raw(" (Enter to search, Esc to cancel)"));
        } else {
            spans.push(Span::raw("/:Search | "));
            let scroll_info = format!(
                "{}/{} ",
                self.state.scroll_offset + 1,
                self.state.max_scroll + 1
            );
            spans.push(Span::raw(scroll_info));
            spans.push(Span::styled(
                "| Esc:Exit",
                Style::default().fg(Color::DarkGray),
            ));
        }

        let status =
            Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

        f.render_widget(status, area);
    }

    /// Get current state for external use
    pub fn get_state(&self) -> &HelpState {
        &self.state
    }

    /// Reset the widget state
    pub fn reset(&mut self) {
        self.state = HelpState::default();
        self.log_debug("HelpWidget state reset");
    }

    /// Called when help mode is entered
    pub fn on_enter(&mut self) {
        self.log_debug("Help mode entered");
        // Reset to general section when entering
        self.state.selected_section = HelpSection::General;
        self.state.scroll_offset = 0;
    }

    /// Called when help mode is exited
    pub fn on_exit(&mut self) {
        self.log_debug("Help mode exited");
    }
}

impl DebugProvider for HelpWidget {
    fn component_name(&self) -> &str {
        "HelpWidget"
    }

    fn debug_info(&self) -> String {
        format!(
            "HelpWidget: section={:?}, scroll={}/{}, search_active={}",
            self.state.selected_section,
            self.state.scroll_offset,
            self.state.max_scroll,
            self.state.search_active
        )
    }

    fn debug_summary(&self) -> Option<String> {
        Some(format!("Help: {:?}", self.state.selected_section))
    }
}

impl DebugInfoProvider for HelpWidget {
    fn debug_info(&self) -> String {
        let mut info = String::from("=== HELP WIDGET ===\n");
        info.push_str(&format!("Section: {:?}\n", self.state.selected_section));
        info.push_str(&format!(
            "Scroll: {}/{}\n",
            self.state.scroll_offset, self.state.max_scroll
        ));
        info.push_str(&format!("Search Active: {}\n", self.state.search_active));
        if self.state.search_active {
            info.push_str(&format!("Search Query: '{}'\n", self.state.search_query));
            info.push_str(&format!("Matches: {}\n", self.state.search_matches.len()));
        }
        info
    }

    fn debug_summary(&self) -> String {
        format!(
            "HelpWidget: {:?} (scroll {}/{})",
            self.state.selected_section, self.state.scroll_offset, self.state.max_scroll
        )
    }
}
