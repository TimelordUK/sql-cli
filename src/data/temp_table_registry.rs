/// Temporary table registry for script execution
/// Manages temporary tables (prefixed with #) that persist across query batches
/// within a single script execution. Tables are dropped when the script completes.
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::data::datatable::DataTable;

/// Registry for temporary tables that persist across GO-separated batches
/// within a single script execution
#[derive(Debug, Clone)]
pub struct TempTableRegistry {
    tables: HashMap<String, Arc<DataTable>>,
}

impl TempTableRegistry {
    /// Create a new empty temp table registry
    pub fn new() -> Self {
        debug!("Creating new TempTableRegistry");
        Self {
            tables: HashMap::new(),
        }
    }

    /// Store a temporary table
    /// Returns error if table already exists (to prevent accidental overwrites)
    pub fn insert(&mut self, name: String, table: Arc<DataTable>) -> Result<()> {
        if !name.starts_with('#') {
            return Err(anyhow!(
                "Temporary table name must start with #, got: {}",
                name
            ));
        }

        if self.tables.contains_key(&name) {
            return Err(anyhow!(
                "Temporary table {} already exists. Drop it first or use a different name.",
                name
            ));
        }

        let row_count = table.row_count();
        let col_count = table.column_count();

        info!(
            "Storing temporary table {} ({} rows, {} columns)",
            name, row_count, col_count
        );

        self.tables.insert(name, table);
        Ok(())
    }

    /// Store a temporary table, replacing if it already exists
    pub fn insert_or_replace(&mut self, name: String, table: Arc<DataTable>) -> Result<()> {
        if !name.starts_with('#') {
            return Err(anyhow!(
                "Temporary table name must start with #, got: {}",
                name
            ));
        }

        let row_count = table.row_count();
        let col_count = table.column_count();

        if self.tables.contains_key(&name) {
            info!(
                "Replacing temporary table {} ({} rows, {} columns)",
                name, row_count, col_count
            );
        } else {
            info!(
                "Storing temporary table {} ({} rows, {} columns)",
                name, row_count, col_count
            );
        }

        self.tables.insert(name, table);
        Ok(())
    }

    /// Get a temporary table by name
    /// Returns None if table doesn't exist
    pub fn get(&self, name: &str) -> Option<Arc<DataTable>> {
        debug!("Looking up temporary table: {}", name);
        self.tables.get(name).cloned()
    }

    /// Check if a temporary table exists
    pub fn contains(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Drop a temporary table
    /// Returns true if table existed and was dropped
    pub fn drop(&mut self, name: &str) -> bool {
        if let Some(_table) = self.tables.remove(name) {
            info!("Dropped temporary table: {}", name);
            true
        } else {
            debug!("Temporary table {} does not exist, cannot drop", name);
            false
        }
    }

    /// Get count of temp tables
    pub fn count(&self) -> usize {
        self.tables.len()
    }

    /// Get all temp table names
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Clear all temp tables
    /// Used when script execution completes
    pub fn clear(&mut self) {
        let count = self.tables.len();
        self.tables.clear();
        if count > 0 {
            info!("Cleared {} temporary tables", count);
        }
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }
}

impl Default for TempTableRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataValue};

    fn create_test_table(rows: usize) -> DataTable {
        let mut table = DataTable::new("test_table");

        table.add_column(DataColumn::new("id".to_string()));
        table.add_column(DataColumn::new("name".to_string()));

        for i in 0..rows {
            table.add_row(DataRow {
                values: vec![
                    DataValue::Integer(i as i64),
                    DataValue::String(format!("row_{}", i)),
                ],
            });
        }

        table
    }

    #[test]
    fn test_new_registry() {
        let registry = TempTableRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table(10);

        let result = registry.insert("#test".to_string(), Arc::new(table));
        assert!(result.is_ok());
        assert_eq!(registry.count(), 1);

        let retrieved = registry.get("#test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().row_count(), 10);
    }

    #[test]
    fn test_insert_without_hash_fails() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table(5);

        let result = registry.insert("test".to_string(), Arc::new(table));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with #"));
    }

    #[test]
    fn test_insert_duplicate_fails() {
        let mut registry = TempTableRegistry::new();
        let table1 = create_test_table(5);
        let table2 = create_test_table(10);

        registry
            .insert("#test".to_string(), Arc::new(table1))
            .unwrap();
        let result = registry.insert("#test".to_string(), Arc::new(table2));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_insert_or_replace() {
        let mut registry = TempTableRegistry::new();
        let table1 = create_test_table(5);
        let table2 = create_test_table(10);

        registry
            .insert_or_replace("#test".to_string(), Arc::new(table1))
            .unwrap();
        assert_eq!(registry.get("#test").unwrap().row_count(), 5);

        registry
            .insert_or_replace("#test".to_string(), Arc::new(table2))
            .unwrap();
        assert_eq!(registry.get("#test").unwrap().row_count(), 10);
    }

    #[test]
    fn test_contains() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table(5);

        assert!(!registry.contains("#test"));
        registry
            .insert("#test".to_string(), Arc::new(table))
            .unwrap();
        assert!(registry.contains("#test"));
    }

    #[test]
    fn test_drop() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table(5);

        registry
            .insert("#test".to_string(), Arc::new(table))
            .unwrap();
        assert_eq!(registry.count(), 1);

        let dropped = registry.drop("#test");
        assert!(dropped);
        assert_eq!(registry.count(), 0);

        let dropped_again = registry.drop("#test");
        assert!(!dropped_again);
    }

    #[test]
    fn test_list_tables() {
        let mut registry = TempTableRegistry::new();
        let table1 = create_test_table(5);
        let table2 = create_test_table(10);

        registry
            .insert("#table1".to_string(), Arc::new(table1))
            .unwrap();
        registry
            .insert("#table2".to_string(), Arc::new(table2))
            .unwrap();

        let tables = registry.list_tables();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"#table1".to_string()));
        assert!(tables.contains(&"#table2".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut registry = TempTableRegistry::new();
        let table1 = create_test_table(5);
        let table2 = create_test_table(10);

        registry
            .insert("#table1".to_string(), Arc::new(table1))
            .unwrap();
        registry
            .insert("#table2".to_string(), Arc::new(table2))
            .unwrap();

        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_multiple_tables() {
        let mut registry = TempTableRegistry::new();

        for i in 0..5 {
            let table = create_test_table(i * 10);
            registry
                .insert(format!("#table{}", i), Arc::new(table))
                .unwrap();
        }

        assert_eq!(registry.count(), 5);

        for i in 0..5 {
            let table = registry.get(&format!("#table{}", i));
            assert!(table.is_some());
            assert_eq!(table.unwrap().row_count(), i * 10);
        }
    }
}
