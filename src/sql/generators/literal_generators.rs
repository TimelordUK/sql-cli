use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// VALUES - Create a table from literal values
/// This generator allows creating a table from a list of literal values
/// Example: SELECT * FROM VALUES(1, 20, 30, 50)
pub struct Values;

impl TableGenerator for Values {
    fn name(&self) -> &str {
        "VALUES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn {
            name: "value".to_string(),
            data_type: DataType::Float, // Use Float to handle both integers and decimals
            nullable: false,
            unique_values: Some(0),
            null_count: 0,
            metadata: HashMap::new(),
            qualified_name: None,
            source_table: None,
        }]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() {
            return Err(anyhow!("VALUES expects at least one argument"));
        }

        let mut table = DataTable::new("values");
        table.add_column(DataColumn::new("value"));

        // Add each argument as a row in the table
        for (i, arg) in args.iter().enumerate() {
            let value = match arg {
                DataValue::Integer(n) => DataValue::Integer(*n),
                DataValue::Float(f) => DataValue::Float(*f),
                DataValue::String(s) => {
                    // Try to parse string as number
                    if let Ok(n) = s.parse::<i64>() {
                        DataValue::Integer(n)
                    } else if let Ok(f) = s.parse::<f64>() {
                        DataValue::Float(f)
                    } else {
                        return Err(anyhow!(
                            "VALUES argument {} ('{}') cannot be converted to a number",
                            i + 1,
                            s
                        ));
                    }
                }
                DataValue::Null => DataValue::Null,
                _ => {
                    return Err(anyhow!(
                        "VALUES expects numeric arguments, got {:?} at position {}",
                        arg,
                        i + 1
                    ))
                }
            };

            table
                .add_row(DataRow::new(vec![value]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate a table from literal values"
    }

    fn arg_count(&self) -> usize {
        0 // Variable number of arguments
    }
}

/// ARRAY - Alternative name for VALUES generator
/// This provides a more intuitive name for creating arrays as tables
pub struct Array;

impl TableGenerator for Array {
    fn name(&self) -> &str {
        "ARRAY"
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
                data_type: DataType::Float,
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
            return Err(anyhow!("ARRAY expects at least one argument"));
        }

        let mut table = DataTable::new("array");
        table.add_column(DataColumn::new("index"));
        table.add_column(DataColumn::new("value"));

        // Add each argument as a row with index
        for (i, arg) in args.iter().enumerate() {
            let value = match arg {
                DataValue::Integer(n) => DataValue::Integer(*n),
                DataValue::Float(f) => DataValue::Float(*f),
                DataValue::String(s) => {
                    // Try to parse string as number
                    if let Ok(n) = s.parse::<i64>() {
                        DataValue::Integer(n)
                    } else if let Ok(f) = s.parse::<f64>() {
                        DataValue::Float(f)
                    } else {
                        return Err(anyhow!(
                            "ARRAY argument {} ('{}') cannot be converted to a number",
                            i + 1,
                            s
                        ));
                    }
                }
                DataValue::Null => DataValue::Null,
                _ => {
                    return Err(anyhow!(
                        "ARRAY expects numeric arguments, got {:?} at position {}",
                        arg,
                        i + 1
                    ))
                }
            };

            table
                .add_row(DataRow::new(vec![DataValue::Integer(i as i64), value]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate a table from an array of literal values with index"
    }

    fn arg_count(&self) -> usize {
        0 // Variable number of arguments
    }
}
