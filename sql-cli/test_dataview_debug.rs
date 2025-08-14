#!/usr/bin/env rust-script
//! DataView Debug Test - Enhanced for debugging
//! 
//! Compile with debug symbols: rustc -g test_dataview_debug.rs
//! Debug with GDB: rust-gdb ./test_dataview_debug
//! Debug with LLDB: rust-lldb ./test_dataview_debug

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

// ========== Minimal DataTable types ==========

#[derive(Debug, Clone)]
pub struct DataColumn {
    pub name: String,
}

impl DataColumn {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone)]
pub enum DataValue {
    String(String),
    Float(f64),
    Boolean(bool),
    Null,
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::String(s) => write!(f, "{}", s),
            DataValue::Float(fl) => write!(f, "{:.2}", fl),
            DataValue::Boolean(b) => write!(f, "{}", b),
            DataValue::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataRow {
    pub values: Vec<DataValue>,
}

impl DataRow {
    pub fn new(values: Vec<DataValue>) -> Self {
        Self { values }
    }
}

#[derive(Debug, Clone)]
pub struct DataTable {
    pub name: String,
    pub columns: Vec<DataColumn>,
    pub rows: Vec<DataRow>,
}

impl DataTable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: DataColumn) {
        self.columns.push(column);
    }

    pub fn add_row(&mut self, row: DataRow) -> Result<(), String> {
        if row.values.len() != self.columns.len() {
            return Err(format!(
                "Row has {} values but table has {} columns",
                row.values.len(),
                self.columns.len()
            ));
        }
        self.rows.push(row);
        Ok(())
    }

    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&DataValue> {
        self.rows.get(row)?.values.get(col)
    }

    pub fn get_row(&self, index: usize) -> Option<&DataRow> {
        self.rows.get(index)
    }
}

// ========== DataView with Debug helpers ==========

#[derive(Debug)]
pub struct DataView {
    source: Arc<DataTable>,
    visible_rows: Vec<usize>,
    visible_columns: Vec<usize>,
    base_rows: Vec<usize>,
    base_columns: Vec<usize>,
    filter_pattern: Option<String>,
    column_search_pattern: Option<String>,
    matching_columns: Vec<(usize, String)>,
    current_column_match: usize,
}

impl DataView {
    pub fn new(source: Arc<DataTable>) -> Self {
        let row_count = source.row_count();
        let col_count = source.column_count();
        let all_rows: Vec<usize> = (0..row_count).collect();
        let all_columns: Vec<usize> = (0..col_count).collect();

        Self {
            source,
            visible_rows: all_rows.clone(),
            visible_columns: all_columns.clone(),
            base_rows: all_rows,
            base_columns: all_columns,
            filter_pattern: None,
            column_search_pattern: None,
            matching_columns: Vec::new(),
            current_column_match: 0,
        }
    }

    // Debug helper: Print internal state
    pub fn debug_state(&self, label: &str) {
        println!("\n🔍 DEBUG: {} ", label);
        println!("  visible_rows: {:?}", self.visible_rows);
        println!("  visible_columns: {:?}", self.visible_columns);
        println!("  filter_pattern: {:?}", self.filter_pattern);
        println!("  column_search_pattern: {:?}", self.column_search_pattern);
        println!("  matching_columns: {:?}", self.matching_columns);
        println!("  current_column_match: {}", self.current_column_match);
    }

    // Debug helper: Print visible data
    pub fn debug_visible_data(&self) {
        println!("\n📊 Visible Data:");
        let col_names = self.column_names();
        println!("  Headers: {}", col_names.join(" | "));
        println!("  {}", "-".repeat(50));
        
        for i in 0..self.row_count().min(10) {
            if let Some(row) = self.get_row(i) {
                let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
                println!("  Row {}: {}", i, values.join(" | "));
            }
        }
        if self.row_count() > 10 {
            println!("  ... ({} more rows)", self.row_count() - 10);
        }
    }

    pub fn row_count(&self) -> usize {
        self.visible_rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.visible_columns.len()
    }

    pub fn column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        self.visible_columns
            .iter()
            .filter_map(|&idx| all_columns.get(idx).cloned())
            .collect()
    }

    pub fn get_row(&self, index: usize) -> Option<DataRow> {
        let row_idx = *self.visible_rows.get(index)?;
        let mut values = Vec::new();
        for &col_idx in &self.visible_columns {
            let value = self
                .source
                .get_value(row_idx, col_idx)
                .cloned()
                .unwrap_or(DataValue::Null);
            values.push(value);
        }
        Some(DataRow::new(values))
    }

    // Column search with debug output
    pub fn search_columns(&mut self, pattern: &str) {
        println!("\n🔎 Searching columns for '{}'", pattern);
        
        self.column_search_pattern = if pattern.is_empty() {
            None
        } else {
            Some(pattern.to_string())
        };

        if pattern.is_empty() {
            self.matching_columns.clear();
            self.current_column_match = 0;
            println!("  Cleared search");
            return;
        }

        let pattern_lower = pattern.to_lowercase();
        self.matching_columns = self
            .visible_columns
            .iter()
            .enumerate()
            .filter_map(|(visible_idx, &source_idx)| {
                let col_name = &self.source.columns[source_idx].name;
                if col_name.to_lowercase().contains(&pattern_lower) {
                    println!("  ✓ Match: column '{}' at index {}", col_name, visible_idx);
                    Some((visible_idx, col_name.clone()))
                } else {
                    None
                }
            })
            .collect();

        self.current_column_match = 0;
        println!("  Total matches: {}", self.matching_columns.len());
    }

    pub fn clear_column_search(&mut self) {
        self.column_search_pattern = None;
        self.matching_columns.clear();
        self.current_column_match = 0;
    }

    pub fn next_column_match(&mut self) -> Option<usize> {
        if self.matching_columns.is_empty() {
            return None;
        }
        self.current_column_match = (self.current_column_match + 1) % self.matching_columns.len();
        let result = self.matching_columns[self.current_column_match].0;
        println!("  → Next match: index {} ('{}')", result, self.matching_columns[self.current_column_match].1);
        Some(result)
    }

    pub fn get_matching_columns(&self) -> &[(usize, String)] {
        &self.matching_columns
    }

    pub fn get_current_column_match(&self) -> Option<usize> {
        if self.matching_columns.is_empty() {
            None
        } else {
            Some(self.matching_columns[self.current_column_match].0)
        }
    }

    pub fn has_column_search(&self) -> bool {
        self.column_search_pattern.is_some()
    }

    // Text filtering with debug output
    pub fn apply_text_filter(&mut self, pattern: &str) {
        println!("\n🔽 Applying text filter: '{}'", pattern);
        
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        self.filter_pattern = Some(pattern.to_string());
        let pattern_lower = pattern.to_lowercase();

        let before_count = self.visible_rows.len();
        
        self.visible_rows = self
            .base_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                if let Some(row) = self.source.get_row(row_idx) {
                    for value in &row.values {
                        let text = value.to_string().to_lowercase();
                        if text.contains(&pattern_lower) {
                            return true;
                        }
                    }
                }
                false
            })
            .collect();
        
        let after_count = self.visible_rows.len();
        println!("  Filtered: {} → {} rows", before_count, after_count);
    }

    pub fn clear_filter(&mut self) {
        println!("\n🔼 Clearing filter");
        let before = self.visible_rows.len();
        self.filter_pattern = None;
        self.visible_rows = self.base_rows.clone();
        let after = self.visible_rows.len();
        println!("  Restored: {} → {} rows", before, after);
    }

    // Column visibility with debug
    pub fn hide_column_by_name(&mut self, column_name: &str) {
        println!("\n👁️  Hiding column: '{}'", column_name);
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            let before = self.visible_columns.len();
            self.visible_columns.retain(|&idx| idx != col_idx);
            let after = self.visible_columns.len();
            println!("  Columns: {} → {}", before, after);
        } else {
            println!("  ⚠️  Column not found!");
        }
    }

    pub fn unhide_all_columns(&mut self) {
        self.visible_columns = self.base_columns.clone();
    }

    pub fn get_hidden_column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        let visible_columns = self.column_names();
        
        all_columns
            .into_iter()
            .filter(|col| !visible_columns.contains(col))
            .collect()
    }

    // Sorting with debug
    pub fn apply_sort(&mut self, column_index: usize, ascending: bool) {
        let col_name = self.source.columns.get(column_index)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "?".to_string());
        
        println!("\n🔄 Sorting by column {} ('{}') {}", 
                 column_index, col_name, 
                 if ascending { "ascending" } else { "descending" });
        
        if column_index >= self.source.column_count() {
            println!("  ⚠️  Column index out of bounds!");
            return;
        }

        let source = &self.source;
        self.visible_rows.sort_by(|&a, &b| {
            let val_a = source.get_value(a, column_index);
            let val_b = source.get_value(b, column_index);

            let cmp = match (val_a, val_b) {
                (Some(DataValue::Float(a)), Some(DataValue::Float(b))) => {
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(DataValue::String(a)), Some(DataValue::String(b))) => a.cmp(&b),
                (Some(DataValue::Boolean(a)), Some(DataValue::Boolean(b))) => a.cmp(&b),
                _ => std::cmp::Ordering::Equal,
            };

            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        self.base_rows = self.visible_rows.clone();
        println!("  ✓ Sort complete");
    }
}

// ========== Interactive Debug Test ==========

fn pause(msg: &str) {
    println!("\n⏸️  {}", msg);
    println!("   Press Enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
}

fn main() {
    println!("=== DataView Debug Test ===");
    println!("This version includes debug output and pause points");
    println!("Perfect for stepping through with a debugger!\n");

    // Create test data
    let mut table = DataTable::new("test_data");
    
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("active"));

    // Add sample rows
    let rows = vec![
        (1, "Alice", 100.50, "Sales", true),
        (2, "Bob", 200.75, "Marketing", false),
        (3, "Charlie", 150.25, "Sales", true),
        (4, "David", 300.00, "Engineering", true),
        (5, "Eve", 175.50, "Marketing", false),
        (6, "Frank", 250.00, "Sales", false),
        (7, "Grace", 180.00, "Engineering", false),
    ];

    for (id, name, amount, category, active) in rows {
        let row = DataRow::new(vec![
            DataValue::String(id.to_string()),
            DataValue::String(name.to_string()),
            DataValue::Float(amount),
            DataValue::String(category.to_string()),
            DataValue::Boolean(active),
        ]);
        table.add_row(row).unwrap();
    }

    let table_arc = Arc::new(table);
    let mut view = DataView::new(table_arc);

    println!("📋 Initial DataView created");
    view.debug_state("Initial state");
    view.debug_visible_data();
    
    pause("Ready to test column search");

    // Test 1: Column Search
    println!("\n═══ Test 1: Column Search ═══");
    
    view.search_columns("a");
    view.debug_state("After searching for 'a'");
    
    view.next_column_match();
    view.next_column_match();
    
    view.clear_column_search();
    view.debug_state("After clearing search");
    
    pause("Ready to test filtering");

    // Test 2: Text Filtering
    println!("\n═══ Test 2: Text Filtering ═══");
    
    view.apply_text_filter("Sales");
    view.debug_state("After filtering for 'Sales'");
    view.debug_visible_data();
    
    view.clear_filter();
    view.debug_state("After clearing filter");
    
    pause("Ready to test sorting");

    // Test 3: Sorting
    println!("\n═══ Test 3: Sorting ═══");
    
    view.apply_sort(2, false); // Sort by amount descending
    view.debug_state("After sorting by amount DESC");
    view.debug_visible_data();
    
    pause("Ready to test combined operations");

    // Test 4: Combined Operations
    println!("\n═══ Test 4: Combined Operations ═══");
    
    println!("Combining: Sort → Filter → Column Search");
    
    view.apply_sort(2, false);
    println!("Step 1: Sorted by amount");
    
    view.apply_text_filter("Sales");
    println!("Step 2: Filtered for Sales");
    
    view.search_columns("name");
    println!("Step 3: Searched columns for 'name'");
    
    view.debug_state("After all operations");
    view.debug_visible_data();
    
    // Test 5: Edge Cases
    println!("\n═══ Test 5: Edge Cases ═══");
    
    println!("Testing empty search:");
    view.search_columns("");
    
    println!("Testing non-existent column:");
    view.hide_column_by_name("nonexistent");
    
    println!("Testing filter with no matches:");
    view.apply_text_filter("XYZ123");
    view.debug_visible_data();
    
    println!("\n✅ All tests complete!");
    println!("\n🐛 Debugging tips:");
    println!("  1. Set breakpoints in search_columns() to see matching logic");
    println!("  2. Set breakpoints in apply_text_filter() to see filtering");
    println!("  3. Watch visible_rows and visible_columns arrays change");
    println!("  4. Use debug_state() anywhere to inspect internal state");
}