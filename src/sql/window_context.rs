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

    /// Check if this is a running aggregate frame (UNBOUNDED PRECEDING to CURRENT ROW)
    pub fn is_running_aggregate_frame(&self) -> bool {
        if let Some(frame) = &self.spec.frame {
            matches!(frame.start, FrameBound::UnboundedPreceding) && frame.end.is_none()
        // None means CURRENT ROW
        } else {
            false
        }
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

    /// Calculate average of a column over the partition containing the given row
    pub fn get_partition_avg(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut sum = 0.0;
        let mut count = 0;

        // Sum all values in the partition
        for &row_idx in &partition.rows {
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
            Some(DataValue::Null)
        } else {
            Some(DataValue::Float(sum / count as f64))
        }
    }

    /// Calculate minimum value of a column over the partition containing the given row
    pub fn get_partition_min(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut min_value: Option<DataValue> = None;

        for &row_idx in &partition.rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                if !matches!(value, DataValue::Null) {
                    match &min_value {
                        None => min_value = Some(value.clone()),
                        Some(current_min) => {
                            use crate::data::datavalue_compare::compare_datavalues;
                            if compare_datavalues(value, current_min).is_lt() {
                                min_value = Some(value.clone());
                            }
                        }
                    }
                }
            }
        }

        min_value.or(Some(DataValue::Null))
    }

    /// Calculate maximum value of a column over the partition containing the given row
    pub fn get_partition_max(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let partition_key = self.row_to_partition.get(&row_index)?;
        let partition = self.partitions.get(partition_key)?;
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut max_value: Option<DataValue> = None;

        for &row_idx in &partition.rows {
            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                if !matches!(value, DataValue::Null) {
                    match &max_value {
                        None => max_value = Some(value.clone()),
                        Some(current_max) => {
                            use crate::data::datavalue_compare::compare_datavalues;
                            if compare_datavalues(value, current_max).is_gt() {
                                max_value = Some(value.clone());
                            }
                        }
                    }
                }
            }
        }

        max_value.or(Some(DataValue::Null))
    }

    /// Get all row indices in the partition containing the given row
    pub fn get_partition_rows(&self, row_index: usize) -> Vec<usize> {
        if let Some(partition_key) = self.row_to_partition.get(&row_index) {
            if let Some(partition) = self.partitions.get(partition_key) {
                return partition.rows.clone();
            }
        }
        vec![]
    }

    /// Calculate minimum value within the frame for a given row
    pub fn get_frame_min(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut min_value: Option<DataValue> = None;

        for &frame_row_idx in &frame_rows {
            if let Some(value) = source_table.get_value(frame_row_idx, col_idx) {
                if !matches!(value, DataValue::Null) {
                    match &min_value {
                        None => min_value = Some(value.clone()),
                        Some(current_min) => {
                            use crate::data::datavalue_compare::compare_datavalues;
                            if compare_datavalues(value, current_min).is_lt() {
                                min_value = Some(value.clone());
                            }
                        }
                    }
                }
            }
        }

        min_value.or(Some(DataValue::Null))
    }

    /// Calculate maximum value within the frame for a given row
    pub fn get_frame_max(&self, row_index: usize, column: &str) -> Option<DataValue> {
        let frame_rows = self.get_frame_rows(row_index);
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;

        let mut max_value: Option<DataValue> = None;

        for &frame_row_idx in &frame_rows {
            if let Some(value) = source_table.get_value(frame_row_idx, col_idx) {
                if !matches!(value, DataValue::Null) {
                    match &max_value {
                        None => max_value = Some(value.clone()),
                        Some(current_max) => {
                            use crate::data::datavalue_compare::compare_datavalues;
                            if compare_datavalues(value, current_max).is_gt() {
                                max_value = Some(value.clone());
                            }
                        }
                    }
                }
            }
        }

        max_value.or(Some(DataValue::Null))
    }

    /// Batch evaluate LAG function for multiple rows
    /// This is significantly faster than calling get_offset_value for each row
    pub fn evaluate_lag_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
        offset: i64,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Validate column exists
        let source_table = self.source.source();
        let _ = source_table
            .get_column_index(column_name)
            .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

        for &row_idx in visible_rows {
            // LAG uses negative offset internally
            let value =
                if let Some(val) = self.get_offset_value(row_idx, -(offset as i32), column_name) {
                    val
                } else {
                    DataValue::Null
                };
            results.push(value);
        }

        debug!(
            "evaluate_lag_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate LEAD function for multiple rows
    pub fn evaluate_lead_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
        offset: i64,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Validate column exists
        let source_table = self.source.source();
        let _ = source_table
            .get_column_index(column_name)
            .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

        for &row_idx in visible_rows {
            // LEAD uses positive offset
            let value =
                if let Some(val) = self.get_offset_value(row_idx, offset as i32, column_name) {
                    val
                } else {
                    DataValue::Null
                };
            results.push(value);
        }

        debug!(
            "evaluate_lead_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate ROW_NUMBER function for multiple rows
    pub fn evaluate_row_number_batch(&self, visible_rows: &[usize]) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        for &row_idx in visible_rows {
            let row_num = self.get_row_number(row_idx);
            if row_num > 0 {
                results.push(DataValue::Integer(row_num as i64));
            } else {
                // This shouldn't happen if WindowContext is properly constructed
                results.push(DataValue::Null);
            }
        }

        debug!(
            "evaluate_row_number_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Get rank of a row within its partition
    /// Ties get the same rank, and next rank(s) are skipped
    pub fn get_rank(&self, row_index: usize) -> i64 {
        if let Some(partition_key) = self.row_to_partition.get(&row_index) {
            if let Some(partition) = self.partitions.get(partition_key) {
                // Get the value at this row for comparison
                if let Some(position) = partition.get_position(row_index) {
                    let mut rows_before = 0;

                    // Compare with all previous rows in the partition
                    for i in 0..position {
                        let prev_row = partition.rows[i];
                        if self.compare_rows_for_rank(prev_row, row_index) < 0 {
                            rows_before += 1;
                        }
                    }

                    let rank = rows_before + 1;
                    return rank;
                }
            }
        }
        1 // Default if not found
    }

    /// Get dense rank of a row within its partition
    /// Ties get the same rank, but ranks are not skipped
    pub fn get_dense_rank(&self, row_index: usize) -> i64 {
        if let Some(partition_key) = self.row_to_partition.get(&row_index) {
            if let Some(partition) = self.partitions.get(partition_key) {
                if let Some(position) = partition.get_position(row_index) {
                    let mut dense_rank = 1;
                    let mut last_value_seen = None;

                    // Look at all rows before this one
                    for i in 0..position {
                        let prev_row = partition.rows[i];
                        let cmp = self.compare_rows_for_rank(prev_row, row_index);

                        if cmp < 0 {
                            // This row has a better rank
                            if last_value_seen.is_none()
                                || last_value_seen.map_or(true, |last| {
                                    self.compare_rows_for_rank(last, prev_row) != 0
                                })
                            {
                                dense_rank += 1;
                                last_value_seen = Some(prev_row);
                            }
                        }
                    }

                    return dense_rank;
                }
            }
        }
        1 // Default if not found
    }

    /// Compare two rows based on ORDER BY columns for ranking
    /// Returns -1 if row1 < row2, 0 if equal, 1 if row1 > row2
    fn compare_rows_for_rank(&self, row1: usize, row2: usize) -> i32 {
        let source_table = self.source.source();

        for order_item in &self.spec.order_by {
            if let SqlExpression::Column(col) = &order_item.expr {
                if let Some(col_idx) = source_table.get_column_index(&col.name) {
                    let val1 = source_table.get_value(row1, col_idx);
                    let val2 = source_table.get_value(row2, col_idx);

                    let cmp = match (val1, val2) {
                        (Some(v1), Some(v2)) => {
                            crate::data::datavalue_compare::compare_datavalues(v1, v2)
                        }
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    };

                    if cmp != std::cmp::Ordering::Equal {
                        let result = match order_item.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        };

                        return match result {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        };
                    }
                }
            }
        }

        0 // All columns equal
    }

    /// Batch evaluate RANK function for multiple rows
    pub fn evaluate_rank_batch(&self, visible_rows: &[usize]) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        for &row_idx in visible_rows {
            let rank = self.get_rank(row_idx);
            results.push(DataValue::Integer(rank));
        }

        debug!(
            "evaluate_rank_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate DENSE_RANK function for multiple rows
    pub fn evaluate_dense_rank_batch(&self, visible_rows: &[usize]) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        for &row_idx in visible_rows {
            let rank = self.get_dense_rank(row_idx);
            results.push(DataValue::Integer(rank));
        }

        debug!(
            "evaluate_dense_rank_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate SUM window aggregate
    pub fn evaluate_sum_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Check if this is a running aggregate (UNBOUNDED PRECEDING to CURRENT ROW)
        let is_running_aggregate = self.is_running_aggregate_frame();

        if is_running_aggregate && !visible_rows.is_empty() {
            // Optimized path for running aggregates
            debug!(
                "Using optimized running sum for {} rows",
                visible_rows.len()
            );

            // Group visible rows by partition
            let mut partition_groups: BTreeMap<PartitionKey, Vec<(usize, usize)>> = BTreeMap::new();
            for (idx, &row_idx) in visible_rows.iter().enumerate() {
                if let Some(partition_key) = self.row_to_partition.get(&row_idx) {
                    partition_groups
                        .entry(partition_key.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, row_idx));
                }
            }

            let source_table = self.source.source();
            let col_idx = source_table
                .get_column_index(column_name)
                .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

            // Initialize results with nulls
            results.resize(visible_rows.len(), DataValue::Null);

            // Process each partition
            for (_partition_key, rows) in partition_groups {
                if let Some(partition) = self.partitions.get(&_partition_key) {
                    let mut running_sum = 0.0;
                    let mut has_float = false;
                    let mut position_to_sum: HashMap<usize, DataValue> = HashMap::new();

                    // Calculate running sum for all positions in this partition
                    for (pos, &row_idx) in partition.rows.iter().enumerate() {
                        if let Some(value) = source_table.get_value(row_idx, col_idx) {
                            match value {
                                DataValue::Integer(i) => {
                                    running_sum += *i as f64;
                                }
                                DataValue::Float(f) => {
                                    running_sum += f;
                                    has_float = true;
                                }
                                DataValue::Null => {
                                    // Skip NULL values
                                }
                                _ => {
                                    // Non-numeric value
                                }
                            }
                        }

                        // Store the running sum for this position
                        if !has_float
                            && running_sum.fract() == 0.0
                            && running_sum >= i64::MIN as f64
                            && running_sum <= i64::MAX as f64
                        {
                            position_to_sum.insert(pos, DataValue::Integer(running_sum as i64));
                        } else {
                            position_to_sum.insert(pos, DataValue::Float(running_sum));
                        }
                    }

                    // Fill in results for visible rows in this partition
                    for (result_idx, row_idx) in rows {
                        if let Some(pos) = partition.get_position(row_idx) {
                            results[result_idx] = position_to_sum
                                .get(&pos)
                                .cloned()
                                .unwrap_or(DataValue::Null);
                        }
                    }
                }
            }
        } else {
            // Regular path for non-running aggregates or other frame types
            for &row_idx in visible_rows {
                let value = if self.has_frame() {
                    self.get_frame_sum(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                } else {
                    self.get_partition_sum(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                };
                results.push(value);
            }
        }

        debug!(
            "evaluate_sum_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate AVG window aggregate
    pub fn evaluate_avg_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Check if this is a running aggregate (UNBOUNDED PRECEDING to CURRENT ROW)
        let is_running_aggregate = self.is_running_aggregate_frame();

        if is_running_aggregate && !visible_rows.is_empty() {
            // Optimized path for running averages
            debug!(
                "Using optimized running average for {} rows",
                visible_rows.len()
            );

            // Group visible rows by partition
            let mut partition_groups: BTreeMap<PartitionKey, Vec<(usize, usize)>> = BTreeMap::new();
            for (idx, &row_idx) in visible_rows.iter().enumerate() {
                if let Some(partition_key) = self.row_to_partition.get(&row_idx) {
                    partition_groups
                        .entry(partition_key.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, row_idx));
                }
            }

            let source_table = self.source.source();
            let col_idx = source_table
                .get_column_index(column_name)
                .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

            // Initialize results with nulls
            results.resize(visible_rows.len(), DataValue::Null);

            // Process each partition
            for (_partition_key, rows) in partition_groups {
                if let Some(partition) = self.partitions.get(&_partition_key) {
                    let mut running_sum = 0.0;
                    let mut count = 0;
                    let mut position_to_avg: HashMap<usize, DataValue> = HashMap::new();

                    // Calculate running average for all positions in this partition
                    for (pos, &row_idx) in partition.rows.iter().enumerate() {
                        if let Some(value) = source_table.get_value(row_idx, col_idx) {
                            match value {
                                DataValue::Integer(i) => {
                                    running_sum += *i as f64;
                                    count += 1;
                                }
                                DataValue::Float(f) => {
                                    running_sum += f;
                                    count += 1;
                                }
                                DataValue::Null => {
                                    // Skip NULL values
                                }
                                _ => {
                                    // Non-numeric value
                                }
                            }
                        }

                        // Store the running average for this position
                        if count > 0 {
                            position_to_avg
                                .insert(pos, DataValue::Float(running_sum / count as f64));
                        } else {
                            position_to_avg.insert(pos, DataValue::Null);
                        }
                    }

                    // Fill in results for visible rows in this partition
                    for (result_idx, row_idx) in rows {
                        if let Some(pos) = partition.get_position(row_idx) {
                            results[result_idx] = position_to_avg
                                .get(&pos)
                                .cloned()
                                .unwrap_or(DataValue::Null);
                        }
                    }
                }
            }
        } else {
            // Regular path for non-running aggregates or other frame types
            for &row_idx in visible_rows {
                let value = if self.has_frame() {
                    self.get_frame_avg(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                } else {
                    self.get_partition_avg(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                };
                results.push(value);
            }
        }

        debug!(
            "evaluate_avg_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate MIN window aggregate
    pub fn evaluate_min_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Check if this is a running aggregate (UNBOUNDED PRECEDING to CURRENT ROW)
        let is_running_aggregate = self.is_running_aggregate_frame();

        if is_running_aggregate && !visible_rows.is_empty() {
            // Optimized path for running minimum
            debug!(
                "Using optimized running minimum for {} rows",
                visible_rows.len()
            );

            // Group visible rows by partition
            let mut partition_groups: BTreeMap<PartitionKey, Vec<(usize, usize)>> = BTreeMap::new();
            for (idx, &row_idx) in visible_rows.iter().enumerate() {
                if let Some(partition_key) = self.row_to_partition.get(&row_idx) {
                    partition_groups
                        .entry(partition_key.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, row_idx));
                }
            }

            let source_table = self.source.source();
            let col_idx = source_table
                .get_column_index(column_name)
                .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

            // Initialize results with nulls
            results.resize(visible_rows.len(), DataValue::Null);

            // Process each partition
            for (_partition_key, rows) in partition_groups {
                if let Some(partition) = self.partitions.get(&_partition_key) {
                    let mut running_min: Option<DataValue> = None;
                    let mut position_to_min: HashMap<usize, DataValue> = HashMap::new();

                    // Calculate running minimum for all positions in this partition
                    for (pos, &row_idx) in partition.rows.iter().enumerate() {
                        if let Some(value) = source_table.get_value(row_idx, col_idx) {
                            if !matches!(value, DataValue::Null) {
                                match &running_min {
                                    None => running_min = Some(value.clone()),
                                    Some(current_min) => {
                                        use crate::data::datavalue_compare::compare_datavalues;
                                        if compare_datavalues(value, current_min).is_lt() {
                                            running_min = Some(value.clone());
                                        }
                                    }
                                }
                            }
                        }

                        // Store the running minimum for this position
                        position_to_min.insert(pos, running_min.clone().unwrap_or(DataValue::Null));
                    }

                    // Fill in results for visible rows in this partition
                    for (result_idx, row_idx) in rows {
                        if let Some(pos) = partition.get_position(row_idx) {
                            results[result_idx] = position_to_min
                                .get(&pos)
                                .cloned()
                                .unwrap_or(DataValue::Null);
                        }
                    }
                }
            }
        } else {
            // Regular path for non-running aggregates or other frame types
            for &row_idx in visible_rows {
                let value = if self.has_frame() {
                    self.get_frame_min(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                } else {
                    self.get_partition_min(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                };
                results.push(value);
            }
        }

        debug!(
            "evaluate_min_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate MAX window aggregate
    pub fn evaluate_max_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Check if this is a running aggregate (UNBOUNDED PRECEDING to CURRENT ROW)
        let is_running_aggregate = self.is_running_aggregate_frame();

        if is_running_aggregate && !visible_rows.is_empty() {
            // Optimized path for running maximum
            debug!(
                "Using optimized running maximum for {} rows",
                visible_rows.len()
            );

            // Group visible rows by partition
            let mut partition_groups: BTreeMap<PartitionKey, Vec<(usize, usize)>> = BTreeMap::new();
            for (idx, &row_idx) in visible_rows.iter().enumerate() {
                if let Some(partition_key) = self.row_to_partition.get(&row_idx) {
                    partition_groups
                        .entry(partition_key.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, row_idx));
                }
            }

            let source_table = self.source.source();
            let col_idx = source_table
                .get_column_index(column_name)
                .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?;

            // Initialize results with nulls
            results.resize(visible_rows.len(), DataValue::Null);

            // Process each partition
            for (_partition_key, rows) in partition_groups {
                if let Some(partition) = self.partitions.get(&_partition_key) {
                    let mut running_max: Option<DataValue> = None;
                    let mut position_to_max: HashMap<usize, DataValue> = HashMap::new();

                    // Calculate running maximum for all positions in this partition
                    for (pos, &row_idx) in partition.rows.iter().enumerate() {
                        if let Some(value) = source_table.get_value(row_idx, col_idx) {
                            if !matches!(value, DataValue::Null) {
                                match &running_max {
                                    None => running_max = Some(value.clone()),
                                    Some(current_max) => {
                                        use crate::data::datavalue_compare::compare_datavalues;
                                        if compare_datavalues(value, current_max).is_gt() {
                                            running_max = Some(value.clone());
                                        }
                                    }
                                }
                            }
                        }

                        // Store the running maximum for this position
                        position_to_max.insert(pos, running_max.clone().unwrap_or(DataValue::Null));
                    }

                    // Fill in results for visible rows in this partition
                    for (result_idx, row_idx) in rows {
                        if let Some(pos) = partition.get_position(row_idx) {
                            results[result_idx] = position_to_max
                                .get(&pos)
                                .cloned()
                                .unwrap_or(DataValue::Null);
                        }
                    }
                }
            }
        } else {
            // Regular path for non-running aggregates or other frame types
            for &row_idx in visible_rows {
                let value = if self.has_frame() {
                    self.get_frame_max(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                } else {
                    self.get_partition_max(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                };
                results.push(value);
            }
        }

        debug!(
            "evaluate_max_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate COUNT window aggregate
    pub fn evaluate_count_batch(
        &self,
        visible_rows: &[usize],
        column_name: Option<&str>,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        // Check if this is a running aggregate (UNBOUNDED PRECEDING to CURRENT ROW)
        let is_running_aggregate = self.is_running_aggregate_frame();

        if is_running_aggregate && !visible_rows.is_empty() {
            // Optimized path for running count
            debug!(
                "Using optimized running count for {} rows",
                visible_rows.len()
            );

            // Group visible rows by partition
            let mut partition_groups: BTreeMap<PartitionKey, Vec<(usize, usize)>> = BTreeMap::new();
            for (idx, &row_idx) in visible_rows.iter().enumerate() {
                if let Some(partition_key) = self.row_to_partition.get(&row_idx) {
                    partition_groups
                        .entry(partition_key.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, row_idx));
                }
            }

            let source_table = self.source.source();
            let col_idx = if let Some(col_name) = column_name {
                source_table.get_column_index(col_name)
            } else {
                None
            };

            // Initialize results with nulls
            results.resize(visible_rows.len(), DataValue::Null);

            // Process each partition
            for (_partition_key, rows) in partition_groups {
                if let Some(partition) = self.partitions.get(&_partition_key) {
                    let mut running_count = 0i64;
                    let mut position_to_count: HashMap<usize, DataValue> = HashMap::new();

                    // Calculate running count for all positions in this partition
                    for (pos, &row_idx) in partition.rows.iter().enumerate() {
                        if let Some(col_idx) = col_idx {
                            // COUNT(column) - count non-null values
                            if let Some(value) = source_table.get_value(row_idx, col_idx) {
                                if !matches!(value, DataValue::Null) {
                                    running_count += 1;
                                }
                            }
                        } else {
                            // COUNT(*) - always increment
                            running_count += 1;
                        }

                        // Store the running count for this position
                        position_to_count.insert(pos, DataValue::Integer(running_count));
                    }

                    // Fill in results for visible rows in this partition
                    for (result_idx, row_idx) in rows {
                        if let Some(pos) = partition.get_position(row_idx) {
                            results[result_idx] = position_to_count
                                .get(&pos)
                                .cloned()
                                .unwrap_or(DataValue::Null);
                        }
                    }
                }
            }
        } else {
            // Regular path for non-running aggregates or other frame types
            for &row_idx in visible_rows {
                let value = if self.has_frame() {
                    self.get_frame_count(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                } else {
                    self.get_partition_count(row_idx, column_name)
                        .unwrap_or(DataValue::Null)
                };
                results.push(value);
            }
        }

        debug!(
            "evaluate_count_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate FIRST_VALUE window function
    pub fn evaluate_first_value_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        for &row_idx in visible_rows {
            let value = if self.has_frame() {
                self.get_frame_first_value(row_idx, column_name)
                    .unwrap_or(DataValue::Null)
            } else {
                self.get_first_value(row_idx, column_name)
                    .unwrap_or(DataValue::Null)
            };
            results.push(value);
        }

        debug!(
            "evaluate_first_value_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }

    /// Batch evaluate LAST_VALUE window function
    pub fn evaluate_last_value_batch(
        &self,
        visible_rows: &[usize],
        column_name: &str,
    ) -> Result<Vec<DataValue>> {
        let start = Instant::now();
        let mut results = Vec::with_capacity(visible_rows.len());

        for &row_idx in visible_rows {
            let value = if self.has_frame() {
                self.get_frame_last_value(row_idx, column_name)
                    .unwrap_or(DataValue::Null)
            } else {
                self.get_last_value(row_idx, column_name)
                    .unwrap_or(DataValue::Null)
            };
            results.push(value);
        }

        debug!(
            "evaluate_last_value_batch: {} rows in {:.3}ms ({:.2}μs/row)",
            visible_rows.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            start.elapsed().as_micros() as f64 / visible_rows.len() as f64
        );

        Ok(results)
    }
}
