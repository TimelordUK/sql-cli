/// Centralized value comparison logic for consistent behavior across all query paths
use crate::data::datatable::DataValue;
use crate::data::value_parsing::parse_bool;
use crate::sql::functions::date_time::parse_datetime;
use std::cmp::Ordering;

/// Compare two DataValues for equality
pub fn values_equal(left: &DataValue, right: &DataValue, case_insensitive: bool) -> bool {
    compare_values(left, right, case_insensitive) == Some(Ordering::Equal)
}

/// Compare two DataValues for inequality
pub fn values_not_equal(left: &DataValue, right: &DataValue, case_insensitive: bool) -> bool {
    compare_values(left, right, case_insensitive) != Some(Ordering::Equal)
}

/// Compare two DataValues with a comparison operator
pub fn compare_with_op(
    left: &DataValue,
    right: &DataValue,
    op: &str,
    case_insensitive: bool,
) -> bool {
    let ordering = compare_values(left, right, case_insensitive);

    match (ordering, op) {
        (Some(Ordering::Equal), "=" | "==") => true,
        (Some(ord), "!=" | "<>") if ord != Ordering::Equal => true,
        (Some(Ordering::Less), "<") => true,
        (Some(ord), "<=") if ord != Ordering::Greater => true,
        (Some(Ordering::Greater), ">") => true,
        (Some(ord), ">=") if ord != Ordering::Less => true,
        _ => false,
    }
}

/// Core comparison logic that returns an Ordering
/// This is the single source of truth for all value comparisons
pub fn compare_values(
    left: &DataValue,
    right: &DataValue,
    case_insensitive: bool,
) -> Option<Ordering> {
    match (left, right) {
        // Null comparisons
        (DataValue::Null, DataValue::Null) => Some(Ordering::Equal),
        (DataValue::Null, _) | (_, DataValue::Null) => None, // NULL comparisons are undefined

        // Boolean comparisons
        (DataValue::Boolean(a), DataValue::Boolean(b)) => Some(a.cmp(b)),

        // Integer comparisons
        (DataValue::Integer(a), DataValue::Integer(b)) => Some(a.cmp(b)),

        // Float comparisons
        (DataValue::Float(a), DataValue::Float(b)) => {
            // Handle NaN properly
            if a.is_nan() && b.is_nan() {
                Some(Ordering::Equal)
            } else if a.is_nan() {
                Some(Ordering::Greater) // NaN sorts last
            } else if b.is_nan() {
                Some(Ordering::Less)
            } else {
                a.partial_cmp(b)
            }
        }

        // Mixed numeric comparisons
        (DataValue::Integer(a), DataValue::Float(b)) => {
            let a_float = *a as f64;
            if b.is_nan() {
                Some(Ordering::Less)
            } else {
                a_float.partial_cmp(b)
            }
        }
        (DataValue::Float(a), DataValue::Integer(b)) => {
            let b_float = *b as f64;
            if a.is_nan() {
                Some(Ordering::Greater)
            } else {
                a.partial_cmp(&b_float)
            }
        }

        // DateTime comparisons
        (DataValue::DateTime(a), DataValue::DateTime(b)) => {
            // Parse both as dates for comparison
            match (parse_datetime(a), parse_datetime(b)) {
                (Ok(a_dt), Ok(b_dt)) => Some(a_dt.cmp(&b_dt)),
                _ => Some(a.cmp(b)), // Fall back to string comparison
            }
        }

        // String comparisons
        (DataValue::String(a), DataValue::String(b)) => {
            // Try date comparison first
            if let (Ok(a_dt), Ok(b_dt)) = (parse_datetime(a), parse_datetime(b)) {
                return Some(a_dt.cmp(&b_dt));
            }

            // String comparison
            if case_insensitive {
                Some(a.to_lowercase().cmp(&b.to_lowercase()))
            } else {
                Some(a.cmp(b))
            }
        }

        // InternedString comparisons
        (DataValue::InternedString(a), DataValue::InternedString(b)) => {
            // Try date comparison first
            if let (Ok(a_dt), Ok(b_dt)) = (parse_datetime(a.as_str()), parse_datetime(b.as_str())) {
                return Some(a_dt.cmp(&b_dt));
            }

            // String comparison
            if case_insensitive {
                Some(a.to_lowercase().cmp(&b.to_lowercase()))
            } else {
                Some(a.as_str().cmp(b.as_str()))
            }
        }

        // Mixed String/InternedString comparisons
        (DataValue::String(a), DataValue::InternedString(b)) => {
            // Try date comparison first
            if let (Ok(a_dt), Ok(b_dt)) = (parse_datetime(a), parse_datetime(b.as_str())) {
                return Some(a_dt.cmp(&b_dt));
            }

            // String comparison
            if case_insensitive {
                Some(a.to_lowercase().cmp(&b.to_lowercase()))
            } else {
                Some(a.as_str().cmp(b))
            }
        }
        (DataValue::InternedString(a), DataValue::String(b)) => {
            // Try date comparison first
            if let (Ok(a_dt), Ok(b_dt)) = (parse_datetime(a.as_str()), parse_datetime(b)) {
                return Some(a_dt.cmp(&b_dt));
            }

            // String comparison
            if case_insensitive {
                Some(a.to_lowercase().cmp(&b.to_lowercase()))
            } else {
                Some(a.as_str().cmp(b))
            }
        }

        // DateTime vs String comparisons
        (DataValue::DateTime(dt), DataValue::String(s)) => {
            match (parse_datetime(dt), parse_datetime(s)) {
                (Ok(dt_val), Ok(s_val)) => Some(dt_val.cmp(&s_val)),
                _ => None,
            }
        }
        (DataValue::String(s), DataValue::DateTime(dt)) => {
            match (parse_datetime(s), parse_datetime(dt)) {
                (Ok(s_val), Ok(dt_val)) => Some(s_val.cmp(&dt_val)),
                _ => None,
            }
        }
        (DataValue::DateTime(dt), DataValue::InternedString(s)) => {
            match (parse_datetime(dt), parse_datetime(s.as_str())) {
                (Ok(dt_val), Ok(s_val)) => Some(dt_val.cmp(&s_val)),
                _ => None,
            }
        }
        (DataValue::InternedString(s), DataValue::DateTime(dt)) => {
            match (parse_datetime(s.as_str()), parse_datetime(dt)) {
                (Ok(s_val), Ok(dt_val)) => Some(s_val.cmp(&dt_val)),
                _ => None,
            }
        }

        // Cross-type comparisons with coercion

        // Boolean vs String
        (DataValue::Boolean(a), DataValue::String(b))
        | (DataValue::String(b), DataValue::Boolean(a)) => {
            // Try to parse string as boolean
            if let Some(b_bool) = parse_bool(b) {
                Some(a.cmp(&b_bool))
            } else {
                // Fall back to string comparison
                let a_str = a.to_string();
                if case_insensitive {
                    Some(a_str.to_lowercase().cmp(&b.to_lowercase()))
                } else {
                    Some(a_str.cmp(b))
                }
            }
        }

        // Boolean vs Integer (0/1 as false/true)
        (DataValue::Boolean(a), DataValue::Integer(b))
        | (DataValue::Integer(b), DataValue::Boolean(a)) => {
            match *b {
                0 => Some(a.cmp(&false)),
                1 => Some(a.cmp(&true)),
                _ => None, // Other integers don't compare to booleans
            }
        }

        // String vs Numeric
        (DataValue::String(a), DataValue::Integer(b))
        | (DataValue::Integer(b), DataValue::String(a)) => {
            if let Ok(a_int) = a.parse::<i64>() {
                Some(a_int.cmp(b))
            } else {
                None
            }
        }
        (DataValue::String(a), DataValue::Float(b))
        | (DataValue::Float(b), DataValue::String(a)) => {
            if let Ok(a_float) = a.parse::<f64>() {
                if b.is_nan() {
                    Some(Ordering::Less)
                } else {
                    a_float.partial_cmp(b)
                }
            } else {
                None
            }
        }

        // InternedString vs Numeric
        (DataValue::InternedString(a), DataValue::Integer(b))
        | (DataValue::Integer(b), DataValue::InternedString(a)) => {
            if let Ok(a_int) = a.parse::<i64>() {
                Some(a_int.cmp(b))
            } else {
                None
            }
        }
        (DataValue::InternedString(a), DataValue::Float(b))
        | (DataValue::Float(b), DataValue::InternedString(a)) => {
            if let Ok(a_float) = a.parse::<f64>() {
                if b.is_nan() {
                    Some(Ordering::Less)
                } else {
                    a_float.partial_cmp(b)
                }
            } else {
                None
            }
        }

        // InternedString vs Boolean
        (DataValue::Boolean(a), DataValue::InternedString(b)) => {
            if let Some(b_bool) = parse_bool(b.as_str()) {
                Some(a.cmp(&b_bool))
            } else {
                let a_str = a.to_string();
                if case_insensitive {
                    Some(a_str.to_lowercase().cmp(&b.to_lowercase()))
                } else {
                    Some(a_str.as_str().cmp(b.as_str()))
                }
            }
        }
        (DataValue::InternedString(a), DataValue::Boolean(b)) => {
            if let Some(a_bool) = parse_bool(a.as_str()) {
                Some(a_bool.cmp(b))
            } else {
                let b_str = b.to_string();
                if case_insensitive {
                    Some(a.to_lowercase().cmp(&b_str.to_lowercase()))
                } else {
                    Some(a.as_str().cmp(&b_str))
                }
            }
        }

        // Default: types don't compare
        _ => None,
    }
}

/// Helper to evaluate a comparison for WHERE clauses
/// Handles the special case of comparing a DataValue from the table with a literal value
pub fn compare_value_with_literal(
    table_value: &DataValue,
    op: &str,
    literal: &str,
    case_insensitive: bool,
) -> bool {
    // Parse the literal into the most appropriate DataValue type
    let literal_value = parse_literal(literal);
    compare_with_op(table_value, &literal_value, op, case_insensitive)
}

/// Parse a literal string from SQL into a DataValue
fn parse_literal(literal: &str) -> DataValue {
    // Try boolean literals
    if let Some(b) = parse_bool(literal) {
        return DataValue::Boolean(b);
    }

    // Try integer
    if let Ok(i) = literal.parse::<i64>() {
        return DataValue::Integer(i);
    }

    // Try float
    if let Ok(f) = literal.parse::<f64>() {
        return DataValue::Float(f);
    }

    // Try date/time
    if let Ok(dt) = parse_datetime(literal) {
        return DataValue::DateTime(dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string());
    }

    // Default to string
    DataValue::String(literal.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_comparisons() {
        let true_val = DataValue::Boolean(true);
        let false_val = DataValue::Boolean(false);

        assert!(values_equal(&true_val, &true_val, false));
        assert!(!values_equal(&true_val, &false_val, false));
        assert!(compare_with_op(&true_val, &false_val, ">", false));
    }

    #[test]
    fn test_boolean_string_coercion() {
        let bool_val = DataValue::Boolean(true);
        let str_val = DataValue::String("true".to_string());
        let num_str = DataValue::String("1".to_string());

        assert!(values_equal(&bool_val, &str_val, false));
        assert!(values_equal(&bool_val, &num_str, false));
    }

    #[test]
    fn test_boolean_integer_coercion() {
        let bool_true = DataValue::Boolean(true);
        let bool_false = DataValue::Boolean(false);
        let one = DataValue::Integer(1);
        let zero = DataValue::Integer(0);

        assert!(values_equal(&bool_true, &one, false));
        assert!(values_equal(&bool_false, &zero, false));
    }

    #[test]
    fn test_case_insensitive_strings() {
        let upper = DataValue::String("HELLO".to_string());
        let lower = DataValue::String("hello".to_string());

        assert!(!values_equal(&upper, &lower, false));
        assert!(values_equal(&upper, &lower, true));
    }
}
