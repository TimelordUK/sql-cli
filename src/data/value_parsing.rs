/// Centralized value parsing utilities for consistent type detection and conversion
use crate::data::datatable::DataValue;
use crate::sql::functions::date_time::parse_datetime;

/// Parse a string into a boolean value using consistent rules across the codebase
/// Accepts: true/false, t/f, yes/no, y/n, 1/0 (case-insensitive except for numbers)
pub fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "t" | "yes" | "y" => Some(true),
        "false" | "f" | "no" | "n" => Some(false),
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// Parse a string into a boolean for CSV loading (excludes numeric forms)
/// Only accepts: true/false, t/f, yes/no, y/n (case-insensitive)
pub fn parse_bool_strict(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "t" | "yes" | "y" => Some(true),
        "false" | "f" | "no" | "n" => Some(false),
        _ => None,
    }
}

/// Check if a string can be parsed as a boolean
pub fn is_bool_str(s: &str) -> bool {
    parse_bool(s).is_some()
}

/// Check if a DataValue can be considered a boolean
pub fn is_bool_value(value: &DataValue) -> bool {
    match value {
        DataValue::Boolean(_) => true,
        DataValue::String(s) => is_bool_str(s),
        DataValue::InternedString(s) => is_bool_str(s.as_str()),
        DataValue::Integer(i) => *i == 0 || *i == 1,
        _ => false,
    }
}

/// Parse a string value into the appropriate DataValue type
/// Order of precedence: Null -> Boolean (text only) -> Integer -> Float -> DateTime -> String
/// Note: "1" and "0" are parsed as integers, not booleans, for data consistency
pub fn parse_value(field: &str, is_null: bool) -> DataValue {
    // Handle null/empty cases
    if is_null {
        return DataValue::Null;
    }

    if field.is_empty() {
        return DataValue::String(String::new());
    }

    // Try boolean first, but only text forms (not numeric "1"/"0")
    if let Some(b) = parse_bool_strict(field) {
        return DataValue::Boolean(b);
    }

    // Try integer
    if let Ok(i) = field.parse::<i64>() {
        return DataValue::Integer(i);
    }

    // Try float
    if let Ok(f) = field.parse::<f64>() {
        return DataValue::Float(f);
    }

    // Try date/time if it looks like one
    if looks_like_datetime(field) {
        if let Ok(dt) = parse_datetime(field) {
            // Store as ISO 8601 string for consistent comparisons
            return DataValue::DateTime(dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string());
        }
    }

    // Default to string
    DataValue::String(field.to_string())
}

/// Check if a string looks like it might be a date/time
fn looks_like_datetime(field: &str) -> bool {
    // Must contain date separators (-, /, or T) and be reasonable length
    (field.contains('-') || field.contains('/') || field.contains('T'))
        && field.len() >= 8  // Minimum for a date like "1/1/2024"
        && field.len() <= 30 // Maximum reasonable date length
        && !field.starts_with("--") // Avoid things like "--option"
        && field.chars().filter(|c| c.is_ascii_digit()).count() >= 4 // At least 4 digits
}
