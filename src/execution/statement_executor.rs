//! Core statement executor - unified execution path for all modes
//!
//! This module provides the single source of truth for executing SQL statements.
//! Both script mode and single query mode should use this executor to ensure
//! consistent behavior and eliminate code duplication.

use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use crate::data::query_engine::QueryEngine;
use crate::query_plan::{create_pipeline_with_config, IntoClauseRemover};
use crate::sql::parser::ast::SelectStatement;

use super::config::ExecutionConfig;
use super::context::ExecutionContext;

/// Result of executing a SQL statement
#[derive(Debug)]
pub struct ExecutionResult {
    /// The resulting DataView from query execution
    pub dataview: DataView,

    /// Execution statistics
    pub stats: ExecutionStats,

    /// The transformed AST (after preprocessing)
    pub transformed_ast: Option<SelectStatement>,
}

/// Statistics about query execution
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Time spent preprocessing (transforming AST)
    pub preprocessing_time_ms: f64,

    /// Time spent executing the query
    pub execution_time_ms: f64,

    /// Total time (preprocessing + execution)
    pub total_time_ms: f64,

    /// Number of rows in result
    pub row_count: usize,

    /// Number of columns in result
    pub column_count: usize,

    /// Whether preprocessing was applied
    pub preprocessing_applied: bool,
}

impl ExecutionStats {
    fn new() -> Self {
        Self {
            preprocessing_time_ms: 0.0,
            execution_time_ms: 0.0,
            total_time_ms: 0.0,
            row_count: 0,
            column_count: 0,
            preprocessing_applied: false,
        }
    }
}

/// Core statement executor
///
/// This is the unified execution engine used by both script mode and single query mode.
/// It ensures consistent behavior by:
/// 1. Parsing SQL exactly once
/// 2. Applying preprocessing pipeline exactly once (if needed)
/// 3. Executing the AST directly (no re-parsing)
/// 4. Managing temp tables and context properly
pub struct StatementExecutor {
    config: ExecutionConfig,
}

impl StatementExecutor {
    /// Create a new statement executor with default configuration
    pub fn new() -> Self {
        Self {
            config: ExecutionConfig::default(),
        }
    }

    /// Create executor with custom configuration
    pub fn with_config(config: ExecutionConfig) -> Self {
        Self { config }
    }

    /// Execute a single SQL statement that has already been parsed
    ///
    /// # Arguments
    /// * `stmt` - The parsed SQL statement (AST)
    /// * `context` - Execution context (temp tables, variables, etc.)
    ///
    /// # Returns
    /// ExecutionResult containing the DataView and statistics
    ///
    /// # Example
    /// ```ignore
    /// let executor = StatementExecutor::new();
    /// let mut context = ExecutionContext::new(source_table);
    /// let stmt = Parser::new("SELECT * FROM test").parse()?;
    /// let result = executor.execute(&stmt, &mut context)?;
    /// ```
    pub fn execute(
        &self,
        stmt: SelectStatement,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionResult> {
        let total_start = Instant::now();
        let mut stats = ExecutionStats::new();

        // Step 0: Check if this statement has an INTO clause (before preprocessing removes it!)
        let into_table_name = stmt.into_table.as_ref().map(|it| it.name.clone());

        // Step 1: Determine source table
        // For most queries, this is straightforward. For derived tables and PIVOT,
        // we need to find the base table referenced by the innermost query.
        let source_table = if let Some(ref from_source) = stmt.from_source {
            match from_source {
                crate::sql::parser::ast::TableSource::Table(table_name) => {
                    context.resolve_table(table_name)
                }
                crate::sql::parser::ast::TableSource::DerivedTable { query, .. } => {
                    // For derived tables, find the base table from the inner query
                    Self::extract_base_table(&**query, context)
                }
                crate::sql::parser::ast::TableSource::Pivot { source, .. } => {
                    // For PIVOT (though it should be expanded), extract from source
                    Self::extract_base_table_from_source(source, context)
                }
            }
        } else {
            // Fallback to deprecated field for backward compatibility
            #[allow(deprecated)]
            if let Some(ref from_table) = stmt.from_table {
                context.resolve_table(from_table)
            } else {
                // No FROM clause - use DUAL table for expression evaluation
                Arc::new(DataTable::dual())
            }
        };

        // Step 2: Apply preprocessing pipeline (if applicable)
        let preprocess_start = Instant::now();
        let (transformed_stmt, preprocessing_applied) = self.apply_preprocessing(stmt)?;
        stats.preprocessing_time_ms = preprocess_start.elapsed().as_secs_f64() * 1000.0;
        stats.preprocessing_applied = preprocessing_applied;

        // Step 3: Execute the transformed statement directly via QueryEngine
        let exec_start = Instant::now();
        let result_view = self.execute_ast(transformed_stmt.clone(), source_table, context)?;
        stats.execution_time_ms = exec_start.elapsed().as_secs_f64() * 1000.0;

        // Step 4: If this was a SELECT INTO statement, store the result as a temp table
        if let Some(table_name) = into_table_name {
            // Materialize the view into a DataTable using QueryEngine's method
            let engine = QueryEngine::with_case_insensitive(self.config.case_insensitive);
            let temp_table = engine.materialize_view(result_view.clone())?;

            // Store in temp table registry
            context.store_temp_table(table_name.clone(), Arc::new(temp_table))?;
            tracing::debug!("Stored temp table: {}", table_name);
        }

        // Step 5: Collect statistics
        stats.total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;
        stats.row_count = result_view.row_count();
        stats.column_count = result_view.column_count();

        Ok(ExecutionResult {
            dataview: result_view,
            stats,
            transformed_ast: Some(transformed_stmt),
        })
    }

    /// Apply preprocessing pipeline to a statement
    ///
    /// Returns (transformed_statement, preprocessing_applied)
    fn apply_preprocessing(&self, mut stmt: SelectStatement) -> Result<(SelectStatement, bool)> {
        // Check if statement has a FROM clause - only preprocess if it does
        // (queries without FROM have special semantics in this tool)
        let has_from_clause = if stmt.from_source.is_some() {
            true
        } else {
            // Fallback to deprecated fields
            #[allow(deprecated)]
            {
                stmt.from_table.is_some()
                    || stmt.from_subquery.is_some()
                    || stmt.from_function.is_some()
            }
        };

        if !has_from_clause {
            // No preprocessing for queries without FROM
            return Ok((stmt, false));
        }

        // Create preprocessing pipeline with configured transformers
        let mut pipeline = create_pipeline_with_config(
            self.config.show_preprocessing,
            self.config.show_sql_transformations,
            self.config.transformer_config.clone(),
        );

        // Apply transformations
        match pipeline.process(stmt.clone()) {
            Ok(transformed) => {
                // Remove INTO clause if present (executor doesn't handle INTO syntax)
                let final_stmt = if transformed.into_table.is_some() {
                    IntoClauseRemover::remove_into_clause(transformed)
                } else {
                    transformed
                };

                Ok((final_stmt, true))
            }
            Err(e) => {
                // If preprocessing fails, fall back to original statement
                tracing::debug!("Preprocessing failed: {}, using original statement", e);

                // Still remove INTO clause even on fallback
                let fallback = if stmt.into_table.is_some() {
                    IntoClauseRemover::remove_into_clause(stmt)
                } else {
                    stmt
                };

                Ok((fallback, false))
            }
        }
    }

    /// Execute an AST directly using QueryEngine
    ///
    /// This is the core execution method - it takes a parsed/transformed AST
    /// and executes it directly without re-parsing.
    fn execute_ast(
        &self,
        stmt: SelectStatement,
        source_table: Arc<DataTable>,
        context: &ExecutionContext,
    ) -> Result<DataView> {
        // Create QueryEngine with case sensitivity setting
        let engine = QueryEngine::with_case_insensitive(self.config.case_insensitive);

        // Execute the statement with temp table support
        // This is the key method that does the actual work
        engine.execute_statement_with_temp_tables(source_table, stmt, Some(&context.temp_tables))
    }

    /// Get the current configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ExecutionConfig) {
        self.config = config;
    }

    /// Extract the base table from a SelectStatement
    /// Recursively traverses derived tables to find the underlying table
    fn extract_base_table(stmt: &SelectStatement, context: &ExecutionContext) -> Arc<DataTable> {
        if let Some(ref from_source) = stmt.from_source {
            Self::extract_base_table_from_source(from_source, context)
        } else {
            // Fallback to deprecated fields
            #[allow(deprecated)]
            if let Some(ref from_table) = stmt.from_table {
                context.resolve_table(from_table)
            } else {
                Arc::new(DataTable::dual())
            }
        }
    }

    /// Extract base table from a TableSource
    fn extract_base_table_from_source(
        source: &crate::sql::parser::ast::TableSource,
        context: &ExecutionContext,
    ) -> Arc<DataTable> {
        match source {
            crate::sql::parser::ast::TableSource::Table(table_name) => {
                context.resolve_table(table_name)
            }
            crate::sql::parser::ast::TableSource::DerivedTable { query, .. } => {
                // Recursively extract from nested derived table
                Self::extract_base_table(&**query, context)
            }
            crate::sql::parser::ast::TableSource::Pivot { source, .. } => {
                // Extract from PIVOT source
                Self::extract_base_table_from_source(&**source, context)
            }
        }
    }
}

impl Default for StatementExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataType, DataValue};
    use crate::sql::recursive_parser::Parser;

    fn create_test_table(name: &str, rows: usize) -> DataTable {
        let mut table = DataTable::new(name);
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("name").with_type(DataType::String));

        for i in 0..rows {
            let _ = table.add_row(DataRow {
                values: vec![
                    DataValue::Integer(i as i64),
                    DataValue::String(format!("name_{}", i)),
                ],
            });
        }

        table
    }

    #[test]
    fn test_new_executor() {
        let executor = StatementExecutor::new();
        assert!(!executor.config().case_insensitive);
        assert!(!executor.config().show_preprocessing);
    }

    #[test]
    fn test_executor_with_config() {
        let config = ExecutionConfig::new()
            .with_case_insensitive(true)
            .with_show_preprocessing(true);

        let executor = StatementExecutor::with_config(config);
        assert!(executor.config().case_insensitive);
        assert!(executor.config().show_preprocessing);
    }

    #[test]
    fn test_execute_simple_select() {
        let table = create_test_table("test", 10);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        // Parse and execute a simple SELECT
        let mut parser = Parser::new("SELECT id, name FROM test WHERE id < 5");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 5);
        assert_eq!(result.dataview.column_count(), 2);
        assert!(result.stats.total_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_select_star() {
        let table = create_test_table("test", 5);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        let mut parser = Parser::new("SELECT * FROM test");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 5);
        assert_eq!(result.dataview.column_count(), 2);
    }

    #[test]
    fn test_execute_with_dual() {
        let table = create_test_table("test", 5);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        // Query without FROM - should use DUAL
        let mut parser = Parser::new("SELECT 1+1 as result");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 1);
        assert_eq!(result.dataview.column_count(), 1);
    }

    #[test]
    fn test_execute_with_temp_table() {
        let base_table = create_test_table("base", 10);
        let mut context = ExecutionContext::new(Arc::new(base_table));
        let executor = StatementExecutor::new();

        // Create and store a temp table
        let temp_table = create_test_table("#temp", 3);
        context
            .store_temp_table("#temp".to_string(), Arc::new(temp_table))
            .unwrap();

        // Query the temp table
        let mut parser = Parser::new("SELECT * FROM #temp");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 3);
    }

    #[test]
    fn test_preprocessing_applied_with_from() {
        let table = create_test_table("test", 10);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        // Query with FROM - preprocessing should be attempted
        let mut parser = Parser::new("SELECT id FROM test WHERE id > 0");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        // Preprocessing should have been attempted (may or may not transform anything)
        assert!(result.stats.preprocessing_time_ms >= 0.0);
    }

    #[test]
    fn test_no_preprocessing_without_from() {
        let table = create_test_table("test", 10);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        // Query without FROM - no preprocessing
        let mut parser = Parser::new("SELECT 42 as answer");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        // No preprocessing should have been applied
        assert!(!result.stats.preprocessing_applied);
    }

    #[test]
    fn test_execution_stats() {
        let table = create_test_table("test", 100);
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        let mut parser = Parser::new("SELECT * FROM test WHERE id < 50");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        let stats = result.stats;
        assert_eq!(stats.row_count, 50);
        assert_eq!(stats.column_count, 2);
        assert!(stats.total_time_ms >= 0.0);
        assert!(stats.total_time_ms >= stats.preprocessing_time_ms);
        assert!(stats.total_time_ms >= stats.execution_time_ms);
    }

    #[test]
    fn test_case_insensitive_execution() {
        let table = create_test_table("test", 10);
        let mut context = ExecutionContext::new(Arc::new(table));

        let config = ExecutionConfig::new().with_case_insensitive(true);
        let executor = StatementExecutor::with_config(config);

        // Use uppercase column name - should work with case insensitive
        let mut parser = Parser::new("SELECT ID FROM test");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context);

        // Should succeed with case insensitive mode
        assert!(result.is_ok());
    }
}
