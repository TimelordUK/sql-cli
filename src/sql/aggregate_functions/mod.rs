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
        self.register(Box::new(CountStarFunction));
        self.register(Box::new(SumFunction));
        self.register(Box::new(AvgFunction));
        self.register(Box::new(MinFunction));
        self.register(Box::new(MaxFunction));

        // String aggregates
        self.register(Box::new(StringAggFunction::new()));

        // Statistical aggregates
        self.register(Box::new(MedianFunction));
        self.register(Box::new(ModeFunction));
        self.register(Box::new(StdDevFunction));
        self.register(Box::new(StdDevPFunction));
        self.register(Box::new(VarianceFunction));
        self.register(Box::new(VariancePFunction));
        self.register(Box::new(PercentileFunction));
    }
}

// ============= COUNT Implementation =============

struct CountFunction;

impl AggregateFunction for CountFunction {
    fn name(&self) -> &str {
        "COUNT"
    }

    fn description(&self) -> &str {
        "Count the number of non-null values"
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
        // COUNT(column) counts non-nulls only
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

// COUNT(*) - counts all rows including nulls
struct CountStarFunction;

impl AggregateFunction for CountStarFunction {
    fn name(&self) -> &str {
        "COUNT_STAR"
    }

    fn description(&self) -> &str {
        "Count all rows including nulls"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CountStarState { count: 0 })
    }
}

struct CountStarState {
    count: i64,
}

impl AggregateState for CountStarState {
    fn accumulate(&mut self, _value: &DataValue) -> Result<()> {
        // COUNT(*) counts all rows, even nulls
        self.count += 1;
        Ok(())
    }

    fn finalize(self: Box<Self>) -> DataValue {
        DataValue::Integer(self.count)
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(CountStarState { count: self.count })
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
            DataValue::Boolean(b) => {
                // Coerce boolean to integer: true=1, false=0
                // Enables patterns like AVG(x > 5), SUM(col = 'value')
                let n = if *b { 1i64 } else { 0i64 };
                self.has_values = true;
                if let Some(ref mut sum) = self.int_sum {
                    *sum = sum.saturating_add(n);
                } else if let Some(ref mut fsum) = self.float_sum {
                    *fsum += n as f64;
                } else {
                    self.int_sum = Some(n);
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
            DataValue::Vector(v) => {
                let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                self.values.push(format!("[{}]", components.join(",")));
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

// ============= MEDIAN Implementation =============

struct MedianFunction;

impl AggregateFunction for MedianFunction {
    fn name(&self) -> &str {
        "MEDIAN"
    }

    fn description(&self) -> &str {
        "Calculate the median (middle value) of numeric values"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::Median,
        })
    }
}

// ============= MODE Implementation =============

struct ModeFunction;

impl AggregateFunction for ModeFunction {
    fn name(&self) -> &str {
        "MODE"
    }

    fn description(&self) -> &str {
        "Find the most frequently occurring value"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::Mode,
        })
    }
}

// ============= STDDEV Implementation =============

struct StdDevFunction;

impl AggregateFunction for StdDevFunction {
    fn name(&self) -> &str {
        "STDDEV"
    }

    fn description(&self) -> &str {
        "Calculate the sample standard deviation"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::StdDev,
        })
    }
}

// ============= STDDEV_POP Implementation =============

struct StdDevPFunction;

impl AggregateFunction for StdDevPFunction {
    fn name(&self) -> &str {
        "STDDEV_POP"
    }

    fn description(&self) -> &str {
        "Calculate the population standard deviation"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::StdDevP,
        })
    }
}

// ============= VARIANCE Implementation =============

struct VarianceFunction;

impl AggregateFunction for VarianceFunction {
    fn name(&self) -> &str {
        "VARIANCE"
    }

    fn description(&self) -> &str {
        "Calculate the sample variance"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::Variance,
        })
    }
}

// ============= VARIANCE_POP Implementation =============

struct VariancePFunction;

impl AggregateFunction for VariancePFunction {
    fn name(&self) -> &str {
        "VARIANCE_POP"
    }

    fn description(&self) -> &str {
        "Calculate the population variance"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: Vec::new(),
            function_type: CollectorFunction::VarianceP,
        })
    }
}

// ============= PERCENTILE Implementation =============

struct PercentileFunction;

impl AggregateFunction for PercentileFunction {
    fn name(&self) -> &str {
        "PERCENTILE"
    }

    fn description(&self) -> &str {
        "Calculate the nth percentile of values"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(PercentileState {
            values: Vec::new(),
            percentile: 50.0, // Default to median
        })
    }

    fn set_parameters(&self, params: &[DataValue]) -> Result<Box<dyn AggregateFunction>> {
        // PERCENTILE takes the percentile value as a parameter
        if params.is_empty() {
            return Ok(Box::new(PercentileFunction));
        }

        let percentile = match &params[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            _ => {
                return Err(anyhow!(
                    "PERCENTILE parameter must be a number between 0 and 100"
                ))
            }
        };

        if percentile < 0.0 || percentile > 100.0 {
            return Err(anyhow!("PERCENTILE must be between 0 and 100"));
        }

        Ok(Box::new(PercentileWithParam { percentile }))
    }
}

struct PercentileWithParam {
    percentile: f64,
}

impl AggregateFunction for PercentileWithParam {
    fn name(&self) -> &str {
        "PERCENTILE"
    }

    fn description(&self) -> &str {
        "Calculate the nth percentile of values"
    }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(PercentileState {
            values: Vec::new(),
            percentile: self.percentile,
        })
    }
}

// ============= Collector State for functions that need all values =============

enum CollectorFunction {
    Median,
    Mode,
    StdDev,    // Sample standard deviation
    StdDevP,   // Population standard deviation
    Variance,  // Sample variance
    VarianceP, // Population variance
}

struct CollectorState {
    values: Vec<f64>,
    function_type: CollectorFunction,
}

impl AggregateState for CollectorState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        match value {
            DataValue::Null => Ok(()), // Skip nulls
            DataValue::Integer(n) => {
                self.values.push(*n as f64);
                Ok(())
            }
            DataValue::Float(f) => {
                self.values.push(*f);
                Ok(())
            }
            _ => match self.function_type {
                CollectorFunction::Mode => {
                    // Mode can work with non-numeric types, but we'll handle that separately
                    Err(anyhow!("MODE currently only supports numeric values"))
                }
                _ => Err(anyhow!("Statistical functions require numeric values")),
            },
        }
    }

    fn finalize(self: Box<Self>) -> DataValue {
        if self.values.is_empty() {
            return DataValue::Null;
        }

        match self.function_type {
            CollectorFunction::Median => {
                let mut sorted = self.values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let len = sorted.len();
                if len % 2 == 0 {
                    DataValue::Float((sorted[len / 2 - 1] + sorted[len / 2]) / 2.0)
                } else {
                    DataValue::Float(sorted[len / 2])
                }
            }
            CollectorFunction::Mode => {
                // Tally each distinct value, remembering the position it was
                // first seen at. `values` is in input order, so that position
                // is the tie-break: highest count wins, and on a tie the value
                // seen earliest wins.
                //
                // Without the tie-break the winner came from `HashMap`
                // iteration order, so the same binary on the same data returned
                // different answers run to run (P41). Earliest-seen is the
                // reference engine's rule.
                use std::collections::HashMap;
                let mut counts: HashMap<u64, (usize, usize)> = HashMap::new();
                for (position, value) in self.values.iter().enumerate() {
                    let entry = counts.entry(value.to_bits()).or_insert((0, position));
                    entry.0 += 1;
                }
                counts
                    .into_iter()
                    .max_by(|(_, a), (_, b)| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
                    .map_or(DataValue::Null, |(bits, _)| {
                        DataValue::Float(f64::from_bits(bits))
                    })
            }
            CollectorFunction::StdDev | CollectorFunction::Variance => {
                // Sample standard deviation and variance
                if self.values.len() < 2 {
                    return DataValue::Null;
                }
                let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
                let variance = self.values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                    / (self.values.len() - 1) as f64; // N-1 for sample

                match self.function_type {
                    CollectorFunction::StdDev => DataValue::Float(variance.sqrt()),
                    CollectorFunction::Variance => DataValue::Float(variance),
                    _ => unreachable!(),
                }
            }
            CollectorFunction::StdDevP | CollectorFunction::VarianceP => {
                // Population standard deviation and variance
                let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
                let variance = self.values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                    / self.values.len() as f64; // N for population

                match self.function_type {
                    CollectorFunction::StdDevP => DataValue::Float(variance.sqrt()),
                    CollectorFunction::VarianceP => DataValue::Float(variance),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(CollectorState {
            values: self.values.clone(),
            function_type: match self.function_type {
                CollectorFunction::Median => CollectorFunction::Median,
                CollectorFunction::Mode => CollectorFunction::Mode,
                CollectorFunction::StdDev => CollectorFunction::StdDev,
                CollectorFunction::StdDevP => CollectorFunction::StdDevP,
                CollectorFunction::Variance => CollectorFunction::Variance,
                CollectorFunction::VarianceP => CollectorFunction::VarianceP,
            },
        })
    }

    fn reset(&mut self) {
        self.values.clear();
    }
}

// ============= Percentile State =============

struct PercentileState {
    values: Vec<f64>,
    percentile: f64,
}

impl AggregateState for PercentileState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        match value {
            DataValue::Null => Ok(()), // Skip nulls
            DataValue::Integer(n) => {
                self.values.push(*n as f64);
                Ok(())
            }
            DataValue::Float(f) => {
                self.values.push(*f);
                Ok(())
            }
            _ => Err(anyhow!("PERCENTILE requires numeric values")),
        }
    }

    fn finalize(self: Box<Self>) -> DataValue {
        if self.values.is_empty() {
            return DataValue::Null;
        }

        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate the position in the sorted array
        let position = (self.percentile / 100.0) * (sorted.len() - 1) as f64;
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;

        if lower == upper {
            DataValue::Float(sorted[lower])
        } else {
            // Linear interpolation between two values
            let weight = position - lower as f64;
            DataValue::Float(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateState> {
        Box::new(PercentileState {
            values: self.values.clone(),
            percentile: self.percentile,
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

#[cfg(test)]
mod mode_tie_break_tests {
    use super::{AggregateFunctionRegistry, DataValue};

    /// Drives MODE through the registry, which is what the evaluator does — the
    /// point of these tests is the path that actually runs, not a state type
    /// that happens to be named after the function. See P41 in SQL_PARITY.md.
    fn mode(values: &[DataValue]) -> DataValue {
        let registry = AggregateFunctionRegistry::new();
        let func = registry.get("MODE").expect("MODE should be registered");
        let mut state = func.create_state();
        for v in values {
            state
                .accumulate(v)
                .expect("MODE accumulate should not fail");
        }
        state.finalize()
    }

    fn ints(values: &[i64]) -> Vec<DataValue> {
        values.iter().map(|i| DataValue::Integer(*i)).collect()
    }

    #[test]
    fn outright_winner_is_the_most_frequent_value() {
        assert_eq!(mode(&ints(&[7, 3, 7, 3, 7])), DataValue::Float(7.0));
    }

    #[test]
    fn a_tie_resolves_to_the_value_seen_first() {
        assert_eq!(mode(&ints(&[0, 0, 1, 1])), DataValue::Float(0.0));
        assert_eq!(mode(&ints(&[1, 1, 0, 0])), DataValue::Float(1.0));
    }

    #[test]
    fn the_tie_break_is_first_seen_not_smallest() {
        // The discriminating case: every value ties at one occurrence and the
        // first one seen is not the smallest. "Smallest wins" would pass the
        // test above and still diverge from the reference engine here.
        assert_eq!(mode(&ints(&[5, 3, 9, 1])), DataValue::Float(5.0));
    }

    #[test]
    fn nulls_are_ignored_and_do_not_shift_the_tie_break() {
        let values = vec![
            DataValue::Null,
            DataValue::Integer(9),
            DataValue::Null,
            DataValue::Null,
            DataValue::Integer(2),
            DataValue::Null,
        ];
        assert_eq!(mode(&values), DataValue::Float(9.0));
    }

    #[test]
    fn all_null_and_empty_inputs_are_null() {
        assert_eq!(mode(&[]), DataValue::Null);
        assert_eq!(mode(&[DataValue::Null, DataValue::Null]), DataValue::Null);
    }

    #[test]
    fn repeated_runs_over_a_tie_give_the_same_answer() {
        // The P41 symptom itself: the old implementation took whichever entry
        // `HashMap` iteration surfaced last, so a perfect tie returned a
        // different value between runs of the same binary. Enough distinct keys
        // that hash ordering actually varies.
        let values: Vec<DataValue> = (0..64).map(|i| DataValue::Integer(i % 32)).collect();
        let first = mode(&values);
        assert_eq!(first, DataValue::Float(0.0));
        for _ in 0..50 {
            assert_eq!(mode(&values), first);
        }
    }
}
