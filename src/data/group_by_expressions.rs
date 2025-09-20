// GROUP BY expression evaluation support

use anyhow::{anyhow, Result};
use fxhash::FxHashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::data::query_engine::QueryEngine;
use crate::sql::aggregates::contains_aggregate;
use crate::sql::parser::ast::{ColumnRef, SelectItem, SqlExpression};
use tracing::debug;

/// Key for grouping rows - contains the evaluated expression values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupKey(pub Vec<DataValue>);

/// Extension methods for QueryEngine to handle GROUP BY expressions
pub trait GroupByExpressions {
    /// Group rows by evaluating expressions for each row
    fn group_by_expressions(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
    ) -> Result<FxHashMap<GroupKey, DataView>>;

    /// Apply GROUP BY with expressions to the view
    fn apply_group_by_expressions(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
        select_items: &[SelectItem],
        having: Option<&SqlExpression>,
        _case_insensitive: bool,
        date_notation: String,
    ) -> Result<DataView>;
}

impl GroupByExpressions for QueryEngine {
    fn group_by_expressions(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
    ) -> Result<FxHashMap<GroupKey, DataView>> {
        // Estimate cardinality for pre-sizing
        let estimated_groups = self.estimate_group_cardinality(&view, group_by_exprs);
        let mut groups = FxHashMap::with_capacity_and_hasher(estimated_groups, Default::default());
        let mut group_rows: FxHashMap<GroupKey, Vec<usize>> =
            FxHashMap::with_capacity_and_hasher(estimated_groups, Default::default());

        // Process each visible row
        for row_idx in view.get_visible_rows().iter().copied() {
            // Evaluate GROUP BY expressions for this row
            let mut key_values = Vec::new();
            for expr in group_by_exprs {
                let mut evaluator = ArithmeticEvaluator::new(view.source());
                let value = evaluator.evaluate(expr, row_idx).unwrap_or(DataValue::Null);
                key_values.push(value);
            }

            let key = GroupKey(key_values);
            group_rows.entry(key).or_default().push(row_idx);
        }

        // Create DataViews for each group
        for (key, rows) in group_rows {
            let mut group_view = DataView::new(view.source_arc());
            group_view = group_view.with_rows(rows);
            groups.insert(key, group_view);
        }

        Ok(groups)
    }

    fn apply_group_by_expressions(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
        select_items: &[SelectItem],
        having: Option<&SqlExpression>,
        _case_insensitive: bool,
        date_notation: String,
    ) -> Result<DataView> {
        debug!(
            "apply_group_by_expressions - grouping by {} expressions",
            group_by_exprs.len()
        );

        // Build groups by evaluating expressions for each row
        let groups = self.group_by_expressions(view.clone(), group_by_exprs)?;
        debug!(
            "apply_group_by_expressions - created {} groups",
            groups.len()
        );

        // Create a result table for the grouped data
        let mut result_table = DataTable::new("grouped_result");

        // First, scan SELECT items to find non-aggregate expressions and their aliases
        let mut aggregate_columns = Vec::new();
        let mut non_aggregate_exprs = Vec::new();
        let mut group_by_aliases = Vec::new();

        // Map GROUP BY expressions to their aliases from SELECT items
        for (i, group_expr) in group_by_exprs.iter().enumerate() {
            let mut found_alias = None;

            // Look for a matching SELECT item with an alias
            for item in select_items {
                if let SelectItem::Expression { expr, alias } = item {
                    if !contains_aggregate(expr) && expressions_match(expr, group_expr) {
                        found_alias = Some(alias.clone());
                        break;
                    }
                }
            }

            // Use found alias or generate a default one
            let alias = found_alias.unwrap_or_else(|| match group_expr {
                SqlExpression::Column(column_ref) => column_ref.name.clone(),
                _ => format!("group_expr_{}", i + 1),
            });

            result_table.add_column(DataColumn::new(&alias));
            group_by_aliases.push(alias);
        }

        // Now process SELECT items to find aggregates and validate non-aggregates
        for item in select_items {
            match item {
                SelectItem::Expression { expr, alias } => {
                    if contains_aggregate(expr) {
                        // Aggregate expression
                        result_table.add_column(DataColumn::new(alias));
                        aggregate_columns.push((expr.clone(), alias.clone()));
                    } else {
                        // Non-aggregate expression - must match a GROUP BY expression
                        let mut found = false;
                        for group_expr in group_by_exprs {
                            if expressions_match(expr, group_expr) {
                                found = true;
                                non_aggregate_exprs.push((expr.clone(), alias.clone()));
                                break;
                            }
                        }
                        if !found {
                            // Check if it's a simple column that's in GROUP BY
                            if let SqlExpression::Column(col) = expr {
                                // Check if this column is referenced in any GROUP BY expression
                                let referenced = group_by_exprs
                                    .iter()
                                    .any(|ge| expression_references_column(ge, &col.name));
                                if !referenced {
                                    return Err(anyhow!(
                                        "Expression '{}' must appear in GROUP BY clause or be used in an aggregate function",
                                        alias
                                    ));
                                }
                            } else {
                                return Err(anyhow!(
                                    "Expression '{}' must appear in GROUP BY clause or be used in an aggregate function",
                                    alias
                                ));
                            }
                        }
                    }
                }
                SelectItem::Column(col_ref) => {
                    // Check if this column is in a GROUP BY expression
                    let in_group_by = group_by_exprs.iter().any(
                        |expr| matches!(expr, SqlExpression::Column(name) if name.name == col_ref.name),
                    );

                    if !in_group_by {
                        return Err(anyhow!(
                            "Column '{}' must appear in GROUP BY clause or be used in an aggregate function",
                            col_ref.name
                        ));
                    }
                }
                SelectItem::Star => {
                    // For GROUP BY queries, * includes GROUP BY columns
                    // Already handled by adding group_by_aliases columns
                }
            }
        }

        // Process each group
        for (group_key, group_view) in groups {
            let mut row_values = Vec::new();

            // Add GROUP BY expression values
            for value in &group_key.0 {
                row_values.push(value.clone());
            }

            // Calculate aggregate values for this group
            for (expr, _col_name) in &aggregate_columns {
                let group_rows = group_view.get_visible_rows();
                let mut evaluator = ArithmeticEvaluator::with_date_notation(
                    group_view.source(),
                    date_notation.clone(),
                )
                .with_visible_rows(group_rows.clone());

                let value = if group_view.row_count() > 0 && !group_rows.is_empty() {
                    evaluator
                        .evaluate(expr, group_rows[0])
                        .unwrap_or(DataValue::Null)
                } else {
                    DataValue::Null
                };

                row_values.push(value);
            }

            // Evaluate HAVING clause if present
            if let Some(having_expr) = having {
                // Create a temporary table with one row containing the group values
                let mut temp_table = DataTable::new("having_eval");

                // Add columns for GROUP BY expressions
                for alias in &group_by_aliases {
                    temp_table.add_column(DataColumn::new(alias));
                }

                // Add columns for aggregates
                for (_, alias) in &aggregate_columns {
                    temp_table.add_column(DataColumn::new(alias));
                }

                temp_table
                    .add_row(DataRow::new(row_values.clone()))
                    .map_err(|e| anyhow!("Failed to create temp table for HAVING: {}", e))?;

                // Evaluate HAVING expression
                let mut evaluator =
                    ArithmeticEvaluator::with_date_notation(&temp_table, date_notation.clone());
                let having_result = evaluator.evaluate(having_expr, 0)?;

                // Skip this group if HAVING condition is not met
                if !is_truthy(&having_result) {
                    continue;
                }
            }

            // Add the row to the result table
            result_table
                .add_row(DataRow::new(row_values))
                .map_err(|e| anyhow!("Failed to add grouped row: {}", e))?;
        }

        Ok(DataView::new(Arc::new(result_table)))
    }
}

/// Check if two expressions are equivalent (for GROUP BY validation)
fn expressions_match(expr1: &SqlExpression, expr2: &SqlExpression) -> bool {
    // Simple equality check for now
    // Could be enhanced to handle semantic equivalence
    format!("{:?}", expr1) == format!("{:?}", expr2)
}

/// Check if an expression references a column
fn expression_references_column(expr: &SqlExpression, column: &str) -> bool {
    match expr {
        SqlExpression::Column(name) => name == column,
        SqlExpression::BinaryOp { left, right, .. } => {
            expression_references_column(left, column)
                || expression_references_column(right, column)
        }
        SqlExpression::FunctionCall { args, .. } => args
            .iter()
            .any(|arg| expression_references_column(arg, column)),
        SqlExpression::Between { expr, lower, upper } => {
            expression_references_column(expr, column)
                || expression_references_column(lower, column)
                || expression_references_column(upper, column)
        }
        _ => false,
    }
}

/// Check if a DataValue is truthy (for HAVING evaluation)
fn is_truthy(value: &DataValue) -> bool {
    match value {
        DataValue::Boolean(b) => *b,
        DataValue::Integer(i) => *i != 0,
        DataValue::Float(f) => *f != 0.0 && !f.is_nan(),
        DataValue::Null => false,
        _ => true,
    }
}
