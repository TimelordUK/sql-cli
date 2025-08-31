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
use serde_json::{json, Value};
use std::{io, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate JSON data like our actual app
    let row_count = 10000;
    let mut data = Vec::with_capacity(row_count);
    for i in 0..row_count {
        data.push(json!({
            "ID": i,
            "Name": format!("Customer {}", i),
            "Email": format!("customer{}@example.com", i),
            "City": "New York",
            "Country": "USA",
            "Amount": i * 100,
        }));
    }

    let headers = vec!["ID", "Name", "Email", "City", "Country", "Amount"];

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State - simulate our virtual table approach
    let mut offset = 0usize;
    let mut cursor = 0usize;
    let mut visible_rows = 30usize;
    let mut render_times = Vec::new();

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if cursor > 0 {
                        cursor -= 1;
                    } else if offset > 0 {
                        offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if cursor < visible_rows - 1 && offset + cursor + 1 < data.len() {
                        cursor += 1;
                    } else if offset + visible_rows < data.len() {
                        offset += 1;
                    }
                }
                KeyCode::Char('G') => {
                    if data.len() > visible_rows {
                        offset = data.len() - visible_rows;
                        cursor = visible_rows - 1;
                    } else {
                        offset = 0;
                        cursor = data.len() - 1;
                    }
                }
                _ => continue,
            }

            let render_start = Instant::now();
            terminal.draw(|f| {
                let area = f.area();
                visible_rows = area.height.saturating_sub(4) as usize;

                let end_offset = (offset + visible_rows).min(data.len());
                let visible_data = &data[offset..end_offset];

                // Create header
                let header_cells: Vec<_> = headers
                    .iter()
                    .map(|h| {
                        ratatui::widgets::Cell::from(*h).style(Style::default().fg(Color::Yellow))
                    })
                    .collect();
                let header = Row::new(header_cells);

                // Create rows - this is where JSON overhead happens
                let rows: Vec<Row> = visible_data
                    .iter()
                    .enumerate()
                    .map(|(_idx, record)| {
                        let cells: Vec<_> = headers
                            .iter()
                            .map(|field| {
                                if let Some(obj) = record.as_object() {
                                    match obj.get(*field) {
                                        Some(Value::String(s)) => {
                                            ratatui::widgets::Cell::from(s.as_str())
                                        }
                                        Some(Value::Number(n)) => {
                                            ratatui::widgets::Cell::from(n.to_string())
                                        }
                                        _ => ratatui::widgets::Cell::from(""),
                                    }
                                } else {
                                    ratatui::widgets::Cell::from("")
                                }
                            })
                            .collect();
                        Row::new(cells)
                    })
                    .collect();

                // Create table state for highlighting
                let mut table_state = TableState::default();
                if cursor < visible_data.len() {
                    table_state.select(Some(cursor));
                }

                let widths = vec![Constraint::Length(10); 6];

                let selected_abs = offset + cursor;
                let avg_render = if render_times.len() > 10 {
                    let sum: f64 = render_times.iter().skip(render_times.len() - 10).sum();
                    sum / 10.0
                } else {
                    0.0
                };

                let table = Table::new(rows, widths)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title(format!(
                        "JSON Test - Row {}/{} | Avg render: {:.1}ms",
                        selected_abs + 1,
                        data.len(),
                        avg_render
                    )))
                    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">> ");

                f.render_stateful_widget(table, area, &mut table_state);
            })?;

            let render_time = render_start.elapsed();
            render_times.push(render_time.as_secs_f64() * 1000.0);

            if render_time.as_millis() > 16 {
                eprintln!("Slow render: {}ms", render_time.as_millis());
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if !render_times.is_empty() {
        let avg: f64 = render_times.iter().sum::<f64>() / render_times.len() as f64;
        println!("\nJSON version stats:");
        println!("  Average render: {:.2}ms", avg);
        println!("  Total renders: {}", render_times.len());
    }

    Ok(())
}
