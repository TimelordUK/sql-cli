use crate::data::data_view::DataView;
use crate::data::query_engine::QueryEngine;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Result of executing a query
pub struct QueryExecutionResult {
    /// The resulting DataView to display
    pub dataview: DataView,

    /// Execution statistics
    pub stats: QueryStats,

    /// Columns that were auto-hidden (if any)
    pub hidden_columns: Vec<String>,

    /// The query that was executed
    pub query: String,
}

/// Statistics about query execution
pub struct QueryStats {
    pub row_count: usize,
    pub column_count: usize,
    pub execution_time: Duration,
    pub query_engine_time: Duration,
}

/// Service responsible for executing queries and managing the resulting DataView
pub struct QueryExecutionService {
    case_insensitive: bool,
    auto_hide_empty: bool,
}

impl QueryExecutionService {
    pub fn new(case_insensitive: bool, auto_hide_empty: bool) -> Self {
        Self {
            case_insensitive,
            auto_hide_empty,
        }
    }

    /// Execute a query and return the result
    /// This encapsulates all the query execution logic that was previously in EnhancedTui
    pub fn execute(
        &self,
        query: &str,
        current_dataview: Option<&DataView>,
    ) -> Result<QueryExecutionResult> {
        // 1. Get the source DataTable
        let source = current_dataview.ok_or_else(|| anyhow::anyhow!("No data loaded"))?;

        // Clone the Arc to the DataTable (cheap - just increments ref count)
        let table_arc = Arc::new(source.source().clone());

        // 2. Execute the query
        let query_start = std::time::Instant::now();
        let engine = QueryEngine::with_case_insensitive(self.case_insensitive);
        let mut new_dataview = engine.execute(table_arc, query)?;
        let query_engine_time = query_start.elapsed();

        // 3. Auto-hide empty columns if configured
        let mut hidden_columns = Vec::new();
        if self.auto_hide_empty {
            let hidden = new_dataview.hide_empty_columns();
            if hidden > 0 {
                info!("Auto-hidden {} empty columns after query execution", hidden);
                // Collect the hidden column names (we'd need to track this in hide_empty_columns)
                // For now, just track the count
                hidden_columns = vec![format!("{} columns", hidden)];
            }
        }

        // 4. Build the result
        let stats = QueryStats {
            row_count: new_dataview.row_count(),
            column_count: new_dataview.column_count(),
            execution_time: query_start.elapsed(),
            query_engine_time,
        };

        Ok(QueryExecutionResult {
            dataview: new_dataview,
            stats,
            hidden_columns,
            query: query.to_string(),
        })
    }

    /// Update configuration
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    pub fn set_auto_hide_empty(&mut self, auto_hide: bool) {
        self.auto_hide_empty = auto_hide;
    }
}

impl QueryExecutionResult {
    /// Generate a user-friendly status message
    pub fn status_message(&self) -> String {
        let hidden_msg = if !self.hidden_columns.is_empty() {
            format!(" ({} auto-hidden)", self.hidden_columns.len())
        } else {
            String::new()
        };

        format!(
            "Query executed: {} rows, {} columns{} ({} ms)",
            self.stats.row_count,
            self.stats.column_count,
            hidden_msg,
            self.stats.execution_time.as_millis()
        )
    }

    /// Get column names for history tracking
    pub fn column_names(&self) -> Vec<String> {
        self.dataview.column_names()
    }

    /// Get table name for history tracking
    pub fn table_name(&self) -> String {
        self.dataview.source().name.clone()
    }
}
