//! Execution context for managing state during query execution
//!
//! This module provides a unified context for both script mode and single query mode,
//! managing temp tables, variables, and source table resolution.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::datatable::DataTable;
use crate::data::temp_table_registry::TempTableRegistry;

/// Context for statement execution
///
/// Holds all state needed to execute SQL statements including:
/// - Base source table(s)
/// - Temporary tables (#table syntax)
/// - Variables (for future use)
/// - Execution configuration
#[derive(Clone)]
pub struct ExecutionContext {
    /// The primary source table (typically from CSV/JSON file)
    pub source_table: Arc<DataTable>,

    /// Registry of temporary tables created during script execution
    pub temp_tables: TempTableRegistry,

    /// Variables for template expansion and substitution
    pub variables: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create a new execution context with a source table
    pub fn new(source_table: Arc<DataTable>) -> Self {
        Self {
            source_table,
            temp_tables: TempTableRegistry::new(),
            variables: HashMap::new(),
        }
    }

    /// Create a context with DUAL table (for queries without FROM clause)
    pub fn with_dual() -> Self {
        Self::new(Arc::new(DataTable::dual()))
    }

    /// Resolve a table name to an Arc<DataTable>
    ///
    /// # Arguments
    /// * `name` - Table name, which may be:
    ///   - A temp table (starts with '#')
    ///   - The base table name
    ///   - "DUAL" for the special DUAL table
    ///
    /// # Returns
    /// Arc<DataTable> for the requested table, or the source table as fallback
    pub fn resolve_table(&self, name: &str) -> Arc<DataTable> {
        if name.starts_with('#') {
            // Temporary table lookup
            self.temp_tables
                .get(name)
                .unwrap_or_else(|| self.source_table.clone())
        } else if name.eq_ignore_ascii_case("DUAL") {
            // Special DUAL table
            Arc::new(DataTable::dual())
        } else {
            // Base source table
            self.source_table.clone()
        }
    }

    /// Try to resolve a table, returning an error if temp table not found
    pub fn resolve_table_strict(&self, name: &str) -> Result<Arc<DataTable>> {
        if name.starts_with('#') {
            self.temp_tables
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("Temporary table '{}' not found", name))
        } else if name.eq_ignore_ascii_case("DUAL") {
            Ok(Arc::new(DataTable::dual()))
        } else {
            Ok(self.source_table.clone())
        }
    }

    /// Store a result as a temporary table
    pub fn store_temp_table(&mut self, name: String, table: Arc<DataTable>) -> Result<()> {
        self.temp_tables.insert(name, table)
    }

    /// Check if a temporary table exists
    pub fn has_temp_table(&self, name: &str) -> bool {
        self.temp_tables.contains(name)
    }

    /// Get all temporary table names
    pub fn temp_table_names(&self) -> Vec<String> {
        self.temp_tables.list_tables()
    }

    /// Set a variable for template expansion
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Clear all temporary tables (useful between script executions)
    pub fn clear_temp_tables(&mut self) {
        self.temp_tables = TempTableRegistry::new();
    }

    /// Clear all variables
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Get source table metadata
    pub fn source_table_info(&self) -> (String, usize, usize) {
        (
            self.source_table.name.clone(),
            self.source_table.row_count(),
            self.source_table.column_count(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table(name: &str, rows: usize) -> DataTable {
        let mut table = DataTable::new(name);
        table.add_column(
            crate::data::datatable::DataColumn::new("id")
                .with_type(crate::data::datatable::DataType::Integer),
        );

        for i in 0..rows {
            let _ = table.add_row(crate::data::datatable::DataRow {
                values: vec![crate::data::datatable::DataValue::Integer(i as i64)],
            });
        }

        table
    }

    #[test]
    fn test_new_context() {
        let table = create_test_table("test", 10);
        let ctx = ExecutionContext::new(Arc::new(table));

        assert_eq!(ctx.source_table.name, "test");
        assert_eq!(ctx.source_table.row_count(), 10);
        assert_eq!(ctx.temp_tables.list_tables().len(), 0);
    }

    #[test]
    fn test_dual_context() {
        let ctx = ExecutionContext::with_dual();
        assert_eq!(ctx.source_table.name, "DUAL");
        assert_eq!(ctx.source_table.row_count(), 1);
    }

    #[test]
    fn test_resolve_source_table() {
        let table = create_test_table("customers", 5);
        let ctx = ExecutionContext::new(Arc::new(table));

        let resolved = ctx.resolve_table("customers");
        assert_eq!(resolved.name, "customers");
        assert_eq!(resolved.row_count(), 5);
    }

    #[test]
    fn test_resolve_dual_table() {
        let table = create_test_table("test", 10);
        let ctx = ExecutionContext::new(Arc::new(table));

        let resolved = ctx.resolve_table("DUAL");
        assert_eq!(resolved.name, "DUAL");
        assert_eq!(resolved.row_count(), 1);
    }

    #[test]
    fn test_store_and_resolve_temp_table() {
        let base_table = create_test_table("base", 10);
        let mut ctx = ExecutionContext::new(Arc::new(base_table));

        // Store a temp table
        let temp_table = create_test_table("#temp1", 5);
        ctx.store_temp_table("#temp1".to_string(), Arc::new(temp_table))
            .unwrap();

        // Verify it exists
        assert!(ctx.has_temp_table("#temp1"));
        assert_eq!(ctx.temp_table_names(), vec!["#temp1"]);

        // Resolve it
        let resolved = ctx.resolve_table("#temp1");
        assert_eq!(resolved.name, "#temp1");
        assert_eq!(resolved.row_count(), 5);
    }

    #[test]
    fn test_resolve_missing_temp_table_fallback() {
        let base_table = create_test_table("base", 10);
        let ctx = ExecutionContext::new(Arc::new(base_table));

        // Should fall back to source table
        let resolved = ctx.resolve_table("#nonexistent");
        assert_eq!(resolved.name, "base");
    }

    #[test]
    fn test_resolve_missing_temp_table_strict() {
        let base_table = create_test_table("base", 10);
        let ctx = ExecutionContext::new(Arc::new(base_table));

        // Should error
        let result = ctx.resolve_table_strict("#nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_variables() {
        let table = create_test_table("test", 5);
        let mut ctx = ExecutionContext::new(Arc::new(table));

        // Set variables
        ctx.set_variable("user_id".to_string(), "123".to_string());
        ctx.set_variable("dept".to_string(), "sales".to_string());

        // Get variables
        assert_eq!(ctx.get_variable("user_id"), Some(&"123".to_string()));
        assert_eq!(ctx.get_variable("dept"), Some(&"sales".to_string()));
        assert_eq!(ctx.get_variable("nonexistent"), None);

        // Clear variables
        ctx.clear_variables();
        assert_eq!(ctx.get_variable("user_id"), None);
    }

    #[test]
    fn test_clear_temp_tables() {
        let base_table = create_test_table("base", 10);
        let mut ctx = ExecutionContext::new(Arc::new(base_table));

        // Add temp tables
        ctx.store_temp_table(
            "#temp1".to_string(),
            Arc::new(create_test_table("#temp1", 5)),
        )
        .unwrap();
        ctx.store_temp_table(
            "#temp2".to_string(),
            Arc::new(create_test_table("#temp2", 3)),
        )
        .unwrap();

        assert_eq!(ctx.temp_table_names().len(), 2);

        // Clear
        ctx.clear_temp_tables();
        assert_eq!(ctx.temp_table_names().len(), 0);
    }

    #[test]
    fn test_source_table_info() {
        let table = create_test_table("sales", 100);
        let ctx = ExecutionContext::new(Arc::new(table));

        let (name, rows, cols) = ctx.source_table_info();
        assert_eq!(name, "sales");
        assert_eq!(rows, 100);
        assert_eq!(cols, 1); // Just 'id' column
    }
}
