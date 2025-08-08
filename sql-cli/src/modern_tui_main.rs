use anyhow::{Context, Result};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use sql_cli::datatable_loaders::{load_csv_to_datatable, load_json_to_datatable};
use sql_cli::modern_tui::ModernTui;
use std::io::{self, stdout};
use std::path::Path;

/// Run the modern TUI with a single data file
pub fn run_modern_tui(api_url: &str, data_file: Option<&str>) -> Result<()> {
    // For now, we'll focus on file-based data since that's what we have working
    // TODO: Integrate with API client later

    let table = if let Some(file_path) = data_file {
        load_data_file(file_path)?
    } else {
        // Create empty table or show help
        return show_usage();
    };

    run_tui_with_table(table)
}

/// Run the modern TUI with multiple data files
pub fn run_modern_tui_multi(api_url: &str, data_files: Vec<&str>) -> Result<()> {
    if data_files.is_empty() {
        return show_usage();
    }

    // For now, just use the first file
    // TODO: Implement multi-file support (tabs, switching, etc.)
    let table = load_data_file(data_files[0])?;

    if data_files.len() > 1 {
        println!(
            "Note: Multi-file support coming soon. Using first file: {}",
            data_files[0]
        );
    }

    run_tui_with_table(table)
}

/// Load a data file into a DataTable
fn load_data_file(file_path: &str) -> Result<sql_cli::datatable::DataTable> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    let table_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();

    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "json" => load_json_to_datatable(path, &table_name)
            .with_context(|| format!("Failed to load JSON file: {}", file_path)),
        "csv" => load_csv_to_datatable(path, &table_name)
            .with_context(|| format!("Failed to load CSV file: {}", file_path)),
        _ => Err(anyhow::anyhow!("Unsupported file type: {}", extension)),
    }
}

/// Run the TUI with a DataTable
fn run_tui_with_table(table: sql_cli::datatable::DataTable) -> Result<()> {
    // Print loading info
    println!(
        "Loading {} with {} rows and {} columns...",
        table.name,
        table.row_count(),
        table.column_count()
    );
    println!("Starting Modern SQL CLI...\n");

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Run the app
    let mut app = ModernTui::new(table);
    let result = app.run(&mut terminal);

    // Cleanup
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;

    result.context("TUI execution failed")
}

/// Show usage information
fn show_usage() -> Result<()> {
    println!("SQL CLI - Modern TUI");
    println!("");
    println!("Usage:");
    println!("  sql-cli <file.json>    Load JSON file");
    println!("  sql-cli <file.csv>     Load CSV file");
    println!("");
    println!("Controls:");
    println!("  Tab                    Switch between Query/Results mode");
    println!("  Esc                    Return to Query mode");
    println!("  Ctrl+Q                 Quit");
    println!("");
    println!("Query Mode:");
    println!("  ↑↓                     Navigate history");
    println!("  Ctrl+R                 Fuzzy search history");
    println!("  Enter                  Execute query (switch to Results)");
    println!("  Ctrl+A/E               Beginning/End of line");
    println!("  Ctrl+W                 Delete word backward");
    println!("  Ctrl+K                 Delete to end of line");
    println!("");
    println!("Results Mode:");
    println!("  ↑↓←→                   Navigate data");
    println!("  f                      Filter");
    println!("  /                      Search");
    println!("  s/S                    Sort ascending/descending");
    println!("  n/N                    Next/previous search match");
    println!("  c                      Clear filters/search");

    Ok(())
}
