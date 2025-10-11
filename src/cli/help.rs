use crossterm::style::Stylize;

/// Print comprehensive help text for SQL CLI
pub fn print_help() {
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
