// Template expansion for WEB CTEs - inject temp table data into URLs and request bodies
// Supports syntax like: ${#table_name}, ${#table.column}, ${#table[0].column}

use crate::data::datatable::DataTable;
use crate::data::temp_table_registry::TempTableRegistry;
use crate::sql::parser::ast::TemplateVar;
use anyhow::{bail, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::Arc;

/// Template expander for injecting temp table data into WEB CTE requests
pub struct TemplateExpander<'a> {
    temp_tables: &'a TempTableRegistry,
}

impl<'a> TemplateExpander<'a> {
    /// Create a new template expander with access to temp tables
    pub fn new(temp_tables: &'a TempTableRegistry) -> Self {
        Self { temp_tables }
    }

    /// Parse template variables from a string
    /// Finds patterns like: ${#table}, ${#table.column}, ${#table[0].column}
    pub fn parse_templates(&self, text: &str) -> Result<Vec<TemplateVar>> {
        let mut vars = Vec::new();

        // Regex to match ${#table_name}, ${#table.column}, ${#table[0]}, ${#table[0].column}
        // Pattern breakdown: ${(#\w+)(?:\[(\d+)\])?(?:\.(\w+))?}
        // Capture groups: 1=#table_name, 2=index, 3=column
        let re = Regex::new(r"\$\{(#\w+)(?:\[(\d+)\])?(?:\.(\w+))?\}").unwrap();

        for cap in re.captures_iter(text) {
            let table_name = cap.get(1).unwrap().as_str().to_string();
            let index = cap.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
            let column = cap.get(3).map(|m| m.as_str().to_string());
            let placeholder = cap.get(0).unwrap().as_str().to_string();

            vars.push(TemplateVar {
                placeholder,
                table_name,
                column,
                index,
            });
        }

        Ok(vars)
    }

    /// Expand templates in a string by replacing placeholders with temp table data
    pub fn expand(&self, text: &str, template_vars: &[TemplateVar]) -> Result<String> {
        let mut result = text.to_string();

        for var in template_vars {
            let replacement = self.resolve_template_var(var)?;
            result = result.replace(&var.placeholder, &replacement);
        }

        Ok(result)
    }

    /// Resolve a single template variable to its JSON representation
    fn resolve_template_var(&self, var: &TemplateVar) -> Result<String> {
        // Get the temp table
        let table = self
            .temp_tables
            .get(&var.table_name)
            .ok_or_else(|| anyhow::anyhow!("Temp table '{}' not found", var.table_name))?;

        // Case 1: ${#table} - entire table as JSON array
        if var.column.is_none() && var.index.is_none() {
            return Ok(self.table_to_json(&table)?);
        }

        // Case 2: ${#table[0]} - single row as JSON object
        if var.column.is_none() && var.index.is_some() {
            let index = var.index.unwrap();
            return Ok(self.row_to_json(&table, index)?);
        }

        // Case 3: ${#table.column} - array of column values
        if var.column.is_some() && var.index.is_none() {
            let column = var.column.as_ref().unwrap();
            return Ok(self.column_to_json(&table, column)?);
        }

        // Case 4: ${#table[0].column} - single cell value
        if var.column.is_some() && var.index.is_some() {
            let column = var.column.as_ref().unwrap();
            let index = var.index.unwrap();
            return Ok(self.cell_to_json(&table, index, column)?);
        }

        bail!("Invalid template variable: {}", var.placeholder)
    }

    /// Convert entire table to JSON array
    fn table_to_json(&self, table: &Arc<DataTable>) -> Result<String> {
        let mut rows = Vec::new();

        for row_idx in 0..table.row_count() {
            let mut row_obj = serde_json::Map::new();
            for (col_idx, col_name) in table.column_names().iter().enumerate() {
                if let Some(cell_value) = table.get_value(row_idx, col_idx) {
                    let value = self.data_value_to_json(cell_value);
                    row_obj.insert(col_name.clone(), value);
                }
            }
            rows.push(Value::Object(row_obj));
        }

        Ok(serde_json::to_string(&rows)?)
    }

    /// Convert single row to JSON object
    fn row_to_json(&self, table: &Arc<DataTable>, row_idx: usize) -> Result<String> {
        if row_idx >= table.row_count() {
            bail!(
                "Row index {} out of bounds (table has {} rows)",
                row_idx,
                table.row_count()
            );
        }

        let mut row_obj = serde_json::Map::new();
        for (col_idx, col_name) in table.column_names().iter().enumerate() {
            if let Some(cell_value) = table.get_value(row_idx, col_idx) {
                let value = self.data_value_to_json(cell_value);
                row_obj.insert(col_name.clone(), value);
            }
        }

        Ok(serde_json::to_string(&Value::Object(row_obj))?)
    }

    /// Convert column to JSON array
    fn column_to_json(&self, table: &Arc<DataTable>, column_name: &str) -> Result<String> {
        let col_idx = table
            .column_names()
            .iter()
            .position(|name| name.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found in table", column_name))?;

        let mut values = Vec::new();
        for row_idx in 0..table.row_count() {
            if let Some(cell_value) = table.get_value(row_idx, col_idx) {
                let value = self.data_value_to_json(cell_value);
                values.push(value);
            }
        }

        Ok(serde_json::to_string(&values)?)
    }

    /// Convert single cell to JSON value
    fn cell_to_json(
        &self,
        table: &Arc<DataTable>,
        row_idx: usize,
        column_name: &str,
    ) -> Result<String> {
        if row_idx >= table.row_count() {
            bail!(
                "Row index {} out of bounds (table has {} rows)",
                row_idx,
                table.row_count()
            );
        }

        let col_idx = table
            .column_names()
            .iter()
            .position(|name| name.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found in table", column_name))?;

        let cell_value = table.get_value(row_idx, col_idx).ok_or_else(|| {
            anyhow::anyhow!("No value found at row {}, column {}", row_idx, col_idx)
        })?;
        let value = self.data_value_to_json(cell_value);
        Ok(serde_json::to_string(&value)?)
    }

    /// Convert DataValue to serde_json::Value
    fn data_value_to_json(&self, data_value: &crate::data::datatable::DataValue) -> Value {
        use crate::data::datatable::DataValue;

        match data_value {
            DataValue::Integer(i) => json!(i),
            DataValue::Float(f) => json!(f),
            DataValue::String(s) => json!(s),
            DataValue::InternedString(s) => json!(s.as_str()),
            DataValue::Boolean(b) => json!(b),
            DataValue::DateTime(dt) => json!(dt),
            DataValue::Vector(v) => json!(v),
            DataValue::Null => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};

    fn create_test_table() -> Arc<DataTable> {
        let mut table = DataTable::new("test_instruments");
        table.add_column(DataColumn::new("symbol").with_type(DataType::String));
        table.add_column(DataColumn::new("price").with_type(DataType::Float));
        table.add_column(DataColumn::new("quantity").with_type(DataType::Integer));

        table
            .add_row(DataRow::new(vec![
                DataValue::String("AAPL".to_string()),
                DataValue::Float(150.5),
                DataValue::Integer(100),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::String("GOOGL".to_string()),
                DataValue::Float(2800.25),
                DataValue::Integer(50),
            ]))
            .unwrap();

        Arc::new(table)
    }

    #[test]
    fn test_parse_templates_simple() {
        let registry = TempTableRegistry::new();
        let expander = TemplateExpander::new(&registry);

        let text = "SELECT * FROM external WHERE symbols IN ${#instruments}";
        let vars = expander.parse_templates(text).unwrap();

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].placeholder, "${#instruments}");
        assert_eq!(vars[0].table_name, "#instruments");
        assert!(vars[0].column.is_none());
        assert!(vars[0].index.is_none());
    }

    #[test]
    fn test_parse_templates_with_column() {
        let registry = TempTableRegistry::new();
        let expander = TemplateExpander::new(&registry);

        let text = "symbols: ${#instruments.symbol}";
        let vars = expander.parse_templates(text).unwrap();

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].placeholder, "${#instruments.symbol}");
        assert_eq!(vars[0].table_name, "#instruments");
        assert_eq!(vars[0].column, Some("symbol".to_string()));
        assert!(vars[0].index.is_none());
    }

    #[test]
    fn test_parse_templates_with_index() {
        let registry = TempTableRegistry::new();
        let expander = TemplateExpander::new(&registry);

        let text = "first row: ${#instruments[0]}";
        let vars = expander.parse_templates(text).unwrap();

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].placeholder, "${#instruments[0]}");
        assert_eq!(vars[0].table_name, "#instruments");
        assert!(vars[0].column.is_none());
        assert_eq!(vars[0].index, Some(0));
    }

    #[test]
    fn test_parse_templates_with_index_and_column() {
        let registry = TempTableRegistry::new();
        let expander = TemplateExpander::new(&registry);

        let text = "first symbol: ${#instruments[0].symbol}";
        let vars = expander.parse_templates(text).unwrap();

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].placeholder, "${#instruments[0].symbol}");
        assert_eq!(vars[0].table_name, "#instruments");
        assert_eq!(vars[0].column, Some("symbol".to_string()));
        assert_eq!(vars[0].index, Some(0));
    }

    #[test]
    fn test_expand_entire_table() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table();
        registry.insert("#instruments".to_string(), table).unwrap();

        let expander = TemplateExpander::new(&registry);
        let text = "Data: ${#instruments}";
        let vars = expander.parse_templates(text).unwrap();
        let result = expander.expand(text, &vars).unwrap();

        assert!(result.contains("AAPL"));
        assert!(result.contains("GOOGL"));
        assert!(result.contains("150.5"));
        assert!(result.contains("2800.25"));
    }

    #[test]
    fn test_expand_single_column() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table();
        registry.insert("#instruments".to_string(), table).unwrap();

        let expander = TemplateExpander::new(&registry);
        let text = "Symbols: ${#instruments.symbol}";
        let vars = expander.parse_templates(text).unwrap();
        let result = expander.expand(text, &vars).unwrap();

        assert!(result.contains("AAPL"));
        assert!(result.contains("GOOGL"));
        assert!(!result.contains("150.5")); // Price should not be included
    }

    #[test]
    fn test_expand_single_cell() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table();
        registry.insert("#instruments".to_string(), table).unwrap();

        let expander = TemplateExpander::new(&registry);
        let text = "First symbol: ${#instruments[0].symbol}";
        let vars = expander.parse_templates(text).unwrap();
        let result = expander.expand(text, &vars).unwrap();

        assert!(result.contains("AAPL"));
        assert!(!result.contains("GOOGL"));
    }

    #[test]
    fn test_expand_multiple_templates() {
        let mut registry = TempTableRegistry::new();
        let table = create_test_table();
        registry.insert("#instruments".to_string(), table).unwrap();

        let expander = TemplateExpander::new(&registry);
        let text = "First: ${#instruments[0].symbol}, All: ${#instruments.symbol}";
        let vars = expander.parse_templates(text).unwrap();
        let result = expander.expand(text, &vars).unwrap();

        assert!(result.contains("AAPL"));
        assert!(result.contains("GOOGL"));
    }
}
