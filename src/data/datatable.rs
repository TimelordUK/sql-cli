use crate::api_client::QueryResponse;
use crate::data::data_provider::DataProvider;
use crate::data::type_inference::{InferredType, TypeInference};
use serde::de::{VariantAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tracing::debug;

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
    #[must_use]
    pub fn infer_from_string(value: &str) -> Self {
        // Handle explicit null string
        if value.eq_ignore_ascii_case("null") {
            return DataType::Null;
        }

        // Use the shared type inference logic
        match TypeInference::infer_from_string(value) {
            InferredType::Null => DataType::Null,
            InferredType::Boolean => DataType::Boolean,
            InferredType::Integer => DataType::Integer,
            InferredType::Float => DataType::Float,
            InferredType::DateTime => DataType::DateTime,
            InferredType::String => DataType::String,
        }
    }

    /// Check if a string looks like a datetime value
    /// Delegates to shared type inference logic
    fn looks_like_datetime(value: &str) -> bool {
        TypeInference::looks_like_datetime(value)
    }

    /// Merge two types (for columns with mixed types)
    #[must_use]
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
    /// Qualified name with table prefix (e.g., "messages.field_name")
    pub qualified_name: Option<String>,
    /// Source table or CTE name
    pub source_table: Option<String>,
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
            qualified_name: None,
            source_table: None,
        }
    }

    #[must_use]
    pub fn with_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }

    /// Set the qualified name (table.column format)
    #[must_use]
    pub fn with_qualified_name(mut self, table_name: &str) -> Self {
        self.qualified_name = Some(format!("{}.{}", table_name, self.name));
        self.source_table = Some(table_name.to_string());
        self
    }

    /// Get the qualified name if available, otherwise return the simple name
    pub fn get_qualified_or_simple_name(&self) -> &str {
        self.qualified_name.as_deref().unwrap_or(&self.name)
    }

    #[must_use]
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}

/// A single cell value in the table
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DataValue {
    String(String),
    InternedString(Arc<String>), // For repeated strings (e.g., status, trader names)
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(String), // Store as ISO 8601 string for now
    Vector(Vec<f64>), // For vector mathematics (physics, geometry, etc.)
    Null,
}

// Custom Hash implementation for DataValue to handle f64
impl std::hash::Hash for DataValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DataValue::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            DataValue::InternedString(s) => {
                1u8.hash(state);
                s.hash(state);
            }
            DataValue::Integer(i) => {
                2u8.hash(state);
                i.hash(state);
            }
            DataValue::Float(f) => {
                3u8.hash(state);
                // Hash the bits of the float for consistency
                f.to_bits().hash(state);
            }
            DataValue::Boolean(b) => {
                4u8.hash(state);
                b.hash(state);
            }
            DataValue::DateTime(dt) => {
                5u8.hash(state);
                dt.hash(state);
            }
            DataValue::Vector(v) => {
                6u8.hash(state);
                // Hash each float's bits
                for f in v {
                    f.to_bits().hash(state);
                }
            }
            DataValue::Null => {
                7u8.hash(state);
            }
        }
    }
}

// Custom Serialize implementation for DataValue to handle Arc<String>
impl Serialize for DataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DataValue::String(s) => {
                serializer.serialize_newtype_variant("DataValue", 0, "String", s)
            }
            DataValue::InternedString(arc_s) => {
                // Serialize the Arc<String> as just the String content
                serializer.serialize_newtype_variant(
                    "DataValue",
                    1,
                    "InternedString",
                    arc_s.as_ref(),
                )
            }
            DataValue::Integer(i) => {
                serializer.serialize_newtype_variant("DataValue", 2, "Integer", i)
            }
            DataValue::Float(f) => serializer.serialize_newtype_variant("DataValue", 3, "Float", f),
            DataValue::Boolean(b) => {
                serializer.serialize_newtype_variant("DataValue", 4, "Boolean", b)
            }
            DataValue::DateTime(dt) => {
                serializer.serialize_newtype_variant("DataValue", 5, "DateTime", dt)
            }
            DataValue::Vector(v) => {
                serializer.serialize_newtype_variant("DataValue", 6, "Vector", v)
            }
            DataValue::Null => serializer.serialize_unit_variant("DataValue", 7, "Null"),
        }
    }
}

// Custom Deserialize implementation for DataValue to handle Arc<String>
impl<'de> Deserialize<'de> for DataValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "PascalCase")]
        enum Field {
            String,
            InternedString,
            Integer,
            Float,
            Boolean,
            DateTime,
            Vector,
            Null,
        }

        struct DataValueVisitor;

        impl<'de> Visitor<'de> for DataValueVisitor {
            type Value = DataValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum DataValue")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                let (field, variant) = data.variant()?;
                match field {
                    Field::String => {
                        let s: String = variant.newtype_variant()?;
                        Ok(DataValue::String(s))
                    }
                    Field::InternedString => {
                        let s: String = variant.newtype_variant()?;
                        Ok(DataValue::InternedString(Arc::new(s)))
                    }
                    Field::Integer => {
                        let i: i64 = variant.newtype_variant()?;
                        Ok(DataValue::Integer(i))
                    }
                    Field::Float => {
                        let f: f64 = variant.newtype_variant()?;
                        Ok(DataValue::Float(f))
                    }
                    Field::Boolean => {
                        let b: bool = variant.newtype_variant()?;
                        Ok(DataValue::Boolean(b))
                    }
                    Field::DateTime => {
                        let dt: String = variant.newtype_variant()?;
                        Ok(DataValue::DateTime(dt))
                    }
                    Field::Vector => {
                        let v: Vec<f64> = variant.newtype_variant()?;
                        Ok(DataValue::Vector(v))
                    }
                    Field::Null => {
                        variant.unit_variant()?;
                        Ok(DataValue::Null)
                    }
                }
            }
        }

        deserializer.deserialize_enum(
            "DataValue",
            &[
                "String",
                "InternedString",
                "Integer",
                "Float",
                "Boolean",
                "DateTime",
                "Vector",
                "Null",
            ],
            DataValueVisitor,
        )
    }
}

// Custom Eq implementation for DataValue
impl Eq for DataValue {}

impl DataValue {
    pub fn from_string(s: &str, data_type: &DataType) -> Self {
        if s.is_empty() || s.eq_ignore_ascii_case("null") {
            return DataValue::Null;
        }

        match data_type {
            DataType::String => DataValue::String(s.to_string()),
            DataType::Integer => s
                .parse::<i64>()
                .map_or_else(|_| DataValue::String(s.to_string()), DataValue::Integer),
            DataType::Float => s
                .parse::<f64>()
                .map_or_else(|_| DataValue::String(s.to_string()), DataValue::Float),
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

    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, DataValue::Null)
    }

    #[must_use]
    pub fn data_type(&self) -> DataType {
        match self {
            DataValue::String(_) | DataValue::InternedString(_) => DataType::String,
            DataValue::Integer(_) => DataType::Integer,
            DataValue::Float(_) => DataType::Float,
            DataValue::Boolean(_) => DataType::Boolean,
            DataValue::DateTime(_) => DataType::DateTime,
            DataValue::Vector(_) => DataType::String, // Display as string "[x,y,z]"
            DataValue::Null => DataType::Null,
        }
    }

    /// Get string representation without allocation when possible
    /// Returns owned String for compatibility but tries to reuse existing strings
    #[must_use]
    pub fn to_string_optimized(&self) -> String {
        match self {
            DataValue::String(s) => s.clone(), // Clone existing string
            DataValue::InternedString(s) => s.as_ref().clone(), // Clone from Rc
            DataValue::DateTime(s) => s.clone(), // Clone existing string
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            DataValue::Vector(v) => {
                // Format as "[x,y,z]"
                let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                format!("[{}]", components.join(","))
            }
            DataValue::Null => String::new(), // Empty string, minimal allocation
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::String(s) => write!(f, "{s}"),
            DataValue::InternedString(s) => write!(f, "{s}"),
            DataValue::Integer(i) => write!(f, "{i}"),
            DataValue::Float(fl) => write!(f, "{fl}"),
            DataValue::Boolean(b) => write!(f, "{b}"),
            DataValue::DateTime(dt) => write!(f, "{dt}"),
            DataValue::Vector(v) => {
                let components: Vec<String> = v.iter().map(|fl| fl.to_string()).collect();
                write!(f, "[{}]", components.join(","))
            }
            DataValue::Null => write!(f, ""),
        }
    }
}

/// A row of data in the table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRow {
    pub values: Vec<DataValue>,
}

impl DataRow {
    #[must_use]
    pub fn new(values: Vec<DataValue>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&DataValue> {
        self.values.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut DataValue> {
        self.values.get_mut(index)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// The main `DataTable` structure
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

    /// Create a DUAL table (similar to Oracle's DUAL) with one row and one column
    /// Used for evaluating expressions without a data source
    #[must_use]
    pub fn dual() -> Self {
        let mut table = DataTable::new("DUAL");
        table.add_column(DataColumn::new("DUMMY").with_type(DataType::String));
        table
            .add_row(DataRow::new(vec![DataValue::String("X".to_string())]))
            .unwrap();
        table
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

    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&DataColumn> {
        self.columns.iter().find(|c| c.name == name)
    }

    #[must_use]
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// Find column index by qualified name (e.g., "messages.field_name")
    #[must_use]
    pub fn find_column_by_qualified_name(&self, qualified_name: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.qualified_name.as_deref() == Some(qualified_name))
    }

    /// Find column by either qualified or simple name
    /// First tries qualified match, then falls back to simple name
    #[must_use]
    pub fn find_column_flexible(&self, name: &str, table_prefix: Option<&str>) -> Option<usize> {
        // If table prefix provided, try qualified match first
        if let Some(prefix) = table_prefix {
            let qualified = format!("{}.{}", prefix, name);
            if let Some(idx) = self.find_column_by_qualified_name(&qualified) {
                return Some(idx);
            }
        }

        // Fall back to simple name match
        self.get_column_index(name)
    }

    /// Enrich all columns with qualified names based on the table name
    pub fn enrich_columns_with_qualified_names(&mut self, table_name: &str) {
        for column in &mut self.columns {
            column.qualified_name = Some(format!("{}.{}", table_name, column.name));
            column.source_table = Some(table_name.to_string());
        }
    }

    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get column names as a vector
    #[must_use]
    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    /// Get mutable access to columns for enrichment
    pub fn columns_mut(&mut self) -> &mut [DataColumn] {
        &mut self.columns
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
    #[must_use]
    pub fn get_value(&self, row: usize, col: usize) -> Option<&DataValue> {
        self.rows.get(row)?.get(col)
    }

    /// Get a value by row index and column name
    #[must_use]
    pub fn get_value_by_name(&self, row: usize, col_name: &str) -> Option<&DataValue> {
        let col_idx = self.get_column_index(col_name)?;
        self.get_value(row, col_idx)
    }

    /// Convert to a vector of string vectors (for display/compatibility)
    #[must_use]
    pub fn to_string_table(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| {
                row.values
                    .iter()
                    .map(DataValue::to_string_optimized)
                    .collect()
            })
            .collect()
    }

    /// Get table statistics
    #[must_use]
    pub fn get_stats(&self) -> DataTableStats {
        DataTableStats {
            row_count: self.row_count(),
            column_count: self.column_count(),
            memory_size: self.estimate_memory_size(),
            null_count: self.columns.iter().map(|c| c.null_count).sum(),
        }
    }

    /// Generate a debug dump string for display
    #[must_use]
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
                output.push_str(&format!("  {key}: {value}\n"));
            }
        }

        output.push_str("\nColumns:\n");
        for column in &self.columns {
            output.push_str(&format!("  {} ({:?})", column.name, column.data_type));
            if column.nullable {
                output.push_str(&format!(" - nullable, {} nulls", column.null_count));
            }
            if let Some(unique) = column.unique_values {
                output.push_str(&format!(", {unique} unique"));
            }
            output.push('\n');
        }

        // Show first few rows
        if self.row_count() > 0 {
            let sample_size = 5.min(self.row_count());
            output.push_str(&format!("\nFirst {sample_size} rows:\n"));

            for row_idx in 0..sample_size {
                output.push_str(&format!("  [{row_idx}]: "));
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

    #[must_use]
    pub fn estimate_memory_size(&self) -> usize {
        // Base structure size
        let mut size = std::mem::size_of::<Self>();

        // Column metadata
        size += self.columns.len() * std::mem::size_of::<DataColumn>();
        for col in &self.columns {
            size += col.name.len();
        }

        // Row structure overhead
        size += self.rows.len() * std::mem::size_of::<DataRow>();

        // Actual data values
        for row in &self.rows {
            for value in &row.values {
                // Base enum size
                size += std::mem::size_of::<DataValue>();
                // Add string content size
                match value {
                    DataValue::String(s) | DataValue::DateTime(s) => size += s.len(),
                    DataValue::Vector(v) => size += v.len() * std::mem::size_of::<f64>(),
                    _ => {} // Numbers and booleans are inline
                }
            }
        }

        size
    }

    /// Convert DataTable to CSV format
    pub fn to_csv(&self) -> String {
        let mut csv_output = String::new();

        // Write headers
        let headers: Vec<String> = self
            .columns
            .iter()
            .map(|col| {
                if col.name.contains(',') || col.name.contains('"') || col.name.contains('\n') {
                    format!("\"{}\"", col.name.replace('"', "\"\""))
                } else {
                    col.name.clone()
                }
            })
            .collect();
        csv_output.push_str(&headers.join(","));
        csv_output.push('\n');

        // Write data rows
        for row in &self.rows {
            let row_values: Vec<String> = row
                .values
                .iter()
                .map(|value| {
                    let str_val = value.to_string();
                    if str_val.contains(',') || str_val.contains('"') || str_val.contains('\n') {
                        format!("\"{}\"", str_val.replace('"', "\"\""))
                    } else {
                        str_val
                    }
                })
                .collect();
            csv_output.push_str(&row_values.join(","));
            csv_output.push('\n');
        }

        csv_output
    }

    /// V46: Create `DataTable` from `QueryResponse`
    /// This is the key conversion function that bridges old and new systems
    pub fn from_query_response(response: &QueryResponse, table_name: &str) -> Result<Self, String> {
        debug!(
            "V46: Converting QueryResponse to DataTable for table '{}'",
            table_name
        );

        // Track memory before conversion
        crate::utils::memory_tracker::track_memory("start_from_query_response");

        let mut table = DataTable::new(table_name);

        // Extract column names and types from first row
        if let Some(first_row) = response.data.first() {
            if let Some(obj) = first_row.as_object() {
                // Create columns based on the keys in the JSON object
                for key in obj.keys() {
                    let column = DataColumn::new(key.clone());
                    table.add_column(column);
                }

                // Now convert all rows
                for json_row in &response.data {
                    if let Some(row_obj) = json_row.as_object() {
                        let mut values = Vec::new();

                        // Ensure we get values in the same order as columns
                        for column in &table.columns {
                            let value = row_obj
                                .get(&column.name)
                                .map_or(DataValue::Null, json_value_to_data_value);
                            values.push(value);
                        }

                        table.add_row(DataRow::new(values))?;
                    }
                }

                // Infer column types from the data
                table.infer_column_types();

                // Add metadata
                if let Some(source) = &response.source {
                    table.metadata.insert("source".to_string(), source.clone());
                }
                if let Some(cached) = response.cached {
                    table
                        .metadata
                        .insert("cached".to_string(), cached.to_string());
                }
                table
                    .metadata
                    .insert("original_count".to_string(), response.count.to_string());

                debug!(
                    "V46: Created DataTable with {} columns and {} rows",
                    table.column_count(),
                    table.row_count()
                );
            } else {
                // Handle non-object JSON (single values)
                table.add_column(DataColumn::new("value"));
                for json_value in &response.data {
                    let value = json_value_to_data_value(json_value);
                    table.add_row(DataRow::new(vec![value]))?;
                }
            }
        }

        Ok(table)
    }

    /// Get a single row by index
    #[must_use]
    pub fn get_row(&self, index: usize) -> Option<&DataRow> {
        self.rows.get(index)
    }

    /// V50: Get a single row as strings
    #[must_use]
    pub fn get_row_as_strings(&self, index: usize) -> Option<Vec<String>> {
        self.rows.get(index).map(|row| {
            row.values
                .iter()
                .map(DataValue::to_string_optimized)
                .collect()
        })
    }

    /// Pretty print the `DataTable` with a nice box drawing
    #[must_use]
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("╔═══════════════════════════════════════════════════════╗\n");
        output.push_str(&format!("║ DataTable: {:^41} ║\n", self.name));
        output.push_str("╠═══════════════════════════════════════════════════════╣\n");

        // Summary stats
        output.push_str(&format!(
            "║ Rows: {:6} | Columns: {:3} | Memory: ~{:6} bytes ║\n",
            self.row_count(),
            self.column_count(),
            self.get_stats().memory_size
        ));

        // Metadata if any
        if !self.metadata.is_empty() {
            output.push_str("╠═══════════════════════════════════════════════════════╣\n");
            output.push_str("║ Metadata:                                             ║\n");
            for (key, value) in &self.metadata {
                let truncated_value = if value.len() > 35 {
                    format!("{}...", &value[..32])
                } else {
                    value.clone()
                };
                output.push_str(&format!(
                    "║   {:15} : {:35} ║\n",
                    Self::truncate_string(key, 15),
                    truncated_value
                ));
            }
        }

        // Column details
        output.push_str("╠═══════════════════════════════════════════════════════╣\n");
        output.push_str("║ Columns:                                              ║\n");
        output.push_str("╟───────────────────┬──────────┬─────────┬──────┬──────╢\n");
        output.push_str("║ Name              │ Type     │ Nullable│ Nulls│Unique║\n");
        output.push_str("╟───────────────────┼──────────┼─────────┼──────┼──────╢\n");

        for column in &self.columns {
            let type_str = match &column.data_type {
                DataType::String => "String",
                DataType::Integer => "Integer",
                DataType::Float => "Float",
                DataType::Boolean => "Boolean",
                DataType::DateTime => "DateTime",
                DataType::Null => "Null",
                DataType::Mixed => "Mixed",
            };

            output.push_str(&format!(
                "║ {:17} │ {:8} │ {:7} │ {:4} │ {:4} ║\n",
                Self::truncate_string(&column.name, 17),
                type_str,
                if column.nullable { "Yes" } else { "No" },
                column.null_count,
                column.unique_values.unwrap_or(0)
            ));
        }

        output.push_str("╚═══════════════════════════════════════════════════════╝\n");

        // Sample data (first 5 rows)
        output.push_str("\nSample Data (first 5 rows):\n");
        let sample_count = self.rows.len().min(5);

        if sample_count > 0 {
            // Column headers
            output.push('┌');
            for (i, _col) in self.columns.iter().enumerate() {
                if i > 0 {
                    output.push('┬');
                }
                output.push_str(&"─".repeat(20));
            }
            output.push_str("┐\n");

            output.push('│');
            for col in &self.columns {
                output.push_str(&format!(" {:^18} │", Self::truncate_string(&col.name, 18)));
            }
            output.push('\n');

            output.push('├');
            for (i, _) in self.columns.iter().enumerate() {
                if i > 0 {
                    output.push('┼');
                }
                output.push_str(&"─".repeat(20));
            }
            output.push_str("┤\n");

            // Data rows
            for row_idx in 0..sample_count {
                if let Some(row) = self.rows.get(row_idx) {
                    output.push('│');
                    for value in &row.values {
                        let value_str = value.to_string();
                        output
                            .push_str(&format!(" {:18} │", Self::truncate_string(&value_str, 18)));
                    }
                    output.push('\n');
                }
            }

            output.push('└');
            for (i, _) in self.columns.iter().enumerate() {
                if i > 0 {
                    output.push('┴');
                }
                output.push_str(&"─".repeat(20));
            }
            output.push_str("┘\n");
        }

        output
    }

    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{}...", &s[..max_len - 3])
        } else {
            s.to_string()
        }
    }

    /// Get a schema summary of the `DataTable`
    #[must_use]
    pub fn get_schema_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!(
            "DataTable Schema ({} columns, {} rows):\n",
            self.columns.len(),
            self.rows.len()
        ));

        for (idx, column) in self.columns.iter().enumerate() {
            let type_str = match &column.data_type {
                DataType::String => "String",
                DataType::Integer => "Integer",
                DataType::Float => "Float",
                DataType::Boolean => "Boolean",
                DataType::DateTime => "DateTime",
                DataType::Null => "Null",
                DataType::Mixed => "Mixed",
            };

            let nullable_str = if column.nullable {
                "nullable"
            } else {
                "not null"
            };
            let null_info = if column.null_count > 0 {
                format!(", {} nulls", column.null_count)
            } else {
                String::new()
            };

            summary.push_str(&format!(
                "  [{:3}] {} : {} ({}{})\n",
                idx, column.name, type_str, nullable_str, null_info
            ));
        }

        summary
    }

    /// Get detailed schema information as a structured format
    #[must_use]
    pub fn get_schema_info(&self) -> Vec<(String, String, bool, usize)> {
        self.columns
            .iter()
            .map(|col| {
                let type_name = format!("{:?}", col.data_type);
                (col.name.clone(), type_name, col.nullable, col.null_count)
            })
            .collect()
    }

    /// Reserve capacity for rows to avoid reallocations
    pub fn reserve_rows(&mut self, additional: usize) {
        self.rows.reserve(additional);
    }

    /// Shrink vectors to fit actual data (removes excess capacity)
    pub fn shrink_to_fit(&mut self) {
        self.rows.shrink_to_fit();
        for _column in &mut self.columns {
            // Shrink any column-specific data if needed
        }
    }

    /// Get actual memory usage estimate (more accurate than `estimate_memory_size`)
    #[must_use]
    pub fn get_memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Account for string allocations
        size += self.name.capacity();

        // Account for columns
        size += self.columns.capacity() * std::mem::size_of::<DataColumn>();
        for col in &self.columns {
            size += col.name.capacity();
        }

        // Account for rows and their capacity
        size += self.rows.capacity() * std::mem::size_of::<DataRow>();

        // Account for actual data values
        for row in &self.rows {
            size += row.values.capacity() * std::mem::size_of::<DataValue>();
            for value in &row.values {
                match value {
                    DataValue::String(s) => size += s.capacity(),
                    DataValue::InternedString(_) => size += std::mem::size_of::<Arc<String>>(),
                    DataValue::DateTime(s) => size += s.capacity(),
                    DataValue::Vector(v) => size += v.capacity() * std::mem::size_of::<f64>(),
                    _ => {} // Other types are inline
                }
            }
        }

        // Account for metadata
        size += self.metadata.capacity() * std::mem::size_of::<(String, String)>();
        for (k, v) in &self.metadata {
            size += k.capacity() + v.capacity();
        }

        size
    }

    /// Serialize DataTable to bytes for caching (using MessagePack for now, can be upgraded to Parquet)
    pub fn to_parquet_bytes(&self) -> Result<Vec<u8>, String> {
        // For now, use MessagePack which is binary-safe and fast
        // Later we can upgrade to actual Parquet format
        rmp_serde::to_vec(self).map_err(|e| format!("Failed to serialize DataTable: {}", e))
    }

    /// Deserialize DataTable from cached bytes
    pub fn from_parquet_bytes(bytes: &[u8]) -> Result<Self, String> {
        // For now, use MessagePack
        // Later we can upgrade to actual Parquet format
        rmp_serde::from_slice(bytes).map_err(|e| format!("Failed to deserialize DataTable: {}", e))
    }
}

/// V46: Helper function to convert JSON value to `DataValue`
fn json_value_to_data_value(json: &JsonValue) -> DataValue {
    match json {
        JsonValue::Null => DataValue::Null,
        JsonValue::Bool(b) => DataValue::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                DataValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                DataValue::Float(f)
            } else {
                DataValue::String(n.to_string())
            }
        }
        JsonValue::String(s) => {
            // Try to detect if it's a date/time
            if s.contains('-') && s.len() >= 8 && s.len() <= 30 {
                // Simple heuristic for dates
                DataValue::DateTime(s.clone())
            } else {
                DataValue::String(s.clone())
            }
        }
        JsonValue::Array(_) | JsonValue::Object(_) => {
            // Store complex types as JSON string
            DataValue::String(json.to_string())
        }
    }
}

/// Statistics about a `DataTable`
#[derive(Debug, Clone)]
pub struct DataTableStats {
    pub row_count: usize,
    pub column_count: usize,
    pub memory_size: usize,
    pub null_count: usize,
}

/// Implementation of `DataProvider` for `DataTable`
/// This allows `DataTable` to be used wherever `DataProvider` trait is expected
impl DataProvider for DataTable {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.rows.get(index).map(|row| {
            row.values
                .iter()
                .map(DataValue::to_string_optimized)
                .collect()
        })
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

    #[test]
    fn test_from_query_response() {
        use crate::api_client::{QueryInfo, QueryResponse};
        use serde_json::json;

        let response = QueryResponse {
            query: QueryInfo {
                select: vec!["id".to_string(), "name".to_string(), "age".to_string()],
                where_clause: None,
                order_by: None,
            },
            data: vec![
                json!({
                    "id": 1,
                    "name": "Alice",
                    "age": 30
                }),
                json!({
                    "id": 2,
                    "name": "Bob",
                    "age": 25
                }),
                json!({
                    "id": 3,
                    "name": "Carol",
                    "age": null
                }),
            ],
            count: 3,
            source: Some("test.csv".to_string()),
            table: Some("test".to_string()),
            cached: Some(false),
        };

        let table = DataTable::from_query_response(&response, "test").unwrap();

        assert_eq!(table.name, "test");
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.column_count(), 3);

        // Check column names
        let col_names = table.column_names();
        assert!(col_names.contains(&"id".to_string()));
        assert!(col_names.contains(&"name".to_string()));
        assert!(col_names.contains(&"age".to_string()));

        // Check metadata
        assert_eq!(table.metadata.get("source"), Some(&"test.csv".to_string()));
        assert_eq!(table.metadata.get("cached"), Some(&"false".to_string()));

        // Check first row values
        assert_eq!(
            table.get_value_by_name(0, "id"),
            Some(&DataValue::Integer(1))
        );
        assert_eq!(
            table.get_value_by_name(0, "name"),
            Some(&DataValue::String("Alice".to_string()))
        );
        assert_eq!(
            table.get_value_by_name(0, "age"),
            Some(&DataValue::Integer(30))
        );

        // Check null handling
        assert_eq!(table.get_value_by_name(2, "age"), Some(&DataValue::Null));
    }
}
