use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use sql_cli::app_state_container::SelectionMode;
use sql_cli::buffer::AppMode;
use sql_cli::ui::input::actions::{Action, ActionContext};
use sql_cli::ui::key_handling::{ChordResult, KeyChordHandler, KeyMapper};
use std::collections::VecDeque;
use std::io;

const MAX_HISTORY: usize = 20;

struct ActionDebugger {
    key_mapper: KeyMapper,
    chord_handler: KeyChordHandler,
    action_history: VecDeque<String>,
    key_history: VecDeque<String>,
    current_mode: AppMode,
    selection_mode: SelectionMode,
    count_buffer: String,
    chord_status: Option<String>,
    should_quit: bool,
}

impl ActionDebugger {
    fn new() -> Self {
        Self {
            key_mapper: KeyMapper::new(),
            chord_handler: KeyChordHandler::new(),
            action_history: VecDeque::new(),
            key_history: VecDeque::new(),
            current_mode: AppMode::Results,
            selection_mode: SelectionMode::Row,
            count_buffer: String::new(),
            chord_status: None,
            should_quit: false,
        }
    }

    fn build_context(&self) -> ActionContext {
        ActionContext {
            mode: self.current_mode.clone(),
            selection_mode: self.selection_mode.clone(),
            has_results: true,
            has_filter: false,
            has_search: false,
            row_count: 100,
            column_count: 10,
            current_row: 5,
            current_column: 2,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Format key for display
        let key_str = format_key(&key);
        self.key_history.push_front(key_str.clone());
        if self.key_history.len() > MAX_HISTORY {
            self.key_history.pop_back();
        }

        // Process through chord handler first (for Results mode)
        if self.current_mode == AppMode::Results {
            let chord_result = self.chord_handler.process_key(key);

            match chord_result {
                ChordResult::CompleteChord(action) => {
                    let msg = format!(
                        "Chord completed: '{}' → {:?}",
                        self.chord_handler
                            .format_debug_info()
                            .lines()
                            .find(|l| l.starts_with("Current chord:"))
                            .unwrap_or("??"),
                        action
                    );
                    self.action_history.push_front(msg);
                    if self.action_history.len() > MAX_HISTORY {
                        self.action_history.pop_back();
                    }
                    self.chord_status = None;
                    return;
                }
                ChordResult::PartialChord(description) => {
                    self.chord_status = Some(description);
                    let msg = format!("Chord partial: '{}'", key_str);
                    self.action_history.push_front(msg);
                    if self.action_history.len() > MAX_HISTORY {
                        self.action_history.pop_back();
                    }
                    return;
                }
                ChordResult::Cancelled => {
                    self.chord_status = None;
                    let msg = format!("Chord cancelled");
                    self.action_history.push_front(msg);
                    if self.action_history.len() > MAX_HISTORY {
                        self.action_history.pop_back();
                    }
                    return;
                }
                ChordResult::SingleKey(_) => {
                    self.chord_status = None;
                    // Continue with normal key processing
                }
            }
        }

        // Check if we're collecting a count
        let was_collecting = self.key_mapper.is_collecting_count();
        let count_before = self.key_mapper.get_count_buffer().to_string();

        // Try to map the key to an action
        let context = self.build_context();
        let action = self.key_mapper.map_key(key, &context);

        // Check count buffer status after mapping
        let is_collecting = self.key_mapper.is_collecting_count();
        let count_after = self.key_mapper.get_count_buffer().to_string();

        // Build action message
        let action_msg = if was_collecting && !is_collecting && !count_before.is_empty() {
            // Count was applied
            format!(
                "Count '{}' + Key '{}' → {:?}",
                count_before, key_str, action
            )
        } else if is_collecting {
            // Still collecting count
            format!("Collecting count: '{}'", count_after)
        } else if let Some(ref act) = action {
            // Normal action
            format!("Key '{}' → {:?}", key_str, act)
        } else {
            // No mapping
            format!("Key '{}' → No mapping", key_str)
        };

        self.action_history.push_front(action_msg);
        if self.action_history.len() > MAX_HISTORY {
            self.action_history.pop_back();
        }

        // Handle the action
        if let Some(action) = action {
            self.process_action(action);
        }

        // Update count buffer display
        self.count_buffer = self.key_mapper.get_count_buffer().to_string();
    }

    fn process_action(&mut self, action: Action) {
        match action {
            Action::Quit | Action::ForceQuit => {
                self.should_quit = true;
            }
            Action::ToggleSelectionMode => {
                self.selection_mode = match self.selection_mode {
                    SelectionMode::Row => SelectionMode::Cell,
                    SelectionMode::Cell => SelectionMode::Column,
                    SelectionMode::Column => SelectionMode::Row,
                };
            }
            Action::SwitchMode(mode) => {
                self.current_mode = mode;
            }
            _ => {
                // Other actions just get logged
            }
        }
    }

    fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Status panel
                Constraint::Min(10),    // Action history
                Constraint::Length(10), // Key history
            ])
            .split(f.area());

        // Status panel
        self.draw_status(f, chunks[0]);

        // Action history
        self.draw_action_history(f, chunks[1]);

        // Key history
        self.draw_key_history(f, chunks[2]);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let status_text = vec![
            Line::from(vec![Span::styled(
                "Action Debugger",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Mode: "),
                Span::styled(
                    format!("{:?}", self.current_mode),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  Selection: "),
                Span::styled(
                    format!("{:?}", self.selection_mode),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Count Buffer: "),
                if self.count_buffer.is_empty() {
                    Span::styled("(none)", Style::default().fg(Color::DarkGray))
                } else {
                    Span::styled(
                        &self.count_buffer,
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    )
                },
                Span::raw("  Chord: "),
                if let Some(ref status) = self.chord_status {
                    Span::styled(
                        status,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled("(none)", Style::default().fg(Color::DarkGray))
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Try: ", Style::default().fg(Color::DarkGray)),
                Span::raw(
                    "j/k (nav), 5j (count+nav), v (mode), p (pin), s (sort), F1 (help), q (quit)",
                ),
            ]),
        ];

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, area);
    }

    fn draw_action_history(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .action_history
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let style = if i == 0 {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(msg.as_str()).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Action History (newest first)"),
        );
        f.render_widget(list, area);
    }

    fn draw_key_history(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .key_history
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let style = if i == 0 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(key.as_str()).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Key History (newest first)"),
        );
        f.render_widget(list, area);
    }
}

fn format_key(key: &KeyEvent) -> String {
    use crossterm::event::KeyModifiers;

    let mut result = String::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    // Add key code
    match key.code {
        KeyCode::Char(c) => result.push(c),
        KeyCode::F(n) => result.push_str(&format!("F{}", n)),
        KeyCode::Up => result.push_str("Up"),
        KeyCode::Down => result.push_str("Down"),
        KeyCode::Left => result.push_str("Left"),
        KeyCode::Right => result.push_str("Right"),
        KeyCode::PageUp => result.push_str("PageUp"),
        KeyCode::PageDown => result.push_str("PageDown"),
        KeyCode::Home => result.push_str("Home"),
        KeyCode::End => result.push_str("End"),
        KeyCode::Enter => result.push_str("Enter"),
        KeyCode::Tab => result.push_str("Tab"),
        KeyCode::BackTab => result.push_str("BackTab"),
        KeyCode::Backspace => result.push_str("Backspace"),
        KeyCode::Delete => result.push_str("Delete"),
        KeyCode::Insert => result.push_str("Insert"),
        KeyCode::Esc => result.push_str("Esc"),
        _ => result.push_str("Unknown"),
    }

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: ActionDebugger) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Filter out key release events
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                app.handle_key(key);

                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = ActionDebugger::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}
