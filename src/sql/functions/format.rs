use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// FORMAT_NUMBER function - Format numbers with specified decimal places and separators
pub struct FormatNumberFunction;

impl SqlFunction for FormatNumberFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FORMAT_NUMBER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 3),
            description: "Format a number with decimal places and thousand separators",
            returns: "STRING",
            examples: vec![
                "SELECT FORMAT_NUMBER(1234567.89, 2)",    // "1,234,567.89"
                "SELECT FORMAT_NUMBER(1234.5, 2, false)", // "1234.50" (no thousands sep)
                "SELECT FORMAT_NUMBER(1234567)",          // "1,234,567"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("FORMAT_NUMBER requires a numeric argument")),
        };

        let decimals = if args.len() >= 2 {
            match &args[1] {
                DataValue::Integer(n) => *n as usize,
                DataValue::Float(f) => *f as usize,
                _ => 2,
            }
        } else {
            0
        };

        let use_separator = if args.len() >= 3 {
            match &args[2] {
                DataValue::Boolean(b) => *b,
                DataValue::Integer(n) => *n != 0,
                _ => true,
            }
        } else {
            true
        };

        // Format with decimal places
        let formatted = if decimals > 0 {
            format!("{:.prec$}", value, prec = decimals)
        } else {
            format!("{:.0}", value)
        };

        // Add thousand separators if requested
        let result = if use_separator {
            add_thousand_separators(&formatted)
        } else {
            formatted
        };

        Ok(DataValue::String(result))
    }
}

fn add_thousand_separators(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1);

    let mut result = String::new();
    let mut count = 0;

    for ch in integer_part.chars().rev() {
        if count > 0 && count % 3 == 0 && ch != '-' {
            result.push(',');
        }
        result.push(ch);
        if ch != '-' {
            count += 1;
        }
    }

    let formatted_integer: String = result.chars().rev().collect();

    if let Some(dec) = decimal_part {
        format!("{}.{}", formatted_integer, dec)
    } else {
        formatted_integer
    }
}

/// LPAD function - Left pad a string to a specified length
pub struct LPadFunction;

impl SqlFunction for LPadFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LPAD",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Left pad a string to a specified length with a fill character",
            returns: "STRING",
            examples: vec![
                "SELECT LPAD('123', 5)",         // "  123"
                "SELECT LPAD('123', 5, '0')",    // "00123"
                "SELECT LPAD('hello', 10, '.')", // ".....hello"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LPAD requires a string or numeric argument")),
        };

        let length = match &args[1] {
            DataValue::Integer(n) if *n >= 0 => *n as usize,
            _ => return Err(anyhow!("LPAD length must be a non-negative integer")),
        };

        let pad_char = if args.len() >= 3 {
            match &args[2] {
                DataValue::String(s) if !s.is_empty() => s.chars().next().unwrap(),
                _ => ' ',
            }
        } else {
            ' '
        };

        if text.len() >= length {
            Ok(DataValue::String(text[..length].to_string()))
        } else {
            let padding = pad_char.to_string().repeat(length - text.len());
            Ok(DataValue::String(format!("{}{}", padding, text)))
        }
    }
}

/// RPAD function - Right pad a string to a specified length
pub struct RPadFunction;

impl SqlFunction for RPadFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RPAD",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Right pad a string to a specified length with a fill character",
            returns: "STRING",
            examples: vec![
                "SELECT RPAD('123', 5)",         // "123  "
                "SELECT RPAD('123', 5, '0')",    // "12300"
                "SELECT RPAD('hello', 10, '.')", // "hello....."
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RPAD requires a string or numeric argument")),
        };

        let length = match &args[1] {
            DataValue::Integer(n) if *n >= 0 => *n as usize,
            _ => return Err(anyhow!("RPAD length must be a non-negative integer")),
        };

        let pad_char = if args.len() >= 3 {
            match &args[2] {
                DataValue::String(s) if !s.is_empty() => s.chars().next().unwrap(),
                _ => ' ',
            }
        } else {
            ' '
        };

        if text.len() >= length {
            Ok(DataValue::String(text[..length].to_string()))
        } else {
            let padding = pad_char.to_string().repeat(length - text.len());
            Ok(DataValue::String(format!("{}{}", text, padding)))
        }
    }
}

/// CENTER function - Center a string within a specified width
pub struct CenterFunction;

impl SqlFunction for CenterFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CENTER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Center a string within a specified width",
            returns: "STRING",
            examples: vec![
                "SELECT CENTER('hello', 11)",     // "   hello   "
                "SELECT CENTER('test', 10, '.')", // "...test..."
                "SELECT CENTER('SQL', 7, '-')",   // "--SQL--"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CENTER requires a string or numeric argument")),
        };

        let width = match &args[1] {
            DataValue::Integer(n) if *n >= 0 => *n as usize,
            _ => return Err(anyhow!("CENTER width must be a non-negative integer")),
        };

        let pad_char = if args.len() >= 3 {
            match &args[2] {
                DataValue::String(s) if !s.is_empty() => s.chars().next().unwrap(),
                _ => ' ',
            }
        } else {
            ' '
        };

        if text.len() >= width {
            Ok(DataValue::String(text[..width].to_string()))
        } else {
            let total_padding = width - text.len();
            let left_padding = total_padding / 2;
            let right_padding = total_padding - left_padding;

            let left = pad_char.to_string().repeat(left_padding);
            let right = pad_char.to_string().repeat(right_padding);

            Ok(DataValue::String(format!("{}{}{}", left, text, right)))
        }
    }
}

/// FORMAT_DATE function - Format dates using format strings
pub struct FormatDateFunction;

impl SqlFunction for FormatDateFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FORMAT_DATE",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(2),
            description: "Format a date using a format string",
            returns: "STRING",
            examples: vec![
                "SELECT FORMAT_DATE(NOW(), '%Y-%m-%d')",      // "2024-03-15"
                "SELECT FORMAT_DATE(NOW(), '%B %d, %Y')",     // "March 15, 2024"
                "SELECT FORMAT_DATE(NOW(), '%Y%m%d_%H%M%S')", // "20240315_143022"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let datetime_str = match &args[0] {
            DataValue::DateTime(dt) => dt,
            DataValue::String(s) => s, // Also accept string dates
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "FORMAT_DATE requires a datetime or string argument"
                ))
            }
        };

        let format_str = match &args[1] {
            DataValue::String(s) => s,
            _ => return Err(anyhow!("FORMAT_DATE format must be a string")),
        };

        // Try to parse as ISO 8601 or common date formats

        let formatted = if let Ok(dt) = datetime_str.parse::<DateTime<Utc>>() {
            dt.format(format_str).to_string()
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
            dt.format(format_str).to_string()
        } else if let Ok(dt) = NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
            dt.format(format_str).to_string()
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S") {
            dt.format(format_str).to_string()
        } else {
            return Err(anyhow!("Cannot parse datetime: {}", datetime_str));
        };

        Ok(DataValue::String(formatted))
    }
}
