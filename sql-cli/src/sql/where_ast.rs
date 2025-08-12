use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum WhereExpr {
    // Logical operators
    And(Box<WhereExpr>, Box<WhereExpr>),
    Or(Box<WhereExpr>, Box<WhereExpr>),
    Not(Box<WhereExpr>),

    // Comparison operators
    Equal(String, WhereValue),
    NotEqual(String, WhereValue),
    GreaterThan(String, WhereValue),
    GreaterThanOrEqual(String, WhereValue),
    LessThan(String, WhereValue),
    LessThanOrEqual(String, WhereValue),

    // Special operators
    Between(String, WhereValue, WhereValue),
    In(String, Vec<WhereValue>),
    NotIn(String, Vec<WhereValue>),
    InIgnoreCase(String, Vec<WhereValue>),
    NotInIgnoreCase(String, Vec<WhereValue>),
    Like(String, String),
    IsNull(String),
    IsNotNull(String),

    // String methods
    Contains(String, String),
    StartsWith(String, String),
    EndsWith(String, String),
    ContainsIgnoreCase(String, String), // Case-insensitive contains
    StartsWithIgnoreCase(String, String), // Case-insensitive starts with
    EndsWithIgnoreCase(String, String), // Case-insensitive ends with
    ToLower(String, ComparisonOp, String), // column.ToLower() == "value"
    ToUpper(String, ComparisonOp, String), // column.ToUpper() == "VALUE"
    IsNullOrEmpty(String),              // String.IsNullOrEmpty(column)

    // Numeric methods
    Length(String, ComparisonOp, i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhereValue {
    String(String),
    Number(f64),
    Null,
}

impl WhereValue {
    pub fn from_json(value: &Value) -> Self {
        match value {
            Value::String(s) => WhereValue::String(s.clone()),
            Value::Number(n) => WhereValue::Number(n.as_f64().unwrap_or(0.0)),
            Value::Null => WhereValue::Null,
            _ => WhereValue::Null,
        }
    }

    /// Try to parse a string as a number, supporting scientific notation
    fn try_parse_number(s: &str) -> Option<f64> {
        // Parse the string as f64, which handles scientific notation (1e-4, 1.5E+3, etc.)
        s.parse::<f64>().ok().filter(|n| n.is_finite())
    }

    /// Try to coerce values for numeric comparison
    /// Returns (left_value, right_value) if coercion is possible
    fn try_coerce_numeric(left: &WhereValue, right: &WhereValue) -> Option<(f64, f64)> {
        match (left, right) {
            // Both are already numbers
            (WhereValue::Number(n1), WhereValue::Number(n2)) => Some((*n1, *n2)),

            // Left is string, right is number - try to parse left
            (WhereValue::String(s), WhereValue::Number(n)) => {
                Self::try_parse_number(s).map(|parsed| (parsed, *n))
            }

            // Left is number, right is string - try to parse right
            (WhereValue::Number(n), WhereValue::String(s)) => {
                Self::try_parse_number(s).map(|parsed| (*n, parsed))
            }

            // Both are strings - try to parse both for numeric comparison
            (WhereValue::String(s1), WhereValue::String(s2)) => {
                match (Self::try_parse_number(s1), Self::try_parse_number(s2)) {
                    (Some(n1), Some(n2)) => Some((n1, n2)),
                    _ => None,
                }
            }

            _ => None,
        }
    }
}

pub fn format_where_ast(expr: &WhereExpr, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match expr {
        WhereExpr::And(left, right) => {
            format!(
                "{}AND\n{}\n{}",
                indent_str,
                format_where_ast(left, indent + 1),
                format_where_ast(right, indent + 1)
            )
        }
        WhereExpr::Or(left, right) => {
            format!(
                "{}OR\n{}\n{}",
                indent_str,
                format_where_ast(left, indent + 1),
                format_where_ast(right, indent + 1)
            )
        }
        WhereExpr::Not(inner) => {
            format!("{}NOT\n{}", indent_str, format_where_ast(inner, indent + 1))
        }
        WhereExpr::Equal(col, val) => {
            format!("{indent_str}EQUAL({col}, {val:?})")
        }
        WhereExpr::NotEqual(col, val) => {
            format!("{indent_str}NOT_EQUAL({col}, {val:?})")
        }
        WhereExpr::GreaterThan(col, val) => {
            format!("{indent_str}GREATER_THAN({col}, {val:?})")
        }
        WhereExpr::GreaterThanOrEqual(col, val) => {
            format!("{indent_str}GREATER_THAN_OR_EQUAL({col}, {val:?})")
        }
        WhereExpr::LessThan(col, val) => {
            format!("{indent_str}LESS_THAN({col}, {val:?})")
        }
        WhereExpr::LessThanOrEqual(col, val) => {
            format!("{indent_str}LESS_THAN_OR_EQUAL({col}, {val:?})")
        }
        WhereExpr::Between(col, lower, upper) => {
            format!("{indent_str}BETWEEN({col}, {lower:?}, {upper:?})")
        }
        WhereExpr::In(col, values) => {
            format!("{indent_str}IN({col}, {values:?})")
        }
        WhereExpr::NotIn(col, values) => {
            format!("{indent_str}NOT_IN({col}, {values:?})")
        }
        WhereExpr::InIgnoreCase(col, values) => {
            format!("{indent_str}IN_IGNORE_CASE({col}, {values:?})")
        }
        WhereExpr::NotInIgnoreCase(col, values) => {
            format!("{indent_str}NOT_IN_IGNORE_CASE({col}, {values:?})")
        }
        WhereExpr::Like(col, pattern) => {
            format!("{indent_str}LIKE({col}, \"{pattern}\")")
        }
        WhereExpr::IsNull(col) => {
            format!("{indent_str}IS_NULL({col})")
        }
        WhereExpr::IsNotNull(col) => {
            format!("{indent_str}IS_NOT_NULL({col})")
        }
        WhereExpr::Contains(col, search) => {
            format!("{indent_str}CONTAINS({col}, \"{search}\")")
        }
        WhereExpr::StartsWith(col, prefix) => {
            format!("{indent_str}STARTS_WITH({col}, \"{prefix}\")")
        }
        WhereExpr::EndsWith(col, suffix) => {
            format!("{indent_str}ENDS_WITH({col}, \"{suffix}\")")
        }
        WhereExpr::ContainsIgnoreCase(col, search) => {
            format!("{indent_str}CONTAINS_IGNORE_CASE({col}, \"{search}\")")
        }
        WhereExpr::StartsWithIgnoreCase(col, prefix) => {
            format!("{indent_str}STARTS_WITH_IGNORE_CASE({col}, \"{prefix}\")")
        }
        WhereExpr::EndsWithIgnoreCase(col, suffix) => {
            format!("{indent_str}ENDS_WITH_IGNORE_CASE({col}, \"{suffix}\")")
        }
        WhereExpr::ToLower(col, op, value) => {
            format!("{indent_str}TO_LOWER({col}, {op:?}, \"{value}\")")
        }
        WhereExpr::ToUpper(col, op, value) => {
            format!("{indent_str}TO_UPPER({col}, {op:?}, \"{value}\")")
        }
        WhereExpr::Length(col, op, value) => {
            format!("{indent_str}LENGTH({col}, {op:?}, {value})")
        }
        WhereExpr::IsNullOrEmpty(col) => {
            format!("{indent_str}IS_NULL_OR_EMPTY({col})")
        }
    }
}

pub fn evaluate_where_expr(expr: &WhereExpr, row: &Value) -> Result<bool> {
    evaluate_where_expr_with_options(expr, row, false)
}

pub fn evaluate_where_expr_with_options(
    expr: &WhereExpr,
    row: &Value,
    case_insensitive: bool,
) -> Result<bool> {
    match expr {
        WhereExpr::And(left, right) => {
            Ok(
                evaluate_where_expr_with_options(left, row, case_insensitive)?
                    && evaluate_where_expr_with_options(right, row, case_insensitive)?,
            )
        }
        WhereExpr::Or(left, right) => {
            Ok(
                evaluate_where_expr_with_options(left, row, case_insensitive)?
                    || evaluate_where_expr_with_options(right, row, case_insensitive)?,
            )
        }
        WhereExpr::Not(inner) => Ok(!evaluate_where_expr_with_options(
            inner,
            row,
            case_insensitive,
        )?),

        WhereExpr::Equal(column, value) => {
            if let Some(field_value) = row.get(column) {
                let left = WhereValue::from_json(field_value);

                // Try numeric comparison first for potential numeric values
                if let Some((n1, n2)) = WhereValue::try_coerce_numeric(&left, value) {
                    return Ok((n1 - n2).abs() < f64::EPSILON);
                }

                // Fall back to string matching (with optional case insensitivity)
                match (&left, value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => {
                        if case_insensitive {
                            Ok(s1.to_lowercase() == s2.to_lowercase())
                        } else {
                            Ok(s1 == s2)
                        }
                    }
                    (WhereValue::Null, WhereValue::Null) => Ok(true),
                    _ => Ok(false),
                }
            } else {
                Ok(matches!(value, WhereValue::Null))
            }
        }

        WhereExpr::NotEqual(column, value) => Ok(!evaluate_where_expr_with_options(
            &WhereExpr::Equal(column.clone(), value.clone()),
            row,
            case_insensitive,
        )?),

        WhereExpr::GreaterThan(column, value) => {
            if let Some(field_value) = row.get(column) {
                let left = WhereValue::from_json(field_value);

                // Try numeric comparison first
                if let Some((n1, n2)) = WhereValue::try_coerce_numeric(&left, value) {
                    return Ok(n1 > n2);
                }

                // Fall back to string comparison
                match (&left, value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 > s2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::GreaterThanOrEqual(column, value) => {
            if let Some(field_value) = row.get(column) {
                let left = WhereValue::from_json(field_value);

                // Try numeric comparison first
                if let Some((n1, n2)) = WhereValue::try_coerce_numeric(&left, value) {
                    return Ok(n1 >= n2);
                }

                // Fall back to string comparison
                match (&left, value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 >= s2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::LessThan(column, value) => {
            if let Some(field_value) = row.get(column) {
                let left = WhereValue::from_json(field_value);

                // Try numeric comparison first
                if let Some((n1, n2)) = WhereValue::try_coerce_numeric(&left, value) {
                    return Ok(n1 < n2);
                }

                // Fall back to string comparison
                match (&left, value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 < s2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::LessThanOrEqual(column, value) => {
            if let Some(field_value) = row.get(column) {
                let left = WhereValue::from_json(field_value);

                // Try numeric comparison first
                if let Some((n1, n2)) = WhereValue::try_coerce_numeric(&left, value) {
                    return Ok(n1 <= n2);
                }

                // Fall back to string comparison
                match (&left, value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 <= s2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::Between(column, lower, upper) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);

                // Try numeric comparison first
                if let (Some((v, l)), Some((_v2, u))) = (
                    WhereValue::try_coerce_numeric(&val, lower),
                    WhereValue::try_coerce_numeric(&val, upper),
                ) {
                    // Both coercions succeeded, use the value from first coercion
                    return Ok(v >= l && v <= u);
                }

                // Fall back to string comparison
                match (&val, lower, upper) {
                    (WhereValue::String(s), WhereValue::String(l), WhereValue::String(u)) => {
                        Ok(s >= l && s <= u)
                    }
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::In(column, values) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);
                Ok(values.contains(&val))
            } else {
                Ok(false)
            }
        }

        WhereExpr::NotIn(column, values) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);
                Ok(!values.contains(&val))
            } else {
                Ok(true) // NULL is not in any list
            }
        }

        WhereExpr::InIgnoreCase(column, values) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);
                // For case-insensitive comparison, convert to lowercase
                if let WhereValue::String(ref field_str) = val {
                    let field_lower = field_str.to_lowercase();
                    Ok(values.iter().any(|v| {
                        if let WhereValue::String(s) = v {
                            s.to_lowercase() == field_lower
                        } else {
                            v == &val
                        }
                    }))
                } else {
                    Ok(values.contains(&val))
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::NotInIgnoreCase(column, values) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);
                // For case-insensitive comparison, convert to lowercase
                if let WhereValue::String(ref field_str) = val {
                    let field_lower = field_str.to_lowercase();
                    Ok(!values.iter().any(|v| {
                        if let WhereValue::String(s) = v {
                            s.to_lowercase() == field_lower
                        } else {
                            v == &val
                        }
                    }))
                } else {
                    Ok(!values.contains(&val))
                }
            } else {
                Ok(true) // NULL is not in any list
            }
        }

        WhereExpr::Like(column, pattern) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };

                // Simple LIKE implementation: % = any chars, _ = single char
                let regex_pattern = pattern.replace("%", ".*").replace("_", ".");

                if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                    Ok(regex.is_match(&str_value))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::IsNull(column) => {
            if let Some(field_value) = row.get(column) {
                Ok(field_value.is_null())
            } else {
                Ok(true) // Missing field is considered NULL
            }
        }

        WhereExpr::IsNotNull(column) => {
            if let Some(field_value) = row.get(column) {
                Ok(!field_value.is_null())
            } else {
                Ok(false) // Missing field is considered NULL
            }
        }

        WhereExpr::Contains(column, search) => {
            if let Some(field_value) = row.get(column) {
                // Try as string first, then coerce other types to string
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(), // For arrays/objects, use JSON representation
                };
                Ok(str_value.contains(search))
            } else {
                Ok(false)
            }
        }

        WhereExpr::StartsWith(column, prefix) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };
                Ok(str_value.starts_with(prefix))
            } else {
                Ok(false)
            }
        }

        WhereExpr::EndsWith(column, suffix) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };
                Ok(str_value.ends_with(suffix))
            } else {
                Ok(false)
            }
        }

        WhereExpr::ContainsIgnoreCase(column, search) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };
                Ok(str_value.to_lowercase().contains(&search.to_lowercase()))
            } else {
                Ok(false)
            }
        }

        WhereExpr::StartsWithIgnoreCase(column, prefix) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };
                Ok(str_value.to_lowercase().starts_with(&prefix.to_lowercase()))
            } else {
                Ok(false)
            }
        }

        WhereExpr::EndsWithIgnoreCase(column, suffix) => {
            if let Some(field_value) = row.get(column) {
                let str_value = match field_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => return Ok(false),
                    _ => field_value.to_string(),
                };
                Ok(str_value.to_lowercase().ends_with(&suffix.to_lowercase()))
            } else {
                Ok(false)
            }
        }

        WhereExpr::ToLower(column, op, value) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    let lower_s = s.to_lowercase();
                    Ok(match op {
                        ComparisonOp::Equal => lower_s == *value,
                        ComparisonOp::NotEqual => lower_s != *value,
                        ComparisonOp::GreaterThan => lower_s > *value,
                        ComparisonOp::GreaterThanOrEqual => lower_s >= *value,
                        ComparisonOp::LessThan => lower_s < *value,
                        ComparisonOp::LessThanOrEqual => lower_s <= *value,
                    })
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::ToUpper(column, op, value) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    let upper_s = s.to_uppercase();
                    Ok(match op {
                        ComparisonOp::Equal => upper_s == *value,
                        ComparisonOp::NotEqual => upper_s != *value,
                        ComparisonOp::GreaterThan => upper_s > *value,
                        ComparisonOp::GreaterThanOrEqual => upper_s >= *value,
                        ComparisonOp::LessThan => upper_s < *value,
                        ComparisonOp::LessThanOrEqual => upper_s <= *value,
                    })
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::Length(column, op, value) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    let len = s.len() as i64;
                    Ok(match op {
                        ComparisonOp::Equal => len == *value,
                        ComparisonOp::NotEqual => len != *value,
                        ComparisonOp::GreaterThan => len > *value,
                        ComparisonOp::GreaterThanOrEqual => len >= *value,
                        ComparisonOp::LessThan => len < *value,
                        ComparisonOp::LessThanOrEqual => len <= *value,
                    })
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        WhereExpr::IsNullOrEmpty(column) => {
            if let Some(field_value) = row.get(column) {
                if field_value.is_null() {
                    Ok(true)
                } else if let Some(s) = field_value.as_str() {
                    Ok(s.is_empty())
                } else {
                    Ok(false)
                }
            } else {
                Ok(true) // Missing field is considered null/empty
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_null_or_empty_with_null() {
        let row = json!({
            "name": null,
            "age": 25
        });

        // Test that null values return true
        let expr = WhereExpr::IsNullOrEmpty("name".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), true);
    }

    #[test]
    fn test_is_null_or_empty_with_empty_string() {
        let row = json!({
            "name": "",
            "age": 25
        });

        // Test that empty strings return true
        let expr = WhereExpr::IsNullOrEmpty("name".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), true);
    }

    #[test]
    fn test_is_null_or_empty_with_non_empty_string() {
        let row = json!({
            "name": "John",
            "age": 25
        });

        // Test that non-empty strings return false
        let expr = WhereExpr::IsNullOrEmpty("name".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), false);
    }

    #[test]
    fn test_is_null_or_empty_with_missing_field() {
        let row = json!({
            "age": 25
        });

        // Test that missing fields are considered null/empty
        let expr = WhereExpr::IsNullOrEmpty("name".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), true);
    }

    #[test]
    fn test_is_null_or_empty_with_whitespace() {
        let row = json!({
            "name": "   ",
            "description": " \t\n "
        });

        // Test that whitespace-only strings are NOT considered empty
        // (following standard IsNullOrEmpty behavior)
        let expr = WhereExpr::IsNullOrEmpty("name".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), false);

        let expr2 = WhereExpr::IsNullOrEmpty("description".to_string());
        assert_eq!(evaluate_where_expr(&expr2, &row).unwrap(), false);
    }

    #[test]
    fn test_is_null_or_empty_with_number_field() {
        let row = json!({
            "count": 0,
            "price": 100.5
        });

        // Test that numeric fields return false (not strings)
        let expr = WhereExpr::IsNullOrEmpty("count".to_string());
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), false);

        let expr2 = WhereExpr::IsNullOrEmpty("price".to_string());
        assert_eq!(evaluate_where_expr(&expr2, &row).unwrap(), false);
    }

    #[test]
    fn test_is_null_or_empty_in_complex_expression() {
        let row = json!({
            "name": "",
            "age": 25,
            "city": "New York"
        });

        // Test: name.IsNullOrEmpty() AND age > 20
        let expr = WhereExpr::And(
            Box::new(WhereExpr::IsNullOrEmpty("name".to_string())),
            Box::new(WhereExpr::GreaterThan(
                "age".to_string(),
                WhereValue::Number(20.0),
            )),
        );
        assert_eq!(evaluate_where_expr(&expr, &row).unwrap(), true);

        // Test: name.IsNullOrEmpty() OR city = "Boston"
        let expr2 = WhereExpr::Or(
            Box::new(WhereExpr::IsNullOrEmpty("name".to_string())),
            Box::new(WhereExpr::Equal(
                "city".to_string(),
                WhereValue::String("Boston".to_string()),
            )),
        );
        assert_eq!(evaluate_where_expr(&expr2, &row).unwrap(), true); // true because name is empty

        // Test: NOT name.IsNullOrEmpty()
        let expr3 = WhereExpr::Not(Box::new(WhereExpr::IsNullOrEmpty("name".to_string())));
        assert_eq!(evaluate_where_expr(&expr3, &row).unwrap(), false);
    }
}
