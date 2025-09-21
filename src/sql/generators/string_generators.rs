use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// SPLIT - Split a string into rows based on delimiter
pub struct Split;

impl TableGenerator for Split {
    fn name(&self) -> &str {
        "SPLIT"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn {
                name: "value".to_string(),
                data_type: DataType::String,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
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
        ]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() {
            return Err(anyhow!(
                "SPLIT requires at least 1 argument (text to split)"
            ));
        }

        // Get text to split
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("SPLIT text cannot be NULL")),
            other => other.to_string(),
        };

        // Get delimiter (default to space)
        let delimiter = if args.len() > 1 {
            match &args[1] {
                DataValue::String(s) => s.clone(),
                DataValue::Null => " ".to_string(),
                other => other.to_string(),
            }
        } else {
            " ".to_string()
        };

        let mut table = DataTable::new("split");
        table.add_column(DataColumn::new("value"));
        table.add_column(DataColumn::new("index"));

        // Split the string and create rows
        if delimiter.is_empty() {
            // Split into individual characters
            for (idx, ch) in text.chars().enumerate() {
                table
                    .add_row(DataRow::new(vec![
                        DataValue::String(ch.to_string()),
                        DataValue::Integer((idx + 1) as i64),
                    ]))
                    .map_err(|e| anyhow!(e))?;
            }
        } else {
            // Split by delimiter
            for (idx, part) in text.split(&delimiter).enumerate() {
                // Skip empty parts
                if part.is_empty() {
                    continue;
                }

                table
                    .add_row(DataRow::new(vec![
                        DataValue::String(part.to_string()),
                        DataValue::Integer((idx + 1) as i64),
                    ]))
                    .map_err(|e| anyhow!(e))?;
            }
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Split a string into rows based on delimiter"
    }

    fn arg_count(&self) -> usize {
        2 // text and optional delimiter
    }
}

/// TOKENIZE - Extract words/tokens from text (similar to SPLIT but with normalization)
pub struct Tokenize;

impl TableGenerator for Tokenize {
    fn name(&self) -> &str {
        "TOKENIZE"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn {
                name: "token".to_string(),
                data_type: DataType::String,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
            DataColumn {
                name: "position".to_string(),
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
            return Err(anyhow!(
                "TOKENIZE requires at least 1 argument (text to tokenize)"
            ));
        }

        // Get text to tokenize
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("TOKENIZE text cannot be NULL")),
            other => other.to_string(),
        };

        // Get case option (default to preserve case)
        let case_option = if args.len() > 1 {
            match &args[1] {
                DataValue::String(s) => s.to_lowercase(),
                _ => "preserve".to_string(),
            }
        } else {
            "preserve".to_string()
        };

        let mut table = DataTable::new("tokenize");
        table.add_column(DataColumn::new("token"));
        table.add_column(DataColumn::new("position"));

        // Tokenize by splitting on non-alphanumeric characters
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in text.chars() {
            if ch.is_alphanumeric() {
                current_token.push(ch);
            } else if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        }

        // Don't forget the last token
        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        // Apply case transformation
        let tokens = match case_option.as_str() {
            "lower" | "lowercase" => tokens.iter().map(|t| t.to_lowercase()).collect(),
            "upper" | "uppercase" => tokens.iter().map(|t| t.to_uppercase()).collect(),
            _ => tokens,
        };

        // Create rows
        for (idx, token) in tokens.iter().enumerate() {
            table
                .add_row(DataRow::new(vec![
                    DataValue::String(token.clone()),
                    DataValue::Integer((idx + 1) as i64),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Extract alphanumeric tokens from text"
    }

    fn arg_count(&self) -> usize {
        2 // text and optional case option
    }
}

/// CHARS - Split string into individual characters
pub struct Chars;

impl TableGenerator for Chars {
    fn name(&self) -> &str {
        "CHARS"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn {
                name: "char".to_string(),
                data_type: DataType::String,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
            DataColumn {
                name: "position".to_string(),
                data_type: DataType::Integer,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
            DataColumn {
                name: "ascii".to_string(),
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
            return Err(anyhow!("CHARS requires 1 argument (text)"));
        }

        // Get text
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("CHARS text cannot be NULL")),
            other => other.to_string(),
        };

        let mut table = DataTable::new("chars");
        table.add_column(DataColumn::new("char"));
        table.add_column(DataColumn::new("position"));
        table.add_column(DataColumn::new("ascii"));

        // Create a row for each character
        for (idx, ch) in text.chars().enumerate() {
            table
                .add_row(DataRow::new(vec![
                    DataValue::String(ch.to_string()),
                    DataValue::Integer((idx + 1) as i64),
                    DataValue::Integer(ch as i64),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Split string into individual characters with ASCII codes"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// LINES - Split text into lines
pub struct Lines;

impl TableGenerator for Lines {
    fn name(&self) -> &str {
        "LINES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn {
                name: "line".to_string(),
                data_type: DataType::String,
                nullable: false,
                unique_values: Some(0),
                null_count: 0,
                metadata: HashMap::new(),
                qualified_name: None,
                source_table: None,
            },
            DataColumn {
                name: "line_number".to_string(),
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
            return Err(anyhow!("LINES requires 1 argument (text)"));
        }

        // Get text
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("LINES text cannot be NULL")),
            other => other.to_string(),
        };

        let mut table = DataTable::new("lines");
        table.add_column(DataColumn::new("line"));
        table.add_column(DataColumn::new("line_number"));

        // Split into lines
        for (idx, line) in text.lines().enumerate() {
            table
                .add_row(DataRow::new(vec![
                    DataValue::String(line.to_string()),
                    DataValue::Integer((idx + 1) as i64),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Split text into lines"
    }

    fn arg_count(&self) -> usize {
        1
    }
}
