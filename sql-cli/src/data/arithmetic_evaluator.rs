use crate::data::datatable::{DataTable, DataValue};
use crate::sql::recursive_parser::SqlExpression;
use anyhow::{anyhow, Result};
use tracing::debug;

/// Evaluates SQL expressions to compute DataValues (for SELECT clauses)
/// This is different from RecursiveWhereEvaluator which returns boolean
pub struct ArithmeticEvaluator<'a> {
    table: &'a DataTable,
}

impl<'a> ArithmeticEvaluator<'a> {
    pub fn new(table: &'a DataTable) -> Self {
        Self { table }
    }

    /// Evaluate an SQL expression to produce a DataValue
    pub fn evaluate(&self, expr: &SqlExpression, row_index: usize) -> Result<DataValue> {
        debug!(
            "ArithmeticEvaluator: evaluating {:?} for row {}",
            expr, row_index
        );

        match expr {
            SqlExpression::Column(column_name) => self.evaluate_column(column_name, row_index),
            SqlExpression::StringLiteral(s) => Ok(DataValue::String(s.clone())),
            SqlExpression::NumberLiteral(n) => self.evaluate_number_literal(n),
            SqlExpression::BinaryOp { left, op, right } => {
                self.evaluate_binary_op(left, op, right, row_index)
            }
            SqlExpression::FunctionCall { name, args } => {
                self.evaluate_function(name, args, row_index)
            }
            _ => Err(anyhow!(
                "Unsupported expression type for arithmetic evaluation: {:?}",
                expr
            )),
        }
    }

    /// Evaluate a column reference
    fn evaluate_column(&self, column_name: &str, row_index: usize) -> Result<DataValue> {
        let col_index = self
            .table
            .get_column_index(column_name)
            .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

        if row_index >= self.table.row_count() {
            return Err(anyhow!("Row index {} out of bounds", row_index));
        }

        let row = self
            .table
            .get_row(row_index)
            .ok_or_else(|| anyhow!("Row {} not found", row_index))?;

        let value = row
            .get(col_index)
            .ok_or_else(|| anyhow!("Column index {} out of bounds for row", col_index))?;

        Ok(value.clone())
    }

    /// Evaluate a number literal (handles both integers and floats)
    fn evaluate_number_literal(&self, number_str: &str) -> Result<DataValue> {
        // Try to parse as integer first
        if let Ok(int_val) = number_str.parse::<i64>() {
            return Ok(DataValue::Integer(int_val));
        }

        // If that fails, try as float
        if let Ok(float_val) = number_str.parse::<f64>() {
            return Ok(DataValue::Float(float_val));
        }

        Err(anyhow!("Invalid number literal: {}", number_str))
    }

    /// Evaluate a binary operation (arithmetic)
    fn evaluate_binary_op(
        &self,
        left: &SqlExpression,
        op: &str,
        right: &SqlExpression,
        row_index: usize,
    ) -> Result<DataValue> {
        let left_val = self.evaluate(left, row_index)?;
        let right_val = self.evaluate(right, row_index)?;

        debug!(
            "ArithmeticEvaluator: {} {} {}",
            self.format_value(&left_val),
            op,
            self.format_value(&right_val)
        );

        match op {
            "+" => self.add_values(&left_val, &right_val),
            "-" => self.subtract_values(&left_val, &right_val),
            "*" => self.multiply_values(&left_val, &right_val),
            "/" => self.divide_values(&left_val, &right_val),
            _ => Err(anyhow!("Unsupported arithmetic operator: {}", op)),
        }
    }

    /// Add two DataValues with type coercion
    fn add_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a + b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 + b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a + *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a + b)),
            _ => Err(anyhow!("Cannot add {:?} and {:?}", left, right)),
        }
    }

    /// Subtract two DataValues with type coercion
    fn subtract_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a - b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 - b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a - *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a - b)),
            _ => Err(anyhow!("Cannot subtract {:?} and {:?}", left, right)),
        }
    }

    /// Multiply two DataValues with type coercion
    fn multiply_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a * b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 * b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a * *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a * b)),
            _ => Err(anyhow!("Cannot multiply {:?} and {:?}", left, right)),
        }
    }

    /// Divide two DataValues with type coercion
    fn divide_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        // Check for division by zero first
        let is_zero = match right {
            DataValue::Integer(0) => true,
            DataValue::Float(f) if f.abs() < f64::EPSILON => true,
            _ => false,
        };

        if is_zero {
            return Err(anyhow!("Division by zero"));
        }

        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => {
                // Integer division - if result is exact, keep as int, otherwise promote to float
                if a % b == 0 {
                    Ok(DataValue::Integer(a / b))
                } else {
                    Ok(DataValue::Float(*a as f64 / *b as f64))
                }
            }
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 / b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a / *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a / b)),
            _ => Err(anyhow!("Cannot divide {:?} and {:?}", left, right)),
        }
    }

    /// Format a DataValue for debug output
    fn format_value(&self, value: &DataValue) -> String {
        match value {
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::String(s) => format!("'{}'", s),
            _ => format!("{:?}", value),
        }
    }

    /// Evaluate a function call
    fn evaluate_function(
        &self,
        name: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        match name {
            "ROUND" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(anyhow!("ROUND requires 1 or 2 arguments"));
                }

                // Evaluate the value to round
                let value = self.evaluate(&args[0], row_index)?;

                // Get decimal places (default to 0 if not specified)
                let decimals = if args.len() == 2 {
                    match self.evaluate(&args[1], row_index)? {
                        DataValue::Integer(n) => n as i32,
                        DataValue::Float(f) => f as i32,
                        _ => return Err(anyhow!("ROUND precision must be a number")),
                    }
                } else {
                    0
                };

                // Perform rounding
                match value {
                    DataValue::Integer(n) => Ok(DataValue::Integer(n)), // Already an integer
                    DataValue::Float(f) => {
                        if decimals >= 0 {
                            let multiplier = 10_f64.powi(decimals);
                            let rounded = (f * multiplier).round() / multiplier;
                            if decimals == 0 {
                                // Return as integer if rounding to 0 decimals
                                Ok(DataValue::Integer(rounded as i64))
                            } else {
                                Ok(DataValue::Float(rounded))
                            }
                        } else {
                            // Negative decimals round to left of decimal point
                            let divisor = 10_f64.powi(-decimals);
                            let rounded = (f / divisor).round() * divisor;
                            Ok(DataValue::Float(rounded))
                        }
                    }
                    _ => Err(anyhow!("ROUND can only be applied to numeric values")),
                }
            }
            "ABS" => {
                if args.len() != 1 {
                    return Err(anyhow!("ABS requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => Ok(DataValue::Integer(n.abs())),
                    DataValue::Float(f) => Ok(DataValue::Float(f.abs())),
                    _ => Err(anyhow!("ABS can only be applied to numeric values")),
                }
            }
            "FLOOR" => {
                if args.len() != 1 {
                    return Err(anyhow!("FLOOR requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => Ok(DataValue::Integer(n)),
                    DataValue::Float(f) => Ok(DataValue::Integer(f.floor() as i64)),
                    _ => Err(anyhow!("FLOOR can only be applied to numeric values")),
                }
            }
            "CEILING" | "CEIL" => {
                if args.len() != 1 {
                    return Err(anyhow!("CEILING requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => Ok(DataValue::Integer(n)),
                    DataValue::Float(f) => Ok(DataValue::Integer(f.ceil() as i64)),
                    _ => Err(anyhow!("CEILING can only be applied to numeric values")),
                }
            }
            _ => Err(anyhow!("Unknown function: {}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};

    fn create_test_table() -> DataTable {
        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("a"));
        table.add_column(DataColumn::new("b"));
        table.add_column(DataColumn::new("c"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(10),
                DataValue::Float(2.5),
                DataValue::Integer(4),
            ]))
            .unwrap();

        table
    }

    #[test]
    fn test_evaluate_column() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        let expr = SqlExpression::Column("a".to_string());
        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Integer(10));
    }

    #[test]
    fn test_evaluate_number_literal() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        let expr = SqlExpression::NumberLiteral("42".to_string());
        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Integer(42));

        let expr = SqlExpression::NumberLiteral("3.14".to_string());
        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Float(3.14));
    }

    #[test]
    fn test_add_values() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        // Integer + Integer
        let result = evaluator
            .add_values(&DataValue::Integer(5), &DataValue::Integer(3))
            .unwrap();
        assert_eq!(result, DataValue::Integer(8));

        // Integer + Float
        let result = evaluator
            .add_values(&DataValue::Integer(5), &DataValue::Float(2.5))
            .unwrap();
        assert_eq!(result, DataValue::Float(7.5));
    }

    #[test]
    fn test_multiply_values() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        // Integer * Float
        let result = evaluator
            .multiply_values(&DataValue::Integer(4), &DataValue::Float(2.5))
            .unwrap();
        assert_eq!(result, DataValue::Float(10.0));
    }

    #[test]
    fn test_divide_values() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        // Exact division
        let result = evaluator
            .divide_values(&DataValue::Integer(10), &DataValue::Integer(2))
            .unwrap();
        assert_eq!(result, DataValue::Integer(5));

        // Non-exact division
        let result = evaluator
            .divide_values(&DataValue::Integer(10), &DataValue::Integer(3))
            .unwrap();
        assert_eq!(result, DataValue::Float(10.0 / 3.0));
    }

    #[test]
    fn test_division_by_zero() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        let result = evaluator.divide_values(&DataValue::Integer(10), &DataValue::Integer(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_binary_op_expression() {
        let table = create_test_table();
        let evaluator = ArithmeticEvaluator::new(&table);

        // a * b where a=10, b=2.5
        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column("a".to_string())),
            op: "*".to_string(),
            right: Box::new(SqlExpression::Column("b".to_string())),
        };

        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Float(25.0));
    }
}
