use crate::data::datatable::{DataTable, DataValue};
use crate::sql::recursive_parser::{Condition, LogicalOp, SqlExpression, WhereClause};
use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use tracing::debug;

/// Evaluates WHERE clauses from recursive_parser directly against DataTable
pub struct RecursiveWhereEvaluator<'a> {
    table: &'a DataTable,
    case_insensitive: bool,
}

impl<'a> RecursiveWhereEvaluator<'a> {
    pub fn new(table: &'a DataTable) -> Self {
        Self {
            table,
            case_insensitive: false,
        }
    }

    pub fn with_case_insensitive(table: &'a DataTable, case_insensitive: bool) -> Self {
        Self {
            table,
            case_insensitive,
        }
    }

    /// Evaluate a WHERE clause for a specific row
    pub fn evaluate(&self, where_clause: &WhereClause, row_index: usize) -> Result<bool> {
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

        // Evaluate first condition
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate() - evaluating first condition for row {}",
                row_index
            );
        }
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

    fn evaluate_expression(&self, expr: &SqlExpression, row_index: usize) -> Result<bool> {
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
        &self,
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
                        let col_index = self
                            .table
                            .get_column_index(object)
                            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", object))?;

                        let value = self.table.get_value(row_index, col_index);
                        let length_value = match value {
                            Some(DataValue::String(s)) => Some(DataValue::Integer(s.len() as i64)),
                            Some(DataValue::InternedString(s)) => {
                                Some(DataValue::Integer(s.len() as i64))
                            }
                            Some(DataValue::Integer(n)) => {
                                Some(DataValue::Integer(n.to_string().len() as i64))
                            }
                            Some(DataValue::Float(f)) => {
                                Some(DataValue::Integer(f.to_string().len() as i64))
                            }
                            _ => Some(DataValue::Integer(0)),
                        };
                        (length_value, format!("{}.Length()", object))
                    }
                    "indexof" => {
                        if args.len() != 1 {
                            return Err(anyhow::anyhow!("IndexOf() requires exactly 1 argument"));
                        }
                        let search_str = self.extract_string_value(&args[0])?;
                        let col_index = self
                            .table
                            .get_column_index(object)
                            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", object))?;

                        let value = self.table.get_value(row_index, col_index);
                        let index_value = match value {
                            Some(DataValue::String(s)) => {
                                // Case-insensitive search by default, following Contains behavior
                                let pos = s
                                    .to_lowercase()
                                    .find(&search_str.to_lowercase())
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
                                Some(DataValue::Integer(pos))
                            }
                            Some(DataValue::InternedString(s)) => {
                                let pos = s
                                    .to_lowercase()
                                    .find(&search_str.to_lowercase())
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
                                Some(DataValue::Integer(pos))
                            }
                            Some(DataValue::Integer(n)) => {
                                let str_val = n.to_string();
                                let pos = str_val
                                    .find(&search_str)
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
                                Some(DataValue::Integer(pos))
                            }
                            Some(DataValue::Float(f)) => {
                                let str_val = f.to_string();
                                let pos = str_val
                                    .find(&search_str)
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
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
                        (index_value, format!("{}.IndexOf('{}')", object, search_str))
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

                let col_index = self
                    .table
                    .get_column_index(&column_name)
                    .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;

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

        // Perform comparison
        match (cell_value, op.to_uppercase().as_str(), &compare_value) {
            (Some(DataValue::String(ref a)), "=", ExprValue::String(b)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: String comparison '{}' = '{}' (case_insensitive={})",
                        a, b, self.case_insensitive
                    );
                }
                if self.case_insensitive {
                    Ok(a.to_lowercase() == b.to_lowercase())
                } else {
                    Ok(a == b)
                }
            }
            (Some(DataValue::InternedString(ref a)), "=", ExprValue::String(b)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: InternedString comparison '{}' = '{}' (case_insensitive={})",
                        a, b, self.case_insensitive
                    );
                }
                if self.case_insensitive {
                    Ok(a.to_lowercase() == b.to_lowercase())
                } else {
                    Ok(a.as_ref() == b)
                }
            }
            (Some(DataValue::String(ref a)), "!=", ExprValue::String(b))
            | (Some(DataValue::String(ref a)), "<>", ExprValue::String(b)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: String comparison '{}' != '{}' (case_insensitive={})",
                        a, b, self.case_insensitive
                    );
                }
                if self.case_insensitive {
                    Ok(a.to_lowercase() != b.to_lowercase())
                } else {
                    Ok(a != b)
                }
            }
            (Some(DataValue::InternedString(ref a)), "!=", ExprValue::String(b))
            | (Some(DataValue::InternedString(ref a)), "<>", ExprValue::String(b)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: InternedString comparison '{}' != '{}' (case_insensitive={})",
                        a, b, self.case_insensitive
                    );
                }
                if self.case_insensitive {
                    Ok(a.to_lowercase() != b.to_lowercase())
                } else {
                    Ok(a.as_ref() != b)
                }
            }
            (Some(DataValue::String(ref a)), ">", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() > b.to_lowercase())
                } else {
                    Ok(a > b)
                }
            }
            (Some(DataValue::InternedString(ref a)), ">", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() > b.to_lowercase())
                } else {
                    Ok(a.as_ref() > b)
                }
            }
            (Some(DataValue::String(ref a)), ">=", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() >= b.to_lowercase())
                } else {
                    Ok(a >= b)
                }
            }
            (Some(DataValue::InternedString(ref a)), ">=", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() >= b.to_lowercase())
                } else {
                    Ok(a.as_ref() >= b)
                }
            }
            (Some(DataValue::String(ref a)), "<", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() < b.to_lowercase())
                } else {
                    Ok(a < b)
                }
            }
            (Some(DataValue::InternedString(ref a)), "<", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() < b.to_lowercase())
                } else {
                    Ok(a.as_ref() < b)
                }
            }
            (Some(DataValue::String(ref a)), "<=", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() <= b.to_lowercase())
                } else {
                    Ok(a <= b)
                }
            }
            (Some(DataValue::InternedString(ref a)), "<=", ExprValue::String(b)) => {
                if self.case_insensitive {
                    Ok(a.to_lowercase() <= b.to_lowercase())
                } else {
                    Ok(a.as_ref() <= b)
                }
            }

            (Some(DataValue::Integer(a)), "=", ExprValue::Number(b)) => Ok(a as f64 == *b),
            (Some(DataValue::Integer(a)), "!=", ExprValue::Number(b))
            | (Some(DataValue::Integer(a)), "<>", ExprValue::Number(b)) => Ok(a as f64 != *b),
            (Some(DataValue::Integer(a)), ">", ExprValue::Number(b)) => Ok(a as f64 > *b),
            (Some(DataValue::Integer(a)), ">=", ExprValue::Number(b)) => Ok(a as f64 >= *b),
            (Some(DataValue::Integer(a)), "<", ExprValue::Number(b)) => Ok((a as f64) < *b),
            (Some(DataValue::Integer(a)), "<=", ExprValue::Number(b)) => Ok(a as f64 <= *b),

            (Some(DataValue::Float(a)), "=", ExprValue::Number(b)) => {
                Ok((a - b).abs() < f64::EPSILON)
            }
            (Some(DataValue::Float(a)), "!=", ExprValue::Number(b))
            | (Some(DataValue::Float(a)), "<>", ExprValue::Number(b)) => {
                Ok((a - b).abs() >= f64::EPSILON)
            }
            (Some(DataValue::Float(a)), ">", ExprValue::Number(b)) => Ok(a > *b),
            (Some(DataValue::Float(a)), ">=", ExprValue::Number(b)) => Ok(a >= *b),
            (Some(DataValue::Float(a)), "<", ExprValue::Number(b)) => Ok(a < *b),
            (Some(DataValue::Float(a)), "<=", ExprValue::Number(b)) => Ok(a <= *b),

            // LIKE operator
            (Some(DataValue::String(ref text)), "LIKE", ExprValue::String(pattern)) => {
                let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
                let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;
                Ok(regex.is_match(text))
            }
            (Some(DataValue::InternedString(ref text)), "LIKE", ExprValue::String(pattern)) => {
                let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
                let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;
                Ok(regex.is_match(text.as_ref()))
            }

            // IS NULL / IS NOT NULL
            (None, "IS", ExprValue::Null) | (Some(DataValue::Null), "IS", ExprValue::Null) => {
                Ok(true)
            }
            (Some(_), "IS", ExprValue::Null) => Ok(false),
            (None, "IS NOT", ExprValue::Null)
            | (Some(DataValue::Null), "IS NOT", ExprValue::Null) => Ok(false),
            (Some(_), "IS NOT", ExprValue::Null) => Ok(true),

            // DateTime comparisons
            (Some(DataValue::String(ref date_str)), op_str, ExprValue::DateTime(dt)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: DateTime comparison '{}' {} '{}' - attempting parse",
                        date_str,
                        op_str,
                        dt.format("%Y-%m-%d %H:%M:%S")
                    );
                }

                // Try to parse the string as a datetime - first try ISO 8601 with UTC
                if let Ok(parsed_dt) = date_str.parse::<DateTime<Utc>>() {
                    let result = match op_str {
                        "=" => parsed_dt == *dt,
                        "!=" | "<>" => parsed_dt != *dt,
                        ">" => parsed_dt > *dt,
                        ">=" => parsed_dt >= *dt,
                        "<" => parsed_dt < *dt,
                        "<=" => parsed_dt <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as UTC: '{}' {} '{}' = {}",
                            parsed_dt.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                }
                // Try ISO 8601 format without timezone (assume UTC)
                else if let Ok(parsed_dt) =
                    NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
                {
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as ISO 8601: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                }
                // Try standard datetime format
                else if let Ok(parsed_dt) =
                    NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                {
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as standard format: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"), op_str, dt.format("%Y-%m-%d %H:%M:%S"), result
                        );
                    }
                    Ok(result)
                }
                // Try date-only format
                else if let Ok(parsed_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    let parsed_dt =
                        NaiveDateTime::new(parsed_date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as date-only: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                } else {
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parse FAILED for '{}' - no matching format",
                            date_str
                        );
                    }
                    Ok(false)
                }
            }
            (Some(DataValue::InternedString(ref date_str)), op_str, ExprValue::DateTime(dt)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: DateTime comparison (interned) '{}' {} '{}' - attempting parse",
                        date_str,
                        op_str,
                        dt.format("%Y-%m-%d %H:%M:%S")
                    );
                }

                // Try to parse the string as a datetime - first try ISO 8601 with UTC
                if let Ok(parsed_dt) = date_str.parse::<DateTime<Utc>>() {
                    let result = match op_str {
                        "=" => parsed_dt == *dt,
                        "!=" | "<>" => parsed_dt != *dt,
                        ">" => parsed_dt > *dt,
                        ">=" => parsed_dt >= *dt,
                        "<" => parsed_dt < *dt,
                        "<=" => parsed_dt <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as UTC: '{}' {} '{}' = {}",
                            parsed_dt.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                }
                // Try ISO 8601 format without timezone (assume UTC)
                else if let Ok(parsed_dt) =
                    NaiveDateTime::parse_from_str(date_str.as_ref(), "%Y-%m-%dT%H:%M:%S")
                {
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as ISO 8601: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                }
                // Try standard datetime format
                else if let Ok(parsed_dt) =
                    NaiveDateTime::parse_from_str(date_str.as_ref(), "%Y-%m-%d %H:%M:%S")
                {
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as standard format: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"), op_str, dt.format("%Y-%m-%d %H:%M:%S"), result
                        );
                    }
                    Ok(result)
                }
                // Try date-only format
                else if let Ok(parsed_date) =
                    NaiveDate::parse_from_str(date_str.as_ref(), "%Y-%m-%d")
                {
                    let parsed_dt =
                        NaiveDateTime::new(parsed_date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parsed as date-only: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                } else {
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime parse FAILED for '{}' - no matching format",
                            date_str
                        );
                    }
                    Ok(false)
                }
            }

            // DateTime vs DateTime comparisons (when column is already parsed as DateTime)
            (Some(DataValue::DateTime(ref date_str)), op_str, ExprValue::DateTime(dt)) => {
                if row_index < 3 {
                    debug!(
                        "RecursiveWhereEvaluator: DateTime vs DateTime comparison '{}' {} '{}' - direct comparison",
                        date_str, op_str, dt.format("%Y-%m-%d %H:%M:%S")
                    );
                }

                // Parse the DataValue::DateTime string to DateTime<Utc>
                if let Ok(parsed_dt) = date_str.parse::<DateTime<Utc>>() {
                    let result = match op_str {
                        "=" => parsed_dt == *dt,
                        "!=" | "<>" => parsed_dt != *dt,
                        ">" => parsed_dt > *dt,
                        ">=" => parsed_dt >= *dt,
                        "<" => parsed_dt < *dt,
                        "<=" => parsed_dt <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime vs DateTime parsed successfully: '{}' {} '{}' = {}",
                            parsed_dt.format("%Y-%m-%d %H:%M:%S"), op_str, dt.format("%Y-%m-%d %H:%M:%S"), result
                        );
                    }
                    Ok(result)
                }
                // Try ISO 8601 format without timezone (assume UTC)
                else if let Ok(parsed_dt) =
                    NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
                {
                    let parsed_utc = Utc.from_utc_datetime(&parsed_dt);
                    let result = match op_str {
                        "=" => parsed_utc == *dt,
                        "!=" | "<>" => parsed_utc != *dt,
                        ">" => parsed_utc > *dt,
                        ">=" => parsed_utc >= *dt,
                        "<" => parsed_utc < *dt,
                        "<=" => parsed_utc <= *dt,
                        _ => false,
                    };
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime vs DateTime ISO 8601: '{}' {} '{}' = {}",
                            parsed_utc.format("%Y-%m-%d %H:%M:%S"),
                            op_str,
                            dt.format("%Y-%m-%d %H:%M:%S"),
                            result
                        );
                    }
                    Ok(result)
                } else {
                    if row_index < 3 {
                        debug!(
                            "RecursiveWhereEvaluator: DateTime vs DateTime parse FAILED for '{}' - no matching format",
                            date_str
                        );
                    }
                    Ok(false)
                }
            }

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

        let cell_value = self.table.get_value(row_index, col_index).cloned();

        for value_expr in values {
            let compare_value = self.extract_value(value_expr)?;
            let matches = match (cell_value.as_ref(), &compare_value) {
                (Some(DataValue::String(a)), ExprValue::String(b)) => {
                    if self.case_insensitive {
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: IN list string comparison '{}' in '{}' (case_insensitive={})", a, b, self.case_insensitive);
                        }
                        a.to_lowercase() == b.to_lowercase()
                    } else {
                        a == b
                    }
                }
                (Some(DataValue::InternedString(a)), ExprValue::String(b)) => {
                    if self.case_insensitive {
                        if row_index < 3 {
                            debug!("RecursiveWhereEvaluator: IN list interned string comparison '{}' in '{}' (case_insensitive={})", a, b, self.case_insensitive);
                        }
                        a.to_lowercase() == b.to_lowercase()
                    } else {
                        a.as_ref() == b
                    }
                }
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

        let cell_value = self.table.get_value(row_index, col_index).cloned();
        let lower_value = self.extract_value(lower)?;
        let upper_value = self.extract_value(upper)?;

        match (cell_value, &lower_value, &upper_value) {
            (Some(DataValue::Integer(n)), ExprValue::Number(l), ExprValue::Number(u)) => {
                Ok(n as f64 >= *l && n as f64 <= *u)
            }
            (Some(DataValue::Float(n)), ExprValue::Number(l), ExprValue::Number(u)) => {
                Ok(n >= *l && n <= *u)
            }
            (Some(DataValue::String(ref s)), ExprValue::String(l), ExprValue::String(u)) => {
                Ok(s >= l && s <= u)
            }
            (
                Some(DataValue::InternedString(ref s)),
                ExprValue::String(l),
                ExprValue::String(u),
            ) => Ok(s.as_ref() >= l && s.as_ref() <= u),
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
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_method_call - object='{}', method='{}', row={}",
                object, method, row_index
            );
        }

        // Get column value
        let col_index = self
            .table
            .get_column_index(object)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", object))?;

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
}

enum ExprValue {
    String(String),
    Number(f64),
    DateTime(DateTime<Utc>),
    Null,
}
