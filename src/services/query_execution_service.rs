use crate::data::data_view::DataView;
use crate::data::query_engine::QueryEngine;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

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
        original_source: Option<&crate::data::datatable::DataTable>,
    ) -> Result<QueryExecutionResult> {
        // Check if query is using DUAL table or has no FROM clause
        use crate::sql::recursive_parser::Parser;
        let mut parser = Parser::new(query);
        let statement = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        let uses_dual = statement
            .from_table
            .as_ref()
            .map(|t| t.to_uppercase() == "DUAL")
            .unwrap_or(false);

        let no_from_clause = statement.from_table.is_none();

        // 1. Get the source DataTable - use DUAL for special cases
        let source_table = if uses_dual || no_from_clause {
            info!("QueryExecutionService: Using DUAL table for expression evaluation");
            crate::data::datatable::DataTable::dual()
        } else if let Some(original) = original_source {
            // Use the original unmodified DataTable for queries
            info!(
                "QueryExecutionService: Using original source with {} columns: {:?}",
                original.column_count(),
                original.column_names()
            );
            debug!(
                "QueryExecutionService: DEBUG - Using original source with {} columns for query",
                original.column_count()
            );
            original.clone()
        } else if let Some(view) = current_dataview {
            // Fallback to current view's source if no original available
            info!(
                "QueryExecutionService: WARNING - No original source, using current view's source with {} columns: {:?}",
                view.source().column_count(),
                view.source().column_names()
            );
            debug!(
                "QueryExecutionService: DEBUG WARNING - No original source, using view source with {} columns",
                view.source().column_count()
            );
            view.source().clone()
        } else {
            return Err(anyhow::anyhow!("No data loaded"));
        };

        // Clone the Arc to the DataTable (cheap - just increments ref count)
        let table_arc = Arc::new(source_table);

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
