use crate::data::datatable::DataValue;
use std::cmp::Ordering;

/// Utility function to compare two `DataValues`, handling all types including `InternedString`
/// This centralizes comparison logic to avoid duplicating `InternedString` handling everywhere
#[must_use]
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

        // Vector comparisons (lexicographic)
        (DataValue::Vector(a), DataValue::Vector(b)) => {
            for (av, bv) in a.iter().zip(b.iter()) {
                match av.partial_cmp(bv).unwrap_or(Ordering::Equal) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            a.len().cmp(&b.len())
        }

        // Null handling
        (DataValue::Null, DataValue::Null) => Ordering::Equal,
        (DataValue::Null, _) => Ordering::Less,
        (_, DataValue::Null) => Ordering::Greater,

        // Cross-type comparisons - treat as unequal with consistent ordering
        // Order: Null < Boolean < Integer < Float < String/InternedString < DateTime < Vector
        (DataValue::Boolean(_), DataValue::Integer(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::Float(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::DateTime(_)) => Ordering::Less,
        (DataValue::Boolean(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::Integer(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::Integer(i), DataValue::Float(f)) => {
            // Compare actual numeric values, not types
            (*i as f64).partial_cmp(f).unwrap_or(Ordering::Equal)
        }
        (DataValue::Integer(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Integer(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Integer(_), DataValue::DateTime(_)) => Ordering::Less,
        (DataValue::Integer(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::Float(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::Float(f), DataValue::Integer(i)) => {
            // Compare actual numeric values, not types
            f.partial_cmp(&(*i as f64)).unwrap_or(Ordering::Equal)
        }
        (DataValue::Float(_), DataValue::String(_)) => Ordering::Less,
        (DataValue::Float(_), DataValue::InternedString(_)) => Ordering::Less,
        (DataValue::Float(_), DataValue::DateTime(_)) => Ordering::Less,
        (DataValue::Float(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::String(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::String(_), DataValue::DateTime(_)) => Ordering::Less,
        (DataValue::String(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::InternedString(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::InternedString(_), DataValue::DateTime(_)) => Ordering::Less,
        (DataValue::InternedString(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::DateTime(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::String(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::InternedString(_)) => Ordering::Greater,
        (DataValue::DateTime(_), DataValue::Vector(_)) => Ordering::Less,

        (DataValue::Vector(_), DataValue::Boolean(_)) => Ordering::Greater,
        (DataValue::Vector(_), DataValue::Integer(_)) => Ordering::Greater,
        (DataValue::Vector(_), DataValue::Float(_)) => Ordering::Greater,
        (DataValue::Vector(_), DataValue::String(_)) => Ordering::Greater,
        (DataValue::Vector(_), DataValue::InternedString(_)) => Ordering::Greater,
        (DataValue::Vector(_), DataValue::DateTime(_)) => Ordering::Greater,
    }
}

/// Is this cell NULL for ordering purposes?
///
/// A missing cell (`None`, e.g. a short row) and an explicit `DataValue::Null`
/// are the same thing to `ORDER BY` - both are the SQL NULL.
#[must_use]
pub fn is_null_for_ordering(v: Option<&DataValue>) -> bool {
    matches!(v, None | Some(DataValue::Null))
}

/// Compare two cells for `ORDER BY`, applying the direction and the NULL rule.
///
/// This is the single comparator behind every `ORDER BY` in the engine - the
/// top-level one and a window's internal one - so that they cannot drift apart
/// again (see P17).
///
/// Two rules, deliberately independent of each other:
/// - `ascending` reverses the comparison of two non-NULL values.
/// - `nulls_first` places NULLs **absolutely**, at the head or the tail of the
///   result. It is *not* reversed by `DESC`: `NULLS LAST` means last in the
///   output whichever direction the values are sorted in, which is what both
///   the SQL standard's explicit clause and DuckDB's default mean.
#[must_use]
pub fn compare_for_order_by(
    a: Option<&DataValue>,
    b: Option<&DataValue>,
    ascending: bool,
    nulls_first: bool,
) -> Ordering {
    match (is_null_for_ordering(a), is_null_for_ordering(b)) {
        (true, true) => return Ordering::Equal,
        (true, false) => {
            return if nulls_first {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (false, true) => {
            return if nulls_first {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (false, false) => {}
    }

    // Both non-NULL: the NULL arms of `compare_datavalues` are unreachable here.
    let cmp = compare_optional_datavalues(a, b);
    if ascending {
        cmp
    } else {
        cmp.reverse()
    }
}

/// Compare `DataValues` with optional values (handling None)
#[must_use]
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
        // Test the type ordering (except Integer/Float which compare by value)
        assert_eq!(
            compare_datavalues(&DataValue::Boolean(true), &DataValue::Integer(1)),
            Ordering::Less
        );

        // Integer and Float now compare by numeric value, not type
        assert_eq!(
            compare_datavalues(&DataValue::Integer(1), &DataValue::Float(1.0)),
            Ordering::Equal // 1 == 1.0
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(1), &DataValue::Float(1.5)),
            Ordering::Less // 1 < 1.5
        );
        assert_eq!(
            compare_datavalues(&DataValue::Integer(2), &DataValue::Float(1.5)),
            Ordering::Greater // 2 > 1.5
        );

        assert_eq!(
            compare_datavalues(&DataValue::Float(1.0), &DataValue::String("a".to_string())),
            Ordering::Less
        );
    }

    // ===== ORDER BY comparator (P17 / P13 stage 2) =====

    const NUM: DataValue = DataValue::Integer(5);

    fn cmp(a: Option<&DataValue>, b: Option<&DataValue>, asc: bool, nf: bool) -> Ordering {
        compare_for_order_by(a, b, asc, nf)
    }

    #[test]
    fn null_placement_is_absolute_not_reversed_by_desc() {
        // The whole point of the rule: NULLS LAST means last in the output, in
        // BOTH directions. A comparator that reversed the NULL arm along with
        // the values would pass the ASC half of this test and fail the DESC half.
        for ascending in [true, false] {
            assert_eq!(
                cmp(Some(&DataValue::Null), Some(&NUM), ascending, false),
                Ordering::Greater,
                "NULLS LAST, ascending={ascending}"
            );
            assert_eq!(
                cmp(Some(&DataValue::Null), Some(&NUM), ascending, true),
                Ordering::Less,
                "NULLS FIRST, ascending={ascending}"
            );
        }
    }

    #[test]
    fn default_is_nulls_last_in_both_directions() {
        // P17: the recorded choice. `nulls_first = false` is what
        // `OrderByItem::nulls_first()` returns for an unspecified clause.
        assert_eq!(
            cmp(Some(&NUM), Some(&DataValue::Null), true, false),
            Ordering::Less
        );
        assert_eq!(
            cmp(Some(&NUM), Some(&DataValue::Null), false, false),
            Ordering::Less
        );
    }

    #[test]
    fn missing_cell_and_explicit_null_are_the_same_null() {
        // A short row yields None; a parsed empty field yields DataValue::Null.
        // ORDER BY must not tell them apart, or NULL placement would depend on
        // how the row happened to be stored.
        assert_eq!(
            cmp(None, Some(&DataValue::Null), true, false),
            Ordering::Equal
        );
        assert_eq!(cmp(None, Some(&NUM), true, false), Ordering::Greater);
        assert_eq!(cmp(None, Some(&NUM), true, true), Ordering::Less);
    }

    #[test]
    fn direction_still_reverses_non_null_values() {
        let ten = DataValue::Integer(10);
        assert_eq!(cmp(Some(&NUM), Some(&ten), true, false), Ordering::Less);
        assert_eq!(cmp(Some(&NUM), Some(&ten), false, false), Ordering::Greater);
        // ...and NULL placement does not disturb that.
        assert_eq!(cmp(Some(&NUM), Some(&ten), false, true), Ordering::Greater);
    }

    #[test]
    fn order_by_compares_mixed_numerics_by_value() {
        // The window comparator used to reach `DataValue`'s derived PartialOrd,
        // which orders by variant: Integer always sorted before Float, and Null
        // (the last variant) sorted as the maximum. Both paths now share this
        // function, so this is the regression guard for that.
        assert_eq!(
            cmp(
                Some(&DataValue::Integer(100)),
                Some(&DataValue::Float(1.0)),
                true,
                false
            ),
            Ordering::Greater
        );
    }
}
