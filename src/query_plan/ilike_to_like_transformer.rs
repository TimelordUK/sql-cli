//! ILIKE to LIKE transformer
//!
//! This transformer converts ILIKE (case-insensitive LIKE) operators to
//! standard LIKE operators by wrapping both sides in UPPER() function calls.
//!
//! # Problem
//!
//! PostgreSQL-style ILIKE is convenient for case-insensitive pattern matching:
//! ```sql
//! SELECT * FROM users WHERE email ILIKE '%@GMAIL.COM'
//! ```
//!
//! But not all SQL engines support ILIKE natively.
//!
//! # Solution
//!
//! Transform ILIKE to UPPER(col) LIKE UPPER(pattern):
//! ```sql
//! -- Input
//! WHERE email ILIKE '%@gmail.com'
//!
//! -- Output
//! WHERE UPPER(email) LIKE UPPER('%@gmail.com')
//! ```
//!
//! # Algorithm
//!
//! 1. Traverse the entire AST
//! 2. Find all BinaryOp expressions with "ILIKE" operator
//! 3. Replace with LIKE operator and wrap both sides in UPPER()
//! 4. Recursively handle all clauses (WHERE, SELECT, HAVING, etc.)

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{
    CTEType, Condition, OrderByItem, SelectItem, SelectStatement, SimpleWhenBranch, SqlExpression,
    WhenBranch, WhereClause, CTE,
};
use anyhow::Result;
use tracing::debug;

/// Transformer that converts ILIKE to UPPER() LIKE UPPER()
pub struct ILikeToLikeTransformer;

impl ILikeToLikeTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Transform an expression, converting ILIKE to LIKE with UPPER()
    fn transform_expression(&self, expr: SqlExpression) -> SqlExpression {
        match expr {
            // Core transformation: ILIKE -> UPPER() LIKE UPPER()
            SqlExpression::BinaryOp { left, op, right } if op == "ILIKE" => {
                debug!("Transforming ILIKE to UPPER() LIKE UPPER()");

                SqlExpression::BinaryOp {
                    left: Box::new(SqlExpression::FunctionCall {
                        name: "UPPER".to_string(),
                        args: vec![self.transform_expression(*left)],
                        distinct: false,
                    }),
                    op: "LIKE".to_string(),
                    right: Box::new(SqlExpression::FunctionCall {
                        name: "UPPER".to_string(),
                        args: vec![self.transform_expression(*right)],
                        distinct: false,
                    }),
                }
            }

            // Recursively transform nested expressions
            SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
                left: Box::new(self.transform_expression(*left)),
                op,
                right: Box::new(self.transform_expression(*right)),
            },

            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => SqlExpression::FunctionCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect(),
                distinct,
            },

            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => SqlExpression::CaseExpression {
                when_branches: when_branches
                    .into_iter()
                    .map(|branch| WhenBranch {
                        condition: Box::new(self.transform_expression(*branch.condition)),
                        result: Box::new(self.transform_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(self.transform_expression(*e))),
            },

            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => SqlExpression::SimpleCaseExpression {
                expr: Box::new(self.transform_expression(*expr)),
                when_branches: when_branches
                    .into_iter()
                    .map(|branch| SimpleWhenBranch {
                        value: Box::new(self.transform_expression(*branch.value)),
                        result: Box::new(self.transform_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(self.transform_expression(*e))),
            },

            SqlExpression::Between { expr, lower, upper } => SqlExpression::Between {
                expr: Box::new(self.transform_expression(*expr)),
                lower: Box::new(self.transform_expression(*lower)),
                upper: Box::new(self.transform_expression(*upper)),
            },

            SqlExpression::InList { expr, values } => SqlExpression::InList {
                expr: Box::new(self.transform_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|v| self.transform_expression(v))
                    .collect(),
            },

            SqlExpression::InSubquery { expr, subquery } => SqlExpression::InSubquery {
                expr: Box::new(self.transform_expression(*expr)),
                subquery: Box::new(self.transform_statement(*subquery)),
            },

            SqlExpression::NotInList { expr, values } => SqlExpression::NotInList {
                expr: Box::new(self.transform_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|v| self.transform_expression(v))
                    .collect(),
            },

            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => SqlExpression::MethodCall {
                object,
                method,
                args: args
                    .into_iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect(),
            },

            SqlExpression::ChainedMethodCall { base, method, args } => {
                SqlExpression::ChainedMethodCall {
                    base: Box::new(self.transform_expression(*base)),
                    method,
                    args: args
                        .into_iter()
                        .map(|arg| self.transform_expression(arg))
                        .collect(),
                }
            }

            SqlExpression::Not { expr } => SqlExpression::Not {
                expr: Box::new(self.transform_expression(*expr)),
            },

            SqlExpression::ScalarSubquery { query } => SqlExpression::ScalarSubquery {
                query: Box::new(self.transform_statement(*query)),
            },

            SqlExpression::NotInSubquery { expr, subquery } => SqlExpression::NotInSubquery {
                expr: Box::new(self.transform_expression(*expr)),
                subquery: Box::new(self.transform_statement(*subquery)),
            },

            SqlExpression::WindowFunction {
                name,
                args,
                window_spec,
            } => SqlExpression::WindowFunction {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect(),
                window_spec,
            },

            SqlExpression::Unnest { column, delimiter } => SqlExpression::Unnest {
                column: Box::new(self.transform_expression(*column)),
                delimiter,
            },

            // Literals and simple expressions don't need transformation
            _ => expr,
        }
    }

    /// Transform WHERE clause
    fn transform_where_clause(&self, where_clause: WhereClause) -> WhereClause {
        WhereClause {
            conditions: where_clause
                .conditions
                .into_iter()
                .map(|condition| Condition {
                    expr: self.transform_expression(condition.expr),
                    connector: condition.connector,
                })
                .collect(),
        }
    }

    /// Transform SELECT items
    fn transform_select_items(&self, items: Vec<SelectItem>) -> Vec<SelectItem> {
        items
            .into_iter()
            .map(|item| match item {
                SelectItem::Expression {
                    expr,
                    alias,
                    leading_comments,
                    trailing_comment,
                } => SelectItem::Expression {
                    expr: self.transform_expression(expr),
                    alias,
                    leading_comments,
                    trailing_comment,
                },
                SelectItem::Column {
                    column,
                    leading_comments,
                    trailing_comment,
                } => SelectItem::Column {
                    column,
                    leading_comments,
                    trailing_comment,
                },
                SelectItem::Star {
                    table_prefix,
                    leading_comments,
                    trailing_comment,
                } => SelectItem::Star {
                    table_prefix,
                    leading_comments,
                    trailing_comment,
                },
            })
            .collect()
    }

    /// Transform ORDER BY items
    fn transform_order_by(&self, items: Vec<OrderByItem>) -> Vec<OrderByItem> {
        items
            .into_iter()
            .map(|item| OrderByItem {
                expr: self.transform_expression(item.expr),
                direction: item.direction,
            })
            .collect()
    }

    /// Transform GROUP BY expressions
    fn transform_group_by(&self, exprs: Vec<SqlExpression>) -> Vec<SqlExpression> {
        exprs
            .into_iter()
            .map(|e| self.transform_expression(e))
            .collect()
    }

    /// Transform CTEs
    fn transform_ctes(&self, ctes: Vec<CTE>) -> Vec<CTE> {
        ctes.into_iter()
            .map(|cte| {
                let cte_type = match cte.cte_type {
                    CTEType::Standard(stmt) => CTEType::Standard(self.transform_statement(stmt)),
                    CTEType::Web(web_spec) => CTEType::Web(web_spec), // Don't transform WEB CTEs
                };
                CTE {
                    name: cte.name,
                    column_list: cte.column_list,
                    cte_type,
                }
            })
            .collect()
    }

    /// Transform a complete statement
    fn transform_statement(&self, mut stmt: SelectStatement) -> SelectStatement {
        // Transform CTEs first
        if !stmt.ctes.is_empty() {
            stmt.ctes = self.transform_ctes(stmt.ctes);
        }

        // Transform SELECT clause
        stmt.select_items = self.transform_select_items(stmt.select_items);

        // Transform WHERE clause
        if let Some(where_clause) = stmt.where_clause {
            stmt.where_clause = Some(self.transform_where_clause(where_clause));
        }

        // Transform HAVING clause
        if let Some(having) = stmt.having {
            stmt.having = Some(self.transform_expression(having));
        }

        // Transform ORDER BY clause
        if let Some(order_by) = stmt.order_by {
            stmt.order_by = Some(self.transform_order_by(order_by));
        }

        // Transform GROUP BY clause
        if let Some(group_by) = stmt.group_by {
            stmt.group_by = Some(self.transform_group_by(group_by));
        }

        // Transform QUALIFY clause
        if let Some(qualify) = stmt.qualify {
            stmt.qualify = Some(self.transform_expression(qualify));
        }

        stmt
    }
}

impl Default for ILikeToLikeTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for ILikeToLikeTransformer {
    fn name(&self) -> &str {
        "ILikeToLikeTransformer"
    }

    fn description(&self) -> &str {
        "Converts ILIKE (case-insensitive LIKE) to UPPER() LIKE UPPER() pattern"
    }

    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
        Ok(self.transform_statement(stmt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::{ColumnRef, QuoteStyle};

    #[test]
    fn test_ilike_simple() {
        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                "email".to_string(),
            ))),
            op: "ILIKE".to_string(),
            right: Box::new(SqlExpression::StringLiteral("%@gmail.com".to_string())),
        };

        let transformer = ILikeToLikeTransformer::new();
        let result = transformer.transform_expression(expr);

        // Should be UPPER(email) LIKE UPPER('%@gmail.com')
        match result {
            SqlExpression::BinaryOp { left, op, right } => {
                assert_eq!(op, "LIKE");

                // Check left is UPPER(email)
                match *left {
                    SqlExpression::FunctionCall { ref name, .. } => {
                        assert_eq!(name, "UPPER");
                    }
                    _ => panic!("Expected FunctionCall on left"),
                }

                // Check right is UPPER('%@gmail.com')
                match *right {
                    SqlExpression::FunctionCall { ref name, .. } => {
                        assert_eq!(name, "UPPER");
                    }
                    _ => panic!("Expected FunctionCall on right"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_ilike_in_where_clause() {
        let mut stmt = SelectStatement::default();

        stmt.where_clause = Some(WhereClause {
            conditions: vec![Condition {
                expr: SqlExpression::BinaryOp {
                    left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                        "name".to_string(),
                    ))),
                    op: "ILIKE".to_string(),
                    right: Box::new(SqlExpression::StringLiteral("%john%".to_string())),
                },
                connector: None,
            }],
        });

        let mut transformer = ILikeToLikeTransformer::new();
        let result = transformer.transform(stmt).unwrap();

        let where_clause = result.where_clause.unwrap();
        let condition = &where_clause.conditions[0];

        match &condition.expr {
            SqlExpression::BinaryOp { op, .. } => {
                assert_eq!(op, "LIKE");
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_like_unchanged() {
        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                "email".to_string(),
            ))),
            op: "LIKE".to_string(),
            right: Box::new(SqlExpression::StringLiteral("%@gmail.com".to_string())),
        };

        let transformer = ILikeToLikeTransformer::new();
        let result = transformer.transform_expression(expr.clone());

        // LIKE should remain unchanged
        match result {
            SqlExpression::BinaryOp { op, .. } => {
                assert_eq!(op, "LIKE");
                // Should NOT be wrapped in UPPER()
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
}
