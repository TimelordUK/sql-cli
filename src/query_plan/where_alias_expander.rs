//! WHERE clause alias expansion transformer
//!
//! This transformer allows users to reference SELECT clause aliases in WHERE clauses
//! by automatically expanding those aliases to their full expressions.
//!
//! # Problem
//!
//! Users often want to reference complex SELECT expressions by their aliases in WHERE:
//! ```sql
//! SELECT a, a * 2 as double_a FROM t WHERE double_a > 10
//! ```
//!
//! This fails because WHERE is evaluated before SELECT, so aliases don't exist yet.
//!
//! # Solution
//!
//! The transformer rewrites to:
//! ```sql
//! SELECT a, a * 2 as double_a FROM t WHERE a * 2 > 10
//! ```
//!
//! # Algorithm
//!
//! 1. Extract all aliases from SELECT clause and their corresponding expressions
//! 2. Scan WHERE clause for column references
//! 3. If a column reference matches an alias name, replace it with the full expression
//! 4. Handle nested expressions (BinaryOp, CASE, etc.) recursively
//!
//! # Limitations
//!
//! - Only works for simple column aliases (not table.alias references)
//! - Aliases take precedence over actual column names if they conflict
//! - Complex expressions are duplicated (no common subexpression elimination)

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{SelectItem, SelectStatement, SqlExpression};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

/// Transformer that expands SELECT aliases in WHERE clauses
pub struct WhereAliasExpander {
    /// Counter for tracking number of expansions
    expansions: usize,
}

impl WhereAliasExpander {
    pub fn new() -> Self {
        Self { expansions: 0 }
    }

    /// Extract aliases from SELECT clause
    /// Returns a map of alias name -> expression
    fn extract_aliases(select_items: &[SelectItem]) -> HashMap<String, SqlExpression> {
        let mut aliases = HashMap::new();

        for item in select_items {
            if let SelectItem::Expression { expr, alias, .. } = item {
                if !alias.is_empty() {
                    aliases.insert(alias.clone(), expr.clone());
                    debug!("Found SELECT alias: {} -> {:?}", alias, expr);
                }
            }
        }

        aliases
    }

    /// Recursively expand aliases in an expression
    /// Returns the expanded expression and whether any expansion occurred
    fn expand_expression(
        expr: &SqlExpression,
        aliases: &HashMap<String, SqlExpression>,
    ) -> (SqlExpression, bool) {
        match expr {
            // Check if this column reference is actually an alias
            SqlExpression::Column(col_ref) => {
                // Only expand if it's a simple column (no table prefix)
                if col_ref.table_prefix.is_none() {
                    if let Some(alias_expr) = aliases.get(&col_ref.name) {
                        debug!(
                            "Expanding alias '{}' in WHERE to: {:?}",
                            col_ref.name, alias_expr
                        );
                        return (alias_expr.clone(), true);
                    }
                }
                (expr.clone(), false)
            }

            // Recursively expand binary operations
            SqlExpression::BinaryOp { left, op, right } => {
                let (new_left, left_expanded) = Self::expand_expression(left, aliases);
                let (new_right, right_expanded) = Self::expand_expression(right, aliases);
                let expanded = left_expanded || right_expanded;

                (
                    SqlExpression::BinaryOp {
                        left: Box::new(new_left),
                        op: op.clone(),
                        right: Box::new(new_right),
                    },
                    expanded,
                )
            }

            // Expand in NOT expressions
            SqlExpression::Not { expr: inner } => {
                let (new_expr, expanded) = Self::expand_expression(inner, aliases);
                (
                    SqlExpression::Not {
                        expr: Box::new(new_expr),
                    },
                    expanded,
                )
            }

            // Expand in function arguments
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => {
                let mut expanded = false;
                let new_args: Vec<SqlExpression> = args
                    .iter()
                    .map(|arg| {
                        let (new_arg, arg_expanded) = Self::expand_expression(arg, aliases);
                        expanded = expanded || arg_expanded;
                        new_arg
                    })
                    .collect();

                (
                    SqlExpression::FunctionCall {
                        name: name.clone(),
                        args: new_args,
                        distinct: *distinct,
                    },
                    expanded,
                )
            }

            // Expand in IN list expressions
            SqlExpression::InList {
                expr: inner,
                values,
            } => {
                let (new_expr, expr_expanded) = Self::expand_expression(inner, aliases);
                let mut expanded = expr_expanded;

                let new_values: Vec<SqlExpression> = values
                    .iter()
                    .map(|val| {
                        let (new_val, val_expanded) = Self::expand_expression(val, aliases);
                        expanded = expanded || val_expanded;
                        new_val
                    })
                    .collect();

                (
                    SqlExpression::InList {
                        expr: Box::new(new_expr),
                        values: new_values,
                    },
                    expanded,
                )
            }

            // Expand in NOT IN list expressions
            SqlExpression::NotInList {
                expr: inner,
                values,
            } => {
                let (new_expr, expr_expanded) = Self::expand_expression(inner, aliases);
                let mut expanded = expr_expanded;

                let new_values: Vec<SqlExpression> = values
                    .iter()
                    .map(|val| {
                        let (new_val, val_expanded) = Self::expand_expression(val, aliases);
                        expanded = expanded || val_expanded;
                        new_val
                    })
                    .collect();

                (
                    SqlExpression::NotInList {
                        expr: Box::new(new_expr),
                        values: new_values,
                    },
                    expanded,
                )
            }

            // Expand in BETWEEN expressions
            SqlExpression::Between { expr, lower, upper } => {
                let (new_expr, expr_expanded) = Self::expand_expression(expr, aliases);
                let (new_lower, lower_expanded) = Self::expand_expression(lower, aliases);
                let (new_upper, upper_expanded) = Self::expand_expression(upper, aliases);
                let expanded = expr_expanded || lower_expanded || upper_expanded;

                (
                    SqlExpression::Between {
                        expr: Box::new(new_expr),
                        lower: Box::new(new_lower),
                        upper: Box::new(new_upper),
                    },
                    expanded,
                )
            }

            // Expand in CASE expressions
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                let mut expanded = false;
                let new_branches: Vec<_> = when_branches
                    .iter()
                    .map(|branch| {
                        let (new_condition, cond_expanded) =
                            Self::expand_expression(&branch.condition, aliases);
                        let (new_result, result_expanded) =
                            Self::expand_expression(&branch.result, aliases);
                        expanded = expanded || cond_expanded || result_expanded;

                        crate::sql::parser::ast::WhenBranch {
                            condition: Box::new(new_condition),
                            result: Box::new(new_result),
                        }
                    })
                    .collect();

                let new_else = else_branch.as_ref().map(|e| {
                    let (new_e, else_expanded) = Self::expand_expression(e, aliases);
                    expanded = expanded || else_expanded;
                    Box::new(new_e)
                });

                (
                    SqlExpression::CaseExpression {
                        when_branches: new_branches,
                        else_branch: new_else,
                    },
                    expanded,
                )
            }

            // Expand in simple CASE expressions
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => {
                let (new_expr, expr_expanded) = Self::expand_expression(expr, aliases);
                let mut expanded = expr_expanded;

                let new_branches: Vec<_> = when_branches
                    .iter()
                    .map(|branch| {
                        let (new_value, value_expanded) =
                            Self::expand_expression(&branch.value, aliases);
                        let (new_result, result_expanded) =
                            Self::expand_expression(&branch.result, aliases);
                        expanded = expanded || value_expanded || result_expanded;

                        crate::sql::parser::ast::SimpleWhenBranch {
                            value: Box::new(new_value),
                            result: Box::new(new_result),
                        }
                    })
                    .collect();

                let new_else = else_branch.as_ref().map(|e| {
                    let (new_e, else_expanded) = Self::expand_expression(e, aliases);
                    expanded = expanded || else_expanded;
                    Box::new(new_e)
                });

                (
                    SqlExpression::SimpleCaseExpression {
                        expr: Box::new(new_expr),
                        when_branches: new_branches,
                        else_branch: new_else,
                    },
                    expanded,
                )
            }

            // Expand in method calls, e.g. `alias.Contains('x')`.
            // The receiver is a bare column-name string, so an alias can only be
            // substituted if it resolves to a simple (un-prefixed) column.
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => {
                let mut expanded = false;
                let new_args: Vec<SqlExpression> = args
                    .iter()
                    .map(|arg| {
                        let (new_arg, arg_expanded) = Self::expand_expression(arg, aliases);
                        expanded = expanded || arg_expanded;
                        new_arg
                    })
                    .collect();

                let mut new_object = object.clone();
                if let Some(SqlExpression::Column(col_ref)) = aliases.get(object) {
                    if col_ref.table_prefix.is_none() {
                        debug!(
                            "Expanding alias '{}' in WHERE method call to column '{}'",
                            object, col_ref.name
                        );
                        new_object = col_ref.name.clone();
                        expanded = true;
                    }
                }

                (
                    SqlExpression::MethodCall {
                        object: new_object,
                        method: method.clone(),
                        args: new_args,
                    },
                    expanded,
                )
            }

            // Expand in chained method calls, e.g. `(alias).Trim().Contains('x')`.
            // The base is itself an expression, so recurse into it normally.
            SqlExpression::ChainedMethodCall { base, method, args } => {
                let (new_base, base_expanded) = Self::expand_expression(base, aliases);
                let mut expanded = base_expanded;
                let new_args: Vec<SqlExpression> = args
                    .iter()
                    .map(|arg| {
                        let (new_arg, arg_expanded) = Self::expand_expression(arg, aliases);
                        expanded = expanded || arg_expanded;
                        new_arg
                    })
                    .collect();

                (
                    SqlExpression::ChainedMethodCall {
                        base: Box::new(new_base),
                        method: method.clone(),
                        args: new_args,
                    },
                    expanded,
                )
            }

            // For all other expressions, return as-is
            _ => (expr.clone(), false),
        }
    }

    /// Expand aliases in WHERE clause conditions
    fn expand_where_clause(
        &mut self,
        where_clause: &mut crate::sql::parser::ast::WhereClause,
        aliases: &HashMap<String, SqlExpression>,
    ) -> bool {
        let mut any_expanded = false;

        for condition in &mut where_clause.conditions {
            let (new_expr, expanded) = Self::expand_expression(&condition.expr, aliases);
            if expanded {
                condition.expr = new_expr;
                any_expanded = true;
                self.expansions += 1;
            }
        }

        any_expanded
    }
}

impl Default for WhereAliasExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for WhereAliasExpander {
    fn name(&self) -> &str {
        "WhereAliasExpander"
    }

    fn description(&self) -> &str {
        "Expands SELECT aliases in WHERE clauses to their full expressions"
    }

    fn transform(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        // Only process if there's a WHERE clause
        if stmt.where_clause.is_none() {
            return Ok(stmt);
        }

        // Step 1: Extract all aliases from SELECT clause
        let aliases = Self::extract_aliases(&stmt.select_items);

        if aliases.is_empty() {
            // No aliases to expand
            return Ok(stmt);
        }

        // Step 2: Expand aliases in WHERE clause
        if let Some(ref mut where_clause) = stmt.where_clause {
            let expanded = self.expand_where_clause(where_clause, &aliases);
            if expanded {
                debug!(
                    "Expanded {} alias reference(s) in WHERE clause",
                    self.expansions
                );
            }
        }

        Ok(stmt)
    }

    fn begin(&mut self) -> Result<()> {
        // Reset expansion counter for each query
        self.expansions = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::{ColumnRef, Condition, QuoteStyle, WhereClause};

    #[test]
    fn test_extract_aliases() {
        let double_a_expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef {
                name: "a".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })),
            op: "*".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
        };

        let select_items = vec![SelectItem::Expression {
            expr: double_a_expr.clone(),
            alias: "double_a".to_string(),
            leading_comments: vec![],
            trailing_comment: None,
        }];

        let aliases = WhereAliasExpander::extract_aliases(&select_items);
        assert_eq!(aliases.len(), 1);
        assert!(aliases.contains_key("double_a"));
    }

    #[test]
    fn test_expand_simple_column_reference() {
        let aliases = HashMap::from([(
            "double_a".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
                op: "*".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
            },
        )]);

        let expr = SqlExpression::Column(ColumnRef::unquoted("double_a".to_string()));
        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);

        assert!(changed);
        assert!(matches!(expanded, SqlExpression::BinaryOp { .. }));
    }

    #[test]
    fn test_expand_in_binary_op() {
        let aliases = HashMap::from([(
            "double_a".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
                op: "*".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
            },
        )]);

        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                "double_a".to_string(),
            ))),
            op: ">".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("10".to_string())),
        };

        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);

        assert!(changed);
        if let SqlExpression::BinaryOp { left, op, right } = expanded {
            assert_eq!(op, ">");
            assert!(matches!(left.as_ref(), SqlExpression::BinaryOp { .. }));
            assert!(matches!(
                right.as_ref(),
                SqlExpression::NumberLiteral(s) if s == "10"
            ));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_transform_with_no_where() {
        let mut transformer = WhereAliasExpander::new();
        let stmt = SelectStatement {
            where_clause: None,
            ..Default::default()
        };

        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_expands_alias() {
        let mut transformer = WhereAliasExpander::new();

        let double_a_expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
            op: "*".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
        };

        let stmt = SelectStatement {
            select_items: vec![SelectItem::Expression {
                expr: double_a_expr.clone(),
                alias: "double_a".to_string(),
                leading_comments: vec![],
                trailing_comment: None,
            }],
            where_clause: Some(WhereClause {
                conditions: vec![Condition {
                    expr: SqlExpression::BinaryOp {
                        left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                            "double_a".to_string(),
                        ))),
                        op: ">".to_string(),
                        right: Box::new(SqlExpression::NumberLiteral("10".to_string())),
                    },
                    connector: None,
                }],
            }),
            ..Default::default()
        };

        let result = transformer.transform(stmt).unwrap();

        // Check that WHERE was rewritten
        if let Some(where_clause) = &result.where_clause {
            if let SqlExpression::BinaryOp { left, .. } = &where_clause.conditions[0].expr {
                // Left side should now be the expanded expression (a * 2), not the column "double_a"
                assert!(matches!(left.as_ref(), SqlExpression::BinaryOp { .. }));
            } else {
                panic!("Expected BinaryOp in WHERE");
            }
        } else {
            panic!("Expected WHERE clause");
        }

        assert_eq!(transformer.expansions, 1);
    }

    #[test]
    fn test_expand_alias_in_method_call_receiver() {
        // `SELECT "name.common" as name ... WHERE name.Contains('x')`
        // The alias `name` resolves to the column `name.common`, so the method
        // call's receiver should be rewritten to that column name.
        let aliases = HashMap::from([(
            "name".to_string(),
            SqlExpression::Column(ColumnRef {
                name: "name.common".to_string(),
                quote_style: QuoteStyle::DoubleQuotes,
                table_prefix: None,
            }),
        )]);

        let expr = SqlExpression::MethodCall {
            object: "name".to_string(),
            method: "Contains".to_string(),
            args: vec![SqlExpression::StringLiteral("united".to_string())],
        };

        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);

        assert!(changed);
        match expanded {
            SqlExpression::MethodCall { object, method, .. } => {
                assert_eq!(object, "name.common");
                assert_eq!(method, "Contains");
            }
            other => panic!("Expected MethodCall, got {other:?}"),
        }
    }

    #[test]
    fn test_does_not_expand_method_call_for_nonalias() {
        // A method call whose receiver is a real column (not an alias) is untouched.
        let aliases = HashMap::from([(
            "name".to_string(),
            SqlExpression::Column(ColumnRef::unquoted("name.common".to_string())),
        )]);

        let expr = SqlExpression::MethodCall {
            object: "capital".to_string(),
            method: "Contains".to_string(),
            args: vec![SqlExpression::StringLiteral("x".to_string())],
        };

        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);

        assert!(!changed);
        assert!(matches!(
            expanded,
            SqlExpression::MethodCall { object, .. } if object == "capital"
        ));
    }

    #[test]
    fn test_does_not_expand_table_prefixed_columns() {
        let aliases = HashMap::from([(
            "double_a".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("a".to_string()))),
                op: "*".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
            },
        )]);

        // Column with table prefix should NOT be expanded
        let expr = SqlExpression::Column(ColumnRef {
            name: "double_a".to_string(),
            quote_style: QuoteStyle::None,
            table_prefix: Some("t".to_string()),
        });

        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);

        assert!(!changed);
        assert!(matches!(expanded, SqlExpression::Column(_)));
    }
}
