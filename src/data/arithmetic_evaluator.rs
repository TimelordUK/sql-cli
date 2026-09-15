use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::trilean::Trilean;
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
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{debug, info};

/// The registries every evaluator reads. None of them change after
/// construction, so the process builds them once and every evaluator shares
/// them, rather than each construction site rebuilding all four (R13 slice 3).
struct EvaluatorRegistries {
    function: Arc<FunctionRegistry>,
    aggregate: Arc<AggregateRegistry>,
    new_aggregate: Arc<AggregateFunctionRegistry>,
    window: Arc<WindowFunctionRegistry>,
}

fn shared_registries() -> &'static EvaluatorRegistries {
    static REGISTRIES: OnceLock<EvaluatorRegistries> = OnceLock::new();
    REGISTRIES.get_or_init(|| EvaluatorRegistries {
        function: Arc::new(FunctionRegistry::new()),
        aggregate: Arc::new(AggregateRegistry::new()),
        new_aggregate: Arc::new(AggregateFunctionRegistry::new()),
        window: Arc::new(WindowFunctionRegistry::new()),
    })
}

/// Evaluates SQL expressions to compute `DataValues` (for SELECT clauses)
/// This is different from `RecursiveWhereEvaluator` which returns boolean
pub struct ArithmeticEvaluator<'a> {
    table: &'a DataTable,
    function_registry: Arc<FunctionRegistry>,
    aggregate_registry: Arc<AggregateRegistry>, // Old registry (being phased out)
    new_aggregate_registry: Arc<AggregateFunctionRegistry>, // New registry
    window_function_registry: Arc<WindowFunctionRegistry>,
    visible_rows: Option<Vec<usize>>, // For aggregate functions on filtered views
    window_contexts: HashMap<u64, Arc<WindowContext>>, // Cache window contexts by hash
    table_aliases: HashMap<String, String>, // Map alias -> table name for qualified columns
    case_insensitive: bool,           // The engine's string-comparison mode, as WHERE honours it
}

impl<'a> ArithmeticEvaluator<'a> {
    #[must_use]
    pub fn new(table: &'a DataTable) -> Self {
        let registries = shared_registries();
        Self {
            table,
            function_registry: Arc::clone(&registries.function),
            aggregate_registry: Arc::clone(&registries.aggregate),
            new_aggregate_registry: Arc::clone(&registries.new_aggregate),
            window_function_registry: Arc::clone(&registries.window),
            visible_rows: None,
            window_contexts: HashMap::new(),
            table_aliases: HashMap::new(),
            case_insensitive: false,
        }
    }

    /// Set visible rows for aggregate functions (for filtered views)
    #[must_use]
    pub fn with_visible_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = Some(rows);
        self
    }

    /// Compare strings case-insensitively, as the engine's case-insensitive
    /// mode does for WHERE: comparisons, IN, BETWEEN and LIKE.
    #[must_use]
    pub fn with_case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// Set table aliases for qualified column resolution
    #[must_use]
    pub fn with_table_aliases(mut self, aliases: HashMap<String, String>) -> Self {
        self.table_aliases = aliases;
        self
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
    ///
    /// No logging on this path: it runs once per expression node per row, and
    /// non-interactive mode records TRACE to a file, so a `debug!` formatting
    /// the expression here cost more than evaluating it (2.5 s of a 5 s
    /// `WHERE price + 0 > quantity` over 100k rows).
    pub fn evaluate(&mut self, expr: &SqlExpression, row_index: usize) -> Result<DataValue> {
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
            // BETWEEN is `value >= lower AND value <= upper`, so it takes AND's
            // truth table: FALSE on either side wins even over UNKNOWN.
            SqlExpression::Between { expr, lower, upper } => {
                let val = self.evaluate(expr, row_index)?;
                let lo = self.evaluate(lower, row_index)?;
                let hi = self.evaluate(upper, row_index)?;
                let ge = compare_trilean(&val, &lo, ">=", self.case_insensitive);
                let le = compare_trilean(&val, &hi, "<=", self.case_insensitive);
                Ok(ge.and(le).to_value())
            }
            // Logical negation as a value-producing expression. Reached, e.g., by
            // a post-aggregation `HAVING NOT (COUNT(*) > 2)` predicate (P10).
            // NOT UNKNOWN is UNKNOWN - which only holds if the operand arrives
            // as NULL, hence the three-valued comparison arms above.
            SqlExpression::Not { expr } => {
                let inner = self.evaluate(expr, row_index)?;
                Ok(truth_of(&inner)?.negate().to_value())
            }
            // IN / NOT IN as a value-producing expression — used by SELECT,
            // HAVING and CASE. After subquery rewriting, an `x IN (SELECT ...)`
            // arrives here as an InList of literals.
            SqlExpression::InList { expr, values } => {
                Ok(self.evaluate_in_list(expr, values, row_index)?.to_value())
            }
            SqlExpression::NotInList { expr, values } => Ok(self
                .evaluate_in_list(expr, values, row_index)?
                .negate()
                .to_value()),
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => self.evaluate_case_expression(when_branches, else_branch, row_index),
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => self.evaluate_simple_case_expression(expr, when_branches, else_branch, row_index),
            SqlExpression::DateTimeConstructor {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => self.evaluate_datetime_constructor(*year, *month, *day, *hour, *minute, *second),
            SqlExpression::DateTimeToday {
                hour,
                minute,
                second,
            } => self.evaluate_datetime_today(*hour, *minute, *second),
            _ => Err(anyhow!(
                "Unsupported expression type for arithmetic evaluation: {:?}",
                expr
            )),
        }
    }

    /// Evaluate a column reference with proper table scoping
    fn evaluate_column_ref(&self, column_ref: &ColumnRef, row_index: usize) -> Result<DataValue> {
        if let Some(table_prefix) = &column_ref.table_prefix {
            // Resolve alias if it exists in table_aliases map
            let actual_table = self
                .table_aliases
                .get(table_prefix)
                .map(|s| s.as_str())
                .unwrap_or(table_prefix);

            // Try qualified lookup with resolved table name
            let qualified_name = format!("{}.{}", actual_table, column_ref.name);

            if let Some(col_idx) = self.table.find_column_by_qualified_name(&qualified_name) {
                return self
                    .table
                    .get_value(row_index, col_idx)
                    .ok_or_else(|| anyhow!("Row {} out of bounds", row_index))
                    .map(|v| v.clone());
            }

            // Fallback: try unqualified lookup
            if let Some(col_idx) = self.table.get_column_index(&column_ref.name) {
                return self
                    .table
                    .get_value(row_index, col_idx)
                    .ok_or_else(|| anyhow!("Row {} out of bounds", row_index))
                    .map(|v| v.clone());
            }

            // If not found, say which of the two failures this is.
            Err(
                crate::data::column_resolution_error::qualified_column_not_found(
                    self.table,
                    table_prefix,
                    actual_table,
                    &column_ref.name,
                ),
            )
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
            // Predicates are three-valued: UNKNOWN is returned as NULL, and only
            // the clause that filters on the result collapses it (R13). See
            // tests/evaluator_matrix_tests.rs for the per-operator contract.
            ">" | "<" | ">=" | "<=" | "=" | "!=" | "<>" => {
                Ok(compare_trilean(&left_val, &right_val, op, self.case_insensitive).to_value())
            }
            // IS NULL / IS NOT NULL are the sanctioned NULL tests, so two-valued.
            "IS NULL" => Ok(DataValue::Boolean(matches!(left_val, DataValue::Null))),
            "IS NOT NULL" => Ok(DataValue::Boolean(!matches!(left_val, DataValue::Null))),
            "AND" => Ok(truth_of(&left_val)?.and(truth_of(&right_val)?).to_value()),
            "OR" => Ok(truth_of(&left_val)?.or(truth_of(&right_val)?).to_value()),
            // LIKE operator - SQL pattern matching
            "LIKE" => {
                if matches!(left_val, DataValue::Null) || matches!(right_val, DataValue::Null) {
                    return Ok(DataValue::Null);
                }
                let mut text = self.value_to_string(&left_val);
                let mut pattern = self.value_to_string(&right_val);
                if self.case_insensitive {
                    text = text.to_lowercase();
                    pattern = pattern.to_lowercase();
                }
                let matches = self.sql_like_match(&text, &pattern);
                Ok(DataValue::Boolean(matches))
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

    /// Convert DataValue to string for pattern matching
    fn value_to_string(&self, value: &DataValue) -> String {
        match value {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Vector(v) => {
                // Format as "[x,y,z]"
                let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                format!("[{}]", components.join(","))
            }
            DataValue::Null => String::new(),
        }
    }

    /// SQL LIKE pattern matching
    /// Supports % (any chars) and _ (single char)
    fn sql_like_match(&self, text: &str, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        self.like_match_recursive(&text_chars, 0, &pattern_chars, 0)
    }

    /// Recursive helper for LIKE matching
    fn like_match_recursive(
        &self,
        text: &[char],
        text_pos: usize,
        pattern: &[char],
        pattern_pos: usize,
    ) -> bool {
        // If we've consumed both text and pattern, it's a match
        if pattern_pos >= pattern.len() {
            return text_pos >= text.len();
        }

        // Handle % wildcard (matches zero or more characters)
        if pattern[pattern_pos] == '%' {
            // Try matching zero characters (skip the %)
            if self.like_match_recursive(text, text_pos, pattern, pattern_pos + 1) {
                return true;
            }
            // Try matching one or more characters
            if text_pos < text.len() {
                return self.like_match_recursive(text, text_pos + 1, pattern, pattern_pos);
            }
            return false;
        }

        // If text is consumed but pattern isn't, no match
        if text_pos >= text.len() {
            return false;
        }

        // Handle _ wildcard (matches exactly one character)
        if pattern[pattern_pos] == '_' {
            return self.like_match_recursive(text, text_pos + 1, pattern, pattern_pos + 1);
        }

        // Handle literal character match
        if text[text_pos] == pattern[pattern_pos] {
            return self.like_match_recursive(text, text_pos + 1, pattern, pattern_pos + 1);
        }

        false
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
    /// Public to allow pre-creation of contexts in query engine (optimization)
    pub fn get_or_create_window_context(
        &mut self,
        spec: &WindowSpec,
    ) -> Result<Arc<WindowContext>> {
        let overall_start = Instant::now();

        // Create a hash-based key for fast caching (much faster than format!("{:?}", spec))
        let key = spec.compute_hash();

        if let Some(context) = self.window_contexts.get(&key) {
            info!(
                "WindowContext cache hit for spec (lookup: {:.2}μs)",
                overall_start.elapsed().as_micros()
            );
            return Ok(Arc::clone(context));
        }

        info!("WindowContext cache miss - creating new context");
        let dataview_start = Instant::now();

        // Create a DataView from the table, restricted to the visible rows when the
        // query filtered. Window functions must partition over the post-WHERE row set:
        // SQL evaluates them after FROM/WHERE/GROUP BY/HAVING, so a filtered-out row
        // must not appear in a partition, occupy a ROW_NUMBER slot, or be counted.
        //
        // The indices here are source-table indices, which is what DataView::with_rows
        // expects and what WindowContext reads back via get_visible_rows() - so the
        // whole path stays in one index space.
        let data_view = if let Some(ref visible_rows) = self.visible_rows {
            DataView::new(Arc::new(self.table.clone())).with_rows(visible_rows.clone())
        } else {
            DataView::new(Arc::new(self.table.clone()))
        };

        info!(
            "DataView creation took {:.2}μs",
            dataview_start.elapsed().as_micros()
        );
        let context_start = Instant::now();

        // Create the WindowContext with the full spec (including frame)
        let context = WindowContext::new_with_spec(Arc::new(data_view), spec.clone())?;

        info!(
            "WindowContext::new_with_spec took {:.2}ms (rows: {})",
            context_start.elapsed().as_secs_f64() * 1000.0,
            self.table.row_count()
        );

        let context = Arc::new(context);
        self.window_contexts.insert(key, Arc::clone(&context));

        info!(
            "Total WindowContext creation (cache miss) took {:.2}ms",
            overall_start.elapsed().as_secs_f64() * 1000.0
        );

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
        let func_start = Instant::now();
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
                    row_index: usize,
                ) -> Result<DataValue> {
                    self.evaluator.evaluate(expr, row_index)
                }
            }

            let mut adapter = EvaluatorAdapter {
                evaluator: self,
                row_index,
            };

            let compute_start = Instant::now();
            // Call the window function's compute method
            let result = window_fn.compute(&context, row_index, args, &mut adapter);

            info!(
                "{} (registry) evaluation: total={:.2}μs, compute={:.2}μs",
                name_upper,
                func_start.elapsed().as_micros(),
                compute_start.elapsed().as_micros()
            );

            return result;
        }

        // Fall back to built-in window functions
        let context_start = Instant::now();
        let context = self.get_or_create_window_context(spec)?;
        let context_time = context_start.elapsed();

        let eval_start = Instant::now();

        let result = match name_upper.as_str() {
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

                let offset_start = Instant::now();
                // Get value at offset
                let value = context
                    .get_offset_value(row_index, -offset, &column.name)
                    .unwrap_or(DataValue::Null);

                debug!(
                    "LAG offset access took {:.2}μs (offset={})",
                    offset_start.elapsed().as_micros(),
                    offset
                );

                Ok(value)
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

                let offset_start = Instant::now();
                // Get value at offset
                let value = context
                    .get_offset_value(row_index, offset, &column.name)
                    .unwrap_or(DataValue::Null);

                debug!(
                    "LEAD offset access took {:.2}μs (offset={})",
                    offset_start.elapsed().as_micros(),
                    offset
                );

                Ok(value)
            }
            "ROW_NUMBER" => {
                // ROW_NUMBER() - no arguments
                Ok(DataValue::Integer(context.get_row_number(row_index) as i64))
            }
            "RANK" => {
                // RANK() - no arguments
                Ok(DataValue::Integer(context.get_rank(row_index)))
            }
            "DENSE_RANK" => {
                // DENSE_RANK() - no arguments
                Ok(DataValue::Integer(context.get_dense_rank(row_index)))
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

                // Use frame-aware avg if frame is specified, otherwise use partition avg
                if context.has_frame() {
                    Ok(context
                        .get_frame_avg(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                } else {
                    Ok(context
                        .get_partition_avg(row_index, &column.name)
                        .unwrap_or(DataValue::Null))
                }
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
        };

        let eval_time = eval_start.elapsed();

        info!(
            "{} (built-in) evaluation: total={:.2}μs, context={:.2}μs, eval={:.2}μs",
            name_upper,
            func_start.elapsed().as_micros(),
            context_time.as_micros(),
            eval_time.as_micros()
        );

        result
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
        // Method-call syntax (`x.Method(...)`) dispatches through the method
        // registry first, so a function can give its C#-style method form
        // different semantics from its SQL function form (e.g. SUBSTRING is
        // 1-based as a function but `.Substring()` is 0-based like .NET).
        // The default `evaluate_method` just prepends the receiver and calls
        // `evaluate`, so this is behavior-preserving for every other method.
        if let Some(method_fn) = self.function_registry.get_method(method) {
            let mut method_args = Vec::with_capacity(args.len());
            for arg in args {
                method_args.push(self.evaluate(arg, row_index)?);
            }
            return method_fn.evaluate_method(value, &method_args);
        }

        // Otherwise, proxy the method through the function registry.
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
        // Evaluate each WHEN condition in order
        for branch in when_branches {
            // Evaluate the condition as a boolean
            let condition_result = self.evaluate_condition_as_bool(&branch.condition, row_index)?;

            if condition_result {
                return self.evaluate(&branch.result, row_index);
            }
        }

        // If no WHEN condition matched, evaluate ELSE clause (or return NULL)
        if let Some(else_expr) = else_branch {
            self.evaluate(else_expr, row_index)
        } else {
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
        // Evaluate the main expression once
        let case_value = self.evaluate(expr, row_index)?;

        // Compare against each WHEN value in order
        for branch in when_branches {
            // Evaluate the WHEN value
            let when_value = self.evaluate(&branch.value, row_index)?;

            // Check for equality
            if self.values_equal(&case_value, &when_value)? {
                return self.evaluate(&branch.result, row_index);
            }
        }

        // If no WHEN value matched, evaluate ELSE clause (or return NULL)
        if let Some(else_expr) = else_branch {
            self.evaluate(else_expr, row_index)
        } else {
            Ok(DataValue::Null)
        }
    }

    /// `x IN (a, b, ...)` is `x = a OR x = b OR ...`, so it takes OR's truth
    /// table: a match anywhere is TRUE, otherwise any UNKNOWN comparison (a NULL
    /// probe, or a NULL in the list) makes the answer UNKNOWN rather than FALSE.
    /// `NOT IN` is the negation, which is what keeps NULL rows out of it.
    fn evaluate_in_list(
        &mut self,
        expr: &SqlExpression,
        values: &[SqlExpression],
        row_index: usize,
    ) -> Result<Trilean> {
        let val = self.evaluate(expr, row_index)?;
        let mut result = Trilean::False;
        for v in values {
            let item = self.evaluate(v, row_index)?;
            result = result.or(compare_trilean(&val, &item, "=", self.case_insensitive));
            if result.is_true() {
                break;
            }
        }
        Ok(result)
    }

    /// Check if two DataValues are equal, for simple `CASE x WHEN v`. The WHEN
    /// test is `x = v`, so a NULL on either side is not a match - including
    /// `CASE NULL WHEN NULL`.
    fn values_equal(&self, left: &DataValue, right: &DataValue) -> Result<bool> {
        match (left, right) {
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

    /// Evaluate a DATETIME constructor expression
    fn evaluate_datetime_constructor(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: Option<u32>,
        minute: Option<u32>,
        second: Option<u32>,
    ) -> Result<DataValue> {
        use chrono::{NaiveDate, TimeZone, Utc};

        // Create a NaiveDate
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow!("Invalid date: {}-{}-{}", year, month, day))?;

        // Create datetime with provided time components or defaults
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);

        let naive_datetime = date
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| anyhow!("Invalid time: {}:{}:{}", hour, minute, second))?;

        // Convert to UTC DateTime
        let datetime = Utc.from_utc_datetime(&naive_datetime);

        // Format as string with milliseconds
        let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        Ok(DataValue::String(datetime_str))
    }

    /// Evaluate a DATETIME.TODAY constructor expression
    fn evaluate_datetime_today(
        &self,
        hour: Option<u32>,
        minute: Option<u32>,
        second: Option<u32>,
    ) -> Result<DataValue> {
        use chrono::{TimeZone, Utc};

        // Get today's date in UTC
        let today = Utc::now().date_naive();

        // Create datetime with provided time components or defaults
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);

        let naive_datetime = today
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| anyhow!("Invalid time: {}:{}:{}", hour, minute, second))?;

        // Convert to UTC DateTime
        let datetime = Utc.from_utc_datetime(&naive_datetime);

        // Format as string with milliseconds
        let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        Ok(DataValue::String(datetime_str))
    }
}

/// A comparison under SQL three-valued logic: NULL on either side is UNKNOWN.
///
/// The NULL test belongs here, at the predicate layer, and must not move into
/// `compare_with_op`: the `compare_values` beneath it reports `NULL = NULL` as
/// equal on purpose, because ORDER BY needs NULLs to group. Reusing that answer
/// for predicates is what made `SELECT x = NULL` TRUE for NULL rows (P48, the
/// value-evaluator twin of P18). Same rule as the WHERE evaluator's
/// `compare_trilean`, which R13 slice 4 will make the only copy.
fn compare_trilean(
    left: &DataValue,
    right: &DataValue,
    op: &str,
    case_insensitive: bool,
) -> Trilean {
    if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
        return Trilean::Unknown;
    }
    Trilean::from_bool(compare_with_op(left, right, op, case_insensitive))
}

/// The truth value of an operand of AND / OR / NOT. A value with no truth value
/// (a string, a date) is an error, as it was before three-valued logic.
fn truth_of(value: &DataValue) -> Result<Trilean> {
    Trilean::from_value(value).ok_or_else(|| anyhow!("Cannot convert {:?} to boolean", value))
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
    fn test_evaluate_between_column_in_range() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

        // column 'a' is 10 — 5 <= 10 <= 20 is true
        let expr = SqlExpression::Between {
            expr: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
            lower: Box::new(SqlExpression::NumberLiteral("5".to_string())),
            upper: Box::new(SqlExpression::NumberLiteral("20".to_string())),
        };
        assert_eq!(
            evaluator.evaluate(&expr, 0).unwrap(),
            DataValue::Boolean(true)
        );
    }

    #[test]
    fn test_evaluate_between_column_out_of_range() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

        // column 'a' is 10 — 11 <= 10 <= 20 is false
        let expr = SqlExpression::Between {
            expr: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
            lower: Box::new(SqlExpression::NumberLiteral("11".to_string())),
            upper: Box::new(SqlExpression::NumberLiteral("20".to_string())),
        };
        assert_eq!(
            evaluator.evaluate(&expr, 0).unwrap(),
            DataValue::Boolean(false)
        );
    }

    #[test]
    fn test_evaluate_between_endpoints_inclusive() {
        let table = create_test_table();
        let mut evaluator = ArithmeticEvaluator::new(&table);

        // column 'a' is 10 — 10 <= 10 <= 10 is true (both endpoints inclusive)
        let expr = SqlExpression::Between {
            expr: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
            lower: Box::new(SqlExpression::NumberLiteral("10".to_string())),
            upper: Box::new(SqlExpression::NumberLiteral("10".to_string())),
        };
        assert_eq!(
            evaluator.evaluate(&expr, 0).unwrap(),
            DataValue::Boolean(true)
        );
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
