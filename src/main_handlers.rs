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

/// Check if we're in non-interactive mode (has -q or -f flags)
pub fn is_non_interactive(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-q" || arg == "--query" || arg == "-f" || arg == "--query-file")
}
