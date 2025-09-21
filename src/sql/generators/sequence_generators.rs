use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// RANGE - Generate numeric sequence
pub struct Range;

impl TableGenerator for Range {
    fn name(&self) -> &str {
        "RANGE"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn {
            name: "value".to_string(),
            data_type: DataType::Integer,
            nullable: false,
            unique_values: Some(0),
            null_count: 0,
            metadata: HashMap::new(),
            qualified_name: None,
            source_table: None,
        }]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        let (start, end, step) = match args.len() {
            1 => {
                // RANGE(n) - from 1 to n
                let end = match args[0] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                (1, end, 1)
            }
            2 => {
                // RANGE(start, end) - from start to end with step 1
                let start = match args[0] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                let end = match args[1] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                (start, end, 1)
            }
            3 => {
                // RANGE(start, end, step) - from start to end with given step
                let start = match args[0] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                let end = match args[1] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                let step = match args[2] {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("RANGE expects numeric arguments")),
                };
                (start, end, step)
            }
            _ => {
                return Err(anyhow!(
                    "RANGE expects 1-3 arguments (end), (start, end), or (start, end, step)"
                ))
            }
        };

        if step == 0 {
            return Err(anyhow!("RANGE step cannot be zero"));
        }

        if (step > 0 && start > end) || (step < 0 && start < end) {
            return Err(anyhow!(
                "RANGE parameters invalid: start={}, end={}, step={}",
                start,
                end,
                step
            ));
        }

        let mut table = DataTable::new("range");
        table.add_column(DataColumn::new("value"));

        let mut current = start;
        while (step > 0 && current <= end) || (step < 0 && current >= end) {
            table
                .add_row(DataRow::new(vec![DataValue::Integer(current)]))
                .map_err(|e| anyhow!(e))?;
            current += step;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate numeric sequence"
    }

    fn arg_count(&self) -> usize {
        3 // Can accept 1, 2, or 3 arguments
    }
}

/// SERIES - Generate a series with index (like pandas)
pub struct Series;

impl TableGenerator for Series {
    fn name(&self) -> &str {
        "SERIES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn {
                name: "index".to_string(),
                data_type: DataType::Integer,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
            DataColumn {
                name: "value".to_string(),
                data_type: DataType::Integer,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
        ]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() {
            return Err(anyhow!("SERIES requires at least 1 argument (count)"));
        }

        let count = match args[0] {
            DataValue::Integer(n) => n,
            DataValue::Float(f) => f as i64,
            _ => return Err(anyhow!("SERIES count must be numeric")),
        };

        let start = if args.len() > 1 {
            match args[1] {
                DataValue::Integer(n) => n,
                DataValue::Float(f) => f as i64,
                _ => 0,
            }
        } else {
            0
        };

        let mut table = DataTable::new("series");
        table.add_column(DataColumn::new("index"));
        table.add_column(DataColumn::new("value"));

        for i in 0..count {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(i),
                    DataValue::Integer(start + i),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate indexed series (0-based index with values)"
    }

    fn arg_count(&self) -> usize {
        2 // count and optional start value
    }
}

/// DATES - Generate date sequence
pub struct Dates;

impl TableGenerator for Dates {
    fn name(&self) -> &str {
        "DATES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn {
            name: "date".to_string(),
            data_type: DataType::String, // We'll store dates as ISO strings
            nullable: false,
            unique_values: Some(0),
            null_count: 0,
            metadata: HashMap::new(),
            qualified_name: None,
            source_table: None,
        }]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        use chrono::{Duration, NaiveDate};

        if args.len() < 2 {
            return Err(anyhow!(
                "DATES requires at least 2 arguments (start_date, end_date)"
            ));
        }

        // Parse start date
        let start_str = match &args[0] {
            DataValue::String(s) => s,
            _ => return Err(anyhow!("DATES start_date must be a string")),
        };

        let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid start date format (expected YYYY-MM-DD): {}", e))?;

        // Parse end date
        let end_str = match &args[1] {
            DataValue::String(s) => s,
            _ => return Err(anyhow!("DATES end_date must be a string")),
        };

        let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid end date format (expected YYYY-MM-DD): {}", e))?;

        // Get step (default to 1 day)
        let step_days = if args.len() > 2 {
            match args[2] {
                DataValue::Integer(n) => n,
                DataValue::Float(f) => f as i64,
                _ => 1,
            }
        } else {
            1
        };

        if step_days == 0 {
            return Err(anyhow!("DATES step cannot be zero"));
        }

        let mut table = DataTable::new("dates");
        table.add_column(DataColumn::new("date"));

        let mut current = start;
        let step = Duration::days(step_days);

        if step_days > 0 {
            while current <= end {
                table
                    .add_row(DataRow::new(vec![DataValue::String(
                        current.format("%Y-%m-%d").to_string(),
                    )]))
                    .map_err(|e| anyhow!(e))?;
                current = current + step;
            }
        } else {
            while current >= end {
                table
                    .add_row(DataRow::new(vec![DataValue::String(
                        current.format("%Y-%m-%d").to_string(),
                    )]))
                    .map_err(|e| anyhow!(e))?;
                current = current + step;
            }
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate date sequence between two dates"
    }

    fn arg_count(&self) -> usize {
        3 // start_date, end_date, optional step_days
    }
}
