//! Concrete implementations of aggregate functions

use anyhow::Result;

use super::{AggregateFunction, AggregateState, AvgState, MinMaxState, SumState};
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
}
