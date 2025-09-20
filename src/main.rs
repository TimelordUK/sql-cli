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

    // Check for SQL formatting mode
    if args.contains(&"--format".to_string()) || args.contains(&"-F".to_string()) {
        use sql_cli::sql::recursive_parser::FormatConfig;
        use std::io::Read;

        // Check if query is provided via stdin or file
        let query = if let Some(pos) = args.iter().position(|arg| arg == "--format" || arg == "-F")
        {
            if let Some(file_path) = args.get(pos + 1).filter(|arg| !arg.starts_with('-')) {
                // Read from file
                std::fs::read_to_string(file_path)?
            } else {
                // Read from stdin
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer
            }
        } else {
            // Read from stdin
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        };

        // Check for configuration options
        let config = if args.contains(&"--compact".to_string()) {
            FormatConfig {
                indent: "  ".to_string(),
                items_per_line: 10,
                uppercase_keywords: !args.contains(&"--lowercase".to_string()),
                compact: true,
            }
        } else {
            FormatConfig {
                indent: if args.contains(&"--tabs".to_string()) {
                    "\t"
                } else {
                    "    "
                }
                .to_string(),
                items_per_line: 5,
                uppercase_keywords: !args.contains(&"--lowercase".to_string()),
                compact: false,
            }
        };

        match sql_cli::sql::recursive_parser::format_sql_ast_with_config(&query.trim(), &config) {
            Ok(formatted) => {
                println!("{}", formatted);
            }
            Err(e) => {
                eprintln!("Error formatting SQL: {}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Check for function documentation flags
    if args.contains(&"--list-functions".to_string()) {
        let registry = sql_cli::sql::functions::FunctionRegistry::new();
        let window_registry = sql_cli::sql::window_functions::WindowFunctionRegistry::new();

        println!("{}", registry.list_functions());

        // Add window functions to the listing
        println!("Window Functions (Syntactic Sugar):");
        for func_name in window_registry.list_functions() {
            if let Some(func) = window_registry.get(&func_name) {
                println!(
                    "  {:20} - {}",
                    format!("{}() OVER", func.name()),
                    func.description()
                );
            }
        }
        println!("\nNote: Window functions require an OVER clause with ORDER BY");
        println!("Example: MOVING_AVG(column, 20) OVER (ORDER BY date)");

        return Ok(());
    }

    // Unified help for functions, aggregates, and generators
    if let Some(pos) = args
        .iter()
        .position(|arg| arg == "--item-help" || arg == "--ihelp")
    {
        if let Some(name) = args.get(pos + 1) {
            let func_registry = sql_cli::sql::functions::FunctionRegistry::new();
            let gen_registry = sql_cli::sql::generators::GeneratorRegistry::new();
            let agg_registry = sql_cli::sql::aggregate_functions::AggregateFunctionRegistry::new();
            let old_agg_registry = sql_cli::sql::aggregates::AggregateRegistry::new();

            // Try regular function first
            if let Some(help) = func_registry.generate_function_help(name) {
                println!("{help}");
                return Ok(());
            }

            // Try new aggregate registry
            let name_upper = name.to_uppercase();
            if agg_registry.contains(&name_upper) {
                if let Some(func) = agg_registry.get(&name_upper) {
                    println!("Function: {}()", func.name());
                    println!("Category: Aggregate");
                    println!("Description: {}", func.description());
                    println!("Arguments: 1 argument (column)");
                    println!("Returns: Aggregated value\n");
                    println!("Examples:");
                    println!("  SELECT {}(value) FROM table", func.name());
                    println!(
                        "  SELECT category, {}(amount) FROM table GROUP BY category",
                        func.name()
                    );
                    return Ok(());
                }
            }

            // Try old aggregate registry
            if old_agg_registry.is_aggregate(&name_upper) {
                if let Some(func) = old_agg_registry.get(&name_upper) {
                    println!("Function: {}()", func.name());
                    println!("Category: Aggregate");
                    println!("Description: Aggregate function");
                    println!("Arguments: 1 argument (column)");
                    println!("Returns: Aggregated value\n");
                    println!("Examples:");
                    println!("  SELECT {}(value) FROM table", func.name());
                    return Ok(());
                }
            }

            // Try generator if not found elsewhere
            if let Some(help) = gen_registry.get_generator_help(name) {
                println!("{help}");
                return Ok(());
            }

            // Try window functions (both syntactic sugar and standard)
            let window_registry = sql_cli::sql::window_functions::WindowFunctionRegistry::new();
            let name_upper = name.to_uppercase();

            // Check syntactic sugar window functions
            if window_registry.contains(&name_upper) {
                if let Some(func) = window_registry.get(&name_upper) {
                    println!("Function: {}() OVER", func.name());
                    println!("Category: Window Function (Syntactic Sugar)");
                    println!("Description: {}", func.description());
                    println!("Signature: {}", func.signature());
                    println!("\nNote: Requires an OVER clause with ORDER BY");
                    println!("Example: {}() OVER (ORDER BY date)", func.name());
                    return Ok(());
                }
            }

            // Check standard window functions
            let standard_window_funcs = vec![
                (
                    "ROW_NUMBER",
                    "Assigns a unique sequential integer to each row within a partition",
                    "ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "RANK",
                    "Assigns a rank to each row within a partition with gaps",
                    "RANK() OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "DENSE_RANK",
                    "Assigns a rank to each row within a partition without gaps",
                    "DENSE_RANK() OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "LAG",
                    "Access data from a previous row in the same result set",
                    "LAG(column, offset, default) OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "LEAD",
                    "Access data from a following row in the same result set",
                    "LEAD(column, offset, default) OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "FIRST_VALUE",
                    "Returns the first value in an ordered set of values",
                    "FIRST_VALUE(column) OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "LAST_VALUE",
                    "Returns the last value in an ordered set of values",
                    "LAST_VALUE(column) OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "NTH_VALUE",
                    "Returns the value at the nth position in an ordered set",
                    "NTH_VALUE(column, n) OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "PERCENT_RANK",
                    "Calculates the relative rank of a row as a percentage",
                    "PERCENT_RANK() OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "CUME_DIST",
                    "Calculates the cumulative distribution of a value",
                    "CUME_DIST() OVER (PARTITION BY ... ORDER BY ...)",
                ),
                (
                    "NTILE",
                    "Distributes rows into a specified number of groups",
                    "NTILE(n) OVER (PARTITION BY ... ORDER BY ...)",
                ),
            ];

            for (func_name, desc, signature) in standard_window_funcs {
                if name_upper == func_name {
                    println!("Function: {}() OVER", func_name);
                    println!("Category: Standard Window Function");
                    println!("Description: {}", desc);
                    println!("Signature: {}", signature);
                    println!("\nExamples:");
                    match func_name {
                        "ROW_NUMBER" => {
                            println!("  SELECT ROW_NUMBER() OVER (ORDER BY column) AS row_num FROM table");
                            println!("  SELECT ROW_NUMBER() OVER (PARTITION BY category ORDER BY value) AS rank_in_category FROM table");
                        }
                        "LAG" => {
                            println!(
                                "  SELECT LAG(price) OVER (ORDER BY date) AS prev_price FROM table"
                            );
                            println!("  SELECT LAG(value, 2, 0) OVER (ORDER BY id) AS two_rows_back FROM table");
                        }
                        "LEAD" => {
                            println!("  SELECT LEAD(price) OVER (ORDER BY date) AS next_price FROM table");
                            println!("  SELECT LEAD(value, 1, -1) OVER (ORDER BY id) AS next_value FROM table");
                        }
                        _ => {
                            println!("  SELECT {}() OVER (ORDER BY column) FROM table", func_name);
                        }
                    }
                    return Ok(());
                }
            }

            eprintln!(
                "'{}' not found in functions, aggregates, generators, or window functions",
                name
            );
            eprintln!("\nUse --list-functions, --list-aggregates, or --list-generators to see available items");
        } else {
            eprintln!("Error: --item-help requires a name");
            eprintln!("Usage: sql-cli --item-help <function_aggregate_or_generator_name>");
        }
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|arg| arg == "--function-help") {
        if let Some(func_name) = args.get(pos + 1) {
            let registry = sql_cli::sql::functions::FunctionRegistry::new();
            if let Some(help) = registry.generate_function_help(func_name) {
                println!("{help}");
            } else {
                // Check window function registry
                let window_registry = sql_cli::sql::window_functions::WindowFunctionRegistry::new();
                let func_name_upper = func_name.to_uppercase();
                if let Some(func) = window_registry.get(&func_name_upper) {
                    println!("Window Function: {}\n", func.name());
                    println!("Description: {}\n", func.description());
                    println!("Signature: {}\n", func.signature());
                    println!("Usage: Requires OVER clause with ORDER BY");
                    println!("\nExample:");
                    println!("  SELECT date, close,");
                    println!("    {} OVER (ORDER BY date) as result", func.signature());
                    println!("  FROM table_name");
                } else {
                    eprintln!("Function '{func_name}' not found");
                    eprintln!("\nUse --list-functions to see all available functions");
                }
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

    // Check for generator documentation flags
    if args.contains(&"--list-generators".to_string()) {
        let registry = sql_cli::sql::generators::GeneratorRegistry::new();
        println!("{}", registry.list_generators_formatted());
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|arg| arg == "--generator-help") {
        if let Some(gen_name) = args.get(pos + 1) {
            let registry = sql_cli::sql::generators::GeneratorRegistry::new();
            if let Some(help) = registry.get_generator_help(gen_name) {
                println!("{help}");
            } else {
                eprintln!("Generator '{}' not found", gen_name);
                eprintln!("\nUse --list-generators to see all available generators");
            }
        } else {
            eprintln!("Error: --generator-help requires a generator name");
            eprintln!("Usage: sql-cli --generator-help <generator_name>");
        }
        return Ok(());
    }

    // Check for benchmark mode
    if args.contains(&"--benchmark".to_string()) {
        use sql_cli::benchmarks::{BenchmarkRunner, QueryCategory};

        // Parse benchmark-specific arguments
        let sizes = if let Some(pos) = args.iter().position(|arg| arg == "--sizes") {
            args.get(pos + 1)
                .and_then(|s| {
                    let parts: Result<Vec<usize>, _> =
                        s.split(',').map(|n| n.trim().parse::<usize>()).collect();
                    parts.ok()
                })
                .unwrap_or_else(|| vec![100, 1000, 10000, 50000, 100000])
        } else {
            vec![100, 1000, 10000, 50000, 100000]
        };

        let category = args
            .iter()
            .position(|arg| arg == "--category")
            .and_then(|pos| args.get(pos + 1))
            .and_then(|s| match s.as_str() {
                "basic" => Some(QueryCategory::BasicOperations),
                "aggregation" => Some(QueryCategory::Aggregations),
                "sorting" => Some(QueryCategory::SortingAndLimits),
                "window" => Some(QueryCategory::WindowFunctions),
                "complex" => Some(QueryCategory::ComplexQueries),
                _ => None,
            });

        let progressive = args.contains(&"--progressive".to_string());
        let report_file = args
            .iter()
            .position(|arg| arg == "--report")
            .and_then(|pos| args.get(pos + 1));
        let csv_file = args
            .iter()
            .position(|arg| arg == "--csv")
            .and_then(|pos| args.get(pos + 1));

        println!("=== SQL CLI Performance Benchmark Tool ===\n");

        let mut runner = BenchmarkRunner::new();

        if progressive {
            // Progressive benchmark mode (10k increments)
            let increment = args
                .iter()
                .position(|arg| arg == "--increment")
                .and_then(|pos| args.get(pos + 1))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10000);

            let max_rows = args
                .iter()
                .position(|arg| arg == "--max-rows")
                .and_then(|pos| args.get(pos + 1))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100000);

            runner.run_progressive_benchmarks(increment, max_rows);
        } else if let Some(cat) = category {
            // Category-specific benchmarks
            runner.run_category_benchmarks(cat, &sizes);
        } else {
            // Full comprehensive benchmarks
            runner.run_comprehensive_benchmarks(&sizes);
        }

        // Print summary
        runner.print_summary();

        // Save CSV results if requested
        if let Some(csv_path) = csv_file {
            match runner.save_results_csv(csv_path) {
                Ok(()) => println!("\nBenchmark results saved to: {}", csv_path),
                Err(e) => eprintln!("Error saving CSV results: {}", e),
            }
        }

        // Generate and save report if requested
        if let Some(report_path) = report_file {
            let report = runner.generate_report();
            match std::fs::write(report_path, report) {
                Ok(()) => println!("Benchmark report saved to: {}", report_path),
                Err(e) => eprintln!("Error saving report: {}", e),
            }
        }

        return Ok(());
    }

    // Check for schema inspection (JSON format)
    if args.contains(&"--schema-json".to_string()) {
        // Find the file argument
        let file_arg = args
            .iter()
            .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
            .or_else(|| args.last().filter(|arg| !arg.starts_with('-')));

        if let Some(file_path) = file_arg {
            // Load the table using the appropriate loader
            let table_name = std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("data");

            let table = if file_path.ends_with(".json") {
                sql_cli::data::datatable_loaders::load_json_to_datatable(file_path, table_name)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            } else {
                sql_cli::data::datatable_loaders::load_csv_to_datatable(file_path, table_name)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            };

            // Output JSON schema
            let mut schema = serde_json::json!({
                "table": table.name,
                "rows": table.row_count(),
                "columns": []
            });

            // Analyze column types by sampling data
            let mut columns = Vec::new();
            for (idx, column) in table.columns.iter().enumerate() {
                let mut type_counts = std::collections::HashMap::new();
                let mut null_count = 0;
                let sample_size = std::cmp::min(100, table.row_count());

                for row_idx in 0..sample_size {
                    if let Some(value) = table.get_value(row_idx, idx) {
                        match value {
                            sql_cli::data::datatable::DataValue::Null => null_count += 1,
                            sql_cli::data::datatable::DataValue::Integer(_) => {
                                *type_counts.entry("INTEGER").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::Float(_) => {
                                *type_counts.entry("FLOAT").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::String(_)
                            | sql_cli::data::datatable::DataValue::InternedString(_) => {
                                *type_counts.entry("STRING").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::Boolean(_) => {
                                *type_counts.entry("BOOLEAN").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::DateTime(_) => {
                                *type_counts.entry("DATETIME").or_insert(0) += 1
                            }
                        }
                    }
                }

                // Determine primary type
                let primary_type = type_counts
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(type_name, _)| *type_name)
                    .unwrap_or("UNKNOWN");

                columns.push(serde_json::json!({
                    "name": column.name,
                    "type": primary_type,
                    "nullable": null_count > 0,
                    "null_percentage": if sample_size > 0 { (null_count * 100) / sample_size } else { 0 }
                }));
            }

            schema["columns"] = serde_json::json!(columns);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        } else {
            eprintln!("Error: No data file specified");
            eprintln!("Usage: sql-cli <file.csv|file.json> --schema-json");
        }
        return Ok(());
    }

    // Check for schema inspection (colored format)
    if args.contains(&"--schema".to_string()) {
        // Find the file argument
        let file_arg = args
            .iter()
            .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
            .or_else(|| args.last().filter(|arg| !arg.starts_with('-')));

        if let Some(file_path) = file_arg {
            // Load the table using the appropriate loader
            let table_name = std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("data");

            let table = if file_path.ends_with(".json") {
                sql_cli::data::datatable_loaders::load_json_to_datatable(file_path, table_name)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            } else {
                sql_cli::data::datatable_loaders::load_csv_to_datatable(file_path, table_name)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            };

            // Print schema information
            println!("{}", "Table Schema".blue().bold());
            println!("{}", "═".repeat(60));
            println!("Table: {}", table.name);
            println!("Rows: {}", table.row_count());
            println!("Columns: {}", table.column_count());
            println!();
            println!("{}", "Column Information:".yellow());
            println!("{}", "─".repeat(60));

            // Analyze column types by sampling data
            for (idx, column) in table.columns.iter().enumerate() {
                let mut type_counts = std::collections::HashMap::new();
                let mut null_count = 0;
                let sample_size = std::cmp::min(100, table.row_count());

                for row_idx in 0..sample_size {
                    if let Some(value) = table.get_value(row_idx, idx) {
                        match value {
                            sql_cli::data::datatable::DataValue::Null => null_count += 1,
                            sql_cli::data::datatable::DataValue::Integer(_) => {
                                *type_counts.entry("INTEGER").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::Float(_) => {
                                *type_counts.entry("FLOAT").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::String(_)
                            | sql_cli::data::datatable::DataValue::InternedString(_) => {
                                *type_counts.entry("STRING").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::Boolean(_) => {
                                *type_counts.entry("BOOLEAN").or_insert(0) += 1
                            }
                            sql_cli::data::datatable::DataValue::DateTime(_) => {
                                *type_counts.entry("DATETIME").or_insert(0) += 1
                            }
                        }
                    }
                }

                // Determine primary type
                let primary_type = type_counts
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(type_name, _)| *type_name)
                    .unwrap_or("UNKNOWN");

                println!(
                    "  {:3}. {:<30} {:<10} {}",
                    idx + 1,
                    column.name.clone().green(),
                    primary_type.cyan(),
                    if null_count > 0 {
                        format!("({}% NULL)", null_count * 100 / sample_size)
                            .red()
                            .to_string()
                    } else {
                        "".to_string()
                    }
                );
            }

            println!();
            println!("{}", "Note: Types inferred from first 100 rows".italic());
        } else {
            eprintln!("Error: No data file specified");
            eprintln!("Usage: sql-cli <file.csv|file.json> --schema");
        }
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

    let show_work_units_arg = args
        .iter()
        .any(|arg| arg == "--show-work-units" || arg == "--show_work_units");

    let lift_in_arg = args
        .iter()
        .any(|arg| arg == "--check-in-lifting" || arg == "--check_in_lifting");

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
        };

        // Use script executor if GO separator is detected, otherwise normal execution
        if is_script {
            return sql_cli::non_interactive::execute_script(config).map_err(io::Error::other);
        } else {
            return sql_cli::non_interactive::execute_non_interactive(config)
                .map_err(io::Error::other);
        }
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
