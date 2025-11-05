// Window aggregate functions that can handle expressions
// These wrap standard aggregates but evaluate expressions within window contexts

use super::{ExpressionEvaluator, WindowFunction};
use crate::data::datatable::DataValue;
use crate::sql::parser::ast::SqlExpression;
use crate::sql::window_context::WindowContext;
use anyhow::{anyhow, Result};

/// Window SUM aggregate that can handle expressions
/// Example: SUM(price * quantity) OVER (PARTITION BY ...)
pub struct WindowSumFunction;

impl WindowFunction for WindowSumFunction {
    fn name(&self) -> &str {
        "SUM"
    }

    fn description(&self) -> &str {
        "Calculate sum of expression over window"
    }

    fn signature(&self) -> &str {
        "SUM(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("SUM requires 1 argument"));
        }

        // Get the rows to sum over (frame-aware)
        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        // Evaluate the expression for each row and sum
        let mut sum: Option<DataValue> = None;
        let mut has_non_null = false;

        for &frame_row_idx in &frame_rows {
            // Evaluate the expression at this row
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;

            if !matches!(value, DataValue::Null) {
                has_non_null = true;
                match (&sum, &value) {
                    (None, DataValue::Integer(v)) => sum = Some(DataValue::Integer(*v)),
                    (None, DataValue::Float(v)) => sum = Some(DataValue::Float(*v)),
                    (Some(DataValue::Integer(s)), DataValue::Integer(v)) => {
                        sum = Some(DataValue::Integer(s + v));
                    }
                    (Some(DataValue::Integer(s)), DataValue::Float(v)) => {
                        sum = Some(DataValue::Float(*s as f64 + v));
                    }
                    (Some(DataValue::Float(s)), DataValue::Integer(v)) => {
                        sum = Some(DataValue::Float(s + *v as f64));
                    }
                    (Some(DataValue::Float(s)), DataValue::Float(v)) => {
                        sum = Some(DataValue::Float(s + v));
                    }
                    _ => {} // Skip non-numeric values
                }
            }
        }

        Ok(sum.unwrap_or(DataValue::Null))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("SUM requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Window AVG aggregate that can handle expressions
pub struct WindowAvgFunction;

impl WindowFunction for WindowAvgFunction {
    fn name(&self) -> &str {
        "AVG"
    }

    fn description(&self) -> &str {
        "Calculate average of expression over window"
    }

    fn signature(&self) -> &str {
        "AVG(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("AVG requires 1 argument"));
        }

        // Get the rows to average over
        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        // Evaluate the expression for each row and compute average
        let mut sum = 0.0;
        let mut count = 0;

        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;

            match value {
                DataValue::Integer(v) => {
                    sum += v as f64;
                    count += 1;
                }
                DataValue::Float(v) => {
                    sum += v;
                    count += 1;
                }
                DataValue::Null => {} // Skip nulls
                _ => {}               // Skip non-numeric values
            }
        }

        if count > 0 {
            Ok(DataValue::Float(sum / count as f64))
        } else {
            Ok(DataValue::Null)
        }
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("AVG requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Window MIN aggregate that can handle expressions
pub struct WindowMinFunction;

impl WindowFunction for WindowMinFunction {
    fn name(&self) -> &str {
        "MIN"
    }

    fn description(&self) -> &str {
        "Calculate minimum of expression over window"
    }

    fn signature(&self) -> &str {
        "MIN(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("MIN requires 1 argument"));
        }

        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        let mut min_value: Option<DataValue> = None;

        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;

            if !matches!(value, DataValue::Null) {
                match &min_value {
                    None => min_value = Some(value),
                    Some(current_min) => {
                        if value < *current_min {
                            min_value = Some(value);
                        }
                    }
                }
            }
        }

        Ok(min_value.unwrap_or(DataValue::Null))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("MIN requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Window MAX aggregate that can handle expressions
pub struct WindowMaxFunction;

impl WindowFunction for WindowMaxFunction {
    fn name(&self) -> &str {
        "MAX"
    }

    fn description(&self) -> &str {
        "Calculate maximum of expression over window"
    }

    fn signature(&self) -> &str {
        "MAX(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("MAX requires 1 argument"));
        }

        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        let mut max_value: Option<DataValue> = None;

        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;

            if !matches!(value, DataValue::Null) {
                match &max_value {
                    None => max_value = Some(value),
                    Some(current_max) => {
                        if value > *current_max {
                            max_value = Some(value);
                        }
                    }
                }
            }
        }

        Ok(max_value.unwrap_or(DataValue::Null))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("MAX requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Window COUNT aggregate that can handle expressions
pub struct WindowCountFunction;

impl WindowFunction for WindowCountFunction {
    fn name(&self) -> &str {
        "COUNT"
    }

    fn description(&self) -> &str {
        "Count non-null values of expression over window"
    }

    fn signature(&self) -> &str {
        "COUNT(expression | *) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        // Handle COUNT(*)
        if args.is_empty()
            || (args.len() == 1
                && matches!(&args[0],
                SqlExpression::Column(col) if col.name == "*" || 
                matches!(&args[0], SqlExpression::StringLiteral(s) if s == "*")))
        {
            return Ok(DataValue::Integer(frame_rows.len() as i64));
        }

        // COUNT(expression) - count non-null values
        let mut count = 0;
        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;
            if !matches!(value, DataValue::Null) {
                count += 1;
            }
        }

        Ok(DataValue::Integer(count))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() > 1 {
            return Err(anyhow!("COUNT requires 0 or 1 arguments"));
        }
        Ok(())
    }
}

/// Window STDDEV aggregate that can handle expressions
pub struct WindowStddevFunction;

impl WindowFunction for WindowStddevFunction {
    fn name(&self) -> &str {
        "STDDEV"
    }

    fn description(&self) -> &str {
        "Calculate standard deviation of expression over window"
    }

    fn signature(&self) -> &str {
        "STDDEV(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("STDDEV requires 1 argument"));
        }

        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        // Collect values
        let mut values = Vec::new();
        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;
            match value {
                DataValue::Integer(v) => values.push(v as f64),
                DataValue::Float(v) => values.push(v),
                DataValue::Null => {} // Skip nulls
                _ => {}               // Skip non-numeric
            }
        }

        if values.len() <= 1 {
            return Ok(DataValue::Null);
        }

        // Calculate mean
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate standard deviation
        let variance =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;

        Ok(DataValue::Float(variance.sqrt()))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("STDDEV requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Window VARIANCE aggregate that can handle expressions
pub struct WindowVarianceFunction;

impl WindowFunction for WindowVarianceFunction {
    fn name(&self) -> &str {
        "VARIANCE"
    }

    fn description(&self) -> &str {
        "Calculate variance of expression over window"
    }

    fn signature(&self) -> &str {
        "VARIANCE(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("VARIANCE requires 1 argument"));
        }

        let frame_rows = if context.has_frame() {
            context.get_frame_rows(row_index)
        } else {
            context.get_partition_rows(row_index)
        };

        // Collect values
        let mut values = Vec::new();
        for &frame_row_idx in &frame_rows {
            let value = evaluator.evaluate(&args[0], frame_row_idx)?;
            match value {
                DataValue::Integer(v) => values.push(v as f64),
                DataValue::Float(v) => values.push(v),
                DataValue::Null => {} // Skip nulls
                _ => {}               // Skip non-numeric
            }
        }

        if values.len() <= 1 {
            return Ok(DataValue::Null);
        }

        // Calculate mean
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;

        Ok(DataValue::Float(variance))
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow!("VARIANCE requires exactly 1 argument"));
        }
        Ok(())
    }
}

/// Alias for STDDEV
pub struct WindowStdevFunction;

impl WindowFunction for WindowStdevFunction {
    fn name(&self) -> &str {
        "STDEV"
    }

    fn description(&self) -> &str {
        "Calculate standard deviation of expression over window (alias for STDDEV)"
    }

    fn signature(&self) -> &str {
        "STDEV(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        // Delegate to STDDEV implementation
        WindowStddevFunction.compute(context, row_index, args, evaluator)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        WindowStddevFunction.validate_args(args)
    }
}

/// Alias for VARIANCE
pub struct WindowVarFunction;

impl WindowFunction for WindowVarFunction {
    fn name(&self) -> &str {
        "VAR"
    }

    fn description(&self) -> &str {
        "Calculate variance of expression over window (alias for VARIANCE)"
    }

    fn signature(&self) -> &str {
        "VAR(expression) OVER (...)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        // Delegate to VARIANCE implementation
        WindowVarianceFunction.compute(context, row_index, args, evaluator)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        WindowVarianceFunction.validate_args(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::ColumnRef;

    #[test]
    fn test_window_sum_function() {
        let func = WindowSumFunction;
        assert_eq!(func.name(), "SUM");

        // Validate args
        let args = vec![SqlExpression::Column(ColumnRef::unquoted(
            "amount".to_string(),
        ))];
        assert!(func.validate_args(&args).is_ok());

        let empty_args: Vec<SqlExpression> = vec![];
        assert!(func.validate_args(&empty_args).is_err());
    }

    #[test]
    fn test_window_count_function() {
        let func = WindowCountFunction;
        assert_eq!(func.name(), "COUNT");

        // COUNT(*) with no args is valid
        let empty_args: Vec<SqlExpression> = vec![];
        assert!(func.validate_args(&empty_args).is_ok());

        // COUNT(column) is valid
        let args = vec![SqlExpression::Column(ColumnRef::unquoted("id".to_string()))];
        assert!(func.validate_args(&args).is_ok());

        // COUNT with 2 args is invalid
        let two_args = vec![
            SqlExpression::Column(ColumnRef::unquoted("id".to_string())),
            SqlExpression::Column(ColumnRef::unquoted("name".to_string())),
        ];
        assert!(func.validate_args(&two_args).is_err());
    }
}
