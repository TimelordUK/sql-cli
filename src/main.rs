use crossterm::style::Stylize;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Emacs, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, ValidationResult, Validator,
};
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::DataValue;
use sql_cli::non_interactive::{OutputFormat, TableStyle};
use sql_cli::utils::app_paths::AppPaths;
use sql_cli::utils::string_utils::display_width;
use std::io::Write;
use std::{borrow::Cow, io};

mod completer;
mod main_handlers;
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
            ParseState::Start => ValidationResult::Incomplete,
            ParseState::AfterSelect => ValidationResult::Incomplete,
            ParseState::AfterFrom => ValidationResult::Incomplete,
            _ => ValidationResult::Complete,
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
    println!(
        "  {}   - Purge all cache entries (requires SQL_CLI_CACHE=true)",
        "--cache-purge".green()
    );

    println!();
    println!("{}", "SQL Refactoring Tools:".yellow());
    println!(
        "  {} - Generate banding CASE statement",
        "--generate-bands".green()
    );
    println!("    Usage: --generate-bands --column <name> --bands <spec>");
    println!("    Example: --generate-bands --column age --bands \"0-24,25-49,50-74,75+\"");
    println!(
        "  {} - Generate CASE from data analysis",
        "--generate-case".green()
    );
    println!(
        "    Usage: --generate-case <file> --column <name> [--style values|ranges] [--labels label1,label2,...]"
    );
    println!("    Example: --generate-case data.csv --column ocean_proximity --style values");
    println!(
        "  {} - Generate CASE for numeric range",
        "--generate-case-range".green()
    );
    println!(
        "    Usage: --generate-case-range --column <name> --min <n> --max <n> --bands <n> [--labels label1,label2,...]"
    );
    println!("    Example: --generate-case-range --column value --min 0 --max 100 --bands 5");

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
        "  {} <style> - Table style: markdown, utf8, ascii, etc. (default: default)",
        "--table-style".green()
    );
    println!(
        "  {} - List all available table styles",
        "--list-table-styles".green()
    );
    println!(
        "  {} <col> - Show distinct values with counts for column",
        "--distinct-column".green()
    );
    println!(
        "  {}, {} <n>       - Limit output to n rows",
        "-l".green(),
        "--limit".green()
    );
    println!(
        "  {} <n>  - Maximum column width for table output (default: 50, 0 = unlimited)",
        "--max-col-width".green()
    );
    println!(
        "  {} <n> - Rows to sample for column width (default: 100, 0 = all rows)",
        "--col-sample-rows".green()
    );
    println!(
        "  {}        - Apply color styling rules to table output",
        "--styled".green()
    );
    println!(
        "  {} <file> - Path to YAML style configuration file",
        "--style-file".green()
    );
    println!(
        "  {} - Case-insensitive matching",
        "--case-insensitive".green()
    );
    println!(
        "  {}, {} - Enable debug tracing for query execution",
        "--debug".green(),
        "--debug-trace".green()
    );
    println!(
        "  {}  - Auto-hide empty columns",
        "--auto-hide-empty".green()
    );
    println!(
        "  {}         - Show SQL query AST (parse tree)",
        "--query-plan".green()
    );
    println!(
        "  {}    - Show query execution work units",
        "--show-work-units".green()
    );
    println!(
        "  {}    - Show detailed execution plan with timings",
        "--execution-plan".green()
    );
    println!(
        "  {} - Show AST preprocessing transformations",
        "--show-preprocessing".green()
    );
    println!(
        "  {} - Show SQL before/after each transformation",
        "--show-transformations".green()
    );
    println!(
        "  {}  - Launch action system logger (console)",
        "--keys-simple".green()
    );

    println!();
    println!(
        "{}",
        "Query Analysis (for IDE/plugin integration):".yellow()
    );
    println!(
        "  {}     - Analyze query structure (JSON output)",
        "--analyze-query".green()
    );
    println!(
        "  {}        - Expand SELECT * to column names (JSON output)",
        "--expand-star".green()
    );
    println!(
        "  {} <name> - Extract CTE as standalone query",
        "--extract-cte".green()
    );
    println!(
        "  {} <line:col> - Find query context at position (JSON output)",
        "--query-at-position".green()
    );

    println!();
    println!(
        "{}",
        "Script Execution (for IDE/plugin integration):".yellow()
    );
    println!(
        "  {} <n> - Execute statement N from script with dependencies",
        "--execute-statement".green()
    );
    println!("    Example: sql-cli -f script.sql --execute-statement 3");
    println!("    Only executes statement #3 and its dependencies (temp tables, CTEs)");
    println!(
        "  {} <line> - Get available columns at specific line",
        "--get-columns-at".green()
    );
    println!("    Example: sql-cli -f script.sql --get-columns-at 25");
    println!("    Returns CSV of columns available at that line (for \\sX expansion)");
    println!(
        "  {}       - Show dependency analysis without execution",
        "--dry-run".green()
    );

    println!();
    println!("{}", "SQL Formatting:".yellow());
    println!(
        "  {}, {} [file|-]   - Format SQL query (stdin if - or no file)",
        "-F".green(),
        "--format".green()
    );
    println!(
        "  {}       - Format SQL from file",
        "--format-sql <file>".green()
    );
    println!(
        "  {}  - Preserve comments when formatting (use with -F/--format)",
        "--preserve-comments".green()
    );

    println!();
    println!("{}", "Data Inspection:".yellow());
    println!(
        "  {}               - Show table schema (columns and types)",
        "--schema".green()
    );
    println!(
        "  {}          - Show table schema as JSON (nvim plugin)",
        "--schema-json".green()
    );

    println!();
    println!("{}", "Documentation & Help:".yellow());
    println!(
        "  {} <name>         - Show help for any function, aggregate, or generator",
        "--item-help".green()
    );
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
    println!(
        "  {}       - List all available generator functions",
        "--list-generators".green()
    );
    println!(
        "  {} <name> - Show help for a specific generator",
        "--generator-help".green()
    );

    println!();
    println!("{}", "Performance Benchmarking:".yellow());
    println!(
        "  {}            - Run performance benchmarks",
        "--benchmark".green()
    );
    println!(
        "  {}  - Benchmark sizes (default: 100,1000,10000,50000,100000)",
        "--sizes <n1,n2,n3>".green()
    );
    println!(
        "  {} <cat>  - Run specific category (basic|aggregation|sorting|window|complex)",
        "--category".green()
    );
    println!(
        "  {}        - Run progressive benchmarks (10k increments)",
        "--progressive".green()
    );
    println!(
        "  {}  - Set increment for progressive (default: 10000)",
        "--increment <n>".green()
    );
    println!(
        "  {}  - Set max rows for progressive (default: 100000)",
        "--max-rows <n>".green()
    );
    println!("  {}  - Save results as CSV", "--csv <output.csv>".green());
    println!(
        "  {}  - Generate markdown report",
        "--report <file.md>".green()
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
    println!("  # Debug query with execution plan");
    println!("  sql-cli data.csv -q \"SELECT * FROM data WHERE id > 100\" --execution-plan");

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

/// Parsed command-line arguments for non-interactive mode
struct NonInteractiveArgs {
    query_arg: Option<String>,
    query_file_arg: Option<String>,
    output_format_arg: String,
    output_file_arg: Option<String>,
    table_style_arg: String,
    query_plan_arg: bool,
    show_work_units_arg: bool,
    execution_plan_arg: bool,
    show_preprocessing_arg: bool,
    show_transformations_arg: bool,
    analyze_correlations_arg: bool,
    cte_info_arg: bool,
    rewrite_analysis_arg: bool,
    lift_in_arg: bool,
    debug_arg: bool,
    no_where_expansion_arg: bool,
    no_group_by_expansion_arg: bool,
    no_having_expansion_arg: bool,
    no_order_by_expansion_arg: bool,
    no_expression_lifter_arg: bool,
    no_cte_hoister_arg: bool,
    no_in_lifter_arg: bool,
    limit_arg: Option<usize>,
    analyze_query_arg: bool,
    expand_star_arg: bool,
    extract_cte_arg: Option<String>,
    query_at_position_arg: Option<String>,
    execute_statement_arg: Option<usize>,
    get_columns_at_arg: Option<usize>,
    dry_run_arg: bool,
    styled_arg: bool,
    style_file_arg: Option<String>,
}

/// Parse command-line arguments into a structured context object
fn parse_non_interactive_args(args: &[String]) -> NonInteractiveArgs {
    NonInteractiveArgs {
        query_arg: args
            .iter()
            .position(|arg| arg == "-q" || arg == "--query")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),

        query_file_arg: args
            .iter()
            .position(|arg| arg == "-f" || arg == "--query-file")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),

        output_format_arg: args
            .iter()
            .position(|arg| arg == "-o" || arg == "--output")
            .and_then(|pos| args.get(pos + 1))
            .map_or_else(|| "csv".to_string(), std::string::ToString::to_string),

        output_file_arg: args
            .iter()
            .position(|arg| arg == "-O" || arg == "--output-file")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),

        table_style_arg: args
            .iter()
            .position(|arg| arg == "--table-style")
            .and_then(|pos| args.get(pos + 1))
            .map_or_else(|| "default".to_string(), std::string::ToString::to_string),

        query_plan_arg: args
            .iter()
            .any(|arg| arg == "--query-plan" || arg == "--query_plan"),

        show_work_units_arg: args
            .iter()
            .any(|arg| arg == "--show-work-units" || arg == "--show_work_units"),

        lift_in_arg: args
            .iter()
            .any(|arg| arg == "--check-in-lifting" || arg == "--check_in_lifting"),

        analyze_query_arg: args
            .iter()
            .any(|arg| arg == "--analyze-query" || arg == "--analyze_query"),

        expand_star_arg: args
            .iter()
            .any(|arg| arg == "--expand-star" || arg == "--expand_star"),

        extract_cte_arg: args
            .iter()
            .position(|arg| arg == "--extract-cte" || arg == "--extract_cte")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),

        query_at_position_arg: args
            .iter()
            .position(|arg| arg == "--query-at-position" || arg == "--query_at_position")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),

        execution_plan_arg: args
            .iter()
            .any(|arg| arg == "--execution-plan" || arg == "--execution_plan"),

        show_preprocessing_arg: args
            .iter()
            .any(|arg| arg == "--show-preprocessing" || arg == "--show_preprocessing"),

        show_transformations_arg: args
            .iter()
            .any(|arg| arg == "--show-transformations" || arg == "--show_transformations"),

        analyze_correlations_arg: args
            .iter()
            .any(|arg| arg == "--analyze-correlations" || arg == "--analyze_correlations"),

        cte_info_arg: args
            .iter()
            .any(|arg| arg == "--cte-info" || arg == "--cte-json"),

        rewrite_analysis_arg: args
            .iter()
            .any(|arg| arg == "--analyze-rewrite" || arg == "--rewrite-analysis"),

        debug_arg: args
            .iter()
            .any(|arg| arg == "--debug" || arg == "--debug-trace"),

        no_where_expansion_arg: args.iter().any(|arg| arg == "--no-where-expansion"),

        no_group_by_expansion_arg: args.iter().any(|arg| arg == "--no-group-by-expansion"),

        no_having_expansion_arg: args.iter().any(|arg| arg == "--no-having-expansion"),

        no_order_by_expansion_arg: args.iter().any(|arg| arg == "--no-order-by-expansion"),

        no_expression_lifter_arg: args.iter().any(|arg| arg == "--no-expression-lifter"),

        no_cte_hoister_arg: args.iter().any(|arg| arg == "--no-cte-hoister"),

        no_in_lifter_arg: args.iter().any(|arg| arg == "--no-in-lifter"),

        limit_arg: args
            .iter()
            .position(|arg| arg == "-l" || arg == "--limit")
            .and_then(|pos| args.get(pos + 1))
            .and_then(|s| s.parse::<usize>().ok()),

        execute_statement_arg: args
            .iter()
            .position(|arg| arg == "--execute-statement")
            .and_then(|pos| args.get(pos + 1))
            .and_then(|s| s.parse::<usize>().ok()),

        get_columns_at_arg: args
            .iter()
            .position(|arg| arg == "--get-columns-at")
            .and_then(|pos| args.get(pos + 1))
            .and_then(|s| s.parse::<usize>().ok()),

        dry_run_arg: args.iter().any(|arg| arg == "--dry-run"),

        styled_arg: args.iter().any(|arg| arg == "--styled"),

        style_file_arg: args
            .iter()
            .position(|arg| arg == "--style-file")
            .and_then(|pos| args.get(pos + 1))
            .map(std::string::ToString::to_string),
    }
}

/// Handle action debugger mode (--keys and --keys-simple)
/// Launches either the action_debugger or action_logger binary
fn handle_key_debugger_mode(args: &[String]) -> Option<io::Result<()>> {
    if !args.contains(&"--keys".to_string()) && !args.contains(&"--keys-simple".to_string()) {
        return None;
    }

    let use_simple = args.contains(&"--keys-simple".to_string());

    if use_simple {
        println!("Launching Action System Logger (Simple Version)...");
        println!("This tool shows how keys map to actions in real-time.\n");
    } else {
        println!("Launching Action System Debugger...");
        println!("This interactive TUI shows key mappings, history, and state.\n");
    }

    use std::process::Command;

    let binary_name = if use_simple {
        "action_logger"
    } else {
        "action_debugger"
    };

    let status = Command::new(
        std::env::current_exe()
            .ok()?
            .parent()
            .unwrap()
            .join(binary_name),
    )
    .status();

    match status {
        Ok(exit_status) if exit_status.success() => Some(Ok(())),
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

/// Handle config file generation (--generate-config)
/// Creates a default configuration file with comments
fn handle_config_generation(args: &[String]) -> Option<io::Result<()>> {
    if !args.contains(&"--generate-config".to_string()) {
        return None;
    }

    match sql_cli::config::config::Config::get_config_path() {
        Ok(path) => {
            let config_content = sql_cli::config::config::Config::create_default_with_comments();
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
            Some(Ok(()))
        }
        Err(e) => {
            eprintln!("Error determining config path: {e}");
            std::process::exit(1);
        }
    }
}

/// Handle enhanced TUI mode launch
/// Launches the enhanced TUI with single or multiple data files
fn launch_enhanced_tui(data_file: Option<String>, data_files: Vec<String>) -> io::Result<()> {
    if let Some(file_path) = &data_file {
        let file_type = if file_path.ends_with(".json") {
            "JSON"
        } else {
            "CSV"
        };
        println!("Starting enhanced TUI in {file_type} mode with file: {file_path}");
    } else {
        println!("Starting enhanced TUI mode... (use --simple for basic TUI, --classic for CLI)");
    }

    let api_url =
        std::env::var("TRADE_API_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());

    // Use the enhanced TUI with either single or multiple files
    let result = if data_files.len() > 1 {
        let file_refs: Vec<&str> = data_files.iter().map(std::string::String::as_str).collect();
        sql_cli::ui::enhanced_tui::run_enhanced_tui_multi(&api_url, file_refs)
    } else {
        sql_cli::ui::enhanced_tui::run_enhanced_tui(&api_url, data_file.as_deref())
    };

    if let Err(e) = result {
        // Ensure terminal is restored in case of error
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
            crossterm::cursor::Show
        );

        eprintln!("Enhanced TUI Error: {e}");
        eprintln!("Falling back to classic CLI mode...");
        eprintln!();
        // Return error so caller can handle fallback - convert anyhow::Error to io::Error
        return Err(io::Error::other(e));
    }

    Ok(())
}

/// Run classic console mode with Reedline-based SQL editor
/// This is the original CLI interface with history, completion, and validation
fn run_classic_console_mode() -> io::Result<()> {
    // Print help first
    print_help();

    // Set up history
    let history_file = AppPaths::history_file()
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".sql_cli_history"));
    let history = Box::new(
        FileBackedHistory::with_file(50, history_file).expect("Error configuring history"),
    );

    // Set up SQL completion
    let completer = Box::new(SqlCompleter::new());
    let completion_menu = Box::new(
        ColumnarMenu::default()
            .with_name("sql_completion")
            .with_columns(1)
            .with_column_width(None)
            .with_column_padding(2),
    );

    // Configure keybindings
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Menu("sql_completion".to_string()),
    );
    let edit_mode = Box::new(Emacs::new(keybindings));

    // Create line editor
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

    // Main REPL loop
    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(buffer) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Handle special commands
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

                // Execute query
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

/// Handle --execute-statement flag with dependency-aware execution
/// Analyzes dependencies and executes only the minimal subset of statements needed
fn handle_execute_statement(
    script: &str,
    data_file: &str,
    target_statement: usize,
    dry_run: bool,
    parsed_args: &NonInteractiveArgs,
) -> io::Result<()> {
    use sql_cli::analysis::statement_dependencies::DependencyAnalyzer;
    use sql_cli::sql::recursive_parser::Parser;
    use sql_cli::sql::script_parser::ScriptParser;

    // Parse the script into statements
    let parser = ScriptParser::new(script);
    let statements = parser.parse_statements();

    if statements.is_empty() {
        eprintln!("Error: No statements found in script");
        std::process::exit(1);
    }

    // Analyze dependencies
    let analyzed = DependencyAnalyzer::analyze_statements(&statements)
        .map_err(|e| io::Error::other(format!("Dependency analysis failed: {}", e)))?;

    // Compute execution plan
    let plan = DependencyAnalyzer::compute_execution_plan(&analyzed, target_statement)
        .map_err(|e| io::Error::other(format!("Failed to compute execution plan: {}", e)))?;

    // If dry-run, just show the plan and exit
    if dry_run {
        println!("{}", plan.format_debug_trace(&analyzed));
        return Ok(());
    }

    // Execute the plan
    println!(
        "Executing statement #{} with {} dependencies...",
        target_statement,
        plan.statements_to_execute.len() - 1
    );
    println!("Execution order: {:?}", plan.statements_to_execute);
    println!();

    // Use the non-interactive executor to run the minimal set of statements
    use sql_cli::data::datatable_loaders::load_csv_to_datatable;
    use sql_cli::data::query_engine::QueryEngine;
    use sql_cli::data::temp_table_registry::TempTableRegistry;
    use std::sync::Arc;

    // Load the data source if provided, otherwise use DUAL table
    let data_table = if !data_file.is_empty() {
        load_csv_to_datatable(
            std::path::Path::new(data_file),
            &format!("data_{}", target_statement),
        )
        .map_err(|e| io::Error::other(format!("Failed to load data file: {}", e)))?
    } else {
        // No data file provided, use DUAL table (allows WEB CTEs and queries without data files)
        use sql_cli::data::datatable::DataTable;
        DataTable::dual()
    };

    let table_arc = Arc::new(data_table);
    let engine = QueryEngine::new();
    let mut temp_tables = TempTableRegistry::new();

    // Execute statements in order from the plan
    let mut last_result = None;
    for stmt_num in &plan.statements_to_execute {
        let stmt_sql = &statements[stmt_num - 1]; // Convert to 0-based index

        println!("Executing statement #{}...", stmt_num);

        // Parse the SQL statement into an AST
        let mut stmt_parser = Parser::new(stmt_sql);
        let parsed_stmt = stmt_parser.parse().map_err(|e| {
            io::Error::other(format!("Failed to parse statement #{}: {}", stmt_num, e))
        })?;

        // Check if this statement has an INTO clause (creates a temp table)
        let into_table_name = parsed_stmt.into_table.as_ref().map(|it| it.name.clone());

        match engine.execute_statement_with_temp_tables(
            table_arc.clone(),
            parsed_stmt,
            Some(&temp_tables),
        ) {
            Ok(result) => {
                println!(
                    "  ✓ Statement #{} completed ({} rows)",
                    stmt_num,
                    result.row_count()
                );

                // If this statement creates a temp table, register it
                if let Some(temp_name) = into_table_name {
                    // Materialize the result into a DataTable
                    let temp_table = engine.materialize_view(result.clone()).map_err(|e| {
                        io::Error::other(format!("Failed to materialize temp table: {}", e))
                    })?;
                    temp_tables.insert(temp_name.clone(), Arc::new(temp_table));
                    println!("  → Temp table {} created", temp_name);
                }

                if *stmt_num == target_statement {
                    last_result = Some(result);
                }
            }
            Err(e) => {
                eprintln!("  ✗ Statement #{} failed: {}", stmt_num, e);
                std::process::exit(1);
            }
        }
    }

    // Output the results from the target statement
    if let Some(result_view) = last_result {
        println!();
        println!("=== Results from statement #{} ===", target_statement);
        println!();

        // Use the configured output format - build a NonInteractiveConfig to reuse execute_non_interactive logic
        // But we'll inline the output logic here since we already have the results

        let output_format =
            sql_cli::non_interactive::OutputFormat::from_str(&parsed_args.output_format_arg)
                .map_err(io::Error::other)?;

        let table_style =
            sql_cli::non_interactive::TableStyle::from_str(&parsed_args.table_style_arg)
                .map_err(io::Error::other)?;

        // Output to file or stdout
        if let Some(ref output_file) = parsed_args.output_file_arg {
            let mut file = std::fs::File::create(output_file)
                .map_err(|e| io::Error::other(format!("Failed to create output file: {}", e)))?;
            output_dataview(
                &result_view,
                output_format,
                &mut file,
                None,
                100,
                0.0,
                table_style,
            )
            .map_err(|e| io::Error::other(format!("Output failed: {}", e)))?;
        } else {
            let mut stdout = io::stdout();
            output_dataview(
                &result_view,
                output_format,
                &mut stdout,
                None,
                100,
                0.0,
                table_style,
            )
            .map_err(|e| io::Error::other(format!("Output failed: {}", e)))?;
        }
    }

    Ok(())
}

/// Handle --get-columns-at flag for IDE/plugin integration
/// Returns available columns at a specific line in the script
fn handle_get_columns_at(script: &str, data_file: &str, target_line: usize) -> io::Result<()> {
    use sql_cli::analysis::statement_dependencies::DependencyAnalyzer;
    use sql_cli::sql::recursive_parser::Parser;
    use sql_cli::sql::script_parser::ScriptParser;

    // Parse the script into statements
    let parser = ScriptParser::new(script);
    let statements = parser.parse_statements();

    if statements.is_empty() {
        eprintln!("Error: No statements found in script");
        std::process::exit(1);
    }

    // Find which statement contains the target line by scanning the original script
    // We need to map line numbers in the original file to statement indices
    let script_lines: Vec<&str> = script.lines().collect();
    let mut target_statement = 0;

    // Build a map of line ranges to statement indices
    // by scanning for GO separators in the original script
    let mut statement_ranges: Vec<(usize, usize)> = Vec::new(); // (start_line, end_line) for each statement
    let mut stmt_start_line = 1; // 1-based line numbering
    let mut in_statement = false;
    let mut last_non_empty_line = 1;

    for (line_idx, line) in script_lines.iter().enumerate() {
        let line_num = line_idx + 1; // Convert to 1-based
        let trimmed = line.trim();

        // Check if this is a GO separator
        if trimmed.eq_ignore_ascii_case("go") {
            // End current statement (if we have one)
            if in_statement {
                statement_ranges.push((stmt_start_line, last_non_empty_line));
                in_statement = false;
            }
            // Next statement starts after the GO
            stmt_start_line = line_num + 1;
        } else if !trimmed.is_empty() {
            // Non-empty, non-GO line - we're in a statement
            if !in_statement {
                in_statement = true;
                stmt_start_line = line_num;
            }
            last_non_empty_line = line_num;
        }
    }

    // Don't forget the last statement if we're still in one
    if in_statement {
        statement_ranges.push((stmt_start_line, script_lines.len()));
    }

    // Validate we found the same number of statements
    if statement_ranges.len() != statements.len() {
        eprintln!(
            "Warning: Found {} statement ranges but ScriptParser returned {} statements",
            statement_ranges.len(),
            statements.len()
        );
    }

    // Find which statement contains the target line
    for (idx, (start, end)) in statement_ranges.iter().enumerate() {
        if target_line >= *start && target_line <= *end {
            target_statement = idx + 1; // 1-based statement numbering
            break;
        }
    }

    if target_statement == 0 {
        eprintln!("Error: Line {} not found in script", target_line);
        eprintln!("Script has {} lines total", script_lines.len());
        eprintln!("Found {} statements with ranges:", statement_ranges.len());
        for (idx, (start, end)) in statement_ranges.iter().enumerate() {
            eprintln!("  Statement {}: lines {}-{}", idx + 1, start, end);
        }
        std::process::exit(1);
    }

    // Analyze dependencies
    let analyzed = DependencyAnalyzer::analyze_statements(&statements)
        .map_err(|e| io::Error::other(format!("Dependency analysis failed: {}", e)))?;

    // Compute execution plan for the target statement
    let plan = DependencyAnalyzer::compute_execution_plan(&analyzed, target_statement)
        .map_err(|e| io::Error::other(format!("Failed to compute execution plan: {}", e)))?;

    // Load the data source if provided
    use sql_cli::data::datatable_loaders::load_csv_to_datatable;
    use sql_cli::data::query_engine::QueryEngine;
    use sql_cli::data::temp_table_registry::TempTableRegistry;
    use std::sync::Arc;

    let data_table = if !data_file.is_empty() {
        load_csv_to_datatable(std::path::Path::new(data_file), "data")
            .map_err(|e| io::Error::other(format!("Failed to load data file: {}", e)))?
    } else {
        use sql_cli::data::datatable::DataTable;
        DataTable::dual()
    };

    let table_arc = Arc::new(data_table);
    let engine = QueryEngine::new();
    let mut temp_tables = TempTableRegistry::new();

    // Execute all dependent statements to materialize temp tables
    for stmt_num in &plan.statements_to_execute {
        if *stmt_num >= target_statement {
            break; // Only execute dependencies, not the target itself
        }

        let stmt_sql = &statements[stmt_num - 1];
        let mut stmt_parser = Parser::new(stmt_sql);
        let parsed_stmt = stmt_parser.parse().map_err(|e| {
            io::Error::other(format!("Failed to parse statement #{}: {}", stmt_num, e))
        })?;

        let into_table_name = parsed_stmt.into_table.as_ref().map(|it| it.name.clone());

        match engine.execute_statement_with_temp_tables(
            table_arc.clone(),
            parsed_stmt,
            Some(&temp_tables),
        ) {
            Ok(result) => {
                if let Some(temp_name) = into_table_name {
                    let temp_table = engine.materialize_view(result).map_err(|e| {
                        io::Error::other(format!("Failed to materialize temp table: {}", e))
                    })?;
                    temp_tables.insert(temp_name.clone(), Arc::new(temp_table));
                }
            }
            Err(e) => {
                eprintln!("Error executing statement #{}: {}", stmt_num, e);
                std::process::exit(1);
            }
        }
    }

    // Now determine columns available at target line
    // Execute the target statement to get its schema (this handles CTEs, subqueries, etc.)
    let target_sql = &statements[target_statement - 1];
    let mut target_parser = Parser::new(target_sql);
    let target_ast = target_parser
        .parse()
        .map_err(|e| io::Error::other(format!("Failed to parse target statement: {}", e)))?;

    // Execute with LIMIT 0 to get just the schema (no data)
    let mut limited_ast = target_ast.clone();
    limited_ast.limit = Some(0);

    // Execute to get schema
    let result = engine
        .execute_statement_with_temp_tables(table_arc.clone(), limited_ast, Some(&temp_tables))
        .map_err(|e| {
            io::Error::other(format!(
                "Failed to execute target statement for schema: {}",
                e
            ))
        })?;

    // Get column names from the result
    let columns = result.column_names();

    // Output columns as CSV (one line, comma-separated)
    println!("{}", columns.join(","));
    Ok(())
}

/// Helper function to output DataView results
/// This is a simplified version that calls the non_interactive module's private functions
fn output_dataview<W: Write>(
    dataview: &DataView,
    format: OutputFormat,
    writer: &mut W,
    max_col_width: Option<usize>,
    _col_sample_rows: usize,
    _exec_time_ms: f64,
    table_style: TableStyle,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Csv => output_csv_helper(dataview, writer, ','),
        OutputFormat::Tsv => output_csv_helper(dataview, writer, '\t'),
        OutputFormat::Json => output_json_helper(dataview, writer),
        OutputFormat::Table => output_table_helper(dataview, writer, max_col_width, table_style),
        _ => Err(anyhow::anyhow!(
            "Output format not supported in this context"
        )),
    }
}

fn output_csv_helper<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    delimiter: char,
) -> anyhow::Result<()> {
    let columns = dataview.column_names();
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, "{delimiter}")?;
        }
        write!(writer, "{}", col)?;
    }
    writeln!(writer)?;

    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i > 0 {
                    write!(writer, "{delimiter}")?;
                }
                let s = format_datavalue(value);
                write!(writer, "{}", s)?;
            }
            writeln!(writer)?;
        }
    }
    Ok(())
}

fn output_json_helper<W: Write>(dataview: &DataView, writer: &mut W) -> anyhow::Result<()> {
    let columns = dataview.column_names();
    let mut rows = Vec::new();

    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            let mut json_row = serde_json::Map::new();
            for (col_idx, value) in row.values.iter().enumerate() {
                if col_idx < columns.len() {
                    json_row.insert(columns[col_idx].clone(), datavalue_to_json(value));
                }
            }
            rows.push(serde_json::Value::Object(json_row));
        }
    }

    let json = serde_json::to_string_pretty(&rows)?;
    writeln!(writer, "{json}")?;
    Ok(())
}

fn output_table_helper<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    max_col_width: Option<usize>,
    _style: TableStyle,
) -> anyhow::Result<()> {
    let columns = dataview.column_names();
    let mut widths = vec![0; columns.len()];

    for (i, col) in columns.iter().enumerate() {
        widths[i] = col.len();
    }

    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_datavalue(value);
                    widths[i] = widths[i].max(display_width(&value_str));
                }
            }
        }
    }

    if let Some(max_width) = max_col_width {
        for width in &mut widths {
            *width = (*width).min(max_width);
        }
    }

    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    write!(writer, "|")?;
    for (i, col) in columns.iter().enumerate() {
        write!(writer, " {:^width$} |", col, width = widths[i])?;
    }
    writeln!(writer)?;

    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            write!(writer, "|")?;
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_datavalue(value);
                    let display_len = display_width(&value_str);

                    // For ANSI-colored strings, manual padding is needed
                    // because format! uses byte length, not display width
                    write!(writer, " {}", value_str)?;
                    let padding_needed = if display_len < widths[i] {
                        widths[i] - display_len
                    } else {
                        0
                    };
                    write!(writer, "{} |", " ".repeat(padding_needed))?;
                }
            }
            writeln!(writer)?;
        }
    }

    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    Ok(())
}

fn format_datavalue(value: &DataValue) -> String {
    use sql_cli::data::datatable::DataValue;
    match value {
        DataValue::Null => String::new(),
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => f.to_string(),
        DataValue::String(s) => s.clone(),
        DataValue::InternedString(s) => s.to_string(),
        DataValue::Boolean(b) => b.to_string(),
        DataValue::DateTime(dt) => dt.to_string(),
    }
}

fn datavalue_to_json(value: &DataValue) -> serde_json::Value {
    use sql_cli::data::datatable::DataValue;
    match value {
        DataValue::Null => serde_json::Value::Null,
        DataValue::Integer(i) => serde_json::Value::Number((*i).into()),
        DataValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        }
        DataValue::String(s) => serde_json::Value::String(s.clone()),
        DataValue::InternedString(s) => serde_json::Value::String(s.to_string()),
        DataValue::Boolean(b) => serde_json::Value::Bool(*b),
        DataValue::DateTime(dt) => serde_json::Value::String(dt.clone()),
    }
}

/// Handle non-interactive query mode
/// Executes queries from command line or file and outputs results
fn handle_non_interactive_query(
    args: &[String],
    parsed_args: NonInteractiveArgs,
) -> io::Result<()> {
    // Read query from file if specified
    let query_file = parsed_args.query_file_arg.clone();
    let query = if let Some(file) = &query_file {
        std::fs::read_to_string(file)
            .map_err(|e| io::Error::other(format!("Failed to read query file {file}: {e}")))?
    } else {
        parsed_args.query_arg.clone().unwrap()
    };

    // Check if this is a multi-statement script (contains GO separator)
    let is_script = query
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case("go"));

    // Find the data file if provided
    let data_file = args
        .iter()
        .filter(|arg| !arg.starts_with('-'))
        .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
        .cloned()
        .unwrap_or_default(); // Use empty string if no data file

    // Handle --execute-statement flag (dependency-aware execution)
    if let Some(target_stmt) = parsed_args.execute_statement_arg {
        if !is_script {
            eprintln!(
                "Error: --execute-statement requires a multi-statement script with GO separators"
            );
            eprintln!("Use -f to provide a script file, or add GO between statements");
            std::process::exit(1);
        }

        return handle_execute_statement(
            &query,
            &data_file,
            target_stmt,
            parsed_args.dry_run_arg,
            &parsed_args,
        );
    }

    // Handle --get-columns-at flag (for IDE/plugin integration)
    if let Some(target_line) = parsed_args.get_columns_at_arg {
        if !is_script {
            eprintln!(
                "Error: --get-columns-at requires a multi-statement script with GO separators"
            );
            eprintln!("Use -f to provide a script file");
            std::process::exit(1);
        }

        return handle_get_columns_at(&query, &data_file, target_line);
    }

    // Parse max column width (0 = unlimited, default = 50)
    let max_col_width = if let Some(pos) = args.iter().position(|arg| arg == "--max-col-width") {
        // Flag was provided, parse the value
        args.get(pos + 1)
            .and_then(|s| s.parse::<usize>().ok())
            .and_then(|n| if n == 0 { None } else { Some(n) })
    } else {
        // Flag not provided, use default of 50
        Some(50)
    };

    // Parse column sample rows (0 = all rows, default = 100)
    let col_sample_rows = args
        .iter()
        .position(|arg| arg == "--col-sample-rows")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    // Handle query analysis commands (for IDE/plugin integration)
    if parsed_args.analyze_query_arg
        || parsed_args.expand_star_arg
        || parsed_args.extract_cte_arg.is_some()
        || parsed_args.query_at_position_arg.is_some()
    {
        use sql_cli::analysis;
        use sql_cli::sql::recursive_parser::Parser;

        // Parse the query (already loaded from file or command line)
        let mut parser = Parser::new(&query);
        let ast = parser.parse().map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Parse error: {e}"))
        })?;

        // Handle different analysis commands
        if parsed_args.analyze_query_arg {
            let analysis = analysis::analyze_query(&ast, &query);
            println!(
                "{}",
                serde_json::to_string_pretty(&analysis).map_err(io::Error::other)?
            );
            return Ok(());
        }

        if parsed_args.expand_star_arg {
            // TODO: Implement column expansion
            // This requires loading data or executing CTEs to get schema
            eprintln!("--expand-star: Not yet implemented");
            eprintln!("Coming soon: Will expand SELECT * to actual column names");
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "expand-star not yet implemented",
            ));
        }

        if let Some(cte_name) = parsed_args.extract_cte_arg {
            if let Some(cte_query) = analysis::extract_cte(&ast, &cte_name) {
                println!("{}", cte_query);
                return Ok(());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("CTE '{}' not found in query", cte_name),
                ));
            }
        }

        if let Some(position) = parsed_args.query_at_position_arg {
            let parts: Vec<&str> = position.split(':').collect();
            if parts.len() != 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Position must be in format line:column (e.g., 45:10)",
                ));
            }

            let line = parts[0].parse::<usize>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid line number: {e}"),
                )
            })?;

            let column = parts[1].parse::<usize>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid column number: {e}"),
                )
            })?;

            let context = analysis::find_query_context(&ast, line, column);
            println!(
                "{}",
                serde_json::to_string_pretty(&context).map_err(io::Error::other)?
            );
            return Ok(());
        }
    }

    let config = sql_cli::non_interactive::NonInteractiveConfig {
        data_file,
        query,
        output_format: sql_cli::non_interactive::OutputFormat::from_str(
            &parsed_args.output_format_arg,
        )
        .map_err(io::Error::other)?,
        output_file: parsed_args.output_file_arg,
        case_insensitive: args.contains(&"--case-insensitive".to_string()),
        auto_hide_empty: args.contains(&"--auto-hide-empty".to_string()),
        limit: parsed_args.limit_arg,
        query_plan: parsed_args.query_plan_arg,
        show_work_units: parsed_args.show_work_units_arg,
        execution_plan: parsed_args.execution_plan_arg,
        show_preprocessing: parsed_args.show_preprocessing_arg,
        show_transformations: parsed_args.show_transformations_arg,
        cte_info: parsed_args.cte_info_arg,
        rewrite_analysis: parsed_args.rewrite_analysis_arg,
        lift_in_expressions: parsed_args.lift_in_arg,
        script_file: parsed_args.query_file_arg.clone(),
        debug_trace: parsed_args.debug_arg,
        max_col_width,
        col_sample_rows,
        table_style: sql_cli::non_interactive::TableStyle::from_str(&parsed_args.table_style_arg)
            .map_err(io::Error::other)?,
        styled: parsed_args.styled_arg,
        style_file: parsed_args.style_file_arg.clone(),
        no_where_expansion: parsed_args.no_where_expansion_arg,
        no_group_by_expansion: parsed_args.no_group_by_expansion_arg,
        no_having_expansion: parsed_args.no_having_expansion_arg,
        no_order_by_expansion: parsed_args.no_order_by_expansion_arg,
        no_expression_lifter: parsed_args.no_expression_lifter_arg,
        no_cte_hoister: parsed_args.no_cte_hoister_arg,
        no_in_lifter: parsed_args.no_in_lifter_arg,
    };

    // Use script executor if GO separator is detected, otherwise normal execution
    if is_script {
        sql_cli::non_interactive::execute_script(config).map_err(io::Error::other)
    } else {
        sql_cli::non_interactive::execute_non_interactive(config).map_err(io::Error::other)
    }
}

fn main() -> io::Result<()> {
    // Parse arguments first to handle version/help before logging init
    let args: Vec<String> = std::env::args().collect();

    // Handle quick exit flags (version, help, list-table-styles)
    if let Some(result) = main_handlers::handle_quick_flags(&args) {
        return result;
    }

    // Handle SQL refactoring tools (--generate-bands, --generate-case, etc.)
    if let Some(result) = main_handlers::handle_refactoring_flags(&args) {
        return result;
    }

    // Handle cache operations
    if let Some(result) = main_handlers::handle_cache_flags(&args) {
        return result;
    }

    // Handle SQL formatting
    if let Some(result) = main_handlers::handle_format_flags(&args) {
        return result;
    }

    // Handle documentation flags (functions, aggregates, generators)
    if let Some(result) = main_handlers::handle_doc_flags(&args) {
        return result;
    }

    // Handle distinct column analysis
    if let Some(result) = main_handlers::handle_distinct_column_flag(&args) {
        return result;
    }

    // Handle benchmark mode
    if let Some(result) = main_handlers::handle_benchmark_flags(&args) {
        return result;
    }

    // Handle schema inspection
    if let Some(result) = main_handlers::handle_schema_flags(&args) {
        return result;
    }

    // Parse non-interactive arguments into a structured context
    let parsed_args = parse_non_interactive_args(&args);

    // Initialize unified logging (tracing + dual logging) for both modes
    // Do this before non-interactive mode so we get debug logs
    sql_cli::utils::logging::init_tracing_with_dual_logging();

    // Check if running in non-interactive mode
    let is_non_interactive =
        parsed_args.query_arg.is_some() || parsed_args.query_file_arg.is_some();

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
        return handle_non_interactive_query(&args, parsed_args);
    }

    // Handle config initialization wizard
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

    // Handle action debugger mode (--keys and --keys-simple)
    if let Some(result) = handle_key_debugger_mode(&args) {
        return result;
    }

    // Handle config file generation (--generate-config)
    if let Some(result) = handle_config_generation(&args) {
        return result;
    }

    // Don't launch TUI if we're just checking schema
    let is_schema_check =
        args.contains(&"--schema".to_string()) || args.contains(&"--schema-json".to_string());

    let use_classic_tui = args.contains(&"--simple".to_string());
    let use_tui = !args.contains(&"--classic".to_string()) && !is_schema_check;

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
            // Launch enhanced TUI - on error, fall through to classic mode
            if launch_enhanced_tui(data_file.clone(), data_files.clone()).is_ok() {
                return Ok(());
            }
            // Error already printed by launch_enhanced_tui, just fall through
        }
        return Ok(());
    }

    // Classic console mode (Reedline-based SQL editor)
    run_classic_console_mode()
}
