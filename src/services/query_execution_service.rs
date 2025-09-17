use crate::config::config::BehaviorConfig;
use crate::data::data_view::DataView;
use crate::data::query_engine::QueryEngine;
use crate::execution_plan::ExecutionPlan;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Result of executing a query
pub struct QueryExecutionResult {
    /// The resulting `DataView` to display
    pub dataview: DataView,

    /// Execution statistics
    pub stats: QueryStats,

    /// Columns that were auto-hidden (if any)
    pub hidden_columns: Vec<String>,

    /// The query that was executed
    pub query: String,

    /// The execution plan (if tracked)
    pub execution_plan: Option<ExecutionPlan>,
}

/// Statistics about query execution
pub struct QueryStats {
    pub row_count: usize,
    pub column_count: usize,
    pub execution_time: Duration,
    pub query_engine_time: Duration,
}

/// Service responsible for executing queries and managing the resulting `DataView`
pub struct QueryExecutionService {
    case_insensitive: bool,
    auto_hide_empty: bool,
    date_notation: String,
    behavior_config: Option<BehaviorConfig>,
}

impl QueryExecutionService {
    #[must_use]
    pub fn new(case_insensitive: bool, auto_hide_empty: bool) -> Self {
        Self {
            case_insensitive,
            auto_hide_empty,
            date_notation: "us".to_string(),
            behavior_config: None,
        }
    }

    #[must_use]
    pub fn with_behavior_config(behavior_config: BehaviorConfig) -> Self {
        let case_insensitive = behavior_config.case_insensitive_default;
        let auto_hide_empty = behavior_config.hide_empty_columns;
        let date_notation = behavior_config.default_date_notation.clone();
        Self {
            case_insensitive,
            auto_hide_empty,
            date_notation,
            behavior_config: Some(behavior_config),
        }
    }

    #[must_use]
    pub fn with_date_notation(
        case_insensitive: bool,
        auto_hide_empty: bool,
        date_notation: String,
    ) -> Self {
        Self {
            case_insensitive,
            auto_hide_empty,
            date_notation,
            behavior_config: None,
        }
    }

    /// Execute a query and return the result
    /// This encapsulates all the query execution logic that was previously in `EnhancedTui`
    pub fn execute(
        &self,
        query: &str,
        current_dataview: Option<&DataView>,
        original_source: Option<&crate::data::datatable::DataTable>,
    ) -> Result<QueryExecutionResult> {
        // Check if query is using DUAL table or has no FROM clause
        use crate::query_plan::{CTEHoister, ExpressionLifter};
        use crate::sql::recursive_parser::Parser;

        let mut parser = Parser::new(query);
        let mut statement = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        // Apply expression lifting for column alias dependencies
        let mut expr_lifter = ExpressionLifter::new();
        let lifted = expr_lifter.lift_expressions(&mut statement);
        if !lifted.is_empty() {
            info!(
                "Applied expression lifting - {} CTEs generated",
                lifted.len()
            );
        }

        // Apply CTE hoisting to handle nested CTEs
        statement = CTEHoister::hoist_ctes(statement);

        let uses_dual = statement
            .from_table
            .as_ref()
            .is_some_and(|t| t.to_uppercase() == "DUAL");

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
        let engine = if let Some(ref config) = self.behavior_config {
            QueryEngine::with_behavior_config(config.clone())
        } else {
            QueryEngine::with_case_insensitive_and_date_notation(
                self.case_insensitive,
                self.date_notation.clone(),
            )
        };
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
            execution_plan: None,
        })
    }

    /// Update configuration
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    pub fn set_auto_hide_empty(&mut self, auto_hide: bool) {
        self.auto_hide_empty = auto_hide;
    }

    pub fn set_date_notation(&mut self, date_notation: String) {
        self.date_notation = date_notation;
    }
}

impl QueryExecutionResult {
    /// Generate a user-friendly status message
    #[must_use]
    pub fn status_message(&self) -> String {
        let hidden_msg = if self.hidden_columns.is_empty() {
            String::new()
        } else {
            format!(" ({} auto-hidden)", self.hidden_columns.len())
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
    #[must_use]
    pub fn column_names(&self) -> Vec<String> {
        self.dataview.column_names()
    }

    /// Get table name for history tracking
    #[must_use]
    pub fn table_name(&self) -> String {
        self.dataview.source().name.clone()
    }
}
