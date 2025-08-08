use crate::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Load a CSV file into a DataTable
pub fn load_csv_to_datatable<P: AsRef<Path>>(path: P, table_name: &str) -> Result<DataTable> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open CSV file: {:?}", path.as_ref()))?;

    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    // Get headers and create columns
    let headers = reader.headers()?.clone();
    let mut table = DataTable::new(table_name);

    // Add metadata about the source
    table
        .metadata
        .insert("source_type".to_string(), "csv".to_string());
    table.metadata.insert(
        "source_path".to_string(),
        path.as_ref().display().to_string(),
    );

    // Create columns from headers (types will be inferred later)
    for header in headers.iter() {
        table.add_column(DataColumn::new(header));
    }

    // Read all rows first to collect data
    let mut string_rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        string_rows.push(row);
    }

    // Infer column types by sampling the data
    let mut column_types = vec![DataType::Null; headers.len()];
    let sample_size = string_rows.len().min(100); // Sample first 100 rows for type inference

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

    // Convert string data to typed DataValues and add rows
    for string_row in string_rows {
        let mut values = Vec::new();
        for (col_idx, value) in string_row.iter().enumerate() {
            let data_value = DataValue::from_string(value, &column_types[col_idx]);
            values.push(data_value);
        }
        table
            .add_row(DataRow::new(values))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    // Update column statistics
    table.infer_column_types();

    Ok(table)
}

/// Load a JSON file into a DataTable
pub fn load_json_to_datatable<P: AsRef<Path>>(path: P, table_name: &str) -> Result<DataTable> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open JSON file: {:?}", path.as_ref()))?;
    let reader = BufReader::new(file);

    // Parse JSON - expect an array of objects
    let json_data: Vec<JsonValue> =
        serde_json::from_reader(reader).with_context(|| "Failed to parse JSON file")?;

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
        .insert("source_type".to_string(), "json".to_string());
    table.metadata.insert(
        "source_path".to_string(),
        path.as_ref().display().to_string(),
    );

    // Create columns from keys
    let column_names: Vec<String> = first_obj.keys().cloned().collect();
    for name in &column_names {
        table.add_column(DataColumn::new(name));
    }

    // Collect all values as strings first for type inference
    let mut string_rows = Vec::new();
    for json_obj in &json_data {
        if let Some(obj) = json_obj.as_object() {
            let mut row = Vec::new();
            for name in &column_names {
                let value_str = match obj.get(name) {
                    Some(JsonValue::Null) | None => String::new(),
                    Some(JsonValue::Bool(b)) => b.to_string(),
                    Some(JsonValue::Number(n)) => n.to_string(),
                    Some(JsonValue::String(s)) => s.clone(),
                    Some(JsonValue::Array(arr)) => format!("{:?}", arr), // Arrays as debug string for now
                    Some(JsonValue::Object(obj)) => format!("{:?}", obj), // Objects as debug string for now
                };
                row.push(value_str);
            }
            string_rows.push(row);
        }
    }

    // Infer column types
    let mut column_types = vec![DataType::Null; column_names.len()];
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

    // Convert to DataRows
    for string_row in string_rows {
        let mut values = Vec::new();
        for (col_idx, value) in string_row.iter().enumerate() {
            let data_value = DataValue::from_string(value, &column_types[col_idx]);
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

/// Load JSON data directly (already parsed) into a DataTable
pub fn load_json_data_to_datatable(data: Vec<JsonValue>, table_name: &str) -> Result<DataTable> {
    if data.is_empty() {
        return Ok(DataTable::new(table_name));
    }

    // Extract column names from all objects (union of all keys)
    let mut all_columns = HashSet::new();
    for item in &data {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                all_columns.insert(key.clone());
            }
        }
    }

    let column_names: Vec<String> = all_columns.into_iter().collect();
    let mut table = DataTable::new(table_name);

    // Add metadata
    table
        .metadata
        .insert("source_type".to_string(), "json_data".to_string());

    // Create columns
    for name in &column_names {
        table.add_column(DataColumn::new(name));
    }

    // Process data similar to file loading
    let mut string_rows = Vec::new();
    for json_obj in &data {
        if let Some(obj) = json_obj.as_object() {
            let mut row = Vec::new();
            for name in &column_names {
                let value_str = match obj.get(name) {
                    Some(JsonValue::Null) | None => String::new(),
                    Some(JsonValue::Bool(b)) => b.to_string(),
                    Some(JsonValue::Number(n)) => n.to_string(),
                    Some(JsonValue::String(s)) => s.clone(),
                    Some(JsonValue::Array(arr)) => format!("{:?}", arr),
                    Some(JsonValue::Object(obj)) => format!("{:?}", obj),
                };
                row.push(value_str);
            }
            string_rows.push(row);
        }
    }

    // Infer types and convert to DataRows (same as above)
    let mut column_types = vec![DataType::Null; column_names.len()];
    let sample_size = string_rows.len().min(100);

    for row in string_rows.iter().take(sample_size) {
        for (col_idx, value) in row.iter().enumerate() {
            if !value.is_empty() {
                let inferred = DataType::infer_from_string(value);
                column_types[col_idx] = column_types[col_idx].merge(&inferred);
            }
        }
    }

    for (col_idx, column) in table.columns.iter_mut().enumerate() {
        column.data_type = column_types[col_idx].clone();
    }

    for string_row in string_rows {
        let mut values = Vec::new();
        for (col_idx, value) in string_row.iter().enumerate() {
            let data_value = DataValue::from_string(value, &column_types[col_idx]);
            values.push(data_value);
        }
        table
            .add_row(DataRow::new(values))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    table.infer_column_types();

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() -> Result<()> {
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,price,quantity")?;
        writeln!(temp_file, "1,Widget,9.99,100")?;
        writeln!(temp_file, "2,Gadget,19.99,50")?;
        writeln!(temp_file, "3,Doohickey,5.00,200")?;
        temp_file.flush()?;

        let table = load_csv_to_datatable(temp_file.path(), "products")?;

        assert_eq!(table.name, "products");
        assert_eq!(table.column_count(), 4);
        assert_eq!(table.row_count(), 3);

        // Check column types were inferred correctly
        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[0].data_type, DataType::Integer);

        assert_eq!(table.columns[1].name, "name");
        assert_eq!(table.columns[1].data_type, DataType::String);

        assert_eq!(table.columns[2].name, "price");
        assert_eq!(table.columns[2].data_type, DataType::Float);

        assert_eq!(table.columns[3].name, "quantity");
        assert_eq!(table.columns[3].data_type, DataType::Integer);

        // Check data
        let value = table.get_value_by_name(0, "name").unwrap();
        assert_eq!(value.to_string(), "Widget");

        Ok(())
    }

    #[test]
    fn test_load_json() -> Result<()> {
        // Create a temporary JSON file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"[
            {{"id": 1, "name": "Alice", "active": true, "score": 95.5}},
            {{"id": 2, "name": "Bob", "active": false, "score": 87.3}},
            {{"id": 3, "name": "Charlie", "active": true, "score": null}}
        ]"#
        )?;
        temp_file.flush()?;

        let table = load_json_to_datatable(temp_file.path(), "users")?;

        assert_eq!(table.name, "users");
        assert_eq!(table.column_count(), 4);
        assert_eq!(table.row_count(), 3);

        // Check that null handling works
        let score = table.get_value_by_name(2, "score").unwrap();
        assert!(score.is_null());

        Ok(())
    }
}
