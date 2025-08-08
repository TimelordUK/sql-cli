use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents the data type of a column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Null,
    Mixed, // For columns with mixed types
}

impl DataType {
    /// Infer type from a string value
    pub fn infer_from_string(value: &str) -> Self {
        if value.is_empty() || value.eq_ignore_ascii_case("null") {
            return DataType::Null;
        }

        // Try parsing as boolean
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            return DataType::Boolean;
        }

        // Try parsing as integer
        if value.parse::<i64>().is_ok() {
            return DataType::Integer;
        }

        // Try parsing as float
        if value.parse::<f64>().is_ok() {
            return DataType::Float;
        }

        // Check if it looks like a date/time
        // Simple heuristic - contains dashes or colons in expected positions
        if (value.contains('-') && value.len() >= 8) || (value.contains(':') && value.len() >= 5) {
            // TODO: Proper date/time parsing
            return DataType::DateTime;
        }

        DataType::String
    }

    /// Merge two types (for columns with mixed types)
    pub fn merge(&self, other: &DataType) -> DataType {
        if self == other {
            return self.clone();
        }

        match (self, other) {
            (DataType::Null, t) | (t, DataType::Null) => t.clone(),
            (DataType::Integer, DataType::Float) | (DataType::Float, DataType::Integer) => {
                DataType::Float
            }
            _ => DataType::Mixed,
        }
    }
}

/// Column metadata and definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataColumn {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub unique_values: Option<usize>,
    pub null_count: usize,
    pub metadata: HashMap<String, String>,
}

impl DataColumn {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DataType::String,
            nullable: true,
            unique_values: None,
            null_count: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }

    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}

/// A single cell value in the table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(String), // Store as ISO 8601 string for now
    Null,
}

impl DataValue {
    pub fn from_string(s: &str, data_type: &DataType) -> Self {
        if s.is_empty() || s.eq_ignore_ascii_case("null") {
            return DataValue::Null;
        }

        match data_type {
            DataType::String => DataValue::String(s.to_string()),
            DataType::Integer => s
                .parse::<i64>()
                .map(DataValue::Integer)
                .unwrap_or_else(|_| DataValue::String(s.to_string())),
            DataType::Float => s
                .parse::<f64>()
                .map(DataValue::Float)
                .unwrap_or_else(|_| DataValue::String(s.to_string())),
            DataType::Boolean => {
                let lower = s.to_lowercase();
                DataValue::Boolean(lower == "true" || lower == "1" || lower == "yes")
            }
            DataType::DateTime => DataValue::DateTime(s.to_string()),
            DataType::Null => DataValue::Null,
            DataType::Mixed => {
                // Try to infer for mixed columns
                let inferred = DataType::infer_from_string(s);
                Self::from_string(s, &inferred)
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DataValue::String(s) => s.clone(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.clone(),
            DataValue::Null => String::new(),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, DataValue::Null)
    }

    pub fn data_type(&self) -> DataType {
        match self {
            DataValue::String(_) => DataType::String,
            DataValue::Integer(_) => DataType::Integer,
            DataValue::Float(_) => DataType::Float,
            DataValue::Boolean(_) => DataType::Boolean,
            DataValue::DateTime(_) => DataType::DateTime,
            DataValue::Null => DataType::Null,
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// A row of data in the table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRow {
    pub values: Vec<DataValue>,
}

impl DataRow {
    pub fn new(values: Vec<DataValue>) -> Self {
        Self { values }
    }

    pub fn get(&self, index: usize) -> Option<&DataValue> {
        self.values.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut DataValue> {
        self.values.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// The main DataTable structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTable {
    pub name: String,
    pub columns: Vec<DataColumn>,
    pub rows: Vec<DataRow>,
    pub metadata: HashMap<String, String>,
}

impl DataTable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, column: DataColumn) -> &mut Self {
        self.columns.push(column);
        self
    }

    pub fn add_row(&mut self, row: DataRow) -> Result<(), String> {
        if row.len() != self.columns.len() {
            return Err(format!(
                "Row has {} values but table has {} columns",
                row.len(),
                self.columns.len()
            ));
        }
        self.rows.push(row);
        Ok(())
    }

    pub fn get_column(&self, name: &str) -> Option<&DataColumn> {
        self.columns.iter().find(|c| c.name == name)
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

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get column names as a vector
    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    /// Infer and update column types based on data
    pub fn infer_column_types(&mut self) {
        for (col_idx, column) in self.columns.iter_mut().enumerate() {
            let mut inferred_type = DataType::Null;
            let mut null_count = 0;
            let mut unique_values = std::collections::HashSet::new();

            for row in &self.rows {
                if let Some(value) = row.get(col_idx) {
                    if value.is_null() {
                        null_count += 1;
                    } else {
                        let value_type = value.data_type();
                        inferred_type = inferred_type.merge(&value_type);
                        unique_values.insert(value.to_string());
                    }
                }
            }

            column.data_type = inferred_type;
            column.null_count = null_count;
            column.nullable = null_count > 0;
            column.unique_values = Some(unique_values.len());
        }
    }

    /// Get a value at specific row and column
    pub fn get_value(&self, row: usize, col: usize) -> Option<&DataValue> {
        self.rows.get(row)?.get(col)
    }

    /// Get a value by row index and column name
    pub fn get_value_by_name(&self, row: usize, col_name: &str) -> Option<&DataValue> {
        let col_idx = self.get_column_index(col_name)?;
        self.get_value(row, col_idx)
    }

    /// Convert to a vector of string vectors (for display/compatibility)
    pub fn to_string_table(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| row.values.iter().map(|v| v.to_string()).collect())
            .collect()
    }

    /// Get table statistics
    pub fn get_stats(&self) -> DataTableStats {
        DataTableStats {
            row_count: self.row_count(),
            column_count: self.column_count(),
            memory_size: self.estimate_memory_size(),
            null_count: self.columns.iter().map(|c| c.null_count).sum(),
        }
    }

    /// Generate a debug dump string for display
    pub fn debug_dump(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("DataTable: {}\n", self.name));
        output.push_str(&format!(
            "Rows: {} | Columns: {}\n",
            self.row_count(),
            self.column_count()
        ));

        if !self.metadata.is_empty() {
            output.push_str("Metadata:\n");
            for (key, value) in &self.metadata {
                output.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        output.push_str("\nColumns:\n");
        for column in &self.columns {
            output.push_str(&format!("  {} ({:?})", column.name, column.data_type));
            if column.nullable {
                output.push_str(&format!(" - nullable, {} nulls", column.null_count));
            }
            if let Some(unique) = column.unique_values {
                output.push_str(&format!(", {} unique", unique));
            }
            output.push('\n');
        }

        // Show first few rows
        if self.row_count() > 0 {
            let sample_size = 5.min(self.row_count());
            output.push_str(&format!("\nFirst {} rows:\n", sample_size));

            for row_idx in 0..sample_size {
                output.push_str(&format!("  [{}]: ", row_idx));
                for (col_idx, value) in self.rows[row_idx].values.iter().enumerate() {
                    if col_idx > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&value.to_string());
                }
                output.push('\n');
            }
        }

        output
    }

    fn estimate_memory_size(&self) -> usize {
        // Rough estimate of memory usage
        std::mem::size_of::<Self>()
            + self.columns.len() * std::mem::size_of::<DataColumn>()
            + self.rows.len() * std::mem::size_of::<DataRow>()
            + self
                .rows
                .iter()
                .map(|r| r.values.len() * std::mem::size_of::<DataValue>())
                .sum::<usize>()
    }
}

/// Statistics about a DataTable
#[derive(Debug, Clone)]
pub struct DataTableStats {
    pub row_count: usize,
    pub column_count: usize,
    pub memory_size: usize,
    pub null_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_inference() {
        assert_eq!(DataType::infer_from_string("123"), DataType::Integer);
        assert_eq!(DataType::infer_from_string("123.45"), DataType::Float);
        assert_eq!(DataType::infer_from_string("true"), DataType::Boolean);
        assert_eq!(DataType::infer_from_string("hello"), DataType::String);
        assert_eq!(DataType::infer_from_string(""), DataType::Null);
        assert_eq!(
            DataType::infer_from_string("2024-01-01"),
            DataType::DateTime
        );
    }

    #[test]
    fn test_datatable_creation() {
        let mut table = DataTable::new("test");

        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("name").with_type(DataType::String));
        table.add_column(DataColumn::new("active").with_type(DataType::Boolean));

        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 0);

        let row = DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Alice".to_string()),
            DataValue::Boolean(true),
        ]);

        table.add_row(row).unwrap();
        assert_eq!(table.row_count(), 1);

        let value = table.get_value_by_name(0, "name").unwrap();
        assert_eq!(value.to_string(), "Alice");
    }

    #[test]
    fn test_type_inference() {
        let mut table = DataTable::new("test");

        // Add columns without types
        table.add_column(DataColumn::new("mixed"));

        // Add rows with different types
        table
            .add_row(DataRow::new(vec![DataValue::Integer(1)]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![DataValue::Float(2.5)]))
            .unwrap();
        table.add_row(DataRow::new(vec![DataValue::Null])).unwrap();

        table.infer_column_types();

        // Should infer Float since we have both Integer and Float
        assert_eq!(table.columns[0].data_type, DataType::Float);
        assert_eq!(table.columns[0].null_count, 1);
        assert!(table.columns[0].nullable);
    }
}
