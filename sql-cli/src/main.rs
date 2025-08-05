use crossterm::style::Stylize;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Emacs, FileBackedHistory, KeyCode,
    KeyModifiers, MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch,
    PromptHistorySearchStatus, Reedline, ReedlineEvent, ReedlineMenu, Signal, ValidationResult,
    Validator,
};
use std::{borrow::Cow, io};

mod parser;
mod completer;
mod api_client;
mod table_display;
mod tui_app;
mod enhanced_tui;
mod smart_parser;
mod cursor_aware_parser;
mod recursive_parser;
mod hybrid_parser;
mod history;
mod csv_fixes;
mod sql_highlighter;
mod schema_config;

use completer::SqlCompleter;
use parser::{SqlParser, ParseState};
use api_client::ApiClient;
use table_display::{display_results, export_to_csv};

struct SqlValidator;

impl Validator for SqlValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.trim().is_empty() {
            return ValidationResult::Complete;
        }
        
        let mut parser = SqlParser::new();
        match parser.parse_partial(line) {
            Ok(state) => {
                match state {
                    ParseState::Start => ValidationResult::Incomplete,
                    ParseState::AfterSelect => ValidationResult::Incomplete,
                    ParseState::AfterFrom => ValidationResult::Incomplete,
                    _ => ValidationResult::Complete,
                }
            }
            Err(_) => ValidationResult::Complete,
        }
    }
}

struct SqlPrompt;

impl Prompt for SqlPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed("sql> ")
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "> ".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                reedline::PromptViMode::Normal => "N> ".into(),
                reedline::PromptViMode::Insert => "I> ".into(),
            },
            PromptEditMode::Custom(str) => format!("{str}> ").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!("({}reverse search: {})", prefix, history_search.term))
    }
}

fn print_help() {
    println!("{}", "SQL CLI - Syntax-aware SQL editor".blue().bold());
    println!();
    println!("{}", "Commands:".yellow());
    println!("  {}  - Execute query and fetch results", "Enter".green());
    println!("  {}    - Syntax-aware completion", "Tab".green());
    println!("  {} - Previous command", "Ctrl+P".green());
    println!("  {} - Next command", "Ctrl+N".green());
    println!("  {} - Search history", "Ctrl+R".green());
    println!("  {} - Exit", "Ctrl+D".green());
    println!("  {}  - Show this help", "\\help".green());
    println!("  {} - Clear screen", "\\clear".green());
    println!("  {} - Export last results to CSV", "\\export <filename>".green());
    println!();
    println!("{}", "Supported syntax:".yellow());
    println!("  SELECT column1, column2 FROM trade_deal");
    println!("  SELECT * FROM trade_deal WHERE price > 100");
    println!("  SELECT * FROM trade_deal WHERE platformOrderId.Contains('123')");
    println!("  SELECT * FROM trade_deal ORDER BY tradeDate DESC");
    println!();
}

fn execute_query(client: &ApiClient, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("Executing: {}", query).cyan());
    
    match client.query_trades(query) {
        Ok(response) => {
            display_results(&response.data, &response.query.select);
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("Error: {}", e).red());
            Err(e)
        }
    }
}

fn main() -> io::Result<()> {
    // Check if user wants TUI mode (default) or classic mode
    let args: Vec<String> = std::env::args().collect();
    let use_classic_tui = args.contains(&"--simple".to_string());
    let use_tui = !args.contains(&"--classic".to_string());
    
    // Check for data file argument (CSV or JSON)
    // First check for --csv flag (legacy)
    let csv_file_flag = args.iter()
        .position(|arg| arg == "--csv")
        .and_then(|pos| args.get(pos + 1))
        .map(|s| s.to_string());
    
    // If no --csv flag, check if last argument is a file
    let data_file = csv_file_flag.or_else(|| {
        args.last()
            .filter(|arg| !arg.starts_with("--"))
            .filter(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
            .map(|s| s.to_string())
    });
    
    if use_tui {
        if use_classic_tui {
            println!("Starting simple TUI mode... (use --enhanced for csvlens-style features)");
            if let Err(e) = tui_app::run_tui_app() {
                eprintln!("TUI Error: {}", e);
                std::process::exit(1);
            }
        } else {
            if let Some(file_path) = &data_file {
                let file_type = if file_path.ends_with(".json") { "JSON" } else { "CSV" };
                println!("Starting enhanced TUI in {} mode with file: {}", file_type, file_path);
            } else {
                println!("Starting enhanced TUI mode... (use --simple for basic TUI, --classic for CLI)");
            }
            let api_url = std::env::var("TRADE_API_URL")
                .unwrap_or_else(|_| "http://localhost:5000".to_string());
            if let Err(e) = enhanced_tui::run_enhanced_tui(&api_url, data_file.as_deref()) {
                eprintln!("Enhanced TUI Error: {}", e);
                eprintln!("Falling back to classic CLI mode...");
                eprintln!("");
                // Don't exit, fall through to classic mode
            } else {
                return Ok(());
            }
        }
        return Ok(());
    }
    
    // Classic mode (original interface)
    print_help();
    
    let history_file = dirs::home_dir()
        .unwrap()
        .join(".sql_cli_history");
    let history = Box::new(
        FileBackedHistory::with_file(50, history_file.to_path_buf())
            .expect("Error configuring history"),
    );
    
    let completer = Box::new(SqlCompleter::new());
    
    let completion_menu = Box::new(
        ColumnarMenu::default()
            .with_name("sql_completion")
            .with_columns(1)
            .with_column_width(None)
            .with_column_padding(2),
    );
    
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Menu("sql_completion".to_string()),
    );
    
    let edit_mode = Box::new(Emacs::new(keybindings));
    
    let mut line_editor = Reedline::create()
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_validator(Box::new(SqlValidator))
        .with_history(history)
        .with_edit_mode(edit_mode);
    
    let prompt = SqlPrompt;
    
    // Initialize API client
    let api_url = std::env::var("TRADE_API_URL")
        .unwrap_or_else(|_| "http://localhost:5000".to_string());
    let api_client = ApiClient::new(&api_url);
    
    println!("{}", format!("Connected to API: {}", api_url).cyan());
    
    let mut last_results: Option<Vec<serde_json::Value>> = None;
    
    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(buffer) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                if trimmed == "\\help" {
                    print_help();
                    continue;
                }
                
                if trimmed == "\\clear" {
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    continue;
                }
                
                if trimmed.starts_with("\\export") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() < 2 {
                        eprintln!("{}", "Usage: \\export <filename>".red());
                        continue;
                    }
                    
                    if let Some(ref results) = last_results {
                        match export_to_csv(results, &vec!["*".to_string()], parts[1]) {
                            Ok(_) => {},
                            Err(e) => eprintln!("{}", format!("Export error: {}", e).red()),
                        }
                    } else {
                        eprintln!("{}", "No results to export. Run a query first.".red());
                    }
                    continue;
                }
                
                match api_client.query_trades(&buffer) {
                    Ok(response) => {
                        display_results(&response.data, &response.query.select);
                        last_results = Some(response.data);
                    }
                    Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
                }
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nGoodbye!");
                break;
            }
        }
    }
    
    Ok(())
}
