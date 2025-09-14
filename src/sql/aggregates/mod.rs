//! Aggregate functions for GROUP BY operations
//!
//! This module provides SQL aggregate functions like SUM, AVG, COUNT, MIN, MAX
//! that work with the `DataView` partitioning system for efficient GROUP BY queries.

use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;

pub mod analytics;
pub mod functions;

/// State maintained during aggregation
#[derive(Debug, Clone)]
pub enum AggregateState {
    Count(i64),
    Sum(SumState),
    Avg(AvgState),
    MinMax(MinMaxState),
    Variance(VarianceState),
    CollectList(Vec<DataValue>),
    Percentile(PercentileState),
    Mode(ModeState),
    Analytics(analytics::AnalyticsState),
    StringAgg(StringAggState),
}

/// State for SUM aggregation
#[derive(Debug, Clone)]
pub struct SumState {
    pub int_sum: Option<i64>,
    pub float_sum: Option<f64>,
    pub has_values: bool,
}

impl Default for SumState {
    fn default() -> Self {
        Self::new()
    }
}

impl SumState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            int_sum: None,
            float_sum: None,
            has_values: false,
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
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

    #[must_use]
    pub fn finalize(self) -> DataValue {
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
}

/// State for AVG aggregation
#[derive(Debug, Clone)]
pub struct AvgState {
    pub sum: SumState,
    pub count: i64,
}

impl Default for AvgState {
    fn default() -> Self {
        Self::new()
    }
}

impl AvgState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sum: SumState::new(),
            count: 0,
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        if !matches!(value, DataValue::Null) {
            self.sum.add(value)?;
            self.count += 1;
        }
        Ok(())
    }

    #[must_use]
    pub fn finalize(self) -> DataValue {
        if self.count == 0 {
            return DataValue::Null;
        }

        let sum = self.sum.finalize();
        match sum {
            DataValue::Integer(n) => DataValue::Float(n as f64 / self.count as f64),
            DataValue::Float(f) => DataValue::Float(f / self.count as f64),
            _ => DataValue::Null,
        }
    }
}

/// State for MIN/MAX aggregation
#[derive(Debug, Clone)]
pub struct MinMaxState {
    pub is_min: bool,
    pub current: Option<DataValue>,
}

impl MinMaxState {
    #[must_use]
    pub fn new(is_min: bool) -> Self {
        Self {
            is_min,
            current: None,
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        if matches!(value, DataValue::Null) {
            return Ok(());
        }

        if let Some(ref current) = self.current {
            let should_update = if self.is_min {
                value < current
            } else {
                value > current
            };

            if should_update {
                self.current = Some(value.clone());
            }
        } else {
            self.current = Some(value.clone());
        }

        Ok(())
    }

    #[must_use]
    pub fn finalize(self) -> DataValue {
        self.current.unwrap_or(DataValue::Null)
    }
}

/// State for VARIANCE/STDDEV aggregation
#[derive(Debug, Clone)]
pub struct VarianceState {
    pub sum: f64,
    pub sum_of_squares: f64,
    pub count: i64,
}

impl Default for VarianceState {
    fn default() -> Self {
        Self::new()
    }
}

impl VarianceState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            sum_of_squares: 0.0,
            count: 0,
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        match value {
            DataValue::Null => Ok(()), // Skip nulls
            DataValue::Integer(n) => {
                let f = *n as f64;
                self.sum += f;
                self.sum_of_squares += f * f;
                self.count += 1;
                Ok(())
            }
            DataValue::Float(f) => {
                self.sum += f;
                self.sum_of_squares += f * f;
                self.count += 1;
                Ok(())
            }
            _ => Err(anyhow!("Cannot compute variance of non-numeric value")),
        }
    }

    #[must_use]
    pub fn variance(&self) -> f64 {
        if self.count <= 1 {
            return 0.0;
        }
        let mean = self.sum / self.count as f64;
        (self.sum_of_squares / self.count as f64) - (mean * mean)
    }

    #[must_use]
    pub fn stddev(&self) -> f64 {
        self.variance().sqrt()
    }

    #[must_use]
    pub fn finalize_variance(self) -> DataValue {
        if self.count == 0 {
            DataValue::Null
        } else {
            DataValue::Float(self.variance())
        }
    }

    #[must_use]
    pub fn finalize_stddev(self) -> DataValue {
        if self.count == 0 {
            DataValue::Null
        } else {
            DataValue::Float(self.stddev())
        }
    }
}

/// State for PERCENTILE aggregation
#[derive(Debug, Clone)]
pub struct PercentileState {
    pub values: Vec<DataValue>,
    pub percentile: f64,
}

impl Default for PercentileState {
    fn default() -> Self {
        Self::new(50.0) // Default to median
    }
}

impl PercentileState {
    #[must_use]
    pub fn new(percentile: f64) -> Self {
        Self {
            values: Vec::new(),
            percentile: percentile.clamp(0.0, 100.0),
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        if !matches!(value, DataValue::Null) {
            self.values.push(value.clone());
        }
        Ok(())
    }

    #[must_use]
    pub fn finalize(mut self) -> DataValue {
        if self.values.is_empty() {
            return DataValue::Null;
        }

        // Sort values for percentile calculation
        self.values.sort_by(|a, b| {
            use std::cmp::Ordering;
            match (a, b) {
                (DataValue::Integer(a), DataValue::Integer(b)) => a.cmp(b),
                (DataValue::Float(a), DataValue::Float(b)) => {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                }
                (DataValue::Integer(a), DataValue::Float(b)) => {
                    (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
                }
                (DataValue::Float(a), DataValue::Integer(b)) => {
                    a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
                }
                _ => Ordering::Equal,
            }
        });

        let n = self.values.len();
        if self.percentile == 0.0 {
            return self.values[0].clone();
        }
        if self.percentile == 100.0 {
            return self.values[n - 1].clone();
        }

        // Calculate percentile using linear interpolation
        let pos = (self.percentile / 100.0) * ((n - 1) as f64);
        let lower_idx = pos.floor() as usize;
        let upper_idx = pos.ceil() as usize;

        if lower_idx == upper_idx {
            // Exact position
            self.values[lower_idx].clone()
        } else {
            // Interpolate between two values
            let fraction = pos - lower_idx as f64;
            let lower_val = &self.values[lower_idx];
            let upper_val = &self.values[upper_idx];

            match (lower_val, upper_val) {
                (DataValue::Integer(a), DataValue::Integer(b)) => {
                    let result = *a as f64 + fraction * (*b - *a) as f64;
                    if result.fract() == 0.0 {
                        DataValue::Integer(result as i64)
                    } else {
                        DataValue::Float(result)
                    }
                }
                (DataValue::Float(a), DataValue::Float(b)) => {
                    DataValue::Float(a + fraction * (b - a))
                }
                (DataValue::Integer(a), DataValue::Float(b)) => {
                    DataValue::Float(*a as f64 + fraction * (b - *a as f64))
                }
                (DataValue::Float(a), DataValue::Integer(b)) => {
                    DataValue::Float(a + fraction * (*b as f64 - a))
                }
                // For non-numeric, return the lower value
                _ => lower_val.clone(),
            }
        }
    }
}

/// State for MODE aggregation (most frequent value)
#[derive(Debug, Clone)]
pub struct ModeState {
    pub counts: std::collections::HashMap<String, (DataValue, i64)>,
}

impl Default for ModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        if matches!(value, DataValue::Null) {
            return Ok(());
        }

        // Convert value to string for hashing, but keep original value for result
        let key = match value {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Null => return Ok(()),
        };

        // Update count and store the original value
        let entry = self.counts.entry(key).or_insert((value.clone(), 0));
        entry.1 += 1;

        Ok(())
    }

    #[must_use]
    pub fn finalize(self) -> DataValue {
        if self.counts.is_empty() {
            return DataValue::Null;
        }

        // Find the value with the highest count
        let max_entry = self.counts.iter().max_by_key(|(_, (_, count))| count);

        match max_entry {
            Some((_, (value, _count))) => value.clone(),
            None => DataValue::Null,
        }
    }
}

/// Trait for all aggregate functions
pub trait AggregateFunction: Send + Sync {
    /// Name of the function (e.g., "SUM", "AVG")
    fn name(&self) -> &str;

    /// Initialize the aggregation state
    fn init(&self) -> AggregateState;

    /// Add a value to the aggregation
    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()>;

    /// Finalize and return the result
    fn finalize(&self, state: AggregateState) -> DataValue;

    /// Check if this function requires numeric input
    fn requires_numeric(&self) -> bool {
        false
    }
}

/// State for STRING_AGG aggregation
#[derive(Debug, Clone)]
pub struct StringAggState {
    pub values: Vec<String>,
    pub separator: String,
}

impl Default for StringAggState {
    fn default() -> Self {
        Self::new(",")
    }
}

impl StringAggState {
    #[must_use]
    pub fn new(separator: &str) -> Self {
        Self {
            values: Vec::new(),
            separator: separator.to_string(),
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
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

    #[must_use]
    pub fn finalize(self) -> DataValue {
        if self.values.is_empty() {
            DataValue::Null
        } else {
            DataValue::String(self.values.join(&self.separator))
        }
    }
}

/// Registry of aggregate functions
pub struct AggregateRegistry {
    functions: Vec<Box<dyn AggregateFunction>>,
}

impl AggregateRegistry {
    #[must_use]
    pub fn new() -> Self {
        use analytics::{
            CumMaxFunction, CumMinFunction, DeltasFunction, MavgFunction, PctChangeFunction,
            RankFunction, SumsFunction,
        };
        use functions::{
            AvgFunction, CountFunction, CountStarFunction, MaxFunction, MedianFunction,
            MinFunction, ModeFunction, PercentileFunction, StdDevFunction, StringAggFunction,
            SumFunction, VarianceFunction,
        };

        let functions: Vec<Box<dyn AggregateFunction>> = vec![
            Box::new(CountFunction),
            Box::new(CountStarFunction),
            Box::new(SumFunction),
            Box::new(AvgFunction),
            Box::new(MinFunction),
            Box::new(MaxFunction),
            Box::new(StdDevFunction),
            Box::new(VarianceFunction),
            Box::new(MedianFunction),
            Box::new(ModeFunction),
            Box::new(PercentileFunction),
            Box::new(StringAggFunction),
            // Analytics functions
            Box::new(DeltasFunction),
            Box::new(SumsFunction),
            Box::new(MavgFunction),
            Box::new(PctChangeFunction),
            Box::new(RankFunction),
            Box::new(CumMaxFunction),
            Box::new(CumMinFunction),
        ];

        Self { functions }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn AggregateFunction> {
        let name_upper = name.to_uppercase();
        self.functions
            .iter()
            .find(|f| f.name() == name_upper)
            .map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn is_aggregate(&self, name: &str) -> bool {
        self.get(name).is_some() || name.to_uppercase() == "COUNT" // COUNT(*) special case
    }
}

impl Default for AggregateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if an expression contains aggregate functions
pub fn contains_aggregate(expr: &crate::recursive_parser::SqlExpression) -> bool {
    use crate::recursive_parser::SqlExpression;

    match expr {
        SqlExpression::FunctionCall { name, args, .. } => {
            let registry = AggregateRegistry::new();
            if registry.is_aggregate(name) {
                return true;
            }
            // Check nested expressions
            args.iter().any(contains_aggregate)
        }
        SqlExpression::BinaryOp { left, right, .. } => {
            contains_aggregate(left) || contains_aggregate(right)
        }
        SqlExpression::Not { expr } => contains_aggregate(expr),
        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            when_branches.iter().any(|branch| {
                contains_aggregate(&branch.condition) || contains_aggregate(&branch.result)
            }) || else_branch.as_ref().is_some_and(|e| contains_aggregate(e))
        }
        _ => false,
    }
}

/// Check if an expression is a constant (string literal, number literal, boolean, null)
/// Constants are compatible with aggregate queries and should produce a single row
pub fn is_constant_expression(expr: &crate::recursive_parser::SqlExpression) -> bool {
    use crate::recursive_parser::SqlExpression;

    match expr {
        SqlExpression::StringLiteral(_) => true,
        SqlExpression::NumberLiteral(_) => true,
        SqlExpression::BooleanLiteral(_) => true,
        SqlExpression::Null => true,
        SqlExpression::DateTimeConstructor { .. } => true,
        SqlExpression::DateTimeToday { .. } => true,
        // Binary operations between constants are also constant
        SqlExpression::BinaryOp { left, right, .. } => {
            is_constant_expression(left) && is_constant_expression(right)
        }
        // NOT of a constant is still constant
        SqlExpression::Not { expr } => is_constant_expression(expr),
        // Case expressions with constant conditions and results are constant
        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            when_branches.iter().all(|branch| {
                is_constant_expression(&branch.condition) && is_constant_expression(&branch.result)
            }) && else_branch
                .as_ref()
                .map_or(true, |e| is_constant_expression(e))
        }
        // Function calls that don't reference columns or aggregates are constant
        // (like CONVERT(100, 'km', 'miles') or mathematical constants)
        SqlExpression::FunctionCall { args, .. } => {
            // Only if all arguments are constants and it's not an aggregate
            !contains_aggregate(expr) && args.iter().all(is_constant_expression)
        }
        _ => false,
    }
}

/// Check if an expression is aggregate-compatible (either an aggregate or a constant)
/// This is used to determine if a SELECT list should produce a single row
pub fn is_aggregate_compatible(expr: &crate::recursive_parser::SqlExpression) -> bool {
    contains_aggregate(expr) || is_constant_expression(expr)
}
