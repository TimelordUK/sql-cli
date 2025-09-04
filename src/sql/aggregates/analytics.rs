//! Analytics aggregate functions for time series and statistical operations

use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

use super::{AggregateFunction, AggregateState};
use crate::data::datatable::DataValue;

/// State for analytics aggregates
#[derive(Debug, Clone)]
pub struct AnalyticsState {
    pub function_type: AnalyticsType,
    pub values: Vec<f64>,
    pub window_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum AnalyticsType {
    Deltas,
    Sums,
    Mavg,
    PctChange,
    Rank,
    CumMax,
    CumMin,
}

impl AnalyticsState {
    pub fn new(function_type: AnalyticsType, window_size: Option<usize>) -> Self {
        Self {
            function_type,
            values: Vec::new(),
            window_size,
        }
    }

    pub fn add(&mut self, value: &DataValue) -> Result<()> {
        let num = match value {
            DataValue::Null => return Ok(()), // Skip nulls
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            _ => return Err(anyhow!("Analytics functions require numeric values")),
        };
        self.values.push(num);
        Ok(())
    }

    pub fn finalize(self) -> DataValue {
        if self.values.is_empty() {
            return DataValue::Null;
        }

        let result = match self.function_type {
            AnalyticsType::Deltas => compute_deltas(&self.values),
            AnalyticsType::Sums => compute_sums(&self.values),
            AnalyticsType::Mavg => compute_mavg(&self.values, self.window_size.unwrap_or(3)),
            AnalyticsType::PctChange => compute_pct_change(&self.values),
            AnalyticsType::Rank => compute_rank(&self.values),
            AnalyticsType::CumMax => compute_cummax(&self.values),
            AnalyticsType::CumMin => compute_cummin(&self.values),
        };

        DataValue::String(format!("[{}]", result.join(", ")))
    }
}

fn compute_deltas(values: &[f64]) -> Vec<String> {
    if values.is_empty() {
        return vec![];
    }

    let mut deltas = vec![format_number(values[0])];
    for i in 1..values.len() {
        let delta = values[i] - values[i - 1];
        deltas.push(format_number(delta));
    }
    deltas
}

fn compute_sums(values: &[f64]) -> Vec<String> {
    let mut sums = Vec::new();
    let mut running_sum = 0.0;

    for val in values {
        running_sum += val;
        sums.push(format_number(running_sum));
    }
    sums
}

fn compute_mavg(values: &[f64], window: usize) -> Vec<String> {
    let mut results = Vec::new();

    for i in 0..values.len() {
        let start = if i >= window { i - window + 1 } else { 0 };
        let end = i + 1;
        let window_values = &values[start..end];
        let avg = window_values.iter().sum::<f64>() / window_values.len() as f64;
        results.push(format_number(avg));
    }
    results
}

fn compute_pct_change(values: &[f64]) -> Vec<String> {
    if values.is_empty() {
        return vec![];
    }

    let mut changes = vec!["null".to_string()]; // First value has no previous
    for i in 1..values.len() {
        if values[i - 1] == 0.0 {
            changes.push("null".to_string());
        } else {
            let pct = ((values[i] - values[i - 1]) / values[i - 1]) * 100.0;
            changes.push(format!("{:.2}%", pct));
        }
    }
    changes
}

fn compute_rank(values: &[f64]) -> Vec<String> {
    // Create sorted unique values with their ranks
    let mut sorted_unique: Vec<f64> = values.to_vec();
    sorted_unique.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sorted_unique.dedup();

    let mut rank_map = BTreeMap::new();
    for (i, val) in sorted_unique.iter().enumerate() {
        rank_map.insert(val.to_bits(), i + 1); // Use to_bits for exact matching
    }

    // Map each value to its rank
    values
        .iter()
        .map(|v| rank_map[&v.to_bits()].to_string())
        .collect()
}

fn compute_cummax(values: &[f64]) -> Vec<String> {
    let mut results = Vec::new();
    let mut current_max = f64::NEG_INFINITY;

    for val in values {
        current_max = current_max.max(*val);
        results.push(format_number(current_max));
    }
    results
}

fn compute_cummin(values: &[f64]) -> Vec<String> {
    let mut results = Vec::new();
    let mut current_min = f64::INFINITY;

    for val in values {
        current_min = current_min.min(*val);
        results.push(format_number(current_min));
    }
    results
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e10 {
        format!("{}", n as i64)
    } else {
        // Format with 2 decimal places, removing trailing zeros
        let formatted = format!("{:.2}", n);
        if formatted.contains('.') {
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            formatted
        }
    }
}

// Aggregate function implementations

pub struct DeltasFunction;
impl AggregateFunction for DeltasFunction {
    fn name(&self) -> &str {
        "DELTAS"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::Deltas, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for DELTAS"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct SumsFunction;
impl AggregateFunction for SumsFunction {
    fn name(&self) -> &str {
        "SUMS"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::Sums, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for SUMS"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct MavgFunction;
impl AggregateFunction for MavgFunction {
    fn name(&self) -> &str {
        "MAVG"
    }

    fn init(&self) -> AggregateState {
        // Default window size is 3
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::Mavg, Some(3)))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for MAVG"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct PctChangeFunction;
impl AggregateFunction for PctChangeFunction {
    fn name(&self) -> &str {
        "PCT_CHANGE"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::PctChange, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for PCT_CHANGE"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct RankFunction;
impl AggregateFunction for RankFunction {
    fn name(&self) -> &str {
        "RANK"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::Rank, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for RANK"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct CumMaxFunction;
impl AggregateFunction for CumMaxFunction {
    fn name(&self) -> &str {
        "CUMMAX"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::CumMax, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for CUMMAX"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

pub struct CumMinFunction;
impl AggregateFunction for CumMinFunction {
    fn name(&self) -> &str {
        "CUMMIN"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Analytics(AnalyticsState::new(AnalyticsType::CumMin, None))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Analytics(ref mut analytics) = state {
            analytics.add(value)
        } else {
            Err(anyhow!("Invalid state for CUMMIN"))
        }
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Analytics(analytics) = state {
            analytics.finalize()
        } else {
            DataValue::Null
        }
    }
}

