use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Terminal,
};
use std::{io, time::Instant};

/// Simple row data - just a Vec of strings, no JSON
struct SimpleData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test data - 10k rows, 6 columns
    let row_count = 10000;
    let headers = vec!["ID", "Name", "Email", "City", "Country", "Amount"]
        .into_iter()
        .map(String::from)
        .collect();

    let mut rows = Vec::with_capacity(row_count);
    for i in 0..row_count {
        rows.push(vec![
            i.to_string(),
            format!("Customer {}", i),
            format!("customer{}@example.com", i),
            "New York".to_string(),
            "USA".to_string(),
            (i * 100).to_string(),
        ]);
    }

    let data = SimpleData { headers, rows };

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
        let area = f.area();
        let empty_rows: Vec<Row> = vec![];
        let empty_constraints: Vec<Constraint> = vec![];
        let table = Table::new(empty_rows, empty_constraints).block(
            Block::default()
                .borders(Borders::ALL)
                .title("10k rows loaded - Use arrows/g/G to navigate, 'q' to quit"),
        );
        f.render_widget(table, area);
    })?;

    loop {
        // Block waiting for events - no busy loop!
        if let Event::Key(key) = event::read()? {
            let render_start = Instant::now();

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
                    table_state.select(Some(29.min(data.rows.len() - offset - 1)));
                }
                _ => continue, // Don't render for unhandled keys
            }

            // Render after handling navigation
            terminal.draw(|f| {
                let area = f.area();

                // Calculate visible rows
                let visible_height = area.height.saturating_sub(4) as usize;
                let end_offset = (offset + visible_height).min(data.rows.len());

                // Create header
                let header_cells: Vec<_> = data
                    .headers
                    .iter()
                    .map(|h| {
                        ratatui::widgets::Cell::from(h.as_str())
                            .style(Style::default().fg(Color::Yellow))
                    })
                    .collect();
                let header = Row::new(header_cells);

                // Create only visible rows - using slice directly
                let visible_rows = &data.rows[offset..end_offset];
                let rows: Vec<Row> = visible_rows
                    .iter()
                    .map(|row| {
                        let cells: Vec<_> = row
                            .iter()
                            .map(|cell| ratatui::widgets::Cell::from(cell.as_str()))
                            .collect();
                        Row::new(cells)
                    })
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
                        "Simple String Data - Row {}/{} | Avg frame: {:.1}ms | 'q' to quit",
                        selected_abs + 1,
                        data.rows.len(),
                        avg_frame
                    )))
                    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">> ");

                f.render_stateful_widget(table, area, &mut table_state.clone());
            })?;

            let frame_time = render_start.elapsed();
            frame_times.push(frame_time.as_secs_f64() * 1000.0);

            if frame_time.as_millis() > 16 {
                eprintln!("Slow frame: {}ms", frame_time.as_millis());
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

        println!("\nFrame time stats:");
        println!("  Average: {:.2}ms", avg);
        println!("  Min: {:.2}ms", min);
        println!("  Max: {:.2}ms", max);
        println!("  Total frames: {}", frame_times.len());
    }

    Ok(())
}
