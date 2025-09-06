use anyhow::Result;
use clap::{Arg, Command};
use sql_cli::{
    chart::types::{ChartConfig, ChartType},
    chart::{ChartEngine, ChartTui},
    data::data_view::DataView,
    data::datatable_loaders::{load_csv_to_datatable, load_json_to_datatable},
    data::query_engine::QueryEngine,
};
use std::path::Path;

fn main() -> Result<()> {
    let matches = Command::new("sql-cli-chart")
        .version("1.34.0")
        .about("Standalone charting tool for CSV/JSON data with SQL queries")
        .arg(
            Arg::new("file")
                .help("Input CSV or JSON file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .value_name("SQL")
                .help("SQL query to filter/transform data")
                .required(true),
        )
        .arg(
            Arg::new("x-axis")
                .short('x')
                .long("x-axis")
                .value_name("COLUMN")
                .help("Column for X-axis")
                .required(true),
        )
        .arg(
            Arg::new("y-axis")
                .short('y')
                .long("y-axis")
                .value_name("COLUMN")
                .help("Column for Y-axis")
                .required(true),
        )
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .help("Chart title")
                .default_value("SQL CLI Chart"),
        )
        .arg(
            Arg::new("chart-type")
                .short('c')
                .long("chart-type")
                .value_name("TYPE")
                .help("Chart type (line, scatter, bar)")
                .default_value("line"),
        )
        .get_matches();

    // Parse arguments
    let file_path = matches.get_one::<String>("file").unwrap();
    let query = matches.get_one::<String>("query").unwrap();
    let x_axis = matches.get_one::<String>("x-axis").unwrap();
    let y_axis = matches.get_one::<String>("y-axis").unwrap();
    let title = matches.get_one::<String>("title").unwrap();
    let chart_type_str = matches.get_one::<String>("chart-type").unwrap();

    // Parse chart type
    let chart_type = match chart_type_str.to_lowercase().as_str() {
        "line" => ChartType::Line,
        "scatter" => ChartType::Scatter,
        "bar" => ChartType::Bar,
        "candlestick" => ChartType::Candlestick,
        _ => {
            eprintln!(
                "Invalid chart type '{chart_type_str}'. Supported types: line, scatter, bar, candlestick"
            );
            std::process::exit(1);
        }
    };

    // Load data
    println!("Loading data from '{file_path}'...");
    let data_view = load_data_file(file_path)?;
    println!(
        "Loaded {} rows, {} columns",
        data_view.row_count(),
        data_view.column_count()
    );

    // Execute the SQL query to get filtered data
    println!("Executing query: {query}");
    let query_engine = QueryEngine::new();
    let filtered_view =
        query_engine.execute(std::sync::Arc::new(data_view.source().clone()), query)?;
    println!(
        "Query result: {} rows, {} columns",
        filtered_view.row_count(),
        filtered_view.column_count()
    );

    // Create chart configuration (query already executed)
    let config = ChartConfig {
        title: title.clone(),
        x_axis: x_axis.clone(),
        y_axis: y_axis.clone(),
        chart_type,
        query: query.clone(),
    };

    // Create chart engine with filtered data
    let chart_engine = ChartEngine::new(filtered_view);
    let mut chart_tui = ChartTui::new(config);

    println!("Starting chart TUI... Press 'q' to quit");

    // Run the chart TUI
    chart_tui.run(chart_engine)?;

    println!("Chart closed.");
    Ok(())
}

fn load_data_file(file_path: &str) -> Result<DataView> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow::anyhow!("File '{}' does not exist", file_path));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let data_table = match extension.as_str() {
        "csv" => load_csv_to_datatable(file_path, "chart_data")?,
        "json" => load_json_to_datatable(file_path, "chart_data")?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported file format '{}'. Supported formats: csv, json",
                extension
            ));
        }
    };

    // Create DataView from DataTable
    Ok(DataView::new(std::sync::Arc::new(data_table)))
}
