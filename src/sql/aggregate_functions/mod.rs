// Aggregate Function Registry
// Provides a clean API for group-based aggregate computations
// Moves all aggregate logic out of the evaluator into a registry pattern

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::datatable::DataValue;

/// State maintained during aggregation
/// Each aggregate function manages its own state type
pub trait AggregateState: Send + Sync {
    /// Add a value to the aggregate
    fn accumulate(&mut self, value: &DataValue) -> Result<()>;

    /// Finalize and return the aggregate result
    fn finalize(self: Box<Self>) -> DataValue;

    /// Create a new instance of this state
    fn clone_box(&self) -> Box<dyn AggregateState>;

    /// Reset the state for reuse
    fn reset(&mut self);
}

/// Aggregate function trait
/// Each aggregate function (SUM, COUNT, AVG, etc.) implements this
pub trait AggregateFunction: Send + Sync {
    /// Function name (e.g., "SUM", "COUNT", "STRING_AGG")
    fn name(&self) -> &str;

    /// Description for help system
    fn description(&self) -> &str;

    /// Create initial state for this aggregate
    fn create_state(&self) -> Box<dyn AggregateState>;

    /// Does this aggregate support DISTINCT?
    fn supports_distinct(&self) -> bool {
        true // Most aggregates should support DISTINCT
    }

    /// For aggregates with parameters (like STRING_AGG separator)
    fn set_parameters(&self, _params: &[DataValue]) -> Result<Box<dyn AggregateFunction>> {
        // Default implementation for aggregates without parameters
        Ok(Box::new(DummyClone(self.name().to_string())))
    }
}

// Dummy clone helper for default implementation
struct DummyClone(String);
impl AggregateFunction for DummyClone {
    fn name(&self) -> &str {
        &self.0
    }
    fn description(&self) -> &str {
        ""
    }
    fn create_state(&self) -> Box<dyn AggregateState> {
        panic!("DummyClone should not be used")
    }
}

/// Registry for aggregate functions
pub struct AggregateFunctionRegistry {
    functions: HashMap<String, Arc<Box<dyn AggregateFunction>>>,
}

impl AggregateFunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_builtin_functions();
        registry
    }

    /// Register an aggregate function
    pub fn register(&mut self, function: Box<dyn AggregateFunction>) {
        let name = function.name().to_uppercase();
        self.functions.insert(name, Arc::new(function));
    }

    /// Get an aggregate function by name
    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn AggregateFunction>>> {
        self.functions.get(&name.to_uppercase()).cloned()
    }

    /// Check if a function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    /// List all registered functions
    pub fn list_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Register built-in aggregate functions
    fn register_builtin_functions(&mut self) {
        // Basic aggregates
        self.register(Box::new(CountFunction));
        self.register(Box::new(SumFunction));
        self.register(Box::new(AvgFunction));
        self.register(Box::new(MinFunction));
        self.register(Box::new(MaxFunction));

        // String aggregates
        self.register(Box::new(StringAggFunction::new()));

        // Statistical aggregates (to be implemented)
        // self.register(Box::new(StdDevFunction));
        // self.register(Box::new(VarianceFunction));
    }
}

// ============= COUNT Implementation =============

struct CountFunction;

impl AggregateFunction for CountFunction {
    fn name(&self) -> &str {
        "COUNT"
    }

    fn description(&self) -> &str {
        "Count the number of non-null values or rows"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CountState { count: 0 })
    }
}

struct CountState {
    count: i64,
}

impl AggregateState for CountState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        // COUNT(*) counts all rows, COUNT(column) counts non-nulls
        if !matches!(value, DataValue::Null) {
            self.count += 1;
        }
        Ok(())
    }

    fn finalize(self: Box<Self>) -> DataValue {
        DataValue::Integer(self.count)
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(CountState { count: self.count })
    }

    fn reset(&mut self) {
        self.count = 0;
    }
}

// ============= SUM Implementation =============

struct SumFunction;

impl AggregateFunction for SumFunction {
    fn name(&self) -> &str {
        "SUM"
    }

    fn description(&self) -> &str {
        "Calculate the sum of values"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(SumState {
            int_sum: None,
            float_sum: None,
            has_values: false,
        })
    }
}

struct SumState {
    int_sum: Option<i64>,
    float_sum: Option<f64>,
    has_values: bool,
}

impl AggregateState for SumState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        match value {
            DataValue::Null => Ok(()), // Skip nulls
            DataValue::Integer(n) => {
                self.has_values = true;
                if let Some(ref mut sum) = self.int_sum {
                    *sum = sum.saturating_add(*n);
                } else if let Some(ref mut fsum) = self.float_sum {
                    *fsum += *n as f64;
                } else {
                    self.int_sum = Some(*n);
                }
                Ok(())
            }
            DataValue::Float(f) => {
                self.has_values = true;
                // Once we have a float, convert everything to float
                if let Some(isum) = self.int_sum.take() {
                    self.float_sum = Some(isum as f64 + f);
                } else if let Some(ref mut fsum) = self.float_sum {
                    *fsum += f;
                } else {
                    self.float_sum = Some(*f);
                }
                Ok(())
            }
            _ => Err(anyhow!("Cannot sum non-numeric value")),
        }
    }

    fn finalize(self: Box<Self>) -> DataValue {
        if !self.has_values {
            return DataValue::Null;
        }

        if let Some(fsum) = self.float_sum {
            DataValue::Float(fsum)
        } else if let Some(isum) = self.int_sum {
            DataValue::Integer(isum)
        } else {
            DataValue::Null
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(SumState {
            int_sum: self.int_sum,
            float_sum: self.float_sum,
            has_values: self.has_values,
        })
    }

    fn reset(&mut self) {
        self.int_sum = None;
        self.float_sum = None;
        self.has_values = false;
    }
}

// ============= AVG Implementation =============

struct AvgFunction;

impl AggregateFunction for AvgFunction {
    fn name(&self) -> &str {
        "AVG"
    }

    fn description(&self) -> &str {
        "Calculate the average of values"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(AvgState {
            sum: SumState {
                int_sum: None,
                float_sum: None,
                has_values: false,
            },
            count: 0,
        })
    }
}

struct AvgState {
    sum: SumState,
    count: i64,
}

impl AggregateState for AvgState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        if !matches!(value, DataValue::Null) {
            self.sum.accumulate(value)?;
            self.count += 1;
        }
        Ok(())
    }

    fn finalize(self: Box<Self>) -> DataValue {
        if self.count == 0 {
            return DataValue::Null;
        }

        let sum = Box::new(self.sum).finalize();
        match sum {
            DataValue::Integer(n) => DataValue::Float(n as f64 / self.count as f64),
            DataValue::Float(f) => DataValue::Float(f / self.count as f64),
            _ => DataValue::Null,
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(AvgState {
            sum: SumState {
                int_sum: self.sum.int_sum,
                float_sum: self.sum.float_sum,
                has_values: self.sum.has_values,
            },
            count: self.count,
        })
    }

    fn reset(&mut self) {
        self.sum.reset();
        self.count = 0;
    }
}

// ============= MIN Implementation =============

struct MinFunction;

impl AggregateFunction for MinFunction {
    fn name(&self) -> &str {
        "MIN"
    }

    fn description(&self) -> &str {
        "Find the minimum value"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(MinMaxState {
            is_min: true,
            current: None,
        })
    }
}

// ============= MAX Implementation =============

struct MaxFunction;

impl AggregateFunction for MaxFunction {
    fn name(&self) -> &str {
        "MAX"
    }

    fn description(&self) -> &str {
        "Find the maximum value"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(MinMaxState {
            is_min: false,
            current: None,
        })
    }
}

struct MinMaxState {
    is_min: bool,
    current: Option<DataValue>,
}

impl AggregateState for MinMaxState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        if matches!(value, DataValue::Null) {
            return Ok(());
        }

        match &self.current {
            None => {
                self.current = Some(value.clone());
            }
            Some(current) => {
                let should_update = if self.is_min {
                    value < current
                } else {
                    value > current
                };

                if should_update {
                    self.current = Some(value.clone());
                }
            }
        }

        Ok(())
    }

    fn finalize(self: Box<Self>) -> DataValue {
        self.current.unwrap_or(DataValue::Null)
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(MinMaxState {
            is_min: self.is_min,
            current: self.current.clone(),
        })
    }

    fn reset(&mut self) {
        self.current = None;
    }
}

// ============= STRING_AGG Implementation =============

struct StringAggFunction {
    separator: String,
}

impl StringAggFunction {
    fn new() -> Self {
        Self {
            separator: ",".to_string(), // Default separator
        }
    }

    fn with_separator(separator: String) -> Self {
        Self { separator }
    }
}

impl AggregateFunction for StringAggFunction {
    fn name(&self) -> &str {
        "STRING_AGG"
    }

    fn description(&self) -> &str {
        "Concatenate strings with a separator"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(StringAggState {
            values: Vec::new(),
            separator: self.separator.clone(),
        })
    }

    fn set_parameters(&self, params: &[DataValue]) -> Result<Box<dyn AggregateFunction>> {
        // STRING_AGG takes a separator as second parameter
        if params.is_empty() {
            return Ok(Box::new(StringAggFunction::new()));
        }

        let separator = match &params[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            _ => return Err(anyhow!("STRING_AGG separator must be a string")),
        };

        Ok(Box::new(StringAggFunction::with_separator(separator)))
    }
}

struct StringAggState {
    values: Vec<String>,
    separator: String,
}

impl AggregateState for StringAggState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        match value {
            DataValue::Null => Ok(()), // Skip nulls
            DataValue::String(s) => {
                self.values.push(s.clone());
                Ok(())
            }
            DataValue::InternedString(s) => {
                self.values.push(s.to_string());
                Ok(())
            }
            DataValue::Integer(n) => {
                self.values.push(n.to_string());
                Ok(())
            }
            DataValue::Float(f) => {
                self.values.push(f.to_string());
                Ok(())
            }
            DataValue::Boolean(b) => {
                self.values.push(b.to_string());
                Ok(())
            }
            DataValue::DateTime(dt) => {
                self.values.push(dt.to_string());
                Ok(())
            }
        }
    }

    fn finalize(self: Box<Self>) -> DataValue {
        if self.values.is_empty() {
            DataValue::Null
        } else {
            DataValue::String(self.values.join(&self.separator))
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(StringAggState {
            values: self.values.clone(),
            separator: self.separator.clone(),
        })
    }

    fn reset(&mut self) {
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AggregateFunctionRegistry::new();
        assert!(registry.contains("COUNT"));
        assert!(registry.contains("SUM"));
        assert!(registry.contains("AVG"));
        assert!(registry.contains("MIN"));
        assert!(registry.contains("MAX"));
        assert!(registry.contains("STRING_AGG"));
    }

    #[test]
    fn test_count_aggregate() {
        let func = CountFunction;
        let mut state = func.create_state();

        state.accumulate(&DataValue::Integer(1)).unwrap();
        state.accumulate(&DataValue::Null).unwrap();
        state.accumulate(&DataValue::Integer(3)).unwrap();

        let result = state.finalize();
        assert_eq!(result, DataValue::Integer(2));
    }

    #[test]
    fn test_string_agg() {
        let func = StringAggFunction::with_separator(", ".to_string());
        let mut state = func.create_state();

        state
            .accumulate(&DataValue::String("apple".to_string()))
            .unwrap();
        state
            .accumulate(&DataValue::String("banana".to_string()))
            .unwrap();
        state
            .accumulate(&DataValue::String("cherry".to_string()))
            .unwrap();

        let result = state.finalize();
        assert_eq!(
            result,
            DataValue::String("apple, banana, cherry".to_string())
        );
    }
}
