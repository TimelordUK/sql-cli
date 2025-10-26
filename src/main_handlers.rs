/// Handler functions for main.rs CLI flag processing
///
/// This module contains handler functions that were extracted from the massive
/// main() function to improve readability and maintainability.
use std::io;

/// Handle quick exit flags that don't need any setup
/// Returns Some(Ok(())) if handled, None if flag not found
pub fn handle_quick_flags(args: &[String]) -> Option<io::Result<()>> {
    // Version flag
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("sql-cli {}", env!("CARGO_PKG_VERSION"));
        return Some(Ok(()));
    }

    // List table styles
    if args.contains(&"--list-table-styles".to_string()) {
        println!("{}", sql_cli::non_interactive::TableStyle::list_styles());
        return Some(Ok(()));
    }

    // Help flag
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        super::print_help();
        return Some(Ok(()));
    }

    None
}

/// Handle SQL refactoring tool flags (--generate-bands, --generate-case, etc.)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_refactoring_flags(args: &[String]) -> Option<io::Result<()>> {
    // Banding generation
    if args.contains(&"--generate-bands".to_string()) {
        return Some(sql_cli::cli::refactoring::handle_banding_generation(args));
    }

    // CASE generation from data
    if args.contains(&"--generate-case".to_string()) {
        return Some(sql_cli::cli::refactoring::handle_case_generation(args));
    }

    // CASE generation from numeric range
    if args.contains(&"--generate-case-range".to_string()) {
        return Some(sql_cli::cli::refactoring::handle_case_range_generation(
            args,
        ));
    }

    None
}

/// Handle cache-related flags
/// Returns Some(result) if handled, None if flag not found
pub fn handle_cache_flags(args: &[String]) -> Option<io::Result<()>> {
    if args.contains(&"--cache-purge".to_string()) {
        use sql_cli::redis_cache_module::RedisCache;
        let mut cache = RedisCache::new();

        if !cache.is_enabled() {
            eprintln!("❌ Cache not enabled (set SQL_CLI_CACHE=true)");
            std::process::exit(1);
        }

        match cache.purge_all() {
            Ok(count) => {
                println!("✅ Purged {} cache entries", count);
                return Some(Ok(()));
            }
            Err(e) => {
                eprintln!("❌ Failed to purge cache: {}", e);
                std::process::exit(1);
            }
        }
    }

    None
}

/// Handle SQL formatting flags (--format, -F)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_format_flags(args: &[String]) -> Option<io::Result<()>> {
    if !args.contains(&"--format".to_string()) && !args.contains(&"-F".to_string()) {
        return None;
    }

    use sql_cli::sql::parser::ast_formatter::format_select_with_config;
    use sql_cli::sql::recursive_parser::{FormatConfig, Parser, ParserMode};
    use std::io::Read;

    // Check if query is provided via stdin or file
    let query = if let Some(pos) = args.iter().position(|arg| arg == "--format" || arg == "-F") {
        if let Some(file_path) = args.get(pos + 1).filter(|arg| !arg.starts_with('-')) {
            // Read from file
            match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => return Some(Err(e)),
            }
        } else {
            // Read from stdin
            let mut buffer = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                return Some(Err(e));
            }
            buffer
        }
    } else {
        // Read from stdin
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
            return Some(Err(e));
        }
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

    // Check if comment preservation is requested
    let preserve_comments = args.contains(&"--preserve-comments".to_string());

    // Parse with appropriate mode
    let parser_mode = if preserve_comments {
        ParserMode::PreserveComments
    } else {
        ParserMode::Standard
    };

    let mut parser = Parser::with_mode(&query.trim(), parser_mode);
    match parser.parse() {
        Ok(stmt) => {
            let formatted = format_select_with_config(&stmt, &config);
            println!("{}", formatted);
            Some(Ok(()))
        }
        Err(e) => {
            eprintln!("Error formatting SQL: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle documentation flags (--list-functions, --function-help, --item-help, etc.)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_doc_flags(args: &[String]) -> Option<io::Result<()>> {
    // --list-functions
    if args.contains(&"--list-functions".to_string()) {
        return Some(handle_list_functions());
    }

    // --item-help (unified help for functions, aggregates, generators)
    if let Some(pos) = args
        .iter()
        .position(|arg| arg == "--item-help" || arg == "--ihelp")
    {
        return Some(handle_item_help(args, pos));
    }

    // --function-help
    if let Some(pos) = args.iter().position(|arg| arg == "--function-help") {
        return Some(handle_function_help(args, pos));
    }

    // --generate-docs
    if args.contains(&"--generate-docs".to_string()) {
        return Some(handle_generate_docs());
    }

    // --list-generators
    if args.contains(&"--list-generators".to_string()) {
        return Some(handle_list_generators());
    }

    // --generator-help
    if let Some(pos) = args.iter().position(|arg| arg == "--generator-help") {
        return Some(handle_generator_help(args, pos));
    }

    None
}

fn handle_list_functions() -> io::Result<()> {
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

    Ok(())
}

fn handle_item_help(args: &[String], pos: usize) -> io::Result<()> {
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
        if let Some(result) = check_window_functions(name) {
            println!("{}", result);
            return Ok(());
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
    Ok(())
}

fn check_window_functions(name: &str) -> Option<String> {
    let window_registry = sql_cli::sql::window_functions::WindowFunctionRegistry::new();
    let name_upper = name.to_uppercase();

    // Check syntactic sugar window functions
    if window_registry.contains(&name_upper) {
        if let Some(func) = window_registry.get(&name_upper) {
            return Some(format!(
                "Function: {}() OVER\nCategory: Window Function (Syntactic Sugar)\nDescription: {}\nSignature: {}\n\nNote: Requires an OVER clause with ORDER BY\nExample: {}() OVER (ORDER BY date)",
                func.name(),
                func.description(),
                func.signature(),
                func.name()
            ));
        }
    }

    // Check standard window functions
    let standard_window_funcs = vec![
        (
            "ROW_NUMBER",
            "Assigns a unique sequential integer to each row within a partition",
            "ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT ROW_NUMBER() OVER (ORDER BY column) AS row_num FROM table\n  SELECT ROW_NUMBER() OVER (PARTITION BY category ORDER BY value) AS rank_in_category FROM table",
        ),
        (
            "RANK",
            "Assigns a rank to each row within a partition with gaps",
            "RANK() OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT RANK() OVER (ORDER BY column) FROM table",
        ),
        (
            "DENSE_RANK",
            "Assigns a rank to each row within a partition without gaps",
            "DENSE_RANK() OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT DENSE_RANK() OVER (ORDER BY column) FROM table",
        ),
        (
            "LAG",
            "Access data from a previous row in the same result set",
            "LAG(column, offset, default) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT LAG(price) OVER (ORDER BY date) AS prev_price FROM table\n  SELECT LAG(value, 2, 0) OVER (ORDER BY id) AS two_rows_back FROM table",
        ),
        (
            "LEAD",
            "Access data from a following row in the same result set",
            "LEAD(column, offset, default) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT LEAD(price) OVER (ORDER BY date) AS next_price FROM table\n  SELECT LEAD(value, 1, -1) OVER (ORDER BY id) AS next_value FROM table",
        ),
        (
            "FIRST_VALUE",
            "Returns the first value in an ordered set of values",
            "FIRST_VALUE(column) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT FIRST_VALUE() OVER (ORDER BY column) FROM table",
        ),
        (
            "LAST_VALUE",
            "Returns the last value in an ordered set of values",
            "LAST_VALUE(column) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT LAST_VALUE() OVER (ORDER BY column) FROM table",
        ),
        (
            "NTH_VALUE",
            "Returns the value at the nth position in an ordered set",
            "NTH_VALUE(column, n) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT NTH_VALUE() OVER (ORDER BY column) FROM table",
        ),
        (
            "PERCENT_RANK",
            "Calculates the relative rank of a row as a percentage",
            "PERCENT_RANK() OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT PERCENT_RANK() OVER (ORDER BY column) FROM table",
        ),
        (
            "CUME_DIST",
            "Calculates the cumulative distribution of a value",
            "CUME_DIST() OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT CUME_DIST() OVER (ORDER BY column) FROM table",
        ),
        (
            "NTILE",
            "Distributes rows into a specified number of groups",
            "NTILE(n) OVER (PARTITION BY ... ORDER BY ...)",
            "  SELECT NTILE() OVER (ORDER BY column) FROM table",
        ),
    ];

    for (func_name, desc, signature, examples) in standard_window_funcs {
        if name_upper == func_name {
            return Some(format!(
                "Function: {}() OVER\nCategory: Standard Window Function\nDescription: {}\nSignature: {}\n\nExamples:\n{}",
                func_name, desc, signature, examples
            ));
        }
    }

    None
}

fn handle_function_help(args: &[String], pos: usize) -> io::Result<()> {
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
    Ok(())
}

fn handle_generate_docs() -> io::Result<()> {
    let registry = sql_cli::sql::functions::FunctionRegistry::new();
    let docs = registry.generate_markdown_docs();
    let doc_path = "docs/FUNCTION_REFERENCE.md";
    std::fs::write(doc_path, docs)?;
    println!("Generated function reference documentation at: {doc_path}");
    Ok(())
}

fn handle_list_generators() -> io::Result<()> {
    let registry = sql_cli::sql::generators::GeneratorRegistry::new();
    println!("{}", registry.list_generators_formatted());
    Ok(())
}

fn handle_generator_help(args: &[String], pos: usize) -> io::Result<()> {
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
    Ok(())
}

/// Handle distinct column analysis flag (--distinct-column)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_distinct_column_flag(args: &[String]) -> Option<io::Result<()>> {
    let pos = args.iter().position(|arg| arg == "--distinct-column")?;

    let column_name = match args.get(pos + 1) {
        Some(name) => name,
        None => {
            eprintln!("Error: --distinct-column requires a column name");
            eprintln!("Usage: sql-cli -q \"SELECT * FROM data\" --distinct-column <column_name>");
            std::process::exit(1);
        }
    };

    // Get the query from -q or --query
    let query_pos = args.iter().position(|arg| arg == "-q" || arg == "--query");
    let query = match query_pos.and_then(|qpos| args.get(qpos + 1)) {
        Some(q) => q,
        None => {
            eprintln!("Error: --distinct-column requires a query via -q or --query");
            std::process::exit(1);
        }
    };

    // Build distinct query (wrap in CTE if needed)
    let distinct_query = build_distinct_query(query, column_name);

    // Get data file if provided
    let data_file = args
        .iter()
        .filter(|arg| !arg.starts_with('-'))
        .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
        .cloned()
        .unwrap_or_default();

    let config = sql_cli::non_interactive::NonInteractiveConfig {
        data_file,
        query: distinct_query,
        output_format: sql_cli::non_interactive::OutputFormat::Csv,
        output_file: None,
        case_insensitive: false,
        auto_hide_empty: false,
        limit: None,
        query_plan: false,
        show_work_units: false,
        execution_plan: false,
        show_preprocessing: false,
        cte_info: false,
        rewrite_analysis: false,
        lift_in_expressions: false,
        script_file: None,
        debug_trace: false,
        max_col_width: None,
        col_sample_rows: 100,
        table_style: sql_cli::non_interactive::TableStyle::Default,
        styled: false,
        style_file: None,
        no_where_expansion: false,
        no_group_by_expansion: false,
        no_having_expansion: false,
        no_expression_lifter: false,
        no_cte_hoister: false,
        no_in_lifter: false,
    };

    // Execute using the non-interactive interface
    match sql_cli::non_interactive::execute_non_interactive(config) {
        Ok(_) => Some(Ok(())),
        Err(e) => {
            eprintln!("Error executing query: {}", e);
            std::process::exit(1);
        }
    }
}

/// Build a distinct value query from a base query and column name
fn build_distinct_query(query: &str, column_name: &str) -> String {
    // Check if query already contains a CTE (WITH keyword)
    if query.trim().to_uppercase().starts_with("WITH ") {
        // Extract CTE name
        let cte_name = extract_cte_name(query);

        // Extract just the CTE part (everything up to and including the closing paren)
        let cte_part = extract_cte_part(query);

        format!(
            "{}\nSELECT {}, COUNT(*) as count FROM {} GROUP BY {} ORDER BY count DESC, {} LIMIT 100",
            cte_part, column_name, cte_name, column_name, column_name
        )
    } else {
        // No CTE, wrap in one
        format!(
            "WITH base_query AS ({}) SELECT {}, COUNT(*) as count FROM base_query GROUP BY {} ORDER BY count DESC, {} LIMIT 100",
            query, column_name, column_name, column_name
        )
    }
}

/// Extract CTE name from a query that starts with WITH
fn extract_cte_name(query: &str) -> String {
    if let Some(pos) = query.find(" AS ") {
        let before_as = &query[..pos];
        // Extract the name after WITH [WEB]
        let name_part = before_as
            .trim_start_matches("WITH ")
            .trim_start_matches("with ")
            .trim_start_matches("WEB ")
            .trim_start_matches("web ")
            .trim();
        name_part
            .split_whitespace()
            .next()
            .unwrap_or("data")
            .to_string()
    } else {
        "data".to_string()
    }
}

/// Extract the CTE part from a query (everything up to the closing paren)
fn extract_cte_part(query: &str) -> String {
    let mut paren_depth = 0;
    let mut in_cte_body = false;
    let mut cte_end_pos = query.len();

    for (i, ch) in query.chars().enumerate() {
        if ch == '(' && !in_cte_body {
            // Found the opening paren of AS (...)
            in_cte_body = true;
            paren_depth = 1;
        } else if in_cte_body {
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    // Found the closing paren of the CTE
                    cte_end_pos = i + 1;
                    break;
                }
            }
        }
    }

    query[..cte_end_pos].to_string()
}

/// Handle benchmark mode flags (--benchmark and related options)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_benchmark_flags(args: &[String]) -> Option<io::Result<()>> {
    if !args.contains(&"--benchmark".to_string()) {
        return None;
    }

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

    Some(Ok(()))
}

/// Handle schema inspection flags (--schema and --schema-json)
/// Returns Some(result) if handled, None if flag not found
pub fn handle_schema_flags(args: &[String]) -> Option<io::Result<()>> {
    let is_json = args.contains(&"--schema-json".to_string());
    let is_colored = args.contains(&"--schema".to_string());

    if !is_json && !is_colored {
        return None;
    }

    // Find the file argument
    let file_arg = args
        .iter()
        .find(|arg| arg.ends_with(".csv") || arg.ends_with(".json"))
        .or_else(|| args.last().filter(|arg| !arg.starts_with('-')));

    let file_path = match file_arg {
        Some(path) => path,
        None => {
            eprintln!("Error: No data file specified");
            if is_json {
                eprintln!("Usage: sql-cli <file.csv|file.json> --schema-json");
            } else {
                eprintln!("Usage: sql-cli <file.csv|file.json> --schema");
            }
            std::process::exit(1);
        }
    };

    // Load the table
    let table = match load_table_for_schema(file_path) {
        Ok(t) => t,
        Err(e) => return Some(Err(e)),
    };

    // Output schema in requested format
    if is_json {
        Some(output_schema_json(&table))
    } else {
        Some(output_schema_colored(&table))
    }
}

/// Load a table from CSV or JSON file
fn load_table_for_schema(file_path: &str) -> io::Result<sql_cli::data::datatable::DataTable> {
    let table_name = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data");

    if file_path.ends_with(".json") {
        sql_cli::data::datatable_loaders::load_json_to_datatable(file_path, table_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    } else {
        sql_cli::data::datatable_loaders::load_csv_to_datatable(file_path, table_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

/// Analyze column types by sampling data
fn analyze_column_types(
    table: &sql_cli::data::datatable::DataTable,
    column_idx: usize,
) -> (std::collections::HashMap<&'static str, usize>, usize) {
    use sql_cli::data::datatable::DataValue;
    let mut type_counts = std::collections::HashMap::new();
    let mut null_count = 0;
    let sample_size = std::cmp::min(100, table.row_count());

    for row_idx in 0..sample_size {
        if let Some(value) = table.get_value(row_idx, column_idx) {
            match value {
                DataValue::Null => null_count += 1,
                DataValue::Integer(_) => *type_counts.entry("INTEGER").or_insert(0) += 1,
                DataValue::Float(_) => *type_counts.entry("FLOAT").or_insert(0) += 1,
                DataValue::String(_) | DataValue::InternedString(_) => {
                    *type_counts.entry("STRING").or_insert(0) += 1
                }
                DataValue::Boolean(_) => *type_counts.entry("BOOLEAN").or_insert(0) += 1,
                DataValue::DateTime(_) => *type_counts.entry("DATETIME").or_insert(0) += 1,
            }
        }
    }

    (type_counts, null_count)
}

/// Output schema in JSON format
fn output_schema_json(table: &sql_cli::data::datatable::DataTable) -> io::Result<()> {
    let mut schema = serde_json::json!({
        "table": table.name,
        "rows": table.row_count(),
        "columns": []
    });

    let mut columns = Vec::new();
    for (idx, column) in table.columns.iter().enumerate() {
        let (type_counts, null_count) = analyze_column_types(table, idx);
        let sample_size = std::cmp::min(100, table.row_count());

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
    Ok(())
}

/// Output schema in colored terminal format
fn output_schema_colored(table: &sql_cli::data::datatable::DataTable) -> io::Result<()> {
    use crossterm::style::Stylize;

    println!("{}", "Table Schema".blue().bold());
    println!("{}", "═".repeat(60));
    println!("Table: {}", table.name);
    println!("Rows: {}", table.row_count());
    println!("Columns: {}", table.column_count());
    println!();
    println!("{}", "Column Information:".yellow());
    println!("{}", "─".repeat(60));

    for (idx, column) in table.columns.iter().enumerate() {
        let (type_counts, null_count) = analyze_column_types(table, idx);
        let sample_size = std::cmp::min(100, table.row_count());

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
    Ok(())
}

/// Check if we're in non-interactive mode (has -q or -f flags)
pub fn is_non_interactive(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-q" || arg == "--query" || arg == "-f" || arg == "--query-file")
}
