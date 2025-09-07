use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};

pub struct VirtualTableGenerator;

impl VirtualTableGenerator {
    pub fn generate_range(
        start: i64,
        end: i64,
        step: Option<i64>,
        column_name: Option<String>,
    ) -> Result<Arc<DataTable>> {
        let step = step.unwrap_or(1);
        let column_name = column_name.unwrap_or_else(|| "value".to_string());

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

        let mut table = DataTable::new("range_table");

        let column = DataColumn {
            name: column_name,
            data_type: DataType::Integer,
            nullable: false,
            unique_values: Some(0),
            null_count: 0,
            metadata: HashMap::new(),
        };
        table.add_column(column);

        let mut current = start;
        while (step > 0 && current <= end) || (step < 0 && current >= end) {
            let row = DataRow {
                values: vec![DataValue::Integer(current)],
            };
            table.add_row(row);
            current += step;
        }

        Ok(Arc::new(table))
    }

    pub fn generate_series(count: i64, start: Option<i64>) -> Result<Arc<DataTable>> {
        let start = start.unwrap_or(1);
        Self::generate_range(start, start + count - 1, Some(1), Some("index".to_string()))
    }
}
