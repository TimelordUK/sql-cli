//! Shared type inference logic for data loaders
//!
//! This module provides centralized type detection logic to ensure
//! consistent behavior across CSV, JSON, and other data sources.

use regex::Regex;
use std::sync::LazyLock;

/// Static compiled regex patterns for date detection
/// Using LazyLock for thread-safe initialization
static DATE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // YYYY-MM-DD (year must be 19xx or 20xx, month 01-12, day 01-31)
        Regex::new(r"^(19|20)\d{2}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$").unwrap(),
        // MM/DD/YYYY
        Regex::new(r"^(0[1-9]|1[0-2])/(0[1-9]|[12]\d|3[01])/(19|20)\d{2}$").unwrap(),
        // DD/MM/YYYY
        Regex::new(r"^(0[1-9]|[12]\d|3[01])/(0[1-9]|1[0-2])/(19|20)\d{2}$").unwrap(),
        // DD-MM-YYYY
        Regex::new(r"^(0[1-9]|[12]\d|3[01])-(0[1-9]|1[0-2])-(19|20)\d{2}$").unwrap(),
        // YYYY/MM/DD
        Regex::new(r"^(19|20)\d{2}/(0[1-9]|1[0-2])/(0[1-9]|[12]\d|3[01])$").unwrap(),
        // ISO 8601 with time: YYYY-MM-DDTHH:MM:SS
        Regex::new(r"^(19|20)\d{2}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])T\d{2}:\d{2}:\d{2}")
            .unwrap(),
        // ISO 8601 with timezone: YYYY-MM-DDTHH:MM:SS+/-HH:MM or Z
        Regex::new(
            r"^(19|20)\d{2}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$",
        )
        .unwrap(),
    ]
});

/// Detected data type for a value or column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferredType {
    Boolean,
    Integer,
    Float,
    DateTime,
    String,
    Null,
}

/// Type inference utilities
pub struct TypeInference;

impl TypeInference {
    /// Infer the type of a single string value
    ///
    /// This is the main entry point for type detection.
    /// Order of checks is important for performance and accuracy.
    pub fn infer_from_string(value: &str) -> InferredType {
        // Empty values are null
        if value.is_empty() {
            return InferredType::Null;
        }

        // Check boolean first (fast string comparison)
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            return InferredType::Boolean;
        }

        // Try integer (common case, relatively fast)
        if value.parse::<i64>().is_ok() {
            return InferredType::Integer;
        }

        // Try float (includes scientific notation)
        if value.parse::<f64>().is_ok() {
            return InferredType::Float;
        }

        // Check if it looks like a datetime
        // This is the most expensive check, so we do it last
        if Self::looks_like_datetime(value) {
            return InferredType::DateTime;
        }

        // Default to string
        InferredType::String
    }

    /// Check if a string looks like a datetime value
    ///
    /// Uses strict regex patterns to avoid false positives with ID strings
    /// like "BQ-123456" or "ORDER-2024-001"
    pub fn looks_like_datetime(value: &str) -> bool {
        // Quick length check - dates are typically 8-30 chars
        if value.len() < 8 || value.len() > 35 {
            return false;
        }

        // Check against our compiled patterns
        DATE_PATTERNS.iter().any(|pattern| pattern.is_match(value))
    }

    /// Merge two types when a column has mixed types
    ///
    /// Rules:
    /// - Same type -> keep it
    /// - Null with anything -> the other type
    /// - Integer + Float -> Float
    /// - Any numeric + String -> String
    /// - DateTime + String -> String
    /// - Everything else -> String
    pub fn merge_types(type1: InferredType, type2: InferredType) -> InferredType {
        use InferredType::*;

        match (type1, type2) {
            // Same type
            (t1, t2) if t1 == t2 => t1,

            // Null merges to the other type
            (Null, t) | (t, Null) => t,

            // Integer and Float -> Float
            (Integer, Float) | (Float, Integer) => Float,

            // Boolean stays boolean only with itself or null
            (Boolean, _) | (_, Boolean) => String,

            // DateTime only compatible with itself or null
            (DateTime, _) | (_, DateTime) => String,

            // Default to String for mixed types
            _ => String,
        }
    }

    /// Infer type from multiple sample values
    ///
    /// Useful for determining column type from a sample of rows.
    /// Returns the most specific type that fits all non-null values.
    pub fn infer_from_samples<'a, I>(values: I) -> InferredType
    where
        I: Iterator<Item = &'a str>,
    {
        let mut result_type = InferredType::Null;

        for value in values {
            let value_type = Self::infer_from_string(value);
            result_type = Self::merge_types(result_type, value_type);

            // Early exit if we've degraded to String
            if result_type == InferredType::String {
                break;
            }
        }

        result_type
    }

    /// Check if a value can be coerced to a specific type
    pub fn can_coerce_to(value: &str, target_type: InferredType) -> bool {
        match target_type {
            InferredType::Boolean => {
                value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("false")
                    || value == "0"
                    || value == "1"
            }
            InferredType::Integer => value.parse::<i64>().is_ok(),
            InferredType::Float => value.parse::<f64>().is_ok(),
            InferredType::DateTime => Self::looks_like_datetime(value),
            InferredType::String => true, // Everything can be a string
            InferredType::Null => value.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_inference() {
        assert_eq!(
            TypeInference::infer_from_string("123"),
            InferredType::Integer
        );
        assert_eq!(
            TypeInference::infer_from_string("123.45"),
            InferredType::Float
        );
        assert_eq!(
            TypeInference::infer_from_string("true"),
            InferredType::Boolean
        );
        assert_eq!(
            TypeInference::infer_from_string("FALSE"),
            InferredType::Boolean
        );
        assert_eq!(
            TypeInference::infer_from_string("hello"),
            InferredType::String
        );
        assert_eq!(TypeInference::infer_from_string(""), InferredType::Null);
    }

    #[test]
    fn test_datetime_detection() {
        // Valid dates should be detected
        assert_eq!(
            TypeInference::infer_from_string("2024-01-15"),
            InferredType::DateTime
        );
        assert_eq!(
            TypeInference::infer_from_string("01/15/2024"),
            InferredType::DateTime
        );
        assert_eq!(
            TypeInference::infer_from_string("15-01-2024"),
            InferredType::DateTime
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-01-15T10:30:00"),
            InferredType::DateTime
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-01-15T10:30:00Z"),
            InferredType::DateTime
        );
    }

    #[test]
    fn test_id_strings_not_detected_as_datetime() {
        // These should be detected as String, not DateTime
        assert_eq!(
            TypeInference::infer_from_string("BQ-81198596"),
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("ORDER-2024-001"),
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("ID-123-456"),
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("ABC-DEF-GHI"),
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-ABC-123"),
            InferredType::String
        );
    }

    #[test]
    fn test_invalid_dates_not_detected() {
        // Invalid month/day combinations
        assert_eq!(
            TypeInference::infer_from_string("2024-13-01"), // Month 13
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-00-15"), // Month 00
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-01-32"), // Day 32
            InferredType::String
        );
        assert_eq!(
            TypeInference::infer_from_string("2024-01-00"), // Day 00
            InferredType::String
        );
    }

    #[test]
    fn test_type_merging() {
        use InferredType::*;

        // Same type
        assert_eq!(TypeInference::merge_types(Integer, Integer), Integer);
        assert_eq!(TypeInference::merge_types(String, String), String);

        // Null with anything
        assert_eq!(TypeInference::merge_types(Null, Integer), Integer);
        assert_eq!(TypeInference::merge_types(Float, Null), Float);

        // Integer and Float
        assert_eq!(TypeInference::merge_types(Integer, Float), Float);
        assert_eq!(TypeInference::merge_types(Float, Integer), Float);

        // Mixed types degrade to String
        assert_eq!(TypeInference::merge_types(Integer, String), String);
        assert_eq!(TypeInference::merge_types(DateTime, Integer), String);
        assert_eq!(TypeInference::merge_types(Boolean, Float), String);
    }

    #[test]
    fn test_infer_from_samples() {
        // All integers
        let samples = vec!["1", "2", "3", "4", "5"];
        assert_eq!(
            TypeInference::infer_from_samples(samples.into_iter()),
            InferredType::Integer
        );

        // Mixed integer and float
        let samples = vec!["1", "2.5", "3", "4.0"];
        assert_eq!(
            TypeInference::infer_from_samples(samples.into_iter()),
            InferredType::Float
        );

        // Mixed types degrade to string
        let samples = vec!["1", "hello", "3"];
        assert_eq!(
            TypeInference::infer_from_samples(samples.into_iter()),
            InferredType::String
        );

        // With nulls (empty strings)
        let samples = vec!["", "1", "", "2", "3"];
        assert_eq!(
            TypeInference::infer_from_samples(samples.into_iter()),
            InferredType::Integer
        );
    }

    #[test]
    fn test_can_coerce() {
        // Boolean coercion
        assert!(TypeInference::can_coerce_to("true", InferredType::Boolean));
        assert!(TypeInference::can_coerce_to("1", InferredType::Boolean));
        assert!(TypeInference::can_coerce_to("0", InferredType::Boolean));
        assert!(!TypeInference::can_coerce_to(
            "hello",
            InferredType::Boolean
        ));

        // Integer coercion
        assert!(TypeInference::can_coerce_to("123", InferredType::Integer));
        assert!(!TypeInference::can_coerce_to(
            "123.45",
            InferredType::Integer
        ));
        assert!(!TypeInference::can_coerce_to(
            "hello",
            InferredType::Integer
        ));

        // Everything can be a string
        assert!(TypeInference::can_coerce_to("123", InferredType::String));
        assert!(TypeInference::can_coerce_to("hello", InferredType::String));
        assert!(TypeInference::can_coerce_to("", InferredType::String));
    }
}
