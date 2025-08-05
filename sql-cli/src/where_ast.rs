use serde_json::Value;
use anyhow::Result;

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
    Like(String, String),
    IsNull(String),
    IsNotNull(String),
    
    // String methods
    Contains(String, String),
    StartsWith(String, String),
    EndsWith(String, String),
    ToLower(String, ComparisonOp, String),  // column.ToLower() == "value"
    ToUpper(String, ComparisonOp, String),  // column.ToUpper() == "VALUE"
    
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
}

pub fn format_where_ast(expr: &WhereExpr, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match expr {
        WhereExpr::And(left, right) => {
            format!("{}AND\n{}\n{}", 
                indent_str,
                format_where_ast(left, indent + 1),
                format_where_ast(right, indent + 1)
            )
        }
        WhereExpr::Or(left, right) => {
            format!("{}OR\n{}\n{}", 
                indent_str,
                format_where_ast(left, indent + 1),
                format_where_ast(right, indent + 1)
            )
        }
        WhereExpr::Not(inner) => {
            format!("{}NOT\n{}", 
                indent_str,
                format_where_ast(inner, indent + 1)
            )
        }
        WhereExpr::Equal(col, val) => {
            format!("{}EQUAL({}, {:?})", indent_str, col, val)
        }
        WhereExpr::NotEqual(col, val) => {
            format!("{}NOT_EQUAL({}, {:?})", indent_str, col, val)
        }
        WhereExpr::GreaterThan(col, val) => {
            format!("{}GREATER_THAN({}, {:?})", indent_str, col, val)
        }
        WhereExpr::GreaterThanOrEqual(col, val) => {
            format!("{}GREATER_THAN_OR_EQUAL({}, {:?})", indent_str, col, val)
        }
        WhereExpr::LessThan(col, val) => {
            format!("{}LESS_THAN({}, {:?})", indent_str, col, val)
        }
        WhereExpr::LessThanOrEqual(col, val) => {
            format!("{}LESS_THAN_OR_EQUAL({}, {:?})", indent_str, col, val)
        }
        WhereExpr::Between(col, lower, upper) => {
            format!("{}BETWEEN({}, {:?}, {:?})", indent_str, col, lower, upper)
        }
        WhereExpr::In(col, values) => {
            format!("{}IN({}, {:?})", indent_str, col, values)
        }
        WhereExpr::NotIn(col, values) => {
            format!("{}NOT_IN({}, {:?})", indent_str, col, values)
        }
        WhereExpr::Like(col, pattern) => {
            format!("{}LIKE({}, \"{}\")", indent_str, col, pattern)
        }
        WhereExpr::IsNull(col) => {
            format!("{}IS_NULL({})", indent_str, col)
        }
        WhereExpr::IsNotNull(col) => {
            format!("{}IS_NOT_NULL({})", indent_str, col)
        }
        WhereExpr::Contains(col, search) => {
            format!("{}CONTAINS({}, \"{}\")", indent_str, col, search)
        }
        WhereExpr::StartsWith(col, prefix) => {
            format!("{}STARTS_WITH({}, \"{}\")", indent_str, col, prefix)
        }
        WhereExpr::EndsWith(col, suffix) => {
            format!("{}ENDS_WITH({}, \"{}\")", indent_str, col, suffix)
        }
        WhereExpr::ToLower(col, op, value) => {
            format!("{}TO_LOWER({}, {:?}, \"{}\")", indent_str, col, op, value)
        }
        WhereExpr::ToUpper(col, op, value) => {
            format!("{}TO_UPPER({}, {:?}, \"{}\")", indent_str, col, op, value)
        }
        WhereExpr::Length(col, op, value) => {
            format!("{}LENGTH({}, {:?}, {})", indent_str, col, op, value)
        }
    }
}

pub fn evaluate_where_expr(expr: &WhereExpr, row: &Value) -> Result<bool> {
    match expr {
        WhereExpr::And(left, right) => {
            Ok(evaluate_where_expr(left, row)? && evaluate_where_expr(right, row)?)
        }
        WhereExpr::Or(left, right) => {
            Ok(evaluate_where_expr(left, row)? || evaluate_where_expr(right, row)?)
        }
        WhereExpr::Not(inner) => {
            Ok(!evaluate_where_expr(inner, row)?)
        }
        
        WhereExpr::Equal(column, value) => {
            if let Some(field_value) = row.get(column) {
                match (WhereValue::from_json(field_value), value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 == *s2),
                    (WhereValue::Number(n1), WhereValue::Number(n2)) => Ok((n1 - n2).abs() < f64::EPSILON),
                    (WhereValue::Null, WhereValue::Null) => Ok(true),
                    _ => Ok(false),
                }
            } else {
                Ok(matches!(value, WhereValue::Null))
            }
        }
        
        WhereExpr::NotEqual(column, value) => {
            Ok(!evaluate_where_expr(&WhereExpr::Equal(column.clone(), value.clone()), row)?)
        }
        
        WhereExpr::GreaterThan(column, value) => {
            if let Some(field_value) = row.get(column) {
                match (WhereValue::from_json(field_value), value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 > *s2),
                    (WhereValue::Number(n1), WhereValue::Number(n2)) => Ok(n1 > *n2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::GreaterThanOrEqual(column, value) => {
            if let Some(field_value) = row.get(column) {
                match (WhereValue::from_json(field_value), value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 >= *s2),
                    (WhereValue::Number(n1), WhereValue::Number(n2)) => Ok(n1 >= *n2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::LessThan(column, value) => {
            if let Some(field_value) = row.get(column) {
                match (WhereValue::from_json(field_value), value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 < *s2),
                    (WhereValue::Number(n1), WhereValue::Number(n2)) => Ok(n1 < *n2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::LessThanOrEqual(column, value) => {
            if let Some(field_value) = row.get(column) {
                match (WhereValue::from_json(field_value), value) {
                    (WhereValue::String(s1), WhereValue::String(s2)) => Ok(s1 <= *s2),
                    (WhereValue::Number(n1), WhereValue::Number(n2)) => Ok(n1 <= *n2),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::Between(column, lower, upper) => {
            if let Some(field_value) = row.get(column) {
                let val = WhereValue::from_json(field_value);
                match (&val, lower, upper) {
                    (WhereValue::String(s), WhereValue::String(l), WhereValue::String(u)) => {
                        Ok(s >= l && s <= u)
                    }
                    (WhereValue::Number(n), WhereValue::Number(l), WhereValue::Number(u)) => {
                        Ok(*n >= *l && *n <= *u)
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
        
        WhereExpr::Like(column, pattern) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    // Simple LIKE implementation: % = any chars, _ = single char
                    let regex_pattern = pattern
                        .replace("%", ".*")
                        .replace("_", ".");
                    
                    if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                        Ok(regex.is_match(s))
                    } else {
                        Ok(false)
                    }
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
                if let Some(s) = field_value.as_str() {
                    Ok(s.contains(search))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::StartsWith(column, prefix) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    Ok(s.starts_with(prefix))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        
        WhereExpr::EndsWith(column, suffix) => {
            if let Some(field_value) = row.get(column) {
                if let Some(s) = field_value.as_str() {
                    Ok(s.ends_with(suffix))
                } else {
                    Ok(false)
                }
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
    }
}