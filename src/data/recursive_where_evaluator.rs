use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::evaluation_context::EvaluationContext;
use crate::data::query_engine::ExecutionContext;
use crate::data::trilean::Trilean;
use crate::data::value_comparisons::compare_with_op;
use crate::sql::recursive_parser::{Condition, LogicalOp, SqlExpression, WhereClause};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use tracing::debug;

/// Evaluates WHERE clauses from `recursive_parser` directly against `DataTable`
pub struct RecursiveWhereEvaluator<'a, 'ctx, 'exec> {
    table: &'a DataTable,
    case_insensitive: bool,
    context: Option<&'ctx mut EvaluationContext>,
    exec_context: Option<&'exec ExecutionContext>,
}

impl<'a, 'ctx, 'exec> RecursiveWhereEvaluator<'a, 'ctx, 'exec> {
    #[must_use]
    pub fn new(table: &'a DataTable) -> RecursiveWhereEvaluator<'a, 'static, 'static> {
        RecursiveWhereEvaluator {
            table,
            case_insensitive: false,
            context: None,
            exec_context: None,
        }
    }

    /// Create evaluator with an evaluation context for caching
    pub fn with_context(table: &'a DataTable, context: &'ctx mut EvaluationContext) -> Self {
        let case_insensitive = context.is_case_insensitive();
        Self {
            table,
            case_insensitive,
            context: Some(context),
            exec_context: None,
        }
    }

    /// Create evaluator with execution context for alias resolution
    pub fn with_exec_context(
        table: &'a DataTable,
        exec_context: &'exec ExecutionContext,
        case_insensitive: bool,
    ) -> Self {
        Self {
            table,
            case_insensitive,
            context: None,
            exec_context: Some(exec_context),
        }
    }

    /// Create evaluator with both execution context (for alias resolution) and evaluation context (for regex caching)
    pub fn with_both_contexts(
        table: &'a DataTable,
        context: &'ctx mut EvaluationContext,
        exec_context: &'exec ExecutionContext,
    ) -> Self {
        let case_insensitive = context.is_case_insensitive();
        Self {
            table,
            case_insensitive,
            context: Some(context),
            exec_context: Some(exec_context),
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
                let cost = usize::from(c1 != c2);
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

    #[must_use]
    pub fn with_case_insensitive(
        table: &'a DataTable,
        case_insensitive: bool,
    ) -> RecursiveWhereEvaluator<'a, 'static, 'static> {
        RecursiveWhereEvaluator {
            table,
            case_insensitive,
            context: None,
            exec_context: None,
        }
    }

    #[must_use]
    pub fn with_config(
        table: &'a DataTable,
        case_insensitive: bool,
        _date_notation: String, // No longer needed since we use centralized parse_datetime
    ) -> RecursiveWhereEvaluator<'a, 'static, 'static> {
        RecursiveWhereEvaluator {
            table,
            case_insensitive,
            context: None,
            exec_context: None,
        }
    }

    /// Compare two values under SQL three-valued logic: if **either** operand
    /// is NULL the answer is UNKNOWN, never TRUE or FALSE.
    ///
    /// This test has to live here, at the predicate layer, and must not be
    /// pushed down into `compare_with_op`. That function answers in `bool`, and
    /// the `compare_values` beneath it deliberately reports `NULL = NULL` as
    /// `Ordering::Equal` because **`ORDER BY` needs NULLs to group together**.
    /// Correct for sorting, wrong for predicates — and using the one answer for
    /// both is what made `WHERE score = NULL` return the NULL rows (P18).
    ///
    /// `IS NULL` / `IS NOT NULL` / `IS [NOT] NULL` are handled by their own
    /// match arms before reaching any caller of this, and stay two-valued —
    /// they are the sanctioned way to test for NULL, which is why nothing is
    /// lost by making `=` never match one.
    fn compare_trilean(&self, left: &DataValue, right: &DataValue, op: &str) -> Trilean {
        if matches!(left, DataValue::Null) || matches!(right, DataValue::Null) {
            return Trilean::Unknown;
        }
        Trilean::from_bool(compare_with_op(left, right, op, self.case_insensitive))
    }

    /// Convert ExprValue to DataValue for centralized comparison
    fn expr_value_to_data_value(&self, expr_value: &ExprValue) -> DataValue {
        match expr_value {
            ExprValue::String(s) => DataValue::String(s.clone()),
            ExprValue::Number(n) => {
                // Check if it's an integer or float
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    DataValue::Integer(*n as i64)
                } else {
                    DataValue::Float(*n)
                }
            }
            ExprValue::Boolean(b) => DataValue::Boolean(*b),
            ExprValue::DateTime(dt) => {
                DataValue::DateTime(dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
            }
            ExprValue::Null => DataValue::Null,
        }
    }

    /// Evaluate the `Length()` method on a column value
    fn evaluate_length(
        &self,
        object: &str,
        row_index: usize,
    ) -> Result<(Option<DataValue>, String)> {
        // Handle qualified column names (table.column or alias.column)
        let resolved_column = if object.contains('.') {
            if let Some(dot_pos) = object.rfind('.') {
                let col_name = &object[dot_pos + 1..];
                col_name
            } else {
                object
            }
        } else {
            object
        };

        let col_index = if let Some(idx) = self.table.get_column_index(resolved_column) {
            idx
        } else if resolved_column != object {
            // If not found, try the original name
            if let Some(idx) = self.table.get_column_index(object) {
                idx
            } else {
                let suggestion = self.find_similar_column(resolved_column);
                return Err(match suggestion {
                    Some(similar) => {
                        anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                    }
                    None => anyhow!("Column '{}' not found", object),
                });
            }
        } else {
            let suggestion = self.find_similar_column(resolved_column);
            return Err(match suggestion {
                Some(similar) => {
                    anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                }
                None => anyhow!("Column '{}' not found", object),
            });
        };

        let value = self.table.get_value(row_index, col_index);
        let length_value = match value {
            Some(DataValue::String(s)) => Some(DataValue::Integer(s.len() as i64)),
            Some(DataValue::InternedString(s)) => Some(DataValue::Integer(s.len() as i64)),
            Some(DataValue::Integer(n)) => Some(DataValue::Integer(n.to_string().len() as i64)),
            Some(DataValue::Float(f)) => Some(DataValue::Integer(f.to_string().len() as i64)),
            _ => Some(DataValue::Integer(0)),
        };
        Ok((length_value, format!("{object}.Length()")))
    }

    /// Evaluate the `IndexOf()` method on a column value
    fn evaluate_indexof(
        &self,
        object: &str,
        search_str: &str,
        row_index: usize,
    ) -> Result<(Option<DataValue>, String)> {
        // Handle qualified column names (table.column or alias.column)
        let resolved_column = if object.contains('.') {
            if let Some(dot_pos) = object.rfind('.') {
                let col_name = &object[dot_pos + 1..];
                col_name
            } else {
                object
            }
        } else {
            object
        };

        let col_index = if let Some(idx) = self.table.get_column_index(resolved_column) {
            idx
        } else if resolved_column != object {
            // If not found, try the original name
            if let Some(idx) = self.table.get_column_index(object) {
                idx
            } else {
                let suggestion = self.find_similar_column(resolved_column);
                return Err(match suggestion {
                    Some(similar) => {
                        anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                    }
                    None => anyhow!("Column '{}' not found", object),
                });
            }
        } else {
            let suggestion = self.find_similar_column(resolved_column);
            return Err(match suggestion {
                Some(similar) => {
                    anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                }
                None => anyhow!("Column '{}' not found", object),
            });
        };

        let value = self.table.get_value(row_index, col_index);
        let index_value = match value {
            Some(DataValue::String(s)) => {
                // Case-insensitive search by default, following Contains behavior
                let pos = s
                    .to_lowercase()
                    .find(&search_str.to_lowercase())
                    .map_or(-1, |idx| idx as i64);
                Some(DataValue::Integer(pos))
            }
            Some(DataValue::InternedString(s)) => {
                let pos = s
                    .to_lowercase()
                    .find(&search_str.to_lowercase())
                    .map_or(-1, |idx| idx as i64);
                Some(DataValue::Integer(pos))
            }
            Some(DataValue::Integer(n)) => {
                let str_val = n.to_string();
                let pos = str_val.find(search_str).map_or(-1, |idx| idx as i64);
                Some(DataValue::Integer(pos))
            }
            Some(DataValue::Float(f)) => {
                let str_val = f.to_string();
                let pos = str_val.find(search_str).map_or(-1, |idx| idx as i64);
                Some(DataValue::Integer(pos))
            }
            _ => Some(DataValue::Integer(-1)), // Return -1 for not found
        };

        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: Row {} IndexOf('{}') = {:?}",
                row_index, search_str, index_value
            );
        }
        Ok((index_value, format!("{object}.IndexOf('{search_str}')")))
    }

    /// Apply the appropriate trim operation based on `trim_type`
    fn apply_trim<'b>(s: &'b str, trim_type: &str) -> &'b str {
        match trim_type {
            "trim" => s.trim(),
            "trimstart" => s.trim_start(),
            "trimend" => s.trim_end(),
            _ => s,
        }
    }

    /// Evaluate trim methods (Trim, `TrimStart`, `TrimEnd`) on a column value
    fn evaluate_trim(
        &self,
        object: &str,
        row_index: usize,
        trim_type: &str,
    ) -> Result<(Option<DataValue>, String)> {
        // Handle qualified column names (table.column or alias.column)
        let resolved_column = if object.contains('.') {
            if let Some(dot_pos) = object.rfind('.') {
                let col_name = &object[dot_pos + 1..];
                col_name
            } else {
                object
            }
        } else {
            object
        };

        let col_index = if let Some(idx) = self.table.get_column_index(resolved_column) {
            idx
        } else if resolved_column != object {
            // If not found, try the original name
            if let Some(idx) = self.table.get_column_index(object) {
                idx
            } else {
                let suggestion = self.find_similar_column(resolved_column);
                return Err(match suggestion {
                    Some(similar) => {
                        anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                    }
                    None => anyhow!("Column '{}' not found", object),
                });
            }
        } else {
            let suggestion = self.find_similar_column(resolved_column);
            return Err(match suggestion {
                Some(similar) => {
                    anyhow!("Column '{}' not found. Did you mean '{}'?", object, similar)
                }
                None => anyhow!("Column '{}' not found", object),
            });
        };

        let value = self.table.get_value(row_index, col_index);
        let trimmed_value = match value {
            Some(DataValue::String(s)) => Some(DataValue::String(
                Self::apply_trim(s, trim_type).to_string(),
            )),
            Some(DataValue::InternedString(s)) => Some(DataValue::String(
                Self::apply_trim(s, trim_type).to_string(),
            )),
            Some(DataValue::Integer(n)) => {
                let str_val = n.to_string();
                Some(DataValue::String(
                    Self::apply_trim(&str_val, trim_type).to_string(),
                ))
            }
            Some(DataValue::Float(f)) => {
                let str_val = f.to_string();
                Some(DataValue::String(
                    Self::apply_trim(&str_val, trim_type).to_string(),
                ))
            }
            _ => Some(DataValue::String(String::new())),
        };

        let method_name = match trim_type {
            "trim" => "Trim",
            "trimstart" => "TrimStart",
            "trimend" => "TrimEnd",
            _ => "Trim",
        };
        Ok((trimmed_value, format!("{object}.{method_name}()")))
    }

    /// Evaluate a WHERE clause for a specific row
    pub fn evaluate(&mut self, where_clause: &WhereClause, row_index: usize) -> Result<Trilean> {
        // Only log for first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate() ENTRY - row {}, {} conditions, case_insensitive={}",
                row_index,
                where_clause.conditions.len(),
                self.case_insensitive
            );
        }

        if where_clause.conditions.is_empty() {
            if row_index < 3 {
                debug!("RecursiveWhereEvaluator: evaluate() EXIT - no conditions, returning true");
            }
            return Ok(Trilean::True);
        }

        // With the new expression tree structure, we should have a single condition
        // containing the entire WHERE clause expression tree
        if where_clause.conditions.len() == 1 {
            // New structure: single expression tree
            if row_index < 3 {
                debug!(
                    "RecursiveWhereEvaluator: evaluate() - evaluating expression tree for row {}",
                    row_index
                );
            }
            self.evaluate_condition(&where_clause.conditions[0], row_index)
        } else {
            // Legacy structure: multiple conditions with connectors
            // This path is kept for backward compatibility
            if row_index < 3 {
                debug!(
                    "RecursiveWhereEvaluator: evaluate() - evaluating {} conditions with connectors for row {}",
                    where_clause.conditions.len(),
                    row_index
                );
            }
            let mut result = self.evaluate_condition(&where_clause.conditions[0], row_index)?;

            // Apply connectors (AND/OR) with subsequent conditions
            for i in 1..where_clause.conditions.len() {
                let next_result =
                    self.evaluate_condition(&where_clause.conditions[i], row_index)?;

                // Use the connector from the previous condition
                if let Some(connector) = &where_clause.conditions[i - 1].connector {
                    result = match connector {
                        LogicalOp::And => result.and(next_result),
                        LogicalOp::Or => result.or(next_result),
                    };
                }
            }

            Ok(result)
        }
    }

    fn evaluate_condition(&mut self, condition: &Condition, row_index: usize) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_condition() ENTRY - row {}",
                row_index
            );
        }
        let result = self.evaluate_expression(&condition.expr, row_index);
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_condition() EXIT - row {}, result = {:?}",
                row_index, result
            );
        }
        result
    }

    fn evaluate_expression(&mut self, expr: &SqlExpression, row_index: usize) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_expression() ENTRY - row {}, expr = {:?}",
                row_index, expr
            );
        }

        let result = match expr {
            SqlExpression::BinaryOp { left, op, right } => {
                self.evaluate_binary_op(left, op, right, row_index)
            }
            SqlExpression::InList { expr, values } => {
                self.evaluate_in_list(expr, values, row_index, false)
            }
            SqlExpression::NotInList { expr, values } => {
                let in_result = self.evaluate_in_list(expr, values, row_index, false)?;
                // Three-valued negation: once `evaluate_in_list` can return
                // UNKNOWN (a NULL operand, or a NULL in the list), this must
                // stay UNKNOWN rather than flip to TRUE. That flip is P19.
                Ok(in_result.negate())
            }
            SqlExpression::Between { expr, lower, upper } => {
                self.evaluate_between(expr, lower, upper, row_index)
            }
            SqlExpression::Not { expr } => {
                let inner_result = self.evaluate_expression(expr, row_index)?;
                // `NOT UNKNOWN` is UNKNOWN, not TRUE — see P18/P19.
                Ok(inner_result.negate())
            }
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => {
                if row_index < 3 {
                    debug!("RecursiveWhereEvaluator: evaluate_expression() - found MethodCall, delegating to evaluate_method_call");
                }
                self.evaluate_method_call(object, method, args, row_index)
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                if row_index < 3 {
                    debug!("RecursiveWhereEvaluator: evaluate_expression() - found CaseExpression, evaluating");
                }
                self.evaluate_case_expression_as_bool(when_branches, else_branch, row_index)
            }
            // A bare value expression used as a predicate -- `WHERE flag`,
            // `WHERE true`, and the `WHERE lifted_value` that
            // `ExpressionLifter` rewrites a window comparison into. This arm
            // used to answer FALSE for every row, which is how P37 turned an
            // unsupported shape into a silently empty result set.
            _ => self.evaluate_value_as_predicate(expr, row_index),
        };

        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_expression() EXIT - row {}, result = {:?}",
                row_index, result
            );
        }
        result
    }

    fn evaluate_binary_op(
        &mut self,
        left: &SqlExpression,
        op: &str,
        right: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_binary_op() ENTRY - row {}, op = '{}'",
                row_index, op
            );
        }

        // Handle logical operators (AND, OR) specially
        if op.to_uppercase() == "OR" || op.to_uppercase() == "AND" {
            let left_result = self.evaluate_expression(left, row_index)?;
            let right_result = self.evaluate_expression(right, row_index)?;

            return Ok(match op.to_uppercase().as_str() {
                // `Trilean::or` / `and` are the SQL truth tables, so FALSE still
                // dominates AND and TRUE still dominates OR even when the other
                // side is UNKNOWN — which `||`/`&&` over a collapsed bool could
                // not express.
                "OR" => left_result.or(right_result),
                "AND" => left_result.and(right_result),
                _ => unreachable!(),
            });
        }

        // For complex expressions (arithmetic, functions), use ArithmeticEvaluator
        if matches!(left, SqlExpression::BinaryOp { .. })
            || matches!(left, SqlExpression::FunctionCall { .. })
            || matches!(right, SqlExpression::BinaryOp { .. })
            || matches!(right, SqlExpression::FunctionCall { .. })
        {
            let comparison_expr = SqlExpression::BinaryOp {
                left: Box::new(left.clone()),
                op: op.to_string(),
                right: Box::new(right.clone()),
            };

            let mut evaluator = ArithmeticEvaluator::new(self.table);
            let result = evaluator.evaluate(&comparison_expr, row_index)?;

            // Convert the result to a truth value. A NULL out of the
            // arithmetic evaluator means a NULL propagated through the
            // expression, so the comparison is UNKNOWN rather than false.
            return match result {
                DataValue::Boolean(b) => Ok(Trilean::from_bool(b)),
                DataValue::Null => Ok(Trilean::Unknown),
                _ => Err(anyhow!("Comparison did not return a boolean value")),
            };
        }

        // For simple comparisons, use the original WHERE clause logic with improved date parsing
        // Handle left side - could be a column or a method call
        let (cell_value, column_name) = match left {
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => {
                // Handle method calls that return values (like Length(), IndexOf())
                match method.to_lowercase().as_str() {
                    "length" => {
                        if !args.is_empty() {
                            return Err(anyhow::anyhow!("Length() takes no arguments"));
                        }
                        self.evaluate_length(object, row_index)?
                    }
                    "indexof" => {
                        if args.len() != 1 {
                            return Err(anyhow::anyhow!("IndexOf() requires exactly 1 argument"));
                        }
                        let search_str = self.extract_string_value(&args[0])?;
                        self.evaluate_indexof(object, &search_str, row_index)?
                    }
                    "trim" => {
                        if !args.is_empty() {
                            return Err(anyhow::anyhow!("Trim() takes no arguments"));
                        }
                        self.evaluate_trim(object, row_index, "trim")?
                    }
                    "trimstart" => {
                        if !args.is_empty() {
                            return Err(anyhow::anyhow!("TrimStart() takes no arguments"));
                        }
                        self.evaluate_trim(object, row_index, "trimstart")?
                    }
                    "trimend" => {
                        if !args.is_empty() {
                            return Err(anyhow::anyhow!("TrimEnd() takes no arguments"));
                        }
                        self.evaluate_trim(object, row_index, "trimend")?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Method '{}' cannot be used in comparisons",
                            method
                        ));
                    }
                }
            }
            _ => {
                // Regular column reference
                let column_name = self.extract_column_name(left)?;
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: evaluate_binary_op() - column_name = '{}'",
                        column_name
                    );
                }

                let col_index = self.table.get_column_index(&column_name).ok_or_else(|| {
                    let suggestion = self.find_similar_column(&column_name);
                    match suggestion {
                        Some(similar) => anyhow!(
                            "Column '{}' not found. Did you mean '{}'?",
                            column_name,
                            similar
                        ),
                        None => anyhow!("Column '{}' not found", column_name),
                    }
                })?;

                let cell_value = self.table.get_value(row_index, col_index).cloned();
                (cell_value, column_name)
            }
        };

        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_binary_op() - row {} column '{}' value = {:?}",
                row_index, column_name, cell_value
            );
        }

        // Get comparison value from right side
        let compare_value = self.extract_value(right)?;

        // Handle special operators that aren't standard comparisons
        let op_upper = op.to_uppercase();
        match op_upper.as_str() {
            // LIKE operator - handle specially
            "LIKE" => {
                let table_value = cell_value.unwrap_or(DataValue::Null);
                let pattern = match compare_value {
                    ExprValue::String(s) => s,
                    // A NULL pattern makes the whole predicate UNKNOWN; any
                    // other non-string pattern is simply not a match.
                    ExprValue::Null => return Ok(Trilean::Unknown),
                    _ => return Ok(Trilean::False),
                };

                let text = match &table_value {
                    DataValue::String(s) => s.as_str(),
                    DataValue::InternedString(s) => s.as_str(),
                    // Likewise on the value side: NULL LIKE '...' is UNKNOWN,
                    // which matters under NOT — see P19.
                    DataValue::Null => return Ok(Trilean::Unknown),
                    _ => return Ok(Trilean::False),
                };

                // Use cached regex if context is available, otherwise compile fresh
                if let Some(ctx) = &mut self.context {
                    let regex = ctx
                        .get_or_compile_like_regex(&pattern)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    Ok(Trilean::from_bool(regex.is_match(text)))
                } else {
                    // Fallback to compiling regex each time (old behavior)
                    let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
                    let regex = regex::RegexBuilder::new(&format!("^{regex_pattern}$"))
                        .case_insensitive(self.case_insensitive)
                        .build()
                        .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;
                    Ok(Trilean::from_bool(regex.is_match(text)))
                }
            }

            // IS NULL / IS NOT NULL
            "IS NULL" => Ok(Trilean::from_bool(
                cell_value.is_none() || matches!(cell_value, Some(DataValue::Null)),
            )),
            "IS NOT NULL" => Ok(Trilean::from_bool(
                cell_value.is_some() && !matches!(cell_value, Some(DataValue::Null)),
            )),

            // Handle IS / IS NOT with NULL explicitly
            "IS" if matches!(compare_value, ExprValue::Null) => Ok(Trilean::from_bool(
                cell_value.is_none() || matches!(cell_value, Some(DataValue::Null)),
            )),
            "IS NOT" if matches!(compare_value, ExprValue::Null) => Ok(Trilean::from_bool(
                cell_value.is_some() && !matches!(cell_value, Some(DataValue::Null)),
            )),

            // Standard comparison operators - use centralized logic
            _ => {
                let table_value = cell_value.unwrap_or(DataValue::Null);
                let comparison_value = self.expr_value_to_data_value(&compare_value);

                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: Using centralized comparison - table: {:?}, op: '{}', comparison: {:?}, case_insensitive: {}",
                        table_value, op, comparison_value, self.case_insensitive
                    );
                }

                Ok(self.compare_trilean(&table_value, &comparison_value, op))
            }
        }
    }

    /// Resolve the value of a WHERE operand that is expected to yield a scalar:
    /// a plain column reference (looked up in the row) or an arbitrary
    /// expression (evaluated with the `ArithmeticEvaluator`). IN / BETWEEN take
    /// such an operand on their left; it is usually a bare column, but after an
    /// IN-subquery is substituted into an IN-list the LHS keeps its original
    /// expression form (e.g. `price * 2`), which the `InOperatorLifter` never
    /// got to lift because that runs before subquery substitution. Mirrors the
    /// arithmetic delegation `evaluate_binary_op` already does for its LHS.
    fn evaluate_operand_value(
        &self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<Option<DataValue>> {
        match expr {
            SqlExpression::Column(_) => {
                let column_name = self.extract_column_name(expr)?;
                let col_index = self
                    .table
                    .get_column_index(&column_name)
                    .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;
                Ok(self.table.get_value(row_index, col_index).cloned())
            }
            _ => {
                let mut evaluator = ArithmeticEvaluator::new(self.table);
                Ok(Some(evaluator.evaluate(expr, row_index)?))
            }
        }
    }

    fn evaluate_in_list(
        &self,
        expr: &SqlExpression,
        values: &[SqlExpression],
        row_index: usize,
        _ignore_case: bool,
    ) -> Result<Trilean> {
        let cell_value = self.evaluate_operand_value(expr, row_index)?;

        // `x IN (a, b, ...)` is `x = a OR x = b OR ...`, so it inherits OR's
        // truth table: a TRUE anywhere wins outright, but if nothing matched
        // and any comparison was UNKNOWN, the answer is UNKNOWN — not FALSE.
        // That distinction is invisible under `IN` (both drop the row) and is
        // the whole of P19 under `NOT IN`, where FALSE would wrongly negate to
        // TRUE and admit the NULL rows.
        let mut saw_unknown = false;

        for value_expr in values {
            let compare_value = self.extract_value(value_expr)?;
            let table_value = cell_value.as_ref().unwrap_or(&DataValue::Null);
            let comparison_value = self.expr_value_to_data_value(&compare_value);

            match self.compare_trilean(table_value, &comparison_value, "=") {
                Trilean::True => return Ok(Trilean::True),
                Trilean::Unknown => saw_unknown = true,
                Trilean::False => {}
            }
        }

        Ok(if saw_unknown {
            Trilean::Unknown
        } else {
            Trilean::False
        })
    }

    fn evaluate_between(
        &self,
        expr: &SqlExpression,
        lower: &SqlExpression,
        upper: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        let cell_value = self.evaluate_operand_value(expr, row_index)?;
        let lower_value = self.extract_value(lower)?;
        let upper_value = self.extract_value(upper)?;

        let table_value = cell_value.unwrap_or(DataValue::Null);
        let lower_data_value = self.expr_value_to_data_value(&lower_value);
        let upper_data_value = self.expr_value_to_data_value(&upper_value);

        // BETWEEN is defined as `value >= lower AND value <= upper`, so it takes
        // AND's truth table too: FALSE on either side still wins outright (a
        // value below `lower` is not between, whatever `upper` is), and UNKNOWN
        // only survives when the other side is TRUE or UNKNOWN.
        let ge_lower = self.compare_trilean(&table_value, &lower_data_value, ">=");
        let le_upper = self.compare_trilean(&table_value, &upper_data_value, "<=");

        Ok(ge_lower.and(le_upper))
    }

    fn evaluate_method_call(
        &self,
        object: &str,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<Trilean> {
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_method_call - object='{}', method='{}', row={}",
                object, method, row_index
            );
        }

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
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: Row {} column '{}' value = {:?}",
                row_index, object, cell_value
            );
        }

        match method.to_lowercase().as_str() {
            "contains" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("Contains requires exactly 1 argument"));
                }
                let search_str = self.extract_string_value(&args[0])?;
                // Pre-compute lowercase once instead of for every row
                let search_lower = search_str.to_lowercase();

                // Type coercion: convert numeric values to strings for string methods
                match cell_value {
                    Some(DataValue::String(ref s)) => {
                        let result = s.to_lowercase().contains(&search_lower);
                        // Only log first few rows to avoid performance impact
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on '{}' = {} (case-insensitive)", row_index, search_str, s, result);
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    Some(DataValue::InternedString(ref s)) => {
                        let result = s.to_lowercase().contains(&search_lower);
                        // Only log first few rows to avoid performance impact
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on interned '{}' = {} (case-insensitive)", row_index, search_str, s, result);
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    Some(DataValue::Integer(n)) => {
                        let str_val = n.to_string();
                        let result = str_val.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on integer '{}' = {}", row_index, search_str, str_val, result);
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    Some(DataValue::Float(f)) => {
                        let str_val = f.to_string();
                        let result = str_val.contains(&search_str);
                        if row_index < 3 {
                            debug!(
                                "RecursiveWhereEvaluator: Row {} contains('{}') on float '{}' = {}",
                                row_index, search_str, str_val, result
                            );
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    Some(DataValue::Boolean(b)) => {
                        let str_val = b.to_string();
                        let result = str_val.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on boolean '{}' = {}", row_index, search_str, str_val, result);
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    Some(DataValue::DateTime(dt)) => {
                        // DateTime columns can use string methods via coercion
                        let result = dt.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on datetime '{}' = {}", row_index, search_str, dt, result);
                        }
                        Ok(Trilean::from_bool(result))
                    }
                    _ => {
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on null/empty value = false", row_index, search_str);
                        }
                        Ok(Trilean::False)
                    }
                }
            }
            "startswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("StartsWith requires exactly 1 argument"));
                }
                let prefix = self.extract_string_value(&args[0])?;

                // Type coercion: convert numeric values to strings for string methods
                match cell_value {
                    Some(DataValue::String(ref s)) => Ok(Trilean::from_bool(
                        s.to_lowercase().starts_with(&prefix.to_lowercase()),
                    )),
                    Some(DataValue::InternedString(ref s)) => Ok(Trilean::from_bool(
                        s.to_lowercase().starts_with(&prefix.to_lowercase()),
                    )),
                    Some(DataValue::Integer(n)) => {
                        Ok(Trilean::from_bool(n.to_string().starts_with(&prefix)))
                    }
                    Some(DataValue::Float(f)) => {
                        Ok(Trilean::from_bool(f.to_string().starts_with(&prefix)))
                    }
                    Some(DataValue::Boolean(b)) => {
                        Ok(Trilean::from_bool(b.to_string().starts_with(&prefix)))
                    }
                    Some(DataValue::DateTime(dt)) => {
                        Ok(Trilean::from_bool(dt.starts_with(&prefix)))
                    }
                    _ => Ok(Trilean::False),
                }
            }
            "endswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("EndsWith requires exactly 1 argument"));
                }
                let suffix = self.extract_string_value(&args[0])?;

                // Type coercion: convert numeric values to strings for string methods
                match cell_value {
                    Some(DataValue::String(ref s)) => Ok(Trilean::from_bool(
                        s.to_lowercase().ends_with(&suffix.to_lowercase()),
                    )),
                    Some(DataValue::InternedString(ref s)) => Ok(Trilean::from_bool(
                        s.to_lowercase().ends_with(&suffix.to_lowercase()),
                    )),
                    Some(DataValue::Integer(n)) => {
                        Ok(Trilean::from_bool(n.to_string().ends_with(&suffix)))
                    }
                    Some(DataValue::Float(f)) => {
                        Ok(Trilean::from_bool(f.to_string().ends_with(&suffix)))
                    }
                    Some(DataValue::Boolean(b)) => {
                        Ok(Trilean::from_bool(b.to_string().ends_with(&suffix)))
                    }
                    Some(DataValue::DateTime(dt)) => Ok(Trilean::from_bool(dt.ends_with(&suffix))),
                    _ => Ok(Trilean::False),
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported method: {}", method)),
        }
    }

    fn extract_column_name(&self, expr: &SqlExpression) -> Result<String> {
        match expr {
            SqlExpression::Column(column_ref) => {
                // Use ExecutionContext for proper alias resolution if available
                if let Some(exec_ctx) = self.exec_context {
                    // Use the unified column resolution
                    let col_idx = exec_ctx.resolve_column_index(self.table, column_ref)?;
                    // Return the actual column name (not the qualified one)
                    Ok(self.table.column_names()[col_idx].clone())
                } else {
                    // Fallback: Handle qualified column names the old way (for backward compatibility)
                    if column_ref.name.contains('.') {
                        if let Some(dot_pos) = column_ref.name.rfind('.') {
                            Ok(column_ref.name[dot_pos + 1..].to_string())
                        } else {
                            Ok(column_ref.name.clone())
                        }
                    } else {
                        Ok(column_ref.name.clone())
                    }
                }
            }
            _ => Err(anyhow::anyhow!("Expected column name, got: {:?}", expr)),
        }
    }

    fn extract_string_value(&self, expr: &SqlExpression) -> Result<String> {
        match expr {
            SqlExpression::StringLiteral(s) => Ok(s.clone()),
            _ => Err(anyhow::anyhow!("Expected string literal, got: {:?}", expr)),
        }
    }

    fn extract_value(&self, expr: &SqlExpression) -> Result<ExprValue> {
        match expr {
            SqlExpression::StringLiteral(s) => Ok(ExprValue::String(s.clone())),
            SqlExpression::BooleanLiteral(b) => Ok(ExprValue::Boolean(*b)),
            SqlExpression::NumberLiteral(n) => {
                if let Ok(num) = n.parse::<f64>() {
                    Ok(ExprValue::Number(num))
                } else {
                    Ok(ExprValue::String(n.clone()))
                }
            }
            SqlExpression::DateTimeConstructor {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                // Create a DateTime from the constructor
                let naive_date = NaiveDate::from_ymd_opt(*year, *month, *day)
                    .ok_or_else(|| anyhow::anyhow!("Invalid date: {}-{}-{}", year, month, day))?;
                let naive_time = NaiveTime::from_hms_opt(
                    hour.unwrap_or(0),
                    minute.unwrap_or(0),
                    second.unwrap_or(0),
                )
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
                let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
                let datetime = Utc.from_utc_datetime(&naive_datetime);
                Ok(ExprValue::DateTime(datetime))
            }
            SqlExpression::DateTimeToday {
                hour,
                minute,
                second,
            } => {
                // Get today's date with optional time
                let today = Local::now().date_naive();
                let time = NaiveTime::from_hms_opt(
                    hour.unwrap_or(0),
                    minute.unwrap_or(0),
                    second.unwrap_or(0),
                )
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
                let naive_datetime = NaiveDateTime::new(today, time);
                let datetime = Utc.from_utc_datetime(&naive_datetime);
                Ok(ExprValue::DateTime(datetime))
            }
            _ => Ok(ExprValue::Null),
        }
    }

    /// Evaluate a CASE expression as a boolean (for WHERE clauses)
    fn evaluate_case_expression_as_bool(
        &mut self,
        when_branches: &[crate::sql::recursive_parser::WhenBranch],
        else_branch: &Option<Box<SqlExpression>>,
        row_index: usize,
    ) -> Result<Trilean> {
        debug!(
            "RecursiveWhereEvaluator: evaluating CASE expression as bool for row {}",
            row_index
        );

        // Evaluate each WHEN condition in order
        for branch in when_branches {
            // Evaluate the condition as a boolean
            let condition_result = self.evaluate_expression(&branch.condition, row_index)?;

            // A WHEN whose condition is UNKNOWN does not match, exactly like
            // FALSE — the same rule the row filter applies.
            if condition_result.is_true() {
                debug!("CASE: WHEN condition matched, evaluating result expression as bool");
                // Evaluate the result and convert to boolean
                return self.evaluate_expression_as_bool(&branch.result, row_index);
            }
        }

        // If no WHEN condition matched, evaluate ELSE clause (or return false)
        if let Some(else_expr) = else_branch {
            debug!("CASE: No WHEN matched, evaluating ELSE expression as bool");
            self.evaluate_expression_as_bool(else_expr, row_index)
        } else {
            debug!("CASE: No WHEN matched and no ELSE, returning false");
            Ok(Trilean::False)
        }
    }

    /// Evaluate an expression for its VALUE and coerce that value to a
    /// predicate (P37).
    ///
    /// This is the tail of both `evaluate_expression` (a bare value used
    /// directly as a `WHERE` predicate: `WHERE flag`, `WHERE true`, and the
    /// `WHERE lifted_value` that `ExpressionLifter` rewrites a window
    /// comparison into) and `evaluate_expression_as_bool` (the result of a
    /// CASE branch). Both used to have their own copy of the coercion table
    /// and disagreed on NULL; they now share this one.
    ///
    /// A raw `WindowFunction` reaching here means the lifter did not hoist it,
    /// which is a defect rather than a value to coerce -- so it errors. That
    /// is the P37 rule: a loud failure beats a silently empty result.
    fn evaluate_value_as_predicate(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        if let SqlExpression::WindowFunction { name, .. } = expr {
            return Err(anyhow::anyhow!(
                "Window function {name} cannot be used directly as a predicate (the expression was not lifted to a CTE column)"
            ));
        }

        let mut evaluator = crate::data::arithmetic_evaluator::ArithmeticEvaluator::new(self.table);
        let value = evaluator.evaluate(expr, row_index)?;

        use crate::data::datatable::DataValue;
        Ok(match value {
            DataValue::Boolean(b) => Trilean::from_bool(b),
            DataValue::Integer(i) => Trilean::from_bool(i != 0),
            DataValue::Float(f) => Trilean::from_bool(f != 0.0),
            // A NULL predicate is UNKNOWN, not FALSE. Under `WHERE` alone the
            // two are indistinguishable -- both drop the row -- and they only
            // diverge under `NOT`, which is exactly the P18/P19 trap. The CASE
            // path used to answer FALSE here; this is the one deliberate
            // behaviour change in unifying the two copies.
            DataValue::Null => Trilean::Unknown,
            DataValue::String(ref s) => Trilean::from_bool(!s.is_empty()),
            DataValue::InternedString(ref s) => Trilean::from_bool(!s.is_empty()),
            _ => Trilean::True,
        })
    }

    fn evaluate_expression_as_bool(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        match expr {
            // For expressions that naturally return booleans, use the existing evaluator
            SqlExpression::BinaryOp { .. }
            | SqlExpression::InList { .. }
            | SqlExpression::NotInList { .. }
            | SqlExpression::Between { .. }
            | SqlExpression::Not { .. }
            | SqlExpression::MethodCall { .. } => self.evaluate_expression(expr, row_index),
            // For CASE expressions, recurse
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => self.evaluate_case_expression_as_bool(when_branches, else_branch, row_index),
            // For other expressions (columns, literals), evaluate the value
            // and coerce -- the same rule the WHERE predicate path uses.
            _ => self.evaluate_value_as_predicate(expr, row_index),
        }
    }
}

enum ExprValue {
    String(String),
    Number(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Null,
}

#[cfg(test)]
mod three_valued_logic_tests {
    //! Regression tests for P18/P19 — SQL three-valued logic in `WHERE`.
    //!
    //! These assert the `Trilean` the evaluator produces for a single row,
    //! rather than the rows a query returns, because that is where the defect
    //! actually lived: UNKNOWN and FALSE are indistinguishable by row count
    //! under `WHERE` (both drop the row) and only diverge under `NOT`. A
    //! row-counting test would have passed against the broken evaluator for
    //! half of these cases.
    //!
    //! They also run without DuckDB, unlike the corpus cases in
    //! `tests/comparison/corpus/` that pin the same behaviour end to end.

    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};
    use crate::sql::recursive_parser::Parser;

    /// Rows: 0 = score 50, 1 = score 30, 2 = score NULL.
    fn table_with_nulls() -> DataTable {
        let mut table = DataTable::new("t");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("score"));
        table.add_column(DataColumn::new("label"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::Integer(50),
                DataValue::String("alpha".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Integer(30),
                DataValue::String("beta".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Null,
                DataValue::Null,
            ]))
            .unwrap();

        table
    }

    /// Evaluate a WHERE clause against one row and return its truth value.
    fn eval(table: &DataTable, predicate: &str, row: usize) -> Trilean {
        let sql = format!("SELECT * FROM t WHERE {predicate}");
        let mut parser = Parser::new(&sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(table);
        evaluator
            .evaluate(&where_clause, row)
            .expect("evaluation failed")
    }

    // --- P18: `= NULL` yields UNKNOWN, never a match ---

    #[test]
    fn equals_null_is_unknown_for_every_row() {
        let t = table_with_nulls();
        // Including the NULL row itself: `NULL = NULL` is UNKNOWN, not TRUE.
        // Returning TRUE here is exactly what made `WHERE score = NULL` behave
        // like `IS NULL`.
        for row in 0..3 {
            assert_eq!(eval(&t, "score = NULL", row), Trilean::Unknown, "row {row}");
        }
    }

    #[test]
    fn comparison_against_a_null_column_is_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score > 10", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score <> 50", 2), Trilean::Unknown);
    }

    #[test]
    fn is_null_stays_two_valued() {
        // The sanctioned way to match a NULL must keep working — that is what
        // makes losing `= NULL` costless.
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score IS NULL", 2), Trilean::True);
        assert_eq!(eval(&t, "score IS NULL", 0), Trilean::False);
        assert_eq!(eval(&t, "score IS NOT NULL", 0), Trilean::True);
        assert_eq!(eval(&t, "score IS NOT NULL", 2), Trilean::False);
    }

    #[test]
    fn in_list_with_a_null_matches_only_real_equals() {
        let t = table_with_nulls();
        // A match still wins outright, even with a NULL in the list.
        assert_eq!(eval(&t, "score IN (50, NULL)", 0), Trilean::True);
        // No match, but a NULL was compared: UNKNOWN, not FALSE. Returning
        // FALSE here is invisible under IN and wrong under NOT IN.
        assert_eq!(eval(&t, "score IN (50, NULL)", 1), Trilean::Unknown);
        // NULL column against a list with no NULL in it: also UNKNOWN.
        assert_eq!(eval(&t, "score IN (50, 70)", 2), Trilean::Unknown);
        // Nothing NULL anywhere: ordinary FALSE.
        assert_eq!(eval(&t, "score IN (50, 70)", 1), Trilean::False);
    }

    // --- P19: NOT must not turn UNKNOWN into TRUE ---

    #[test]
    fn not_in_excludes_nulls() {
        let t = table_with_nulls();
        // The bug: UNKNOWN collapsed to false and `!false` admitted the row.
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 0), Trilean::False);
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 1), Trilean::True);
    }

    #[test]
    fn not_over_an_unknown_comparison_stays_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "NOT (score > 50)", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT (score > 50)", 1), Trilean::True);
    }

    // --- The truth tables, exercised through real predicates ---

    /// CONTROL — passes with and without the P18/P19 fix, because the old
    /// bool evaluator also produced FALSE here (for the wrong reason: it
    /// collapsed the UNKNOWN rather than letting FALSE dominate). Kept because
    /// it pins the half of AND's truth table the fix must not disturb; do not
    /// read a pass here as evidence the fix works.
    #[test]
    fn false_still_dominates_and_over_unknown() {
        let t = table_with_nulls();
        // Row 1 has score 30, so `score = 50` is FALSE. FALSE AND UNKNOWN is
        // FALSE — the row is excluded for a definite reason, not an unknown
        // one. Getting this wrong would make the NOT of it wrong too.
        assert_eq!(eval(&t, "score = 50 AND label = NULL", 1), Trilean::False);
        assert_eq!(
            eval(&t, "NOT (score = 50 AND label = NULL)", 1),
            Trilean::True
        );
    }

    #[test]
    fn true_still_dominates_or_over_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50 OR label = NULL", 0), Trilean::True);
        // Neither side definite: UNKNOWN.
        assert_eq!(eval(&t, "score = 99 OR label = NULL", 1), Trilean::Unknown);
    }

    #[test]
    fn between_follows_ands_truth_table() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 0), Trilean::True);
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 1), Trilean::False);
        // NULL operand: UNKNOWN, and so is its negation.
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 2), Trilean::Unknown);
        assert_eq!(
            eval(&t, "NOT (score BETWEEN 40 AND 60)", 2),
            Trilean::Unknown
        );
        // Row 1 is 30, below the lower bound, so the answer is FALSE outright
        // even though the upper bound is NULL and that comparison is UNKNOWN.
        assert_eq!(eval(&t, "score BETWEEN 40 AND NULL", 1), Trilean::False);
    }

    #[test]
    fn like_against_a_null_is_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "label LIKE 'a%'", 0), Trilean::True);
        assert_eq!(eval(&t, "label LIKE 'a%'", 1), Trilean::False);
        assert_eq!(eval(&t, "label LIKE 'a%'", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT (label LIKE 'a%')", 2), Trilean::Unknown);
    }

    // --- Controls: ordinary predicates over non-NULL data are untouched ---

    #[test]
    fn control_non_null_predicates_are_unchanged() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50", 0), Trilean::True);
        assert_eq!(eval(&t, "score = 50", 1), Trilean::False);
        assert_eq!(eval(&t, "score > 40 AND label = 'alpha'", 0), Trilean::True);
        assert_eq!(eval(&t, "score > 40 OR label = 'beta'", 1), Trilean::True);
        assert_eq!(eval(&t, "NOT (score = 50)", 1), Trilean::True);
    }
}

#[cfg(test)]
mod bare_value_predicate_tests {
    //! Regression tests for P37 — a bare value expression used as a `WHERE`
    //! predicate.
    //!
    //! These live here rather than in `tests/comparison/corpus/` because the
    //! parity harness structurally cannot see this fix. The corpus case that
    //! found P37 (`09_window.toml :: window_in_where_inline`) is bucketed
    //! `OURS_ONLY`: DuckDB rejects a window function in `WHERE` outright, so we
    //! are in that bucket whether we return the right rows or, as before, zero
    //! rows with a success exit code. `runner.py --check` stays green either
    //! way, and would stay green through a regression too.
    //!
    //! So they assert the general shape rather than the window that exposed
    //! it: the defect was never window-specific. `WHERE true` returned no rows
    //! for the same reason.

    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};
    use crate::sql::recursive_parser::Parser;

    /// Rows: 0 = (true, 1, "x"), 1 = (false, 0, ""), 2 = (NULL, NULL, NULL).
    ///
    /// `flag` stands in for the column `ExpressionLifter` synthesises when it
    /// hoists a window comparison out of `WHERE` — the lifted CTE column is a
    /// plain boolean, and `WHERE lifted_value` is what the main query is left
    /// referencing.
    fn table_with_flags() -> DataTable {
        let mut table = DataTable::new("t");
        table.add_column(DataColumn::new("flag"));
        table.add_column(DataColumn::new("n"));
        table.add_column(DataColumn::new("s"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Boolean(true),
                DataValue::Integer(1),
                DataValue::String("x".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Boolean(false),
                DataValue::Integer(0),
                DataValue::String(String::new()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Null,
                DataValue::Null,
                DataValue::Null,
            ]))
            .unwrap();

        table
    }

    fn eval(table: &DataTable, predicate: &str, row: usize) -> Trilean {
        let sql = format!("SELECT * FROM t WHERE {predicate}");
        let mut parser = Parser::new(&sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(table);
        evaluator
            .evaluate(&where_clause, row)
            .expect("evaluation failed")
    }

    // --- P37: the shapes that used to be FALSE for every row ---

    #[test]
    fn a_boolean_column_is_a_predicate_in_its_own_right() {
        let t = table_with_flags();
        assert_eq!(eval(&t, "flag", 0), Trilean::True);
        assert_eq!(eval(&t, "flag", 1), Trilean::False);
    }

    #[test]
    fn a_boolean_literal_is_a_predicate() {
        let t = table_with_flags();
        // `WHERE true` returned zero rows before the fix, on any table.
        assert_eq!(eval(&t, "true", 0), Trilean::True);
        assert_eq!(eval(&t, "false", 0), Trilean::False);
    }

    #[test]
    fn a_null_valued_predicate_is_unknown_not_false() {
        let t = table_with_flags();
        // The P18/P19 distinction: UNKNOWN and FALSE both drop the row under
        // `WHERE`, and only diverge under `NOT`.
        assert_eq!(eval(&t, "flag", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT flag", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT flag", 1), Trilean::True);
    }

    #[test]
    fn a_bare_value_composes_with_ordinary_predicates() {
        // The lifter can leave `WHERE lifted_value` beside other conditions,
        // so the bare form has to survive AND/OR like any other predicate.
        let t = table_with_flags();
        assert_eq!(eval(&t, "flag AND n = 1", 0), Trilean::True);
        assert_eq!(eval(&t, "flag AND n = 99", 0), Trilean::False);
        // Row 1 is flag=false, n=0 -- so the right operand has to be a miss
        // for the OR to come out FALSE.
        assert_eq!(eval(&t, "flag OR n = 99", 1), Trilean::False);
        assert_eq!(eval(&t, "flag OR n = 99", 0), Trilean::True);
    }

    #[test]
    fn numeric_values_coerce_by_zero_ness() {
        let t = table_with_flags();
        assert_eq!(eval(&t, "n", 0), Trilean::True);
        assert_eq!(eval(&t, "n", 1), Trilean::False);
    }

    // --- Control: the unlifted window is loud, not silently empty ---

    #[test]
    fn an_unlifted_window_function_errors_rather_than_filtering_everything() {
        let t = table_with_flags();
        let sql = "SELECT * FROM t WHERE ROW_NUMBER() OVER (ORDER BY n)";
        let mut parser = Parser::new(sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(&t);
        let err = evaluator
            .evaluate(&where_clause, 0)
            .expect_err("a raw window function in WHERE must not evaluate quietly");
        assert!(
            err.to_string().contains("ROW_NUMBER"),
            "error should name the function, got: {err}"
        );
    }
}
