use crate::data::datatable::{DataTable, DataValue};
use crate::sql::recursive_parser::SqlExpression;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Utc};
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

    /// Find a column name similar to the given name using edit distance
    fn find_similar_column(&self, name: &str) -> Option<String> {
        let columns = self.table.column_names();
        let mut best_match: Option<(String, usize)> = None;

        for col in columns {
            let distance = self.edit_distance(&col.to_lowercase(), &name.to_lowercase());
            // Only suggest if distance is small (likely a typo)
            // Allow up to 3 edits for longer names
            let max_distance = if name.len() > 10 { 3 } else { 2 };
            if distance <= max_distance {
                match &best_match {
                    None => best_match = Some((col, distance)),
                    Some((_, best_dist)) if distance < *best_dist => {
                        best_match = Some((col, distance));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(name, _)| name)
    }

    /// Calculate Levenshtein edit distance between two strings
    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    std::cmp::min(
                        matrix[i + 1][j] + 1, // insertion
                        matrix[i][j] + cost,  // substitution
                    ),
                );
            }
        }

        matrix[len1][len2]
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
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => self.evaluate_method_call(object, method, args, row_index),
            SqlExpression::ChainedMethodCall { base, method, args } => {
                // Evaluate the base expression first, then apply the method
                let base_value = self.evaluate(base, row_index)?;
                self.evaluate_method_on_value(&base_value, method, args, row_index)
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => self.evaluate_case_expression(when_branches, else_branch, row_index),
            _ => Err(anyhow!(
                "Unsupported expression type for arithmetic evaluation: {:?}",
                expr
            )),
        }
    }

    /// Evaluate a column reference
    fn evaluate_column(&self, column_name: &str, row_index: usize) -> Result<DataValue> {
        let col_index = self.table.get_column_index(column_name).ok_or_else(|| {
            let suggestion = self.find_similar_column(column_name);
            match suggestion {
                Some(similar) => anyhow!(
                    "Column '{}' not found. Did you mean '{}'?",
                    column_name,
                    similar
                ),
                None => anyhow!("Column '{}' not found", column_name),
            }
        })?;

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
            // Comparison operators (return boolean results)
            ">" => self.compare_values(&left_val, &right_val, |a, b| a > b),
            "<" => self.compare_values(&left_val, &right_val, |a, b| a < b),
            ">=" => self.compare_values(&left_val, &right_val, |a, b| a >= b),
            "<=" => self.compare_values(&left_val, &right_val, |a, b| a <= b),
            "=" => self.compare_values(&left_val, &right_val, |a, b| a == b),
            "!=" | "<>" => self.compare_values(&left_val, &right_val, |a, b| a != b),
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
            DataValue::Float(f) if *f == 0.0 => true, // Only check for exact zero, not epsilon
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

    /// Compare two DataValues using the provided comparison function
    fn compare_values<F>(&self, left: &DataValue, right: &DataValue, op: F) -> Result<DataValue>
    where
        F: Fn(f64, f64) -> bool,
    {
        debug!(
            "ArithmeticEvaluator: comparing values {:?} and {:?}",
            left, right
        );

        let result = match (left, right) {
            // Integer comparisons
            (DataValue::Integer(a), DataValue::Integer(b)) => op(*a as f64, *b as f64),
            (DataValue::Integer(a), DataValue::Float(b)) => op(*a as f64, *b),
            (DataValue::Float(a), DataValue::Integer(b)) => op(*a, *b as f64),
            (DataValue::Float(a), DataValue::Float(b)) => op(*a, *b),

            // String comparisons (lexicographic)
            (DataValue::String(a), DataValue::String(b)) => {
                let a_num = a.parse::<f64>();
                let b_num = b.parse::<f64>();
                match (a_num, b_num) {
                    (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                    _ => op(a.len() as f64, b.len() as f64),    // Fallback to length comparison
                }
            }
            (DataValue::InternedString(a), DataValue::InternedString(b)) => {
                let a_num = a.parse::<f64>();
                let b_num = b.parse::<f64>();
                match (a_num, b_num) {
                    (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                    _ => op(a.len() as f64, b.len() as f64),    // Fallback to length comparison
                }
            }
            (DataValue::String(a), DataValue::InternedString(b)) => {
                let a_num = a.parse::<f64>();
                let b_num = b.parse::<f64>();
                match (a_num, b_num) {
                    (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                    _ => op(a.len() as f64, b.len() as f64),    // Fallback to length comparison
                }
            }
            (DataValue::InternedString(a), DataValue::String(b)) => {
                let a_num = a.parse::<f64>();
                let b_num = b.parse::<f64>();
                match (a_num, b_num) {
                    (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                    _ => op(a.len() as f64, b.len() as f64),    // Fallback to length comparison
                }
            }

            // Mixed type comparisons (try to convert to numbers)
            (DataValue::String(a), DataValue::Integer(b)) => {
                match a.parse::<f64>() {
                    Ok(a_val) => op(a_val, *b as f64),
                    Err(_) => false, // String can't be compared with number
                }
            }
            (DataValue::Integer(a), DataValue::String(b)) => {
                match b.parse::<f64>() {
                    Ok(b_val) => op(*a as f64, b_val),
                    Err(_) => false, // String can't be compared with number
                }
            }
            (DataValue::String(a), DataValue::Float(b)) => match a.parse::<f64>() {
                Ok(a_val) => op(a_val, *b),
                Err(_) => false,
            },
            (DataValue::Float(a), DataValue::String(b)) => match b.parse::<f64>() {
                Ok(b_val) => op(*a, b_val),
                Err(_) => false,
            },

            // NULL comparisons
            (DataValue::Null, _) | (_, DataValue::Null) => false,

            // Boolean comparisons
            (DataValue::Boolean(a), DataValue::Boolean(b)) => {
                op(if *a { 1.0 } else { 0.0 }, if *b { 1.0 } else { 0.0 })
            }

            _ => {
                debug!(
                    "ArithmeticEvaluator: unsupported comparison between {:?} and {:?}",
                    left, right
                );
                false
            }
        };

        debug!("ArithmeticEvaluator: comparison result: {}", result);
        Ok(DataValue::Boolean(result))
    }

    /// Evaluate a function call
    fn evaluate_function(
        &self,
        name: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        // Convert function name to uppercase for case-insensitive matching
        match name.to_uppercase().as_str() {
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
            "MOD" => {
                if args.len() != 2 {
                    return Err(anyhow!("MOD requires exactly 2 arguments"));
                }

                let dividend = self.evaluate(&args[0], row_index)?;
                let divisor = self.evaluate(&args[1], row_index)?;

                match (&dividend, &divisor) {
                    (DataValue::Integer(n), DataValue::Integer(d)) => {
                        if *d == 0 {
                            return Err(anyhow!("Division by zero in MOD"));
                        }
                        Ok(DataValue::Integer(n % d))
                    }
                    _ => {
                        // Convert to float for mixed types
                        let n = match dividend {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("MOD requires numeric arguments")),
                        };
                        let d = match divisor {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("MOD requires numeric arguments")),
                        };
                        if d == 0.0 {
                            return Err(anyhow!("Division by zero in MOD"));
                        }
                        Ok(DataValue::Float(n % d))
                    }
                }
            }
            "QUOTIENT" => {
                if args.len() != 2 {
                    return Err(anyhow!("QUOTIENT requires exactly 2 arguments"));
                }

                let numerator = self.evaluate(&args[0], row_index)?;
                let denominator = self.evaluate(&args[1], row_index)?;

                match (&numerator, &denominator) {
                    (DataValue::Integer(n), DataValue::Integer(d)) => {
                        if *d == 0 {
                            return Err(anyhow!("Division by zero in QUOTIENT"));
                        }
                        Ok(DataValue::Integer(n / d))
                    }
                    _ => {
                        // Convert to float for mixed types
                        let n = match numerator {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("QUOTIENT requires numeric arguments")),
                        };
                        let d = match denominator {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("QUOTIENT requires numeric arguments")),
                        };
                        if d == 0.0 {
                            return Err(anyhow!("Division by zero in QUOTIENT"));
                        }
                        Ok(DataValue::Integer((n / d).trunc() as i64))
                    }
                }
            }
            "POWER" | "POW" => {
                if args.len() != 2 {
                    return Err(anyhow!("POWER requires exactly 2 arguments"));
                }

                let base = self.evaluate(&args[0], row_index)?;
                let exponent = self.evaluate(&args[1], row_index)?;

                match (&base, &exponent) {
                    (DataValue::Integer(b), DataValue::Integer(e)) => {
                        if *e >= 0 && *e <= i32::MAX as i64 {
                            Ok(DataValue::Float((*b as f64).powi(*e as i32)))
                        } else {
                            Ok(DataValue::Float((*b as f64).powf(*e as f64)))
                        }
                    }
                    _ => {
                        // Convert to float for mixed types or floats
                        let b = match base {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("POWER requires numeric arguments")),
                        };
                        let e = match exponent {
                            DataValue::Integer(i) => i as f64,
                            DataValue::Float(f) => f,
                            _ => return Err(anyhow!("POWER requires numeric arguments")),
                        };
                        Ok(DataValue::Float(b.powf(e)))
                    }
                }
            }
            "SQRT" => {
                if args.len() != 1 {
                    return Err(anyhow!("SQRT requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => {
                        if n < 0 {
                            return Err(anyhow!("SQRT of negative number"));
                        }
                        Ok(DataValue::Float((n as f64).sqrt()))
                    }
                    DataValue::Float(f) => {
                        if f < 0.0 {
                            return Err(anyhow!("SQRT of negative number"));
                        }
                        Ok(DataValue::Float(f.sqrt()))
                    }
                    _ => Err(anyhow!("SQRT can only be applied to numeric values")),
                }
            }
            "EXP" => {
                if args.len() != 1 {
                    return Err(anyhow!("EXP requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => Ok(DataValue::Float((n as f64).exp())),
                    DataValue::Float(f) => Ok(DataValue::Float(f.exp())),
                    _ => Err(anyhow!("EXP can only be applied to numeric values")),
                }
            }
            "LN" => {
                if args.len() != 1 {
                    return Err(anyhow!("LN requires exactly 1 argument"));
                }

                let value = self.evaluate(&args[0], row_index)?;
                match value {
                    DataValue::Integer(n) => {
                        if n <= 0 {
                            return Err(anyhow!("LN of non-positive number"));
                        }
                        Ok(DataValue::Float((n as f64).ln()))
                    }
                    DataValue::Float(f) => {
                        if f <= 0.0 {
                            return Err(anyhow!("LN of non-positive number"));
                        }
                        Ok(DataValue::Float(f.ln()))
                    }
                    _ => Err(anyhow!("LN can only be applied to numeric values")),
                }
            }
            "LOG" | "LOG10" => {
                if name == "LOG" && args.len() == 2 {
                    // LOG with custom base
                    let value = self.evaluate(&args[0], row_index)?;
                    let base = self.evaluate(&args[1], row_index)?;

                    let n = match value {
                        DataValue::Integer(i) => i as f64,
                        DataValue::Float(f) => f,
                        _ => return Err(anyhow!("LOG requires numeric arguments")),
                    };
                    let b = match base {
                        DataValue::Integer(i) => i as f64,
                        DataValue::Float(f) => f,
                        _ => return Err(anyhow!("LOG requires numeric arguments")),
                    };

                    if n <= 0.0 {
                        return Err(anyhow!("LOG of non-positive number"));
                    }
                    if b <= 0.0 || b == 1.0 {
                        return Err(anyhow!("Invalid LOG base"));
                    }
                    Ok(DataValue::Float(n.log(b)))
                } else if (name == "LOG" && args.len() == 1) || name == "LOG10" {
                    // LOG10 or LOG with default base 10
                    if args.len() != 1 {
                        return Err(anyhow!("{} requires exactly 1 argument", name));
                    }

                    let value = self.evaluate(&args[0], row_index)?;
                    match value {
                        DataValue::Integer(n) => {
                            if n <= 0 {
                                return Err(anyhow!("LOG10 of non-positive number"));
                            }
                            Ok(DataValue::Float((n as f64).log10()))
                        }
                        DataValue::Float(f) => {
                            if f <= 0.0 {
                                return Err(anyhow!("LOG10 of non-positive number"));
                            }
                            Ok(DataValue::Float(f.log10()))
                        }
                        _ => Err(anyhow!("LOG10 can only be applied to numeric values")),
                    }
                } else {
                    Err(anyhow!("LOG requires 1 or 2 arguments"))
                }
            }
            "PI" => {
                if !args.is_empty() {
                    return Err(anyhow!("PI takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::PI))
            }
            "DATEDIFF" => {
                if args.len() != 3 {
                    return Err(anyhow!(
                        "DATEDIFF requires exactly 3 arguments: unit, date1, date2"
                    ));
                }

                // First argument: unit (day, month, year, hour, minute, second)
                let unit = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s.to_lowercase(),
                    DataValue::InternedString(s) => s.to_lowercase(),
                    _ => return Err(anyhow!("DATEDIFF unit must be a string")),
                };

                // Helper function to parse date/datetime strings
                let parse_datetime = |value: DataValue| -> Result<DateTime<Utc>> {
                    let parse_string = |s: &str| -> Result<DateTime<Utc>> {
                        // Try various date/datetime formats

                        // ISO formats (most common)
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // US format: MM/DD/YYYY or MM-DD-YYYY
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m-%d-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // European format: DD/MM/YYYY or DD-MM-YYYY
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // Excel/Windows format: DD-MMM-YYYY (e.g., 15-Jan-2024)
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%b-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // Full month names: January 15, 2024 or 15 January 2024
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d %B %Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // With time: MM/DD/YYYY HH:MM:SS
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }

                        // ISO 8601 / RFC3339
                        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                            return Ok(dt);
                        }

                        Err(anyhow!("Could not parse date: {}. Supported formats: YYYY-MM-DD, MM/DD/YYYY, DD/MM/YYYY, DD-MMM-YYYY", s))
                    };

                    match value {
                        DataValue::String(s) | DataValue::DateTime(s) => parse_string(&s),
                        DataValue::InternedString(s) => parse_string(s.as_str()),
                        _ => Err(anyhow!("DATEDIFF requires date/datetime values")),
                    }
                };

                // Parse both dates
                let date1 = parse_datetime(self.evaluate(&args[1], row_index)?)?;
                let date2 = parse_datetime(self.evaluate(&args[2], row_index)?)?;

                // Calculate difference based on unit
                let diff = match unit.as_str() {
                    "day" | "days" => {
                        let duration = date2.signed_duration_since(date1);
                        duration.num_days()
                    }
                    "month" | "months" => {
                        // Approximate months as 30.44 days
                        let duration = date2.signed_duration_since(date1);
                        duration.num_days() / 30
                    }
                    "year" | "years" => {
                        // Approximate years as 365.25 days
                        let duration = date2.signed_duration_since(date1);
                        duration.num_days() / 365
                    }
                    "hour" | "hours" => {
                        let duration = date2.signed_duration_since(date1);
                        duration.num_hours()
                    }
                    "minute" | "minutes" => {
                        let duration = date2.signed_duration_since(date1);
                        duration.num_minutes()
                    }
                    "second" | "seconds" => {
                        let duration = date2.signed_duration_since(date1);
                        duration.num_seconds()
                    }
                    _ => {
                        return Err(anyhow!(
                        "Unknown DATEDIFF unit: {}. Use: day, month, year, hour, minute, second",
                        unit
                    ))
                    }
                };

                Ok(DataValue::Integer(diff))
            }
            "NOW" => {
                if !args.is_empty() {
                    return Err(anyhow!("NOW takes no arguments"));
                }
                let now = Utc::now();
                Ok(DataValue::DateTime(
                    now.format("%Y-%m-%d %H:%M:%S").to_string(),
                ))
            }
            "TODAY" => {
                if !args.is_empty() {
                    return Err(anyhow!("TODAY takes no arguments"));
                }
                let today = Utc::now().date_naive();
                Ok(DataValue::String(today.format("%Y-%m-%d").to_string()))
            }
            "DATEADD" => {
                if args.len() != 3 {
                    return Err(anyhow!(
                        "DATEADD requires exactly 3 arguments: unit, number, date"
                    ));
                }

                // First argument: unit (day, month, year, hour, minute, second)
                let unit = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s.to_lowercase(),
                    DataValue::InternedString(s) => s.to_lowercase(),
                    _ => return Err(anyhow!("DATEADD unit must be a string")),
                };

                // Second argument: number to add (can be negative for subtraction)
                let amount = match self.evaluate(&args[1], row_index)? {
                    DataValue::Integer(i) => i,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("DATEADD amount must be a number")),
                };

                // Third argument: base date
                let base_date_value = self.evaluate(&args[2], row_index)?;

                // Reuse the parse_datetime function from DATEDIFF
                let parse_datetime = |value: DataValue| -> Result<DateTime<Utc>> {
                    let parse_string = |s: &str| -> Result<DateTime<Utc>> {
                        // Try various date/datetime formats (same as DATEDIFF)

                        // ISO formats (most common)
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // US format: MM/DD/YYYY or MM-DD-YYYY
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m-%d-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // European format: DD/MM/YYYY or DD-MM-YYYY
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // Excel/Windows format: DD-MMM-YYYY (e.g., 15-Jan-2024)
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%b-%Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // Full month names: January 15, 2024 or 15 January 2024
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }
                        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d %B %Y") {
                            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
                        }

                        // With time: MM/DD/YYYY HH:MM:SS
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }
                        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S") {
                            return Ok(Utc.from_utc_datetime(&dt));
                        }

                        // ISO 8601 / RFC3339
                        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                            return Ok(dt);
                        }

                        Err(anyhow!("Could not parse date: {}. Supported formats: YYYY-MM-DD, MM/DD/YYYY, DD/MM/YYYY, DD-MMM-YYYY", s))
                    };

                    match value {
                        DataValue::String(s) | DataValue::DateTime(s) => parse_string(&s),
                        DataValue::InternedString(s) => parse_string(s.as_str()),
                        _ => Err(anyhow!("DATEADD requires date/datetime values")),
                    }
                };

                // Parse the base date
                let base_date = parse_datetime(base_date_value)?;

                // Add the specified amount based on unit
                let result_date = match unit.as_str() {
                    "day" | "days" => base_date + chrono::Duration::days(amount),
                    "month" | "months" => {
                        // For months, we need to be careful about month boundaries
                        let mut year = base_date.year();
                        let mut month = base_date.month() as i32;
                        let day = base_date.day();

                        month += amount as i32;

                        // Handle month overflow/underflow
                        while month > 12 {
                            month -= 12;
                            year += 1;
                        }
                        while month < 1 {
                            month += 12;
                            year -= 1;
                        }

                        // Create new date, handling day overflow (e.g., Jan 31 + 1 month = Feb 28/29)
                        let target_date = NaiveDate::from_ymd_opt(year, month as u32, day)
                            .unwrap_or_else(|| {
                                // If day doesn't exist in target month, use the last day of that month
                                // Try decreasing days until we find a valid one
                                for test_day in (1..=day).rev() {
                                    if let Some(date) =
                                        NaiveDate::from_ymd_opt(year, month as u32, test_day)
                                    {
                                        return date;
                                    }
                                }
                                // This should never happen, but fallback to day 28 as safety
                                NaiveDate::from_ymd_opt(year, month as u32, 28).unwrap()
                            });

                        Utc.from_utc_datetime(&target_date.and_time(base_date.time()))
                    }
                    "year" | "years" => {
                        let new_year = base_date.year() + amount as i32;
                        let target_date =
                            NaiveDate::from_ymd_opt(new_year, base_date.month(), base_date.day())
                                .unwrap_or_else(|| {
                                    // Handle Feb 29 in non-leap years
                                    NaiveDate::from_ymd_opt(new_year, base_date.month(), 28)
                                        .unwrap()
                                });
                        Utc.from_utc_datetime(&target_date.and_time(base_date.time()))
                    }
                    "hour" | "hours" => base_date + chrono::Duration::hours(amount),
                    "minute" | "minutes" => base_date + chrono::Duration::minutes(amount),
                    "second" | "seconds" => base_date + chrono::Duration::seconds(amount),
                    _ => {
                        return Err(anyhow!(
                            "Unknown DATEADD unit: {}. Use: day, month, year, hour, minute, second",
                            unit
                        ))
                    }
                };

                // Return as datetime string
                Ok(DataValue::DateTime(
                    result_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                ))
            }
            "TEXTJOIN" => {
                if args.len() < 3 {
                    return Err(anyhow!("TEXTJOIN requires at least 3 arguments: delimiter, ignore_empty, text1, [text2, ...]"));
                }

                // First argument: delimiter
                let delimiter = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::Null => String::new(),
                    _ => String::new(),
                };

                // Second argument: ignore_empty (treat as boolean - 0 is false, anything else is true)
                let ignore_empty = match self.evaluate(&args[1], row_index)? {
                    DataValue::Integer(n) => n != 0,
                    DataValue::Float(f) => f != 0.0,
                    DataValue::Boolean(b) => b,
                    DataValue::String(s) => {
                        !s.is_empty() && s != "0" && s.to_lowercase() != "false"
                    }
                    DataValue::InternedString(s) => {
                        !s.is_empty() && s.as_str() != "0" && s.to_lowercase() != "false"
                    }
                    DataValue::Null => false,
                    _ => true,
                };

                // Remaining arguments: values to join
                let mut values = Vec::new();
                for i in 2..args.len() {
                    let value = self.evaluate(&args[i], row_index)?;
                    let string_value = match value {
                        DataValue::String(s) => Some(s),
                        DataValue::InternedString(s) => Some(s.to_string()),
                        DataValue::Integer(n) => Some(n.to_string()),
                        DataValue::Float(f) => Some(f.to_string()),
                        DataValue::Boolean(b) => Some(b.to_string()),
                        DataValue::DateTime(dt) => Some(dt),
                        DataValue::Null => {
                            if ignore_empty {
                                None
                            } else {
                                Some(String::new())
                            }
                        }
                        _ => {
                            if ignore_empty {
                                None
                            } else {
                                Some(String::new())
                            }
                        }
                    };

                    if let Some(s) = string_value {
                        if !ignore_empty || !s.is_empty() {
                            values.push(s);
                        }
                    }
                }

                Ok(DataValue::String(values.join(&delimiter)))
            }
            "PI" => {
                // PI constant - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("PI() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::PI))
            }
            "EULER" => {
                // Euler's number constant - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("EULER() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::E))
            }
            "TAU" => {
                // Tau constant (2*PI) - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("TAU() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::TAU))
            }
            "PHI" => {
                // Golden ratio constant - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("PHI() takes no arguments"));
                }
                Ok(DataValue::Float(1.618033988749895))
            }
            "SQRT2" => {
                // Square root of 2 constant - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("SQRT2() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::SQRT_2))
            }
            "LN2" => {
                // Natural logarithm of 2 - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("LN2() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::LN_2))
            }
            "LN10" => {
                // Natural logarithm of 10 - no arguments
                if !args.is_empty() {
                    return Err(anyhow!("LN10() takes no arguments"));
                }
                Ok(DataValue::Float(std::f64::consts::LN_10))
            }
            // Physics Constants - Fundamental Constants
            "C" | "SPEED_OF_LIGHT" => {
                // Speed of light in vacuum (m/s)
                if !args.is_empty() {
                    return Err(anyhow!("C() takes no arguments"));
                }
                Ok(DataValue::Float(299792458.0))
            }
            "G" | "GRAVITATIONAL_CONSTANT" => {
                // Gravitational constant (m^3 kg^-1 s^-2)
                if !args.is_empty() {
                    return Err(anyhow!("G() takes no arguments"));
                }
                Ok(DataValue::Float(6.67430e-11))
            }
            "H" | "PLANCK" => {
                // Planck constant (J⋅s)
                if !args.is_empty() {
                    return Err(anyhow!("PLANCK() takes no arguments"));
                }
                Ok(DataValue::Float(6.62607015e-34))
            }
            "HBAR" => {
                // Reduced Planck constant ℏ = h/(2π) (J⋅s)
                if !args.is_empty() {
                    return Err(anyhow!("HBAR() takes no arguments"));
                }
                Ok(DataValue::Float(1.054571817e-34))
            }
            "K" | "BOLTZMANN" => {
                // Boltzmann constant (J/K)
                if !args.is_empty() {
                    return Err(anyhow!("BOLTZMANN() takes no arguments"));
                }
                Ok(DataValue::Float(1.380649e-23))
            }
            "NA" | "AVOGADRO" => {
                // Avogadro's number (mol^-1)
                if !args.is_empty() {
                    return Err(anyhow!("AVOGADRO() takes no arguments"));
                }
                Ok(DataValue::Float(6.02214076e23))
            }
            "R" | "GAS_CONSTANT" => {
                // Universal gas constant (J mol^-1 K^-1)
                if !args.is_empty() {
                    return Err(anyhow!("R() takes no arguments"));
                }
                Ok(DataValue::Float(8.314462618))
            }
            // Physics Constants - Electromagnetic
            "E0" | "EPSILON0" | "PERMITTIVITY" => {
                // Electric permittivity of vacuum (F/m)
                if !args.is_empty() {
                    return Err(anyhow!("E0() takes no arguments"));
                }
                Ok(DataValue::Float(8.8541878128e-12))
            }
            "MU0" | "PERMEABILITY" => {
                // Magnetic permeability of vacuum (N/A^2)
                if !args.is_empty() {
                    return Err(anyhow!("MU0() takes no arguments"));
                }
                Ok(DataValue::Float(1.25663706212e-6))
            }
            "QE" | "ELEMENTARY_CHARGE" => {
                // Elementary charge (C)
                if !args.is_empty() {
                    return Err(anyhow!("QE() takes no arguments"));
                }
                Ok(DataValue::Float(1.602176634e-19))
            }
            // Physics Constants - Particle Masses
            "ME" | "MASS_ELECTRON" => {
                // Electron mass (kg)
                if !args.is_empty() {
                    return Err(anyhow!("ME() takes no arguments"));
                }
                Ok(DataValue::Float(9.1093837015e-31))
            }
            "MP" | "MASS_PROTON" => {
                // Proton mass (kg)
                if !args.is_empty() {
                    return Err(anyhow!("MP() takes no arguments"));
                }
                Ok(DataValue::Float(1.67262192369e-27))
            }
            "MN" | "MASS_NEUTRON" => {
                // Neutron mass (kg)
                if !args.is_empty() {
                    return Err(anyhow!("MN() takes no arguments"));
                }
                Ok(DataValue::Float(1.67492749804e-27))
            }
            "AMU" | "ATOMIC_MASS_UNIT" => {
                // Atomic mass unit (kg)
                if !args.is_empty() {
                    return Err(anyhow!("AMU() takes no arguments"));
                }
                Ok(DataValue::Float(1.66053906660e-27))
            }
            // Physics Constants - Other
            "ALPHA" | "FINE_STRUCTURE" => {
                // Fine structure constant (dimensionless)
                if !args.is_empty() {
                    return Err(anyhow!("ALPHA() takes no arguments"));
                }
                Ok(DataValue::Float(7.2973525693e-3))
            }
            "RY" | "RYDBERG" => {
                // Rydberg constant (m^-1)
                if !args.is_empty() {
                    return Err(anyhow!("RYDBERG() takes no arguments"));
                }
                Ok(DataValue::Float(10973731.568160))
            }
            "SIGMA" | "STEFAN_BOLTZMANN" => {
                // Stefan-Boltzmann constant (W m^-2 K^-4)
                if !args.is_empty() {
                    return Err(anyhow!("SIGMA() takes no arguments"));
                }
                Ok(DataValue::Float(5.670374419e-8))
            }
            // Angle conversion functions (demonstrating easy extensibility)
            "DEGREES" => {
                // Convert radians to degrees
                if args.len() != 1 {
                    return Err(anyhow!("DEGREES requires exactly 1 argument"));
                }
                let radians = match self.evaluate(&args[0], row_index)? {
                    DataValue::Integer(n) => n as f64,
                    DataValue::Float(f) => f,
                    _ => return Err(anyhow!("DEGREES requires a numeric argument")),
                };
                Ok(DataValue::Float(radians * 180.0 / std::f64::consts::PI))
            }
            "RADIANS" => {
                // Convert degrees to radians
                if args.len() != 1 {
                    return Err(anyhow!("RADIANS requires exactly 1 argument"));
                }
                let degrees = match self.evaluate(&args[0], row_index)? {
                    DataValue::Integer(n) => n as f64,
                    DataValue::Float(f) => f,
                    _ => return Err(anyhow!("RADIANS requires a numeric argument")),
                };
                Ok(DataValue::Float(degrees * std::f64::consts::PI / 180.0))
            }
            "MID" => {
                // MID(text, start_position, length) - Excel-compatible, 1-based indexing
                if args.len() != 3 {
                    return Err(anyhow!(
                        "MID requires exactly 3 arguments: text, start_position, length"
                    ));
                }

                let text = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Null => String::new(),
                    _ => return Err(anyhow!("MID first argument must be convertible to text")),
                };

                let start_pos = match self.evaluate(&args[1], row_index)? {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("MID start_position must be a number")),
                };

                let length = match self.evaluate(&args[2], row_index)? {
                    DataValue::Integer(n) => n,
                    DataValue::Float(f) => f as i64,
                    _ => return Err(anyhow!("MID length must be a number")),
                };

                // Excel MID uses 1-based indexing
                if start_pos < 1 {
                    return Err(anyhow!("MID start_position must be >= 1"));
                }
                if length < 0 {
                    return Err(anyhow!("MID length must be >= 0"));
                }

                // Convert to 0-based index
                let start_idx = (start_pos - 1) as usize;
                let chars: Vec<char> = text.chars().collect();

                // If start position is beyond string length, return empty string
                if start_idx >= chars.len() {
                    return Ok(DataValue::String(String::new()));
                }

                // Extract substring
                let end_idx = std::cmp::min(start_idx + length as usize, chars.len());
                let result: String = chars[start_idx..end_idx].iter().collect();

                Ok(DataValue::String(result))
            }
            "UPPER" => {
                if args.len() != 1 {
                    return Err(anyhow!("UPPER requires exactly 1 argument"));
                }

                let text = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Null => String::new(),
                    _ => return Err(anyhow!("UPPER argument must be convertible to text")),
                };

                Ok(DataValue::String(text.to_uppercase()))
            }
            "LOWER" => {
                if args.len() != 1 {
                    return Err(anyhow!("LOWER requires exactly 1 argument"));
                }

                let text = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Null => String::new(),
                    _ => return Err(anyhow!("LOWER argument must be convertible to text")),
                };

                Ok(DataValue::String(text.to_lowercase()))
            }
            "TRIM" => {
                if args.len() != 1 {
                    return Err(anyhow!("TRIM requires exactly 1 argument"));
                }

                let text = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Null => String::new(),
                    _ => return Err(anyhow!("TRIM argument must be convertible to text")),
                };

                Ok(DataValue::String(text.trim().to_string()))
            }
            _ => Err(anyhow!("Unknown function: {}", name)),
        }
    }

    /// Evaluate a method call on a column (e.g., column.Trim())
    fn evaluate_method_call(
        &self,
        object: &str,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        // Get column value
        let col_index = self.table.get_column_index(object).ok_or_else(|| {
            let suggestion = self.find_similar_column(object);
            match suggestion {
                Some(similar) => {
                    anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                }
                None => anyhow!("Column '{}' not found", object),
            }
        })?;

        let cell_value = self.table.get_value(row_index, col_index).cloned();

        self.evaluate_method_on_value(
            &cell_value.unwrap_or(DataValue::Null),
            method,
            args,
            row_index,
        )
    }

    /// Evaluate a method on a value
    fn evaluate_method_on_value(
        &self,
        value: &DataValue,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        match method.to_lowercase().as_str() {
            "trim" | "trimstart" | "trimend" => {
                if !args.is_empty() {
                    return Err(anyhow!("{} takes no arguments", method));
                }

                // Convert value to string and apply trim
                let str_val = match value {
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Null => return Ok(DataValue::Null),
                };

                let result = match method.to_lowercase().as_str() {
                    "trim" => str_val.trim().to_string(),
                    "trimstart" => str_val.trim_start().to_string(),
                    "trimend" => str_val.trim_end().to_string(),
                    _ => unreachable!(),
                };

                Ok(DataValue::String(result))
            }
            "length" => {
                if !args.is_empty() {
                    return Err(anyhow!("Length takes no arguments"));
                }

                // Get string length
                let len = match value {
                    DataValue::String(s) => s.len(),
                    DataValue::InternedString(s) => s.len(),
                    DataValue::Integer(n) => n.to_string().len(),
                    DataValue::Float(f) => f.to_string().len(),
                    DataValue::Boolean(b) => b.to_string().len(),
                    DataValue::DateTime(dt) => dt.len(),
                    DataValue::Null => return Ok(DataValue::Integer(0)),
                };

                Ok(DataValue::Integer(len as i64))
            }
            "indexof" => {
                if args.len() != 1 {
                    return Err(anyhow!("IndexOf requires exactly 1 argument"));
                }

                // Get the search string from args
                let search_str = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    _ => return Err(anyhow!("IndexOf argument must be a string")),
                };

                // Convert value to string and find index
                let str_val = match value {
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Null => return Ok(DataValue::Integer(-1)),
                };

                let index = str_val.find(&search_str).map(|i| i as i64).unwrap_or(-1);

                Ok(DataValue::Integer(index))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(anyhow!("Contains requires exactly 1 argument"));
                }

                // Get the search string from args
                let search_str = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    _ => return Err(anyhow!("Contains argument must be a string")),
                };

                // Convert value to string and check contains
                let str_val = match value {
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Null => return Ok(DataValue::Boolean(false)),
                };

                // Case-insensitive search
                let result = str_val.to_lowercase().contains(&search_str.to_lowercase());
                Ok(DataValue::Boolean(result))
            }
            "startswith" => {
                if args.len() != 1 {
                    return Err(anyhow!("StartsWith requires exactly 1 argument"));
                }

                // Get the prefix from args
                let prefix = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    _ => return Err(anyhow!("StartsWith argument must be a string")),
                };

                // Convert value to string and check starts_with
                let str_val = match value {
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Null => return Ok(DataValue::Boolean(false)),
                };

                // Case-insensitive check
                let result = str_val.to_lowercase().starts_with(&prefix.to_lowercase());
                Ok(DataValue::Boolean(result))
            }
            "endswith" => {
                if args.len() != 1 {
                    return Err(anyhow!("EndsWith requires exactly 1 argument"));
                }

                // Get the suffix from args
                let suffix = match self.evaluate(&args[0], row_index)? {
                    DataValue::String(s) => s,
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    _ => return Err(anyhow!("EndsWith argument must be a string")),
                };

                // Convert value to string and check ends_with
                let str_val = match value {
                    DataValue::String(s) => s.clone(),
                    DataValue::InternedString(s) => s.to_string(),
                    DataValue::Integer(n) => n.to_string(),
                    DataValue::Float(f) => f.to_string(),
                    DataValue::Boolean(b) => b.to_string(),
                    DataValue::DateTime(dt) => dt.clone(),
                    DataValue::Null => return Ok(DataValue::Boolean(false)),
                };

                // Case-insensitive check
                let result = str_val.to_lowercase().ends_with(&suffix.to_lowercase());
                Ok(DataValue::Boolean(result))
            }
            _ => Err(anyhow!("Unsupported method: {}", method)),
        }
    }

    /// Evaluate a CASE expression
    fn evaluate_case_expression(
        &self,
        when_branches: &[crate::sql::recursive_parser::WhenBranch],
        else_branch: &Option<Box<SqlExpression>>,
        row_index: usize,
    ) -> Result<DataValue> {
        debug!(
            "ArithmeticEvaluator: evaluating CASE expression for row {}",
            row_index
        );

        // Evaluate each WHEN condition in order
        for branch in when_branches {
            // Evaluate the condition as a boolean
            let condition_result = self.evaluate_condition_as_bool(&branch.condition, row_index)?;

            if condition_result {
                debug!("CASE: WHEN condition matched, evaluating result expression");
                return self.evaluate(&branch.result, row_index);
            }
        }

        // If no WHEN condition matched, evaluate ELSE clause (or return NULL)
        match else_branch {
            Some(else_expr) => {
                debug!("CASE: No WHEN matched, evaluating ELSE expression");
                self.evaluate(else_expr, row_index)
            }
            None => {
                debug!("CASE: No WHEN matched and no ELSE, returning NULL");
                Ok(DataValue::Null)
            }
        }
    }

    /// Helper method to evaluate an expression as a boolean (for CASE WHEN conditions)
    fn evaluate_condition_as_bool(&self, expr: &SqlExpression, row_index: usize) -> Result<bool> {
        let value = self.evaluate(expr, row_index)?;

        match value {
            DataValue::Boolean(b) => Ok(b),
            DataValue::Integer(i) => Ok(i != 0),
            DataValue::Float(f) => Ok(f != 0.0),
            DataValue::Null => Ok(false),
            DataValue::String(s) => Ok(!s.is_empty()),
            DataValue::InternedString(s) => Ok(!s.is_empty()),
            _ => Ok(true), // Other types are considered truthy
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
