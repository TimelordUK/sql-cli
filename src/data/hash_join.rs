//! Hash join implementation for efficient JOIN operations

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::data::value_comparisons::compare_with_op;
use crate::sql::parser::ast::{JoinClause, JoinOperator, JoinType};
use crate::sql::recursive_parser::SqlExpression;

/// Normalize a value into a canonical form for join-key matching.
///
/// Equi-joins index keys in a `HashMap<DataValue, _>`, which keys on the exact
/// `DataValue` variant. That means `String("220")` and `Integer(220)` never
/// collide, so a join between a string column (e.g. a value pulled out of JSON
/// via `SUBSTR`) and an integer column silently produces no matches — even
/// though `WHERE a = b` would coerce and match them
/// (see `value_comparisons::compare_values`).
///
/// Two normalizations are applied:
///   - `InternedString` always collapses to a plain `String` — the same
///     logical type, just a different in-memory representation, so they must
///     always compare equal to a matching `String`.
///   - whole floats always collapse to integers (so `220.0` matches `220`,
///     even between two numeric columns).
///   - When `coerce_numeric` is set, numeric-looking strings also become
///     numbers (so `"220"` matches `220`).
///
/// `coerce_numeric` is decided per join site by [`join_key_coercion`]: it is
/// enabled only when the two columns hold different *kinds* of value (a string
/// column vs a numeric column). When both sides are strings — notably after
/// `TO_STRING(...)` on both — no string→number coercion happens, so `"007"` and
/// `"7"` stay distinct. The hash path must decide this per column (it never
/// sees the opposite key), unlike WHERE's pairwise comparison; the nested-loop
/// path defers to WHERE's comparator directly.
///
/// Note: like WHERE, parsing is not whitespace-trimmed (`" 220"` stays a
/// string).
fn canonical_join_key(value: &DataValue, coerce_numeric: bool) -> DataValue {
    match value {
        DataValue::String(s) => normalize_join_text(s, coerce_numeric),
        DataValue::InternedString(s) => normalize_join_text(s.as_str(), coerce_numeric),
        DataValue::Float(f) => fold_whole_float(*f),
        other => other.clone(),
    }
}

/// Canonicalize a textual join key: always returned as a plain `String` unless
/// numeric coercion is enabled and the text parses as a number.
fn normalize_join_text(s: &str, coerce_numeric: bool) -> DataValue {
    if coerce_numeric {
        if let Ok(i) = s.parse::<i64>() {
            return DataValue::Integer(i);
        }
        if let Ok(f) = s.parse::<f64>() {
            if f.is_finite() {
                return fold_whole_float(f);
            }
        }
    }
    DataValue::String(s.to_string())
}

/// The broad "kind" of a join-key value. String↔number coercion is applied
/// only when the two columns hold different kinds.
#[derive(PartialEq, Eq)]
enum KeyKind {
    Stringy,
    Numeric,
    Other,
}

fn value_kind(value: &DataValue) -> KeyKind {
    match value {
        DataValue::String(_) | DataValue::InternedString(_) => KeyKind::Stringy,
        DataValue::Integer(_) | DataValue::Float(_) => KeyKind::Numeric,
        _ => KeyKind::Other,
    }
}

/// The kind of a column, sampled from its first non-null value.
///
/// We sample actual values rather than the column's declared `data_type`
/// because a materialized temp table does not reliably carry accurate column
/// types (a column holding integers can still be typed as `String`/`Mixed`).
fn column_key_kind(table: &DataTable, col_idx: usize) -> Option<KeyKind> {
    table
        .rows
        .iter()
        .filter_map(|r| r.values.get(col_idx))
        .find(|v| !matches!(v, DataValue::Null))
        .map(value_kind)
}

/// Whether an equi-join between two columns should coerce string keys to
/// numbers. Enabled only when the columns hold different value kinds (e.g. a
/// string column vs a numeric column); two string columns join on exact text.
/// If a column's kind can't be determined (empty/all-null) we default to
/// coercing — the permissive behaviour that fixes the cross-type case.
fn join_key_coercion(
    left_table: &DataTable,
    left_col_idx: usize,
    right_table: &DataTable,
    right_col_idx: usize,
) -> bool {
    match (
        column_key_kind(left_table, left_col_idx),
        column_key_kind(right_table, right_col_idx),
    ) {
        (Some(l), Some(r)) => l != r,
        _ => true,
    }
}

/// Collapse a float with no fractional part to an integer so that `220.0`
/// hashes/compares equal to `220`.
fn fold_whole_float(f: f64) -> DataValue {
    if f.is_finite() && f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
        DataValue::Integer(f as i64)
    } else {
        DataValue::Float(f)
    }
}

/// Hash join executor for efficient JOIN operations
pub struct HashJoinExecutor {
    case_insensitive: bool,
}

impl HashJoinExecutor {
    pub fn new(case_insensitive: bool) -> Self {
        Self { case_insensitive }
    }

    /// Execute a single join operation
    pub fn execute_join(
        &self,
        left_table: Arc<DataTable>,
        join_clause: &JoinClause,
        right_table: Arc<DataTable>,
    ) -> Result<DataTable> {
        info!(
            "Executing {:?} JOIN: {} rows x {} rows with {} conditions",
            join_clause.join_type,
            left_table.row_count(),
            right_table.row_count(),
            join_clause.condition.conditions.len()
        );

        // For multiple conditions, we need to track all column indices
        // If any condition has a complex right expression, we must use nested loop
        let mut condition_indices = Vec::new();
        let mut all_equal = true;
        let mut has_complex_expr = false;

        for single_condition in &join_clause.condition.conditions {
            // Check if both sides are simple column references
            let left_col_name = Self::extract_simple_column_name(&single_condition.left_expr);
            let right_col_name = Self::extract_simple_column_name(&single_condition.right_expr);

            if left_col_name.is_none() || right_col_name.is_none() {
                // Complex expression on either side - must use nested loop with expression evaluation
                has_complex_expr = true;
                all_equal = false; // Force nested loop
                break;
            }

            let (left_col_idx, right_col_idx) = self.resolve_join_columns(
                &left_table,
                &right_table,
                &left_col_name.unwrap(),
                &right_col_name.unwrap(),
            )?;

            if single_condition.operator != JoinOperator::Equal {
                all_equal = false;
            }

            condition_indices.push((
                left_col_idx,
                right_col_idx,
                single_condition.operator.clone(),
            ));
        }

        // Choose join algorithm based on operators - use hash join only if:
        // 1. All conditions use equality
        // 2. No complex expressions (all simple column references)
        let use_hash_join = all_equal && !has_complex_expr;

        // Perform the appropriate join based on type and operator
        match join_clause.join_type {
            JoinType::Inner => {
                if use_hash_join && condition_indices.len() == 1 {
                    // Single equality condition with simple columns - use optimized hash join
                    let (left_col_idx, right_col_idx, _) = condition_indices[0];
                    let left_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].left_expr,
                    )
                    .expect("left_expr should be a simple column in hash join path");
                    let right_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].right_expr,
                    )
                    .expect("right_expr should be a simple column in hash join path");
                    self.hash_join_inner(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &left_col_name,
                        &right_col_name,
                        &join_clause.alias,
                    )
                } else {
                    // Multiple conditions, inequality, or expressions - use nested loop join
                    self.nested_loop_join_inner_multi(
                        left_table,
                        right_table,
                        &join_clause.condition.conditions,
                        &join_clause.alias,
                        true, // join-alias table is the `right_table` argument
                    )
                }
            }
            JoinType::Left => {
                if use_hash_join && condition_indices.len() == 1 {
                    // Single equality condition with simple columns - use optimized hash join
                    let (left_col_idx, right_col_idx, _) = condition_indices[0];
                    let left_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].left_expr,
                    )
                    .expect("left_expr should be a simple column in hash join path");
                    let right_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].right_expr,
                    )
                    .expect("right_expr should be a simple column in hash join path");
                    self.hash_join_left(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &left_col_name,
                        &right_col_name,
                        &join_clause.alias,
                    )
                } else {
                    // Multiple conditions, inequality, or expressions - use nested loop join
                    self.nested_loop_join_left_multi(
                        left_table,
                        right_table,
                        &join_clause.condition.conditions,
                        &join_clause.alias,
                        true, // join-alias table is the `right_table` argument
                    )
                }
            }
            JoinType::Right => {
                // Swap condition indices for right join
                let swapped_indices: Vec<(usize, usize, JoinOperator)> = condition_indices
                    .into_iter()
                    .map(|(l, r, op)| (r, l, self.reverse_operator(&op)))
                    .collect();

                if use_hash_join && swapped_indices.len() == 1 {
                    // Right join is just a left join with tables swapped
                    let (right_col_idx, left_col_idx, _) = swapped_indices[0];
                    let left_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].left_expr,
                    )
                    .expect("left_expr should be a simple column in hash join path");
                    let right_col_name = Self::extract_simple_column_name(
                        &join_clause.condition.conditions[0].right_expr,
                    )
                    .expect("right_expr should be a simple column in hash join path");
                    self.hash_join_left(
                        right_table,
                        left_table,
                        right_col_idx,
                        left_col_idx,
                        &right_col_name,
                        &left_col_name,
                        &join_clause.alias,
                    )
                } else {
                    // Right join is just a left join with tables swapped
                    // Pass the original conditions - nested_loop_join_left_multi will handle the swap.
                    // Tables are swapped, so the join-alias columns live in the `left_table`
                    // argument here, not the `right_table` one.
                    self.nested_loop_join_left_multi(
                        right_table,
                        left_table,
                        &join_clause.condition.conditions,
                        &join_clause.alias,
                        false, // swapped: join-alias table is the `left_table` argument
                    )
                }
            }
            JoinType::Cross => self.cross_join(left_table, right_table),
            JoinType::Full => {
                return Err(anyhow!("FULL OUTER JOIN not yet implemented"));
            }
        }
    }

    /// Extract column name from expression if it's a simple column reference
    /// Returns None if the expression is complex (function, operation, etc.)
    fn extract_simple_column_name(expr: &SqlExpression) -> Option<String> {
        match expr {
            SqlExpression::Column(col_ref) => {
                // Build the full column name including table prefix if present
                if let Some(table_prefix) = &col_ref.table_prefix {
                    Some(format!("{}.{}", table_prefix, col_ref.name))
                } else {
                    Some(col_ref.name.clone())
                }
            }
            _ => None, // Complex expression - cannot use fast path
        }
    }

    /// The table/alias qualifier of a simple column operand, if any
    /// (e.g. `b` for `b.price`). Non-column or unqualified operands yield `None`.
    fn expr_table_prefix(expr: &SqlExpression) -> Option<&str> {
        match expr {
            SqlExpression::Column(col) => col.table_prefix.as_deref(),
            _ => None,
        }
    }

    /// Decide which physical table a multi-condition ON operand should be
    /// evaluated against (P7). Historically these paths evaluated the syntactic
    /// left operand against the left table and the right operand against the
    /// right table, which silently reversed predicates written right-table-first
    /// (e.g. `b.price < a.price` became `a.price < b.price`). We instead route by
    /// the operand's alias qualifier: an operand whose prefix is the join alias
    /// belongs to the joined table; any other prefix belongs to the opposite
    /// table. `join_alias_is_right` says which argument holds the join-alias
    /// columns (the `right_table` arg for INNER/LEFT, the `left_table` arg for the
    /// swapped RIGHT path). Unqualified operands fall back to the syntactic
    /// position (`default_is_right`).
    fn operand_uses_right(
        &self,
        expr: &SqlExpression,
        join_alias: &Option<String>,
        join_alias_is_right: bool,
        default_is_right: bool,
    ) -> bool {
        if let (Some(prefix), Some(alias)) = (Self::expr_table_prefix(expr), join_alias.as_deref())
        {
            let matches_join_alias = if self.case_insensitive {
                prefix.eq_ignore_ascii_case(alias)
            } else {
                prefix == alias
            };
            // Operand belongs to the join-alias table when its prefix matches,
            // otherwise to the opposite side. Map that to right/left arg.
            return if matches_join_alias {
                join_alias_is_right
            } else {
                !join_alias_is_right
            };
        }
        default_is_right
    }

    /// Evaluate a single ON-condition operand against the table it actually
    /// belongs to (chosen via [`operand_uses_right`]), using the matching
    /// per-row index. This is what makes `b.price < a.price` evaluate correctly
    /// regardless of which side is written first (P7).
    #[allow(clippy::too_many_arguments)]
    fn eval_join_operand(
        &self,
        expr: &SqlExpression,
        left_evaluator: &mut ArithmeticEvaluator,
        right_evaluator: &mut ArithmeticEvaluator,
        left_row_idx: usize,
        right_row_idx: usize,
        join_alias: &Option<String>,
        join_alias_is_right: bool,
        default_is_right: bool,
    ) -> Result<DataValue> {
        if self.operand_uses_right(expr, join_alias, join_alias_is_right, default_is_right) {
            right_evaluator.evaluate(expr, right_row_idx)
        } else {
            left_evaluator.evaluate(expr, left_row_idx)
        }
    }

    /// Resolve which table each column belongs to in a join condition
    fn resolve_join_columns(
        &self,
        left_table: &DataTable,
        right_table: &DataTable,
        left_col_name: &str,
        right_col_name: &str,
    ) -> Result<(usize, usize)> {
        // Try to find the left column in left table, then right table
        let left_col_idx = if let Ok(idx) = self.find_column_index(left_table, left_col_name) {
            idx
        } else if let Ok(_idx) = self.find_column_index(right_table, left_col_name) {
            // The "left" column in the condition is actually from the right table
            // This means we need to swap the comparison
            return Err(anyhow!(
                "Column '{}' found in right table but specified as left operand. \
                Please rewrite the condition with columns in correct positions.",
                left_col_name
            ));
        } else {
            return Err(anyhow!(
                "Column '{}' not found in either table",
                left_col_name
            ));
        };

        // Try to find the right column in right table, then left table
        let right_col_idx = if let Ok(idx) = self.find_column_index(right_table, right_col_name) {
            idx
        } else if let Ok(_idx) = self.find_column_index(left_table, right_col_name) {
            // The "right" column in the condition is actually from the left table
            // This means we need to swap the comparison
            return Err(anyhow!(
                "Column '{}' found in left table but specified as right operand. \
                Please rewrite the condition with columns in correct positions.",
                right_col_name
            ));
        } else {
            return Err(anyhow!(
                "Column '{}' not found in either table",
                right_col_name
            ));
        };

        Ok((left_col_idx, right_col_idx))
    }

    /// Find column index in a table
    fn find_column_index(&self, table: &DataTable, col_name: &str) -> Result<usize> {
        // Handle table-qualified column names (e.g., "t1.id")
        let col_name = if let Some(dot_pos) = col_name.rfind('.') {
            &col_name[dot_pos + 1..]
        } else {
            col_name
        };

        debug!(
            "Looking for column '{}' in table with columns: {:?}",
            col_name,
            table.column_names()
        );

        table
            .columns
            .iter()
            .position(|col| {
                if self.case_insensitive {
                    col.name.to_lowercase() == col_name.to_lowercase()
                } else {
                    col.name == col_name
                }
            })
            .ok_or_else(|| anyhow!("Column '{}' not found in table", col_name))
    }

    /// Hash join implementation for INNER JOIN
    fn hash_join_inner(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        _left_col_name: &str,
        _right_col_name: &str,
        join_alias: &Option<String>,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        // Numeric coercion is enabled only when the two join columns have
        // different declared types (e.g. string vs integer). Decided per column
        // because the hash index canonicalizes each key without seeing its mate.
        let coerce = join_key_coercion(&left_table, left_col_idx, &right_table, right_col_idx);

        // Determine which table to use for building the hash index (prefer smaller)
        let (build_table, probe_table, build_col_idx, probe_col_idx, build_is_left) =
            if left_table.row_count() <= right_table.row_count() {
                (
                    left_table.clone(),
                    right_table.clone(),
                    left_col_idx,
                    right_col_idx,
                    true,
                )
            } else {
                (
                    right_table.clone(),
                    left_table.clone(),
                    right_col_idx,
                    left_col_idx,
                    false,
                )
            };

        debug!(
            "Building hash index on {} table ({} rows)",
            if build_is_left { "left" } else { "right" },
            build_table.row_count()
        );

        // Build hash index on the smaller table
        let mut hash_index: HashMap<DataValue, Vec<usize>> = HashMap::new();
        for (row_idx, row) in build_table.rows.iter().enumerate() {
            let key = canonical_join_key(&row.values[build_col_idx], coerce);
            hash_index.entry(key).or_default().push(row_idx);
        }

        debug!(
            "Hash index built with {} unique keys in {:?}",
            hash_index.len(),
            start.elapsed()
        );

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(), // Preserve qualified name
                source_table: col.source_table.clone(),     // Preserve source table
            });
        }

        // Add columns from right table
        for col in &right_table.columns {
            // Skip columns with duplicate names for now
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
                });
            } else {
                // If there's a name conflict, add with a suffix
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    // Use the join alias for the column name
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    // Fall back to _right suffix
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        debug!(
            "Joined table will have {} columns: {:?}",
            result.column_count(),
            result.column_names()
        );

        // Probe phase: iterate through the larger table
        let mut match_count = 0;
        for probe_row in &probe_table.rows {
            let probe_key = canonical_join_key(&probe_row.values[probe_col_idx], coerce);

            if let Some(matching_indices) = hash_index.get(&probe_key) {
                for &build_idx in matching_indices {
                    let build_row = &build_table.rows[build_idx];

                    // Create joined row based on which table was used for building
                    let mut joined_row = DataRow { values: Vec::new() };

                    if build_is_left {
                        // Build was left, probe was right
                        joined_row.values.extend_from_slice(&build_row.values);
                        joined_row.values.extend_from_slice(&probe_row.values);
                    } else {
                        // Build was right, probe was left
                        joined_row.values.extend_from_slice(&probe_row.values);
                        joined_row.values.extend_from_slice(&build_row.values);
                    }

                    result.add_row(joined_row);
                    match_count += 1;
                }
            }
        }

        // Debug: log the qualified names in the result table
        let qualified_cols: Vec<String> = result
            .columns
            .iter()
            .filter_map(|c| c.qualified_name.clone())
            .collect();

        info!(
            "INNER JOIN complete: {} matches found in {:?}. Result has {} columns ({} qualified: {:?})",
            match_count,
            start.elapsed(),
            result.columns.len(),
            qualified_cols.len(),
            qualified_cols
        );

        Ok(result)
    }

    /// Hash join implementation for LEFT OUTER JOIN
    fn hash_join_left(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        _left_col_name: &str,
        _right_col_name: &str,
        join_alias: &Option<String>,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        // Coerce string keys only when the join columns differ in type.
        let coerce = join_key_coercion(&left_table, left_col_idx, &right_table, right_col_idx);

        debug!(
            "Building hash index on right table ({} rows)",
            right_table.row_count()
        );

        // Build hash index on right table
        let mut hash_index: HashMap<DataValue, Vec<usize>> = HashMap::new();
        for (row_idx, row) in right_table.rows.iter().enumerate() {
            let key = canonical_join_key(&row.values[right_col_idx], coerce);
            hash_index.entry(key).or_default().push(row_idx);
        }

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(), // Preserve qualified name
                source_table: col.source_table.clone(),     // Preserve source table
            });
        }

        // Add columns from right table (all nullable for LEFT JOIN)
        for col in &right_table.columns {
            // Skip columns with duplicate names for now
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
                });
            } else {
                // If there's a name conflict, add with a suffix
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    // Use the join alias for the column name
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    // Fall back to _right suffix
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        debug!(
            "LEFT JOIN table will have {} columns: {:?}",
            result.column_count(),
            result.column_names()
        );

        // Probe phase: iterate through left table
        let mut match_count = 0;
        let mut null_count = 0;

        for left_row in &left_table.rows {
            let left_key = canonical_join_key(&left_row.values[left_col_idx], coerce);

            if let Some(matching_indices) = hash_index.get(&left_key) {
                // Found matches - emit joined rows
                for &right_idx in matching_indices {
                    let right_row = &right_table.rows[right_idx];

                    let mut joined_row = DataRow { values: Vec::new() };
                    joined_row.values.extend_from_slice(&left_row.values);
                    joined_row.values.extend_from_slice(&right_row.values);

                    result.add_row(joined_row);
                    match_count += 1;
                }
            } else {
                // No match - emit left row with NULLs for right columns
                let mut joined_row = DataRow { values: Vec::new() };
                joined_row.values.extend_from_slice(&left_row.values);

                // Add NULL values for all right table columns
                for _ in 0..right_table.column_count() {
                    joined_row.values.push(DataValue::Null);
                }

                result.add_row(joined_row);
                null_count += 1;
            }
        }

        // Debug: log the qualified names in the result table
        let qualified_cols: Vec<String> = result
            .columns
            .iter()
            .filter_map(|c| c.qualified_name.clone())
            .collect();

        info!(
            "LEFT JOIN complete: {} matches, {} nulls in {:?}. Result has {} columns ({} qualified: {:?})",
            match_count,
            null_count,
            start.elapsed(),
            result.columns.len(),
            qualified_cols.len(),
            qualified_cols
        );

        Ok(result)
    }

    /// Cross join implementation
    fn cross_join(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        // Check for potential memory explosion
        let result_rows = left_table.row_count() * right_table.row_count();
        if result_rows > 1_000_000 {
            return Err(anyhow!(
                "CROSS JOIN would produce {} rows, which exceeds the safety limit",
                result_rows
            ));
        }

        // Create result table
        let mut result = DataTable::new("joined");

        // Add columns from both tables
        for col in &left_table.columns {
            result.add_column(col.clone());
        }
        for col in &right_table.columns {
            result.add_column(col.clone());
        }

        // Generate Cartesian product
        for left_row in &left_table.rows {
            for right_row in &right_table.rows {
                let mut joined_row = DataRow { values: Vec::new() };
                joined_row.values.extend_from_slice(&left_row.values);
                joined_row.values.extend_from_slice(&right_row.values);
                result.add_row(joined_row);
            }
        }

        info!(
            "CROSS JOIN complete: {} rows in {:?}",
            result.row_count(),
            start.elapsed()
        );

        Ok(result)
    }

    /// Qualify column name to avoid conflicts
    fn qualify_column_name(
        &self,
        col_name: &str,
        table_side: &str,
        left_join_col: &str,
        right_join_col: &str,
    ) -> String {
        // Extract base column name (without table prefix)
        let base_name = if let Some(dot_pos) = col_name.rfind('.') {
            &col_name[dot_pos + 1..]
        } else {
            col_name
        };

        let left_base = if let Some(dot_pos) = left_join_col.rfind('.') {
            &left_join_col[dot_pos + 1..]
        } else {
            left_join_col
        };

        let right_base = if let Some(dot_pos) = right_join_col.rfind('.') {
            &right_join_col[dot_pos + 1..]
        } else {
            right_join_col
        };

        // If this column name appears in both join columns, qualify it
        if base_name == left_base || base_name == right_base {
            format!("{}_{}", table_side, base_name)
        } else {
            col_name.to_string()
        }
    }

    /// Reverse a join operator for right joins
    fn reverse_operator(&self, op: &JoinOperator) -> JoinOperator {
        match op {
            JoinOperator::Equal => JoinOperator::Equal,
            JoinOperator::NotEqual => JoinOperator::NotEqual,
            JoinOperator::LessThan => JoinOperator::GreaterThan,
            JoinOperator::GreaterThan => JoinOperator::LessThan,
            JoinOperator::LessThanOrEqual => JoinOperator::GreaterThanOrEqual,
            JoinOperator::GreaterThanOrEqual => JoinOperator::LessThanOrEqual,
        }
    }

    /// Compare two values based on the join operator.
    ///
    /// The nested-loop path has both values in hand, so it defers to the same
    /// pairwise comparator WHERE uses (`value_comparisons::compare_with_op`).
    /// That keeps JOIN equality identical to WHERE equality — including its
    /// type-aware coercion (`String` vs `Integer` coerces; `String` vs `String`
    /// compares as text) — so the nested-loop and hash paths agree.
    fn compare_values(&self, left: &DataValue, right: &DataValue, op: &JoinOperator) -> bool {
        let op_str = match op {
            JoinOperator::Equal => "=",
            JoinOperator::NotEqual => "!=",
            JoinOperator::LessThan => "<",
            JoinOperator::GreaterThan => ">",
            JoinOperator::LessThanOrEqual => "<=",
            JoinOperator::GreaterThanOrEqual => ">=",
        };
        compare_with_op(left, right, op_str, self.case_insensitive)
    }

    /// Nested loop join for INNER JOIN with inequality conditions
    fn nested_loop_join_inner(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        operator: &JoinOperator,
        join_alias: &Option<String>,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        info!(
            "Executing nested loop INNER JOIN with {:?} operator: {} x {} rows",
            operator,
            left_table.row_count(),
            right_table.row_count()
        );

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(), // Preserve qualified name
                source_table: col.source_table.clone(),     // Preserve source table
            });
        }

        // Add columns from right table
        for col in &right_table.columns {
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
                });
            } else {
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    // Use the join alias for the column name
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    // Fall back to _right suffix
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        // Nested loop join
        let mut match_count = 0;
        for left_row in &left_table.rows {
            let left_value = &left_row.values[left_col_idx];

            for right_row in &right_table.rows {
                let right_value = &right_row.values[right_col_idx];

                if self.compare_values(left_value, right_value, operator) {
                    let mut joined_row = DataRow { values: Vec::new() };
                    joined_row.values.extend_from_slice(&left_row.values);
                    joined_row.values.extend_from_slice(&right_row.values);
                    result.add_row(joined_row);
                    match_count += 1;
                }
            }
        }

        info!(
            "Nested loop INNER JOIN complete: {} matches found in {:?}",
            match_count,
            start.elapsed()
        );

        Ok(result)
    }

    /// Nested loop join for INNER JOIN with multiple conditions
    fn nested_loop_join_inner_multi(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        conditions: &[crate::sql::parser::ast::SingleJoinCondition],
        join_alias: &Option<String>,
        join_alias_is_right: bool,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        info!(
            "Executing nested loop INNER JOIN with {} conditions: {} x {} rows",
            conditions.len(),
            left_table.row_count(),
            right_table.row_count()
        );

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(),
                source_table: col.source_table.clone(),
            });
        }

        // Add columns from right table
        for col in &right_table.columns {
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(),
                    source_table: col.source_table.clone(),
                });
            } else {
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        // Create evaluators for both sides
        let mut left_evaluator = ArithmeticEvaluator::new(&left_table);
        let mut right_evaluator = ArithmeticEvaluator::new(&right_table);

        // Nested loop join with multiple conditions
        let mut match_count = 0;
        for (left_row_idx, left_row) in left_table.rows.iter().enumerate() {
            for (right_row_idx, right_row) in right_table.rows.iter().enumerate() {
                // Check all conditions - all must be true for a match
                let mut all_conditions_met = true;
                for condition in conditions.iter() {
                    // Route each operand to its owning table by alias qualifier
                    // rather than syntactic position (P7). `left_expr` defaults to
                    // the left table, `right_expr` to the right table, but an
                    // explicit alias overrides that default.
                    let left_val = self.eval_join_operand(
                        &condition.left_expr,
                        &mut left_evaluator,
                        &mut right_evaluator,
                        left_row_idx,
                        right_row_idx,
                        join_alias,
                        join_alias_is_right,
                        false, // left_expr defaults to the left table
                    );
                    let left_value = match left_val {
                        Ok(val) => val,
                        Err(_) => {
                            all_conditions_met = false;
                            break;
                        }
                    };

                    let right_val = self.eval_join_operand(
                        &condition.right_expr,
                        &mut left_evaluator,
                        &mut right_evaluator,
                        left_row_idx,
                        right_row_idx,
                        join_alias,
                        join_alias_is_right,
                        true, // right_expr defaults to the right table
                    );
                    let right_value = match right_val {
                        Ok(val) => val,
                        Err(_) => {
                            all_conditions_met = false;
                            break;
                        }
                    };

                    if !self.compare_values(&left_value, &right_value, &condition.operator) {
                        all_conditions_met = false;
                        break;
                    }
                }

                if all_conditions_met {
                    let mut joined_row = DataRow { values: Vec::new() };
                    joined_row.values.extend_from_slice(&left_row.values);
                    joined_row.values.extend_from_slice(&right_row.values);
                    result.add_row(joined_row);
                    match_count += 1;
                }
            }
        }

        info!(
            "Nested loop INNER JOIN complete: {} matches found in {:?}",
            match_count,
            start.elapsed()
        );

        Ok(result)
    }

    /// Nested loop join for LEFT JOIN with multiple conditions
    fn nested_loop_join_left_multi(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        conditions: &[crate::sql::parser::ast::SingleJoinCondition],
        join_alias: &Option<String>,
        join_alias_is_right: bool,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        info!(
            "Executing nested loop LEFT JOIN with {} conditions: {} x {} rows",
            conditions.len(),
            left_table.row_count(),
            right_table.row_count()
        );

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(),
                source_table: col.source_table.clone(),
            });
        }

        // Add columns from right table (all nullable for LEFT JOIN)
        for col in &right_table.columns {
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(),
                    source_table: col.source_table.clone(),
                });
            } else {
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        // Create evaluators for both sides
        let mut left_evaluator = ArithmeticEvaluator::new(&left_table);
        let mut right_evaluator = ArithmeticEvaluator::new(&right_table);

        // Nested loop join with multiple conditions
        let mut match_count = 0;
        let mut null_count = 0;

        for (left_row_idx, left_row) in left_table.rows.iter().enumerate() {
            let mut found_match = false;

            for (right_row_idx, right_row) in right_table.rows.iter().enumerate() {
                // Check all conditions - all must be true for a match
                let mut all_conditions_met = true;
                for condition in conditions.iter() {
                    // Route each operand to its owning table by alias qualifier
                    // rather than syntactic position (P7).
                    let left_val = self.eval_join_operand(
                        &condition.left_expr,
                        &mut left_evaluator,
                        &mut right_evaluator,
                        left_row_idx,
                        right_row_idx,
                        join_alias,
                        join_alias_is_right,
                        false, // left_expr defaults to the left table
                    );
                    let left_value = match left_val {
                        Ok(val) => val,
                        Err(_) => {
                            all_conditions_met = false;
                            break;
                        }
                    };

                    let right_val = self.eval_join_operand(
                        &condition.right_expr,
                        &mut left_evaluator,
                        &mut right_evaluator,
                        left_row_idx,
                        right_row_idx,
                        join_alias,
                        join_alias_is_right,
                        true, // right_expr defaults to the right table
                    );
                    let right_value = match right_val {
                        Ok(val) => val,
                        Err(_) => {
                            all_conditions_met = false;
                            break;
                        }
                    };

                    if !self.compare_values(&left_value, &right_value, &condition.operator) {
                        all_conditions_met = false;
                        break;
                    }
                }

                if all_conditions_met {
                    let mut joined_row = DataRow { values: Vec::new() };
                    joined_row.values.extend_from_slice(&left_row.values);
                    joined_row.values.extend_from_slice(&right_row.values);
                    result.add_row(joined_row);
                    match_count += 1;
                    found_match = true;
                }
            }

            // If no match found, emit left row with NULLs for right columns
            if !found_match {
                let mut joined_row = DataRow { values: Vec::new() };
                joined_row.values.extend_from_slice(&left_row.values);
                for _ in 0..right_table.column_count() {
                    joined_row.values.push(DataValue::Null);
                }
                result.add_row(joined_row);
                null_count += 1;
            }
        }

        info!(
            "Nested loop LEFT JOIN complete: {} matches, {} nulls in {:?}",
            match_count,
            null_count,
            start.elapsed()
        );

        Ok(result)
    }

    /// Nested loop join for LEFT JOIN with inequality conditions
    fn nested_loop_join_left(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        operator: &JoinOperator,
        join_alias: &Option<String>,
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        info!(
            "Executing nested loop LEFT JOIN with {:?} operator: {} x {} rows",
            operator,
            left_table.row_count(),
            right_table.row_count()
        );

        // Create result table with columns from both tables
        let mut result = DataTable::new("joined");

        // Add columns from left table
        for col in &left_table.columns {
            result.add_column(DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(), // Preserve qualified name
                source_table: col.source_table.clone(),     // Preserve source table
            });
        }

        // Add columns from right table (all nullable for LEFT JOIN)
        for col in &right_table.columns {
            if !left_table
                .columns
                .iter()
                .any(|left_col| left_col.name == col.name)
            {
                result.add_column(DataColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
                });
            } else {
                let (column_name, qualified_name) = if let Some(alias) = join_alias {
                    // Use the join alias for the column name
                    (
                        format!("{}.{}", alias, col.name),
                        Some(format!("{}.{}", alias, col.name)),
                    )
                } else {
                    // Fall back to _right suffix
                    (format!("{}_right", col.name), col.qualified_name.clone())
                };
                result.add_column(DataColumn {
                    name: column_name,
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name,
                    source_table: join_alias.clone().or_else(|| col.source_table.clone()),
                });
            }
        }

        // Nested loop join
        let mut match_count = 0;
        let mut null_count = 0;

        for left_row in &left_table.rows {
            let left_value = &left_row.values[left_col_idx];
            let mut found_match = false;

            for right_row in &right_table.rows {
                let right_value = &right_row.values[right_col_idx];

                if self.compare_values(left_value, right_value, operator) {
                    let mut joined_row = DataRow { values: Vec::new() };
                    joined_row.values.extend_from_slice(&left_row.values);
                    joined_row.values.extend_from_slice(&right_row.values);
                    result.add_row(joined_row);
                    match_count += 1;
                    found_match = true;
                }
            }

            // If no match found, emit left row with NULLs for right columns
            if !found_match {
                let mut joined_row = DataRow { values: Vec::new() };
                joined_row.values.extend_from_slice(&left_row.values);
                for _ in 0..right_table.column_count() {
                    joined_row.values.push(DataValue::Null);
                }
                result.add_row(joined_row);
                null_count += 1;
            }
        }

        info!(
            "Nested loop LEFT JOIN complete: {} matches, {} nulls in {:?}",
            match_count,
            null_count,
            start.elapsed()
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn numeric_string_folds_to_integer_when_coercing() {
        // A string pulled from JSON/SUBSTR must match an integer join key when
        // the columns differ in type (coerce = true).
        assert_eq!(
            canonical_join_key(&DataValue::String("220".to_string()), true),
            DataValue::Integer(220)
        );
        assert_eq!(
            canonical_join_key(&DataValue::Integer(220), true),
            DataValue::Integer(220)
        );
        assert_eq!(
            canonical_join_key(&DataValue::String("220".to_string()), true),
            canonical_join_key(&DataValue::Integer(220), true)
        );
    }

    #[test]
    fn numeric_strings_stay_distinct_when_not_coercing() {
        // Same-typed columns (e.g. String vs String) do not numerically coerce,
        // so "007" and "7" remain distinct keys. This is the TO_STRING opt-out.
        assert_eq!(
            canonical_join_key(&DataValue::String("007".to_string()), false),
            DataValue::String("007".to_string())
        );
        assert_ne!(
            canonical_join_key(&DataValue::String("007".to_string()), false),
            canonical_join_key(&DataValue::String("7".to_string()), false)
        );
        // A string is never folded into an integer when not coercing.
        assert_ne!(
            canonical_join_key(&DataValue::String("7".to_string()), false),
            canonical_join_key(&DataValue::Integer(7), false)
        );
    }

    #[test]
    fn interned_and_plain_strings_collapse_regardless_of_coercion() {
        for coerce in [true, false] {
            assert_eq!(
                canonical_join_key(
                    &DataValue::InternedString(Arc::new("North".to_string())),
                    coerce
                ),
                canonical_join_key(&DataValue::String("North".to_string()), coerce),
                "interned/plain strings must collapse (coerce = {coerce})"
            );
        }
    }

    #[test]
    fn whole_float_folds_to_integer_when_coercing() {
        assert_eq!(
            canonical_join_key(&DataValue::Float(220.0), true),
            DataValue::Integer(220)
        );
        assert_eq!(
            canonical_join_key(&DataValue::String("220.0".to_string()), true),
            DataValue::Integer(220)
        );
        // Fractional floats stay floats.
        assert_eq!(
            canonical_join_key(&DataValue::Float(220.5), true),
            DataValue::Float(220.5)
        );
        // Whole floats fold to integers regardless of string coercion, so a
        // numeric (int) column joins a numeric (float) column.
        assert_eq!(
            canonical_join_key(&DataValue::Float(220.0), false),
            DataValue::Integer(220)
        );
    }

    #[test]
    fn non_numeric_text_is_preserved() {
        assert_eq!(
            canonical_join_key(&DataValue::String("North".to_string()), true),
            DataValue::String("North".to_string())
        );
        // Leading whitespace is not trimmed, matching WHERE parse semantics.
        assert_eq!(
            canonical_join_key(&DataValue::String(" 220".to_string()), true),
            DataValue::String(" 220".to_string())
        );
    }

    #[test]
    fn non_finite_strings_stay_strings() {
        assert_eq!(
            canonical_join_key(&DataValue::String("inf".to_string()), true),
            DataValue::String("inf".to_string())
        );
        assert_eq!(
            canonical_join_key(&DataValue::String("NaN".to_string()), true),
            DataValue::String("NaN".to_string())
        );
    }

    #[test]
    fn null_is_unchanged() {
        assert_eq!(canonical_join_key(&DataValue::Null, true), DataValue::Null);
    }

    #[test]
    fn coercion_enabled_only_for_differing_value_kinds() {
        // Column kind is sampled from actual values, not declared types.
        let stringy = single_col_table(DataValue::String("7".to_string()));
        let numeric = single_col_table(DataValue::Integer(7));
        let stringy2 = single_col_table(DataValue::String("8".to_string()));
        let empty = DataTable::new("empty"); // no columns/rows

        // Stringy vs numeric -> coerce.
        assert!(join_key_coercion(&stringy, 0, &numeric, 0));
        // Stringy vs stringy -> no coercion.
        assert!(!join_key_coercion(&stringy, 0, &stringy2, 0));
        // Numeric vs numeric -> no coercion (float-folding still applies).
        assert!(!join_key_coercion(&numeric, 0, &numeric, 0));
        // Undeterminable kind -> permissive (coerce).
        assert!(join_key_coercion(&stringy, 0, &empty, 0));
    }

    fn single_col_table(value: DataValue) -> DataTable {
        let mut t = DataTable::new("t");
        t.add_column(DataColumn::new("k"));
        let _ = t.add_row(DataRow {
            values: vec![value],
        });
        t
    }
}
