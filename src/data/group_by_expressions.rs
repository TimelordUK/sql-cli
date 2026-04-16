// GROUP BY expression evaluation support

use anyhow::{anyhow, Result};
use fxhash::FxHashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::data::query_engine::QueryEngine;
use crate::sql::aggregates::contains_aggregate;
use crate::sql::parser::ast::{SelectItem, SqlExpression};
use tracing::debug;

/// Detailed phase information for GROUP BY operations
#[derive(Debug, Clone)]
pub struct GroupByPhaseInfo {
    pub total_rows: usize,
    pub num_groups: usize,
    pub num_expressions: usize,
    pub phase1_cardinality_estimation: Duration,
    pub phase2_key_building: Duration,
    pub phase2_expression_evaluation: Duration,
    pub phase3_dataview_creation: Duration,
    pub phase4_aggregation: Duration,
    pub phase4_having_evaluation: Duration,
    pub groups_filtered_by_having: usize,
    pub total_time: Duration,
}

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
    ) -> Result<(DataView, GroupByPhaseInfo)>;
}

impl GroupByExpressions for QueryEngine {
    fn group_by_expressions(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
    ) -> Result<FxHashMap<GroupKey, DataView>> {
        use std::time::Instant;
        let start = Instant::now();

        // Phase 1: Estimate cardinality for pre-sizing
        let phase1_start = Instant::now();
        let estimated_groups = self.estimate_group_cardinality(&view, group_by_exprs);
        let mut groups = FxHashMap::with_capacity_and_hasher(estimated_groups, Default::default());
        let mut group_rows: FxHashMap<GroupKey, Vec<usize>> =
            FxHashMap::with_capacity_and_hasher(estimated_groups, Default::default());
        let phase1_time = phase1_start.elapsed();
        debug!(
            "GROUP BY Phase 1 (cardinality estimation): {:?}, estimated {} groups",
            phase1_time, estimated_groups
        );

        // Phase 2: Process each visible row and build group keys
        let phase2_start = Instant::now();
        let visible_rows = view.get_visible_rows();
        let total_rows = visible_rows.len();
        debug!("GROUP BY Phase 2 starting: processing {} rows", total_rows);

        // OPTIMIZATION: Create evaluator once outside the loop!
        let mut evaluator = ArithmeticEvaluator::new(view.source());

        // OPTIMIZATION: Pre-allocate key_values vector with the right capacity
        let mut key_values = Vec::with_capacity(group_by_exprs.len());

        for row_idx in visible_rows.iter().copied() {
            // Clear and reuse the vector instead of allocating new one
            key_values.clear();

            // Evaluate GROUP BY expressions for this row
            for expr in group_by_exprs {
                let value = evaluator.evaluate(expr, row_idx).unwrap_or(DataValue::Null);
                key_values.push(value);
            }

            let key = GroupKey(key_values.clone()); // Need to clone here for the key
            group_rows.entry(key).or_default().push(row_idx);
        }
        let phase2_time = phase2_start.elapsed();
        debug!(
            "GROUP BY Phase 2 (expression evaluation & key building): {:?}, created {} unique keys",
            phase2_time,
            group_rows.len()
        );

        // Phase 3: Create DataViews for each group
        let phase3_start = Instant::now();
        for (key, rows) in group_rows {
            let mut group_view = DataView::new(view.source_arc());
            group_view = group_view.with_rows(rows);
            groups.insert(key, group_view);
        }
        let phase3_time = phase3_start.elapsed();
        debug!("GROUP BY Phase 3 (DataView creation): {:?}", phase3_time);

        let total_time = start.elapsed();
        debug!(
            "GROUP BY Total time: {:?} (P1: {:?}, P2: {:?}, P3: {:?})",
            total_time, phase1_time, phase2_time, phase3_time
        );

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
    ) -> Result<(DataView, GroupByPhaseInfo)> {
        use std::time::Instant;
        let start = Instant::now();

        debug!(
            "apply_group_by_expressions - grouping by {} expressions, {} select items",
            group_by_exprs.len(),
            select_items.len()
        );

        // Phase 1: Build groups by evaluating expressions for each row
        let phase1_start = Instant::now();
        let groups = self.group_by_expressions(view.clone(), group_by_exprs)?;
        let phase1_time = phase1_start.elapsed();
        debug!(
            "apply_group_by_expressions Phase 1 (group building): {:?}, created {} groups",
            phase1_time,
            groups.len()
        );

        // Create a result table for the grouped data
        let mut result_table = DataTable::new("grouped_result");

        // First, scan SELECT items to find non-aggregate expressions and their aliases
        let mut aggregate_columns = Vec::new();
        let mut non_aggregate_exprs = Vec::new();
        // Computed expressions that depend only on GROUP BY columns but aren't
        // direct matches (e.g. `user_id + 10` when GROUP BY is `user_id`).
        // These need a result column and per-group evaluation.
        let mut derived_grouped_exprs: Vec<(SqlExpression, String)> = Vec::new();
        let mut group_by_aliases = Vec::new();

        // Map GROUP BY expressions to their aliases from SELECT items
        for (i, group_expr) in group_by_exprs.iter().enumerate() {
            let mut found_alias = None;

            // Look for a matching SELECT item with an alias
            for item in select_items {
                if let SelectItem::Expression { expr, alias, .. } = item {
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
                SelectItem::Expression { expr, alias, .. } => {
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
                            // Walk the expression and confirm every column ref
                            // it touches is grouped (either appears in a GROUP
                            // BY expression directly, or is referenced by one).
                            // This permits computed expressions that depend
                            // only on grouping keys, e.g.:
                            //   SELECT user_id + 10 FROM t GROUP BY user_id
                            // which standard SQL allows.
                            let mut referenced_cols = Vec::new();
                            collect_column_refs(expr, &mut referenced_cols);
                            let all_grouped = !referenced_cols.is_empty()
                                && referenced_cols.iter().all(|col_name| {
                                    group_by_exprs
                                        .iter()
                                        .any(|ge| expression_references_column(ge, col_name))
                                });
                            if !all_grouped {
                                return Err(anyhow!(
                                    "Expression '{}' must appear in GROUP BY clause or be used in an aggregate function",
                                    alias
                                ));
                            }
                            // Add a column for the derived expression and remember
                            // it so Phase 2 evaluates it on each group's first row.
                            result_table.add_column(DataColumn::new(alias));
                            derived_grouped_exprs.push((expr.clone(), alias.clone()));
                        }
                    }
                }
                SelectItem::Column {
                    column: col_ref, ..
                } => {
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
                SelectItem::Star { .. } => {
                    // For GROUP BY queries, * includes GROUP BY columns
                    // Already handled by adding group_by_aliases columns
                }
                SelectItem::StarExclude { .. } => {
                    // StarExclude behaves like Star in GROUP BY context
                    // Expansion happens later in the query execution pipeline
                }
            }
        }

        // Phase 2: Process each group (aggregate computation)
        let phase2_start = Instant::now();
        let mut aggregation_time = std::time::Duration::ZERO;
        let mut having_time = std::time::Duration::ZERO;
        let mut groups_processed = 0;
        let mut groups_filtered = 0;

        for (group_key, group_view) in groups {
            let mut row_values = Vec::new();

            // Add GROUP BY expression values
            for value in &group_key.0 {
                row_values.push(value.clone());
            }

            // Calculate aggregate values for this group
            let agg_start = Instant::now();
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

            // Evaluate derived expressions that depend only on GROUP BY columns.
            // Same value for every row in the group, so evaluating on the first
            // row of the group is correct.
            for (expr, _alias) in &derived_grouped_exprs {
                let group_rows = group_view.get_visible_rows();
                let value = if !group_rows.is_empty() {
                    let mut evaluator = ArithmeticEvaluator::with_date_notation(
                        group_view.source(),
                        date_notation.clone(),
                    );
                    evaluator
                        .evaluate(expr, group_rows[0])
                        .unwrap_or(DataValue::Null)
                } else {
                    DataValue::Null
                };
                row_values.push(value);
            }

            aggregation_time += agg_start.elapsed();

            // Evaluate HAVING clause if present
            let having_start = Instant::now();
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
                    groups_filtered += 1;
                    having_time += having_start.elapsed();
                    continue;
                }
            }
            having_time += having_start.elapsed();

            groups_processed += 1;

            // Add the row to the result table
            result_table
                .add_row(DataRow::new(row_values))
                .map_err(|e| anyhow!("Failed to add grouped row: {}", e))?;
        }

        let phase2_time = phase2_start.elapsed();
        let total_time = start.elapsed();

        debug!(
            "apply_group_by_expressions Phase 2 (aggregation): {:?}",
            phase2_time
        );
        debug!("  - Aggregation time: {:?}", aggregation_time);
        debug!("  - HAVING evaluation time: {:?}", having_time);
        debug!(
            "  - Groups processed: {}, filtered by HAVING: {}",
            groups_processed, groups_filtered
        );
        debug!(
            "apply_group_by_expressions Total time: {:?} (P1: {:?}, P2: {:?})",
            total_time, phase1_time, phase2_time
        );

        let phase_info = GroupByPhaseInfo {
            total_rows: view.row_count(),
            num_groups: groups_processed,
            num_expressions: group_by_exprs.len(),
            phase1_cardinality_estimation: Duration::ZERO, // Not tracked separately in phase1
            phase2_key_building: phase1_time,              // This is actually the grouping phase
            phase2_expression_evaluation: Duration::ZERO,  // Included in phase2_key_building
            phase3_dataview_creation: Duration::ZERO,      // Included in phase1_time
            phase4_aggregation: aggregation_time,
            phase4_having_evaluation: having_time,
            groups_filtered_by_having: groups_filtered,
            total_time,
        };

        Ok((DataView::new(Arc::new(result_table)), phase_info))
    }
}

/// Check if two expressions are equivalent (for GROUP BY validation)
fn expressions_match(expr1: &SqlExpression, expr2: &SqlExpression) -> bool {
    // Simple equality check for now
    // Could be enhanced to handle semantic equivalence
    format!("{:?}", expr1) == format!("{:?}", expr2)
}

/// Recursively collect every column name referenced inside an expression.
/// Used to validate that computed SELECT items in a GROUP BY query depend
/// only on grouped columns.
fn collect_column_refs(expr: &SqlExpression, out: &mut Vec<String>) {
    match expr {
        SqlExpression::Column(col_ref) => out.push(col_ref.name.clone()),
        SqlExpression::BinaryOp { left, right, .. } => {
            collect_column_refs(left, out);
            collect_column_refs(right, out);
        }
        SqlExpression::FunctionCall { args, .. } => {
            for arg in args {
                collect_column_refs(arg, out);
            }
        }
        SqlExpression::Between { expr, lower, upper } => {
            collect_column_refs(expr, out);
            collect_column_refs(lower, out);
            collect_column_refs(upper, out);
        }
        SqlExpression::Not { expr } => collect_column_refs(expr, out),
        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            for branch in when_branches {
                collect_column_refs(&branch.condition, out);
                collect_column_refs(&branch.result, out);
            }
            if let Some(else_expr) = else_branch {
                collect_column_refs(else_expr, out);
            }
        }
        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => {
            collect_column_refs(expr, out);
            for branch in when_branches {
                collect_column_refs(&branch.value, out);
                collect_column_refs(&branch.result, out);
            }
            if let Some(else_expr) = else_branch {
                collect_column_refs(else_expr, out);
            }
        }
        // Literals, NULL, subqueries, etc. — no plain column refs to collect
        _ => {}
    }
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
