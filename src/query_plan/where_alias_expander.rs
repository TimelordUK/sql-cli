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
use crate::sql::parser::walk;
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

    /// Recursively expand aliases in an expression.
    /// Returns the expanded expression and whether any expansion occurred.
    ///
    /// Only two node kinds carry a real rule; everything else is pure structural
    /// recursion, so it is delegated to [`walk::map_children`]. That helper is
    /// exhaustive by construction and treats a nested subquery's `SelectStatement`
    /// as an **opaque scope boundary** — it descends into the same-scope operands
    /// of `InSubquery` / `NotInSubquery` / the tuple forms (fixing P11: an alias
    /// on the LHS of `x IN (SELECT ...)`) while never reaching into the subquery
    /// body, where an outer alias must not leak. Before this migration those
    /// variants fell into a `_ => (clone, false)` catch-all and the LHS operand
    /// was silently skipped along with the subquery.
    fn expand_expression(
        expr: &SqlExpression,
        aliases: &HashMap<String, SqlExpression>,
    ) -> (SqlExpression, bool) {
        match expr {
            // Rule 1: a bare (un-prefixed) column reference that names a SELECT
            // alias is replaced by that alias's expression.
            SqlExpression::Column(col_ref) => {
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

            // Rule 2: a method call's receiver is a bare column-name *string*, not
            // a child expression, so the walker can't reach it. Substitute the
            // receiver here when it names an alias resolving to a simple column,
            // then recurse into the args normally.
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

            // Everything else: structural recursion via the walker. It visits
            // same-scope children (including subquery LHS operands) and leaves
            // subquery bodies opaque.
            other => {
                let mut expanded = false;
                let new_expr = walk::map_children(other.clone(), |child| {
                    let (new_child, child_expanded) = Self::expand_expression(&child, aliases);
                    expanded = expanded || child_expanded;
                    new_child
                });
                (new_expr, expanded)
            }
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
    fn test_expands_alias_on_in_subquery_lhs_not_body() {
        // P11: `WHERE dbl IN (SELECT ...)` where `dbl` aliases `price * 2`.
        // The walker migration must expand the same-scope LHS operand while
        // leaving the subquery body (a different scope) untouched.
        let double = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("price".into()))),
            op: "*".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("2".to_string())),
        };
        let aliases = HashMap::from([("dbl".to_string(), double.clone())]);

        // The subquery body also references `dbl` — it must NOT be expanded,
        // because that name belongs to the subquery's own scope.
        let body = SelectStatement {
            where_clause: Some(WhereClause {
                conditions: vec![Condition {
                    expr: SqlExpression::Column(ColumnRef::unquoted("dbl".into())),
                    connector: None,
                }],
            }),
            ..Default::default()
        };

        let expr = SqlExpression::InSubquery {
            expr: Box::new(SqlExpression::Column(ColumnRef::unquoted("dbl".into()))),
            subquery: Box::new(body.clone()),
        };

        let (expanded, changed) = WhereAliasExpander::expand_expression(&expr, &aliases);
        assert!(changed, "the LHS alias should have been expanded");

        match expanded {
            SqlExpression::InSubquery { expr, subquery } => {
                // LHS expanded to the aliased expression.
                assert!(matches!(expr.as_ref(), SqlExpression::BinaryOp { .. }));
                // Subquery body left verbatim: still the bare `dbl` column.
                let inner = &subquery.where_clause.as_ref().unwrap().conditions[0].expr;
                assert!(
                    matches!(inner, SqlExpression::Column(c) if c.name == "dbl"),
                    "subquery body must not be touched (different scope), got {inner:?}"
                );
            }
            other => panic!("expected InSubquery, got {other:?}"),
        }
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
