//! Concrete implementations of aggregate functions

use anyhow::Result;

use super::{
    AggregateFunction, AggregateState, AvgState, MinMaxState, ModeState, PercentileState,
    StringAggState, SumState, VarianceState,
};
use crate::data::datatable::DataValue;

/// COUNT(*) - counts all rows including nulls
pub struct CountStarFunction;

impl AggregateFunction for CountStarFunction {
    fn name(&self) -> &'static str {
        "COUNT_STAR"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Count(0)
    }

    fn accumulate(&self, state: &mut AggregateState, _value: &DataValue) -> Result<()> {
        if let AggregateState::Count(ref mut count) = state {
            *count += 1;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Count(count) = state {
            DataValue::Integer(count)
        } else {
            DataValue::Null
        }
    }
}

/// COUNT(column) - counts non-null values
pub struct CountFunction;

impl AggregateFunction for CountFunction {
    fn name(&self) -> &'static str {
        "COUNT"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Count(0)
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Count(ref mut count) = state {
            if !matches!(value, DataValue::Null) {
                *count += 1;
            }
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Count(count) = state {
            DataValue::Integer(count)
        } else {
            DataValue::Null
        }
    }
}

/// SUM(column) - sums numeric values
pub struct SumFunction;

impl AggregateFunction for SumFunction {
    fn name(&self) -> &'static str {
        "SUM"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Sum(SumState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Sum(ref mut sum_state) = state {
            sum_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Sum(sum_state) = state {
            sum_state.finalize()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

/// AVG(column) - averages numeric values
pub struct AvgFunction;

impl AggregateFunction for AvgFunction {
    fn name(&self) -> &'static str {
        "AVG"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Avg(AvgState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Avg(ref mut avg_state) = state {
            avg_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Avg(avg_state) = state {
            avg_state.finalize()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

/// MIN(column) - finds minimum value
pub struct MinFunction;

impl AggregateFunction for MinFunction {
    fn name(&self) -> &'static str {
        "MIN"
    }

    fn init(&self) -> AggregateState {
        AggregateState::MinMax(MinMaxState::new(true))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::MinMax(ref mut minmax_state) = state {
            minmax_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::MinMax(minmax_state) = state {
            minmax_state.finalize()
        } else {
            DataValue::Null
        }
    }
}

/// MAX(column) - finds maximum value
pub struct MaxFunction;

impl AggregateFunction for MaxFunction {
    fn name(&self) -> &'static str {
        "MAX"
    }

    fn init(&self) -> AggregateState {
        AggregateState::MinMax(MinMaxState::new(false))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::MinMax(ref mut minmax_state) = state {
            minmax_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::MinMax(minmax_state) = state {
            minmax_state.finalize()
        } else {
            DataValue::Null
        }
    }
}

/// VARIANCE(column) - computes population variance
pub struct VarianceFunction;

impl AggregateFunction for VarianceFunction {
    fn name(&self) -> &'static str {
        "VARIANCE"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_variance()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

/// STDDEV(column) - computes population standard deviation
pub struct StdDevFunction;

impl AggregateFunction for StdDevFunction {
    fn name(&self) -> &'static str {
        "STDDEV"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_stddev()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

/// STRING_AGG(column, separator) - concatenates strings with separator
pub struct StringAggFunction;

impl AggregateFunction for StringAggFunction {
    fn name(&self) -> &'static str {
        "STRING_AGG"
    }

    fn init(&self) -> AggregateState {
        AggregateState::StringAgg(StringAggState::new(","))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::StringAgg(ref mut agg_state) = state {
            agg_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::StringAgg(agg_state) = state {
            agg_state.finalize()
        } else {
            DataValue::Null
        }
    }
}

/// MEDIAN(column) - finds the median value
pub struct MedianFunction;

impl AggregateFunction for MedianFunction {
    fn name(&self) -> &'static str {
        "MEDIAN"
    }

    fn init(&self) -> AggregateState {
        AggregateState::CollectList(Vec::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::CollectList(ref mut values) = state {
            // Skip null values
            if !matches!(value, DataValue::Null) {
                values.push(value.clone());
            }
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::CollectList(mut values) = state {
            if values.is_empty() {
                return DataValue::Null;
            }

            // Sort values for median calculation
            values.sort_by(|a, b| {
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
                    (DataValue::String(a), DataValue::String(b)) => a.cmp(b),
                    (DataValue::InternedString(a), DataValue::InternedString(b)) => a.cmp(b),
                    (DataValue::String(a), DataValue::InternedString(b)) => a.cmp(&**b),
                    (DataValue::InternedString(a), DataValue::String(b)) => (**a).cmp(b),
                    _ => Ordering::Equal,
                }
            });

            let len = values.len();
            if len % 2 == 1 {
                // Odd number of elements - return middle element
                values[len / 2].clone()
            } else {
                // Even number of elements - return average of two middle elements
                let mid1 = &values[len / 2 - 1];
                let mid2 = &values[len / 2];

                // For numeric values, calculate average
                match (mid1, mid2) {
                    (DataValue::Integer(a), DataValue::Integer(b)) => {
                        let avg = (*a + *b) as f64 / 2.0;
                        if avg.fract() == 0.0 {
                            DataValue::Integer(avg as i64)
                        } else {
                            DataValue::Float(avg)
                        }
                    }
                    (DataValue::Float(a), DataValue::Float(b)) => DataValue::Float((a + b) / 2.0),
                    (DataValue::Integer(a), DataValue::Float(b)) => {
                        DataValue::Float((*a as f64 + b) / 2.0)
                    }
                    (DataValue::Float(a), DataValue::Integer(b)) => {
                        DataValue::Float((a + *b as f64) / 2.0)
                    }
                    // For non-numeric, return the first middle element
                    _ => mid1.clone(),
                }
            }
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        false // MEDIAN works on any sortable type
    }
}

/// PERCENTILE(column, percentile) - finds the nth percentile value
pub struct PercentileFunction;

impl AggregateFunction for PercentileFunction {
    fn name(&self) -> &'static str {
        "PERCENTILE"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Percentile(PercentileState::new(50.0))
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Percentile(ref mut percentile_state) = state {
            percentile_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Percentile(percentile_state) = state {
            percentile_state.finalize()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true // PERCENTILE typically works on numeric data
    }
}

/// MODE(column) - finds the most frequently occurring value
pub struct ModeFunction;

/// STDDEV_POP(column) - computes population standard deviation (same as STDDEV)
pub struct StdDevPopFunction;

/// STDDEV_SAMP(column) - computes sample standard deviation
pub struct StdDevSampFunction;

/// VAR_POP(column) - computes population variance (same as VARIANCE)
pub struct VarPopFunction;

/// VAR_SAMP(column) - computes sample variance
pub struct VarSampFunction;

impl AggregateFunction for StdDevPopFunction {
    fn name(&self) -> &'static str {
        "STDDEV_POP"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_stddev()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

impl AggregateFunction for StdDevSampFunction {
    fn name(&self) -> &'static str {
        "STDDEV_SAMP"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_stddev_sample()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

impl AggregateFunction for VarPopFunction {
    fn name(&self) -> &'static str {
        "VAR_POP"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_variance()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

impl AggregateFunction for VarSampFunction {
    fn name(&self) -> &'static str {
        "VAR_SAMP"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Variance(VarianceState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Variance(ref mut var_state) = state {
            var_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Variance(var_state) = state {
            var_state.finalize_variance_sample()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        true
    }
}

impl AggregateFunction for ModeFunction {
    fn name(&self) -> &'static str {
        "MODE"
    }

    fn init(&self) -> AggregateState {
        AggregateState::Mode(ModeState::new())
    }

    fn accumulate(&self, state: &mut AggregateState, value: &DataValue) -> Result<()> {
        if let AggregateState::Mode(ref mut mode_state) = state {
            mode_state.add(value)?;
        }
        Ok(())
    }

    fn finalize(&self, state: AggregateState) -> DataValue {
        if let AggregateState::Mode(mode_state) = state {
            mode_state.finalize()
        } else {
            DataValue::Null
        }
    }

    fn requires_numeric(&self) -> bool {
        false // MODE works on any data type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_star() {
        let func = CountStarFunction;
        let mut state = func.init();

        // COUNT(*) counts everything including nulls
        func.accumulate(&mut state, &DataValue::Integer(5)).unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap();
        func.accumulate(&mut state, &DataValue::String("test".to_string()))
            .unwrap();

        let result = func.finalize(state);
        assert_eq!(result, DataValue::Integer(3));
    }

    #[test]
    fn test_count_column() {
        let func = CountFunction;
        let mut state = func.init();

        // COUNT(column) skips nulls
        func.accumulate(&mut state, &DataValue::Integer(5)).unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap();
        func.accumulate(&mut state, &DataValue::String("test".to_string()))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap();

        let result = func.finalize(state);
        assert_eq!(result, DataValue::Integer(2));
    }

    #[test]
    fn test_sum_integers() {
        let func = SumFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(20))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(30))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Ignored

        let result = func.finalize(state);
        assert_eq!(result, DataValue::Integer(60));
    }

    #[test]
    fn test_sum_mixed() {
        let func = SumFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Float(20.5))
            .unwrap(); // Converts to float
        func.accumulate(&mut state, &DataValue::Integer(30))
            .unwrap();

        let result = func.finalize(state);
        match result {
            DataValue::Float(f) => assert!((f - 60.5).abs() < 0.001),
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_avg() {
        let func = AvgFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(20))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(30))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Ignored

        let result = func.finalize(state);
        match result {
            DataValue::Float(f) => assert!((f - 20.0).abs() < 0.001),
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_min() {
        let func = MinFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(30))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(20))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Ignored

        let result = func.finalize(state);
        assert_eq!(result, DataValue::Integer(10));
    }

    #[test]
    fn test_max() {
        let func = MaxFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(30))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(20))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Ignored

        let result = func.finalize(state);
        assert_eq!(result, DataValue::Integer(30));
    }

    #[test]
    fn test_max_strings() {
        let func = MaxFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::String("apple".to_string()))
            .unwrap();
        func.accumulate(&mut state, &DataValue::String("zebra".to_string()))
            .unwrap();
        func.accumulate(&mut state, &DataValue::String("banana".to_string()))
            .unwrap();

        let result = func.finalize(state);
        assert_eq!(result, DataValue::String("zebra".to_string()));
    }

    #[test]
    fn test_variance() {
        let func = VarianceFunction;
        let mut state = func.init();

        // Test data: [2, 4, 6, 8, 10]
        // Mean = 6, Variance = 8
        func.accumulate(&mut state, &DataValue::Integer(2)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(4)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(6)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(8)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();

        let result = func.finalize(state);
        match result {
            DataValue::Float(f) => assert!((f - 8.0).abs() < 0.001),
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_stddev() {
        let func = StdDevFunction;
        let mut state = func.init();

        // Test data: [2, 4, 6, 8, 10]
        // Mean = 6, Variance = 8, StdDev = sqrt(8) ≈ 2.828
        func.accumulate(&mut state, &DataValue::Integer(2)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(4)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(6)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(8)).unwrap();
        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();

        let result = func.finalize(state);
        match result {
            DataValue::Float(f) => assert!((f - 2.8284271247461903).abs() < 0.001),
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_variance_with_nulls() {
        let func = VarianceFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::Integer(5)).unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Should be ignored
        func.accumulate(&mut state, &DataValue::Integer(10))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Integer(15))
            .unwrap();

        let result = func.finalize(state);
        match result {
            DataValue::Float(f) => {
                // Mean = 10, values = [5, 10, 15]
                // Variance = ((5-10)² + (10-10)² + (15-10)²) / 3 = (25 + 0 + 25) / 3 ≈ 16.67
                assert!((f - 16.666666666666668).abs() < 0.001);
            }
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_string_agg() {
        let func = StringAggFunction;
        let mut state = func.init();

        func.accumulate(&mut state, &DataValue::String("apple".to_string()))
            .unwrap();
        func.accumulate(&mut state, &DataValue::String("banana".to_string()))
            .unwrap();
        func.accumulate(&mut state, &DataValue::Null).unwrap(); // Should be ignored
        func.accumulate(&mut state, &DataValue::String("cherry".to_string()))
            .unwrap();

        let result = func.finalize(state);
        assert_eq!(result, DataValue::String("apple,banana,cherry".to_string()));
    }
}
