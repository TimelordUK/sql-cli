use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::evaluation_context::EvaluationContext;
use crate::data::query_engine::ExecutionContext;
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
    pub fn evaluate(&mut self, where_clause: &WhereClause, row_index: usize) -> Result<bool> {
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
            return Ok(true);
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
                        LogicalOp::And => result && next_result,
                        LogicalOp::Or => result || next_result,
                    };
                }
            }

            Ok(result)
        }
    }

    fn evaluate_condition(&mut self, condition: &Condition, row_index: usize) -> Result<bool> {
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

    fn evaluate_expression(&mut self, expr: &SqlExpression, row_index: usize) -> Result<bool> {
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
                Ok(!in_result)
            }
            SqlExpression::Between { expr, lower, upper } => {
                self.evaluate_between(expr, lower, upper, row_index)
            }
            SqlExpression::Not { expr } => {
                let inner_result = self.evaluate_expression(expr, row_index)?;
                Ok(!inner_result)
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
            _ => {
                if row_index < 3 {
                    debug!("RecursiveWhereEvaluator: evaluate_expression() - unsupported expression type, returning false");
                }
                Ok(false) // Default to false for unsupported expressions
            }
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
    ) -> Result<bool> {
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
                "OR" => left_result || right_result,
                "AND" => left_result && right_result,
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

            // Convert the result to boolean
            return match result {
                DataValue::Boolean(b) => Ok(b),
                DataValue::Null => Ok(false),
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
                    _ => return Ok(false),
                };

                let text = match &table_value {
                    DataValue::String(s) => s.as_str(),
                    DataValue::InternedString(s) => s.as_str(),
                    _ => return Ok(false),
                };

                // Use cached regex if context is available, otherwise compile fresh
                if let Some(ctx) = &mut self.context {
                    let regex = ctx
                        .get_or_compile_like_regex(&pattern)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    Ok(regex.is_match(text))
                } else {
                    // Fallback to compiling regex each time (old behavior)
                    let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
                    let regex = regex::RegexBuilder::new(&format!("^{regex_pattern}$"))
                        .case_insensitive(self.case_insensitive)
                        .build()
                        .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;
                    Ok(regex.is_match(text))
                }
            }

            // IS NULL / IS NOT NULL
            "IS NULL" => Ok(cell_value.is_none() || matches!(cell_value, Some(DataValue::Null))),
            "IS NOT NULL" => {
                Ok(cell_value.is_some() && !matches!(cell_value, Some(DataValue::Null)))
            }

            // Handle IS / IS NOT with NULL explicitly
            "IS" if matches!(compare_value, ExprValue::Null) => {
                Ok(cell_value.is_none() || matches!(cell_value, Some(DataValue::Null)))
            }
            "IS NOT" if matches!(compare_value, ExprValue::Null) => {
                Ok(cell_value.is_some() && !matches!(cell_value, Some(DataValue::Null)))
            }

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

                Ok(compare_with_op(
                    &table_value,
                    &comparison_value,
                    op,
                    self.case_insensitive,
                ))
            }
        }
    }

    fn evaluate_in_list(
        &self,
        expr: &SqlExpression,
        values: &[SqlExpression],
        row_index: usize,
        _ignore_case: bool,
    ) -> Result<bool> {
        let column_name = self.extract_column_name(expr)?;
        let col_index = self
            .table
            .get_column_index(&column_name)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;

        let cell_value = self.table.get_value(row_index, col_index).cloned();

        for value_expr in values {
            let compare_value = self.extract_value(value_expr)?;
            let table_value = cell_value.as_ref().unwrap_or(&DataValue::Null);
            let comparison_value = self.expr_value_to_data_value(&compare_value);

            // Use centralized comparison for equality
            if compare_with_op(table_value, &comparison_value, "=", self.case_insensitive) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn evaluate_between(
        &self,
        expr: &SqlExpression,
        lower: &SqlExpression,
        upper: &SqlExpression,
        row_index: usize,
    ) -> Result<bool> {
        let column_name = self.extract_column_name(expr)?;
        let col_index = self
            .table
            .get_column_index(&column_name)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;

        let cell_value = self.table.get_value(row_index, col_index).cloned();
        let lower_value = self.extract_value(lower)?;
        let upper_value = self.extract_value(upper)?;

        let table_value = cell_value.unwrap_or(DataValue::Null);
        let lower_data_value = self.expr_value_to_data_value(&lower_value);
        let upper_data_value = self.expr_value_to_data_value(&upper_value);

        // Use centralized comparison for BETWEEN: value >= lower AND value <= upper
        let ge_lower =
            compare_with_op(&table_value, &lower_data_value, ">=", self.case_insensitive);
        let le_upper =
            compare_with_op(&table_value, &upper_data_value, "<=", self.case_insensitive);

        Ok(ge_lower && le_upper)
    }

    fn evaluate_method_call(
        &self,
        object: &str,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<bool> {
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
                        Ok(result)
                    }
                    Some(DataValue::InternedString(ref s)) => {
                        let result = s.to_lowercase().contains(&search_lower);
                        // Only log first few rows to avoid performance impact
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on interned '{}' = {} (case-insensitive)", row_index, search_str, s, result);
                        }
                        Ok(result)
                    }
                    Some(DataValue::Integer(n)) => {
                        let str_val = n.to_string();
                        let result = str_val.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on integer '{}' = {}", row_index, search_str, str_val, result);
                        }
                        Ok(result)
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
                        Ok(result)
                    }
                    Some(DataValue::Boolean(b)) => {
                        let str_val = b.to_string();
                        let result = str_val.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on boolean '{}' = {}", row_index, search_str, str_val, result);
                        }
                        Ok(result)
                    }
                    Some(DataValue::DateTime(dt)) => {
                        // DateTime columns can use string methods via coercion
                        let result = dt.contains(&search_str);
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on datetime '{}' = {}", row_index, search_str, dt, result);
                        }
                        Ok(result)
                    }
                    _ => {
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: Row {} contains('{}') on null/empty value = false", row_index, search_str);
                        }
                        Ok(false)
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
                    Some(DataValue::String(ref s)) => {
                        Ok(s.to_lowercase().starts_with(&prefix.to_lowercase()))
                    }
                    Some(DataValue::InternedString(ref s)) => {
                        Ok(s.to_lowercase().starts_with(&prefix.to_lowercase()))
                    }
                    Some(DataValue::Integer(n)) => Ok(n.to_string().starts_with(&prefix)),
                    Some(DataValue::Float(f)) => Ok(f.to_string().starts_with(&prefix)),
                    Some(DataValue::Boolean(b)) => Ok(b.to_string().starts_with(&prefix)),
                    Some(DataValue::DateTime(dt)) => Ok(dt.starts_with(&prefix)),
                    _ => Ok(false),
                }
            }
            "endswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("EndsWith requires exactly 1 argument"));
                }
                let suffix = self.extract_string_value(&args[0])?;

                // Type coercion: convert numeric values to strings for string methods
                match cell_value {
                    Some(DataValue::String(ref s)) => {
                        Ok(s.to_lowercase().ends_with(&suffix.to_lowercase()))
                    }
                    Some(DataValue::InternedString(ref s)) => {
                        Ok(s.to_lowercase().ends_with(&suffix.to_lowercase()))
                    }
                    Some(DataValue::Integer(n)) => Ok(n.to_string().ends_with(&suffix)),
                    Some(DataValue::Float(f)) => Ok(f.to_string().ends_with(&suffix)),
                    Some(DataValue::Boolean(b)) => Ok(b.to_string().ends_with(&suffix)),
                    Some(DataValue::DateTime(dt)) => Ok(dt.ends_with(&suffix)),
                    _ => Ok(false),
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
    ) -> Result<bool> {
        debug!(
            "RecursiveWhereEvaluator: evaluating CASE expression as bool for row {}",
            row_index
        );

        // Evaluate each WHEN condition in order
        for branch in when_branches {
            // Evaluate the condition as a boolean
            let condition_result = self.evaluate_expression(&branch.condition, row_index)?;

            if condition_result {
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
            Ok(false)
        }
    }

    /// Helper method to evaluate any expression as a boolean
    fn evaluate_expression_as_bool(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<bool> {
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
            // For other expressions (columns, literals), use ArithmeticEvaluator and convert
            _ => {
                // Use ArithmeticEvaluator to get the value, then convert to boolean
                let mut evaluator =
                    crate::data::arithmetic_evaluator::ArithmeticEvaluator::new(self.table);
                let value = evaluator.evaluate(expr, row_index)?;

                match value {
                    crate::data::datatable::DataValue::Boolean(b) => Ok(b),
                    crate::data::datatable::DataValue::Integer(i) => Ok(i != 0),
                    crate::data::datatable::DataValue::Float(f) => Ok(f != 0.0),
                    crate::data::datatable::DataValue::Null => Ok(false),
                    crate::data::datatable::DataValue::String(s) => Ok(!s.is_empty()),
                    crate::data::datatable::DataValue::InternedString(s) => Ok(!s.is_empty()),
                    _ => Ok(true), // Other types are considered truthy
                }
            }
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
