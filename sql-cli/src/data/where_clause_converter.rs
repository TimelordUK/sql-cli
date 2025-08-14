use crate::sql::recursive_parser::{Condition, LogicalOp, SqlExpression, WhereClause};
use crate::sql::where_ast::{ComparisonOp, WhereExpr, WhereValue};
use anyhow::Result;

/// Converts recursive_parser's WhereClause to where_ast's WhereExpr
pub struct WhereClauseConverter;

impl WhereClauseConverter {
    /// Convert a WhereClause from recursive_parser to WhereExpr for where_evaluator
    pub fn convert(where_clause: &WhereClause) -> Result<WhereExpr> {
        if where_clause.conditions.is_empty() {
            return Err(anyhow::anyhow!("Empty WHERE clause"));
        }

        // Convert all conditions
        let mut expr = Self::convert_condition(&where_clause.conditions[0])?;

        // Chain conditions with AND/OR
        for i in 1..where_clause.conditions.len() {
            let next_expr = Self::convert_condition(&where_clause.conditions[i])?;

            // Use the connector from the previous condition
            if let Some(connector) = &where_clause.conditions[i - 1].connector {
                expr = match connector {
                    LogicalOp::And => WhereExpr::And(Box::new(expr), Box::new(next_expr)),
                    LogicalOp::Or => WhereExpr::Or(Box::new(expr), Box::new(next_expr)),
                };
            }
        }

        Ok(expr)
    }

    fn convert_condition(condition: &Condition) -> Result<WhereExpr> {
        Self::convert_expression(&condition.expr)
    }

    fn convert_expression(expr: &SqlExpression) -> Result<WhereExpr> {
        match expr {
            SqlExpression::BinaryOp { left, op, right } => Self::convert_binary_op(left, op, right),
            SqlExpression::InList { expr, values } => {
                let column = Self::extract_column_name(expr)?;
                let where_values = values
                    .iter()
                    .map(Self::convert_to_where_value)
                    .collect::<Result<Vec<_>>>()?;
                Ok(WhereExpr::In(column, where_values))
            }
            SqlExpression::NotInList { expr, values } => {
                let column = Self::extract_column_name(expr)?;
                let where_values = values
                    .iter()
                    .map(Self::convert_to_where_value)
                    .collect::<Result<Vec<_>>>()?;
                Ok(WhereExpr::NotIn(column, where_values))
            }
            SqlExpression::Between { expr, lower, upper } => {
                let column = Self::extract_column_name(expr)?;
                let lower_value = Self::convert_to_where_value(lower)?;
                let upper_value = Self::convert_to_where_value(upper)?;
                Ok(WhereExpr::Between(column, lower_value, upper_value))
            }
            SqlExpression::Not { expr } => {
                let inner = Self::convert_expression(expr)?;
                Ok(WhereExpr::Not(Box::new(inner)))
            }
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => Self::convert_method_call(object, method, args),
            _ => Err(anyhow::anyhow!("Unsupported expression type: {:?}", expr)),
        }
    }

    fn convert_binary_op(
        left: &SqlExpression,
        op: &str,
        right: &SqlExpression,
    ) -> Result<WhereExpr> {
        // For debugging - let's see what we're getting
        eprintln!("Converting binary op: {:?} {} {:?}", left, op, right);

        let column = Self::extract_column_name(left)?;
        let value = Self::convert_to_where_value(right)?;

        match op.to_uppercase().as_str() {
            "=" | "==" => Ok(WhereExpr::Equal(column, value)),
            "!=" | "<>" => Ok(WhereExpr::NotEqual(column, value)),
            ">" => Ok(WhereExpr::GreaterThan(column, value)),
            ">=" => Ok(WhereExpr::GreaterThanOrEqual(column, value)),
            "<" => Ok(WhereExpr::LessThan(column, value)),
            "<=" => Ok(WhereExpr::LessThanOrEqual(column, value)),
            "LIKE" => {
                if let WhereValue::String(pattern) = value {
                    Ok(WhereExpr::Like(column, pattern))
                } else {
                    Err(anyhow::anyhow!("LIKE requires string pattern"))
                }
            }
            "IS" => {
                // Handle IS NULL
                if matches!(value, WhereValue::Null) {
                    Ok(WhereExpr::IsNull(column))
                } else {
                    Err(anyhow::anyhow!("IS only supports NULL"))
                }
            }
            "IS NOT" => {
                // Handle IS NOT NULL
                if matches!(value, WhereValue::Null) {
                    Ok(WhereExpr::IsNotNull(column))
                } else {
                    Err(anyhow::anyhow!("IS NOT only supports NULL"))
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported operator: {}", op)),
        }
    }

    fn convert_method_call(
        object: &str,
        method: &str,
        args: &[SqlExpression],
    ) -> Result<WhereExpr> {
        // Handle string methods like column.Contains("value")
        match method.to_lowercase().as_str() {
            "contains" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("Contains requires exactly 1 argument"));
                }
                let value = Self::extract_string_value(&args[0])?;
                Ok(WhereExpr::Contains(object.to_string(), value))
            }
            "startswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("StartsWith requires exactly 1 argument"));
                }
                let value = Self::extract_string_value(&args[0])?;
                Ok(WhereExpr::StartsWith(object.to_string(), value))
            }
            "endswith" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("EndsWith requires exactly 1 argument"));
                }
                let value = Self::extract_string_value(&args[0])?;
                Ok(WhereExpr::EndsWith(object.to_string(), value))
            }
            "tolower" | "toupper" => {
                // These typically chain with an equals, e.g., column.ToLower() == "value"
                // For now, return an error - would need more context
                Err(anyhow::anyhow!(
                    "ToLower/ToUpper methods need comparison context"
                ))
            }
            _ => Err(anyhow::anyhow!("Unsupported method: {}", method)),
        }
    }

    fn extract_column_name(expr: &SqlExpression) -> Result<String> {
        match expr {
            SqlExpression::Column(name) => Ok(name.clone()),
            _ => Err(anyhow::anyhow!("Expected column name, got: {:?}", expr)),
        }
    }

    fn extract_string_value(expr: &SqlExpression) -> Result<String> {
        match expr {
            SqlExpression::StringLiteral(s) => Ok(s.clone()),
            _ => Err(anyhow::anyhow!("Expected string literal, got: {:?}", expr)),
        }
    }

    fn convert_to_where_value(expr: &SqlExpression) -> Result<WhereValue> {
        match expr {
            SqlExpression::StringLiteral(s) => Ok(WhereValue::String(s.clone())),
            SqlExpression::NumberLiteral(n) => {
                // Try to parse as number
                if let Ok(num) = n.parse::<f64>() {
                    Ok(WhereValue::Number(num))
                } else {
                    Ok(WhereValue::String(n.clone()))
                }
            }
            SqlExpression::Column(_) => {
                // Column references in values not supported yet
                Err(anyhow::anyhow!(
                    "Column references in WHERE values not yet supported"
                ))
            }
            _ => Ok(WhereValue::Null),
        }
    }
}
