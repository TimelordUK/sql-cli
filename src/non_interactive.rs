use anyhow::{Context, Result};
use comfy_table::presets::*;
use comfy_table::{ContentArrangement, Table};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

use crate::config::config::Config;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::datatable_loaders::load_json_to_datatable;
// Phase 1: TempTableRegistry no longer used directly - it's in ExecutionContext
// use crate::data::temp_table_registry::TempTableRegistry;
use crate::services::query_execution_service::QueryExecutionService;
use crate::sql::parser::ast::{CTEType, TableSource, CTE};
use crate::sql::recursive_parser::Parser;
use crate::sql::script_parser::{ScriptParser, ScriptResult};
use crate::utils::string_utils::display_width;

/// Check if a query references temporary tables (starting with #)
/// Temporary tables are only valid in script mode
fn check_temp_table_usage(query: &str) -> Result<()> {
    use crate::sql::recursive_parser::Parser;

    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => {
            // Check FROM clause
            if let Some(from_table) = &stmt.from_table {
                if from_table.starts_with('#') {
                    anyhow::bail!(
                        "Temporary table '{}' cannot be used in single-query mode. \
                        Temporary tables are only available in script mode (using -f flag with GO separators).",
                        from_table
                    );
                }
            }

            // Check INTO clause
            if let Some(into_table) = &stmt.into_table {
                anyhow::bail!(
                    "INTO clause for temporary table '{}' is only supported in script mode. \
                    Use -f flag with GO separators to create temporary tables.",
                    into_table.name
                );
            }

            Ok(())
        }
        Err(_) => {
            // If parse fails, let it fail later in the actual execution
            Ok(())
        }
    }
}

/// Extract dependencies from a CTE by analyzing what tables it references
fn extract_cte_dependencies(cte: &CTE) -> Vec<String> {
    // For now, we'll just return the from_table if it exists
    // This could be enhanced to do deeper analysis of the query AST
    let mut deps = Vec::new();

    if let CTEType::Standard(query) = &cte.cte_type {
        if let Some(from_table) = &query.from_table {
            deps.push(from_table.clone());
        }

        // Could also check joins, subqueries, etc.
        for join in &query.joins {
            match &join.table {
                TableSource::Table(table_name) => {
                    deps.push(table_name.clone());
                }
                TableSource::DerivedTable { alias, .. } => {
                    deps.push(alias.clone());
                }
                TableSource::Pivot { alias, .. } => {
                    // Use the pivot alias if available
                    if let Some(pivot_alias) = alias {
                        deps.push(pivot_alias.clone());
                    }
                }
            }
        }
    }

    deps
}

/// Output format for query results
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Csv,
    Json,
    JsonStructured, // Structured JSON with metadata for IDE/plugin integration
    Table,
    Tsv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "json-structured" => Ok(OutputFormat::JsonStructured),
            "table" => Ok(OutputFormat::Table),
            "tsv" => Ok(OutputFormat::Tsv),
            _ => Err(anyhow::anyhow!(
                "Invalid output format: {}. Use csv, json, json-structured, table, or tsv",
                s
            )),
        }
    }
}

/// Table styling presets for table output format
#[derive(Debug, Clone, Copy)]
pub enum TableStyle {
    /// Current default ASCII style with borders
    Default,
    /// ASCII table with full borders
    AsciiFull,
    /// ASCII table with condensed rows
    AsciiCondensed,
    /// ASCII table with only outer borders
    AsciiBordersOnly,
    /// ASCII table with horizontal lines only
    AsciiHorizontalOnly,
    /// ASCII table with no borders
    AsciiNoBorders,
    /// Markdown-style table
    Markdown,
    /// UTF8 table with box-drawing characters
    Utf8Full,
    /// UTF8 table with condensed rows
    Utf8Condensed,
    /// UTF8 table with only outer borders
    Utf8BordersOnly,
    /// UTF8 table with horizontal lines only
    Utf8HorizontalOnly,
    /// UTF8 table with no borders
    Utf8NoBorders,
    /// No table formatting, just data
    Plain,
}

impl TableStyle {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "default" => Ok(TableStyle::Default),
            "ascii" | "ascii-full" => Ok(TableStyle::AsciiFull),
            "ascii-condensed" => Ok(TableStyle::AsciiCondensed),
            "ascii-borders" | "ascii-borders-only" => Ok(TableStyle::AsciiBordersOnly),
            "ascii-horizontal" | "ascii-horizontal-only" => Ok(TableStyle::AsciiHorizontalOnly),
            "ascii-noborders" | "ascii-no-borders" => Ok(TableStyle::AsciiNoBorders),
            "markdown" | "md" => Ok(TableStyle::Markdown),
            "utf8" | "utf8-full" => Ok(TableStyle::Utf8Full),
            "utf8-condensed" => Ok(TableStyle::Utf8Condensed),
            "utf8-borders" | "utf8-borders-only" => Ok(TableStyle::Utf8BordersOnly),
            "utf8-horizontal" | "utf8-horizontal-only" => Ok(TableStyle::Utf8HorizontalOnly),
            "utf8-noborders" | "utf8-no-borders" => Ok(TableStyle::Utf8NoBorders),
            "plain" | "none" => Ok(TableStyle::Plain),
            _ => Err(anyhow::anyhow!(
                "Invalid table style: {}. Use: default, ascii, ascii-condensed, ascii-borders, ascii-horizontal, ascii-noborders, markdown, utf8, utf8-condensed, utf8-borders, utf8-horizontal, utf8-noborders, plain",
                s
            )),
        }
    }

    pub fn list_styles() -> &'static str {
        "Available table styles:
  default            - Current default ASCII style
  ascii              - ASCII with full borders
  ascii-condensed    - ASCII with condensed rows
  ascii-borders      - ASCII with outer borders only
  ascii-horizontal   - ASCII with horizontal lines only
  ascii-noborders    - ASCII with no borders
  markdown           - GitHub-flavored Markdown table
  utf8               - UTF8 box-drawing characters
  utf8-condensed     - UTF8 with condensed rows
  utf8-borders       - UTF8 with outer borders only
  utf8-horizontal    - UTF8 with horizontal lines only
  utf8-noborders     - UTF8 with no borders
  plain              - No formatting, data only"
    }
}

/// Configuration for non-interactive query execution
pub struct NonInteractiveConfig {
    pub data_file: String,
    pub query: String,
    pub output_format: OutputFormat,
    pub output_file: Option<String>,
    pub case_insensitive: bool,
    pub auto_hide_empty: bool,
    pub limit: Option<usize>,
    pub query_plan: bool,
    pub show_work_units: bool,
    pub execution_plan: bool,
    pub show_preprocessing: bool,
    pub show_transformations: bool,
    pub cte_info: bool,
    pub rewrite_analysis: bool,
    pub lift_in_expressions: bool,
    pub script_file: Option<String>, // Path to the script file for relative path resolution
    pub debug_trace: bool,
    pub max_col_width: Option<usize>, // Maximum column width for table output (None = unlimited)
    pub col_sample_rows: usize,       // Number of rows to sample for column width (0 = all rows)
    pub table_style: TableStyle,      // Table styling preset (only affects table output format)
    pub styled: bool,                 // Whether to apply color styling rules
    pub style_file: Option<String>,   // Path to YAML style configuration file
    pub no_where_expansion: bool,     // Disable WHERE clause alias expansion
    pub no_group_by_expansion: bool,  // Disable GROUP BY clause alias expansion
    pub no_having_expansion: bool,    // Disable HAVING clause auto-aliasing
    pub no_order_by_expansion: bool,  // Disable ORDER BY aggregate expansion
    pub no_qualify_to_where: bool,    // Disable QUALIFY to WHERE transformation
    pub no_expression_lifter: bool,   // Disable expression lifting transformer
    pub no_cte_hoister: bool,         // Disable CTE hoisting transformer
    pub no_in_lifter: bool,           // Disable IN operator lifting transformer
    pub delimiter_override: Option<u8>, // Explicit --delimiter flag; overrides extension auto-detect
}

/// Convert NonInteractiveConfig flags to TransformerConfig
fn make_transformer_config(config: &NonInteractiveConfig) -> crate::query_plan::TransformerConfig {
    crate::query_plan::TransformerConfig {
        enable_pivot_expander: true, // Always enabled
        enable_expression_lifter: !config.no_expression_lifter,
        enable_where_expansion: !config.no_where_expansion,
        enable_group_by_expansion: !config.no_group_by_expansion,
        enable_having_expansion: !config.no_having_expansion,
        enable_order_by_expansion: !config.no_order_by_expansion,
        enable_qualify_to_where: !config.no_qualify_to_where,
        enable_ilike_to_like: true, // Always enabled
        enable_cte_hoister: !config.no_cte_hoister,
        enable_in_lifter: !config.no_in_lifter,
    }
}

/// Execute a query in non-interactive mode
pub fn execute_non_interactive(config: NonInteractiveConfig) -> Result<()> {
    let start_time = Instant::now();

    // Phase 2: REMOVED temp table blocking - temp tables now work in single query mode!
    // check_temp_table_usage(&config.query)?;

    // Phase 3.3: Check for data file hint in query if no data file specified
    let data_file_to_use = if config.data_file.is_empty() {
        // Look for -- #! hint in first 10 lines
        let lines: Vec<&str> = config.query.lines().take(10).collect();
        let mut hint_path: Option<String> = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("-- #!") {
                hint_path = Some(trimmed[5..].trim().to_string());
                break;
            }
        }

        if let Some(path) = hint_path {
            debug!("Found data file hint: {}", path);
            // Resolve relative paths
            let resolved_path = if path.starts_with("../") {
                // Relative to current directory
                std::path::Path::new(&path).to_path_buf()
            } else if path.starts_with("data/") {
                // Relative to project root (assume current dir)
                std::path::Path::new(&path).to_path_buf()
            } else {
                std::path::Path::new(&path).to_path_buf()
            };

            if resolved_path.exists() {
                info!("Using data file from hint: {:?}", resolved_path);
                resolved_path.to_string_lossy().to_string()
            } else {
                debug!("Data file hint path does not exist: {:?}", resolved_path);
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        config.data_file.clone()
    };

    // Phase 2: Load data file and create unified execution context
    let (data_table, _is_dual) = if data_file_to_use.is_empty() {
        info!("No data file provided, using DUAL table");
        (crate::data::datatable::DataTable::dual(), true)
    } else {
        info!("Loading data from: {}", data_file_to_use);
        let table = load_data_file(&data_file_to_use, config.delimiter_override)?;
        info!(
            "Loaded {} rows with {} columns",
            table.row_count(),
            table.column_count()
        );
        (table, false)
    };
    let _table_name = data_table.name.clone();

    // Phase 2: Create unified execution context (replaces standalone DataView)
    use crate::execution::{ExecutionConfig, ExecutionContext, StatementExecutor};
    let mut context = ExecutionContext::new(std::sync::Arc::new(data_table));

    // Keep DataView for backward compatibility with special flags
    let dataview = DataView::new(context.source_table.clone());

    // 3. Execute the query
    info!("Executing query: {}", config.query);

    // If execution_plan is requested, show detailed execution information
    if config.execution_plan {
        println!("\n=== EXECUTION PLAN ===");
        println!("Query: {}", config.query);
        println!("\nExecution Steps:");
        println!("1. PARSE - Parse SQL query");
        println!("2. LOAD_DATA - Load data from {}", &config.data_file);
        println!(
            "   • Loaded {} rows, {} columns",
            dataview.row_count(),
            dataview.column_count()
        );
    }

    // If show_work_units is requested, analyze and display work units
    if config.show_work_units {
        use crate::query_plan::{ExpressionLifter, QueryAnalyzer};
        use crate::sql::recursive_parser::Parser;

        let mut parser = Parser::new(&config.query);
        match parser.parse() {
            Ok(stmt) => {
                let mut analyzer = QueryAnalyzer::new();
                let mut lifter = ExpressionLifter::new();

                // Check if the query has liftable expressions
                let mut stmt_copy = stmt.clone();
                let lifted = lifter.lift_expressions(&mut stmt_copy);

                // Build the query plan
                match analyzer.analyze(&stmt_copy, config.query.clone()) {
                    Ok(plan) => {
                        println!("\n{}", plan.explain());

                        if !lifted.is_empty() {
                            println!("\nLifted CTEs:");
                            for cte in &lifted {
                                println!("  - {}", cte.name);
                            }
                        }

                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error analyzing query: {}", e);
                        return Err(anyhow::anyhow!("Query analysis failed: {}", e));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing query: {}", e);
                return Err(anyhow::anyhow!("Parse error: {}", e));
            }
        }
    }

    // If query_plan is requested, parse and display the AST
    if config.query_plan {
        use crate::sql::recursive_parser::Parser;
        let mut parser = Parser::new(&config.query);
        match parser.parse() {
            Ok(statement) => {
                println!("\n=== QUERY PLAN (AST) ===");
                println!("{statement:#?}");
                println!("=== END QUERY PLAN ===\n");
            }
            Err(e) => {
                eprintln!("Failed to parse query for plan: {e}");
            }
        }
    }

    // If rewrite analysis is requested, analyze query for optimization opportunities
    if config.rewrite_analysis {
        use crate::sql::query_rewriter::{QueryRewriter, RewriteAnalysis};
        use crate::sql::recursive_parser::Parser;
        use serde_json::json;

        let mut parser = Parser::new(&config.query);
        match parser.parse() {
            Ok(statement) => {
                let mut rewriter = QueryRewriter::new();
                let suggestions = rewriter.analyze(&statement);

                let analysis = RewriteAnalysis::from_suggestions(suggestions);
                println!("{}", serde_json::to_string_pretty(&analysis).unwrap());
                return Ok(());
            }
            Err(e) => {
                let output = json!({
                    "success": false,
                    "error": format!("{}", e),
                    "suggestions": [],
                    "can_auto_rewrite": false,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
                return Ok(());
            }
        }
    }

    // If CTE info is requested, parse and output CTE information as JSON
    if config.cte_info {
        use crate::sql::recursive_parser::Parser;
        use serde_json::json;

        let mut parser = Parser::new(&config.query);
        match parser.parse() {
            Ok(statement) => {
                let mut cte_info = Vec::new();

                // Extract CTE information
                for (index, cte) in statement.ctes.iter().enumerate() {
                    let cte_json = json!({
                        "index": index,
                        "name": cte.name,
                        "columns": cte.column_list,
                        // We can't easily get line numbers from the AST,
                        // but we can provide the structure
                        "dependencies": extract_cte_dependencies(cte),
                    });
                    cte_info.push(cte_json);
                }

                let output = json!({
                    "success": true,
                    "ctes": cte_info,
                    "total": statement.ctes.len(),
                    "has_final_select": !statement.columns.is_empty() || !statement.select_items.is_empty(),
                });

                println!("{}", serde_json::to_string_pretty(&output).unwrap());
                return Ok(());
            }
            Err(e) => {
                let output = json!({
                    "success": false,
                    "error": format!("{}", e),
                    "ctes": [],
                    "total": 0,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
                return Ok(());
            }
        }
    }

    let query_start = Instant::now();

    // Load configuration file to get date notation and other settings
    let app_config = Config::load().unwrap_or_else(|e| {
        debug!("Could not load config file: {}. Using defaults.", e);
        Config::default()
    });

    // Initialize global config for function registry
    crate::config::global::init_config(app_config.clone());

    // Phase 2: Create unified execution config from CLI flags
    let exec_config = ExecutionConfig::from_cli_flags(
        config.show_preprocessing,
        config.show_transformations,
        config.case_insensitive,
        config.auto_hide_empty,
        config.no_expression_lifter,
        config.no_where_expansion,
        config.no_group_by_expansion,
        config.no_having_expansion,
        config.no_order_by_expansion,
        config.no_qualify_to_where,
        config.no_cte_hoister,
        config.no_in_lifter,
        config.debug_trace,
    );

    // Phase 2: Create unified statement executor
    let executor = StatementExecutor::with_config(exec_config);

    // Phase 2: Parse and execute using unified infrastructure
    let exec_start = Instant::now();
    let result = {
        use crate::sql::recursive_parser::Parser;

        // Phase 3.1: Expand templates BEFORE parsing (same as script mode)
        use crate::sql::template_expander::TemplateExpander;
        let expander = TemplateExpander::new(&context.temp_tables);

        let query_to_parse = match expander.parse_templates(&config.query) {
            Ok(vars) => {
                if vars.is_empty() {
                    config.query.clone()
                } else {
                    match expander.expand(&config.query, &vars) {
                        Ok(expanded) => {
                            debug!(
                                "Template expansion in single query mode: {} vars expanded",
                                vars.len()
                            );
                            for var in &vars {
                                debug!("  {} -> resolved", var.placeholder);
                            }
                            expanded
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Template expansion failed: {}. Available tables: {}",
                                e,
                                context.temp_tables.list_tables().join(", ")
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Template parsing failed: {}", e));
            }
        };

        let mut parser = Parser::new(&query_to_parse);
        match parser.parse() {
            Ok(stmt) => {
                // Phase 2: Execute using unified StatementExecutor
                // This handles:
                // - Table resolution (base, temp tables, DUAL)
                // - Preprocessing pipeline (all transformers)
                // - Direct AST execution (no re-parsing!)
                match executor.execute(stmt, &mut context) {
                    Ok(exec_result) => {
                        // Convert to QueryExecutionResult for compatibility
                        Ok(
                            crate::services::query_execution_service::QueryExecutionResult {
                                dataview: exec_result.dataview,
                                stats: crate::services::query_execution_service::QueryStats {
                                    row_count: exec_result.stats.row_count,
                                    column_count: exec_result.stats.column_count,
                                    execution_time: exec_start.elapsed(),
                                    query_engine_time: exec_start.elapsed(),
                                },
                                hidden_columns: Vec::new(),
                                query: config.query.clone(),
                                execution_plan: None,
                                debug_trace: None,
                            },
                        )
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => {
                // Parse failed, return error
                Err(anyhow::anyhow!("Parse error: {}", e))
            }
        }
    }?;
    let exec_time = exec_start.elapsed();

    let query_time = query_start.elapsed();
    info!("Query executed in {:?}", query_time);
    info!(
        "Result: {} rows, {} columns",
        result.dataview.row_count(),
        result.dataview.column_count()
    );

    // Show execution plan details if requested
    if config.execution_plan {
        // Try to get detailed execution plan
        use crate::data::query_engine::QueryEngine;

        let query_engine = QueryEngine::new();

        match query_engine.execute_with_plan(
            std::sync::Arc::new(dataview.source().clone()),
            &config.query,
        ) {
            Ok((_view, plan)) => {
                // Display the detailed execution plan tree
                print!("{}", plan.format_tree());
            }
            Err(e) => {
                // Fall back to simple execution plan display
                eprintln!("Could not generate detailed execution plan: {}", e);
                println!(
                    "3. QUERY_EXECUTION [{:.3}ms]",
                    exec_time.as_secs_f64() * 1000.0
                );

                // Parse query to understand what operations are being performed
                use crate::sql::recursive_parser::Parser;
                let mut parser = Parser::new(&config.query);
                if let Ok(stmt) = parser.parse() {
                    if stmt.where_clause.is_some() {
                        println!("   • WHERE clause filtering applied");
                        println!("   • Rows after filter: {}", result.dataview.row_count());
                    }

                    if let Some(ref order_by) = stmt.order_by {
                        println!("   • ORDER BY: {} column(s)", order_by.len());
                    }

                    if let Some(ref group_by) = stmt.group_by {
                        println!("   • GROUP BY: {} column(s)", group_by.len());
                    }

                    if let Some(limit) = stmt.limit {
                        println!("   • LIMIT: {} rows", limit);
                    }

                    if stmt.distinct {
                        println!("   • DISTINCT applied");
                    }
                }
            }
        }

        println!("\nExecution Statistics:");
        println!(
            "  Preparation:    {:.3}ms",
            (exec_start - start_time).as_secs_f64() * 1000.0
        );
        println!(
            "  Query time:     {:.3}ms",
            exec_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Total time:     {:.3}ms",
            query_time.as_secs_f64() * 1000.0
        );
        println!("  Rows returned:  {}", result.dataview.row_count());
        println!("  Columns:        {}", result.dataview.column_count());
        println!("\n=== END EXECUTION PLAN ===");
        println!();
    }

    // 4. Apply limit if specified
    let final_view = if let Some(limit) = config.limit {
        let limited_table = limit_results(&result.dataview, limit)?;
        DataView::new(std::sync::Arc::new(limited_table))
    } else {
        result.dataview
    };

    // 5. Output debug trace if enabled
    if let Some(ref trace_output) = result.debug_trace {
        eprintln!("{}", trace_output);
    }

    // 6. Output the results
    let exec_time_ms = exec_time.as_secs_f64() * 1000.0;
    let output_result = if let Some(ref path) = config.output_file {
        let mut file = fs::File::create(path)
            .with_context(|| format!("Failed to create output file: {path}"))?;
        output_results(
            &final_view,
            config.output_format,
            &mut file,
            config.max_col_width,
            config.col_sample_rows,
            exec_time_ms,
            config.table_style,
            config.styled,
            config.style_file.as_deref(),
        )?;
        info!("Results written to: {}", path);
        Ok(())
    } else {
        output_results(
            &final_view,
            config.output_format,
            &mut io::stdout(),
            config.max_col_width,
            config.col_sample_rows,
            exec_time_ms,
            config.table_style,
            config.styled,
            config.style_file.as_deref(),
        )?;
        Ok(())
    };

    let total_time = start_time.elapsed();
    debug!("Total execution time: {:?}", total_time);

    // Print stats to stderr so they don't interfere with output
    if config.output_file.is_none() {
        eprintln!(
            "\n# Query completed: {} rows in {:?}",
            final_view.row_count(),
            query_time
        );
    }

    output_result
}

/// Execute a script file with multiple SQL statements separated by GO
pub fn execute_script(config: NonInteractiveConfig) -> Result<()> {
    let _start_time = Instant::now();

    // Parse the script into individual statements with directives
    let parser = ScriptParser::new(&config.query);
    let script_statements = parser.parse_script_statements();

    if script_statements.is_empty() {
        anyhow::bail!("No statements found in script");
    }

    info!("Found {} statements in script", script_statements.len());

    // Determine data file to use (command-line overrides script hint)
    let data_file = if !config.data_file.is_empty() {
        // Command-line argument takes precedence
        config.data_file.clone()
    } else if let Some(hint) = parser.data_file_hint() {
        // Use data file hint from script
        info!("Using data file from script hint: {}", hint);

        // Resolve relative paths relative to script file if provided
        if let Some(script_path) = config.script_file.as_ref() {
            let script_dir = std::path::Path::new(script_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let hint_path = std::path::Path::new(hint);

            if hint_path.is_relative() {
                script_dir.join(hint_path).to_string_lossy().to_string()
            } else {
                hint.to_string()
            }
        } else {
            hint.to_string()
        }
    } else {
        String::new()
    };

    // Load the data file if provided, otherwise use DUAL
    let (data_table, _is_dual) = if data_file.is_empty() {
        // No data file provided, use DUAL table
        info!("No data file provided, using DUAL table");
        (DataTable::dual(), true)
    } else {
        // Check if file exists before trying to load
        if !std::path::Path::new(&data_file).exists() {
            anyhow::bail!(
                "Data file not found: {}\n\
                Please check the path is correct",
                data_file
            );
        }

        info!("Loading data from: {}", data_file);
        let table = load_data_file(&data_file, config.delimiter_override)?;
        info!(
            "Loaded {} rows with {} columns",
            table.row_count(),
            table.column_count()
        );
        (table, false)
    };

    // Track script results
    let mut script_result = ScriptResult::new();
    let mut output = Vec::new();

    // Phase 1: Create unified execution context (replaces TempTableRegistry)
    use crate::execution::{ExecutionConfig, ExecutionContext, StatementExecutor};
    let mut context = ExecutionContext::new(std::sync::Arc::new(data_table));

    // Phase 1: Create unified execution config from CLI flags
    let exec_config = ExecutionConfig::from_cli_flags(
        config.show_preprocessing,
        config.show_transformations,
        config.case_insensitive,
        config.auto_hide_empty,
        config.no_expression_lifter,
        config.no_where_expansion,
        config.no_group_by_expansion,
        config.no_having_expansion,
        config.no_order_by_expansion,
        config.no_qualify_to_where,
        config.no_cte_hoister,
        config.no_in_lifter,
        false, // debug_trace - not used in script mode
    );

    // Phase 1: Create unified statement executor
    let executor = StatementExecutor::with_config(exec_config);

    // Execute each statement
    for (idx, script_stmt) in script_statements.iter().enumerate() {
        let statement_num = idx + 1;
        let stmt_start = Instant::now();

        // Check if this is an EXIT statement
        if script_stmt.is_exit() {
            let exit_code = script_stmt.get_exit_code().unwrap_or(0);
            info!("EXIT statement encountered (code: {})", exit_code);

            // Print message for table format
            if matches!(config.output_format, OutputFormat::Table) {
                if idx > 0 {
                    output.push(String::new());
                }
                output.push(format!("-- Statement {} --", statement_num));
                output.push(format!("Script execution stopped by EXIT {}", exit_code));
            }

            // Mark as success in results
            script_result.add_success(
                statement_num,
                format!("EXIT {}", exit_code),
                0,
                stmt_start.elapsed().as_secs_f64() * 1000.0,
            );

            // Stop execution
            break;
        }

        // Check if this statement should be skipped
        if script_stmt.should_skip() {
            info!(
                "Skipping statement {} due to [SKIP] directive",
                statement_num
            );

            // Print message for table format
            if matches!(config.output_format, OutputFormat::Table) {
                if idx > 0 {
                    output.push(String::new());
                }
                output.push(format!("-- Statement {} [SKIPPED] --", statement_num));
            }

            // Mark as success but with 0 rows
            script_result.add_success(
                statement_num,
                "[SKIPPED]".to_string(),
                0,
                stmt_start.elapsed().as_secs_f64() * 1000.0,
            );

            continue;
        }

        // Get the SQL query
        let statement = match script_stmt.get_query() {
            Some(sql) => sql,
            None => continue, // Should not happen
        };

        // Print separator for table format
        if matches!(config.output_format, OutputFormat::Table) {
            if idx > 0 {
                output.push(String::new()); // Empty line between queries
            }
            output.push(format!("-- Query {} --", statement_num));
        }

        // Step 1: Expand templates in the SQL string FIRST (before parsing)
        // Phase 1: Use context.temp_tables instead of temp_tables directly
        use crate::sql::template_expander::TemplateExpander;
        let expander = TemplateExpander::new(&context.temp_tables);

        let expanded_statement = match expander.parse_templates(statement) {
            Ok(vars) => {
                if vars.is_empty() {
                    statement.to_string()
                } else {
                    match expander.expand(statement, &vars) {
                        Ok(expanded) => {
                            debug!("Expanded templates in SQL: {} vars found", vars.len());
                            for var in &vars {
                                debug!(
                                    "  {} -> expanding from {}",
                                    var.placeholder, var.table_name
                                );
                            }
                            expanded
                        }
                        Err(e) => {
                            let msg =
                                format!("Query {} template expansion error: {}", statement_num, e);
                            if matches!(config.output_format, OutputFormat::Table) {
                                output.push(msg.clone());
                            } else {
                                eprintln!("{}", msg);
                            }
                            script_result.add_failure(
                                statement_num,
                                statement.to_string(),
                                msg,
                                stmt_start.elapsed().as_secs_f64() * 1000.0,
                            );
                            continue; // Skip this statement
                        }
                    }
                }
            }
            Err(e) => {
                let msg = format!("Query {} template parse error: {}", statement_num, e);
                if matches!(config.output_format, OutputFormat::Table) {
                    output.push(msg.clone());
                } else {
                    eprintln!("{}", msg);
                }
                script_result.add_failure(
                    statement_num,
                    statement.to_string(),
                    msg,
                    stmt_start.elapsed().as_secs_f64() * 1000.0,
                );
                continue; // Skip this statement
            }
        };

        // Use the expanded statement for the rest of the processing
        let statement = expanded_statement.as_str();

        // Step 2: Parse the (possibly expanded) statement
        let mut parser = Parser::new(statement);
        let parsed_stmt = match parser.parse() {
            Ok(stmt) => stmt,
            Err(e) => {
                // If parsing fails, record error and stop (scripts stop on first error)
                let msg = format!("Query {} parse error: {}", statement_num, e);
                if matches!(config.output_format, OutputFormat::Table) {
                    output.push(msg.clone());
                } else {
                    eprintln!("{}", msg);
                }
                script_result.add_failure(
                    statement_num,
                    statement.to_string(),
                    msg,
                    stmt_start.elapsed().as_secs_f64() * 1000.0,
                );
                break;
            }
        };

        // Phase 1: Check if temp table is referenced (for better error message)
        // The executor will handle resolution, but we check here for early validation
        if let Some(from_table) = &parsed_stmt.from_table {
            if from_table.starts_with('#') && !context.has_temp_table(from_table) {
                let msg = format!(
                    "Query {} failed: Temporary table {} not found",
                    statement_num, from_table
                );
                if matches!(config.output_format, OutputFormat::Table) {
                    output.push(msg.clone());
                } else {
                    eprintln!("{}", msg);
                }
                script_result.add_failure(
                    statement_num,
                    statement.to_string(),
                    msg,
                    stmt_start.elapsed().as_secs_f64() * 1000.0,
                );
                break;
            }
        }

        // Phase 1: Remove INTO clause before execution (executor doesn't handle INTO syntax)
        // We'll handle it ourselves after execution
        let into_table = parsed_stmt.into_table.clone();
        let stmt_without_into = if into_table.is_some() {
            use crate::query_plan::IntoClauseRemover;
            IntoClauseRemover::remove_into_clause(parsed_stmt)
        } else {
            parsed_stmt
        };

        // Phase 1: Execute using unified StatementExecutor
        // This handles:
        // - Table resolution (temp tables, base table, DUAL)
        // - Preprocessing pipeline (alias expansion, transformers)
        // - Direct AST execution (no re-parsing!)
        let result = executor.execute(stmt_without_into, &mut context);

        match result {
            Ok(exec_result) => {
                let exec_time = stmt_start.elapsed().as_secs_f64() * 1000.0;
                let final_view = exec_result.dataview;

                // Phase 1: Check if this is an INTO statement - store result in temp table
                if let Some(into_table) = &into_table {
                    // Get the source table from the DataView - this contains the query result
                    let result_table = final_view.source_arc();
                    let row_count = result_table.row_count();

                    // Phase 1: Use context.store_temp_table() instead of temp_tables.insert()
                    match context.store_temp_table(into_table.name.clone(), result_table) {
                        Ok(_) => {
                            info!(
                                "Stored {} rows in temporary table {}",
                                row_count, into_table.name
                            );

                            // For INTO statements, output a confirmation message only for table format
                            // CSV/TSV/JSON formats should not include this message as it pollutes machine-readable output
                            if matches!(config.output_format, OutputFormat::Table) {
                                let mut statement_output = Vec::new();
                                writeln!(
                                    &mut statement_output,
                                    "({} rows affected) -> {}",
                                    row_count, into_table.name
                                )?;
                                output.extend(
                                    String::from_utf8_lossy(&statement_output)
                                        .lines()
                                        .map(String::from),
                                );
                            }

                            script_result.add_success(
                                statement_num,
                                statement.to_string(),
                                row_count,
                                exec_time,
                            );
                            continue; // Skip normal output formatting for INTO statements
                        }
                        Err(e) => {
                            script_result.add_failure(
                                statement_num,
                                statement.to_string(),
                                e.to_string(),
                                exec_time,
                            );
                            break;
                        }
                    }
                }

                // Format the output based on the output format (normal query without INTO)
                let mut statement_output = Vec::new();
                match config.output_format {
                    OutputFormat::Csv => {
                        output_csv(&final_view, &mut statement_output, ',')?;
                    }
                    OutputFormat::Json => {
                        output_json(&final_view, &mut statement_output)?;
                    }
                    OutputFormat::JsonStructured => {
                        output_json_structured(&final_view, &mut statement_output, exec_time)?;
                    }
                    OutputFormat::Table => {
                        output_table(
                            &final_view,
                            &mut statement_output,
                            config.max_col_width,
                            config.col_sample_rows,
                            config.table_style,
                            config.styled,
                            config.style_file.as_deref(),
                        )?;
                        writeln!(
                            &mut statement_output,
                            "Query completed: {} rows in {:.2}ms",
                            final_view.row_count(),
                            exec_time
                        )?;
                    }
                    OutputFormat::Tsv => {
                        output_csv(&final_view, &mut statement_output, '\t')?;
                    }
                }

                // Add to overall output
                output.extend(
                    String::from_utf8_lossy(&statement_output)
                        .lines()
                        .map(String::from),
                );

                script_result.add_success(
                    statement_num,
                    statement.to_string(),
                    final_view.row_count(),
                    exec_time,
                );
            }
            Err(e) => {
                let exec_time = stmt_start.elapsed().as_secs_f64() * 1000.0;
                let error_msg = format!("Query {} failed: {:#}", statement_num, e);

                // Table mode embeds the error in the output stream (alongside the
                // result tables). CSV/JSON/TSV go to stderr instead so the parseable
                // output on stdout stays clean.
                if matches!(config.output_format, OutputFormat::Table) {
                    output.push(error_msg.clone());
                } else {
                    eprintln!("{}", error_msg);
                }

                script_result.add_failure(
                    statement_num,
                    statement.to_string(),
                    e.to_string(),
                    exec_time,
                );

                // Continue to next statement (don't stop on error)
            }
        }
    }

    // Write output
    if let Some(ref output_file) = config.output_file {
        let mut file = fs::File::create(output_file)?;
        for line in &output {
            writeln!(file, "{}", line)?;
        }
        info!("Results written to: {}", output_file);
    } else {
        for line in &output {
            println!("{}", line);
        }
    }

    // Print summary if in table mode
    if matches!(config.output_format, OutputFormat::Table) {
        println!("\n=== Script Summary ===");
        println!("Total statements: {}", script_result.total_statements);
        println!("Successful: {}", script_result.successful_statements);
        println!("Failed: {}", script_result.failed_statements);
        println!(
            "Total execution time: {:.2}ms",
            script_result.total_execution_time_ms
        );
    }

    if !script_result.all_successful() {
        return Err(anyhow::anyhow!(
            "{} of {} statements failed",
            script_result.failed_statements,
            script_result.total_statements
        ));
    }

    Ok(())
}

/// Load a data file (CSV or JSON) into a `DataTable`.
///
/// `delimiter_override` (typically from `--delimiter`) wins over extension
/// auto-detect. `None` falls back to `.tsv` → tab / `.psv` → pipe / else comma.
fn load_data_file(path: &str, delimiter_override: Option<u8>) -> Result<DataTable> {
    use crate::data::datatable_loaders::load_csv_to_datatable_with_opts;
    use crate::data::stream_loader::{resolve_delimiter, CsvReadOptions};

    let path = Path::new(path);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    // Determine file type by extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    let table_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("data")
        .to_string();

    // .csv/.tsv/.psv share the CSV loader. So does any other extension when
    // an explicit --delimiter override is supplied — the user has told us
    // it's a delimited file, so we trust them.
    let is_csv_family =
        matches!(extension.as_str(), "csv" | "tsv" | "psv") || delimiter_override.is_some();
    if is_csv_family {
        let path_str = path.display().to_string();
        let opts = CsvReadOptions {
            delimiter: resolve_delimiter(&path_str, delimiter_override),
            has_headers: true,
        };
        return load_csv_to_datatable_with_opts(path, &table_name, &opts)
            .with_context(|| format!("Failed to load CSV-family file: {}", path.display()));
    }

    match extension.as_str() {
        "json" | "jsonl" | "ndjson" => load_json_to_datatable(path, &table_name)
            .with_context(|| format!("Failed to load JSON file: {}", path.display())),
        _ => Err(anyhow::anyhow!(
            "Unsupported file type: {}. Use .csv, .tsv, .psv, .json, .jsonl, or .ndjson \
             (or pass --delimiter to force CSV parsing on an unknown extension)",
            extension
        )),
    }
}

/// Limit the number of rows in results
fn limit_results(dataview: &DataView, limit: usize) -> Result<DataTable> {
    let source = dataview.source();
    let mut limited_table = DataTable::new(&source.name);

    // Copy columns
    for col in &source.columns {
        limited_table.add_column(col.clone());
    }

    // Copy limited rows
    let rows_to_copy = dataview.row_count().min(limit);
    for i in 0..rows_to_copy {
        if let Some(row) = dataview.get_row(i) {
            let _ = limited_table.add_row(row.clone());
        }
    }

    Ok(limited_table)
}

/// Output query results in the specified format
fn output_results<W: Write>(
    dataview: &DataView,
    format: OutputFormat,
    writer: &mut W,
    max_col_width: Option<usize>,
    col_sample_rows: usize,
    exec_time_ms: f64,
    table_style: TableStyle,
    styled: bool,
    style_file: Option<&str>,
) -> Result<()> {
    match format {
        OutputFormat::Csv => output_csv(dataview, writer, ','),
        OutputFormat::Tsv => output_csv(dataview, writer, '\t'),
        OutputFormat::Json => output_json(dataview, writer),
        OutputFormat::JsonStructured => output_json_structured(dataview, writer, exec_time_ms),
        OutputFormat::Table => output_table(
            dataview,
            writer,
            max_col_width,
            col_sample_rows,
            table_style,
            styled,
            style_file,
        ),
    }
}

/// Output results as CSV/TSV
fn output_csv<W: Write>(dataview: &DataView, writer: &mut W, delimiter: char) -> Result<()> {
    // Write headers
    let columns = dataview.column_names();
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, "{delimiter}")?;
        }
        write!(writer, "{}", escape_csv_field(col, delimiter))?;
    }
    writeln!(writer)?;

    // Write rows
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i > 0 {
                    write!(writer, "{delimiter}")?;
                }
                write!(
                    writer,
                    "{}",
                    escape_csv_field(&format_value(value), delimiter)
                )?;
            }
            writeln!(writer)?;
        }
    }

    Ok(())
}

/// Output results as JSON
fn output_json<W: Write>(dataview: &DataView, writer: &mut W) -> Result<()> {
    let columns = dataview.column_names();
    let mut rows = Vec::new();

    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            let mut json_row = serde_json::Map::new();
            for (col_idx, value) in row.values.iter().enumerate() {
                if col_idx < columns.len() {
                    json_row.insert(columns[col_idx].clone(), value_to_json(value));
                }
            }
            rows.push(serde_json::Value::Object(json_row));
        }
    }

    let json = serde_json::to_string_pretty(&rows)?;
    writeln!(writer, "{json}")?;

    Ok(())
}

/// Output results as structured JSON with metadata for IDE/plugin integration
fn output_json_structured<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    exec_time: f64,
) -> Result<()> {
    let column_names = dataview.column_names();
    let data_table = dataview.source();

    // Build column metadata
    let mut columns = Vec::new();
    for (idx, name) in column_names.iter().enumerate() {
        let col_type = data_table
            .columns
            .get(idx)
            .map(|c| format!("{:?}", c.data_type))
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // Calculate max width for this column
        let mut max_width = name.len();
        for row_idx in 0..dataview.row_count() {
            if let Some(row) = dataview.get_row(row_idx) {
                if let Some(value) = row.values.get(idx) {
                    let display_width = match value {
                        DataValue::Null => 4, // "NULL"
                        DataValue::Integer(i) => i.to_string().len(),
                        DataValue::Float(f) => format!("{:.2}", f).len(),
                        DataValue::String(s) => s.len(),
                        DataValue::InternedString(s) => s.len(),
                        DataValue::Boolean(b) => {
                            if *b {
                                4
                            } else {
                                5
                            }
                        } // "true" or "false"
                        DataValue::DateTime(dt) => dt.len(),
                        DataValue::Vector(v) => {
                            let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                            format!("[{}]", components.join(",")).len()
                        }
                    };
                    max_width = max_width.max(display_width);
                }
            }
        }

        let alignment = match data_table.columns.get(idx).map(|c| &c.data_type) {
            Some(crate::data::datatable::DataType::Integer) => "right",
            Some(crate::data::datatable::DataType::Float) => "right",
            _ => "left",
        };

        let col_meta = serde_json::json!({
            "name": name,
            "type": col_type,
            "max_width": max_width,
            "alignment": alignment
        });
        columns.push(col_meta);
    }

    // Build rows as arrays of strings
    let mut rows = Vec::new();
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            let row_values: Vec<String> = row
                .values
                .iter()
                .map(|v| match v {
                    DataValue::Null => String::new(),
                    DataValue::Integer(i) => i.to_string(),
                    DataValue::Float(f) => format!("{:.2}", f),
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Vector(v) => {
                        let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                        format!("[{}]", components.join(","))
                    }
                })
                .collect();
            rows.push(serde_json::Value::Array(
                row_values
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            ));
        }
    }

    // Build complete structured output
    let output = serde_json::json!({
        "columns": columns,
        "rows": rows,
        "metadata": {
            "total_rows": dataview.row_count(),
            "query_time_ms": exec_time
        }
    });

    let json = serde_json::to_string_pretty(&output)?;
    writeln!(writer, "{json}")?;

    Ok(())
}

/// Output results using the old custom ASCII table format (for Nvim compatibility)
fn output_table_old_style<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    max_col_width: Option<usize>,
) -> Result<()> {
    let columns = dataview.column_names();

    // Calculate column widths
    let mut widths = vec![0; columns.len()];
    for (i, col) in columns.iter().enumerate() {
        widths[i] = col.len();
    }

    // Scan all rows for width calculation
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_value(value);
                    widths[i] = widths[i].max(display_width(&value_str));
                }
            }
        }
    }

    // Apply maximum column width if specified
    if let Some(max_width) = max_col_width {
        for width in &mut widths {
            *width = (*width).min(max_width);
        }
    }

    // Print top border
    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    // Print headers
    write!(writer, "|")?;
    for (i, col) in columns.iter().enumerate() {
        write!(writer, " {:^width$} |", col, width = widths[i])?;
    }
    writeln!(writer)?;

    // Print header separator
    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    // Print data rows (no separators between rows)
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            write!(writer, "|")?;
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_value(value);
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

    // Print bottom border
    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "{}", "-".repeat(*width + 2))?;
        write!(writer, "+")?;
    }
    writeln!(writer)?;

    Ok(())
}

/// Output results as a table using comfy-table with styling
fn output_table<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    max_col_width: Option<usize>,
    _col_sample_rows: usize, // Not needed with comfy-table
    style: TableStyle,
    styled: bool,
    style_file: Option<&str>,
) -> Result<()> {
    let mut table = Table::new();

    // Apply the selected style preset
    match style {
        TableStyle::Default => {
            // Use custom old-style renderer for Nvim compatibility
            // This matches what the table navigation parser expects
            return output_table_old_style(dataview, writer, max_col_width);
        }
        TableStyle::AsciiFull => {
            table.load_preset(ASCII_FULL);
        }
        TableStyle::AsciiCondensed => {
            table.load_preset(ASCII_FULL_CONDENSED);
        }
        TableStyle::AsciiBordersOnly => {
            table.load_preset(ASCII_BORDERS_ONLY);
        }
        TableStyle::AsciiHorizontalOnly => {
            table.load_preset(ASCII_HORIZONTAL_ONLY);
        }
        TableStyle::AsciiNoBorders => {
            table.load_preset(ASCII_NO_BORDERS);
        }
        TableStyle::Markdown => {
            table.load_preset(ASCII_MARKDOWN);
        }
        TableStyle::Utf8Full => {
            table.load_preset(UTF8_FULL);
        }
        TableStyle::Utf8Condensed => {
            table.load_preset(UTF8_FULL_CONDENSED);
        }
        TableStyle::Utf8BordersOnly => {
            table.load_preset(UTF8_BORDERS_ONLY);
        }
        TableStyle::Utf8HorizontalOnly => {
            table.load_preset(UTF8_HORIZONTAL_ONLY);
        }
        TableStyle::Utf8NoBorders => {
            table.load_preset(UTF8_NO_BORDERS);
        }
        TableStyle::Plain => {
            table.load_preset(NOTHING);
        }
    }

    // Set content arrangement (automatic width adjustment)
    if max_col_width.is_some() {
        table.set_content_arrangement(ContentArrangement::Dynamic);
    }

    // Set column headers
    let columns = dataview.column_names();

    // Apply color styling if requested
    if styled {
        use crate::output::styled_table::{apply_styles_to_table, StyleConfig};
        use std::path::PathBuf;

        // Load style configuration
        let style_config = if let Some(file_path) = style_file {
            let path = PathBuf::from(file_path);
            StyleConfig::from_file(&path).ok()
        } else {
            StyleConfig::load_default()
        };

        if let Some(config) = style_config {
            // Convert DataView rows to Vec<Vec<String>> for styling
            let rows: Vec<Vec<String>> = (0..dataview.row_count())
                .filter_map(|i| {
                    dataview.get_row(i).map(|row| {
                        row.values
                            .iter()
                            .map(|v| {
                                let s = format_value(v);
                                // Apply max width truncation if specified
                                if let Some(max_width) = max_col_width {
                                    if s.len() > max_width {
                                        format!("{}...", &s[..max_width.saturating_sub(3)])
                                    } else {
                                        s
                                    }
                                } else {
                                    s
                                }
                            })
                            .collect()
                    })
                })
                .collect();

            if let Err(e) = apply_styles_to_table(&mut table, &columns, &rows, &config) {
                eprintln!("Warning: Failed to apply styles: {}", e);
            }
        }
    } else {
        // No styling - add headers and rows normally
        table.set_header(&columns);

        // Add data rows
        for row_idx in 0..dataview.row_count() {
            if let Some(row) = dataview.get_row(row_idx) {
                let row_strings: Vec<String> = row
                    .values
                    .iter()
                    .map(|v| {
                        let s = format_value(v);
                        // Apply max width truncation if specified
                        if let Some(max_width) = max_col_width {
                            if s.len() > max_width {
                                format!("{}...", &s[..max_width.saturating_sub(3)])
                            } else {
                                s
                            }
                        } else {
                            s
                        }
                    })
                    .collect();
                table.add_row(row_strings);
            }
        }
    }

    // Write the table to the writer
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Format a `DataValue` for display
fn format_value(value: &DataValue) -> String {
    match value {
        DataValue::Null => String::new(),
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => f.to_string(),
        DataValue::String(s) => s.clone(),
        DataValue::InternedString(s) => s.to_string(),
        DataValue::Boolean(b) => b.to_string(),
        DataValue::DateTime(dt) => dt.to_string(),
        DataValue::Vector(v) => {
            let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
            format!("[{}]", components.join(","))
        }
    }
}

/// Convert `DataValue` to JSON
fn value_to_json(value: &DataValue) -> serde_json::Value {
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
        DataValue::DateTime(dt) => serde_json::Value::String(dt.to_string()),
        DataValue::Vector(v) => serde_json::Value::Array(
            v.iter()
                .map(|f| {
                    if let Some(n) = serde_json::Number::from_f64(*f) {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    }
                })
                .collect(),
        ),
    }
}

/// Escape a CSV field if it contains special characters
fn escape_csv_field(field: &str, delimiter: char) -> String {
    if field.contains(delimiter)
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r')
    {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
