#!/usr/bin/env rust-script
//! Simple DataView test without external dependencies
//! Run with: rustc test_dataview_simple.rs && ./test_dataview_simple

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
            DataValue::Float(fl) => write!(f, "{}", fl),
            DataValue::Boolean(b) => write!(f, "{}", b),
            DataValue::Null => write!(f, ""),
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

// ========== Simplified DataView ==========

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

    // Column search methods
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

        self.current_column_match = 0;
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
        Some(self.matching_columns[self.current_column_match].0)
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

    // Simple text filtering (case-insensitive)
    pub fn apply_text_filter(&mut self, pattern: &str) {
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        self.filter_pattern = Some(pattern.to_string());
        let pattern_lower = pattern.to_lowercase();

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
    }

    pub fn clear_filter(&mut self) {
        self.filter_pattern = None;
        self.visible_rows = self.base_rows.clone();
    }

    // Column visibility
    pub fn hide_column_by_name(&mut self, column_name: &str) {
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            self.visible_columns.retain(|&idx| idx != col_idx);
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

    // Simple sorting
    pub fn apply_sort(&mut self, column_index: usize, ascending: bool) {
        if column_index >= self.source.column_count() {
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
    }
}

// ========== Test Program ==========

fn main() {
    println!("=== DataView Simple Test (No Dependencies) ===\n");
    println!("To debug: Set breakpoints in this file and run in debugger");
    println!("Example: rust-gdb ./test_dataview_simple");
    println!("Or in VSCode: Add breakpoints and run with F5\n");

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

    println!("Initial state:");
    println!("  Rows: {}", view.row_count());
    println!("  Columns: {}", view.column_count());
    println!("  Column names: {:?}\n", view.column_names());

    // Test 1: Column Search
    println!("Test 1: Column Search");
    println!("  Searching for 'a'...");
    view.search_columns("a");
    println!("  Matches: {:?}", view.get_matching_columns());
    println!("  Current match index: {:?}", view.get_current_column_match());
    
    println!("  Going to next match...");
    let next = view.next_column_match();
    println!("  Next match index: {:?}", next);
    
    println!("  Clearing search...");
    view.clear_column_search();
    println!("  Has search: {}\n", view.has_column_search());

    // Test 2: Text Filtering
    println!("Test 2: Text Filtering");
    println!("  Filtering for 'Sales'...");
    view.apply_text_filter("Sales");
    println!("  Visible rows after filter: {}", view.row_count());
    for i in 0..view.row_count() {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    Row {}: {}", i, values.join(" | "));
        }
    }
    
    println!("  Clearing filter...");
    view.clear_filter();
    println!("  Visible rows after clear: {}\n", view.row_count());

    // Test 3: Sorting
    println!("Test 3: Sorting");
    println!("  Sorting by amount (column 2) descending...");
    view.apply_sort(2, false);
    println!("  First 3 rows after sort:");
    for i in 0..3.min(view.row_count()) {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    {}", values.join(" | "));
        }
    }
    println!();

    // Test 4: Column Visibility
    println!("Test 4: Column Visibility");
    println!("  Current columns: {:?}", view.column_names());
    println!("  Hiding 'active' column...");
    view.hide_column_by_name("active");
    println!("  Visible columns: {:?}", view.column_names());
    println!("  Hidden columns: {:?}", view.get_hidden_column_names());
    
    println!("  Unhiding all columns...");
    view.unhide_all_columns();
    println!("  Visible columns: {:?}\n", view.column_names());

    // Test 5: Combined Operations
    println!("Test 5: Combined Operations");
    println!("  1. Sort by amount descending");
    view.apply_sort(2, false);
    println!("  2. Filter for 'Sales'");
    view.apply_text_filter("Sales");
    println!("  3. Search columns for 'name'");
    view.search_columns("name");
    
    println!("\n  Results:");
    println!("    Visible rows: {} (sorted Sales people)", view.row_count());
    println!("    Column matches: {:?}", view.get_matching_columns());
    println!("    First row should be Charlie (highest Sales amount):");
    if let Some(row) = view.get_row(0) {
        let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
        println!("    {}", values.join(" | "));
    }

    println!("\n=== All Tests Complete ===");
    println!("You can now use this code in your debugger!");
    println!("Compile with: rustc test_dataview_simple.rs");
    println!("Run with: ./test_dataview_simple");
}