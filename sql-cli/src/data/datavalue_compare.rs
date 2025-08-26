use crate::data::datatable::DataValue;
use std::cmp::Ordering;

/// Utility function to compare two DataValues, handling all types including InternedString
/// This centralizes comparison logic to avoid duplicating InternedString handling everywhere
pub fn compare_datavalues(a: &DataValue, b: &DataValue) -> Ordering {
    match (a, b) {
        // Integer comparisons
        (DataValue::Integer(a), DataValue::Integer(b)) => a.cmp(b),

        // Float comparisons
        (DataValue::Float(a), DataValue::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),

        // String comparisons
        (DataValue::String(a), DataValue::String(b)) => a.cmp(b),

        // InternedString comparisons
        (DataValue::InternedString(a), DataValue::InternedString(b)) => a.as_ref().cmp(b.as_ref()),

        // Mixed String and InternedString comparisons
        (DataValue::String(a), DataValue::InternedString(b)) => a.cmp(b.as_ref()),
        (DataValue::InternedString(a), DataValue::String(b)) => a.as_ref().cmp(b),

        // Boolean comparisons
        (DataValue::Boolean(a), DataValue::Boolean(b)) => a.cmp(b),

        // DateTime comparisons
        (DataValue::DateTime(a), DataValue::DateTime(b)) => a.cmp(b),

        // Null handling
        (DataValue::Null, DataValue::Null) => Ordering::Equal,
        (DataValue::Null, _) => Ordering::Less,
        (_, DataValue::Null) => Ordering::Greater,

        // Cross-type comparisons - treat as unequal with consistent ordering
        // Order: Null < Boolean < Integer < Float < String/InternedString < DateTime
        (DataValue::Boolean(_), DataValue::Integer(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::Float(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::DateTime(_)) => Ordering::Less,

        (DataValue::Integer(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::Integer(i), DataValue::Float(f)) => {
            // Compare actual numeric values, not types
            (*i as f64).partial_cmp(f).unwrap_or(Ordering::Equal)
        }
        (DataValue::Integer(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Integer(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Integer(_), DataValue::DateTime(_)) => Ordering::Less,

        (DataValue::Float(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::Float(f), DataValue::Integer(i)) => {
            // Compare actual numeric values, not types
            f.partial_cmp(&(*i as f64)).unwrap_or(Ordering::Equal)
        }
        (DataValue::Float(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Float(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Float(_), DataValue::DateTime(_)) => Ordering::Less,

        (DataValue::String(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::DateTime(_)) => Ordering::Less,

        (DataValue::InternedString(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::DateTime(_)) => Ordering::Less,

        (DataValue::DateTime(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::String(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::InternedString(_)) => Ordering::Greater,
    }
}

/// Compare DataValues with optional values (handling None)
pub fn compare_optional_datavalues(a: Option<&DataValue>, b: Option<&DataValue>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(a), Some(b)) => compare_datavalues(a, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_integer_comparison() {
        assert_eq!(
            compare_datavalues(&DataValue::Integer(1), &DataValue::Integer(2)),
            Ordering::Less
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(2), &DataValue::Integer(2)),
            Ordering::Equal
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(3), &DataValue::Integer(2)),
            Ordering::Greater
        );
    }

    #[test]
    fn test_string_comparison() {
        assert_eq!(
            compare_datavalues(
                &DataValue::String("apple".to_string()),
                &DataValue::String("banana".to_string())
            ),
            Ordering::Less
        );
    }

    #[test]
    fn test_interned_string_comparison() {
        let a = Arc::new("apple".to_string());
        let b = Arc::new("banana".to_string());
        assert_eq!(
            compare_datavalues(&DataValue::InternedString(a), &DataValue::InternedString(b)),
            Ordering::Less
        );
    }

    #[test]
    fn test_mixed_string_comparison() {
        let interned = Arc::new("banana".to_string());
        assert_eq!(
            compare_datavalues(
                &DataValue::String("apple".to_string()),
                &DataValue::InternedString(interned.clone())
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_datavalues(
                &DataValue::InternedString(interned),
                &DataValue::String("apple".to_string())
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn test_null_comparison() {
        assert_eq!(
            compare_datavalues(&DataValue::Null, &DataValue::Integer(1)),
            Ordering::Less
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(1), &DataValue::Null),
            Ordering::Greater
        );
        assert_eq!(
            compare_datavalues(&DataValue::Null, &DataValue::Null),
            Ordering::Equal
        );
    }

    #[test]
    fn test_cross_type_comparison() {
        // Test the type ordering
        assert_eq!(
            compare_datavalues(&DataValue::Boolean(true), &DataValue::Integer(1)),
            Ordering::Less
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(1), &DataValue::Float(1.0)),
            Ordering::Less
        );
        assert_eq!(
            compare_datavalues(&DataValue::Float(1.0), &DataValue::String("a".to_string())),
            Ordering::Less
        );
    }
}
