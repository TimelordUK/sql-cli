//! GROUP BY clause alias expansion transformer
//!
//! This transformer allows users to reference SELECT clause aliases in GROUP BY clauses
//! by automatically expanding those aliases to their full expressions.
//!
//! # Problem
//!
//! Users often want to group by a complex expression using its alias:
//! ```sql
//! SELECT id % 3 as grp, COUNT(*) FROM t GROUP BY grp
//! ```
//!
//! This fails because GROUP BY is evaluated before SELECT, so aliases don't exist yet.
//!
//! # Solution
//!
//! The transformer rewrites to:
//! ```sql
//! SELECT id % 3 as grp, COUNT(*) FROM t GROUP BY id % 3
//! ```
//!
//! # Algorithm
//!
//! 1. Extract all aliases from SELECT clause and their corresponding expressions
//! 2. Scan GROUP BY clause for column references
//! 3. If a column reference matches an alias name, replace it with the full expression
//! 4. Only expand simple column references (not qualified table.column)
//!
//! # Limitations
//!
//! - Only works for simple column aliases (not table.alias references)
//! - Aliases take precedence over actual column names if they conflict
//! - Complex expressions are duplicated (no common subexpression elimination)

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{CTEType, SelectItem, SelectStatement, SqlExpression, TableSource};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

/// Transformer that expands SELECT aliases in GROUP BY clauses
pub struct GroupByAliasExpander {
    /// Counter for tracking number of expansions
    expansions: usize,
}

impl GroupByAliasExpander {
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

    /// Expand aliases in a single GROUP BY expression
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
                            "Expanding alias '{}' in GROUP BY to: {:?}",
                            col_ref.name, alias_expr
                        );
                        return (alias_expr.clone(), true);
                    }
                }
                (expr.clone(), false)
            }

            // For all other expressions (functions, binary ops, etc.), return as-is
            // GROUP BY typically uses simple column references or the full expression
            _ => (expr.clone(), false),
        }
    }

    /// Expand aliases in GROUP BY clause
    fn expand_group_by(
        &mut self,
        group_by: &mut Vec<SqlExpression>,
        aliases: &HashMap<String, SqlExpression>,
    ) -> bool {
        let mut any_expanded = false;

        for expr in group_by.iter_mut() {
            let (new_expr, expanded) = Self::expand_expression(expr, aliases);
            if expanded {
                *expr = new_expr;
                any_expanded = true;
                self.expansions += 1;
            }
        }

        any_expanded
    }

    /// Transform a SelectStatement and recurse into nested SELECT statements
    /// (CTEs, FROM subqueries, set operations). Needed because GROUP BY alias
    /// expansion must happen in every scope where GROUP BY appears.
    #[allow(deprecated)]
    fn transform_statement(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        // Recurse into CTEs
        for cte in stmt.ctes.iter_mut() {
            if let CTEType::Standard(ref mut inner) = cte.cte_type {
                let taken = std::mem::take(inner);
                *inner = self.transform_statement(taken)?;
            }
        }

        // Recurse into FROM DerivedTable subqueries
        if let Some(TableSource::DerivedTable { query, .. }) = stmt.from_source.as_mut() {
            let taken = std::mem::take(query.as_mut());
            **query = self.transform_statement(taken)?;
        }

        // Recurse into legacy from_subquery
        if let Some(subq) = stmt.from_subquery.as_mut() {
            let taken = std::mem::take(subq.as_mut());
            **subq = self.transform_statement(taken)?;
        }

        // Recurse into set operation right-hand sides
        for (_op, rhs) in stmt.set_operations.iter_mut() {
            let taken = std::mem::take(rhs.as_mut());
            **rhs = self.transform_statement(taken)?;
        }

        // Apply GROUP BY alias expansion at this level
        self.apply_expansion(&mut stmt);

        Ok(stmt)
    }

    /// Apply GROUP BY alias expansion to a single statement (no recursion).
    fn apply_expansion(&mut self, stmt: &mut SelectStatement) {
        if stmt.group_by.is_none() {
            return;
        }

        let aliases = Self::extract_aliases(&stmt.select_items);
        if aliases.is_empty() {
            return;
        }

        if let Some(ref mut group_by) = stmt.group_by {
            let expanded = self.expand_group_by(group_by, &aliases);
            if expanded {
                debug!(
                    "Expanded {} alias reference(s) in GROUP BY clause",
                    self.expansions
                );
            }
        }
    }
}

impl Default for GroupByAliasExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for GroupByAliasExpander {
    fn name(&self) -> &str {
        "GroupByAliasExpander"
    }

    fn description(&self) -> &str {
        "Expands SELECT aliases in GROUP BY clauses to their full expressions"
    }

    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
        self.transform_statement(stmt)
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
    use crate::sql::parser::ast::{ColumnRef, QuoteStyle};

    #[test]
    fn test_extract_aliases() {
        let grp_expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef {
                name: "id".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })),
            op: "%".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
        };

        let select_items = vec![SelectItem::Expression {
            expr: grp_expr.clone(),
            alias: "grp".to_string(),
            leading_comments: vec![],
            trailing_comment: None,
        }];

        let aliases = GroupByAliasExpander::extract_aliases(&select_items);
        assert_eq!(aliases.len(), 1);
        assert!(aliases.contains_key("grp"));
    }

    #[test]
    fn test_expand_simple_column_reference() {
        let aliases = HashMap::from([(
            "grp".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("id".to_string()))),
                op: "%".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
            },
        )]);

        let expr = SqlExpression::Column(ColumnRef::unquoted("grp".to_string()));
        let (expanded, changed) = GroupByAliasExpander::expand_expression(&expr, &aliases);

        assert!(changed);
        assert!(matches!(expanded, SqlExpression::BinaryOp { .. }));
    }

    #[test]
    fn test_does_not_expand_full_expressions() {
        let aliases = HashMap::from([(
            "grp".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("id".to_string()))),
                op: "%".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
            },
        )]);

        // Full expression should not be expanded (it's not a simple column reference)
        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("id".to_string()))),
            op: "%".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
        };

        let (expanded, changed) = GroupByAliasExpander::expand_expression(&expr, &aliases);

        assert!(!changed);
        assert!(matches!(expanded, SqlExpression::BinaryOp { .. }));
    }

    #[test]
    fn test_transform_with_no_group_by() {
        let mut transformer = GroupByAliasExpander::new();
        let stmt = SelectStatement {
            group_by: None,
            ..Default::default()
        };

        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_expands_alias() {
        let mut transformer = GroupByAliasExpander::new();

        let grp_expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("id".to_string()))),
            op: "%".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
        };

        let stmt = SelectStatement {
            select_items: vec![SelectItem::Expression {
                expr: grp_expr.clone(),
                alias: "grp".to_string(),
                leading_comments: vec![],
                trailing_comment: None,
            }],
            group_by: Some(vec![SqlExpression::Column(ColumnRef::unquoted(
                "grp".to_string(),
            ))]),
            ..Default::default()
        };

        let result = transformer.transform(stmt).unwrap();

        // Check that GROUP BY was rewritten
        if let Some(group_by) = &result.group_by {
            assert_eq!(group_by.len(), 1);
            // Should now be the expanded expression (id % 3), not the column "grp"
            assert!(matches!(group_by[0], SqlExpression::BinaryOp { .. }));
        } else {
            panic!("Expected GROUP BY clause");
        }

        assert_eq!(transformer.expansions, 1);
    }

    #[test]
    fn test_does_not_expand_table_prefixed_columns() {
        let aliases = HashMap::from([(
            "grp".to_string(),
            SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column(ColumnRef::unquoted("id".to_string()))),
                op: "%".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
            },
        )]);

        // Column with table prefix should NOT be expanded
        let expr = SqlExpression::Column(ColumnRef {
            name: "grp".to_string(),
            quote_style: QuoteStyle::None,
            table_prefix: Some("t".to_string()),
        });

        let (expanded, changed) = GroupByAliasExpander::expand_expression(&expr, &aliases);

        assert!(!changed);
        assert!(matches!(expanded, SqlExpression::Column(_)));
    }

    #[test]
    fn test_multiple_aliases_in_group_by() {
        let mut transformer = GroupByAliasExpander::new();

        let year_expr = SqlExpression::FunctionCall {
            name: "YEAR".to_string(),
            args: vec![SqlExpression::Column(ColumnRef::unquoted(
                "date".to_string(),
            ))],
            distinct: false,
        };

        let month_expr = SqlExpression::FunctionCall {
            name: "MONTH".to_string(),
            args: vec![SqlExpression::Column(ColumnRef::unquoted(
                "date".to_string(),
            ))],
            distinct: false,
        };

        let stmt = SelectStatement {
            select_items: vec![
                SelectItem::Expression {
                    expr: year_expr.clone(),
                    alias: "yr".to_string(),
                    leading_comments: vec![],
                    trailing_comment: None,
                },
                SelectItem::Expression {
                    expr: month_expr.clone(),
                    alias: "mon".to_string(),
                    leading_comments: vec![],
                    trailing_comment: None,
                },
            ],
            group_by: Some(vec![
                SqlExpression::Column(ColumnRef::unquoted("yr".to_string())),
                SqlExpression::Column(ColumnRef::unquoted("mon".to_string())),
            ]),
            ..Default::default()
        };

        let result = transformer.transform(stmt).unwrap();

        // Check that both GROUP BY expressions were expanded
        if let Some(group_by) = &result.group_by {
            assert_eq!(group_by.len(), 2);
            assert!(matches!(group_by[0], SqlExpression::FunctionCall { .. }));
            assert!(matches!(group_by[1], SqlExpression::FunctionCall { .. }));
        } else {
            panic!("Expected GROUP BY clause");
        }

        assert_eq!(transformer.expansions, 2);
    }
}
