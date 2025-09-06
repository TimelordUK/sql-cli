/// Direct CSV to `DataTable` loader - bypasses JSON intermediate format
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use anyhow::Result;
use csv;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, info};

pub struct DirectCsvLoader;

impl DirectCsvLoader {
    /// Load CSV directly into `DataTable` without JSON intermediate
    pub fn load_csv_direct<P: AsRef<Path>>(path: P, table_name: &str) -> Result<DataTable> {
        let path = path.as_ref();
        info!("Direct CSV load: Loading {} into DataTable", path.display());

        // Track memory before loading
        crate::utils::memory_tracker::track_memory("direct_csv_start");

        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);

        // Get headers and create columns
        let headers = reader.headers()?.clone(); // Clone to release the borrow
        let mut table = DataTable::new(table_name);

        for header in &headers {
            table.add_column(DataColumn::new(header.to_string()));
        }

        crate::utils::memory_tracker::track_memory("direct_csv_headers");

        // Re-open file to read raw lines for null detection
        let file2 = File::open(path)?;
        let mut line_reader = BufReader::new(file2);
        let mut raw_line = String::new();
        // Skip header line
        line_reader.read_line(&mut raw_line)?;

        // Read rows directly into DataTable
        let mut row_count = 0;
        for result in reader.records() {
            let record = result?;

            // Read the corresponding raw line
            raw_line.clear();
            line_reader.read_line(&mut raw_line)?;

            let mut values = Vec::with_capacity(headers.len());

            for (i, field) in record.iter().enumerate() {
                // Distinguish between null (,,) and empty string ("")
                let value = if field.is_empty() {
                    if Self::is_null_field(&raw_line, i) {
                        DataValue::Null
                    } else {
                        DataValue::String(String::new())
                    }
                } else if let Ok(b) = field.parse::<bool>() {
                    DataValue::Boolean(b)
                } else if let Ok(i) = field.parse::<i64>() {
                    DataValue::Integer(i)
                } else if let Ok(f) = field.parse::<f64>() {
                    DataValue::Float(f)
                } else {
                    // Check for date-like strings
                    if field.contains('-') && field.len() >= 8 && field.len() <= 30 {
                        DataValue::DateTime(field.to_string())
                    } else {
                        DataValue::String(field.to_string())
                    }
                };
                values.push(value);
            }

            table
                .add_row(DataRow::new(values))
                .map_err(|e| anyhow::anyhow!(e))?;
            row_count += 1;

            // Track memory every 5000 rows
            if row_count % 5000 == 0 {
                crate::utils::memory_tracker::track_memory(&format!("direct_csv_{row_count}rows"));
            }
        }

        // Infer column types from the data
        table.infer_column_types();

        crate::utils::memory_tracker::track_memory("direct_csv_complete");

        info!(
            "Direct CSV load complete: {} rows, {} columns, ~{} MB",
            table.row_count(),
            table.column_count(),
            table.estimate_memory_size() / 1024 / 1024
        );

        Ok(table)
    }

    /// Helper to detect if a field in the raw CSV line is a null (unquoted empty)
    fn is_null_field(raw_line: &str, field_index: usize) -> bool {
        let mut comma_count = 0;
        let mut in_quotes = false;
        let mut field_start = 0;
        let mut prev_char = ' ';

        for (i, ch) in raw_line.chars().enumerate() {
            if ch == '"' && prev_char != '\\' {
                in_quotes = !in_quotes;
            }

            if ch == ',' && !in_quotes {
                if comma_count == field_index {
                    let field_end = i;
                    let field_content = &raw_line[field_start..field_end].trim();
                    // If empty, check if it was quoted (quoted empty = empty string, unquoted empty = NULL)
                    if field_content.is_empty() {
                        return true; // Unquoted empty field -> NULL
                    }
                    // If it starts and ends with quotes but is empty inside, it's an empty string, not NULL
                    if field_content.starts_with('"')
                        && field_content.ends_with('"')
                        && field_content.len() == 2
                    {
                        return false; // Quoted empty field -> empty string
                    }
                    return false; // Non-empty field -> not NULL
                }
                comma_count += 1;
                field_start = i + 1;
            }
            prev_char = ch;
        }

        // Check last field
        if comma_count == field_index {
            let field_content = raw_line[field_start..]
                .trim()
                .trim_end_matches('\n')
                .trim_end_matches('\r');
            // If empty, check if it was quoted
            if field_content.is_empty() {
                return true; // Unquoted empty field -> NULL
            }
            // If it starts and ends with quotes but is empty inside, it's an empty string, not NULL
            if field_content.starts_with('"')
                && field_content.ends_with('"')
                && field_content.len() == 2
            {
                return false; // Quoted empty field -> empty string
            }
            return false; // Non-empty field -> not NULL
        }

        false // Field not found -> not NULL (shouldn't happen)
    }

    /// Execute a SQL query directly on a `DataTable` (no JSON)
    pub fn query_datatable(table: &DataTable, sql: &str) -> Result<DataTable> {
        // For now, just return a reference/clone of the table
        // In the future, this would apply WHERE/ORDER BY/etc directly on DataTable
        debug!("Direct query on DataTable: {}", sql);

        // Simple SELECT * for now
        if sql.trim().to_uppercase().starts_with("SELECT *") {
            Ok(table.clone())
        } else {
            // TODO: Implement proper SQL execution on DataTable
            Ok(table.clone())
        }
    }
}
