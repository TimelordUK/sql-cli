use anyhow::{anyhow, Result};
use std::cmp::Ordering;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Helper to compare two DataValues
/// Returns None if values are incomparable (different types that can't be coerced)
fn compare_values(a: &DataValue, b: &DataValue) -> Option<Ordering> {
    match (a, b) {
        // Null handling - NULL is considered smallest
        (DataValue::Null, DataValue::Null) => Some(Ordering::Equal),
        (DataValue::Null, _) => Some(Ordering::Less),
        (_, DataValue::Null) => Some(Ordering::Greater),

        // Same type comparisons
        (DataValue::Integer(x), DataValue::Integer(y)) => Some(x.cmp(y)),
        (DataValue::Float(x), DataValue::Float(y)) => {
            // Handle NaN values
            if x.is_nan() && y.is_nan() {
                Some(Ordering::Equal)
            } else if x.is_nan() {
                Some(Ordering::Less) // NaN is treated as smallest
            } else if y.is_nan() {
                Some(Ordering::Greater)
            } else {
                x.partial_cmp(y)
            }
        }
        (DataValue::String(x), DataValue::String(y)) => Some(x.cmp(y)),
        (DataValue::InternedString(x), DataValue::InternedString(y)) => Some(x.cmp(y)),
        (DataValue::String(x), DataValue::InternedString(y))
        | (DataValue::InternedString(y), DataValue::String(x)) => Some(x.as_str().cmp(y.as_str())),
        (DataValue::Boolean(x), DataValue::Boolean(y)) => Some(x.cmp(y)),
        (DataValue::DateTime(x), DataValue::DateTime(y)) => Some(x.cmp(y)),

        // Numeric coercion - allow comparing integers and floats
        (DataValue::Integer(x), DataValue::Float(y)) => {
            let x_float = *x as f64;
            if y.is_nan() {
                Some(Ordering::Greater)
            } else {
                x_float.partial_cmp(y)
            }
        }
        (DataValue::Float(x), DataValue::Integer(y)) => {
            let y_float = *y as f64;
            if x.is_nan() {
                Some(Ordering::Less)
            } else {
                x.partial_cmp(&y_float)
            }
        }

        // Different types that can't be compared
        _ => None,
    }
}

/// GREATEST function - returns the maximum value from a list of values
pub struct GreatestFunction;

impl SqlFunction for GreatestFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "GREATEST",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns the greatest value from a list of values",
            returns: "ANY",
            examples: vec![
                "SELECT GREATEST(10, 20, 5)",
                "SELECT GREATEST(salary, bonus, commission) as max_pay FROM employees",
                "SELECT GREATEST('apple', 'banana', 'cherry')",
                "SELECT GREATEST(date1, date2, date3) as latest_date",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("GREATEST requires at least one argument"));
        }

        // Start with the first non-null value, or return NULL if all are NULL
        let mut greatest = None;

        for arg in args {
            match &greatest {
                None => {
                    // First value or all previous were NULL
                    if !matches!(arg, DataValue::Null) {
                        greatest = Some(arg.clone());
                    }
                }
                Some(current) => {
                    // Compare with current greatest
                    match compare_values(arg, current) {
                        Some(Ordering::Greater) => {
                            greatest = Some(arg.clone());
                        }
                        Some(_) => {
                            // Keep current greatest
                        }
                        None => {
                            // Type mismatch - can't compare
                            return Err(anyhow!(
                                "GREATEST: Cannot compare values of different types: {:?} and {:?}",
                                current,
                                arg
                            ));
                        }
                    }
                }
            }
        }

        // If all values were NULL, return NULL
        Ok(greatest.unwrap_or(DataValue::Null))
    }
}

/// LEAST function - returns the minimum value from a list of values
pub struct LeastFunction;

impl SqlFunction for LeastFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LEAST",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns the smallest value from a list of values",
            returns: "ANY",
            examples: vec![
                "SELECT LEAST(10, 20, 5)",
                "SELECT LEAST(salary, min_wage) as lower_bound FROM employees",
                "SELECT LEAST('apple', 'banana', 'cherry')",
                "SELECT LEAST(date1, date2, date3) as earliest_date",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("LEAST requires at least one argument"));
        }

        // Start with the first non-null value, or return NULL if all are NULL
        let mut least = None;

        for arg in args {
            match &least {
                None => {
                    // First value or all previous were NULL
                    if !matches!(arg, DataValue::Null) {
                        least = Some(arg.clone());
                    }
                }
                Some(current) => {
                    // Compare with current least
                    match compare_values(arg, current) {
                        Some(Ordering::Less) => {
                            least = Some(arg.clone());
                        }
                        Some(_) => {
                            // Keep current least
                        }
                        None => {
                            // Type mismatch - can't compare
                            return Err(anyhow!(
                                "LEAST: Cannot compare values of different types: {:?} and {:?}",
                                current,
                                arg
                            ));
                        }
                    }
                }
            }
        }

        // If all values were NULL, return NULL
        Ok(least.unwrap_or(DataValue::Null))
    }
}

/// COALESCE function - returns the first non-null value
pub struct CoalesceFunction;

impl SqlFunction for CoalesceFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "COALESCE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns the first non-null value from a list",
            returns: "ANY",
            examples: vec![
                "SELECT COALESCE(NULL, 'default', 'backup')",
                "SELECT COALESCE(phone, mobile, email) as contact FROM users",
                "SELECT COALESCE(discount, 0) as discount_amount",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("COALESCE requires at least one argument"));
        }

        // Return the first non-null value
        for arg in args {
            if !matches!(arg, DataValue::Null) {
                return Ok(arg.clone());
            }
        }

        // All values were NULL
        Ok(DataValue::Null)
    }
}

/// NULLIF function - returns NULL if two values are equal
pub struct NullIfFunction;

impl SqlFunction for NullIfFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NULLIF",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Returns NULL if two values are equal, otherwise returns the first value",
            returns: "ANY",
            examples: vec![
                "SELECT NULLIF(0, 0)", // Returns NULL
                "SELECT NULLIF(price, 0) as non_zero_price",
                "SELECT NULLIF(status, 'DELETED') as active_status",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let val1 = &args[0];
        let val2 = &args[1];

        // Check if values are equal
        match compare_values(val1, val2) {
            Some(Ordering::Equal) => Ok(DataValue::Null),
            Some(_) => Ok(val1.clone()),
            None => {
                // Different types - they can't be equal
                Ok(val1.clone())
            }
        }
    }
}

/// Register all comparison functions
pub fn register_comparison_functions(registry: &mut super::FunctionRegistry) {
    registry.register(Box::new(GreatestFunction));
    registry.register(Box::new(LeastFunction));
    registry.register(Box::new(CoalesceFunction));
    registry.register(Box::new(NullIfFunction));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greatest_with_integers() {
        let func = GreatestFunction;
        let args = vec![
            DataValue::Integer(10),
            DataValue::Integer(5),
            DataValue::Integer(20),
            DataValue::Integer(15),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(20));
    }

    #[test]
    fn test_greatest_with_floats() {
        let func = GreatestFunction;
        let args = vec![
            DataValue::Float(10.5),
            DataValue::Float(5.2),
            DataValue::Float(20.8),
            DataValue::Float(15.3),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Float(20.8));
    }

    #[test]
    fn test_greatest_with_mixed_numbers() {
        let func = GreatestFunction;
        let args = vec![
            DataValue::Integer(10),
            DataValue::Float(15.5),
            DataValue::Integer(20),
            DataValue::Float(12.3),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(20));
    }

    #[test]
    fn test_greatest_with_nulls() {
        let func = GreatestFunction;
        let args = vec![
            DataValue::Null,
            DataValue::Integer(10),
            DataValue::Null,
            DataValue::Integer(5),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(10));
    }

    #[test]
    fn test_greatest_all_nulls() {
        let func = GreatestFunction;
        let args = vec![DataValue::Null, DataValue::Null, DataValue::Null];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_greatest_with_strings() {
        let func = GreatestFunction;
        let args = vec![
            DataValue::String("apple".to_string()),
            DataValue::String("banana".to_string()),
            DataValue::String("cherry".to_string()),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::String("cherry".to_string()));
    }

    #[test]
    fn test_least_with_integers() {
        let func = LeastFunction;
        let args = vec![
            DataValue::Integer(10),
            DataValue::Integer(5),
            DataValue::Integer(20),
            DataValue::Integer(15),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(5));
    }

    #[test]
    fn test_least_with_nulls() {
        let func = LeastFunction;
        let args = vec![
            DataValue::Integer(10),
            DataValue::Null,
            DataValue::Integer(5),
            DataValue::Integer(20),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(5));
    }

    #[test]
    fn test_coalesce() {
        let func = CoalesceFunction;
        let args = vec![
            DataValue::Null,
            DataValue::Null,
            DataValue::String("first".to_string()),
            DataValue::String("second".to_string()),
        ];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::String("first".to_string()));
    }

    #[test]
    fn test_nullif_equal() {
        let func = NullIfFunction;
        let args = vec![DataValue::Integer(5), DataValue::Integer(5)];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_nullif_not_equal() {
        let func = NullIfFunction;
        let args = vec![DataValue::Integer(5), DataValue::Integer(10)];

        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Integer(5));
    }
}
