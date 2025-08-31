// Export behavior for saving data to various formats
// Handles CSV, JSON, and other export operations

use crate::data::data_exporter::DataExporter;
use crate::data::data_provider::DataProvider;
use anyhow::Result;

/// Trait for exporting data to various formats
pub trait ExportBehavior {
    // Required methods - provide access to TUI internals
    fn get_data_provider(&self) -> Option<Box<dyn DataProvider>>;

    /// Export current data to CSV file
    fn export_to_csv(&mut self) -> Result<String> {
        if let Some(provider) = self.get_data_provider() {
            DataExporter::export_provider_to_csv(provider.as_ref())
        } else {
            Err(anyhow::anyhow!("No data available to export"))
        }
    }

    /// Export current data to JSON file
    fn export_to_json(&mut self) -> Result<String> {
        if let Some(provider) = self.get_data_provider() {
            DataExporter::export_provider_to_json(provider.as_ref())
        } else {
            Err(anyhow::anyhow!("No data available to export"))
        }
    }

    /// Export current data to a custom format
    fn export_to_format(&mut self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Csv => self.export_to_csv(),
            ExportFormat::Json => self.export_to_json(),
            ExportFormat::Tsv => {
                // TODO: Implement TSV export
                Err(anyhow::anyhow!("TSV export not yet implemented"))
            }
            ExportFormat::Markdown => {
                // TODO: Implement Markdown table export
                Err(anyhow::anyhow!("Markdown export not yet implemented"))
            }
            ExportFormat::Html => {
                // TODO: Implement HTML table export
                Err(anyhow::anyhow!("HTML export not yet implemented"))
            }
            ExportFormat::Sql => {
                // TODO: Implement SQL INSERT statements export
                Err(anyhow::anyhow!("SQL export not yet implemented"))
            }
        }
    }

    /// Export selected rows to CSV
    fn export_selection_to_csv(&mut self) -> Result<String> {
        // TODO: Implement selection-based export
        // This would export only the selected rows/columns
        Err(anyhow::anyhow!("Selection export not yet implemented"))
    }

    /// Export selected rows to JSON
    fn export_selection_to_json(&mut self) -> Result<String> {
        // TODO: Implement selection-based export
        Err(anyhow::anyhow!("Selection export not yet implemented"))
    }

    /// Get export filename with timestamp
    fn get_export_filename(&self, extension: &str) -> String {
        use chrono::Local;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("export_{}.{}", timestamp, extension)
    }

    /// Handle export result with status message
    fn handle_export_result(&mut self, result: Result<String>);
}

/// Export format options
pub enum ExportFormat {
    Csv,
    Json,
    Tsv,
    Markdown,
    Html,
    Sql,
}
