use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use figlet_rs::FIGfont;
use std::collections::HashMap;
use std::sync::Arc;

/// ASCII_ART - Generate ASCII art text using FIGlet fonts
pub struct AsciiArt;

impl TableGenerator for AsciiArt {
    fn name(&self) -> &str {
        "ASCII_ART"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn {
            name: "line".to_string(),
            data_type: DataType::String,
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
            return Err(anyhow!("ASCII_ART requires at least 1 argument (text)"));
        }

        // Get text to convert
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("ASCII_ART text cannot be NULL")),
            other => other.to_string(),
        };

        // Optional: font name (default to standard) - reserved for future use
        let _font_name = if args.len() > 1 {
            match &args[1] {
                DataValue::String(s) => s.clone(),
                DataValue::Null => "standard".to_string(),
                _ => "standard".to_string(),
            }
        } else {
            "standard".to_string()
        };

        // Load the font (for now, always use standard font)
        // In the future, we could add more fonts based on _font_name
        let font =
            FIGfont::standard().map_err(|e| anyhow!("Failed to load standard font: {}", e))?;

        // Convert text to ASCII art
        let ascii_art = font
            .convert(&text)
            .ok_or_else(|| anyhow!("Failed to convert text to ASCII art"))?;

        // Create table with one row per line
        let mut table = DataTable::new("ascii_art");
        table.add_column(DataColumn::new("line"));

        // Split the ASCII art into lines and add as rows
        for line in ascii_art.to_string().lines() {
            table
                .add_row(DataRow::new(vec![DataValue::String(line.to_string())]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate ASCII art text using FIGlet fonts"
    }

    fn arg_count(&self) -> usize {
        1 // Text is required, font is optional
    }
}

/// BIG_TEXT - Alias for ASCII_ART
pub struct BigText;

impl TableGenerator for BigText {
    fn name(&self) -> &str {
        "BIG_TEXT"
    }

    fn columns(&self) -> Vec<DataColumn> {
        AsciiArt.columns()
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        AsciiArt.generate(args)
    }

    fn description(&self) -> &str {
        "Generate big ASCII text (alias for ASCII_ART)"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// BANNER - Create a banner with ASCII art and optional border
pub struct Banner;

impl TableGenerator for Banner {
    fn name(&self) -> &str {
        "BANNER"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn {
            name: "line".to_string(),
            data_type: DataType::String,
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
            return Err(anyhow!("BANNER requires at least 1 argument (text)"));
        }

        // Get text to convert
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Err(anyhow!("BANNER text cannot be NULL")),
            other => other.to_string(),
        };

        // Optional: border character (default to '*')
        let border_char = if args.len() > 1 {
            match &args[1] {
                DataValue::String(s) if !s.is_empty() => s.chars().next().unwrap(),
                _ => '*',
            }
        } else {
            '*'
        };

        // Generate ASCII art
        let font =
            FIGfont::standard().map_err(|e| anyhow!("Failed to load standard font: {}", e))?;

        let ascii_art = font
            .convert(&text)
            .ok_or_else(|| anyhow!("Failed to convert text to ASCII art"))?;

        // Create table with banner
        let mut table = DataTable::new("banner");
        table.add_column(DataColumn::new("line"));

        // Find the maximum line width
        let lines: Vec<String> = ascii_art
            .to_string()
            .lines()
            .map(|s| s.to_string())
            .collect();
        let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

        // Create border line
        let border_line = border_char.to_string().repeat(max_width + 4);

        // Add top border
        table
            .add_row(DataRow::new(vec![DataValue::String(border_line.clone())]))
            .map_err(|e| anyhow!(e))?;

        // Add ASCII art lines with side borders
        for line in lines {
            let padded_line = format!(
                "{} {:<width$} {}",
                border_char,
                line,
                border_char,
                width = max_width
            );
            table
                .add_row(DataRow::new(vec![DataValue::String(padded_line)]))
                .map_err(|e| anyhow!(e))?;
        }

        // Add bottom border
        table
            .add_row(DataRow::new(vec![DataValue::String(border_line)]))
            .map_err(|e| anyhow!(e))?;

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate ASCII art banner with optional border"
    }

    fn arg_count(&self) -> usize {
        1 // Text is required, border character is optional
    }
}
