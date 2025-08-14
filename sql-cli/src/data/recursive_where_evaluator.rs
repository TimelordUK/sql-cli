use crate::data::datatable::{DataTable, DataValue};
use crate::sql::recursive_parser::{Condition, LogicalOp, SqlExpression, WhereClause};
use anyhow::Result;

/// Evaluates WHERE clauses from recursive_parser directly against DataTable
pub struct RecursiveWhereEvaluator<'a> {
    table: &'a DataTable,
}

impl<'a> RecursiveWhereEvaluator<'a> {
    pub fn new(table: &'a DataTable) -> Self {
        Self { table }
    }

    /// Evaluate a WHERE clause for a specific row
    pub fn evaluate(&self, where_clause: &WhereClause, row_index: usize) -> Result<bool> {
        if where_clause.conditions.is_empty() {
            return Ok(true);
        }

        // Evaluate first condition
        let mut result = self.evaluate_condition(&where_clause.conditions[0], row_index)?;

        // Apply connectors (AND/OR) with subsequent conditions
        for i in 1..where_clause.conditions.len() {
            let next_result = self.evaluate_condition(&where_clause.conditions[i], row_index)?;

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

    fn evaluate_condition(&self, condition: &Condition, row_index: usize) -> Result<bool> {
        self.evaluate_expression(&condition.expr, row_index)
    }

    fn evaluate_expression(&self, expr: &SqlExpression, row_index: usize) -> Result<bool> {
        match expr {
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
            } => self.evaluate_method_call(object, method, args, row_index),
            _ => Ok(false), // Default to false for unsupported expressions
        }
    }

    fn evaluate_binary_op(
        &self,
        left: &SqlExpression,
        op: &str,
        right: &SqlExpression,
        row_index: usize,
    ) -> Result<bool> {
        // Get column value from left side
        let column_name = self.extract_column_name(left)?;
        let col_index = self
            .table
            .get_column_index(&column_name)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;

        let cell_value = self.table.get_value(row_index, col_index);

        // Get comparison value from right side
        let compare_value = self.extract_value(right)?;

        // Perform comparison
        match (cell_value, op.to_uppercase().as_str(), &compare_value) {
            (Some(DataValue::String(a)), "=", ExprValue::String(b)) => Ok(a == b),
            (Some(DataValue::String(a)), "!=", ExprValue::String(b))
            | (Some(DataValue::String(a)), "<>", ExprValue::String(b)) => Ok(a != b),
            (Some(DataValue::String(a)), ">", ExprValue::String(b)) => Ok(a > b),
            (Some(DataValue::String(a)), ">=", ExprValue::String(b)) => Ok(a >= b),
            (Some(DataValue::String(a)), "<", ExprValue::String(b)) => Ok(a < b),
            (Some(DataValue::String(a)), "<=", ExprValue::String(b)) => Ok(a <= b),

            (Some(DataValue::Integer(a)), "=", ExprValue::Number(b)) => Ok(*a as f64 == *b),
            (Some(DataValue::Integer(a)), "!=", ExprValue::Number(b))
            | (Some(DataValue::Integer(a)), "<>", ExprValue::Number(b)) => Ok(*a as f64 != *b),
            (Some(DataValue::Integer(a)), ">", ExprValue::Number(b)) => Ok(*a as f64 > *b),
            (Some(DataValue::Integer(a)), ">=", ExprValue::Number(b)) => Ok(*a as f64 >= *b),
            (Some(DataValue::Integer(a)), "<", ExprValue::Number(b)) => Ok((*a as f64) < *b),
            (Some(DataValue::Integer(a)), "<=", ExprValue::Number(b)) => Ok(*a as f64 <= *b),

            (Some(DataValue::Float(a)), "=", ExprValue::Number(b)) => {
                Ok((*a - b).abs() < f64::EPSILON)
            }
            (Some(DataValue::Float(a)), "!=", ExprValue::Number(b))
            | (Some(DataValue::Float(a)), "<>", ExprValue::Number(b)) => {
                Ok((*a - b).abs() >= f64::EPSILON)
            }
            (Some(DataValue::Float(a)), ">", ExprValue::Number(b)) => Ok(*a > *b),
            (Some(DataValue::Float(a)), ">=", ExprValue::Number(b)) => Ok(*a >= *b),
            (Some(DataValue::Float(a)), "<", ExprValue::Number(b)) => Ok(*a < *b),
            (Some(DataValue::Float(a)), "<=", ExprValue::Number(b)) => Ok(*a <= *b),

            // LIKE operator
            (Some(DataValue::String(text)), "LIKE", ExprValue::String(pattern)) => {
                let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
                let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;
                Ok(regex.is_match(text))
            }

            // IS NULL / IS NOT NULL
            (None, "IS", ExprValue::Null) | (Some(DataValue::Null), "IS", ExprValue::Null) => {
                Ok(true)
            }
            (Some(_), "IS", ExprValue::Null) => Ok(false),
            (None, "IS NOT", ExprValue::Null)
            | (Some(DataValue::Null), "IS NOT", ExprValue::Null) => Ok(false),
            (Some(_), "IS NOT", ExprValue::Null) => Ok(true),

            _ => Ok(false),
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

        let cell_value = self.table.get_value(row_index, col_index);

        for value_expr in values {
            let compare_value = self.extract_value(value_expr)?;
            let matches = match (cell_value, &compare_value) {
                (Some(DataValue::String(a)), ExprValue::String(b)) => a == b,
                (Some(DataValue::Integer(a)), ExprValue::Number(b)) => *a as f64 == *b,
                (Some(DataValue::Float(a)), ExprValue::Number(b)) => (*a - b).abs() < f64::EPSILON,
                _ => false,
            };

            if matches {
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

        let cell_value = self.table.get_value(row_index, col_index);
        let lower_value = self.extract_value(lower)?;
        let upper_value = self.extract_value(upper)?;

        match (cell_value, &lower_value, &upper_value) {
            (Some(DataValue::Integer(n)), ExprValue::Number(l), ExprValue::Number(u)) => {
                Ok(*n as f64 >= *l && *n as f64 <= *u)
            }
            (Some(DataValue::Float(n)), ExprValue::Number(l), ExprValue::Number(u)) => {
                Ok(*n >= *l && *n <= *u)
            }
            (Some(DataValue::String(s)), ExprValue::String(l), ExprValue::String(u)) => {
                Ok(s >= l && s <= u)
            }
            _ => Ok(false),
        }
    }

    fn evaluate_method_call(
        &self,
        object: &str,
        method: &str,
        args: &[SqlExpression],
        row_index: usize,
    ) -> Result<bool> {
        // Get column value
        let col_index = self
            .table
            .get_column_index(object)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", object))?;

        let cell_value = self.table.get_value(row_index, col_index);

        match method.to_lowercase().as_str() {
            "contains" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("Contains requires exactly 1 argument"));
                }
                let search_str = self.extract_string_value(&args[0])?;

                match cell_value {
                    Some(DataValue::String(s)) => Ok(s.contains(&search_str)),
                    _ => Ok(false),
                }
            }
            "startswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("StartsWith requires exactly 1 argument"));
                }
                let prefix = self.extract_string_value(&args[0])?;

                match cell_value {
                    Some(DataValue::String(s)) => Ok(s.starts_with(&prefix)),
                    _ => Ok(false),
                }
            }
            "endswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("EndsWith requires exactly 1 argument"));
                }
                let suffix = self.extract_string_value(&args[0])?;

                match cell_value {
                    Some(DataValue::String(s)) => Ok(s.ends_with(&suffix)),
                    _ => Ok(false),
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported method: {}", method)),
        }
    }

    fn extract_column_name(&self, expr: &SqlExpression) -> Result<String> {
        match expr {
            SqlExpression::Column(name) => Ok(name.clone()),
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
            SqlExpression::NumberLiteral(n) => {
                if let Ok(num) = n.parse::<f64>() {
                    Ok(ExprValue::Number(num))
                } else {
                    Ok(ExprValue::String(n.clone()))
                }
            }
            _ => Ok(ExprValue::Null),
        }
    }
}

enum ExprValue {
    String(String),
    Number(f64),
    Null,
}
