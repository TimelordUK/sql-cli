use crate::services::{DataLoaderService, QueryOrchestrator};
use crate::ui::enhanced_tui::EnhancedTuiApp;
use anyhow::Result;
use tracing::info;

/// Orchestrates the entire application lifecycle
/// This removes file loading knowledge from the TUI
pub struct ApplicationOrchestrator {
    data_loader: DataLoaderService,
    query_orchestrator: QueryOrchestrator,
}

impl ApplicationOrchestrator {
    pub fn new(case_insensitive: bool, auto_hide_empty: bool) -> Self {
        Self {
            data_loader: DataLoaderService::new(case_insensitive),
            query_orchestrator: QueryOrchestrator::new(case_insensitive, auto_hide_empty),
        }
    }

    /// Create a TUI with an initial file loaded
    /// The TUI doesn't need to know about file types or loading
    pub fn create_tui_with_file(&self, file_path: &str) -> Result<EnhancedTuiApp> {
        info!("Creating TUI with file: {}", file_path);

        // Load the file using the data loader service
        let load_result = self.data_loader.load_file(file_path)?;

        // Get the status message before moving dataview
        let status_message = load_result.status_message();
        let source_path = load_result.source_path.clone();
        let table_name = load_result.table_name.clone();
        let raw_table_name = load_result.raw_table_name.clone();

        // Create the TUI with just a DataView
        let mut app = EnhancedTuiApp::new_with_dataview(load_result.dataview, &source_path)?;

        // Set the initial status message
        app.set_status_message(status_message);

        // Pre-populate the SQL command with SELECT * FROM table
        app.set_sql_query(&table_name, &raw_table_name);

        Ok(app)
    }

    /// Load additional files into existing TUI
    pub fn load_additional_file(&self, app: &mut EnhancedTuiApp, file_path: &str) -> Result<()> {
        info!("Loading additional file: {}", file_path);

        // Load the file
        let load_result = self.data_loader.load_file(file_path)?;

        // Get the status message before moving dataview
        let status_message = load_result.status_message();
        let source_path = load_result.source_path.clone();
        let table_name = load_result.table_name.clone();
        let raw_table_name = load_result.raw_table_name.clone();

        // Add to the TUI (it will create a new buffer)
        app.add_dataview(load_result.dataview, &source_path)?;

        // Set status message
        app.set_status_message(status_message);

        // Pre-populate the SQL command with SELECT * FROM table for new buffer
        app.set_sql_query(&table_name, &raw_table_name);

        Ok(())
    }

    /// Execute a query in the TUI
    /// Note: This is a simplified version - the TUI itself should use execute_query_v2
    /// which properly handles the closures for apply_to_tui
    pub fn execute_query(&mut self, app: &mut EnhancedTuiApp, query: &str) -> Result<()> {
        // For now, just delegate to the TUI's own execute_query_v2 method
        // This avoids the borrowing issues with closures
        app.execute_query_v2(query)
    }
}

/// Builder pattern for creating the orchestrator with configuration
pub struct ApplicationOrchestratorBuilder {
    case_insensitive: bool,
    auto_hide_empty: bool,
}

impl ApplicationOrchestratorBuilder {
    pub fn new() -> Self {
        Self {
            case_insensitive: false,
            auto_hide_empty: false,
        }
    }

    pub fn with_case_insensitive(mut self, value: bool) -> Self {
        self.case_insensitive = value;
        self
    }

    pub fn with_auto_hide_empty(mut self, value: bool) -> Self {
        self.auto_hide_empty = value;
        self
    }

    pub fn build(self) -> ApplicationOrchestrator {
        ApplicationOrchestrator::new(self.case_insensitive, self.auto_hide_empty)
    }
}
