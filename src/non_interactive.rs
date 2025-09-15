use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

use crate::config::config::Config;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::datatable_loaders::{load_csv_to_datatable, load_json_to_datatable};
use crate::services::query_execution_service::QueryExecutionService;
use crate::sql::recursive_parser::{CTEType, Parser, SelectStatement};
use crate::sql::script_parser::{ScriptParser, ScriptResult};

/// Output format for query results
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Csv,
    Json,
    Table,
    Tsv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "tsv" => Ok(OutputFormat::Tsv),
            _ => Err(anyhow::anyhow!(
                "Invalid output format: {}. Use csv, json, table, or tsv",
                s
            )),
        }
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
    pub script_file: Option<String>, // Path to the script file for relative path resolution
}

/// Execute a query in non-interactive mode
pub fn execute_non_interactive(config: NonInteractiveConfig) -> Result<()> {
    let start_time = Instant::now();

    // 1. Load the data file or create DUAL table
    let (data_table, _is_dual) = if config.data_file.is_empty() {
        info!("No data file provided, using DUAL table");
        (crate::data::datatable::DataTable::dual(), true)
    } else {
        info!("Loading data from: {}", config.data_file);
        let table = load_data_file(&config.data_file)?;
        info!(
            "Loaded {} rows with {} columns",
            table.row_count(),
            table.column_count()
        );
        (table, false)
    };
    let _table_name = data_table.name.clone();

    // 2. Create a DataView from the table
    let dataview = DataView::new(std::sync::Arc::new(data_table));

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

    let query_start = Instant::now();

    // Load configuration file to get date notation and other settings
    let app_config = Config::load().unwrap_or_else(|e| {
        debug!("Could not load config file: {}. Using defaults.", e);
        Config::default()
    });

    // Initialize global config for function registry
    crate::config::global::init_config(app_config.clone());

    // Use QueryExecutionService with full BehaviorConfig
    let mut behavior_config = app_config.behavior.clone();
    debug!(
        "Using date notation: {}",
        behavior_config.default_date_notation
    );
    // Command line args override config file settings
    if config.case_insensitive {
        behavior_config.case_insensitive_default = true;
    }
    if config.auto_hide_empty {
        behavior_config.hide_empty_columns = true;
    }

    let query_service = QueryExecutionService::with_behavior_config(behavior_config);

    let exec_start = Instant::now();
    let result = query_service.execute(&config.query, Some(&dataview), Some(dataview.source()))?;
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

    // 5. Output the results
    let output_result = if let Some(ref path) = config.output_file {
        let mut file = fs::File::create(path)
            .with_context(|| format!("Failed to create output file: {path}"))?;
        output_results(&final_view, config.output_format, &mut file)?;
        info!("Results written to: {}", path);
        Ok(())
    } else {
        output_results(&final_view, config.output_format, &mut io::stdout())?;
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

    // Parse the script into individual statements
    let parser = ScriptParser::new(&config.query);
    let statements = parser.parse_and_validate()?;

    info!("Found {} statements in script", statements.len());

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
        let table = load_data_file(&data_file)?;
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

    // Create Arc<DataTable> once for all statements - avoids expensive cloning
    let arc_data_table = std::sync::Arc::new(data_table);

    // Execute each statement
    for (idx, statement) in statements.iter().enumerate() {
        let statement_num = idx + 1;
        let stmt_start = Instant::now();

        // Print separator for table format
        if matches!(config.output_format, OutputFormat::Table) {
            if idx > 0 {
                output.push(String::new()); // Empty line between queries
            }
            output.push(format!("-- Query {} --", statement_num));
        }

        // Create a fresh DataView for each statement (reuses the Arc)
        let dataview = DataView::new(arc_data_table.clone());

        // Execute the statement
        let service = QueryExecutionService::new(config.case_insensitive, config.auto_hide_empty);
        match service.execute(statement, Some(&dataview), None) {
            Ok(result) => {
                let exec_time = stmt_start.elapsed().as_secs_f64() * 1000.0;
                let final_view = result.dataview;

                // Format the output based on the output format
                let mut statement_output = Vec::new();
                match config.output_format {
                    OutputFormat::Csv => {
                        output_csv(&final_view, &mut statement_output, ',')?;
                    }
                    OutputFormat::Json => {
                        output_json(&final_view, &mut statement_output)?;
                    }
                    OutputFormat::Table => {
                        output_table(&final_view, &mut statement_output)?;
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
                    statement.clone(),
                    final_view.row_count(),
                    exec_time,
                );
            }
            Err(e) => {
                let exec_time = stmt_start.elapsed().as_secs_f64() * 1000.0;
                let error_msg = format!("Query {} failed: {}", statement_num, e);

                if matches!(config.output_format, OutputFormat::Table) {
                    output.push(error_msg.clone());
                }

                script_result.add_failure(
                    statement_num,
                    statement.clone(),
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

/// Load a data file (CSV or JSON) into a `DataTable`
fn load_data_file(path: &str) -> Result<DataTable> {
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

    match extension.as_str() {
        "csv" => load_csv_to_datatable(path, &table_name)
            .with_context(|| format!("Failed to load CSV file: {}", path.display())),
        "json" => load_json_to_datatable(path, &table_name)
            .with_context(|| format!("Failed to load JSON file: {}", path.display())),
        _ => Err(anyhow::anyhow!(
            "Unsupported file type: {}. Use .csv or .json",
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
            limited_table.add_row(row.clone());
        }
    }

    Ok(limited_table)
}

/// Output query results in the specified format
fn output_results<W: Write>(
    dataview: &DataView,
    format: OutputFormat,
    writer: &mut W,
) -> Result<()> {
    match format {
        OutputFormat::Csv => output_csv(dataview, writer, ','),
        OutputFormat::Tsv => output_csv(dataview, writer, '\t'),
        OutputFormat::Json => output_json(dataview, writer),
        OutputFormat::Table => output_table(dataview, writer),
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

/// Output results as an ASCII table
fn output_table<W: Write>(dataview: &DataView, writer: &mut W) -> Result<()> {
    let columns = dataview.column_names();

    // Calculate column widths
    let mut widths = vec![0; columns.len()];
    for (i, col) in columns.iter().enumerate() {
        widths[i] = col.len();
    }

    // Check first 100 rows for width calculation
    let sample_size = dataview.row_count().min(100);
    for row_idx in 0..sample_size {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_value(value);
                    widths[i] = widths[i].max(value_str.len());
                }
            }
        }
    }

    // Limit column widths to 50 characters
    for width in &mut widths {
        *width = (*width).min(50);
    }

    // Print header separator
    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "-{}-+", "-".repeat(*width))?;
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
        write!(writer, "-{}-+", "-".repeat(*width))?;
    }
    writeln!(writer)?;

    // Print rows
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            write!(writer, "|")?;
            for (i, value) in row.values.iter().enumerate() {
                if i < widths.len() {
                    let value_str = format_value(value);
                    let truncated = if value_str.len() > widths[i] {
                        format!("{}...", &value_str[..widths[i] - 3])
                    } else {
                        value_str
                    };
                    write!(writer, " {:<width$} |", truncated, width = widths[i])?;
                }
            }
            writeln!(writer)?;
        }
    }

    // Print bottom separator
    write!(writer, "+")?;
    for width in &widths {
        write!(writer, "-{}-+", "-".repeat(*width))?;
    }
    writeln!(writer)?;

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
