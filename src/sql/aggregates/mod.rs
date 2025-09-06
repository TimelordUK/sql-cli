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
    CollectList(Vec<DataValue>),
    Analytics(analytics::AnalyticsState),
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
            AvgFunction, CountFunction, CountStarFunction, MaxFunction, MinFunction, SumFunction,
        };

        let functions: Vec<Box<dyn AggregateFunction>> = vec![
            Box::new(CountFunction),
            Box::new(CountStarFunction),
            Box::new(SumFunction),
            Box::new(AvgFunction),
            Box::new(MinFunction),
            Box::new(MaxFunction),
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
        SqlExpression::FunctionCall { name, args } => {
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
