use crate::config::global::get_date_notation;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::value_comparisons::compare_with_op;
use crate::sql::aggregate_functions::AggregateFunctionRegistry; // New registry
use crate::sql::aggregates::AggregateRegistry; // Old registry (for migration)
use crate::sql::functions::FunctionRegistry;
use crate::sql::parser::ast::{ColumnRef, WindowSpec};
use crate::sql::recursive_parser::SqlExpression;
use crate::sql::window_context::WindowContext;
use crate::sql::window_functions::{ExpressionEvaluator, WindowFunctionRegistry};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::debug;

/// Evaluates SQL expressions to compute `DataValues` (for SELECT clauses)
/// This is different from `RecursiveWhereEvaluator` which returns boolean
pub struct ArithmeticEvaluator<'a> {
    table: &'a DataTable,
    _date_notation: String,
    function_registry: Arc<FunctionRegistry>,
    aggregate_registry: Arc<AggregateRegistry>, // Old registry (being phased out)
    new_aggregate_registry: Arc<AggregateFunctionRegistry>, // New registry
    window_function_registry: Arc<WindowFunctionRegistry>,
    visible_rows: Option<Vec<usize>>, // For aggregate functions on filtered views
    window_contexts: HashMap<String, Arc<WindowContext>>, // Cache window contexts by spec
    table_aliases: HashMap<String, String>, // Map alias -> table name for qualified columns
}

impl<'a> ArithmeticEvaluator<'a> {
    #[must_use]
    pub fn new(table: &'a DataTable) -> Self {
        Self {
            table,
            _date_notation: get_date_notation(),
            function_registry: Arc::new(FunctionRegistry::new()),
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            new_aggregate_registry: Arc::new(AggregateFunctionRegistry::new()),
            window_function_registry: Arc::new(WindowFunctionRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
            table_aliases: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_date_notation(table: &'a DataTable, date_notation: String) -> Self {
        Self {
            table,
            _date_notation: date_notation,
            function_registry: Arc::new(FunctionRegistry::new()),
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            new_aggregate_registry: Arc::new(AggregateFunctionRegistry::new()),
            window_function_registry: Arc::new(WindowFunctionRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
            table_aliases: HashMap::new(),
        }
    }

    /// Set visible rows for aggregate functions (for filtered views)
    #[must_use]
    pub fn with_visible_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = Some(rows);
        self
    }

    /// Set table aliases for qualified column resolution
    #[must_use]
    pub fn with_table_aliases(mut self, aliases: HashMap<String, String>) -> Self {
        self.table_aliases = aliases;
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
            _date_notation: date_notation,
            function_registry,
            aggregate_registry: Arc::new(AggregateRegistry::new()),
            new_aggregate_registry: Arc::new(AggregateFunctionRegistry::new()),
            window_function_registry: Arc::new(WindowFunctionRegistry::new()),
            visible_rows: None,
            window_contexts: HashMap::new(),
            table_aliases: HashMap::new(),
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
            SqlExpression::Column(column_ref) => self.evaluate_column_ref(column_ref, row_index),
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
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => self.evaluate_simple_case_expression(expr, when_branches, else_branch, row_index),
            _ => Err(anyhow!(
                "Unsupported expression type for arithmetic evaluation: {:?}",
                expr
            )),
        }
    }

    /// Evaluate a column reference with proper table scoping
    fn evaluate_column_ref(&self, column_ref: &ColumnRef, row_index: usize) -> Result<DataValue> {
        if let Some(table_prefix) = &column_ref.table_prefix {
            // For qualified references, ONLY try qualified lookup - no fallback
            let qualified_name = format!("{}.{}", table_prefix, column_ref.name);

            if let Some(col_idx) = self.table.find_column_by_qualified_name(&qualified_name) {
                return self
                    .table
                    .get_value(row_index, col_idx)
                    .ok_or_else(|| anyhow!("Row {} out of bounds", row_index))
                    .map(|v| v.clone());
            }

            // If not found, return error - don't fall back
            Err(anyhow!("Column '{}' not found", qualified_name))
        } else {
            // Simple column name lookup
            self.evaluate_column(&column_ref.name, row_index)
        }
    }

    /// Evaluate a column reference
    fn evaluate_column(&self, column_name: &str, row_index: usize) -> Result<DataValue> {
        // First try to resolve qualified column names (table.column or alias.column)
        let resolved_column = if column_name.contains('.') {
            // Split on last dot to handle cases like "schema.table.column"
            if let Some(dot_pos) = column_name.rfind('.') {
                let _table_or_alias = &column_name[..dot_pos];
                let col_name = &column_name[dot_pos + 1..];

                // For now, just use the column name part
                // In the future, we could validate the table/alias part
                debug!(
                    "Resolving qualified column: {} -> {}",
                    column_name, col_name
                );
                col_name.to_string()
            } else {
                column_name.to_string()
            }
        } else {
            column_name.to_string()
        };

        let col_index = if let Some(idx) = self.table.get_column_index(&resolved_column) {
            idx
        } else if resolved_column != column_name {
            // If not found, try the original name
            if let Some(idx) = self.table.get_column_index(column_name) {
                idx
            } else {
                let suggestion = self.find_similar_column(&resolved_column);
                return Err(match suggestion {
                    Some(similar) => anyhow!(
                        "Column '{}' not found. Did you mean '{}'?",
                        column_name,
                        similar
                    ),
                    None => anyhow!("Column '{}' not found", column_name),
                });
            }
        } else {
            let suggestion = self.find_similar_column(&resolved_column);
            return Err(match suggestion {
                Some(similar) => anyhow!(
                    "Column '{}' not found. Did you mean '{}'?",
                    column_name,
                    similar
                ),
                None => anyhow!("Column '{}' not found", column_name),
            });
        };

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
            // Use centralized comparison logic for consistency
            ">" | "<" | ">=" | "<=" | "=" | "!=" | "<>" => {
                let result = compare_with_op(&left_val, &right_val, op, false);
                Ok(DataValue::Boolean(result))
            }
            // IS NULL / IS NOT NULL operators
            "IS NULL" => Ok(DataValue::Boolean(matches!(left_val, DataValue::Null))),
            "IS NOT NULL" => Ok(DataValue::Boolean(!matches!(left_val, DataValue::Null))),
            // Logical operators
            "AND" => {
                let left_bool = self.to_bool(&left_val)?;
                let right_bool = self.to_bool(&right_val)?;
                Ok(DataValue::Boolean(left_bool && right_bool))
            }
            "OR" => {
                let left_bool = self.to_bool(&left_val)?;
                let right_bool = self.to_bool(&right_val)?;
                Ok(DataValue::Boolean(left_bool || right_bool))
            }
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

    /// Convert a DataValue to boolean for logical operations
    fn to_bool(&self, value: &DataValue) -> Result<bool> {
        match value {
            DataValue::Boolean(b) => Ok(*b),
            DataValue::Integer(i) => Ok(*i != 0),
            DataValue::Float(f) => Ok(*f != 0.0),
            DataValue::Null => Ok(false),
            _ => Err(anyhow!("Cannot convert {:?} to boolean", value)),
        }
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

            // Check if it's an aggregate function in either registry
            if self.aggregate_registry.is_aggregate(&name_upper)
                || self.new_aggregate_registry.contains(&name_upper)
            {
                return self.evaluate_aggregate_with_distinct(&name_upper, args, row_index);
            } else {
                return Err(anyhow!(
                    "DISTINCT can only be used with aggregate functions"
                ));
            }
        }

        // Otherwise, use the regular evaluation
        self.evaluate_function(name, args, row_index)
    }

    fn evaluate_aggregate_with_distinct(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        _row_index: usize,
    ) -> Result<DataValue> {
        let name_upper = name.to_uppercase();

        // Check new aggregate registry first for migrated functions
        if self.new_aggregate_registry.get(&name_upper).is_some() {
            let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
                visible.clone()
            } else {
                (0..self.table.rows.len()).collect()
            };

            // Collect and deduplicate values for DISTINCT
            let mut vals = Vec::new();
            for &row_idx in &rows_to_process {
                if !args.is_empty() {
                    let value = self.evaluate(&args[0], row_idx)?;
                    vals.push(value);
                }
            }

            // Deduplicate values
            let mut seen = HashSet::new();
            let unique_values: Vec<_> = vals
                .into_iter()
                .filter(|v| {
                    let key = format!("{:?}", v);
                    seen.insert(key)
                })
                .collect();

            // Get the aggregate function from the new registry
            let agg_func = self.new_aggregate_registry.get(&name_upper).unwrap();
            let mut state = agg_func.create_state();

            // Use unique values
            for value in &unique_values {
                state.accumulate(value)?;
            }

            return Ok(state.finalize());
        }

        // Check old aggregate registry (DISTINCT handling)
        if self.aggregate_registry.get(&name_upper).is_some() {
            // Determine which rows to process first
            let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
                visible.clone()
            } else {
                (0..self.table.rows.len()).collect()
            };

            // Special handling for STRING_AGG with separator parameter
            if name_upper == "STRING_AGG" && args.len() >= 2 {
                // STRING_AGG(DISTINCT column, separator)
                let mut state = crate::sql::aggregates::AggregateState::StringAgg(
                    // Evaluate the separator (second argument) once
                    if args.len() >= 2 {
                        let separator = self.evaluate(&args[1], 0)?; // Separator doesn't depend on row
                        match separator {
                            DataValue::String(s) => crate::sql::aggregates::StringAggState::new(&s),
                            DataValue::InternedString(s) => {
                                crate::sql::aggregates::StringAggState::new(&s)
                            }
                            _ => crate::sql::aggregates::StringAggState::new(","), // Default separator
                        }
                    } else {
                        crate::sql::aggregates::StringAggState::new(",")
                    },
                );

                // Evaluate the first argument (column) for each row and accumulate
                // Handle DISTINCT - use a HashSet to track seen values
                let mut seen_values = HashSet::new();

                for &row_idx in &rows_to_process {
                    let value = self.evaluate(&args[0], row_idx)?;

                    // Skip if we've seen this value
                    if !seen_values.insert(value.clone()) {
                        continue; // Skip duplicate values
                    }

                    // Now get the aggregate function and accumulate
                    let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
                    agg_func.accumulate(&mut state, &value)?;
                }

                // Finalize the aggregate
                let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
                return Ok(agg_func.finalize(state));
            }

            // For other aggregates with DISTINCT
            // Evaluate the argument expression for each row
            let mut vals = Vec::new();
            for &row_idx in &rows_to_process {
                if !args.is_empty() {
                    let value = self.evaluate(&args[0], row_idx)?;
                    vals.push(value);
                }
            }

            // Deduplicate values for DISTINCT
            let mut seen = HashSet::new();
            let mut unique_values = Vec::new();
            for value in vals {
                if seen.insert(value.clone()) {
                    unique_values.push(value);
                }
            }

            // Now get the aggregate function and process
            let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
            let mut state = agg_func.init();

            // Use unique values
            for value in &unique_values {
                agg_func.accumulate(&mut state, value)?;
            }

            return Ok(agg_func.finalize(state));
        }

        Err(anyhow!("Unknown aggregate function: {}", name))
    }

    fn evaluate_function(
        &mut self,
        name: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<DataValue> {
        // Check if this is an aggregate function
        let name_upper = name.to_uppercase();

        // Check new aggregate registry first (for migrated functions)
        if self.new_aggregate_registry.get(&name_upper).is_some() {
            // Use new registry for SUM
            let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
                visible.clone()
            } else {
                (0..self.table.rows.len()).collect()
            };

            // Get the aggregate function from the new registry
            let agg_func = self.new_aggregate_registry.get(&name_upper).unwrap();
            let mut state = agg_func.create_state();

            // Special handling for COUNT(*)
            if name_upper == "COUNT" || name_upper == "COUNT_STAR" {
                if args.is_empty()
                    || (args.len() == 1
                        && matches!(&args[0], SqlExpression::Column(col) if col.name == "*"))
                    || (args.len() == 1
                        && matches!(&args[0], SqlExpression::StringLiteral(s) if s == "*"))
                {
                    // COUNT(*) or COUNT_STAR - count all rows
                    for _ in &rows_to_process {
                        state.accumulate(&DataValue::Integer(1))?;
                    }
                } else {
                    // COUNT(column) - count non-null values
                    for &row_idx in &rows_to_process {
                        let value = self.evaluate(&args[0], row_idx)?;
                        state.accumulate(&value)?;
                    }
                }
            } else {
                // Other aggregates - evaluate arguments and accumulate
                if !args.is_empty() {
                    for &row_idx in &rows_to_process {
                        let value = self.evaluate(&args[0], row_idx)?;
                        state.accumulate(&value)?;
                    }
                }
            }

            return Ok(state.finalize());
        }

        // Check old aggregate registry (for non-migrated functions)
        if self.aggregate_registry.get(&name_upper).is_some() {
            // Determine which rows to process first
            let rows_to_process: Vec<usize> = if let Some(ref visible) = self.visible_rows {
                visible.clone()
            } else {
                (0..self.table.rows.len()).collect()
            };

            // Special handling for STRING_AGG with separator parameter
            if name_upper == "STRING_AGG" && args.len() >= 2 {
                // STRING_AGG(column, separator) - without DISTINCT (handled separately)
                let mut state = crate::sql::aggregates::AggregateState::StringAgg(
                    // Evaluate the separator (second argument) once
                    if args.len() >= 2 {
                        let separator = self.evaluate(&args[1], 0)?; // Separator doesn't depend on row
                        match separator {
                            DataValue::String(s) => crate::sql::aggregates::StringAggState::new(&s),
                            DataValue::InternedString(s) => {
                                crate::sql::aggregates::StringAggState::new(&s)
                            }
                            _ => crate::sql::aggregates::StringAggState::new(","), // Default separator
                        }
                    } else {
                        crate::sql::aggregates::StringAggState::new(",")
                    },
                );

                // Evaluate the first argument (column) for each row and accumulate
                for &row_idx in &rows_to_process {
                    let value = self.evaluate(&args[0], row_idx)?;
                    // Now get the aggregate function and accumulate
                    let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
                    agg_func.accumulate(&mut state, &value)?;
                }

                // Finalize the aggregate
                let agg_func = self.aggregate_registry.get(&name_upper).unwrap();
                return Ok(agg_func.finalize(state));
            }

            // Evaluate arguments first if needed (to avoid borrow issues)
            let values = if !args.is_empty()
                && !(args.len() == 1
                    && matches!(&args[0], SqlExpression::Column(c) if c.name == "*"))
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
                // Use evaluated values (DISTINCT is handled in evaluate_aggregate_with_distinct)
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

        // Create the WindowContext with the full spec (including frame)
        let context = WindowContext::new_with_spec(Arc::new(data_view), spec.clone())?;

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
        let name_upper = name.to_uppercase();

        // First check if this is a syntactic sugar function in the registry
        debug!("Looking for window function {} in registry", name_upper);
        if let Some(window_fn_arc) = self.window_function_registry.get(&name_upper) {
            debug!("Found window function {} in registry", name_upper);

            // Dereference to get the actual window function
            let window_fn = window_fn_arc.as_ref();

            // Validate arguments
            window_fn.validate_args(args)?;

            // Transform the window spec based on the function's requirements
            let transformed_spec = window_fn.transform_window_spec(spec, args)?;

            // Get or create the window context with the transformed spec
            let context = self.get_or_create_window_context(&transformed_spec)?;

            // Create an expression evaluator adapter
            struct EvaluatorAdapter<'a, 'b> {
                evaluator: &'a mut ArithmeticEvaluator<'b>,
                row_index: usize,
            }

            impl<'a, 'b> ExpressionEvaluator for EvaluatorAdapter<'a, 'b> {
                fn evaluate(
                    &mut self,
                    expr: &SqlExpression,
                    _row_index: usize,
                ) -> Result<DataValue> {
                    self.evaluator.evaluate(expr, self.row_index)
                }
            }

            let mut adapter = EvaluatorAdapter {
                evaluator: self,
                row_index,
            };

            // Call the window function's compute method
            return window_fn.compute(&context, row_index, args, &mut adapter);
        }

        // Fall back to built-in window functions
        let context = self.get_or_create_window_context(spec)?;

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
                    .get_offset_value(row_index, -offset, &column.name)
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
                    .get_offset_value(row_index, offset, &column.name)
                    .unwrap_or(DataValue::Null))
            }
            "ROW_NUMBER" => {
                // ROW_NUMBER() - no arguments
                Ok(DataValue::Integer(context.get_row_number(row_index) as i64))
            }
            "FIRST_VALUE" => {
                // FIRST_VALUE(column) OVER (... ROWS ...)
                if args.is_empty() {
                    return Err(anyhow!("FIRST_VALUE requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("FIRST_VALUE argument must be a column")),
                };

                // Use frame-aware version if frame is specified
                if context.has_frame() {
                    Ok(context
                        .get_frame_first_value(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                } else {
                    Ok(context
                        .get_first_value(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                }
            }
            "LAST_VALUE" => {
                // LAST_VALUE(column) OVER (... ROWS ...)
                if args.is_empty() {
                    return Err(anyhow!("LAST_VALUE requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("LAST_VALUE argument must be a column")),
                };

                // Use frame-aware version if frame is specified
                if context.has_frame() {
                    Ok(context
                        .get_frame_last_value(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                } else {
                    Ok(context
                        .get_last_value(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                }
            }
            "SUM" => {
                // SUM(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("SUM requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("SUM argument must be a column")),
                };

                // Use frame-aware sum if frame is specified, otherwise use partition sum
                if context.has_frame() {
                    Ok(context
                        .get_frame_sum(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                } else {
                    Ok(context
                        .get_partition_sum(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                }
            }
            "AVG" => {
                // AVG(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("AVG requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("AVG argument must be a column")),
                };

                Ok(context
                    .get_frame_avg(row_index, &column.name)
                    .unwrap_or(DataValue::Null))
            }
            "STDDEV" | "STDEV" => {
                // STDDEV(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("STDDEV requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("STDDEV argument must be a column")),
                };

                Ok(context
                    .get_frame_stddev(row_index, &column.name)
                    .unwrap_or(DataValue::Null))
            }
            "VARIANCE" | "VAR" => {
                // VARIANCE(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("VARIANCE requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("VARIANCE argument must be a column")),
                };

                Ok(context
                    .get_frame_variance(row_index, &column.name)
                    .unwrap_or(DataValue::Null))
            }
            "MIN" => {
                // MIN(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("MIN requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("MIN argument must be a column")),
                };

                let frame_rows = context.get_frame_rows(row_index);
                if frame_rows.is_empty() {
                    return Ok(DataValue::Null);
                }

                let source_table = context.source();
                let col_idx = source_table
                    .get_column_index(&column.name)
                    .ok_or_else(|| anyhow!("Column '{}' not found", column.name))?;

                let mut min_value: Option<DataValue> = None;
                for &row_idx in &frame_rows {
                    if let Some(value) = source_table.get_value(row_idx, col_idx) {
                        if !matches!(value, DataValue::Null) {
                            match &min_value {
                                None => min_value = Some(value.clone()),
                                Some(current_min) => {
                                    if value < current_min {
                                        min_value = Some(value.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(min_value.unwrap_or(DataValue::Null))
            }
            "MAX" => {
                // MAX(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                if args.is_empty() {
                    return Err(anyhow!("MAX requires 1 argument"));
                }

                let column = match &args[0] {
                    SqlExpression::Column(col) => col.clone(),
                    _ => return Err(anyhow!("MAX argument must be a column")),
                };

                let frame_rows = context.get_frame_rows(row_index);
                if frame_rows.is_empty() {
                    return Ok(DataValue::Null);
                }

                let source_table = context.source();
                let col_idx = source_table
                    .get_column_index(&column.name)
                    .ok_or_else(|| anyhow!("Column '{}' not found", column.name))?;

                let mut max_value: Option<DataValue> = None;
                for &row_idx in &frame_rows {
                    if let Some(value) = source_table.get_value(row_idx, col_idx) {
                        if !matches!(value, DataValue::Null) {
                            match &max_value {
                                None => max_value = Some(value.clone()),
                                Some(current_max) => {
                                    if value > current_max {
                                        max_value = Some(value.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(max_value.unwrap_or(DataValue::Null))
            }
            "COUNT" => {
                // COUNT(*) or COUNT(column) OVER (PARTITION BY ... ROWS n PRECEDING)
                // Use frame-aware count if frame is specified, otherwise use partition count

                if args.is_empty() {
                    // COUNT(*) OVER (...)
                    if context.has_frame() {
                        Ok(context
                            .get_frame_count(row_index, None)
                            .unwrap_or(DataValue::Null))
                    } else {
                        Ok(context
                            .get_partition_count(row_index, None)
                            .unwrap_or(DataValue::Null))
                    }
                } else {
                    // Check for COUNT(*)
                    let column = match &args[0] {
                        SqlExpression::Column(col) => {
                            if col.name == "*" {
                                // COUNT(*) - count all rows
                                if context.has_frame() {
                                    return Ok(context
                                        .get_frame_count(row_index, None)
                                        .unwrap_or(DataValue::Null));
                                } else {
                                    return Ok(context
                                        .get_partition_count(row_index, None)
                                        .unwrap_or(DataValue::Null));
                                }
                            }
                            col.clone()
                        }
                        SqlExpression::StringLiteral(s) if s == "*" => {
                            // COUNT(*) as StringLiteral
                            if context.has_frame() {
                                return Ok(context
                                    .get_frame_count(row_index, None)
                                    .unwrap_or(DataValue::Null));
                            } else {
                                return Ok(context
                                    .get_partition_count(row_index, None)
                                    .unwrap_or(DataValue::Null));
                            }
                        }
                        _ => return Err(anyhow!("COUNT argument must be a column or *")),
                    };

                    // COUNT(column) - count non-null values
                    if context.has_frame() {
                        Ok(context
                            .get_frame_count(row_index, Some(&column.name))
                            .unwrap_or(DataValue::Null))
                    } else {
                        Ok(context
                            .get_partition_count(row_index, Some(&column.name))
                            .unwrap_or(DataValue::Null))
                    }
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

        // If not in registry, the method is not supported
        // All methods should be registered in the function registry
        Err(anyhow!(
            "Method '{}' not found. It should be registered in the function registry.",
            method
        ))
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

    /// Evaluate a simple CASE expression
    fn evaluate_simple_case_expression(
        &mut self,
        expr: &Box<SqlExpression>,
        when_branches: &[crate::sql::parser::ast::SimpleWhenBranch],
        else_branch: &Option<Box<SqlExpression>>,
        row_index: usize,
    ) -> Result<DataValue> {
        debug!(
            "ArithmeticEvaluator: evaluating simple CASE expression for row {}",
            row_index
        );

        // Evaluate the main expression once
        let case_value = self.evaluate(expr, row_index)?;
        debug!("Simple CASE: evaluated expression to {:?}", case_value);

        // Compare against each WHEN value in order
        for branch in when_branches {
            // Evaluate the WHEN value
            let when_value = self.evaluate(&branch.value, row_index)?;

            // Check for equality
            if self.values_equal(&case_value, &when_value)? {
                debug!("Simple CASE: WHEN value matched, evaluating result expression");
                return self.evaluate(&branch.result, row_index);
            }
        }

        // If no WHEN value matched, evaluate ELSE clause (or return NULL)
        if let Some(else_expr) = else_branch {
            debug!("Simple CASE: No WHEN matched, evaluating ELSE expression");
            self.evaluate(else_expr, row_index)
        } else {
            debug!("Simple CASE: No WHEN matched and no ELSE, returning NULL");
            Ok(DataValue::Null)
        }
    }

    /// Check if two DataValues are equal
    fn values_equal(&self, left: &DataValue, right: &DataValue) -> Result<bool> {
        match (left, right) {
            (DataValue::Null, DataValue::Null) => Ok(true),
            (DataValue::Null, _) | (_, DataValue::Null) => Ok(false),
            (DataValue::Integer(a), DataValue::Integer(b)) => Ok(a == b),
            (DataValue::Float(a), DataValue::Float(b)) => Ok((a - b).abs() < f64::EPSILON),
            (DataValue::String(a), DataValue::String(b)) => Ok(a == b),
            (DataValue::Boolean(a), DataValue::Boolean(b)) => Ok(a == b),
            (DataValue::DateTime(a), DataValue::DateTime(b)) => Ok(a == b),
            // Type coercion for numeric comparisons
            (DataValue::Integer(a), DataValue::Float(b)) => {
                Ok((*a as f64 - b).abs() < f64::EPSILON)
            }
            (DataValue::Float(a), DataValue::Integer(b)) => {
                Ok((a - *b as f64).abs() < f64::EPSILON)
            }
            _ => Ok(false),
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

        let expr = SqlExpression::Column(ColumnRef::unquoted("a".to_string()));
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
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
            op: "*".to_string(),
            right: Box::new(SqlExpression::Column(ColumnRef::unquoted("b".to_string()))),
        };

        let result = evaluator.evaluate(&expr, 0).unwrap();
        assert_eq!(result, DataValue::Float(25.0));
    }
}
