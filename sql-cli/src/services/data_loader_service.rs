use crate::data::csv_datasource::CsvApiClient;
use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use crate::ui::utils::enhanced_tui_helpers;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Service responsible for loading data from various sources
/// This encapsulates all file loading logic that was previously in the TUI
pub struct DataLoaderService {
    case_insensitive: bool,
}

impl DataLoaderService {
    pub fn new(case_insensitive: bool) -> Self {
        Self { case_insensitive }
    }

    /// Load a file and return a DataView
    /// The TUI doesn't need to know about file types or loading strategies
    pub fn load_file(&self, file_path: &str) -> Result<DataLoadResult> {
        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", file_path))?;

        let raw_table_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = enhanced_tui_helpers::sanitize_table_name(&raw_table_name);

        match extension.to_lowercase().as_str() {
            "csv" => self.load_csv(file_path, &table_name, &raw_table_name),
            "json" => self.load_json(file_path, &table_name, &raw_table_name),
            _ => Err(anyhow::anyhow!(
                "Unsupported file type: {}. Use .csv or .json files.",
                extension
            )),
        }
    }

    /// Load a CSV file
    fn load_csv(
        &self,
        file_path: &str,
        table_name: &str,
        raw_table_name: &str,
    ) -> Result<DataLoadResult> {
        info!("Loading CSV file: {}", file_path);
        let start = std::time::Instant::now();

        // Try advanced loader first (with string interning)
        let datatable = match crate::data::advanced_csv_loader::AdvancedCsvLoader::new()
            .load_csv_optimized(file_path, table_name)
        {
            Ok(dt) => {
                info!("Successfully loaded CSV with advanced optimizations");
                dt
            }
            Err(e) => {
                warn!(
                    "Advanced CSV loader failed: {}, falling back to standard loader",
                    e
                );
                crate::data::datatable_loaders::load_csv_to_datatable(file_path, table_name)?
            }
        };

        self.create_result(
            datatable,
            file_path.to_string(),
            table_name.to_string(),
            raw_table_name.to_string(),
            start.elapsed(),
        )
    }

    /// Load a JSON file
    fn load_json(
        &self,
        file_path: &str,
        table_name: &str,
        raw_table_name: &str,
    ) -> Result<DataLoadResult> {
        info!("Loading JSON file: {}", file_path);
        let start = std::time::Instant::now();

        let datatable =
            crate::data::datatable_loaders::load_json_to_datatable(file_path, table_name)?;

        self.create_result(
            datatable,
            file_path.to_string(),
            table_name.to_string(),
            raw_table_name.to_string(),
            start.elapsed(),
        )
    }

    /// Load data using CsvApiClient (for additional files)
    pub fn load_with_client(&self, file_path: &str) -> Result<DataLoadResult> {
        let mut csv_client = CsvApiClient::new();
        csv_client.set_case_insensitive(self.case_insensitive);

        let path = Path::new(file_path);
        let raw_table_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = enhanced_tui_helpers::sanitize_table_name(&raw_table_name);

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", file_path))?;

        let start = std::time::Instant::now();

        match extension.to_lowercase().as_str() {
            "csv" => csv_client.load_csv(file_path, &table_name)?,
            "json" => csv_client.load_json(file_path, &table_name)?,
            _ => return Err(anyhow::anyhow!("Unsupported file type: {}", extension)),
        }

        // Get the DataTable from the client
        let datatable = csv_client
            .get_datatable()
            .ok_or_else(|| anyhow::anyhow!("Failed to load data from {}", file_path))?;

        self.create_result(
            datatable,
            file_path.to_string(),
            table_name,
            raw_table_name,
            start.elapsed(),
        )
    }

    /// Create a DataLoadResult from a DataTable
    fn create_result(
        &self,
        datatable: DataTable,
        source_path: String,
        table_name: String,
        raw_table_name: String,
        load_time: std::time::Duration,
    ) -> Result<DataLoadResult> {
        // Create initial DataView
        let initial_row_count = datatable.row_count();
        let initial_column_count = datatable.column_count();

        // Create DataView
        let dataview = DataView::new(Arc::new(datatable));

        Ok(DataLoadResult {
            dataview,
            source_path,
            table_name,
            raw_table_name,
            initial_row_count,
            initial_column_count,
            load_time,
        })
    }

    /// Update configuration
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }
}

/// Result of loading data
pub struct DataLoadResult {
    /// The loaded DataView ready for display
    pub dataview: DataView,

    /// Path to the source file
    pub source_path: String,

    /// SQL-friendly table name
    pub table_name: String,

    /// Original table name (before sanitization)
    pub raw_table_name: String,

    /// Initial row count (before any filtering)
    pub initial_row_count: usize,

    /// Initial column count
    pub initial_column_count: usize,

    /// Time taken to load the file
    pub load_time: std::time::Duration,
}

impl DataLoadResult {
    /// Generate a status message for the load operation
    pub fn status_message(&self) -> String {
        format!(
            "Loaded {} ({} rows, {} columns) in {} ms",
            self.source_path,
            self.initial_row_count,
            self.initial_column_count,
            self.load_time.as_millis()
        )
    }
}
