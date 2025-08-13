/// Direct CSV to DataTable loader - bypasses JSON intermediate format
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use anyhow::Result;
use csv;
use std::fs::File;
use std::path::Path;
use tracing::{debug, info};

pub struct DirectCsvLoader;

impl DirectCsvLoader {
    /// Load CSV directly into DataTable without JSON intermediate
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

        for header in headers.iter() {
            table.add_column(DataColumn::new(header.to_string()));
        }

        crate::utils::memory_tracker::track_memory("direct_csv_headers");

        // Read rows directly into DataTable
        let mut row_count = 0;
        for result in reader.records() {
            let record = result?;
            let mut values = Vec::with_capacity(headers.len());

            for field in record.iter() {
                // Simple type inference - can be improved later
                let value = if field.is_empty() {
                    DataValue::Null
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
                crate::utils::memory_tracker::track_memory(&format!(
                    "direct_csv_{}rows",
                    row_count
                ));
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

    /// Execute a SQL query directly on a DataTable (no JSON)
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
