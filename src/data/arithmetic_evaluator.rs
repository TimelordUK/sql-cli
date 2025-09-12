use crate::config::global::get_date_notation;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::sql::aggregates::AggregateRegistry;
use crate::sql::functions::date_time::parse_datetime;
use crate::sql::functions::FunctionRegistry;
use crate::sql::recursive_parser::{SqlExpression, WindowSpec};
use crate::sql::window_context::WindowContext;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// Evaluates SQL expressions to compute `DataValues` (for SELECT clauses)
/// This is different from `RecursiveWhereEvaluator` which returns boolean
pub struct ArithmeticEvaluator<'a> {
    table: &'a DataTable,
    date_notation: String,
    function_registry: Arc<FunctionRegistry>,
    aggregate_registry: Arc<AggregateRegistry>,
    visible_rows: Option<Vec<usize>>, // For aggregate functions on filtered views
    window_contexts: HashMap<String, Arc<WindowContext>>, // Cache window contexts by spec
}

impl<'a> ArithmeticEvaluator<'a> {
    #[must_use]
    pub fn new(table: &'a DataTable) -> Self {
        Self {
            table,
            date_notation: get_date_notation(),
            function_registry: Arc::new(FunctionRegistry::new()),
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_date_notation(table: &'a DataTable, date_notation: String) -> Self {
        Self {
            table,
            date_notation,
            function_registry: Arc::new(FunctionRegistry::new()),
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
        }
    }

    /// Set visible rows for aggregate functions (for filtered views)
    #[must_use]
    pub fn with_visible_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = Some(rows);
        self
    }

    #[must_use]
    pub fn with_date_notation_and_registry(
        table: &'a DataTable,
        date_notation: String,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        Self {
            table,
            date_notation,
            function_registry,
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
        }
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
        // Use the shared implementation from string_methods
        crate::sql::functions::string_methods::EditDistanceFunction::calculate_edit_distance(s1, s2)
    }

    /// Evaluate an SQL expression to produce a `DataValue`
    pub fn evaluate(&mut self, expr: &SqlExpression, row_index: usize) -> Result<DataValue> {
        debug!(
            "ArithmeticEvaluator: evaluating {:?} for row {}",
            expr, row_index
        );

        match expr {
            SqlExpression::Column(column_name) => self.evaluate_column(column_name, row_index),
            SqlExpression::StringLiteral(s) => Ok(DataValue::String(s.clone())),
            SqlExpression::BooleanLiteral(b) => Ok(DataValue::Boolean(*b)),
            SqlExpression::NumberLiteral(n) => self.evaluate_number_literal(n),
            SqlExpression::Null => Ok(DataValue::Null),
            SqlExpression::BinaryOp { left, op, right } => {
                self.evaluate_binary_op(left, op, right, row_index)
            }
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => self.evaluate_function_with_distinct(name, args, *distinct, row_index),
            SqlExpression::WindowFunction {
                name,
                args,
                window_spec,
            } => self.evaluate_window_function(name, args, window_spec, row_index),
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
        &mut self,
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
            "%" => {
                // Modulo operator - call MOD function
                let args = vec![left.clone(), right.clone()];
                self.evaluate_function("MOD", &args, row_index)
            }
            // Comparison operators (return boolean results)
            ">" => self.compare_values(&left_val, &right_val, |a, b| a > b),
            "<" => self.compare_values(&left_val, &right_val, |a, b| a < b),
            ">=" => self.compare_values(&left_val, &right_val, |a, b| a >= b),
            "<=" => self.compare_values(&left_val, &right_val, |a, b| a <= b),
            "=" => self.compare_values(&left_val, &right_val, |a, b| a == b),
            "!=" | "<>" => self.compare_values(&left_val, &right_val, |a, b| a != b),
            // IS NULL / IS NOT NULL operators
            "IS NULL" => Ok(DataValue::Boolean(matches!(left_val, DataValue::Null))),
            "IS NOT NULL" => Ok(DataValue::Boolean(!matches!(left_val, DataValue::Null))),
            _ => Err(anyhow!("Unsupported arithmetic operator: {}", op)),
        }
    }

    /// Add two `DataValues` with type coercion
    fn add_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        // NULL handling - any operation with NULL returns NULL
        if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
            return Ok(DataValue::Null);
        }

        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a + b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 + b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a + *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a + b)),
            _ => Err(anyhow!("Cannot add {:?} and {:?}", left, right)),
        }
    }

    /// Subtract two `DataValues` with type coercion
    fn subtract_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        // NULL handling - any operation with NULL returns NULL
        if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
            return Ok(DataValue::Null);
        }

        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a - b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 - b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a - *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a - b)),
            _ => Err(anyhow!("Cannot subtract {:?} and {:?}", left, right)),
        }
    }

    /// Multiply two `DataValues` with type coercion
    fn multiply_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        // NULL handling - any operation with NULL returns NULL
        if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
            return Ok(DataValue::Null);
        }

        match (left, right) {
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(DataValue::Integer(a * b)),
            (DataValue::Integer(a), DataValue::Float(b)) => Ok(DataValue::Float(*a as f64 * b)),
            (DataValue::Float(a), DataValue::Integer(b)) => Ok(DataValue::Float(a * *b as f64)),
            (DataValue::Float(a), DataValue::Float(b)) => Ok(DataValue::Float(a * b)),
            _ => Err(anyhow!("Cannot multiply {:?} and {:?}", left, right)),
        }
    }

    /// Divide two `DataValues` with type coercion
    fn divide_values(&self, left: &DataValue, right: &DataValue) -> Result<DataValue> {
        // NULL handling - any operation with NULL returns NULL
        if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
            return Ok(DataValue::Null);
        }

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

    /// Format a `DataValue` for debug output
    fn format_value(&self, value: &DataValue) -> String {
        match value {
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::String(s) => format!("'{s}'"),
            _ => format!("{value:?}"),
        }
    }

    /// Compare two `DataValues` using the provided comparison function
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

            // String comparisons (try date first, then numeric, then lexicographic)
            (DataValue::String(a), DataValue::String(b)) => {
                // First try to parse as dates
                let a_date = parse_datetime(a);
                let b_date = parse_datetime(b);

                debug!(
                    "ArithmeticEvaluator: Comparing strings '{}' and '{}' - dates: {:?}, {:?}",
                    a,
                    b,
                    a_date.is_ok(),
                    b_date.is_ok()
                );

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => {
                        // Both are valid dates, compare timestamps
                        let result = op(a_dt.timestamp() as f64, b_dt.timestamp() as f64);
                        debug!("ArithmeticEvaluator: Date comparison result: {}", result);
                        result
                    }
                    _ => {
                        // Not dates, try as numbers
                        let a_num = a.parse::<f64>();
                        let b_num = b.parse::<f64>();
                        match (a_num, b_num) {
                            (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                            _ => op(a.len() as f64, b.len() as f64), // Fallback to length comparison
                        }
                    }
                }
            }
            (DataValue::InternedString(a), DataValue::InternedString(b)) => {
                // First try to parse as dates
                let a_date = parse_datetime(a.as_str());
                let b_date = parse_datetime(b.as_str());

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => {
                        // Both are valid dates, compare timestamps
                        op(a_dt.timestamp() as f64, b_dt.timestamp() as f64)
                    }
                    _ => {
                        // Not dates, try as numbers
                        let a_num = a.parse::<f64>();
                        let b_num = b.parse::<f64>();
                        match (a_num, b_num) {
                            (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                            _ => op(a.len() as f64, b.len() as f64), // Fallback to length comparison
                        }
                    }
                }
            }
            (DataValue::String(a), DataValue::InternedString(b)) => {
                // First try to parse as dates
                let a_date = parse_datetime(a);
                let b_date = parse_datetime(b.as_str());

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => {
                        // Both are valid dates, compare timestamps
                        op(a_dt.timestamp() as f64, b_dt.timestamp() as f64)
                    }
                    _ => {
                        // Not dates, try as numbers
                        let a_num = a.parse::<f64>();
                        let b_num = b.parse::<f64>();
                        match (a_num, b_num) {
                            (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                            _ => op(a.len() as f64, b.len() as f64), // Fallback to length comparison
                        }
                    }
                }
            }
            (DataValue::InternedString(a), DataValue::String(b)) => {
                // First try to parse as dates
                let a_date = parse_datetime(a.as_str());
                let b_date = parse_datetime(b);

                debug!("ArithmeticEvaluator: Comparing InternedString '{}' and String '{}' - dates: {:?}, {:?}", a, b, a_date.is_ok(), b_date.is_ok());

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => {
                        // Both are valid dates, compare timestamps
                        let result = op(a_dt.timestamp() as f64, b_dt.timestamp() as f64);
                        debug!("ArithmeticEvaluator: Date comparison result: {}", result);
                        result
                    }
                    _ => {
                        // Not dates, try as numbers
                        let a_num = a.parse::<f64>();
                        let b_num = b.parse::<f64>();
                        match (a_num, b_num) {
                            (Ok(a_val), Ok(b_val)) => op(a_val, b_val), // Both are numbers
                            _ => op(a.len() as f64, b.len() as f64), // Fallback to length comparison
                        }
                    }
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

            // DateTime comparisons (stored as ISO strings in DateTime variant)
            (DataValue::DateTime(a), DataValue::DateTime(b)) => {
                // DateTime values are stored as ISO strings, parse and compare them
                let a_date = parse_datetime(a);
                let b_date = parse_datetime(b);

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => op(a_dt.timestamp() as f64, b_dt.timestamp() as f64),
                    _ => {
                        // Fallback to string comparison if parsing fails
                        op(a.len() as f64, b.len() as f64)
                    }
                }
            }
            (DataValue::DateTime(a), DataValue::String(b))
            | (DataValue::String(b), DataValue::DateTime(a)) => {
                // Compare DateTime with String by parsing both as dates
                let a_date = parse_datetime(a);
                let b_date = parse_datetime(b);

                match (a_date, b_date) {
                    (Ok(a_dt), Ok(b_dt)) => op(a_dt.timestamp() as f64, b_dt.timestamp() as f64),
                    _ => false,
                }
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
    fn evaluate_function_with_distinct(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        distinct: bool,
        row_index: usize,
    ) -> Result<DataValue> {
        // If DISTINCT is specified, handle it specially for aggregate functions
        if distinct {
            let name_upper = name.to_uppercase();

            // DISTINCT is only valid for aggregate functions
            if name_upper == "COUNT"
                || name_upper == "SUM"
                || name_upper == "AVG"
                || name_upper == "MIN"
                || name_upper == "MAX"
            {
                return self.evaluate_aggregate_distinct(&name_upper, args, row_index);
            } else {
                return Err(anyhow!(
                    "DISTINCT can only be used with aggregate functions"
                ));
            }
        }

        // Otherwise, use the regular evaluation
        self.evaluate_function(name, args, row_index)
    }

    fn evaluate_aggregate_distinct(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        use std::collections::HashSet;

        if args.is_empty() {
            return Err(anyhow!("{} DISTINCT requires at least one argument", name));
        }

        // Determine which rows to process
        let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
            visible.clone()
        } else {
            (0..self.table.rows.len()).collect()
        };

        // Collect unique values
        let mut unique_values = HashSet::new();
        let mut numeric_values = Vec::new();

        for row_idx in &rows_to_process {
            // Evaluate the expression for this row
            let value = self.evaluate(&args[0], *row_idx)?;

            // Skip NULL values
            if matches!(value, DataValue::Null) {
                continue;
            }

            // Convert to string for uniqueness check
            let value_str = match &value {
                DataValue::String(s) => s.clone(),
                DataValue::InternedString(s) => s.to_string(),
                DataValue::Integer(i) => i.to_string(),
                DataValue::Float(f) => f.to_string(),
                DataValue::Boolean(b) => b.to_string(),
                DataValue::DateTime(dt) => dt.to_string(),
                DataValue::Null => continue,
            };

            // Only process if we haven't seen this value before
            if unique_values.insert(value_str) {
                // For numeric aggregates, collect the numeric value
                if name != "COUNT" {
                    match value {
                        DataValue::Integer(i) => numeric_values.push(i as f64),
                        DataValue::Float(f) => numeric_values.push(f),
                        _ => {} // Skip non-numeric for SUM/AVG
                    }
                }
            }
        }

        // Calculate the result based on the aggregate function
        match name {
            "COUNT" => Ok(DataValue::Integer(unique_values.len() as i64)),
            "SUM" => {
                if numeric_values.is_empty() {
                    Ok(DataValue::Null)
                } else {
                    let sum: f64 = numeric_values.iter().sum();
                    if sum.fract() == 0.0 && sum.abs() < 1e10 {
                        Ok(DataValue::Integer(sum as i64))
                    } else {
                        Ok(DataValue::Float(sum))
                    }
                }
            }
            "AVG" => {
                if numeric_values.is_empty() {
                    Ok(DataValue::Null)
                } else {
                    let sum: f64 = numeric_values.iter().sum();
                    Ok(DataValue::Float(sum / numeric_values.len() as f64))
                }
            }
            "MIN" => {
                if numeric_values.is_empty() {
                    Ok(DataValue::Null)
                } else {
                    let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    if min.fract() == 0.0 && min.abs() < 1e10 {
                        Ok(DataValue::Integer(min as i64))
                    } else {
                        Ok(DataValue::Float(min))
                    }
                }
            }
            "MAX" => {
                if numeric_values.is_empty() {
                    Ok(DataValue::Null)
                } else {
                    let max = numeric_values
                        .iter()
                        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    if max.fract() == 0.0 && max.abs() < 1e10 {
                        Ok(DataValue::Integer(max as i64))
                    } else {
                        Ok(DataValue::Float(max))
                    }
                }
            }
            _ => Err(anyhow!("Unsupported DISTINCT aggregate: {}", name)),
        }
    }

    fn evaluate_function(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        // Check if this is an aggregate function
        let name_upper = name.to_uppercase();

        // Handle COUNT(*) special case
        if name_upper == "COUNT" && args.len() == 1 {
            match &args[0] {
                SqlExpression::Column(col) if col == "*" => {
                    // COUNT(*) - count all rows (or visible rows if filtered)
                    let count = if let Some(ref visible) = self.visible_rows {
                        visible.len() as i64
                    } else {
                        self.table.rows.len() as i64
                    };
                    return Ok(DataValue::Integer(count));
                }
                SqlExpression::StringLiteral(s) if s == "*" => {
                    // COUNT(*) parsed as StringLiteral
                    let count = if let Some(ref visible) = self.visible_rows {
                        visible.len() as i64
                    } else {
                        self.table.rows.len() as i64
                    };
                    return Ok(DataValue::Integer(count));
                }
                _ => {
                    // COUNT(column) - will be handled below
                }
            }
        }

        // Check aggregate registry
        if self.aggregate_registry.get(&name_upper).is_some() {
            // Determine which rows to process first
            let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
                visible.clone()
            } else {
                (0..self.table.rows.len()).collect()
            };

            // Evaluate arguments first if needed (to avoid borrow issues)
            let values = if !args.is_empty()
                && !(args.len() == 1 && matches!(&args[0], SqlExpression::Column(c) if c == "*"))
            {
                // Evaluate the argument expression for each row
                let mut vals = Vec::new();
                for &row_idx in &rows_to_process {
                    let value = self.evaluate(&args[0], row_idx)?;
                    vals.push(value);
                }
                Some(vals)
            } else {
                None
            };

            // Now get the aggregate function and process
            let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
            let mut state = agg_func.init();

            if let Some(values) = values {
                // Use evaluated values
                for value in &values {
                    agg_func.accumulate(&mut state, value)?;
                }
            } else {
                // COUNT(*) case
                for _ in &rows_to_process {
                    agg_func.accumulate(&mut state, &DataValue::Integer(1))?;
                }
            }

            return Ok(agg_func.finalize(state));
        }

        // First check if this function exists in the registry
        if self.function_registry.get(name).is_some() {
            // Evaluate all arguments first to avoid borrow issues
            let mut evaluated_args = Vec::new();
            for arg in args {
                evaluated_args.push(self.evaluate(arg, row_index)?);
            }

            // Get the function and call it
            let func = self.function_registry.get(name).unwrap();
            return func.evaluate(&evaluated_args);
        }

        // If not in registry, return error for unknown function
        Err(anyhow!("Unknown function: {}", name))
    }

    /// Get or create a WindowContext for the given specification
    fn get_or_create_window_context(&mut self, spec: &WindowSpec) -> Result<Arc<WindowContext>> {
        // Create a key for caching based on the spec
        let key = format!("{:?}", spec);

        if let Some(context) = self.window_contexts.get(&key) {
            return Ok(Arc::clone(context));
        }

        // Create a DataView from the table (with visible rows if filtered)
        let data_view = if let Some(ref _visible_rows) = self.visible_rows {
            // Create a filtered view
            let view = DataView::new(Arc::new(self.table.clone()));
            // Apply filtering based on visible rows
            // Note: This is a simplified approach - in production we'd need proper filtering
            view
        } else {
            DataView::new(Arc::new(self.table.clone()))
        };

        // Create the WindowContext
        let context = WindowContext::new(
            Arc::new(data_view),
            spec.partition_by.clone(),
            spec.order_by.clone(),
        )?;

        let context = Arc::new(context);
        self.window_contexts.insert(key, Arc::clone(&context));
        Ok(context)
    }

    /// Evaluate a window function
    fn evaluate_window_function(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        spec: &WindowSpec,
        row_index: usize,
    ) -> Result<DataValue> {
        let context = self.get_or_create_window_context(spec)?;
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "LAG" => {
                // LAG(column, offset, default)
                if args.is_empty() {
                    return Err(anyhow!("LAG requires at least 1 argument"));
                }

                // Get column name
                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("LAG first argument must be a column")),
                };

                // Get offset (default 1)
                let offset = if args.len() > 1 {
                    match self.evaluate(&args[1], row_index)? {
                        DataValue::Integer(i) => i as i32,
                        _ => return Err(anyhow!("LAG offset must be an integer")),
                    }
                } else {
                    1
                };

                // Get value at offset
                Ok(context
                    .get_offset_value(row_index, -offset, &column)
                    .unwrap_or(DataValue::Null))
            }
            "LEAD" => {
                // LEAD(column, offset, default)
                if args.is_empty() {
                    return Err(anyhow!("LEAD requires at least 1 argument"));
                }

                // Get column name
                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("LEAD first argument must be a column")),
                };

                // Get offset (default 1)
                let offset = if args.len() > 1 {
                    match self.evaluate(&args[1], row_index)? {
                        DataValue::Integer(i) => i as i32,
                        _ => return Err(anyhow!("LEAD offset must be an integer")),
                    }
                } else {
                    1
                };

                // Get value at offset
                Ok(context
                    .get_offset_value(row_index, offset, &column)
                    .unwrap_or(DataValue::Null))
            }
            "ROW_NUMBER" => {
                // ROW_NUMBER() - no arguments
                Ok(DataValue::Integer(context.get_row_number(row_index) as i64))
            }
            "FIRST_VALUE" => {
                // FIRST_VALUE(column)
                if args.is_empty() {
                    return Err(anyhow!("FIRST_VALUE requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("FIRST_VALUE argument must be a column")),
                };

                Ok(context
                    .get_first_value(row_index, &column)
                    .unwrap_or(DataValue::Null))
            }
            "LAST_VALUE" => {
                // LAST_VALUE(column)
                if args.is_empty() {
                    return Err(anyhow!("LAST_VALUE requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("LAST_VALUE argument must be a column")),
                };

                Ok(context
                    .get_last_value(row_index, &column)
                    .unwrap_or(DataValue::Null))
            }
            "SUM" => {
                // SUM(column) OVER (PARTITION BY ...)
                if args.is_empty() {
                    return Err(anyhow!("SUM requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("SUM argument must be a column")),
                };

                Ok(context
                    .get_partition_sum(row_index, &column)
                    .unwrap_or(DataValue::Null))
            }
            "COUNT" => {
                // COUNT(*) or COUNT(column) OVER (PARTITION BY ...)
                // Note: In window functions, COUNT(*) seems to come with no args
                if args.is_empty() {
                    // COUNT(*) OVER (...) - count all rows in partition
                    Ok(context
                        .get_partition_count(row_index, None)
                        .unwrap_or(DataValue::Null))
                } else {
                    // Check for COUNT(*)
                    let column = match &args[0] {
                        SqlExpression::Column(col) => {
                            if col == "*" {
                                // COUNT(*) - count all rows in partition
                                return Ok(context
                                    .get_partition_count(row_index, None)
                                    .unwrap_or(DataValue::Null));
                            }
                            col.clone()
                        }
                        SqlExpression::StringLiteral(s) if s == "*" => {
                            // COUNT(*) as StringLiteral (how parser sends it)
                            return Ok(context
                                .get_partition_count(row_index, None)
                                .unwrap_or(DataValue::Null));
                        }
                        _ => return Err(anyhow!("COUNT argument must be a column or *")),
                    };

                    // COUNT(column) - count non-null values
                    Ok(context
                        .get_partition_count(row_index, Some(&column))
                        .unwrap_or(DataValue::Null))
                }
            }
            _ => Err(anyhow!("Unknown window function: {}", name)),
        }
    }

    /// Evaluate a method call on a column (e.g., `column.Trim()`)
    fn evaluate_method_call(
        &mut self,
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
        &mut self,
        value: &DataValue,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        // First, try to proxy the method through the function registry
        // Many string methods have corresponding functions (TRIM, LENGTH, CONTAINS, etc.)

        // Map method names to function names (case-insensitive matching)
        let function_name = match method.to_lowercase().as_str() {
            "trim" => "TRIM",
            "trimstart" | "trimbegin" => "TRIMSTART",
            "trimend" => "TRIMEND",
            "length" | "len" => "LENGTH",
            "contains" => "CONTAINS",
            "startswith" => "STARTSWITH",
            "endswith" => "ENDSWITH",
            "indexof" => "INDEXOF",
            _ => method, // Try the method name as-is
        };

        // Check if we have this function in the registry
        if self.function_registry.get(function_name).is_some() {
            debug!(
                "Proxying method '{}' through function registry as '{}'",
                method, function_name
            );

            // Prepare arguments: receiver is the first argument, followed by method args
            let mut func_args = vec![value.clone()];

            // Evaluate method arguments and add them
            for arg in args {
                func_args.push(self.evaluate(arg, row_index)?);
            }

            // Get the function and call it
            let func = self.function_registry.get(function_name).unwrap();
            return func.evaluate(&func_args);
        }

        // If not in registry, fall back to the old implementation for compatibility
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

                let index = str_val.find(&search_str).map_or(-1, |i| i as i64);

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
        &mut self,
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
        if let Some(else_expr) = else_branch {
            debug!("CASE: No WHEN matched, evaluating ELSE expression");
            self.evaluate(else_expr, row_index)
        } else {
            debug!("CASE: No WHEN matched and no ELSE, returning NULL");
            Ok(DataValue::Null)
        }
    }

    /// Helper method to evaluate an expression as a boolean (for CASE WHEN conditions)
    fn evaluate_condition_as_bool(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<bool> {
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
        let mut evaluator = ArithmeticEvaluator::new(&table);

        let expr = SqlExpression::Column("a".to_string());
        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Integer(10));
    }

    #[test]
    fn test_evaluate_number_literal() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

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
        let mut evaluator = ArithmeticEvaluator::new(&table);

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
        let mut evaluator = ArithmeticEvaluator::new(&table);

        // Integer * Float
        let result = evaluator
            .multiply_values(&DataValue::Integer(4), &DataValue::Float(2.5))
            .unwrap();
        assert_eq!(result, DataValue::Float(10.0));
    }

    #[test]
    fn test_divide_values() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

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
        let mut evaluator = ArithmeticEvaluator::new(&table);

        let result = evaluator.divide_values(&DataValue::Integer(10), &DataValue::Integer(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_binary_op_expression() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

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
