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

    use sql_cli::sql::recursive_parser::FormatConfig;
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

    match sql_cli::sql::recursive_parser::format_sql_ast_with_config(&query.trim(), &config) {
        Ok(formatted) => {
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

/// Check if we're in non-interactive mode (has -q or -f flags)
pub fn is_non_interactive(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-q" || arg == "--query" || arg == "-f" || arg == "--query-file")
}
