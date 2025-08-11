use crate::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Convert JSON values (from CSV or JSON sources) to DataTable
pub struct DataTableConverter;

impl DataTableConverter {
    /// Convert a vector of JSON values to a DataTable
    pub fn from_json_values(values: &[Value], table_name: impl Into<String>) -> Result<DataTable> {
        if values.is_empty() {
            return Ok(DataTable::new(table_name));
        }

        // Extract column names from first row
        let column_names = Self::extract_column_names(values)?;

        // Infer column types by sampling rows
        let column_types = Self::infer_column_types(values, &column_names);

        // Create DataTable with columns
        let mut table = DataTable::new(table_name);
        for (name, data_type) in column_names.iter().zip(column_types.iter()) {
            table.add_column(
                DataColumn::new(name.clone())
                    .with_type(data_type.clone())
                    .with_nullable(true), // For now, all columns are nullable
            );
        }

        // Convert rows
        for json_row in values {
            let data_row = Self::convert_json_row(json_row, &column_names, &column_types)?;
            table
                .add_row(data_row)
                .map_err(|e| anyhow::anyhow!("Failed to add row: {}", e))?;
        }

        // Update column statistics
        Self::update_column_stats(&mut table);

        Ok(table)
    }

    /// Convert from CSV lines (headers + rows of string values)
    pub fn from_csv_data(
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        table_name: impl Into<String>,
    ) -> Result<DataTable> {
        // First pass: infer column types
        let column_types = Self::infer_types_from_strings(&headers, &rows);

        // Create DataTable
        let mut table = DataTable::new(table_name);
        for (name, data_type) in headers.iter().zip(column_types.iter()) {
            table.add_column(
                DataColumn::new(name.clone())
                    .with_type(data_type.clone())
                    .with_nullable(true),
            );
        }

        // Convert rows
        for row_values in rows {
            let mut data_values = Vec::new();
            for (value, data_type) in row_values.iter().zip(column_types.iter()) {
                data_values.push(DataValue::from_string(value, data_type));
            }
            table
                .add_row(DataRow::new(data_values))
                .map_err(|e| anyhow::anyhow!("Failed to add row: {}", e))?;
        }

        // Update column statistics
        Self::update_column_stats(&mut table);

        Ok(table)
    }

    /// Extract column names from JSON values
    fn extract_column_names(values: &[Value]) -> Result<Vec<String>> {
        if let Some(first) = values.first() {
            if let Some(obj) = first.as_object() {
                Ok(obj.keys().cloned().collect())
            } else {
                Err(anyhow::anyhow!("JSON values must be objects"))
            }
        } else {
            Ok(vec![])
        }
    }

    /// Infer column types by sampling JSON values
    fn infer_column_types(values: &[Value], column_names: &[String]) -> Vec<DataType> {
        let mut column_types = vec![DataType::Null; column_names.len()];

        // Sample up to 100 rows for type inference
        let sample_size = values.len().min(100);

        for row in values.iter().take(sample_size) {
            if let Some(obj) = row.as_object() {
                for (idx, col_name) in column_names.iter().enumerate() {
                    if let Some(value) = obj.get(col_name) {
                        let value_type = Self::infer_json_type(value);
                        column_types[idx] = column_types[idx].merge(&value_type);
                    }
                }
            }
        }

        // Default any remaining Null types to String
        for col_type in &mut column_types {
            if *col_type == DataType::Null {
                *col_type = DataType::String;
            }
        }

        column_types
    }

    /// Infer DataType from a JSON value
    fn infer_json_type(value: &Value) -> DataType {
        match value {
            Value::Null => DataType::Null,
            Value::Bool(_) => DataType::Boolean,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    DataType::Integer
                } else {
                    DataType::Float
                }
            }
            Value::String(s) => DataType::infer_from_string(s),
            Value::Array(_) | Value::Object(_) => DataType::String, // Serialize as JSON string
        }
    }

    /// Convert a JSON row to DataRow
    fn convert_json_row(
        json_row: &Value,
        column_names: &[String],
        column_types: &[DataType],
    ) -> Result<DataRow> {
        let mut values = Vec::new();

        if let Some(obj) = json_row.as_object() {
            for (col_name, col_type) in column_names.iter().zip(column_types.iter()) {
                let value = obj
                    .get(col_name)
                    .map(|v| Self::json_to_datavalue(v, col_type))
                    .unwrap_or(DataValue::Null);
                values.push(value);
            }
        } else {
            return Err(anyhow::anyhow!("JSON row must be an object"));
        }

        Ok(DataRow::new(values))
    }

    /// Convert JSON value to DataValue
    fn json_to_datavalue(json_val: &Value, expected_type: &DataType) -> DataValue {
        match json_val {
            Value::Null => DataValue::Null,
            Value::Bool(b) => DataValue::Boolean(*b),
            Value::Number(n) => match expected_type {
                DataType::Integer => n
                    .as_i64()
                    .map(DataValue::Integer)
                    .unwrap_or(DataValue::Null),
                DataType::Float => n.as_f64().map(DataValue::Float).unwrap_or(DataValue::Null),
                _ => DataValue::String(n.to_string()),
            },
            Value::String(s) => DataValue::from_string(s, expected_type),
            Value::Array(_) | Value::Object(_) => {
                // Serialize complex types as JSON strings
                DataValue::String(json_val.to_string())
            }
        }
    }

    /// Infer types from string data (for CSV)
    fn infer_types_from_strings(headers: &[String], rows: &[Vec<String>]) -> Vec<DataType> {
        let mut column_types = vec![DataType::Null; headers.len()];

        // Sample up to 100 rows
        let sample_size = rows.len().min(100);

        for row in rows.iter().take(sample_size) {
            for (idx, value) in row.iter().enumerate() {
                if idx < column_types.len() {
                    let value_type = DataType::infer_from_string(value);
                    column_types[idx] = column_types[idx].merge(&value_type);
                }
            }
        }

        // Default any remaining Null types to String
        for col_type in &mut column_types {
            if *col_type == DataType::Null {
                *col_type = DataType::String;
            }
        }

        column_types
    }

    /// Update column statistics (null counts, unique values, etc.)
    fn update_column_stats(table: &mut DataTable) {
        for (col_idx, column) in table.columns.iter_mut().enumerate() {
            let mut null_count = 0;
            let mut unique_values = HashMap::new();

            for row in &table.rows {
                if let Some(value) = row.get(col_idx) {
                    if value.is_null() {
                        null_count += 1;
                    } else {
                        unique_values.insert(value.to_string(), ());
                    }
                }
            }

            column.null_count = null_count;
            column.unique_values = Some(unique_values.len());
        }
    }

    /// Debug print a DataTable (for testing)
    pub fn debug_print(table: &DataTable) {
        println!("DataTable: {}", table.name);
        println!("Columns: {}", table.column_count());
        println!("Rows: {}", table.row_count());

        // Print column info
        for col in &table.columns {
            println!(
                "  - {} ({:?}, nulls: {}, unique: {:?})",
                col.name, col.data_type, col.null_count, col.unique_values
            );
        }

        // Print first 5 rows
        println!("\nFirst 5 rows:");
        for (idx, row) in table.rows.iter().take(5).enumerate() {
            print!("  Row {}: ", idx);
            for value in &row.values {
                print!("{}, ", value);
            }
            println!();
        }
    }
}
