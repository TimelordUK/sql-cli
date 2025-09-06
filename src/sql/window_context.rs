//! Window function context for managing partitioned and ordered data views
//!
//! This module provides the WindowContext helper class that enables window functions
//! like LAG, LEAD, ROW_NUMBER, etc. by managing partitions and ordering.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use anyhow::{anyhow, Result};

use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::sql::recursive_parser::{OrderByColumn, SortDirection};

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
    fn new(mut rows: Vec<usize>) -> Self {
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

/// Window specification defining partitioning and ordering
#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub partition_by: Vec<String>,
    pub order_by: Vec<OrderByColumn>,
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
        order_by: Vec<OrderByColumn>,
    ) -> Result<Self> {
        let spec = WindowSpec {
            partition_by: partition_by.clone(),
            order_by: order_by.clone(),
        };

        // If no partition columns, treat entire view as single partition
        if partition_by.is_empty() {
            let single_partition = Self::create_single_partition(&view, &order_by)?;
            let partition_key = PartitionKey::from_values(vec![]);

            // Build row-to-partition mapping
            let mut row_to_partition = HashMap::new();
            for &row_idx in &single_partition.rows {
                row_to_partition.insert(row_idx, partition_key.clone());
            }

            let mut partitions = BTreeMap::new();
            partitions.insert(partition_key, single_partition);

            return Ok(Self {
                source: view,
                partitions,
                row_to_partition,
                spec,
            });
        }

        // Create partitions based on partition_by columns
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

        // Sort each partition according to ORDER BY
        let mut partitions = BTreeMap::new();
        for (key, mut rows) in partition_map {
            // Sort rows within partition
            if !order_by.is_empty() {
                Self::sort_rows(&mut rows, source_table, &order_by)?;
            }

            partitions.insert(key, OrderedPartition::new(rows));
        }

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
        order_by: &[OrderByColumn],
    ) -> Result<OrderedPartition> {
        let mut rows: Vec<usize> = view.get_visible_rows();

        if !order_by.is_empty() {
            Self::sort_rows(&mut rows, view.source(), order_by)?;
        }

        Ok(OrderedPartition::new(rows))
    }

    /// Sort row indices according to ORDER BY specification
    fn sort_rows(
        rows: &mut Vec<usize>,
        table: &DataTable,
        order_by: &[OrderByColumn],
    ) -> Result<()> {
        // Get column indices for ORDER BY columns
        let sort_cols: Vec<(usize, bool)> = order_by
            .iter()
            .map(|col| {
                let idx = table
                    .get_column_index(&col.column)
                    .ok_or_else(|| anyhow!("Invalid ORDER BY column: {}", col.column))?;
                let ascending = matches!(col.direction, SortDirection::Asc);
                Ok((idx, ascending))
            })
            .collect::<Result<Vec<_>>>()?;

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

        Ok(())
    }

    /// Get value at offset from current row (for LAG/LEAD)
    pub fn get_offset_value(
        &self,
        current_row: usize,
        offset: i32,
        column: &str,
    ) -> Option<DataValue> {
        // Find which partition this row belongs to
        let partition_key = self.row_to_partition.get(&current_row)?;
        let partition = self.partitions.get(partition_key)?;

        // Navigate to target row
        let target_row = partition.get_row_at_offset(current_row, offset)?;

        // Get column value from target row
        let source_table = self.source.source();
        let col_idx = source_table.get_column_index(column)?;
        source_table.get_value(target_row, col_idx).cloned()
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
}
