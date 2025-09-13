use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// RENDER_NUMBER function - Format numbers with various separators and abbreviations
pub struct RenderNumberFunction;

impl SqlFunction for RenderNumberFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RENDER_NUMBER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 3),
            description: "Format numbers with separators, abbreviations, or regional formats",
            returns: "STRING",
            examples: vec![
                "SELECT RENDER_NUMBER(1234567.89)",            // "1,234,567.89"
                "SELECT RENDER_NUMBER(1234567.89, 'compact')", // "1.2M"
                "SELECT RENDER_NUMBER(1234.56, 'eu')",         // "1.234,56"
                "SELECT RENDER_NUMBER(1500000, 'compact', 1)", // "1.5M"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RENDER_NUMBER requires a numeric value")),
        };

        let format = if args.len() >= 2 {
            match &args[1] {
                DataValue::String(s) => s.to_lowercase(),
                DataValue::Null => "standard".to_string(),
                _ => "standard".to_string(),
            }
        } else {
            "standard".to_string()
        };

        let decimals = if args.len() >= 3 {
            match &args[2] {
                DataValue::Integer(n) => *n as usize,
                DataValue::Float(f) => *f as usize,
                _ => 2,
            }
        } else {
            2
        };

        let formatted = match format.as_str() {
            "compact" => format_compact(value, decimals),
            "eu" | "european" => format_european(value, decimals),
            "ch" | "swiss" => format_swiss(value, decimals),
            "in" | "indian" => format_indian(value, decimals),
            _ => format_standard(value, decimals), // Default US/UK style
        };

        Ok(DataValue::String(formatted))
    }
}

/// FORMAT_CURRENCY function - Format numbers as currency with flexible options
pub struct FormatCurrencyFunction;

impl SqlFunction for FormatCurrencyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FORMAT_CURRENCY",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 4),
            description: "Format numbers as currency with symbols, codes, or names. Currency can be from a column.",
            returns: "STRING",
            examples: vec![
                "SELECT FORMAT_CURRENCY(1234.56, 'USD')",              // "$1,234.56"
                "SELECT FORMAT_CURRENCY(1234.56, 'GBP', 'symbol')",    // "£1,234.56"
                "SELECT FORMAT_CURRENCY(amount, currency_code)",       // Uses currency from column
                "SELECT FORMAT_CURRENCY(3000, 'GBP', 'compact_code')", // "3k GBP"
                "SELECT FORMAT_CURRENCY(1234.56, 'EUR', 'eu')",        // "1.234,56 €"
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("FORMAT_CURRENCY requires a numeric value")),
        };

        let currency_code = match &args[1] {
            DataValue::String(s) => s.to_uppercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "FORMAT_CURRENCY requires a currency code (e.g., 'USD', 'EUR', 'GBP')"
                ))
            }
        };

        let format = if args.len() >= 3 {
            match &args[2] {
                DataValue::String(s) => s.to_lowercase(),
                DataValue::Null => "symbol".to_string(),
                _ => "symbol".to_string(),
            }
        } else {
            "symbol".to_string()
        };

        let decimals_override = if args.len() >= 4 {
            match &args[3] {
                DataValue::Integer(n) => Some(*n as usize),
                DataValue::Float(f) => Some(*f as usize),
                _ => None,
            }
        } else {
            None
        };

        let currency_info = get_currency_info(&currency_code);
        let decimals = decimals_override.unwrap_or(currency_info.decimals as usize);

        let formatted = format_currency(value, &currency_code, &format, decimals, &currency_info);

        Ok(DataValue::String(formatted))
    }
}

// Currency information structure
struct CurrencyInfo {
    symbol: &'static str,
    name: &'static str,
    decimals: u8,
    symbol_before: bool,
}

// Get currency information
fn get_currency_info(code: &str) -> CurrencyInfo {
    match code {
        "USD" => CurrencyInfo {
            symbol: "$",
            name: "US dollar",
            decimals: 2,
            symbol_before: true,
        },
        "EUR" => CurrencyInfo {
            symbol: "€",
            name: "Euro",
            decimals: 2,
            symbol_before: true,
        },
        "GBP" => CurrencyInfo {
            symbol: "£",
            name: "British pound",
            decimals: 2,
            symbol_before: true,
        },
        "JPY" | "YEN" => CurrencyInfo {
            symbol: "¥",
            name: "Japanese yen",
            decimals: 0,
            symbol_before: true,
        },
        "CHF" => CurrencyInfo {
            symbol: "CHF",
            name: "Swiss franc",
            decimals: 2,
            symbol_before: true,
        },
        "CNY" | "RMB" => CurrencyInfo {
            symbol: "¥",
            name: "Chinese yuan",
            decimals: 2,
            symbol_before: true,
        },
        "INR" => CurrencyInfo {
            symbol: "₹",
            name: "Indian rupee",
            decimals: 2,
            symbol_before: true,
        },
        "AUD" => CurrencyInfo {
            symbol: "A$",
            name: "Australian dollar",
            decimals: 2,
            symbol_before: true,
        },
        "CAD" => CurrencyInfo {
            symbol: "C$",
            name: "Canadian dollar",
            decimals: 2,
            symbol_before: true,
        },
        "SEK" => CurrencyInfo {
            symbol: "kr",
            name: "Swedish krona",
            decimals: 2,
            symbol_before: false,
        },
        "NOK" => CurrencyInfo {
            symbol: "kr",
            name: "Norwegian krone",
            decimals: 2,
            symbol_before: false,
        },
        "DKK" => CurrencyInfo {
            symbol: "kr",
            name: "Danish krone",
            decimals: 2,
            symbol_before: false,
        },
        _ => CurrencyInfo {
            symbol: "¤", // Generic currency symbol
            name: "Unknown currency",
            decimals: 2,
            symbol_before: false,
        },
    }
}

// Format currency based on options
fn format_currency(
    value: f64,
    code: &str,
    format: &str,
    decimals: usize,
    info: &CurrencyInfo,
) -> String {
    let is_negative = value < 0.0;
    let abs_value = value.abs();

    match format {
        "compact" => {
            let compact = format_compact(abs_value, decimals.min(1));
            if info.symbol_before {
                format!(
                    "{}{}{}",
                    if is_negative { "-" } else { "" },
                    info.symbol,
                    compact
                )
            } else {
                format!(
                    "{}{}{}",
                    if is_negative { "-" } else { "" },
                    compact,
                    info.symbol
                )
            }
        }
        "compact_code" => {
            let compact = format_compact(abs_value, decimals.min(1));
            format!("{}{} {}", if is_negative { "-" } else { "" }, compact, code)
        }
        "code" => {
            let formatted = format_standard(abs_value, decimals);
            format!(
                "{}{} {}",
                if is_negative { "-" } else { "" },
                formatted,
                code
            )
        }
        "name" => {
            let formatted = format_standard(abs_value, decimals);
            let plural = if abs_value != 1.0 { "s" } else { "" };
            format!(
                "{}{} {}{}",
                if is_negative { "-" } else { "" },
                formatted,
                info.name,
                plural
            )
        }
        "eu" | "european" => {
            let formatted = format_european(abs_value, decimals);
            if code == "EUR" {
                format!("{}{} €", if is_negative { "-" } else { "" }, formatted)
            } else {
                format!(
                    "{}{} {}",
                    if is_negative { "-" } else { "" },
                    formatted,
                    code
                )
            }
        }
        "ch" | "swiss" => {
            let formatted = format_swiss(abs_value, decimals);
            format!(
                "{}{} {}",
                if is_negative { "-" } else { "" },
                code,
                formatted
            )
        }
        _ => {
            // Default "symbol" format
            let formatted = format_standard(abs_value, decimals);
            if info.symbol_before {
                format!(
                    "{}{}{}",
                    if is_negative { "-" } else { "" },
                    info.symbol,
                    formatted
                )
            } else {
                format!(
                    "{}{}{}",
                    if is_negative { "-" } else { "" },
                    formatted,
                    info.symbol
                )
            }
        }
    }
}

// Format with US/UK style (comma thousands, period decimal)
fn format_standard(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", value, prec = decimals);
    add_separators(&formatted, ',', '.')
}

// Format with European style (period thousands, comma decimal)
fn format_european(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", value, prec = decimals);
    add_separators(&formatted, '.', ',')
}

// Format with Swiss style (apostrophe thousands, period decimal)
fn format_swiss(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", value, prec = decimals);
    add_separators(&formatted, '\'', '.')
}

// Format with Indian style (lakhs and crores)
fn format_indian(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", value, prec = decimals);
    add_indian_separators(&formatted)
}

// Format in compact notation (k, M, B, T)
fn format_compact(value: f64, decimals: usize) -> String {
    let (num, suffix) = if value >= 1_000_000_000_000.0 {
        (value / 1_000_000_000_000.0, "T")
    } else if value >= 1_000_000_000.0 {
        (value / 1_000_000_000.0, "B")
    } else if value >= 1_000_000.0 {
        (value / 1_000_000.0, "M")
    } else if value >= 1_000.0 {
        (value / 1_000.0, "k")
    } else {
        (value, "")
    };

    if suffix.is_empty() {
        format!("{:.prec$}", num, prec = decimals)
    } else {
        // Remove trailing zeros and decimal point if not needed
        let formatted = format!("{:.prec$}", num, prec = decimals);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        format!("{}{}", trimmed, suffix)
    }
}

// Add thousand separators with specified characters
fn add_separators(formatted: &str, thousand_sep: char, decimal_sep: char) -> String {
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1);

    let mut result = String::new();
    let mut count = 0;

    for ch in integer_part.chars().rev() {
        if count == 3 {
            result.push(thousand_sep);
            count = 0;
        }
        result.push(ch);
        count += 1;
    }

    let mut final_result: String = result.chars().rev().collect();

    if let Some(dec) = decimal_part {
        final_result.push(decimal_sep);
        final_result.push_str(dec);
    }

    final_result
}

// Add Indian-style separators (lakhs and crores)
fn add_indian_separators(formatted: &str) -> String {
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1);

    let mut result = String::new();
    let chars: Vec<char> = integer_part.chars().rev().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i == 3 || (i > 3 && i % 2 == 1) {
            result.push(',');
        }
        result.push(*ch);
    }

    let mut final_result: String = result.chars().rev().collect();

    if let Some(dec) = decimal_part {
        final_result.push('.');
        final_result.push_str(dec);
    }

    final_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_number_standard() {
        let func = RenderNumberFunction;

        let result = func.evaluate(&[DataValue::Float(1234567.89)]).unwrap();
        assert_eq!(result, DataValue::String("1,234,567.89".to_string()));

        let result = func.evaluate(&[DataValue::Integer(1000)]).unwrap();
        assert_eq!(result, DataValue::String("1,000.00".to_string()));
    }

    #[test]
    fn test_render_number_compact() {
        let func = RenderNumberFunction;

        let result = func
            .evaluate(&[
                DataValue::Float(1500000.0),
                DataValue::String("compact".to_string()),
            ])
            .unwrap();
        assert_eq!(result, DataValue::String("1.5M".to_string()));

        let result = func
            .evaluate(&[
                DataValue::Integer(3000),
                DataValue::String("compact".to_string()),
            ])
            .unwrap();
        assert_eq!(result, DataValue::String("3k".to_string()));
    }

    #[test]
    fn test_format_currency() {
        let func = FormatCurrencyFunction;

        let result = func
            .evaluate(&[
                DataValue::Float(1234.56),
                DataValue::String("USD".to_string()),
            ])
            .unwrap();
        assert_eq!(result, DataValue::String("$1,234.56".to_string()));

        let result = func
            .evaluate(&[
                DataValue::Float(1234.56),
                DataValue::String("GBP".to_string()),
            ])
            .unwrap();
        assert_eq!(result, DataValue::String("£1,234.56".to_string()));

        let result = func
            .evaluate(&[
                DataValue::Integer(3000),
                DataValue::String("GBP".to_string()),
                DataValue::String("compact_code".to_string()),
            ])
            .unwrap();
        assert_eq!(result, DataValue::String("3k GBP".to_string()));
    }
}
