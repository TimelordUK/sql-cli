//! Data export operations
//!
//! This module contains operations for exporting data to various formats,
//! extracted from the monolithic TUI to improve maintainability and testability.

use crate::data::data_exporter::DataExporter;
use crate::data::data_provider::DataProvider;

/// Context for data export operations
/// Provides the minimal interface needed for export operations without coupling to the full TUI
pub struct DataExportContext<'a> {
    pub data_provider: Option<Box<dyn DataProvider + 'a>>,
}

/// Result of a data export operation
#[derive(Debug)]
pub enum ExportResult {
    Success(String),
    Error(anyhow::Error),
}

impl ExportResult {
    /// Apply the result to a status handler (success message or error)
    pub fn apply_to_status<F, G>(self, set_status: F, set_error: G)
    where
        F: FnOnce(String),
        G: FnOnce(&str, anyhow::Error),
    {
        match self {
            ExportResult::Success(message) => set_status(message),
            ExportResult::Error(e) => set_error("Export failed", e),
        }
    }
}

/// Export data to CSV format
pub fn export_to_csv(ctx: &DataExportContext) -> ExportResult {
    let result = if let Some(ref provider) = ctx.data_provider {
        DataExporter::export_provider_to_csv(provider.as_ref())
    } else {
        Err(anyhow::anyhow!("No data available to export"))
    };

    match result {
        Ok(message) => ExportResult::Success(message),
        Err(e) => ExportResult::Error(e),
    }
}

/// Export data to JSON format
pub fn export_to_json(ctx: &DataExportContext) -> ExportResult {
    let result = if let Some(ref provider) = ctx.data_provider {
        DataExporter::export_provider_to_json(provider.as_ref())
    } else {
        Err(anyhow::anyhow!("No data available to export"))
    };

    match result {
        Ok(message) => ExportResult::Success(message),
        Err(e) => ExportResult::Error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::data_provider::DataProvider;

    // Mock data provider for testing
    #[derive(Debug)]
    struct MockDataProvider {
        should_fail: bool,
    }

    impl DataProvider for MockDataProvider {
        fn get_row(&self, _index: usize) -> Option<Vec<String>> {
            None
        }

        fn get_column_count(&self) -> usize {
            0
        }
        fn get_row_count(&self) -> usize {
            0
        }
        fn get_column_names(&self) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn test_export_csv_no_provider() {
        let ctx = DataExportContext {
            data_provider: None,
        };

        let result = export_to_csv(&ctx);
        match result {
            ExportResult::Error(e) => assert_eq!(e.to_string(), "No data available to export"),
            _ => panic!("Expected error for no provider"),
        }
    }

    #[test]
    fn test_export_json_no_provider() {
        let ctx = DataExportContext {
            data_provider: None,
        };

        let result = export_to_json(&ctx);
        match result {
            ExportResult::Error(e) => assert_eq!(e.to_string(), "No data available to export"),
            _ => panic!("Expected error for no provider"),
        }
    }

    #[test]
    fn test_export_result_apply_success() {
        let result = ExportResult::Success("Export completed".to_string());
        let mut status_set = false;
        let mut error_set = false;

        result.apply_to_status(
            |msg| {
                assert_eq!(msg, "Export completed");
                status_set = true;
            },
            |_prefix, _error| {
                error_set = true;
            },
        );

        assert!(status_set);
        assert!(!error_set);
    }

    #[test]
    fn test_export_result_apply_error() {
        let result = ExportResult::Error(anyhow::anyhow!("Test error"));
        let mut status_set = false;
        let mut error_set = false;

        result.apply_to_status(
            |_msg| {
                status_set = true;
            },
            |prefix, error| {
                assert_eq!(prefix, "Export failed");
                assert_eq!(error.to_string(), "Test error");
                error_set = true;
            },
        );

        assert!(!status_set);
        assert!(error_set);
    }
}
