//! Row Expansion System
//!
//! This module provides a generic framework for functions that multiply rows.
//! Unlike regular functions (value → value) or generators (args → table),
//! row expanders take a single input row and produce multiple output rows.
//!
//! ## Use Cases
//! - UNNEST: Split delimited strings into separate rows
//! - EXPLODE: Expand JSON arrays
//! - GENERATE_SERIES: Create multiple rows from a range
//! - Custom expansions for domain-specific data

use crate::data::datatable::DataValue;
use anyhow::Result;

/// Result of row expansion: a vector of values for this column across N rows
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// The expanded values (one per output row)
    pub values: Vec<DataValue>,
}

impl ExpansionResult {
    /// Create a new expansion result
    pub fn new(values: Vec<DataValue>) -> Self {
        Self { values }
    }

    /// Number of rows this expansion will create
    pub fn row_count(&self) -> usize {
        self.values.len()
    }
}

/// Trait for row expander implementations
///
/// Row expanders are special expressions that cause a single input row
/// to be multiplied into multiple output rows.
pub trait RowExpander: Send + Sync {
    /// Name of the expander (e.g., "UNNEST")
    fn name(&self) -> &str;

    /// Description of what this expander does
    fn description(&self) -> &str;

    /// Expand a single value into multiple values
    ///
    /// # Arguments
    /// * `value` - The input value to expand (e.g., delimited string)
    /// * `args` - Additional arguments (e.g., delimiter)
    ///
    /// # Returns
    /// An ExpansionResult containing the array of values for output rows
    fn expand(&self, value: &DataValue, args: &[DataValue]) -> Result<ExpansionResult>;
}

/// Registry of available row expanders
pub struct RowExpanderRegistry {
    expanders: std::collections::HashMap<String, Box<dyn RowExpander>>,
}

impl RowExpanderRegistry {
    /// Create a new registry with default expanders
    pub fn new() -> Self {
        let mut registry = Self {
            expanders: std::collections::HashMap::new(),
        };

        // Register built-in expanders
        registry.register(Box::new(unnest::UnnestExpander));

        registry
    }

    /// Register a new expander
    pub fn register(&mut self, expander: Box<dyn RowExpander>) {
        self.expanders
            .insert(expander.name().to_uppercase(), expander);
    }

    /// Get an expander by name
    pub fn get(&self, name: &str) -> Option<&dyn RowExpander> {
        self.expanders.get(&name.to_uppercase()).map(|e| e.as_ref())
    }

    /// Check if an expander exists
    pub fn contains(&self, name: &str) -> bool {
        self.expanders.contains_key(&name.to_uppercase())
    }

    /// List all registered expanders
    pub fn list(&self) -> Vec<&str> {
        self.expanders.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for RowExpanderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// UNNEST expander implementation
pub mod unnest {
    use super::*;

    pub struct UnnestExpander;

    impl RowExpander for UnnestExpander {
        fn name(&self) -> &str {
            "UNNEST"
        }

        fn description(&self) -> &str {
            "Split a delimited string into multiple rows"
        }

        fn expand(&self, value: &DataValue, args: &[DataValue]) -> Result<ExpansionResult> {
            // Get the delimiter (first argument)
            let delimiter = match args.first() {
                Some(DataValue::String(s)) => s.as_str(),
                Some(_) => {
                    return Err(anyhow::anyhow!(
                        "UNNEST delimiter must be a string, got {:?}",
                        args.first()
                    ))
                }
                None => return Err(anyhow::anyhow!("UNNEST requires a delimiter argument")),
            };

            // Convert value to string
            let text = match value {
                DataValue::String(s) => s.clone(),
                DataValue::Null => {
                    // NULL expands to a single NULL
                    return Ok(ExpansionResult::new(vec![DataValue::Null]));
                }
                other => other.to_string(),
            };

            // Split the string
            let parts: Vec<DataValue> = if delimiter.is_empty() {
                // Empty delimiter: split into characters
                text.chars()
                    .map(|ch| DataValue::String(ch.to_string()))
                    .collect()
            } else {
                // Split by delimiter, filter out empty parts
                text.split(delimiter)
                    .filter(|s| !s.is_empty())
                    .map(|s| DataValue::String(s.to_string()))
                    .collect()
            };

            // If no parts, return single NULL to maintain row presence
            if parts.is_empty() {
                Ok(ExpansionResult::new(vec![DataValue::Null]))
            } else {
                Ok(ExpansionResult::new(parts))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnest_basic() {
        let expander = unnest::UnnestExpander;
        let value = DataValue::String("A|B|C".to_string());
        let delimiter = DataValue::String("|".to_string());

        let result = expander.expand(&value, &[delimiter]).unwrap();
        assert_eq!(result.row_count(), 3);
        assert_eq!(result.values[0], DataValue::String("A".to_string()));
        assert_eq!(result.values[1], DataValue::String("B".to_string()));
        assert_eq!(result.values[2], DataValue::String("C".to_string()));
    }

    #[test]
    fn test_unnest_null() {
        let expander = unnest::UnnestExpander;
        let value = DataValue::Null;
        let delimiter = DataValue::String("|".to_string());

        let result = expander.expand(&value, &[delimiter]).unwrap();
        assert_eq!(result.row_count(), 1);
        assert_eq!(result.values[0], DataValue::Null);
    }

    #[test]
    fn test_registry() {
        let registry = RowExpanderRegistry::new();
        assert!(registry.contains("UNNEST"));
        assert!(registry.contains("unnest"));

        let expander = registry.get("UNNEST").unwrap();
        assert_eq!(expander.name(), "UNNEST");
    }
}
