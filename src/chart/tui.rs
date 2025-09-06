use crate::chart::{
    engine::ChartEngine,
    renderers::LineRenderer,
    types::{ChartConfig, ChartViewport, DataSeries},
};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

pub struct ChartTui {
    config: ChartConfig,
    data: Option<DataSeries>,
    renderer: Option<LineRenderer>,
    should_exit: bool,
}

impl ChartTui {
    #[must_use]
    pub fn new(config: ChartConfig) -> Self {
        Self {
            config,
            data: None,
            renderer: None,
            should_exit: false,
        }
    }

    pub fn run(&mut self, mut chart_engine: ChartEngine) -> Result<()> {
        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Execute query and prepare data
        let data = chart_engine.execute_chart_query(&self.config)?;
        let viewport = ChartViewport::new(data.x_range, data.y_range);
        let mut renderer = LineRenderer::new(viewport);
        renderer.viewport_mut().auto_scale(&data);

        self.data = Some(data);
        self.renderer = Some(renderer);

        // Main event loop
        let result = self.run_event_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn run_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render_chart(frame))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key.code);
                    }
                }
            }
        }

        Ok(())
    }

    fn render_chart(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Chart
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Render title
        self.render_title(frame, chunks[0]);

        // Render chart
        if let (Some(data), Some(renderer)) = (&self.data, &self.renderer) {
            let chart_area = chunks[1].inner(Margin {
                horizontal: 1,
                vertical: 1,
            });
            renderer.render(frame, chart_area, data, &self.config);
        }

        // Render controls
        self.render_controls(frame, chunks[2]);
    }

    fn render_title(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let title = Paragraph::new(Line::from(vec![Span::styled(
            &self.config.title,
            Style::default().fg(Color::Yellow),
        )]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, area);
    }

    fn render_controls(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let controls = vec![
            Span::raw("Controls: "),
            Span::styled("hjkl", Style::default().fg(Color::Cyan)),
            Span::raw(" pan • "),
            Span::styled("+/-", Style::default().fg(Color::Cyan)),
            Span::raw(" zoom • "),
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" reset • "),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" quit"),
        ];

        let paragraph =
            Paragraph::new(Line::from(controls)).block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    fn handle_key_event(&mut self, key_code: KeyCode) {
        if let Some(renderer) = &mut self.renderer {
            match key_code {
                // Exit
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_exit = true;
                }

                // Pan (vim-style)
                KeyCode::Char('h') => {
                    renderer.viewport_mut().pan(-1.0, 0.0);
                }
                KeyCode::Char('l') => {
                    renderer.viewport_mut().pan(1.0, 0.0);
                }
                KeyCode::Char('k') => {
                    renderer.viewport_mut().pan(0.0, 1.0);
                }
                KeyCode::Char('j') => {
                    renderer.viewport_mut().pan(0.0, -1.0);
                }

                // Zoom
                KeyCode::Char('+' | '=') => {
                    let viewport = renderer.viewport();
                    let center_x = f64::midpoint(viewport.x_min, viewport.x_max);
                    let center_y = f64::midpoint(viewport.y_min, viewport.y_max);
                    renderer.viewport_mut().zoom(1.2, center_x, center_y);
                }
                KeyCode::Char('-') => {
                    let viewport = renderer.viewport();
                    let center_x = f64::midpoint(viewport.x_min, viewport.x_max);
                    let center_y = f64::midpoint(viewport.y_min, viewport.y_max);
                    renderer.viewport_mut().zoom(0.8, center_x, center_y);
                }

                // Reset view
                KeyCode::Char('r') => {
                    if let Some(data) = &self.data {
                        renderer.viewport_mut().auto_scale(data);
                    }
                }

                _ => {}
            }
        }
    }
}
