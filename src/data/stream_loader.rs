// Stream-based data loader that works with any Read source
// This allows the same code to handle files, HTTP responses, or any other data stream

use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use tracing::{debug, info};

use crate::data::advanced_csv_loader::StringInterner;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};

/// Column analysis results for determining interning strategy
#[derive(Debug)]
struct ColumnAnalysis {
    index: usize,
    _name: String,
    _cardinality: usize,
    _sample_size: usize,
    _unique_ratio: f64,
    is_categorical: bool,
    _avg_string_length: usize,
}

/// Advanced stream-based CSV loader with string interning
pub struct StreamCsvLoader {
    sample_size: usize,
    cardinality_threshold: f64,
    interners: HashMap<usize, StringInterner>,
}

impl StreamCsvLoader {
    pub fn new() -> Self {
        Self {
            sample_size: 1000,
            cardinality_threshold: 0.3,
            interners: HashMap::new(),
        }
    }

    /// Analyze columns to determine which should use string interning
    fn analyze_columns(
        &self,
        rows: &[Vec<String>],
        headers: &csv::StringRecord,
    ) -> Vec<ColumnAnalysis> {
        let mut analyses = Vec::new();

        for (col_idx, header) in headers.iter().enumerate() {
            let mut unique_values = HashSet::new();
            let mut total_length = 0;
            let mut non_empty_count = 0;

            // Sample rows to analyze cardinality
            for row in rows.iter().take(self.sample_size) {
                if let Some(value) = row.get(col_idx) {
                    if !value.is_empty() {
                        unique_values.insert(value.clone());
                        total_length += value.len();
                        non_empty_count += 1;
                    }
                }
            }

            let cardinality = unique_values.len();
            let sample_size = rows.len().min(self.sample_size);
            let unique_ratio = if sample_size > 0 {
                cardinality as f64 / sample_size as f64
            } else {
                1.0
            };

            let avg_string_length = if non_empty_count > 0 {
                total_length / non_empty_count
            } else {
                0
            };

            // Consider categorical if low cardinality ratio or short strings with repetition
            let is_categorical = unique_ratio < self.cardinality_threshold
                || (avg_string_length < 20 && cardinality < sample_size / 2);

            analyses.push(ColumnAnalysis {
                index: col_idx,
                _name: header.to_string(),
                _cardinality: cardinality,
                _sample_size: sample_size,
                _unique_ratio: unique_ratio,
                is_categorical,
                _avg_string_length: avg_string_length,
            });
        }

        analyses
    }

    /// Load CSV data with string interning from any Read source
    pub fn load_csv_from_reader<R: Read>(
        &mut self,
        mut reader: R,
        table_name: &str,
        source_type: &str,
        source_path: &str,
    ) -> Result<DataTable> {
        info!(
            "Stream CSV load: Loading {} with optimizations",
            source_path
        );

        // Read all data into memory
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        // First pass: Parse CSV with headers
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&buffer[..]);

        let headers = csv_reader.headers()?.clone();
        let mut table = DataTable::new(table_name);

        // Add metadata about the source
        table
            .metadata
            .insert("source_type".to_string(), source_type.to_string());
        table
            .metadata
            .insert("source_path".to_string(), source_path.to_string());

        // Create columns from headers
        for header in &headers {
            table.add_column(DataColumn::new(header));
        }

        // Collect all rows as strings
        let mut string_rows = Vec::new();
        for result in csv_reader.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            string_rows.push(row);
        }

        // Analyze columns for string interning
        let analyses = self.analyze_columns(&string_rows, &headers);
        let categorical_columns: HashSet<usize> = analyses
            .iter()
            .filter(|a| a.is_categorical)
            .map(|a| a.index)
            .collect();

        info!(
            "Column analysis: {} of {} columns will use string interning",
            categorical_columns.len(),
            analyses.len()
        );

        // Initialize interners for categorical columns
        for col_idx in &categorical_columns {
            self.interners.insert(*col_idx, StringInterner::new());
        }

        // Second pass: Read raw lines for NULL detection
        let mut line_reader = BufReader::new(&buffer[..]);
        let mut raw_lines = Vec::new();
        let mut raw_line = String::new();

        // Skip header
        line_reader.read_line(&mut raw_line)?;
        raw_line.clear();

        // Read all raw lines
        for _ in 0..string_rows.len() {
            line_reader.read_line(&mut raw_line)?;
            raw_lines.push(raw_line.clone());
            raw_line.clear();
        }

        // Infer column types by sampling
        let mut column_types = vec![DataType::Null; headers.len()];
        let sample_size = string_rows.len().min(100);

        for row in string_rows.iter().take(sample_size) {
            for (col_idx, value) in row.iter().enumerate() {
                if !value.is_empty() {
                    let inferred = DataType::infer_from_string(value);
                    column_types[col_idx] = column_types[col_idx].merge(&inferred);
                }
            }
        }

        // Update column types
        for (col_idx, column) in table.columns.iter_mut().enumerate() {
            column.data_type = column_types[col_idx].clone();
        }

        // Convert strings to typed values and add rows
        for (row_idx, string_row) in string_rows.iter().enumerate() {
            let mut values = Vec::new();
            let raw_line = &raw_lines[row_idx];

            for (col_idx, value) in string_row.iter().enumerate() {
                let data_value = if value.is_empty() {
                    // Check if this is NULL (,,) vs empty string ("")
                    if is_null_field(raw_line, col_idx) {
                        DataValue::Null
                    } else if categorical_columns.contains(&col_idx) {
                        // Use interned string for empty categorical values
                        if let Some(interner) = self.interners.get_mut(&col_idx) {
                            DataValue::InternedString(interner.intern(""))
                        } else {
                            DataValue::String(String::new())
                        }
                    } else {
                        DataValue::String(String::new())
                    }
                } else if categorical_columns.contains(&col_idx)
                    && column_types[col_idx] == DataType::String
                {
                    // Use string interning for categorical columns
                    if let Some(interner) = self.interners.get_mut(&col_idx) {
                        DataValue::InternedString(interner.intern(value))
                    } else {
                        DataValue::from_string(value, &column_types[col_idx])
                    }
                } else {
                    DataValue::from_string(value, &column_types[col_idx])
                };
                values.push(data_value);
            }
            table
                .add_row(DataRow::new(values))
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        // Print interner statistics
        for (col_idx, interner) in &self.interners {
            let stats = interner.stats();
            if stats.memory_saved_bytes > 0 {
                debug!(
                    "Column {} interning: {} unique strings, {} references, {} bytes saved",
                    headers.get(*col_idx).unwrap_or(&String::new()),
                    stats.unique_strings,
                    stats.total_references,
                    stats.memory_saved_bytes
                );
            }
        }

        // Update column statistics
        table.infer_column_types();

        Ok(table)
    }
}

/// Simple wrapper for loading CSV without advanced features
pub fn load_csv_from_reader<R: Read>(
    reader: R,
    table_name: &str,
    source_type: &str,
    source_path: &str,
) -> Result<DataTable> {
    let mut loader = StreamCsvLoader::new();
    loader.load_csv_from_reader(reader, table_name, source_type, source_path)
}

/// Load JSON data from any Read source into a DataTable
pub fn load_json_from_reader<R: Read>(
    mut reader: R,
    table_name: &str,
    source_type: &str,
    source_path: &str,
) -> Result<DataTable> {
    let mut json_str = String::new();
    reader.read_to_string(&mut json_str)?;

    let json_data: Vec<JsonValue> =
        serde_json::from_str(&json_str).with_context(|| "Failed to parse JSON data")?;

    if json_data.is_empty() {
        return Ok(DataTable::new(table_name));
    }

    // Extract column names from first object
    let first_obj = json_data[0]
        .as_object()
        .context("JSON data must be an array of objects")?;

    let mut table = DataTable::new(table_name);

    // Add metadata
    table
        .metadata
        .insert("source_type".to_string(), source_type.to_string());
    table
        .metadata
        .insert("source_path".to_string(), source_path.to_string());

    // Create columns
    let column_names: Vec<String> = first_obj.keys().cloned().collect();
    for name in &column_names {
        table.add_column(DataColumn::new(name));
    }

    // Collect values for type inference
    let mut string_rows = Vec::new();
    for json_obj in &json_data {
        if let Some(obj) = json_obj.as_object() {
            let mut row = Vec::new();
            for col_name in &column_names {
                let value = obj
                    .get(col_name)
                    .map(|v| json_value_to_string(v))
                    .unwrap_or_default();
                row.push(value);
            }
            string_rows.push(row);
        }
    }

    // Infer column types
    let mut column_types = vec![DataType::Null; column_names.len()];
    let sample_size = string_rows.len().min(100);

    for row in string_rows.iter().take(sample_size) {
        for (col_idx, value) in row.iter().enumerate() {
            if !value.is_empty() && value != "null" {
                let inferred = DataType::infer_from_string(value);
                column_types[col_idx] = column_types[col_idx].merge(&inferred);
            }
        }
    }

    // Update column types
    for (col_idx, column) in table.columns.iter_mut().enumerate() {
        column.data_type = column_types[col_idx].clone();
    }

    // Convert to typed values and add rows
    for string_row in &string_rows {
        let mut values = Vec::new();
        for (col_idx, value) in string_row.iter().enumerate() {
            let data_value = if value.is_empty() || value == "null" {
                DataValue::Null
            } else {
                DataValue::from_string(value, &column_types[col_idx])
            };
            values.push(data_value);
        }
        table
            .add_row(DataRow::new(values))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    // Update statistics
    table.infer_column_types();

    Ok(table)
}

/// Helper to convert JSON value to string for type inference
fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        JsonValue::Array(arr) => format!("{:?}", arr),
        JsonValue::Object(obj) => format!("{:?}", obj),
    }
}

/// Helper to detect NULL fields in raw CSV lines
fn is_null_field(raw_line: &str, field_index: usize) -> bool {
    let mut comma_count = 0;
    let mut in_quotes = false;
    let mut field_start = 0;
    let mut prev_char = ' ';

    for (i, ch) in raw_line.char_indices() {
        if ch == '"' && prev_char != '\\' {
            in_quotes = !in_quotes;
        } else if ch == ',' && !in_quotes {
            if comma_count == field_index {
                // Found the field - check if it's empty
                return i == field_start
                    || (i == field_start + 1 && raw_line.chars().nth(field_start) == Some(','));
            }
            comma_count += 1;
            field_start = i + 1;
        }
        prev_char = ch;
    }

    // Check last field
    if comma_count == field_index {
        let remaining = raw_line[field_start..].trim_end();
        return remaining.is_empty() || remaining == ",";
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_csv_from_reader() {
        let csv_data = "id,name,value\n1,Alice,100\n2,Bob,200\n3,,300";
        let reader = Cursor::new(csv_data);

        let table =
            load_csv_from_reader(reader, "test", "stream", "memory").expect("Failed to load CSV");

        assert_eq!(table.name, "test");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that empty field is NULL
        let value = table.get_value(2, 1).unwrap();
        assert!(matches!(value, DataValue::Null));
    }

    #[test]
    fn test_json_from_reader() {
        let json_data = r#"[
            {"id": 1, "name": "Alice", "value": 100},
            {"id": 2, "name": "Bob", "value": 200},
            {"id": 3, "name": null, "value": 300}
        ]"#;
        let reader = Cursor::new(json_data);

        let table =
            load_json_from_reader(reader, "test", "stream", "memory").expect("Failed to load JSON");

        assert_eq!(table.name, "test");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that null is handled
        let value = table.get_value(2, 1).unwrap();
        assert!(matches!(value, DataValue::Null));
    }
}
