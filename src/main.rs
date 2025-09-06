use crossterm::style::Stylize;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Emacs, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, ValidationResult, Validator,
};
use sql_cli::utils::app_paths::AppPaths;
use std::{borrow::Cow, io};

mod completer;
mod table_display;

use completer::SqlCompleter;
use sql_cli::api_client::ApiClient;
use sql_cli::sql::parser::{ParseState, SqlParser};
use table_display::{display_results, export_to_csv};

struct SqlValidator;

impl Validator for SqlValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.trim().is_empty() {
            return ValidationResult::Complete;
        }

        let mut parser = SqlParser::new();
        match parser.parse_partial(line) {
            Ok(state) => match state {
                ParseState::Start => ValidationResult::Incomplete,
                ParseState::AfterSelect => ValidationResult::Incomplete,
                ParseState::AfterFrom => ValidationResult::Incomplete,
                _ => ValidationResult::Complete,
            },
            Err(_) => ValidationResult::Complete,
        }
    }
}

struct SqlPrompt;

impl Prompt for SqlPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("sql> ")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<'_, str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "> ".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                reedline::PromptViMode::Normal => "N> ".into(),
                reedline::PromptViMode::Insert => "I> ".into(),
            },
            PromptEditMode::Custom(str) => format!("{str}> ").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse search: {})",
            prefix, history_search.term
        ))
    }
}

fn print_help() {
    println!("{}", "SQL CLI - Syntax-aware SQL editor".blue().bold());
    println!();
    println!("{}", "Usage:".yellow());
    println!("  sql-cli [OPTIONS] [FILE.csv|FILE.json]");
    println!();
    println!("{}", "Options:".yellow());
    println!(
        "  {}, {}    - Show version and exit",
        "--version".green(),
        "-V".green()
    );
    println!(
        "  {}, {}      - Show this help and exit",
        "--help".green(),
        "-h".green()
    );
    println!(
        "  {}  - Initialize configuration with wizard",
        "--init-config".green()
    );
    println!(
        "  {} - Generate config file with defaults",
        "--generate-config".green()
    );
    println!("  {}      - Use classic CLI mode", "--classic".green());
    println!("  {}       - Use simple TUI mode", "--simple".green());
    println!(
        "  {}         - Launch action system debugger (TUI)",
        "--keys".green()
    );

    println!();
    println!("{}", "Non-Interactive Query Mode:".yellow());
    println!(
        "  {}, {} <query>     - Execute SQL query and output results",
        "-q".green(),
        "--query".green()
    );
    println!(
        "  {}, {} <file>  - Execute SQL from file",
        "-f".green(),
        "--query-file".green()
    );
    println!(
        "  {}, {} <format>   - Output format: csv, json, table, tsv (default: csv)",
        "-o".green(),
        "--output".green()
    );
    println!(
        "  {}, {} <file> - Write output to file",
        "-O".green(),
        "--output-file".green()
    );
    println!(
        "  {}, {} <n>       - Limit output to n rows",
        "-l".green(),
        "--limit".green()
    );
    println!(
        "  {} - Case-insensitive matching",
        "--case-insensitive".green()
    );
    println!(
        "  {}  - Auto-hide empty columns",
        "--auto-hide-empty".green()
    );
    println!(
        "  {}  - Launch action system logger (console)",
        "--keys-simple".green()
    );
    println!();
    println!("{}", "Function Documentation:".yellow());
    println!(
        "  {}         - List all available SQL functions",
        "--list-functions".green()
    );
    println!(
        "  {} <name> - Show help for a specific function",
        "--function-help".green()
    );
    println!(
        "  {}    - Generate markdown documentation for all functions",
        "--generate-docs".green()
    );

    println!();
    println!("{}", "Examples:".yellow());
    println!("  # Interactive TUI mode");
    println!("  sql-cli data.csv");
    println!();
    println!("  # Non-interactive query with CSV output");
    println!("  sql-cli data.csv -q \"SELECT * FROM data WHERE price > 100\"");
    println!();
    println!("  # Query with JSON output limited to 10 rows");
    println!("  sql-cli data.json -q \"SELECT id, name FROM data\" -o json -l 10");
    println!();
    println!("  # Query from file with table output");
    println!("  sql-cli trades.csv -f query.sql -o table");
    println!();
    println!("  # Query with output to file");
    println!("  sql-cli data.csv -q \"SELECT * FROM data\" -O results.csv");

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
    println!(
        "  {} - Export last results to CSV",
        "\\export <filename>".green()
    );
    println!();
    println!("{}", "Supported syntax:".yellow());
    println!("  SELECT column1, column2 FROM trade_deal");
    println!("  SELECT * FROM trade_deal WHERE price > 100");
    println!("  SELECT * FROM trade_deal WHERE platformOrderId.Contains('123')");
    println!("  SELECT * FROM trade_deal ORDER BY tradeDate DESC");
    println!();
}

#[allow(dead_code)]
fn execute_query(client: &ApiClient, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("Executing: {query}").cyan());

    match client.query_trades(query) {
        Ok(response) => {
            display_results(&response.data, &response.query.select);
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("Error: {e}").red());
            Err(e)
        }
    }
}

fn main() -> io::Result<()> {
    // Parse arguments first to handle version/help before logging init
    let args: Vec<String> = std::env::args().collect();

    // Check for version flag
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("sql-cli {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Check for help flag
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    // Check for function documentation flags
    if args.contains(&"--list-functions".to_string()) {
        let registry = sql_cli::sql::functions::FunctionRegistry::new();
        println!("{}", registry.list_functions());
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|arg| arg == "--function-help") {
        if let Some(func_name) = args.get(pos + 1) {
            let registry = sql_cli::sql::functions::FunctionRegistry::new();
            if let Some(help) = registry.generate_function_help(func_name) {
                println!("{help}");
            } else {
                eprintln!("Function '{func_name}' not found");
                eprintln!("\nUse --list-functions to see all available functions");
            }
        } else {
            eprintln!("Error: --function-help requires a function name");
            eprintln!("Usage: sql-cli --function-help <function_name>");
        }
        return Ok(());
    }

    if args.contains(&"--generate-docs".to_string()) {
        let registry = sql_cli::sql::functions::FunctionRegistry::new();
        let docs = registry.generate_markdown_docs();
        let doc_path = "docs/FUNCTION_REFERENCE.md";
        std::fs::write(doc_path, docs)?;
        println!("Generated function reference documentation at: {doc_path}");
        return Ok(());
    }

    // Check for non-interactive query mode
    let query_arg = args
        .iter()
        .position(|arg| arg == "-q" || arg == "--query")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    let query_file_arg = args
        .iter()
        .position(|arg| arg == "-f" || arg == "--query-file")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    let output_format_arg = args
        .iter()
        .position(|arg| arg == "-o" || arg == "--output")
        .and_then(|pos| args.get(pos + 1))
        .map_or_else(|| "csv".to_string(), std::string::ToString::to_string);

    let output_file_arg = args
        .iter()
        .position(|arg| arg == "-O" || arg == "--output-file")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    let query_plan_arg = args
        .iter()
        .any(|arg| arg == "--query-plan" || arg == "--query_plan");

    let limit_arg = args
        .iter()
        .position(|arg| arg == "-l" || arg == "--limit")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<usize>().ok());

    // Initialize unified logging (tracing + dual logging) for both modes
    // Do this before non-interactive mode so we get debug logs
    sql_cli::utils::logging::init_tracing_with_dual_logging();

    // Check if running in non-interactive mode
    let is_non_interactive = query_arg.is_some() || query_file_arg.is_some();

    // Only show log path in interactive mode or if debug logging is enabled
    if !is_non_interactive || std::env::var("RUST_LOG").is_ok() {
        if let Some(dual_logger) = sql_cli::utils::dual_logging::get_dual_logger() {
            if !is_non_interactive {
                eprintln!("📝 Debug logs will be written to:");
                eprintln!("   {}", dual_logger.log_path().display());
                eprintln!("   Tail with: tail -f {}", dual_logger.log_path().display());
                eprintln!();
            }
        }
    }

    // If we have a query, run in non-interactive mode
    if is_non_interactive {
        // Read query from file if specified
        let query_file = query_file_arg.clone();
        let query = if let Some(file) = &query_file {
            std::fs::read_to_string(file)
                .map_err(|e| io::Error::other(format!("Failed to read query file {file}: {e}")))?
        } else {
            query_arg.unwrap()
        };

        // Check if this is a multi-statement script (contains GO separator)
        let is_script = query
            .lines()
            .any(|line| line.trim().eq_ignore_ascii_case("go"));

        // Check if query uses DUAL or has no FROM clause
        let uses_dual_or_no_from = {
            use sql_cli::sql::recursive_parser::Parser;
            if let Ok(statement) = Parser::new(&query).parse() {
                statement
                    .from_table
                    .as_ref()
                    .is_none_or(|t| t.to_uppercase() == "DUAL") // No FROM clause means we can use DUAL
            } else {
                false
            }
        };

        // Find the data file (optional if using DUAL)
        let data_file = args
            .iter()
            .filter(|arg| !arg.starts_with('-'))
            .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
            .cloned();

        if let Some(data_file) = data_file {
            let config = sql_cli::non_interactive::NonInteractiveConfig {
                data_file,
                query,
                output_format: sql_cli::non_interactive::OutputFormat::from_str(&output_format_arg)
                    .map_err(io::Error::other)?,
                output_file: output_file_arg,
                case_insensitive: args.contains(&"--case-insensitive".to_string()),
                auto_hide_empty: args.contains(&"--auto-hide-empty".to_string()),
                limit: limit_arg,
                query_plan: query_plan_arg,
            };

            // Use script executor if GO separator is detected
            if is_script {
                return sql_cli::non_interactive::execute_script(config).map_err(io::Error::other);
            } else {
                return sql_cli::non_interactive::execute_non_interactive(config)
                    .map_err(io::Error::other);
            }
        } else if uses_dual_or_no_from {
            // Use empty string for data_file when using DUAL
            let config = sql_cli::non_interactive::NonInteractiveConfig {
                data_file: String::new(),
                query,
                output_format: sql_cli::non_interactive::OutputFormat::from_str(&output_format_arg)
                    .map_err(io::Error::other)?,
                output_file: output_file_arg,
                case_insensitive: args.contains(&"--case-insensitive".to_string()),
                auto_hide_empty: args.contains(&"--auto-hide-empty".to_string()),
                limit: limit_arg,
                query_plan: query_plan_arg,
            };

            // Use script executor if GO separator is detected
            if is_script {
                return sql_cli::non_interactive::execute_script(config).map_err(io::Error::other);
            } else {
                return sql_cli::non_interactive::execute_non_interactive(config)
                    .map_err(io::Error::other);
            }
        }
        eprintln!("Error: Data file (CSV or JSON) required for non-interactive query mode");
        eprintln!("Usage: sql-cli <data.csv> -q \"SELECT * FROM table\"");
        eprintln!("       sql-cli -q \"SELECT expression FROM DUAL\"");
        std::process::exit(1);
    }

    // Check for config initialization
    if args.contains(&"--init-config".to_string()) {
        match sql_cli::config::config::Config::init_wizard() {
            Ok(config) => {
                println!("\nConfiguration initialized successfully!");
                if !config.display.use_glyphs {
                    println!("Note: Simple mode enabled (ASCII icons)");
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error initializing config: {e}");
                std::process::exit(1);
            }
        }
    }

    // Check for action debugger mode
    if args.contains(&"--keys".to_string()) || args.contains(&"--keys-simple".to_string()) {
        let use_simple = args.contains(&"--keys-simple".to_string());

        if use_simple {
            println!("Launching Action System Logger (Simple Version)...");
            println!("This tool shows how keys map to actions in real-time.\n");
        } else {
            println!("Launching Action System Debugger...");
            println!("This interactive TUI shows key mappings, history, and state.\n");
        }

        // Import what we need for the debugger
        use std::process::Command;

        // Choose which binary to run
        let binary_name = if use_simple {
            "action_logger"
        } else {
            "action_debugger"
        };

        // Run the selected binary
        let status =
            Command::new(std::env::current_exe()?.parent().unwrap().join(binary_name)).status();

        match status {
            Ok(exit_status) if exit_status.success() => return Ok(()),
            Ok(_) => {
                eprintln!("{binary_name} exited with error");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to launch {binary_name}: {e}");
                eprintln!("Make sure it's built with: cargo build --bin {binary_name}");
                std::process::exit(1);
            }
        }
    }

    // Check for config file generation
    if args.contains(&"--generate-config".to_string()) {
        match sql_cli::config::config::Config::get_config_path() {
            Ok(path) => {
                let config_content =
                    sql_cli::config::config::Config::create_default_with_comments();
                if let Some(parent) = path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("Error creating config directory: {e}");
                        std::process::exit(1);
                    }
                }
                if let Err(e) = std::fs::write(&path, config_content) {
                    eprintln!("Error writing config file: {e}");
                    std::process::exit(1);
                }
                println!("Configuration file created at: {path:?}");
                println!("Edit this file to customize your SQL CLI experience.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error determining config path: {e}");
                std::process::exit(1);
            }
        }
    }

    let use_classic_tui = args.contains(&"--simple".to_string());
    let use_tui = !args.contains(&"--classic".to_string());

    // Check for data file argument (CSV or JSON)
    // First check for --csv flag (legacy)
    let csv_file_flag = args
        .iter()
        .position(|arg| arg == "--csv")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    // If no --csv flag, check if last argument is a file
    // Collect all data files (CSV/JSON) from arguments
    let data_files: Vec<String> = args
        .iter()
        .filter(|arg| !arg.starts_with("--"))
        .filter(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
        .cloned()
        .collect();

    // For backward compatibility, get the first file as data_file
    let data_file = csv_file_flag.or_else(|| data_files.first().cloned());

    if use_tui {
        if use_classic_tui {
            println!("Starting simple TUI mode... (use --enhanced for csvlens-style features)");
            if let Err(e) = sql_cli::ui::tui_app::run_tui_app() {
                eprintln!("TUI Error: {e}");
                std::process::exit(1);
            }
        } else {
            if let Some(file_path) = &data_file {
                let file_type = if file_path.ends_with(".json") {
                    "JSON"
                } else {
                    "CSV"
                };
                println!("Starting enhanced TUI in {file_type} mode with file: {file_path}");
            } else {
                println!(
                    "Starting enhanced TUI mode... (use --simple for basic TUI, --classic for CLI)"
                );
            }
            let api_url = std::env::var("TRADE_API_URL")
                .unwrap_or_else(|_| "http://localhost:5000".to_string());

            // Use the enhanced TUI by default
            let result = if data_files.len() > 1 {
                let file_refs: Vec<&str> =
                    data_files.iter().map(std::string::String::as_str).collect();
                sql_cli::ui::enhanced_tui::run_enhanced_tui_multi(&api_url, file_refs)
            } else {
                sql_cli::ui::enhanced_tui::run_enhanced_tui(&api_url, data_file.as_deref())
            };

            if let Err(e) = result {
                eprintln!("Enhanced TUI Error: {e}");
                eprintln!("Falling back to classic CLI mode...");
                eprintln!();
                // Don't exit, fall through to classic mode
            } else {
                return Ok(());
            }
        }
        return Ok(());
    }

    // Classic mode (original interface)
    print_help();

    let history_file = AppPaths::history_file()
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".sql_cli_history"));
    let history = Box::new(
        FileBackedHistory::with_file(50, history_file).expect("Error configuring history"),
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
    let api_url =
        std::env::var("TRADE_API_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());
    let api_client = ApiClient::new(&api_url);

    println!("{}", format!("Connected to API: {api_url}").cyan());

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
                        match export_to_csv(results, &["*".to_string()], parts[1]) {
                            Ok(()) => {}
                            Err(e) => eprintln!("{}", format!("Export error: {e}").red()),
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
                    Err(e) => eprintln!("{}", format!("Error: {e}").red()),
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
