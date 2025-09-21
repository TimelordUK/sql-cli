//! Hash join implementation for efficient JOIN operations

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::sql::parser::ast::{JoinClause, JoinOperator, JoinType};

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
            "Executing {:?} JOIN: {} rows x {} rows",
            join_clause.join_type,
            left_table.row_count(),
            right_table.row_count()
        );

        // Extract column references from the join condition
        let (left_col_name, right_col_name) = self.parse_join_columns(join_clause)?;

        // For join conditions, we need to be smart about which table each column belongs to
        // The left column could reference either table, same with right column
        // Try to resolve based on table prefixes or column existence
        let (left_col_idx, right_col_idx) =
            self.resolve_join_columns(&left_table, &right_table, &left_col_name, &right_col_name)?;

        // Choose join algorithm based on operator
        let use_hash_join = join_clause.condition.operator == JoinOperator::Equal;

        // Perform the appropriate join based on type and operator
        match join_clause.join_type {
            JoinType::Inner => {
                if use_hash_join {
                    self.hash_join_inner(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &left_col_name,
                        &right_col_name,
                    )
                } else {
                    self.nested_loop_join_inner(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &join_clause.condition.operator,
                    )
                }
            }
            JoinType::Left => {
                if use_hash_join {
                    self.hash_join_left(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &left_col_name,
                        &right_col_name,
                    )
                } else {
                    self.nested_loop_join_left(
                        left_table,
                        right_table,
                        left_col_idx,
                        right_col_idx,
                        &join_clause.condition.operator,
                    )
                }
            }
            JoinType::Right => {
                if use_hash_join {
                    // Right join is just a left join with tables swapped
                    self.hash_join_left(
                        right_table,
                        left_table,
                        right_col_idx,
                        left_col_idx,
                        &right_col_name,
                        &left_col_name,
                    )
                } else {
                    // Right join is just a left join with tables swapped
                    self.nested_loop_join_left(
                        right_table,
                        left_table,
                        right_col_idx,
                        left_col_idx,
                        &self.reverse_operator(&join_clause.condition.operator),
                    )
                }
            }
            JoinType::Cross => self.cross_join(left_table, right_table),
            JoinType::Full => {
                return Err(anyhow!("FULL OUTER JOIN not yet implemented"));
            }
        }
    }

    /// Parse join columns from the join condition
    fn parse_join_columns(&self, join_clause: &JoinClause) -> Result<(String, String)> {
        Ok((
            join_clause.condition.left_column.clone(),
            join_clause.condition.right_column.clone(),
        ))
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
        } else if let Ok(idx) = self.find_column_index(right_table, left_col_name) {
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
        } else if let Ok(idx) = self.find_column_index(left_table, right_col_name) {
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
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

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
            let key = row.values[build_col_idx].clone();
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
                result.add_column(DataColumn {
                    name: format!("{}_right", col.name),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
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
            let probe_key = &probe_row.values[probe_col_idx];

            if let Some(matching_indices) = hash_index.get(probe_key) {
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
    ) -> Result<DataTable> {
        let start = std::time::Instant::now();

        debug!(
            "Building hash index on right table ({} rows)",
            right_table.row_count()
        );

        // Build hash index on right table
        let mut hash_index: HashMap<DataValue, Vec<usize>> = HashMap::new();
        for (row_idx, row) in right_table.rows.iter().enumerate() {
            let key = row.values[right_col_idx].clone();
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
                result.add_column(DataColumn {
                    name: format!("{}_right", col.name),
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
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
            let left_key = &left_row.values[left_col_idx];

            if let Some(matching_indices) = hash_index.get(left_key) {
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

    /// Compare two values based on the join operator
    fn compare_values(&self, left: &DataValue, right: &DataValue, op: &JoinOperator) -> bool {
        match op {
            JoinOperator::Equal => left == right,
            JoinOperator::NotEqual => left != right,
            JoinOperator::LessThan => left < right,
            JoinOperator::GreaterThan => left > right,
            JoinOperator::LessThanOrEqual => left <= right,
            JoinOperator::GreaterThanOrEqual => left >= right,
        }
    }

    /// Nested loop join for INNER JOIN with inequality conditions
    fn nested_loop_join_inner(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        operator: &JoinOperator,
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
                result.add_column(DataColumn {
                    name: format!("{}_right", col.name),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
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

    /// Nested loop join for LEFT JOIN with inequality conditions
    fn nested_loop_join_left(
        &self,
        left_table: Arc<DataTable>,
        right_table: Arc<DataTable>,
        left_col_idx: usize,
        right_col_idx: usize,
        operator: &JoinOperator,
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
                result.add_column(DataColumn {
                    name: format!("{}_right", col.name),
                    data_type: col.data_type.clone(),
                    nullable: true, // Always nullable for outer join
                    unique_values: col.unique_values,
                    null_count: col.null_count,
                    metadata: col.metadata.clone(),
                    qualified_name: col.qualified_name.clone(), // Preserve qualified name
                    source_table: col.source_table.clone(),     // Preserve source table
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
