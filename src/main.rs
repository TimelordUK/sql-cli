use crossterm::style::Stylize;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Emacs, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, ValidationResult, Validator,
};
use sql_cli::utils::app_paths::AppPaths;
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

/// Handle non-interactive query mode
/// Executes queries from command line or file and outputs results
fn handle_non_interactive_query(
    args: &[String],
    query_arg: Option<String>,
    query_file_arg: Option<String>,
    output_format_arg: String,
    output_file_arg: Option<String>,
    table_style_arg: String,
    query_plan_arg: bool,
    show_work_units_arg: bool,
    execution_plan_arg: bool,
    cte_info_arg: bool,
    rewrite_analysis_arg: bool,
    lift_in_arg: bool,
    debug_arg: bool,
    limit_arg: Option<usize>,
    analyze_query_arg: bool,
    expand_star_arg: bool,
    extract_cte_arg: Option<String>,
    query_at_position_arg: Option<String>,
) -> io::Result<()> {
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

    // Find the data file if provided
    let data_file = args
        .iter()
        .filter(|arg| !arg.starts_with('-'))
        .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
        .cloned()
        .unwrap_or_default(); // Use empty string if no data file

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
    if analyze_query_arg
        || expand_star_arg
        || extract_cte_arg.is_some()
        || query_at_position_arg.is_some()
    {
        use sql_cli::analysis;
        use sql_cli::sql::recursive_parser::Parser;

        // Parse the query (already loaded from file or command line)
        let mut parser = Parser::new(&query);
        let ast = parser.parse().map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Parse error: {e}"))
        })?;

        // Handle different analysis commands
        if analyze_query_arg {
            let analysis = analysis::analyze_query(&ast, &query);
            println!(
                "{}",
                serde_json::to_string_pretty(&analysis).map_err(io::Error::other)?
            );
            return Ok(());
        }

        if expand_star_arg {
            // TODO: Implement column expansion
            // This requires loading data or executing CTEs to get schema
            eprintln!("--expand-star: Not yet implemented");
            eprintln!("Coming soon: Will expand SELECT * to actual column names");
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "expand-star not yet implemented",
            ));
        }

        if let Some(cte_name) = extract_cte_arg {
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

        if let Some(position) = query_at_position_arg {
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
        output_format: sql_cli::non_interactive::OutputFormat::from_str(&output_format_arg)
            .map_err(io::Error::other)?,
        output_file: output_file_arg,
        case_insensitive: args.contains(&"--case-insensitive".to_string()),
        auto_hide_empty: args.contains(&"--auto-hide-empty".to_string()),
        limit: limit_arg,
        query_plan: query_plan_arg,
        show_work_units: show_work_units_arg,
        execution_plan: execution_plan_arg,
        cte_info: cte_info_arg,
        rewrite_analysis: rewrite_analysis_arg,
        lift_in_expressions: lift_in_arg,
        script_file: query_file_arg.clone(),
        debug_trace: debug_arg,
        max_col_width,
        col_sample_rows,
        table_style: sql_cli::non_interactive::TableStyle::from_str(&table_style_arg)
            .map_err(io::Error::other)?,
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

    let table_style_arg = args
        .iter()
        .position(|arg| arg == "--table-style")
        .and_then(|pos| args.get(pos + 1))
        .map_or_else(|| "default".to_string(), std::string::ToString::to_string);

    let query_plan_arg = args
        .iter()
        .any(|arg| arg == "--query-plan" || arg == "--query_plan");

    let show_work_units_arg = args
        .iter()
        .any(|arg| arg == "--show-work-units" || arg == "--show_work_units");

    let lift_in_arg = args
        .iter()
        .any(|arg| arg == "--check-in-lifting" || arg == "--check_in_lifting");

    // Query analysis flags (for IDE/plugin integration)
    let analyze_query_arg = args
        .iter()
        .any(|arg| arg == "--analyze-query" || arg == "--analyze_query");

    let expand_star_arg = args
        .iter()
        .any(|arg| arg == "--expand-star" || arg == "--expand_star");

    let extract_cte_arg = args
        .iter()
        .position(|arg| arg == "--extract-cte" || arg == "--extract_cte")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    let query_at_position_arg = args
        .iter()
        .position(|arg| arg == "--query-at-position" || arg == "--query_at_position")
        .and_then(|pos| args.get(pos + 1))
        .map(std::string::ToString::to_string);

    let execution_plan_arg = args
        .iter()
        .any(|arg| arg == "--execution-plan" || arg == "--execution_plan");

    let cte_info_arg = args
        .iter()
        .any(|arg| arg == "--cte-info" || arg == "--cte-json");

    let rewrite_analysis_arg = args
        .iter()
        .any(|arg| arg == "--analyze-rewrite" || arg == "--rewrite-analysis");

    let debug_arg = args
        .iter()
        .any(|arg| arg == "--debug" || arg == "--debug-trace");

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
        return handle_non_interactive_query(
            &args,
            query_arg,
            query_file_arg,
            output_format_arg,
            output_file_arg,
            table_style_arg,
            query_plan_arg,
            show_work_units_arg,
            execution_plan_arg,
            cte_info_arg,
            rewrite_analysis_arg,
            lift_in_arg,
            debug_arg,
            limit_arg,
            analyze_query_arg,
            expand_star_arg,
            extract_cte_arg,
            query_at_position_arg,
        );
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
