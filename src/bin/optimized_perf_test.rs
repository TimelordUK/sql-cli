use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};
use std::{io, time::Instant};

/// Pre-rendered cells to avoid allocations during render
struct PreRenderedData {
    headers: Vec<Cell<'static>>,
    rows: Vec<Vec<Cell<'static>>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Creating pre-rendered data...");
    let start = Instant::now();

    // Pre-render all cells
    let row_count = 10000;
    let headers = vec!["ID", "Name", "Email", "City", "Country", "Amount"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)))
        .collect();

    let mut rows = Vec::with_capacity(row_count);
    for i in 0..row_count {
        let row_cells = vec![
            Cell::from(i.to_string()),
            Cell::from(format!("Customer {}", i)),
            Cell::from(format!("customer{}@example.com", i)),
            Cell::from("New York"),
            Cell::from("USA"),
            Cell::from((i * 100).to_string()),
        ];
        rows.push(row_cells);
    }

    let data = PreRenderedData { headers, rows };
    eprintln!("Data created in {:?}", start.elapsed());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State
    let mut table_state = TableState::default();
    table_state.select(Some(0));
    let mut offset = 0usize;
    let mut frame_times = Vec::new();

    // Initial render
    terminal.draw(|f| {
        f.render_widget(Block::default().title("Loading..."), f.area());
    })?;

    loop {
        let frame_start = Instant::now();

        // Handle events - blocking read for better responsiveness
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if let Some(selected) = table_state.selected() {
                        if selected > 0 {
                            table_state.select(Some(selected - 1));
                        } else if offset > 0 {
                            offset -= 1;
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(selected) = table_state.selected() {
                        if selected < 29 && offset + selected + 1 < data.rows.len() {
                            table_state.select(Some(selected + 1));
                        } else if offset + 30 < data.rows.len() {
                            offset += 1;
                        }
                    }
                }
                KeyCode::PageUp => {
                    offset = offset.saturating_sub(30);
                }
                KeyCode::PageDown => {
                    offset = (offset + 30).min(data.rows.len().saturating_sub(30));
                }
                KeyCode::Char('g') => {
                    offset = 0;
                    table_state.select(Some(0));
                }
                KeyCode::Char('G') => {
                    offset = data.rows.len().saturating_sub(30);
                    table_state.select(Some(29));
                }
                _ => continue, // Don't render if key not handled
            }

            // Render after handling event
            let render_start = Instant::now();
            terminal.draw(|f| {
                let area = f.area();
                let visible_height = area.height.saturating_sub(4) as usize;
                let end_offset = (offset + visible_height).min(data.rows.len());

                // Create header row
                let header = Row::new(data.headers.clone());

                // Create visible rows - just clone pre-rendered cells
                let rows: Vec<Row> = data.rows[offset..end_offset]
                    .iter()
                    .map(|cells| Row::new(cells.clone()))
                    .collect();

                let widths = vec![Constraint::Length(10); 6];

                let selected_abs = offset + table_state.selected().unwrap_or(0);
                let avg_frame = if frame_times.len() > 10 {
                    let sum: f64 = frame_times.iter().skip(frame_times.len() - 10).sum();
                    sum / 10.0
                } else {
                    0.0
                };

                let table = Table::new(rows, widths)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title(format!(
                        "Row {}/{} | Avg frame: {:.1}ms | 'q' to quit",
                        selected_abs + 1,
                        data.rows.len(),
                        avg_frame
                    )))
                    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">> ");

                f.render_stateful_widget(table, area, &mut table_state.clone());
            })?;

            let render_time = render_start.elapsed();
            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time.as_secs_f64() * 1000.0);

            if render_time.as_millis() > 10 {
                eprintln!("Render took {}ms", render_time.as_millis());
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // Report stats
    if !frame_times.is_empty() {
        let avg: f64 = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
        let max = frame_times.iter().fold(0.0f64, |a, &b| a.max(b));
        let min = frame_times.iter().fold(f64::MAX, |a, &b| a.min(b));

        println!("Frame time stats:");
        println!("  Average: {:.2}ms", avg);
        println!("  Min: {:.2}ms", min);
        println!("  Max: {:.2}ms", max);
        println!("  Total frames: {}", frame_times.len());
    }

    Ok(())
}
