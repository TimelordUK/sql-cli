//! ORDER BY clause alias transformer
//!
//! This transformer rewrites ORDER BY clauses that reference aggregate functions
//! to use the aliases from the SELECT clause instead.
//!
//! # Problem
//!
//! Users often write queries like:
//! ```sql
//! SELECT region, SUM(sales_amount) AS total
//! FROM sales
//! GROUP BY region
//! ORDER BY SUM(sales_amount) DESC
//! ```
//!
//! This fails because the parser treats `SUM(sales_amount)` as a column name "SUM"
//! which doesn't exist.
//!
//! # Solution
//!
//! The transformer rewrites to:
//! ```sql
//! SELECT region, SUM(sales_amount) AS total
//! FROM sales
//! GROUP BY region
//! ORDER BY total DESC
//! ```
//!
//! # Algorithm
//!
//! 1. Find all aggregate functions in SELECT clause and their aliases
//! 2. Scan ORDER BY clause for column names that match aggregate patterns
//! 3. Replace with the corresponding alias from SELECT
//!
//! # Note
//!
//! This transformer works at the string level since ORDER BY currently only
//! supports column names, not full expressions in the AST.

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{
    CTEType, ColumnRef, SelectItem, SelectStatement, SqlExpression, TableSource,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Prefix used for ORDER BY columns promoted into SELECT so they survive
/// projection. Columns with this prefix are stripped from the final output
/// after ORDER BY runs.
pub const HIDDEN_ORDERBY_PREFIX: &str = "__hidden_orderby_";

/// Transformer that rewrites ORDER BY to use aggregate aliases
pub struct OrderByAliasTransformer {
    /// Counter for generating unique alias names if needed
    alias_counter: usize,
    /// Counter for ORDER BY columns promoted into SELECT
    hidden_counter: usize,
}

impl OrderByAliasTransformer {
    pub fn new() -> Self {
        Self {
            alias_counter: 0,
            hidden_counter: 0,
        }
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
        format!("__orderby_agg_{}", self.alias_counter)
    }

    /// Normalize an aggregate expression to match against ORDER BY column names
    ///
    /// ORDER BY might have strings like "SUM(sales_amount)" which the parser
    /// treats as a column name. We need to match these against actual aggregates.
    /// Returns uppercase version for case-insensitive matching.
    fn normalize_aggregate_expr(expr: &SqlExpression) -> String {
        match expr {
            SqlExpression::FunctionCall { name, args, .. } => {
                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        SqlExpression::Column(col_ref) => col_ref.name.to_uppercase(),
                        // Special case: COUNT('*') should match COUNT(*)
                        SqlExpression::StringLiteral(s) if s == "*" => "*".to_string(),
                        SqlExpression::StringLiteral(s) => format!("'{}'", s).to_uppercase(),
                        SqlExpression::NumberLiteral(n) => n.to_uppercase(),
                        _ => format!("{:?}", arg).to_uppercase(), // Fallback for complex args
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name.to_uppercase(), args_str)
            }
            _ => String::new(),
        }
    }

    /// Extract aggregate functions from SELECT clause and build mapping
    /// Returns: (normalized_expr -> alias, normalized_expr -> needs_alias_flag)
    fn build_aggregate_map(
        &mut self,
        select_items: &mut Vec<SelectItem>,
    ) -> HashMap<String, String> {
        let mut aggregate_map = HashMap::new();

        for item in select_items.iter_mut() {
            if let SelectItem::Expression { expr, alias, .. } = item {
                if Self::is_aggregate_function(expr) {
                    let normalized = Self::normalize_aggregate_expr(expr);

                    // If no alias exists, generate one
                    if alias.is_empty() {
                        *alias = self.generate_alias();
                        debug!(
                            "Generated alias '{}' for aggregate in ORDER BY: {}",
                            alias, normalized
                        );
                    }

                    debug!("Mapped aggregate '{}' to alias '{}'", normalized, alias);
                    aggregate_map.insert(normalized, alias.clone());
                }
            }
        }

        aggregate_map
    }

    /// Convert an expression to a string representation for pattern matching
    /// This is a simplified version that handles common cases
    fn expression_to_string(expr: &SqlExpression) -> String {
        match expr {
            SqlExpression::Column(col_ref) => col_ref.name.to_uppercase(),
            // Special case: StringLiteral("*") should render as * (for COUNT(*))
            SqlExpression::StringLiteral(s) if s == "*" => "*".to_string(),
            SqlExpression::StringLiteral(s) => format!("'{}'", s),
            SqlExpression::FunctionCall { name, args, .. } => {
                let args_str = args
                    .iter()
                    .map(|arg| Self::expression_to_string(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name.to_uppercase(), args_str)
            }
            _ => "expr".to_string(), // Fallback for complex expressions
        }
    }

    /// Check if an ORDER BY column matches an aggregate pattern
    /// Returns the normalized aggregate string if it matches
    fn extract_aggregate_from_order_column(column_name: &str) -> Option<String> {
        // Check if column name looks like an aggregate function call
        // e.g., "SUM(sales_amount)" or "COUNT(*)"
        let upper = column_name.to_uppercase();

        if (upper.starts_with("COUNT(") && upper.ends_with(')'))
            || (upper.starts_with("SUM(") && upper.ends_with(')'))
            || (upper.starts_with("AVG(") && upper.ends_with(')'))
            || (upper.starts_with("MIN(") && upper.ends_with(')'))
            || (upper.starts_with("MAX(") && upper.ends_with(')'))
            || (upper.starts_with("COUNT_DISTINCT(") && upper.ends_with(')'))
        {
            // Normalize to uppercase for matching
            Some(upper)
        } else {
            None
        }
    }
}

impl Default for OrderByAliasTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for OrderByAliasTransformer {
    fn name(&self) -> &str {
        "OrderByAliasTransformer"
    }

    fn description(&self) -> &str {
        "Rewrites ORDER BY aggregate expressions to use SELECT aliases"
    }

    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
        self.transform_statement(stmt)
    }
}

impl OrderByAliasTransformer {
    /// Transform a SelectStatement and recurse into nested SELECT statements
    /// (CTEs, FROM subqueries, set operations). Mirrors the recursion pattern
    /// used by HavingAliasTransformer and GroupByAliasExpander.
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

        // Apply ORDER BY alias rewrite at this level
        self.apply_rewrite(&mut stmt);

        Ok(stmt)
    }

    /// Apply ORDER BY alias rewriting to a single statement (no recursion).
    fn apply_rewrite(&mut self, stmt: &mut SelectStatement) {
        if stmt.order_by.is_none() {
            return;
        }

        // Step 1: Build mapping of aggregates to aliases
        let aggregate_map = self.build_aggregate_map(&mut stmt.select_items);

        // Step 2: Rewrite ORDER BY aggregate expressions to use SELECT aliases
        if !aggregate_map.is_empty() {
            if let Some(order_by) = stmt.order_by.as_mut() {
                let mut modified = false;

                for order_col in order_by.iter_mut() {
                    let expr_str = Self::expression_to_string(&order_col.expr);

                    if let Some(normalized) = Self::extract_aggregate_from_order_column(&expr_str) {
                        if let Some(alias) = aggregate_map.get(&normalized) {
                            debug!("Rewriting ORDER BY '{}' to use alias '{}'", expr_str, alias);
                            order_col.expr =
                                SqlExpression::Column(ColumnRef::unquoted(alias.clone()));
                            modified = true;
                        }
                    }
                }

                if modified {
                    debug!(
                        "Rewrote ORDER BY to use {} aggregate alias(es)",
                        aggregate_map.len()
                    );
                }
            }
        }

        // Step 3: Promote ORDER BY columns that aren't visible after projection.
        // Without this, `SELECT name AS results ... ORDER BY name` (or any
        // ORDER BY column not in SELECT) fails because projection narrows the
        // result columns before ORDER BY can resolve them.
        self.promote_hidden_order_by_columns(stmt);
    }

    /// Promote ORDER BY column references that aren't already in the SELECT
    /// output. Each missing column is appended to SELECT as a hidden
    /// expression; the ORDER BY ref is rewritten to use the hidden alias.
    /// Hidden columns get stripped from output by query_engine after sort.
    fn promote_hidden_order_by_columns(&mut self, stmt: &mut SelectStatement) {
        let order_by = match stmt.order_by.as_mut() {
            Some(o) if !o.is_empty() => o,
            _ => return,
        };

        // SELECT * (or similar) keeps all source columns visible — no promotion needed.
        if stmt
            .select_items
            .iter()
            .any(|i| matches!(i, SelectItem::Star { .. } | SelectItem::StarExclude { .. }))
        {
            return;
        }

        // Names exposed by SELECT — output names ORDER BY can already resolve.
        // For Column items the output name is the column's own name; for
        // Expression items it's the alias.
        let mut visible: HashSet<String> = HashSet::new();
        for item in stmt.select_items.iter() {
            match item {
                SelectItem::Column { column, .. } => {
                    visible.insert(column.name.to_lowercase());
                }
                SelectItem::Expression { alias, .. } if !alias.is_empty() => {
                    visible.insert(alias.to_lowercase());
                }
                _ => {}
            }
        }

        // Track column refs we've already promoted in this statement so two
        // ORDER BY items referencing the same column share one hidden alias.
        let mut promoted: HashMap<String, String> = HashMap::new();

        for order_col in order_by.iter_mut() {
            let column_ref = match &order_col.expr {
                SqlExpression::Column(c) => c.clone(),
                _ => continue, // Non-column ORDER BY expressions — out of scope here
            };

            let lookup_key = column_ref.name.to_lowercase();
            if visible.contains(&lookup_key) {
                continue;
            }

            let hidden_alias = if let Some(existing) = promoted.get(&lookup_key) {
                existing.clone()
            } else {
                self.hidden_counter += 1;
                let alias = format!("{}{}", HIDDEN_ORDERBY_PREFIX, self.hidden_counter);
                debug!(
                    "Promoting ORDER BY column '{}' as hidden SELECT item '{}'",
                    column_ref.name, alias
                );
                stmt.select_items.push(SelectItem::Expression {
                    expr: SqlExpression::Column(column_ref.clone()),
                    alias: alias.clone(),
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                });
                promoted.insert(lookup_key, alias.clone());
                alias
            };

            order_col.expr = SqlExpression::Column(ColumnRef::unquoted(hidden_alias));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::{ColumnRef, QuoteStyle, SortDirection};

    #[test]
    fn test_extract_aggregate_from_order_column() {
        assert_eq!(
            OrderByAliasTransformer::extract_aggregate_from_order_column("SUM(sales_amount)"),
            Some("SUM(SALES_AMOUNT)".to_string())
        );

        assert_eq!(
            OrderByAliasTransformer::extract_aggregate_from_order_column("COUNT(*)"),
            Some("COUNT(*)".to_string())
        );

        assert_eq!(
            OrderByAliasTransformer::extract_aggregate_from_order_column("region"),
            None
        );

        assert_eq!(
            OrderByAliasTransformer::extract_aggregate_from_order_column("total"),
            None
        );
    }

    #[test]
    fn test_normalize_aggregate_expr() {
        let expr = SqlExpression::FunctionCall {
            name: "SUM".to_string(),
            args: vec![SqlExpression::Column(ColumnRef {
                name: "sales_amount".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            })],
            distinct: false,
        };

        assert_eq!(
            OrderByAliasTransformer::normalize_aggregate_expr(&expr),
            "SUM(SALES_AMOUNT)"
        );
    }

    #[test]
    fn test_is_aggregate_function() {
        let sum_expr = SqlExpression::FunctionCall {
            name: "SUM".to_string(),
            args: vec![],
            distinct: false,
        };
        assert!(OrderByAliasTransformer::is_aggregate_function(&sum_expr));

        let upper_expr = SqlExpression::FunctionCall {
            name: "UPPER".to_string(),
            args: vec![],
            distinct: false,
        };
        assert!(!OrderByAliasTransformer::is_aggregate_function(&upper_expr));
    }
}
