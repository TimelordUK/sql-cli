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
}

/// Execute a query in non-interactive mode
pub fn execute_non_interactive(config: NonInteractiveConfig) -> Result<()> {
    let start_time = Instant::now();

    // Check if query uses DUAL or has no FROM clause
    use crate::sql::recursive_parser::Parser;
    let mut parser = Parser::new(&config.query);
    let statement = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let uses_dual = statement
        .from_table
        .as_ref()
        .map(|t| t.to_uppercase() == "DUAL")
        .unwrap_or(false);

    let no_from_clause = statement.from_table.is_none();

    // 1. Load the data file or create DUAL table
    let (data_table, is_dual) = if uses_dual || no_from_clause || config.data_file.is_empty() {
        info!("Using DUAL table for expression evaluation");
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
    let table_name = data_table.name.clone();

    // 2. Create a DataView from the table
    let dataview = DataView::new(std::sync::Arc::new(data_table));

    // 3. Execute the query
    info!("Executing query: {}", config.query);

    // If query_plan is requested, parse and display the AST
    if config.query_plan {
        use crate::sql::recursive_parser::Parser;
        let mut parser = Parser::new(&config.query);
        match parser.parse() {
            Ok(statement) => {
                println!("\n=== QUERY PLAN (AST) ===");
                println!("{:#?}", statement);
                println!("=== END QUERY PLAN ===\n");
            }
            Err(e) => {
                eprintln!("Failed to parse query for plan: {}", e);
            }
        }
    }

    let query_start = Instant::now();

    // Load configuration file to get date notation and other settings
    let app_config = Config::load().unwrap_or_else(|e| {
        debug!("Could not load config file: {}. Using defaults.", e);
        Config::default()
    });

    // Use QueryExecutionService with full BehaviorConfig
    let mut behavior_config = app_config.behavior.clone();
    debug!("Using date notation: {}", behavior_config.default_date_notation);
    // Command line args override config file settings
    if config.case_insensitive {
        behavior_config.case_insensitive_default = true;
    }
    if config.auto_hide_empty {
        behavior_config.hide_empty_columns = true;
    }
    
    let query_service = QueryExecutionService::with_behavior_config(behavior_config);
    let result = query_service.execute(&config.query, Some(&dataview), Some(dataview.source()))?;

    let query_time = query_start.elapsed();
    info!("Query executed in {:?}", query_time);
    info!(
        "Result: {} rows, {} columns",
        result.dataview.row_count(),
        result.dataview.column_count()
    );

    // 4. Apply limit if specified
    let final_view = if let Some(limit) = config.limit {
        let limited_table = limit_results(&result.dataview, limit)?;
        DataView::new(std::sync::Arc::new(limited_table))
    } else {
        result.dataview
    };

    // 5. Output the results
    let output_result = match config.output_file {
        Some(ref path) => {
            let mut file = fs::File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path))?;
            output_results(&final_view, config.output_format, &mut file)?;
            info!("Results written to: {}", path);
            Ok(())
        }
        None => {
            output_results(&final_view, config.output_format, &mut io::stdout())?;
            Ok(())
        }
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

/// Load a data file (CSV or JSON) into a DataTable
fn load_data_file(path: &str) -> Result<DataTable> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    // Determine file type by extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
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
    for col in source.columns.iter() {
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
            write!(writer, "{}", delimiter)?;
        }
        write!(writer, "{}", escape_csv_field(col, delimiter))?;
    }
    writeln!(writer)?;

    // Write rows
    for row_idx in 0..dataview.row_count() {
        if let Some(row) = dataview.get_row(row_idx) {
            for (i, value) in row.values.iter().enumerate() {
                if i > 0 {
                    write!(writer, "{}", delimiter)?;
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
    writeln!(writer, "{}", json)?;

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
    for width in widths.iter_mut() {
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

/// Format a DataValue for display
fn format_value(value: &DataValue) -> String {
    match value {
        DataValue::Null => "".to_string(),
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => f.to_string(),
        DataValue::String(s) => s.clone(),
        DataValue::InternedString(s) => s.to_string(),
        DataValue::Boolean(b) => b.to_string(),
        DataValue::DateTime(dt) => dt.to_string(),
    }
}

/// Convert DataValue to JSON
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
