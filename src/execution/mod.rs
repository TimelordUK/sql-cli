//! Unified execution module for SQL statement execution
//!
//! This module provides the foundation for unifying script mode and single query mode
//! by offering a consistent execution path that:
//!
//! 1. Parses SQL exactly once
//! 2. Applies preprocessing exactly once (if needed)
//! 3. Executes AST directly (no re-parsing)
//! 4. Manages execution context (temp tables, variables)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         StatementExecutor               │
//! │  (Core execution engine)                │
//! └──────────────┬──────────────────────────┘
//!                │
//!     ┌──────────┼──────────┐
//!     ▼          ▼          ▼
//! ┌────────┐ ┌────────┐ ┌─────────┐
//! │Context │ │Config  │ │Pipeline │
//! │(State) │ │(Flags) │ │(Preproc)│
//! └────────┘ └────────┘ └─────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use sql_cli::execution::{StatementExecutor, ExecutionContext, ExecutionConfig};
//! use sql_cli::sql::recursive_parser::Parser;
//! use std::sync::Arc;
//!
//! // Create execution context with source table
//! let mut context = ExecutionContext::new(source_table);
//!
//! // Create executor with configuration
//! let config = ExecutionConfig::new()
//!     .with_case_insensitive(true)
//!     .with_show_preprocessing(false);
//! let executor = StatementExecutor::with_config(config);
//!
//! // Parse and execute
//! let stmt = Parser::new("SELECT * FROM test").parse()?;
//! let result = executor.execute(stmt, &mut context)?;
//!
//! println!("Rows: {}", result.dataview.row_count());
//! println!("Execution time: {:.2}ms", result.stats.total_time_ms);
//! ```
//!
//! # Phase 0 Status
//!
//! This module is part of Phase 0 of the execution mode unification plan.
//! It provides the foundation without modifying existing code paths.
//!
//! **Completed:**
//! - ExecutionContext - manages temp tables and state
//! - ExecutionConfig - controls preprocessing and behavior
//! - StatementExecutor - core execution engine
//!
//! **Next Steps (Future Phases):**
//! - Phase 1: Refactor script mode to use StatementExecutor
//! - Phase 2: Refactor single query mode to use StatementExecutor
//! - Phase 3: Achieve feature parity
//! - Phase 4: Integrate with TUI
//!
//! See `docs/EXECUTION_MODE_UNIFICATION_PLAN.md` for complete roadmap.

pub mod config;
pub mod context;
pub mod statement_executor;

// Re-export main types for convenience
pub use config::ExecutionConfig;
pub use context::ExecutionContext;
pub use statement_executor::{ExecutionResult, ExecutionStats, StatementExecutor};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
    use crate::sql::recursive_parser::Parser;
    use std::sync::Arc;

    fn create_sample_table() -> DataTable {
        let mut table = DataTable::new("sample");
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("value").with_type(DataType::Integer));

        for i in 1..=10 {
            let _ = table.add_row(DataRow {
                values: vec![DataValue::Integer(i), DataValue::Integer(i * 10)],
            });
        }

        table
    }

    #[test]
    fn test_module_integration() {
        // Test that all components work together
        let table = create_sample_table();
        let mut context = ExecutionContext::new(Arc::new(table));

        let config = ExecutionConfig::new()
            .with_case_insensitive(false)
            .with_auto_hide_empty(false);

        let executor = StatementExecutor::with_config(config);

        // Execute a simple query
        let mut parser = Parser::new("SELECT id, value FROM sample WHERE id <= 5");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 5);
        assert_eq!(result.dataview.column_count(), 2);
        assert!(result.stats.total_time_ms >= 0.0);
    }

    #[test]
    fn test_temp_table_workflow() {
        // Test the complete workflow with temp tables
        let base_table = create_sample_table();
        let mut context = ExecutionContext::new(Arc::new(base_table));
        let executor = StatementExecutor::new();

        // Create a temp table directly (for Phase 0, we're testing the infrastructure)
        let mut temp_table = DataTable::new("#filtered");
        temp_table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        temp_table.add_column(DataColumn::new("value").with_type(DataType::Integer));
        for i in 1..=3 {
            let _ = temp_table.add_row(DataRow {
                values: vec![DataValue::Integer(i), DataValue::Integer(i * 10)],
            });
        }

        // Store as temp table
        context
            .store_temp_table("#filtered".to_string(), Arc::new(temp_table))
            .unwrap();

        // Query the temp table
        let mut parser = Parser::new("SELECT * FROM #filtered");
        let stmt = parser.parse().unwrap();
        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 3);
        assert!(context.has_temp_table("#filtered"));
    }

    #[test]
    fn test_dual_table_usage() {
        // Test queries without FROM clause (use DUAL)
        let table = create_sample_table();
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        let mut parser = Parser::new("SELECT 1+1 as result, 'hello' as greeting");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        assert_eq!(result.dataview.row_count(), 1);
        assert_eq!(result.dataview.column_count(), 2);
        assert!(!result.stats.preprocessing_applied); // No FROM, no preprocessing
    }

    #[test]
    fn test_configuration_propagation() {
        // Verify configuration affects execution
        let table = create_sample_table();
        let mut context = ExecutionContext::new(Arc::new(table));

        // Test with preprocessing disabled
        let config = ExecutionConfig::new().without_preprocessing();
        let executor = StatementExecutor::with_config(config);

        let mut parser = Parser::new("SELECT * FROM sample");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        // Should still work even with preprocessing disabled
        assert_eq!(result.dataview.row_count(), 10);
    }

    #[test]
    fn test_execution_statistics() {
        // Verify statistics are collected correctly
        let table = create_sample_table();
        let mut context = ExecutionContext::new(Arc::new(table));
        let executor = StatementExecutor::new();

        let mut parser = Parser::new("SELECT * FROM sample WHERE value > 50");
        let stmt = parser.parse().unwrap();

        let result = executor.execute(stmt, &mut context).unwrap();

        // Check statistics
        assert_eq!(result.stats.row_count, 5); // values 60, 70, 80, 90, 100
        assert_eq!(result.stats.column_count, 2);
        assert!(result.stats.total_time_ms >= result.stats.preprocessing_time_ms);
        assert!(result.stats.total_time_ms >= result.stats.execution_time_ms);
    }
}
