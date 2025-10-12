// Window Function Registry
// Provides a clean API for window computations with syntactic sugar

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::datatable::DataValue;
use crate::sql::parser::ast::{SqlExpression, WindowSpec};
use crate::sql::window_context::WindowContext;

/// Window function computation trait
/// Each window function receives:
/// - The window context (partitions, ordering, frames)
/// - The current row index
/// - Arguments (column names, parameters)
pub trait WindowFunction: Send + Sync {
    /// Function name (e.g., "MOVING_AVG")
    fn name(&self) -> &str;

    /// Description for help system
    fn description(&self) -> &str;

    /// Signature for documentation (e.g., "MOVING_AVG(column, window_size)")
    fn signature(&self) -> &str;

    /// Compute the function value for a specific row
    /// This is called once per row in the result set
    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue>;

    /// Optional: Transform/expand the window specification
    /// This allows functions to modify the window (e.g., MOVING_AVG sets ROWS n PRECEDING)
    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        _args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        // Default: use the base spec unchanged
        Ok(base_spec.clone())
    }

    /// Validate arguments at parse time
    fn validate_args(&self, _args: &[SqlExpression]) -> Result<()> {
        Ok(())
    }
}

/// Expression evaluator trait for evaluating arguments
/// This allows window functions to evaluate expressions without depending on ArithmeticEvaluator
pub trait ExpressionEvaluator {
    fn evaluate(&mut self, expr: &SqlExpression, row_index: usize) -> Result<DataValue>;
}

/// Registry for window functions
pub struct WindowFunctionRegistry {
    functions: HashMap<String, Arc<Box<dyn WindowFunction>>>,
}

impl WindowFunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_builtin_functions();
        registry
    }

    /// Register a window function
    pub fn register(&mut self, function: Box<dyn WindowFunction>) {
        let name = function.name().to_uppercase();
        self.functions.insert(name, Arc::new(function));
    }

    /// Get a window function by name
    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn WindowFunction>>> {
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

    /// Register built-in syntactic sugar functions
    fn register_builtin_functions(&mut self) {
        // Moving average and statistics
        self.register(Box::new(MovingAvgFunction));
        self.register(Box::new(RollingStddevFunction));
        self.register(Box::new(CumulativeSumFunction));
        self.register(Box::new(CumulativeAvgFunction));
        self.register(Box::new(ZScoreFunction));

        // Bollinger Bands
        self.register(Box::new(BollingerUpperFunction));
        self.register(Box::new(BollingerLowerFunction));

        // Financial calculations
        self.register(Box::new(PercentChangeFunction));

        // Add more as we implement them
    }
}

// ============= Syntactic Sugar Implementations =============

/// MOVING_AVG(column, window_size)
/// Expands to: AVG(column) OVER (ORDER BY <inherited> ROWS window_size-1 PRECEDING)
struct MovingAvgFunction;

impl WindowFunction for MovingAvgFunction {
    fn name(&self) -> &str {
        "MOVING_AVG"
    }

    fn description(&self) -> &str {
        "Calculate moving average over specified window size"
    }

    fn signature(&self) -> &str {
        "MOVING_AVG(column, window_size)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        // Extract column name
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => {
                return Err(anyhow::anyhow!(
                    "MOVING_AVG first argument must be a column"
                ))
            }
        };

        // The window has already been configured by transform_window_spec
        // Just compute the average over the frame
        context
            .get_frame_avg(row_index, &column.name)
            .ok_or_else(|| anyhow::anyhow!("Failed to compute moving average"))
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        // Extract window size from second argument
        let window_size = match &args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("Invalid window size"))?,
            _ => return Err(anyhow::anyhow!("MOVING_AVG requires numeric window_size")),
        };

        // Create a new spec with ROWS n-1 PRECEDING frame
        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(window_size - 1),
            end: None, // Defaults to CURRENT ROW
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!("MOVING_AVG requires exactly 2 arguments"));
        }
        Ok(())
    }
}

/// ROLLING_STDDEV(column, window_size)
/// Expands to: STDDEV(column) OVER (ORDER BY <inherited> ROWS window_size-1 PRECEDING)
struct RollingStddevFunction;

impl WindowFunction for RollingStddevFunction {
    fn name(&self) -> &str {
        "ROLLING_STDDEV"
    }

    fn description(&self) -> &str {
        "Calculate rolling standard deviation over specified window"
    }

    fn signature(&self) -> &str {
        "ROLLING_STDDEV(column, window_size)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => {
                return Err(anyhow::anyhow!(
                    "ROLLING_STDDEV first argument must be a column"
                ))
            }
        };

        context
            .get_frame_stddev(row_index, &column.name)
            .ok_or_else(|| anyhow::anyhow!("Failed to compute rolling stddev"))
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let window_size = match &args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("Invalid window size"))?,
            _ => {
                return Err(anyhow::anyhow!(
                    "ROLLING_STDDEV requires numeric window_size"
                ))
            }
        };

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(window_size - 1),
            end: None,
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "ROLLING_STDDEV requires exactly 2 arguments"
            ));
        }
        Ok(())
    }
}

/// CUMULATIVE_SUM(column)
/// Expands to: SUM(column) OVER (ORDER BY <inherited> ROWS UNBOUNDED PRECEDING)
struct CumulativeSumFunction;

impl WindowFunction for CumulativeSumFunction {
    fn name(&self) -> &str {
        "CUMULATIVE_SUM"
    }

    fn description(&self) -> &str {
        "Calculate cumulative sum from beginning to current row"
    }

    fn signature(&self) -> &str {
        "CUMULATIVE_SUM(column)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow::anyhow!("CUMULATIVE_SUM argument must be a column")),
        };

        context
            .get_frame_sum(row_index, &column.name)
            .ok_or_else(|| anyhow::anyhow!("Failed to compute cumulative sum"))
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        _args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::UnboundedPreceding,
            end: None, // CURRENT ROW
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "CUMULATIVE_SUM requires exactly 1 argument"
            ));
        }
        Ok(())
    }
}

/// CUMULATIVE_AVG(column)
/// Expands to: AVG(column) OVER (ORDER BY <inherited> ROWS UNBOUNDED PRECEDING)
struct CumulativeAvgFunction;

impl WindowFunction for CumulativeAvgFunction {
    fn name(&self) -> &str {
        "CUMULATIVE_AVG"
    }

    fn description(&self) -> &str {
        "Calculate cumulative average from beginning to current row"
    }

    fn signature(&self) -> &str {
        "CUMULATIVE_AVG(column)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow::anyhow!("CUMULATIVE_AVG argument must be a column")),
        };

        context
            .get_frame_avg(row_index, &column.name)
            .ok_or_else(|| anyhow::anyhow!("Failed to compute cumulative average"))
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        _args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::UnboundedPreceding,
            end: None,
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "CUMULATIVE_AVG requires exactly 1 argument"
            ));
        }
        Ok(())
    }
}

/// Z_SCORE(column, window_size)
/// Calculates: (value - mean) / stddev over the window
struct ZScoreFunction;

impl WindowFunction for ZScoreFunction {
    fn name(&self) -> &str {
        "Z_SCORE"
    }

    fn description(&self) -> &str {
        "Calculate Z-score (standard deviations from mean) over window"
    }

    fn signature(&self) -> &str {
        "Z_SCORE(column, window_size)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow::anyhow!("Z_SCORE first argument must be a column")),
        };

        // Get current value
        let current_value = {
            let source = context.source();
            let col_idx = source
                .get_column_index(&column.name)
                .ok_or_else(|| anyhow::anyhow!("Column {} not found", column))?;
            source
                .get_value(row_index, col_idx)
                .cloned()
                .unwrap_or(DataValue::Null)
        };

        // Get mean and stddev over the window
        let mean = context
            .get_frame_avg(row_index, &column.name)
            .unwrap_or(DataValue::Null);
        let stddev = context
            .get_frame_stddev(row_index, &column.name)
            .unwrap_or(DataValue::Null);

        // Calculate Z-score
        match (current_value, mean, stddev) {
            (DataValue::Integer(v), DataValue::Float(m), DataValue::Float(s)) if s > 0.0 => {
                Ok(DataValue::Float((v as f64 - m) / s))
            }
            (DataValue::Float(v), DataValue::Float(m), DataValue::Float(s)) if s > 0.0 => {
                Ok(DataValue::Float((v - m) / s))
            }
            _ => Ok(DataValue::Null),
        }
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let window_size = match &args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("Invalid window size"))?,
            _ => return Err(anyhow::anyhow!("Z_SCORE requires numeric window_size")),
        };

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(window_size - 1),
            end: None,
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!("Z_SCORE requires exactly 2 arguments"));
        }
        Ok(())
    }
}

/// BOLLINGER_UPPER(column, window_size, num_std)
/// Calculates upper Bollinger Band: MA + (num_std * STDDEV)
struct BollingerUpperFunction;

impl WindowFunction for BollingerUpperFunction {
    fn name(&self) -> &str {
        "BOLLINGER_UPPER"
    }

    fn description(&self) -> &str {
        "Calculate upper Bollinger Band (MA + n*STDDEV)"
    }

    fn signature(&self) -> &str {
        "BOLLINGER_UPPER(column, window_size, num_std)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow!("BOLLINGER_UPPER first argument must be a column")),
        };

        // Get num_std from third argument (default 2)
        let num_std = match args.get(2) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<f64>()
                .map_err(|_| anyhow!("Invalid num_std value"))?,
            _ => 2.0, // Default to 2 standard deviations
        };

        // Get mean and stddev over the window
        let mean = context
            .get_frame_avg(row_index, &column.name)
            .unwrap_or(DataValue::Null);
        let stddev = context
            .get_frame_stddev(row_index, &column.name)
            .unwrap_or(DataValue::Null);

        // Calculate upper band: mean + (num_std * stddev)
        match (mean, stddev) {
            (DataValue::Float(m), DataValue::Float(s)) => Ok(DataValue::Float(m + (num_std * s))),
            _ => Ok(DataValue::Null),
        }
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let window_size = match args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i64>()
                .map_err(|_| anyhow!("Invalid window size"))?,
            _ => return Err(anyhow!("BOLLINGER_UPPER requires numeric window_size")),
        };

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(window_size - 1),
            end: None,
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!("BOLLINGER_UPPER requires 2 or 3 arguments"));
        }
        Ok(())
    }
}

/// BOLLINGER_LOWER(column, window_size, num_std)
/// Calculates lower Bollinger Band: MA - (num_std * STDDEV)
struct BollingerLowerFunction;

impl WindowFunction for BollingerLowerFunction {
    fn name(&self) -> &str {
        "BOLLINGER_LOWER"
    }

    fn description(&self) -> &str {
        "Calculate lower Bollinger Band (MA - n*STDDEV)"
    }

    fn signature(&self) -> &str {
        "BOLLINGER_LOWER(column, window_size, num_std)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow!("BOLLINGER_LOWER first argument must be a column")),
        };

        // Get num_std from third argument (default 2)
        let num_std = match args.get(2) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<f64>()
                .map_err(|_| anyhow!("Invalid num_std value"))?,
            _ => 2.0, // Default to 2 standard deviations
        };

        // Get mean and stddev over the window
        let mean = context
            .get_frame_avg(row_index, &column.name)
            .unwrap_or(DataValue::Null);
        let stddev = context
            .get_frame_stddev(row_index, &column.name)
            .unwrap_or(DataValue::Null);

        // Calculate lower band: mean - (num_std * stddev)
        match (mean, stddev) {
            (DataValue::Float(m), DataValue::Float(s)) => Ok(DataValue::Float(m - (num_std * s))),
            _ => Ok(DataValue::Null),
        }
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        use crate::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame};

        let window_size = match args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i64>()
                .map_err(|_| anyhow!("Invalid window size"))?,
            _ => return Err(anyhow!("BOLLINGER_LOWER requires numeric window_size")),
        };

        let mut spec = base_spec.clone();
        spec.frame = Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(window_size - 1),
            end: None,
        });

        Ok(spec)
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!("BOLLINGER_LOWER requires 2 or 3 arguments"));
        }
        Ok(())
    }
}

/// PERCENT_CHANGE(column, periods)
/// Calculates percentage change from N periods ago
/// Formula: ((current - previous) / previous) * 100
struct PercentChangeFunction;

impl WindowFunction for PercentChangeFunction {
    fn name(&self) -> &str {
        "PERCENT_CHANGE"
    }

    fn description(&self) -> &str {
        "Calculate percentage change from N periods ago"
    }

    fn signature(&self) -> &str {
        "PERCENT_CHANGE(column, periods)"
    }

    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        _evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue> {
        let column = match &args[0] {
            SqlExpression::Column(col) => col,
            _ => return Err(anyhow!("PERCENT_CHANGE first argument must be a column")),
        };

        // Get periods from second argument (default 1)
        let periods = match args.get(1) {
            Some(SqlExpression::NumberLiteral(n)) => n
                .parse::<i32>()
                .map_err(|_| anyhow!("Invalid periods value"))?,
            _ => 1, // Default to 1 period
        };

        // Get current value
        let current_value = {
            let source = context.source();
            let col_idx = source
                .get_column_index(&column.name)
                .ok_or_else(|| anyhow!("Column {} not found", column))?;
            source.get_value(row_index, col_idx).cloned()
        };

        // Get previous value using offset
        let previous_value = context.get_offset_value(row_index, -periods, &column.name);

        // Calculate percent change: ((current - previous) / previous) * 100
        match (current_value, previous_value) {
            (Some(DataValue::Float(curr)), Some(DataValue::Float(prev))) if prev != 0.0 => {
                Ok(DataValue::Float(((curr - prev) / prev) * 100.0))
            }
            (Some(DataValue::Integer(curr)), Some(DataValue::Integer(prev))) if prev != 0 => {
                let curr_f = curr as f64;
                let prev_f = prev as f64;
                Ok(DataValue::Float(((curr_f - prev_f) / prev_f) * 100.0))
            }
            (Some(DataValue::Float(curr)), Some(DataValue::Integer(prev))) if prev != 0 => {
                let prev_f = prev as f64;
                Ok(DataValue::Float(((curr - prev_f) / prev_f) * 100.0))
            }
            (Some(DataValue::Integer(curr)), Some(DataValue::Float(prev))) if prev != 0.0 => {
                let curr_f = curr as f64;
                Ok(DataValue::Float(((curr_f - prev) / prev) * 100.0))
            }
            _ => Ok(DataValue::Null), // Return NULL for invalid comparisons or division by zero
        }
    }

    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        _args: &[SqlExpression],
    ) -> Result<WindowSpec> {
        // PERCENT_CHANGE doesn't need to modify the window frame
        // It uses LAG internally which works within the partition
        Ok(base_spec.clone())
    }

    fn validate_args(&self, args: &[SqlExpression]) -> Result<()> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!("PERCENT_CHANGE requires 1 or 2 arguments"));
        }
        Ok(())
    }
}

// TODO: Add more functions like:
// - EXPONENTIAL_AVG(column, alpha)
// - PERCENT_RANK_IN_WINDOW(column, window)
// - MEDIAN_IN_WINDOW(column, window)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::ColumnRef;

    #[test]
    fn test_registry_creation() {
        let registry = WindowFunctionRegistry::new();
        assert!(registry.contains("MOVING_AVG"));
        assert!(registry.contains("ROLLING_STDDEV"));
        assert!(registry.contains("CUMULATIVE_SUM"));
    }

    #[test]
    fn test_window_spec_transformation() {
        use crate::sql::parser::ast::{FrameBound, WindowSpec};

        let func = MovingAvgFunction;
        let base_spec = WindowSpec {
            partition_by: vec![],
            order_by: vec![],
            frame: None,
        };

        let args = vec![
            SqlExpression::Column(ColumnRef::unquoted("close".to_string())),
            SqlExpression::NumberLiteral("20".to_string()),
        ];

        let transformed = func.transform_window_spec(&base_spec, &args).unwrap();

        assert!(transformed.frame.is_some());
        let frame = transformed.frame.unwrap();
        assert_eq!(frame.start, FrameBound::Preceding(19));
    }
}
