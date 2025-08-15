use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;

use crate::data::data_provider::DataProvider;
use crate::data::datatable::{DataRow, DataTable, DataValue};

/// Sort order for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
    None,
}

/// Sort state tracking
#[derive(Debug, Clone)]
pub struct SortState {
    /// Currently sorted column index (in visible columns order)
    pub column: Option<usize>,
    /// Sort order
    pub order: SortOrder,
}

/// Position where virtual columns can be inserted
#[derive(Debug, Clone, PartialEq)]
pub enum VirtualColumnPosition {
    /// Before all real columns (leftmost)
    Left,
    /// After all real columns (rightmost)  
    Right,
    /// At specific column index
    Index(usize),
}

/// A virtual column that generates values dynamically
#[derive(Clone)]
pub struct VirtualColumn {
    /// Column name
    pub name: String,
    /// Function that generates cell value for a given row index
    pub generator: Arc<dyn Fn(usize) -> String + Send + Sync>,
    /// Preferred width for the column
    pub width: Option<usize>,
    /// Position where this column should appear
    pub position: VirtualColumnPosition,
}

impl std::fmt::Debug for VirtualColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualColumn")
            .field("name", &self.name)
            .field("width", &self.width)
            .field("position", &self.position)
            .finish()
    }
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: None,
            order: SortOrder::None,
        }
    }
}

/// A view over a DataTable that can filter, sort, and project columns
/// without modifying the underlying data
#[derive(Clone)]
pub struct DataView {
    /// The underlying immutable data source
    source: Arc<DataTable>,

    /// Row indices that are visible (after filtering)
    visible_rows: Vec<usize>,

    /// Column indices that are visible (after projection)
    visible_columns: Vec<usize>,

    /// Limit and offset for pagination
    limit: Option<usize>,
    offset: usize,

    /// Base rows before any filtering (for restoring after clear filter)
    /// This allows us to clear filters without losing sort order
    base_rows: Vec<usize>,

    /// Base columns from the original projection (for restoring after unhide all)
    /// This preserves the original column selection if view was created with specific columns
    base_columns: Vec<usize>,

    /// Active filter pattern (if any)
    filter_pattern: Option<String>,

    /// Column search state
    column_search_pattern: Option<String>,
    /// Matching columns for column search (index, name)
    matching_columns: Vec<(usize, String)>,
    /// Current column search match index
    current_column_match: usize,

    /// Pinned columns (always shown on left, in order)
    pinned_columns: Vec<usize>,
    /// Maximum number of pinned columns allowed
    max_pinned_columns: usize,

    /// Sort state
    sort_state: SortState,

    /// Virtual columns that generate dynamic content
    virtual_columns: Vec<VirtualColumn>,
}

impl DataView {
    /// Create a new view showing all data from the table
    pub fn new(source: Arc<DataTable>) -> Self {
        let row_count = source.row_count();
        let col_count = source.column_count();
        let all_rows: Vec<usize> = (0..row_count).collect();
        let all_columns: Vec<usize> = (0..col_count).collect();

        Self {
            source,
            visible_rows: all_rows.clone(),
            visible_columns: all_columns.clone(),
            limit: None,
            offset: 0,
            base_rows: all_rows,
            base_columns: all_columns,
            filter_pattern: None,
            column_search_pattern: None,
            matching_columns: Vec::new(),
            current_column_match: 0,
            pinned_columns: Vec::new(),
            max_pinned_columns: 4,
            sort_state: SortState::default(),
            virtual_columns: Vec::new(),
        }
    }

    /// Create a view with specific columns
    pub fn with_columns(mut self, columns: Vec<usize>) -> Self {
        self.visible_columns = columns.clone();
        self.base_columns = columns; // Store as the base projection
        self
    }

    /// Hide a column by index (cannot hide pinned columns)
    pub fn hide_column(&mut self, column_index: usize) -> bool {
        // Cannot hide a pinned column
        if self.pinned_columns.contains(&column_index) {
            return false;
        }

        self.visible_columns.retain(|&idx| idx != column_index);
        true
    }

    /// Hide a column by name (cannot hide pinned columns)
    pub fn hide_column_by_name(&mut self, column_name: &str) -> bool {
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            self.hide_column(col_idx)
        } else {
            false
        }
    }

    /// Unhide all columns (restore to the base column projection)
    /// This restores to the original column selection, not necessarily all source columns
    pub fn unhide_all_columns(&mut self) {
        self.visible_columns = self.base_columns.clone();
    }

    /// Hide all columns (clear all visible columns)
    pub fn hide_all_columns(&mut self) {
        self.visible_columns.clear();
    }

    /// Check if any columns are visible
    pub fn has_visible_columns(&self) -> bool {
        !self.visible_columns.is_empty()
    }

    /// Move a column left in the view (respects pinned columns)
    /// With wraparound: moving left from first unpinned position moves to last
    pub fn move_column_left(&mut self, display_column_index: usize) -> bool {
        let pinned_count = self.pinned_columns.len();
        let total_columns = pinned_count + self.visible_columns.len();

        if display_column_index >= total_columns {
            return false;
        }

        // If it's a pinned column, move within pinned area
        if display_column_index < pinned_count {
            if display_column_index == 0 {
                // First pinned column - wrap to last pinned position
                if pinned_count > 1 {
                    let col = self.pinned_columns.remove(0);
                    self.pinned_columns.push(col);
                }
            } else {
                // Swap with previous pinned column
                self.pinned_columns
                    .swap(display_column_index - 1, display_column_index);
            }
            return true;
        }

        // It's a visible column - adjust index
        let visible_idx = display_column_index - pinned_count;

        if visible_idx >= self.visible_columns.len() {
            return false;
        }

        if visible_idx == 0 {
            // At first unpinned position - wrap to end
            let col = self.visible_columns.remove(0);
            self.visible_columns.push(col);
        } else {
            // Normal swap with previous
            self.visible_columns.swap(visible_idx - 1, visible_idx);
        }
        true
    }

    /// Move a column right in the view (respects pinned columns)
    /// With wraparound: moving right from last position moves to first
    pub fn move_column_right(&mut self, display_column_index: usize) -> bool {
        let pinned_count = self.pinned_columns.len();
        let total_columns = pinned_count + self.visible_columns.len();

        if display_column_index >= total_columns {
            return false;
        }

        // If it's a pinned column, move within pinned area
        if display_column_index < pinned_count {
            if display_column_index == pinned_count - 1 {
                // Last pinned column - wrap to first pinned position
                if pinned_count > 1 {
                    let col = self.pinned_columns.pop().unwrap();
                    self.pinned_columns.insert(0, col);
                }
            } else {
                // Swap with next pinned column
                self.pinned_columns
                    .swap(display_column_index, display_column_index + 1);
            }
            return true;
        }

        // It's a visible column - adjust index
        let visible_idx = display_column_index - pinned_count;

        if visible_idx >= self.visible_columns.len() {
            return false;
        }

        if visible_idx == self.visible_columns.len() - 1 {
            // At last position - wrap to beginning of unpinned area
            let col = self.visible_columns.pop().unwrap();
            self.visible_columns.insert(0, col);
        } else {
            // Normal swap with next
            self.visible_columns.swap(visible_idx, visible_idx + 1);
        }
        true
    }

    /// Move a column by name to the left
    pub fn move_column_left_by_name(&mut self, column_name: &str) -> bool {
        if let Some(source_idx) = self.source.get_column_index(column_name) {
            if let Some(visible_idx) = self
                .visible_columns
                .iter()
                .position(|&idx| idx == source_idx)
            {
                return self.move_column_left(visible_idx);
            }
        }
        false
    }

    /// Move a column by name to the right
    pub fn move_column_right_by_name(&mut self, column_name: &str) -> bool {
        if let Some(source_idx) = self.source.get_column_index(column_name) {
            if let Some(visible_idx) = self
                .visible_columns
                .iter()
                .position(|&idx| idx == source_idx)
            {
                return self.move_column_right(visible_idx);
            }
        }
        false
    }

    /// Get the names of hidden columns (columns in source but not visible)
    pub fn get_hidden_column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        let visible_columns = self.column_names();

        all_columns
            .into_iter()
            .filter(|col| !visible_columns.contains(col))
            .collect()
    }

    /// Check if there are any hidden columns
    pub fn has_hidden_columns(&self) -> bool {
        self.visible_columns.len() < self.source.column_count()
    }

    // ========== Pinned Column Methods ==========

    /// Pin a column (move it to the pinned area on the left)
    pub fn pin_column(&mut self, column_index: usize) -> Result<()> {
        // Check if we've reached the max
        if self.pinned_columns.len() >= self.max_pinned_columns {
            return Err(anyhow::anyhow!(
                "Maximum {} pinned columns allowed",
                self.max_pinned_columns
            ));
        }

        // Check if already pinned
        if self.pinned_columns.contains(&column_index) {
            return Ok(()); // Already pinned, no-op
        }

        // Check if column exists in source
        if column_index >= self.source.column_count() {
            return Err(anyhow::anyhow!(
                "Column index {} out of bounds",
                column_index
            ));
        }

        // Remove from visible columns if present
        self.visible_columns.retain(|&idx| idx != column_index);

        // Add to pinned columns
        self.pinned_columns.push(column_index);

        Ok(())
    }

    /// Pin a column by name
    pub fn pin_column_by_name(&mut self, column_name: &str) -> Result<()> {
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            self.pin_column(col_idx)
        } else {
            Err(anyhow::anyhow!("Column '{}' not found", column_name))
        }
    }

    /// Unpin a column (move it back to regular visible columns)
    pub fn unpin_column(&mut self, column_index: usize) -> bool {
        if let Some(pos) = self
            .pinned_columns
            .iter()
            .position(|&idx| idx == column_index)
        {
            self.pinned_columns.remove(pos);

            // Add back to visible columns (at the beginning of non-pinned area)
            if !self.visible_columns.contains(&column_index) {
                self.visible_columns.insert(0, column_index);
            }

            true
        } else {
            false // Not pinned
        }
    }

    /// Unpin a column by name
    pub fn unpin_column_by_name(&mut self, column_name: &str) -> bool {
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            self.unpin_column(col_idx)
        } else {
            false
        }
    }

    /// Clear all pinned columns
    pub fn clear_pinned_columns(&mut self) {
        // Move all pinned columns back to visible
        for col_idx in self.pinned_columns.drain(..) {
            if !self.visible_columns.contains(&col_idx) {
                self.visible_columns.push(col_idx);
            }
        }
    }

    /// Check if a column is pinned
    pub fn is_column_pinned(&self, column_index: usize) -> bool {
        self.pinned_columns.contains(&column_index)
    }

    /// Get pinned column indices
    pub fn get_pinned_columns(&self) -> &[usize] {
        &self.pinned_columns
    }

    /// Get the names of pinned columns
    pub fn get_pinned_column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        self.pinned_columns
            .iter()
            .filter_map(|&idx| all_columns.get(idx).cloned())
            .collect()
    }

    /// Get display order of columns (pinned first, then visible)
    pub fn get_display_columns(&self) -> Vec<usize> {
        let mut result = self.pinned_columns.clone();
        result.extend(&self.visible_columns);
        result
    }

    /// Get display column names in order (pinned first, then visible)
    pub fn get_display_column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        self.get_display_columns()
            .iter()
            .filter_map(|&idx| all_columns.get(idx).cloned())
            .collect()
    }

    /// Set maximum number of pinned columns
    pub fn set_max_pinned_columns(&mut self, max: usize) {
        self.max_pinned_columns = max;
        // If we have too many pinned, unpin the extras from the end
        while self.pinned_columns.len() > max {
            if let Some(col_idx) = self.pinned_columns.pop() {
                self.visible_columns.insert(0, col_idx);
            }
        }
    }

    /// Create a view with specific rows
    pub fn with_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = rows.clone();
        self.base_rows = rows; // Update base_rows so clear_filter restores to this
        self
    }

    /// Apply limit and offset
    pub fn with_limit(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = offset;
        self
    }

    /// Filter rows based on a predicate
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&DataTable, usize) -> bool,
    {
        self.visible_rows = self
            .visible_rows
            .into_iter()
            .filter(|&row_idx| predicate(&self.source, row_idx))
            .collect();
        self
    }

    /// Apply a text filter to the view (filters visible rows)
    pub fn apply_text_filter(&mut self, pattern: &str, case_sensitive: bool) {
        info!(
            "DataView::apply_text_filter - pattern='{}', case_sensitive={}, thread={:?}",
            pattern,
            case_sensitive,
            std::thread::current().id()
        );

        if pattern.is_empty() {
            info!("DataView::apply_text_filter - empty pattern, clearing filter");
            self.clear_filter();
            return;
        }

        // Store the filter pattern
        self.filter_pattern = Some(pattern.to_string());

        // Filter from base_rows (not visible_rows) to allow re-filtering
        let pattern_lower = if !case_sensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        info!(
            "DataView::apply_text_filter - searching for '{}' in {} base rows",
            pattern_lower,
            self.base_rows.len()
        );

        let mut matched_count = 0;
        let mut checked_count = 0;

        self.visible_rows = self
            .base_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                checked_count += 1;

                // Check if any cell in the row contains the pattern
                if let Some(row) = self.source.get_row(row_idx) {
                    // Log first few rows for debugging
                    if checked_count <= 3 {
                        let preview = row
                            .values
                            .iter()
                            .take(5)
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        info!(
                            "DataView::apply_text_filter - row {} preview: {}",
                            row_idx, preview
                        );
                    }

                    for value in &row.values {
                        let text = value.to_string();
                        let text_to_match = if !case_sensitive {
                            text.to_lowercase()
                        } else {
                            text.clone()
                        };
                        if text_to_match.contains(&pattern_lower) {
                            matched_count += 1;
                            if checked_count <= 3 {
                                info!(
                                    "DataView::apply_text_filter - MATCH in row {} cell: '{}'",
                                    row_idx, text
                                );
                            }
                            return true;
                        }
                    }
                }
                false
            })
            .collect();

        info!(
            "DataView::apply_text_filter - checked {} rows, matched {} rows",
            checked_count, matched_count
        );
        info!(
            "DataView::apply_text_filter - final visible rows: {}",
            self.visible_rows.len()
        );
    }

    /// Clear the filter and restore all base rows
    pub fn clear_filter(&mut self) {
        self.filter_pattern = None;
        self.visible_rows = self.base_rows.clone();
    }

    /// Check if a filter is active
    pub fn has_filter(&self) -> bool {
        self.filter_pattern.is_some()
    }

    /// Get the current filter pattern
    pub fn get_filter_pattern(&self) -> Option<&str> {
        self.filter_pattern.as_deref()
    }

    /// Apply a fuzzy filter to the view
    /// Supports both fuzzy matching and exact matching (when pattern starts with ')
    pub fn apply_fuzzy_filter(&mut self, pattern: &str, case_insensitive: bool) {
        info!(
            "DataView::apply_fuzzy_filter - pattern='{}', case_insensitive={}, thread={:?}",
            pattern,
            case_insensitive,
            std::thread::current().id()
        );

        if pattern.is_empty() {
            info!("DataView::apply_fuzzy_filter - empty pattern, clearing filter");
            self.clear_filter();
            return;
        }

        // Store the filter pattern
        self.filter_pattern = Some(pattern.to_string());

        // Check if pattern starts with ' for exact matching
        let use_exact = pattern.starts_with('\'');

        self.visible_rows = self
            .base_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                // Get all cell values as a single string for matching
                if let Some(row) = self.source.get_row(row_idx) {
                    // Concatenate all cell values with spaces
                    let row_text = row
                        .values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if use_exact && pattern.len() > 1 {
                        // Exact substring matching (skip the leading ')
                        let exact_pattern = &pattern[1..];
                        if case_insensitive {
                            row_text
                                .to_lowercase()
                                .contains(&exact_pattern.to_lowercase())
                        } else {
                            row_text.contains(exact_pattern)
                        }
                    } else if !use_exact {
                        // Fuzzy matching
                        let matcher = if case_insensitive {
                            SkimMatcherV2::default().ignore_case()
                        } else {
                            SkimMatcherV2::default().respect_case()
                        };

                        // Check if there's a fuzzy match with score > 0
                        matcher
                            .fuzzy_match(&row_text, pattern)
                            .map_or(false, |score| score > 0)
                    } else {
                        // Just a single quote - no pattern to match
                        false
                    }
                } else {
                    false
                }
            })
            .collect();
    }

    /// Get indices of rows that match the fuzzy filter (for compatibility)
    pub fn get_fuzzy_filter_indices(&self) -> Vec<usize> {
        // Return indices relative to the base data, not the view indices
        self.visible_rows.clone()
    }

    /// Get the visible row indices
    pub fn get_visible_rows(&self) -> Vec<usize> {
        self.visible_rows.clone()
    }

    /// Sort rows by a column (consuming version - returns new Self)
    pub fn sort_by(mut self, column_index: usize, ascending: bool) -> Result<Self> {
        self.apply_sort(column_index, ascending)?;
        Ok(self)
    }

    /// Sort rows by a column (mutable version - modifies in place)
    pub fn apply_sort(&mut self, column_index: usize, ascending: bool) -> Result<()> {
        if column_index >= self.source.column_count() {
            return Err(anyhow::anyhow!(
                "Column index {} out of bounds",
                column_index
            ));
        }

        // Update sort state
        self.sort_state.column = Some(column_index);
        self.sort_state.order = if ascending {
            SortOrder::Ascending
        } else {
            SortOrder::Descending
        };

        let source = &self.source;
        self.visible_rows.sort_by(|&a, &b| {
            let val_a = source.get_value(a, column_index);
            let val_b = source.get_value(b, column_index);

            let cmp = match (val_a, val_b) {
                (Some(DataValue::Integer(a)), Some(DataValue::Integer(b))) => a.cmp(&b),
                (Some(DataValue::Float(a)), Some(DataValue::Float(b))) => {
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(DataValue::String(a)), Some(DataValue::String(b))) => a.cmp(&b),
                (Some(DataValue::Boolean(a)), Some(DataValue::Boolean(b))) => a.cmp(&b),
                (Some(DataValue::DateTime(a)), Some(DataValue::DateTime(b))) => a.cmp(&b),
                (Some(DataValue::Null), Some(DataValue::Null)) => std::cmp::Ordering::Equal,
                (Some(DataValue::Null), _) => std::cmp::Ordering::Less,
                (_, Some(DataValue::Null)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };

            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        // Also update base_rows so that clearing filters preserves the sort
        self.base_rows = self.visible_rows.clone();

        Ok(())
    }

    /// Toggle sort on a column - cycles through Ascending -> Descending -> None
    pub fn toggle_sort(&mut self, column_index: usize) -> Result<()> {
        if column_index >= self.source.column_count() {
            return Err(anyhow::anyhow!(
                "Column index {} out of bounds",
                column_index
            ));
        }

        // Determine next sort state
        let next_order = if self.sort_state.column == Some(column_index) {
            // Same column - cycle through states
            match self.sort_state.order {
                SortOrder::None => SortOrder::Ascending,
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::None,
            }
        } else {
            // Different column - start with ascending
            SortOrder::Ascending
        };

        // Apply the sort based on the new state
        match next_order {
            SortOrder::Ascending => self.apply_sort(column_index, true)?,
            SortOrder::Descending => self.apply_sort(column_index, false)?,
            SortOrder::None => {
                self.sort_state.column = None;
                self.sort_state.order = SortOrder::None;
                self.clear_sort();
            }
        }

        Ok(())
    }

    /// Get the current sort state
    pub fn get_sort_state(&self) -> &SortState {
        &self.sort_state
    }

    /// Clear the current sort and restore original row order
    pub fn clear_sort(&mut self) {
        // Clear sort state
        self.sort_state.column = None;
        self.sort_state.order = SortOrder::None;

        let row_count = self.source.row_count();
        self.base_rows = (0..row_count).collect();

        // Reapply any active filter
        if let Some(pattern) = self.filter_pattern.clone() {
            let case_insensitive = false; // Would need to track this
            self.apply_text_filter(&pattern, case_insensitive);
        } else {
            self.visible_rows = self.base_rows.clone();
        }
    }

    // === Virtual Column Management ===

    /// Add a virtual column to the view
    pub fn add_virtual_column(&mut self, virtual_column: VirtualColumn) {
        self.virtual_columns.push(virtual_column);
    }

    /// Add a row number virtual column
    pub fn add_row_numbers(&mut self, position: VirtualColumnPosition) {
        let row_num_column = VirtualColumn {
            name: "#".to_string(),
            generator: Arc::new(|row_index| format!("{}", row_index + 1)),
            width: Some(4), // Room for 4-digit row numbers by default
            position,
        };
        self.add_virtual_column(row_num_column);
    }

    /// Remove all virtual columns of a specific type by name
    pub fn remove_virtual_columns(&mut self, name: &str) {
        self.virtual_columns.retain(|col| col.name != name);
    }

    /// Toggle row numbers on/off
    pub fn toggle_row_numbers(&mut self) {
        if self.virtual_columns.iter().any(|col| col.name == "#") {
            self.remove_virtual_columns("#");
        } else {
            self.add_row_numbers(VirtualColumnPosition::Left);
        }
    }

    /// Check if row numbers are currently shown
    pub fn has_row_numbers(&self) -> bool {
        self.virtual_columns.iter().any(|col| col.name == "#")
    }

    /// Get all column names including virtual columns in display order
    pub fn get_all_column_names(&self) -> Vec<String> {
        let mut result = Vec::new();
        let all_source_names = self.source.column_names();
        // Use get_display_columns() to get columns in correct order (pinned first)
        let real_column_names: Vec<String> = self
            .get_display_columns()
            .iter()
            .map(|&i| {
                all_source_names
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("col_{}", i))
            })
            .collect();

        // Insert virtual columns at their specified positions
        let mut virtual_left = Vec::new();
        let mut virtual_right = Vec::new();
        let mut virtual_indexed = Vec::new();

        for vcol in &self.virtual_columns {
            match vcol.position {
                VirtualColumnPosition::Left => virtual_left.push(vcol.name.clone()),
                VirtualColumnPosition::Right => virtual_right.push(vcol.name.clone()),
                VirtualColumnPosition::Index(idx) => virtual_indexed.push((idx, vcol.name.clone())),
            }
        }

        // Add left virtual columns
        result.extend(virtual_left);

        // Add real columns with indexed virtual columns interspersed
        for (i, real_name) in real_column_names.into_iter().enumerate() {
            // Add any virtual columns that should appear at this index
            for (idx, vname) in &virtual_indexed {
                if *idx == i {
                    result.push(vname.clone());
                }
            }
            result.push(real_name);
        }

        // Add right virtual columns
        result.extend(virtual_right);

        result
    }

    /// Get the number of visible rows
    pub fn row_count(&self) -> usize {
        let count = self.visible_rows.len();

        // Apply limit if set
        if let Some(limit) = self.limit {
            let available = count.saturating_sub(self.offset);
            available.min(limit)
        } else {
            count.saturating_sub(self.offset)
        }
    }

    /// Get the number of visible columns (including pinned and virtual)
    pub fn column_count(&self) -> usize {
        self.pinned_columns.len() + self.visible_columns.len() + self.virtual_columns.len()
    }

    /// Get column names for visible columns (including virtual columns in correct positions)
    pub fn column_names(&self) -> Vec<String> {
        self.get_all_column_names()
    }

    /// Get a row by index (respecting limit/offset) including virtual columns
    pub fn get_row(&self, index: usize) -> Option<DataRow> {
        let actual_index = index + self.offset;

        // Check if within limit
        if let Some(limit) = self.limit {
            if index >= limit {
                return None;
            }
        }

        // Get the actual row index
        let row_idx = *self.visible_rows.get(actual_index)?;

        // Build a row with all columns (real + virtual) in display order
        let mut values = Vec::new();

        // Get real column values
        let mut real_values = Vec::new();
        for &col_idx in self.get_display_columns().iter() {
            let value = self
                .source
                .get_value(row_idx, col_idx)
                .cloned()
                .unwrap_or(DataValue::Null);
            real_values.push(value);
        }

        // Organize virtual columns by position
        let mut virtual_left = Vec::new();
        let mut virtual_right = Vec::new();
        let mut virtual_indexed = Vec::new();

        for vcol in &self.virtual_columns {
            let virtual_value = DataValue::String((vcol.generator)(row_idx));
            match vcol.position {
                VirtualColumnPosition::Left => virtual_left.push(virtual_value),
                VirtualColumnPosition::Right => virtual_right.push(virtual_value),
                VirtualColumnPosition::Index(idx) => virtual_indexed.push((idx, virtual_value)),
            }
        }

        // Add left virtual columns
        values.extend(virtual_left);

        // Add real columns with indexed virtual columns interspersed
        for (i, real_value) in real_values.into_iter().enumerate() {
            // Add any virtual columns that should appear at this index
            for (idx, vvalue) in &virtual_indexed {
                if *idx == i {
                    values.push(vvalue.clone());
                }
            }
            values.push(real_value);
        }

        // Add right virtual columns
        values.extend(virtual_right);

        Some(DataRow::new(values))
    }

    /// Get all visible rows (respecting limit/offset)
    pub fn get_rows(&self) -> Vec<DataRow> {
        let count = self.row_count();
        (0..count).filter_map(|i| self.get_row(i)).collect()
    }

    /// Get the source DataTable
    pub fn source(&self) -> &DataTable {
        &self.source
    }

    /// Check if a column index is visible (either pinned or regular visible)
    pub fn is_column_visible(&self, index: usize) -> bool {
        self.pinned_columns.contains(&index) || self.visible_columns.contains(&index)
    }

    /// Get visible column indices (not including pinned)
    pub fn visible_column_indices(&self) -> &[usize] {
        &self.visible_columns
    }

    /// Get all display column indices (pinned + visible)
    pub fn display_column_indices(&self) -> Vec<usize> {
        self.get_display_columns()
    }

    /// Get visible row indices (before limit/offset)
    pub fn visible_row_indices(&self) -> &[usize] {
        &self.visible_rows
    }

    // ========== Column Search Methods ==========

    /// Start or update column search with a pattern
    pub fn search_columns(&mut self, pattern: &str) {
        self.column_search_pattern = if pattern.is_empty() {
            None
        } else {
            Some(pattern.to_string())
        };

        if pattern.is_empty() {
            self.matching_columns.clear();
            self.current_column_match = 0;
            return;
        }

        // Search through visible columns
        let pattern_lower = pattern.to_lowercase();
        self.matching_columns = self
            .visible_columns
            .iter()
            .enumerate()
            .filter_map(|(visible_idx, &source_idx)| {
                let col_name = &self.source.columns[source_idx].name;
                if col_name.to_lowercase().contains(&pattern_lower) {
                    Some((visible_idx, col_name.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Reset to first match
        self.current_column_match = 0;
    }

    /// Clear column search
    pub fn clear_column_search(&mut self) {
        self.column_search_pattern = None;
        self.matching_columns.clear();
        self.current_column_match = 0;
    }

    /// Go to next column search match
    pub fn next_column_match(&mut self) -> Option<usize> {
        if self.matching_columns.is_empty() {
            return None;
        }

        self.current_column_match = (self.current_column_match + 1) % self.matching_columns.len();
        Some(self.matching_columns[self.current_column_match].0)
    }

    /// Go to previous column search match
    pub fn prev_column_match(&mut self) -> Option<usize> {
        if self.matching_columns.is_empty() {
            return None;
        }

        if self.current_column_match == 0 {
            self.current_column_match = self.matching_columns.len() - 1;
        } else {
            self.current_column_match -= 1;
        }
        Some(self.matching_columns[self.current_column_match].0)
    }

    /// Get current column search pattern
    pub fn column_search_pattern(&self) -> Option<&str> {
        self.column_search_pattern.as_deref()
    }

    /// Get matching columns from search
    pub fn get_matching_columns(&self) -> &[(usize, String)] {
        &self.matching_columns
    }

    /// Get current column match index
    pub fn current_column_match_index(&self) -> usize {
        self.current_column_match
    }

    /// Get current column match (visible column index)
    pub fn get_current_column_match(&self) -> Option<usize> {
        if self.matching_columns.is_empty() {
            None
        } else {
            Some(self.matching_columns[self.current_column_match].0)
        }
    }

    /// Check if column search is active
    pub fn has_column_search(&self) -> bool {
        self.column_search_pattern.is_some()
    }

    /// Get only real column names (excluding virtual columns) in display order
    fn get_real_column_names(&self) -> Vec<String> {
        let all_source_names = self.source.column_names();
        let display_columns = self.get_display_columns();

        display_columns
            .iter()
            .filter_map(|&idx| all_source_names.get(idx).cloned())
            .collect()
    }

    /// Extract only real column values from a row (excluding virtual column values)
    fn extract_real_values_from_row(&self, full_row: &DataRow) -> Vec<DataValue> {
        let mut real_values = Vec::new();
        let mut value_idx = 0;

        // Count left virtual columns to skip
        let left_virtual_count = self
            .virtual_columns
            .iter()
            .filter(|vc| matches!(vc.position, VirtualColumnPosition::Left))
            .count();

        // Skip left virtual columns
        value_idx += left_virtual_count;

        // Collect real column values
        let real_column_count = self.get_display_columns().len();
        for _ in 0..real_column_count {
            if value_idx < full_row.values.len() {
                real_values.push(full_row.values[value_idx].clone());
                value_idx += 1;
            }
        }

        real_values
    }

    /// Export the visible data as JSON
    /// Returns an array of objects where each object represents a row
    pub fn to_json(&self) -> Value {
        // Use only real columns for export, not virtual columns
        let column_names = self.get_real_column_names();
        let mut rows = Vec::new();

        // Iterate through visible rows
        for row_idx in 0..self.row_count() {
            if let Some(full_row) = self.get_row(row_idx) {
                // Extract only the real column values (skip virtual columns)
                let real_values = self.extract_real_values_from_row(&full_row);

                let mut obj = serde_json::Map::new();
                for (col_idx, col_name) in column_names.iter().enumerate() {
                    if let Some(value) = real_values.get(col_idx) {
                        let json_value = match value {
                            DataValue::String(s) => json!(s),
                            DataValue::Integer(i) => json!(i),
                            DataValue::Float(f) => json!(f),
                            DataValue::Boolean(b) => json!(b),
                            DataValue::DateTime(dt) => json!(dt),
                            DataValue::Null => json!(null),
                        };
                        obj.insert(col_name.clone(), json_value);
                    }
                }
                rows.push(json!(obj));
            }
        }

        json!(rows)
    }

    /// Export the visible data as CSV string
    pub fn to_csv(&self) -> Result<String> {
        let mut csv_output = String::new();
        // Use only real columns for export, not virtual columns
        let column_names = self.get_real_column_names();

        // Write header
        csv_output.push_str(&column_names.join(","));
        csv_output.push('\n');

        // Write data rows
        for row_idx in 0..self.row_count() {
            if let Some(full_row) = self.get_row(row_idx) {
                // Extract only the real column values (skip virtual columns)
                let real_values = self.extract_real_values_from_row(&full_row);

                let row_strings: Vec<String> = real_values
                    .iter()
                    .map(|v| {
                        let s = v.to_string();
                        // Quote values that contain commas, quotes, or newlines
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s
                        }
                    })
                    .collect();
                csv_output.push_str(&row_strings.join(","));
                csv_output.push('\n');
            }
        }

        Ok(csv_output)
    }

    /// Export the visible data as TSV (Tab-Separated Values) string
    pub fn to_tsv(&self) -> Result<String> {
        let mut tsv_output = String::new();
        // Use only real columns for export, not virtual columns
        let column_names = self.get_real_column_names();

        // Write header
        tsv_output.push_str(&column_names.join("\t"));
        tsv_output.push('\n');

        // Write data rows
        for row_idx in 0..self.row_count() {
            if let Some(full_row) = self.get_row(row_idx) {
                // Extract only the real column values (skip virtual columns)
                let real_values = self.extract_real_values_from_row(&full_row);

                let row_strings: Vec<String> = real_values.iter().map(|v| v.to_string()).collect();
                tsv_output.push_str(&row_strings.join("\t"));
                tsv_output.push('\n');
            }
        }

        Ok(tsv_output)
    }
}

// Implement DataProvider for compatibility during migration
// This allows DataView to be used where DataProvider is expected
impl DataProvider for DataView {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.get_row(index)
            .map(|row| row.values.iter().map(|v| v.to_string()).collect())
    }

    fn get_column_names(&self) -> Vec<String> {
        self.column_names()
    }

    fn get_row_count(&self) -> usize {
        self.row_count()
    }

    fn get_column_count(&self) -> usize {
        self.column_count()
    }
}

// Also implement Debug for DataView to satisfy DataProvider requirements
impl std::fmt::Debug for DataView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataView")
            .field("source_name", &self.source.name)
            .field("visible_rows", &self.visible_rows.len())
            .field("visible_columns", &self.visible_columns.len())
            .field("has_filter", &self.filter_pattern.is_some())
            .field("has_column_search", &self.column_search_pattern.is_some())
            .finish()
    }
}
