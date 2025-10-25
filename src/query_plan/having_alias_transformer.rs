//! HAVING clause auto-aliasing transformer
//!
//! This transformer automatically adds aliases to aggregate functions in SELECT
//! clauses and rewrites HAVING clauses to use those aliases instead of the
//! aggregate function expressions.
//!
//! # Problem
//!
//! Users often write queries like:
//! ```sql
//! SELECT region, COUNT(*) FROM sales GROUP BY region HAVING COUNT(*) > 5
//! ```
//!
//! This fails because the executor can't evaluate `COUNT(*)` in the HAVING
//! clause - it needs a column reference.
//!
//! # Solution
//!
//! The transformer rewrites to:
//! ```sql
//! SELECT region, COUNT(*) as __agg_1 FROM sales GROUP BY region HAVING __agg_1 > 5
//! ```
//!
//! # Algorithm
//!
//! 1. Find all aggregate functions in SELECT clause
//! 2. For each aggregate without an explicit alias, generate one (__agg_N)
//! 3. Scan HAVING clause for matching aggregate expressions
//! 4. Replace aggregate expressions with column references to the aliases

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{ColumnRef, QuoteStyle, SelectItem, SelectStatement, SqlExpression};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

/// Transformer that adds aliases to aggregates and rewrites HAVING clauses
pub struct HavingAliasTransformer {
    /// Counter for generating unique alias names
    alias_counter: usize,
}

impl HavingAliasTransformer {
    pub fn new() -> Self {
        Self { alias_counter: 0 }
    }

    /// Check if an expression is an aggregate function
    fn is_aggregate_function(expr: &SqlExpression) -> bool {
        matches!(
            expr,
            SqlExpression::FunctionCall { name, .. }
                if matches!(
                    name.to_uppercase().as_str(),
                    "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COUNT_DISTINCT"
                )
        )
    }

    /// Generate a unique alias name
    fn generate_alias(&mut self) -> String {
        self.alias_counter += 1;
        format!("__agg_{}", self.alias_counter)
    }

    /// Normalize an aggregate expression to a canonical form for comparison
    fn normalize_aggregate_expr(expr: &SqlExpression) -> String {
        match expr {
            SqlExpression::FunctionCall { name, args, .. } => {
                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        SqlExpression::Column(col_ref) => {
                            format!("{}", col_ref.name)
                        }
                        SqlExpression::StringLiteral(s) => format!("'{}'", s),
                        SqlExpression::NumberLiteral(n) => n.clone(),
                        _ => format!("{:?}", arg), // Fallback for complex args
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}({})", name.to_uppercase(), args_str)
            }
            _ => format!("{:?}", expr),
        }
    }

    /// Extract aggregate functions from SELECT clause and ensure they have aliases
    fn ensure_aggregate_aliases(
        &mut self,
        select_items: &mut Vec<SelectItem>,
    ) -> HashMap<String, String> {
        let mut aggregate_map = HashMap::new();

        for item in select_items.iter_mut() {
            if let SelectItem::Expression { expr, alias, .. } = item {
                if Self::is_aggregate_function(expr) {
                    // Generate alias if none exists
                    if alias.is_empty() {
                        *alias = self.generate_alias();
                        debug!(
                            "Generated alias '{}' for aggregate: {}",
                            alias,
                            Self::normalize_aggregate_expr(expr)
                        );
                    }

                    // Map normalized expression to alias
                    let normalized = Self::normalize_aggregate_expr(expr);
                    aggregate_map.insert(normalized, alias.clone());
                }
            }
        }

        aggregate_map
    }

    /// Rewrite a HAVING expression to use aliases instead of aggregates
    fn rewrite_having_expression(
        expr: &SqlExpression,
        aggregate_map: &HashMap<String, String>,
    ) -> SqlExpression {
        match expr {
            SqlExpression::FunctionCall { .. } if Self::is_aggregate_function(expr) => {
                let normalized = Self::normalize_aggregate_expr(expr);
                if let Some(alias) = aggregate_map.get(&normalized) {
                    debug!(
                        "Rewriting aggregate {} to column reference {}",
                        normalized, alias
                    );
                    SqlExpression::Column(ColumnRef {
                        name: alias.clone(),
                        quote_style: QuoteStyle::None,
                        table_prefix: None,
                    })
                } else {
                    // Aggregate not found in SELECT - leave as is (will fail later with clear error)
                    expr.clone()
                }
            }
            SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
                left: Box::new(Self::rewrite_having_expression(left, aggregate_map)),
                op: op.clone(),
                right: Box::new(Self::rewrite_having_expression(right, aggregate_map)),
            },
            // For other expressions, return as-is
            _ => expr.clone(),
        }
    }
}

impl Default for HavingAliasTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for HavingAliasTransformer {
    fn name(&self) -> &str {
        "HavingAliasTransformer"
    }

    fn description(&self) -> &str {
        "Adds aliases to aggregate functions and rewrites HAVING clauses to use them"
    }

    fn transform(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        // Only process if there's a HAVING clause
        if stmt.having.is_none() {
            return Ok(stmt);
        }

        // Step 1: Ensure all aggregates in SELECT have aliases and build mapping
        let aggregate_map = self.ensure_aggregate_aliases(&mut stmt.select_items);

        if aggregate_map.is_empty() {
            // No aggregates found, nothing to do
            return Ok(stmt);
        }

        // Step 2: Rewrite HAVING clause to use aliases
        if let Some(having_expr) = stmt.having.take() {
            let rewritten = Self::rewrite_having_expression(&having_expr, &aggregate_map);

            // Only set if something changed
            if format!("{:?}", having_expr) != format!("{:?}", rewritten) {
                debug!(
                    "Rewrote HAVING clause with {} aggregate alias(es)",
                    aggregate_map.len()
                );
                stmt.having = Some(rewritten);
            } else {
                stmt.having = Some(having_expr);
            }
        }

        Ok(stmt)
    }

    fn begin(&mut self) -> Result<()> {
        // Reset counter for each query
        self.alias_counter = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_aggregate_function() {
        let count_expr = SqlExpression::FunctionCall {
            name: "COUNT".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "*".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };
        assert!(HavingAliasTransformer::is_aggregate_function(&count_expr));

        let sum_expr = SqlExpression::FunctionCall {
            name: "SUM".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "amount".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };
        assert!(HavingAliasTransformer::is_aggregate_function(&sum_expr));

        let non_agg = SqlExpression::FunctionCall {
            name: "UPPER".to_string(),
            args: vec![],
            distinct: false,
        };
        assert!(!HavingAliasTransformer::is_aggregate_function(&non_agg));
    }

    #[test]
    fn test_normalize_aggregate_expr() {
        let count_star = SqlExpression::FunctionCall {
            name: "count".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "*".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };
        assert_eq!(
            HavingAliasTransformer::normalize_aggregate_expr(&count_star),
            "COUNT(*)"
        );

        let sum_amount = SqlExpression::FunctionCall {
            name: "SUM".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "amount".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };
        assert_eq!(
            HavingAliasTransformer::normalize_aggregate_expr(&sum_amount),
            "SUM(amount)"
        );
    }

    #[test]
    fn test_generate_alias() {
        let mut transformer = HavingAliasTransformer::new();
        assert_eq!(transformer.generate_alias(), "__agg_1");
        assert_eq!(transformer.generate_alias(), "__agg_2");
        assert_eq!(transformer.generate_alias(), "__agg_3");
    }

    #[test]
    fn test_transform_with_no_having() {
        let mut transformer = HavingAliasTransformer::new();
        let stmt = SelectStatement {
            having: None,
            ..Default::default()
        };

        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_adds_alias_and_rewrites_having() {
        let mut transformer = HavingAliasTransformer::new();

        let count_expr = SqlExpression::FunctionCall {
            name: "COUNT".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "*".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };

        let stmt = SelectStatement {
            select_items: vec![SelectItem::Expression {
                expr: count_expr.clone(),
                alias: String::new(), // No alias initially
                leading_comments: Vec::new(),
                trailing_comment: None,
            }],
            having: Some(SqlExpression::BinaryOp {
                left: Box::new(count_expr.clone()),
                op: ">".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("5".to_string())),
            }),
            ..Default::default()
        };

        let result = transformer.transform(stmt).unwrap();

        // Check that alias was added to SELECT
        if let SelectItem::Expression { alias, .. } = &result.select_items[0] {
            assert_eq!(alias, "__agg_1");
        } else {
            panic!("Expected Expression select item");
        }

        // Check that HAVING was rewritten to use alias
        if let Some(SqlExpression::BinaryOp { left, .. }) = &result.having {
            match left.as_ref() {
                SqlExpression::Column(col_ref) => {
                    assert_eq!(col_ref.name, "__agg_1");
                }
                _ => panic!("Expected column reference in HAVING, got: {:?}", left),
            }
        } else {
            panic!("Expected BinaryOp in HAVING");
        }
    }
}
