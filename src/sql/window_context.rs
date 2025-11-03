//! Window function context for managing partitioned and ordered data views
//!
//! This module provides the WindowContext helper class that enables window functions
//! like LAG, LEAD, ROW_NUMBER, etc. by managing partitions and ordering.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use tracing::{debug, info};

use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::sql::parser::ast::{
    FrameBound, FrameUnit, OrderByItem, SortDirection, SqlExpression, WindowSpec,
};

/// Key for identifying a partition (combination of partition column values)
/// We use String representation for now since DataValue doesn't impl Ord
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PartitionKey(String);

impl PartitionKey {
    /// Create a partition key from data values
    fn from_values(values: Vec<DataValue>) -> Self {
        // Create a unique string representation
        let key_parts: Vec<String> = values
            .iter()
            .map(|v| match v {
                DataValue::String(s) => format!("S:{}", s),
                DataValue::InternedString(s) => format!("S:{}", s),
                DataValue::Integer(i) => format!("I:{}", i),
                DataValue::Float(f) => format!("F:{}", f),
                DataValue::Boolean(b) => format!("B:{}", b),
                DataValue::DateTime(dt) => format!("D:{}", dt),
                DataValue::Vector(v) => {
                    let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                    format!("V:[{}]", components.join(","))
                }
                DataValue::Null => "N".to_string(),
            })
            .collect();
        let key = key_parts.join("|");
        PartitionKey(key)
    }
}

/// An ordered partition containing row indices
#[derive(Debug, Clone)]
pub struct OrderedPartition {
    /// Original row indices from DataView, in sorted order
    rows: Vec<usize>,

    /// Quick lookup: row_index -> position in partition
    row_positions: HashMap<usize, usize>,
}

impl OrderedPartition {
    /// Create a new ordered partition from row indices
    fn new(rows: Vec<usize>) -> Self {
        // Build position lookup
        let row_positions: HashMap<usize, usize> = rows
            .iter()
            .enumerate()
            .map(|(pos, &row_idx)| (row_idx, pos))
            .collect();

        Self {
            rows,
            row_positions,
        }
    }

    /// Navigate to offset from current position
    pub fn get_row_at_offset(&self, current_row: usize, offset: i32) -> Option<usize> {
        let current_pos = self.row_positions.get(&current_row)?;
        let target_pos = (*current_pos as i32) + offset;

        if target_pos >= 0 && target_pos < self.rows.len() as i32 {
            Some(self.rows[target_pos as usize])
        } else {
            None
        }
    }

    /// Get position of row in this partition (0-based)
    pub fn get_position(&self, row_index: usize) -> Option<usize> {
        self.row_positions.get(&row_index).copied()
    }

    /// Get the first row index in this partition
    pub fn first_row(&self) -> Option<usize> {
        self.rows.first().copied()
    }

    /// Get the last row index in this partition
    pub fn last_row(&self) -> Option<usize> {
        self.rows.last().copied()
    }
}

/// Context for evaluating window functions
pub struct WindowContext {
    /// Source data view
    source: Arc<DataView>,

    /// Partitions with their ordered rows
    partitions: BTreeMap<PartitionKey, OrderedPartition>,

    /// Mapping from row index to its partition key
    row_to_partition: HashMap<usize, PartitionKey>,

    /// Window specification
    spec: WindowSpec,
}

impl WindowContext {
    /// Create a new window context with partitioning and ordering
    pub fn new(
        view: Arc<DataView>,
        partition_by: Vec<String>,
        order_by: Vec<OrderByItem>,
    ) -> Result<Self> {
        Self::new_with_spec(
            view,
            WindowSpec {
                partition_by,
                order_by,
                frame: None,
            },
        )
    }

    /// Create a new window context with a full window specification
    pub fn new_with_spec(view: Arc<DataView>, spec: WindowSpec) -> Result<Self> {
        let overall_start = Instant::now();
        let partition_by = spec.partition_by.clone();
        let order_by = spec.order_by.clone();
        let row_count = view.row_count();

        // If no partition columns, treat entire view as single partition
        if partition_by.is_empty() {
            info!(
                "Creating single partition (no PARTITION BY) for {} rows",
                row_count
            );
            let partition_start = Instant::now();

            let single_partition = Self::create_single_partition(&view, &order_by)?;
            let partition_key = PartitionKey::from_values(vec![]);

            // Build row-to-partition mapping
            let mut row_to_partition = HashMap::new();
            for &row_idx in &single_partition.rows {
                row_to_partition.insert(row_idx, partition_key.clone());
            }

            let mut partitions = BTreeMap::new();
            partitions.insert(partition_key, single_partition);

            info!(
                "Single partition created in {:.2}ms (1 partition, {} rows)",
                partition_start.elapsed().as_secs_f64() * 1000.0,
                row_count
            );

            info!(
                "WindowContext::new_with_spec (single partition) took {:.2}ms total",
                overall_start.elapsed().as_secs_f64() * 1000.0
            );

            return Ok(Self {
                source: view,
                partitions,
                row_to_partition,
                spec,
            });
        }

        // Create partitions based on partition_by columns
        info!(
            "Creating partitions with PARTITION BY for {} rows",
            row_count
        );
        let partition_start = Instant::now();

        let mut partition_map: BTreeMap<PartitionKey, Vec<usize>> = BTreeMap::new();
        let mut row_to_partition = HashMap::new();

        // Get column indices for partition columns
        let source_table = view.source();
        let partition_col_indices: Vec<usize> = partition_by
            .iter()
            .map(|col| {
                source_table
                    .get_column_index(col)
                    .ok_or_else(|| anyhow!("Invalid partition column: {}", col))
            })
            .collect::<Result<Vec<_>>>()?;

        // Group rows by partition key
        let grouping_start = Instant::now();
        for row_idx in view.get_visible_rows() {
            // Build partition key from row values
            let mut key_values = Vec::new();
            for &col_idx in &partition_col_indices {
                let value = source_table
                    .get_value(row_idx, col_idx)
                    .ok_or_else(|| anyhow!("Failed to get value for partition"))?
                    .clone();
                key_values.push(value);
            }
            let key = PartitionKey::from_values(key_values);

            // Add row to partition
            partition_map.entry(key.clone()).or_default().push(row_idx);
            row_to_partition.insert(row_idx, key);
        }

        info!(
            "Partition grouping took {:.2}ms ({} partitions created)",
            grouping_start.elapsed().as_secs_f64() * 1000.0,
            partition_map.len()
        );

        // Sort each partition according to ORDER BY
        let sort_start = Instant::now();
        let mut partitions = BTreeMap::new();
        let partition_count = partition_map.len();

        for (key, mut rows) in partition_map {
            // Sort rows within partition
            if !order_by.is_empty() {
                Self::sort_rows(&mut rows, source_table, &order_by)?;
            }

            partitions.insert(key, OrderedPartition::new(rows));
        }

        info!(
            "Partition sorting took {:.2}ms ({} partitions, ORDER BY: {})",
            sort_start.elapsed().as_secs_f64() * 1000.0,
            partition_count,
            !order_by.is_empty()
        );

        info!(
            "Total partition creation took {:.2}ms",
            partition_start.elapsed().as_secs_f64() * 1000.0
        );

        info!(
            "WindowContext::new_with_spec (multi-partition) took {:.2}ms total",
            overall_start.elapsed().as_secs_f64() * 1000.0
        );

        Ok(Self {
            source: view,
            partitions,
            row_to_partition,
            spec,
        })
    }

    /// Create a single partition from the entire view
    fn create_single_partition(
        view: &DataView,
        order_by: &[OrderByItem],
    ) -> Result<OrderedPartition> {
        let mut rows: Vec<usize> = view.get_visible_rows();

        if !order_by.is_empty() {
            let sort_start = Instant::now();
            Self::sort_rows(&mut rows, view.source(), order_by)?;
            debug!(
                "Single partition sort took {:.2}ms ({} rows)",
                sort_start.elapsed().as_secs_f64() * 1000.0,
                rows.len()
            );
        }

        Ok(OrderedPartition::new(rows))
    }

    /// Sort row indices according to ORDER BY specification
    fn sort_rows(rows: &mut Vec<usize>, table: &DataTable, order_by: &[OrderByItem]) -> Result<()> {
        let prep_start = Instant::now();

        // Get column indices for ORDER BY columns
        let sort_cols: Vec<(usize, bool)> = order_by
            .iter()
            .map(|col| {
                // Extract column name from expression (currently only supports simple columns)
                let column_name = match &col.expr {
                    SqlExpression::Column(col_ref) => &col_ref.name,
                    _ => {
                        return Err(anyhow!("Window function ORDER BY only supports simple columns, not expressions"));
                    }
                };
                let idx = table
                    .get_column_index(column_name)
                    .ok_or_else(|| anyhow!("Invalid ORDER BY column: {}", column_name))?;
                let ascending = matches!(col.direction, SortDirection::Asc);
                Ok((idx, ascending))
            })
            .collect::<Result<Vec<_>>>()?;

        debug!(
            "Sort preparation took {:.2}μs ({} sort columns)",
            prep_start.elapsed().as_micros(),
            sort_cols.len()
        );

        let sort_start = Instant::now();

        // Sort rows based on column values
        rows.sort_by(|&a, &b| {
            for &(col_idx, ascending) in &sort_cols {
                let val_a = table.get_value(a, col_idx);
                let val_b = table.get_value(b, col_idx);

                match (val_a, val_b) {
                    (None, None) => continue,
                    (None, Some(_)) => {
                        return if ascending {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    }
                    (Some(_), None) => {
                        return if ascending {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        }
                    }
                    (Some(v_a), Some(v_b)) => {
                        // DataValue only implements PartialOrd, not Ord
                        let ord = v_a.partial_cmp(&v_b).unwrap_or(std::cmp::Ordering::Equal);
                        if ord != std::cmp::Ordering::Equal {
                            return if ascending { ord } else { ord.reverse() };
                        }
                    }
                }
            }
            std::cmp::Ordering::Equal
        });

        debug!(
            "Actual sort operation took {:.2}μs ({} rows)",
            sort_start.elapsed().as_micros(),
            rows.len()
        );

        Ok(())
    }

    /// Get value at offset from current row (for LAG/LEAD)
    pub fn get_offset_value(
        &self,
        current_row: usize,
        offset: i32,
        column: &str,
    ) -> Option<DataValue> {
        // Note: This method is called once per row, so we use trace-level logging
        // to avoid overwhelming the debug output
        let start = Instant::now();

        // Find which partition this row belongs to
        let partition_lookup_start = Instant::now();
        let partition_key = self.row_to_partition.get(&current_row)?;
        let partition = self.partitions.get(partition_key)?;
        let partition_lookup_time = partition_lookup_start.elapsed();

        // Navigate to target row
        let offset_nav_start = Instant::now();
        let target_row = partition.get_row_at_offset(current_row, offset)?;
        let offset_nav_time = offset_nav_start.elapsed();

        // Get column value from target row
        let value_access_start = Instant::now();
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;
        let value = source_table.get_value(target_row, col_idx).cloned();
        let value_access_time = value_access_start.elapsed();

        // Only log if this takes more than 10 microseconds (to avoid spam)
        let total_time = start.elapsed();
        if total_time.as_micros() > 10 {
            debug!(
                "get_offset_value slow: total={:.2}μs, partition_lookup={:.2}μs, offset_nav={:.2}μs, value_access={:.2}μs",
                total_time.as_micros(),
                partition_lookup_time.as_micros(),
                offset_nav_time.as_micros(),
                value_access_time.as_micros()
            );
        }

        value
    }

    /// Get row number within partition (1-based)
    pub fn get_row_number(&self, row_index: usize) -> usize {
        if let Some(partition_key) = self.row_to_partition.get(&row_index) {
            if let Some(partition) = self.partitions.get(partition_key) {
                if let Some(position) = partition.get_position(row_index) {
                    return position + 1; // Convert to 1-based
                }
            }
        }
        0 // Should not happen for valid row
    }

    /// Get first value in frame
    pub fn get_frame_first_value(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Null);
        }

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        // Get the first row in the frame
        let first_row = frame_rows[0];
        source_table.get_value(first_row, col_idx).cloned()
    }

    /// Get last value in frame
    pub fn get_frame_last_value(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Null);
        }

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        // Get the last row in the frame
        let last_row = frame_rows[frame_rows.len() - 1];
        source_table.get_value(last_row, col_idx).cloned()
    }

    /// Get first value in partition
    pub fn get_first_value(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let first_row = partition.first_row()?;

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;
        source_table.get_value(first_row, col_idx).cloned()
    }

    /// Get last value in partition
    pub fn get_last_value(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let last_row = partition.last_row()?;

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;
        source_table.get_value(last_row, col_idx).cloned()
    }

    /// Get the number of partitions
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Check if context has partitions (vs single window)
    pub fn has_partitions(&self) -> bool {
        !self.spec.partition_by.is_empty()
    }

    /// Check if context has a window frame specification
    pub fn has_frame(&self) -> bool {
        self.spec.frame.is_some()
    }

    /// Get the source DataView
    pub fn source(&self) -> &DataTable {
        self.source.source()
    }

    /// Get row indices within the window frame for a given row
    pub fn get_frame_rows(&self, row_index: usize) -> Vec<usize> {
        // Find which partition this row belongs to
        let partition_key = match self.row_to_partition.get(&row_index) {
            Some(key) => key,
            None => return vec![],
        };

        let partition = match self.partitions.get(partition_key) {
            Some(p) => p,
            None => return vec![],
        };

        // Get current row's position in partition
        let current_pos = match partition.get_position(row_index) {
            Some(pos) => pos as i64,
            None => return vec![],
        };

        // If no frame specified, return entire partition (default behavior)
        let frame = match &self.spec.frame {
            Some(f) => f,
            None => return partition.rows.clone(),
        };

        // Calculate frame bounds
        let (start_pos, end_pos) = match frame.unit {
            FrameUnit::Rows => {
                // ROWS frame - based on physical row positions
                let start =
                    self.calculate_frame_position(&frame.start, current_pos, partition.rows.len());
                let end = match &frame.end {
                    Some(bound) => {
                        self.calculate_frame_position(bound, current_pos, partition.rows.len())
                    }
                    None => current_pos, // Default to CURRENT ROW
                };
                (start, end)
            }
            FrameUnit::Range => {
                // RANGE frame - based on ORDER BY values (not yet fully implemented)
                // For now, treat like ROWS
                let start =
                    self.calculate_frame_position(&frame.start, current_pos, partition.rows.len());
                let end = match &frame.end {
                    Some(bound) => {
                        self.calculate_frame_position(bound, current_pos, partition.rows.len())
                    }
                    None => current_pos,
                };
                (start, end)
            }
        };

        // Collect rows within frame bounds
        let mut frame_rows = Vec::new();
        for i in start_pos..=end_pos {
            if i >= 0 && (i as usize) < partition.rows.len() {
                frame_rows.push(partition.rows[i as usize]);
            }
        }

        frame_rows
    }

    /// Calculate absolute position from frame bound
    fn calculate_frame_position(
        &self,
        bound: &FrameBound,
        current_pos: i64,
        partition_size: usize,
    ) -> i64 {
        match bound {
            FrameBound::UnboundedPreceding => 0,
            FrameBound::UnboundedFollowing => partition_size as i64 - 1,
            FrameBound::CurrentRow => current_pos,
            FrameBound::Preceding(n) => current_pos - n,
            FrameBound::Following(n) => current_pos + n,
        }
    }

    /// Calculate sum of a column within the window frame for the given row
    pub fn get_frame_sum(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Null);
        }

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut sum = 0.0;
        let mut has_float = false;
        let mut has_value = false;

        // Sum all values in the frame
        for &row_idx in &frame_rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                match value {
                    DataValue::Integer(i) => {
                        sum += *i as f64;
                        has_value = true;
                    }
                    DataValue::Float(f) => {
                        sum += f;
                        has_float = true;
                        has_value = true;
                    }
                    DataValue::Null => {
                        // Skip NULL values
                    }
                    _ => {
                        // Non-numeric values - return NULL
                        return Some(DataValue::Null);
                    }
                }
            }
        }

        if !has_value {
            return Some(DataValue::Null);
        }

        // Return as integer if all values were integers and sum is whole
        if !has_float && sum.fract() == 0.0 && sum >= i64::MIN as f64 && sum <= i64::MAX as f64 {
            Some(DataValue::Integer(sum as i64))
        } else {
            Some(DataValue::Float(sum))
        }
    }

    /// Calculate count within the window frame
    pub fn get_frame_count(&self, row_index: usize, column: Option<&str>) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Integer(0));
        }

        if let Some(col_name) = column {
            // COUNT(column) - count non-null values in frame
            let source_table = self.source.source();
            let col_idx = source_table.get_column_index(col_name)?;

            let count = frame_rows
                .iter()
                .filter_map(|&row_idx| source_table.get_value(row_idx, col_idx))
                .filter(|v| !matches!(v, DataValue::Null))
                .count();

            Some(DataValue::Integer(count as i64))
        } else {
            // COUNT(*) - count all rows in frame
            Some(DataValue::Integer(frame_rows.len() as i64))
        }
    }

    /// Calculate average of a column within the window frame
    pub fn get_frame_avg(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Null);
        }

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut sum = 0.0;
        let mut count = 0;

        // Sum all non-null values in the frame
        for &row_idx in &frame_rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                match value {
                    DataValue::Integer(i) => {
                        sum += *i as f64;
                        count += 1;
                    }
                    DataValue::Float(f) => {
                        sum += f;
                        count += 1;
                    }
                    DataValue::Null => {
                        // Skip NULL values
                    }
                    _ => {
                        // Non-numeric values - return NULL
                        return Some(DataValue::Null);
                    }
                }
            }
        }

        if count == 0 {
            return Some(DataValue::Null);
        }

        Some(DataValue::Float(sum / count as f64))
    }

    /// Calculate standard deviation within the window frame (sample stddev)
    pub fn get_frame_stddev(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let variance = self.get_frame_variance(row_index, column)?;
        match variance {
            DataValue::Float(v) => Some(DataValue::Float(v.sqrt())),
            DataValue::Null => Some(DataValue::Null),
            _ => Some(DataValue::Null),
        }
    }

    /// Calculate variance within the window frame (sample variance with n-1)
    pub fn get_frame_variance(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        if frame_rows.is_empty() {
            return Some(DataValue::Null);
        }

        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut values = Vec::new();

        // Collect all non-null values in the frame
        for &row_idx in &frame_rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                match value {
                    DataValue::Integer(i) => values.push(*i as f64),
                    DataValue::Float(f) => values.push(*f),
                    DataValue::Null => {
                        // Skip NULL values
                    }
                    _ => {
                        // Non-numeric values - return NULL
                        return Some(DataValue::Null);
                    }
                }
            }
        }

        if values.is_empty() {
            return Some(DataValue::Null);
        }

        if values.len() == 1 {
            // Variance of single value is 0
            return Some(DataValue::Float(0.0));
        }

        // Calculate mean
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate sample variance (n-1 denominator)
        let variance =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;

        Some(DataValue::Float(variance))
    }

    /// Calculate sum of a column over the partition containing the given row
    pub fn get_partition_sum(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut sum = 0.0;
        let mut has_float = false;
        let mut has_value = false;

        // Sum all values in the partition
        for &row_idx in &partition.rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                match value {
                    DataValue::Integer(i) => {
                        sum += *i as f64;
                        has_value = true;
                    }
                    DataValue::Float(f) => {
                        sum += f;
                        has_float = true;
                        has_value = true;
                    }
                    DataValue::Null => {
                        // Skip NULL values
                    }
                    _ => {
                        // Non-numeric values - return NULL
                        return Some(DataValue::Null);
                    }
                }
            }
        }

        if !has_value {
            return Some(DataValue::Null);
        }

        // Return as integer if all values were integers and sum is whole
        if !has_float && sum.fract() == 0.0 && sum >= i64::MIN as f64 && sum <= i64::MAX as f64 {
            Some(DataValue::Integer(sum as i64))
        } else {
            Some(DataValue::Float(sum))
        }
    }

    /// Calculate count of non-null values in a column over the partition
    pub fn get_partition_count(&self, row_index: usize, column: Option<&str>) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;

        if let Some(col_name) = column {
            // COUNT(column) - count non-null values
            let source_table = self.source.source();
            let col_idx = source_table.get_column_index(col_name)?;

            let count = partition
                .rows
                .iter()
                .filter_map(|&row_idx| source_table.get_value(row_idx, col_idx))
                .filter(|v| !matches!(v, DataValue::Null))
                .count();

            Some(DataValue::Integer(count as i64))
        } else {
            // COUNT(*) - count all rows in partition
            Some(DataValue::Integer(partition.rows.len() as i64))
        }
    }
}
