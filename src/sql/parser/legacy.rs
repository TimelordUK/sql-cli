//! Legacy parser types for backward compatibility
//!
//! These types were previously defined in parser.rs and are needed
//! by various parts of the codebase.

use crate::data::datatable::{DataColumn, DataType};

#[derive(Debug, Clone, PartialEq)]
pub enum SqlToken {
    Select,
    From,
    Where,
    OrderBy,
    Identifier(String),
    Column(String),
    Table(String),
    Operator(String),
    String(String),
    Number(String),
    Function(String),
    Comma,
    Dot,
    OpenParen,
    CloseParen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseState {
    Start,
    AfterSelect,
    InColumnList,
    AfterFrom,
    InTableName,
    AfterTable,
    InWhere,
    InOrderBy,
}

impl ParseState {
    pub fn get_suggestions(&self, schema: &Schema) -> Vec<String> {
        match self {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => schema.get_all_columns(),
            ParseState::InColumnList => {
                let mut suggestions = schema.get_all_columns();
                suggestions.push("FROM".to_string());
                suggestions
            }
            ParseState::AfterFrom => schema.get_table_names(),
            ParseState::InTableName => schema.get_table_names(),
            ParseState::AfterTable => vec!["WHERE".to_string(), "ORDER BY".to_string()],
            ParseState::InWhere => schema.get_all_columns(),
            ParseState::InOrderBy => schema.get_all_columns(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseContext {
    pub state: ParseState,
    pub partial_word: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SqlParser {
    pub tokens: Vec<SqlToken>,
    pub current_state: ParseState,
}

impl Default for SqlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current_state: ParseState::Start,
        }
    }

    pub fn parse_quick(&mut self, query: &str) -> ParseState {
        // Simple quick parsing - just look for keywords
        let query_upper = query.to_uppercase();
        if query_upper.contains("SELECT") && query_upper.contains("FROM") {
            if query_upper.contains("WHERE") {
                ParseState::InWhere
            } else {
                ParseState::AfterFrom
            }
        } else if query_upper.contains("SELECT") {
            ParseState::AfterSelect
        } else {
            ParseState::Start
        }
    }

    pub fn parse_to_position(&mut self, query: &str, position: usize) -> ParseState {
        let query_up_to_position = &query[..position.min(query.len())];
        self.parse_quick(query_up_to_position)
    }

    pub fn get_completion_context(&mut self, input: &str) -> ParseContext {
        ParseContext {
            state: self.parse_quick(input),
            partial_word: None, // Simple implementation
        }
    }

    pub fn parse_partial(&mut self, input: &str) -> ParseState {
        self.parse_quick(input)
    }
}

/// The type categories completion actually branches on.
///
/// Deliberately coarser than [`DataType`]: the editor only decides which
/// methods and which literal shapes to offer, and `Integer` vs `Float` makes
/// no difference to either. Keeping it separate is also what keeps the schema
/// a *snapshot* rather than a view onto the loaded table (see [`Schema`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    String,
    Numeric,
    DateTime,
    Boolean,
}

impl ColumnType {
    /// The wire name used by the completion paths that still branch on strings
    /// (`get_string_method_suggestions`, the `AfterComparisonOp` arm).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ColumnType::String => "string",
            ColumnType::Numeric => "numeric",
            ColumnType::DateTime => "datetime",
            ColumnType::Boolean => "boolean",
        }
    }
}

impl From<&DataType> for ColumnType {
    fn from(data_type: &DataType) -> Self {
        match data_type {
            DataType::Integer | DataType::Float => ColumnType::Numeric,
            DataType::DateTime => ColumnType::DateTime,
            DataType::Boolean => ColumnType::Boolean,
            // `Null` (a column empty in every row) and `Mixed` both behave
            // like text as far as completion is concerned.
            DataType::String | DataType::Null | DataType::Mixed => ColumnType::String,
        }
    }
}

/// What the completer knows about one column.
///
/// A bounded snapshot taken at load time, never a live handle to the
/// `DataTable`. Completion staying a pure function of
/// `(query, cursor, schema)` is what makes its tests cheap to write.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnType,
    /// Distinct non-null values seen at load time, when the loader counted
    /// them. The numerator of a low-cardinality gate; [`TableInfo::row_count`]
    /// is the denominator.
    pub cardinality: Option<usize>,
    pub nullable: bool,
}

impl ColumnInfo {
    /// A name-only column. For the callers that genuinely have nothing else -
    /// the reedline completer, tests - which therefore keep the string-typed
    /// behaviour the whole completer used to have.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: ColumnType::String,
            cardinality: None,
            nullable: true,
        }
    }

    #[must_use]
    pub fn with_type(mut self, data_type: ColumnType) -> Self {
        self.data_type = data_type;
        self
    }

    #[must_use]
    pub fn with_cardinality(mut self, cardinality: usize) -> Self {
        self.cardinality = Some(cardinality);
        self
    }

    /// Take the snapshot. `DataTable::infer_column_types()` already populates
    /// every field this reads, on every load path, so this is the whole of the
    /// data-side wiring.
    #[must_use]
    pub fn from_data_column(column: &DataColumn) -> Self {
        Self {
            name: column.name.clone(),
            data_type: ColumnType::from(&column.data_type),
            cardinality: column.unique_values,
            nullable: column.nullable,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    tables: Vec<TableInfo>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    /// Rows the column snapshot was taken over, when known. The denominator
    /// for a cardinality *ratio*: an absolute count alone cannot tell
    /// "5 regions across 250 rows" from "5 rows, every value distinct".
    pub row_count: Option<usize>,
}

impl TableInfo {
    pub fn new(name: impl Into<String>, columns: Vec<ColumnInfo>) -> Self {
        Self {
            name: name.into(),
            columns,
            row_count: None,
        }
    }

    /// Build a table from column names alone - every column types as string,
    /// which is what the completer assumed unconditionally before T2.
    pub fn from_names(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self::new(name, columns.into_iter().map(ColumnInfo::new).collect())
    }

    #[must_use]
    pub fn with_row_count(mut self, row_count: usize) -> Self {
        self.row_count = Some(row_count);
        self
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    pub fn find_column(&self, column_name: &str) -> Option<&ColumnInfo> {
        self.columns
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(column_name))
    }
}

impl Schema {
    /// An empty schema. There is deliberately no built-in table: a completer
    /// that suggests trade-desk columns before a file has loaded is wrong on
    /// every dataset but one.
    #[must_use]
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.iter().map(|t| t.name.clone()).collect()
    }

    pub fn get_table(&self, table_name: &str) -> Option<&TableInfo> {
        self.tables
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(table_name))
    }

    pub fn get_columns_for_table(&self, table_name: &str) -> Vec<String> {
        self.get_table(table_name)
            .map(TableInfo::column_names)
            .unwrap_or_default()
    }

    /// The typed columns of one table; empty if it is not loaded.
    #[must_use]
    pub fn get_column_infos(&self, table_name: &str) -> &[ColumnInfo] {
        self.get_table(table_name)
            .map_or(&[][..], |t| t.columns.as_slice())
    }

    /// Look a column up by name across every table.
    ///
    /// Completion contexts such as `price.<tab>` carry a bare column name with
    /// no table qualifier, so there is nothing to scope the lookup by. In the
    /// single-table case the TUI actually runs in, this is exact.
    pub fn find_column(&self, column_name: &str) -> Option<&ColumnInfo> {
        self.tables.iter().find_map(|t| t.find_column(column_name))
    }

    pub fn get_all_columns(&self) -> Vec<String> {
        let mut all_columns: Vec<String> = self
            .tables
            .iter()
            .flat_map(|t| t.columns.iter().map(|c| c.name.clone()))
            .collect();
        all_columns.sort();
        all_columns.dedup();
        all_columns
    }

    /// Replace the schema with a single fully-typed table.
    pub fn set_single_table_info(&mut self, table: TableInfo) {
        self.tables.clear();
        self.tables.push(table);
    }

    // Legacy compatibility methods
    pub fn set_single_table(&mut self, table_name: &str, columns: Vec<String>) {
        self.set_single_table_info(TableInfo::from_names(table_name, columns));
    }

    pub fn get_columns(&self, table_name: &str) -> Vec<String> {
        self.get_columns_for_table(table_name)
    }

    pub fn get_first_table_name(&self) -> Option<String> {
        self.tables.first().map(|t| t.name.clone())
    }

    pub fn add_table_info(&mut self, table: TableInfo) {
        self.tables.retain(|t| t.name != table.name);
        self.tables.push(table);
    }

    pub fn add_table(&mut self, name: String, columns: Vec<String>) {
        self.add_table_info(TableInfo::from_names(name, columns));
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.get_table(table_name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_schema_is_empty() {
        // Before T2 this returned the trade_deal schema, so the completer
        // suggested `platformOrderId` on a freshly opened CSV.
        let schema = Schema::new();
        assert!(schema.get_table_names().is_empty());
        assert!(schema.get_all_columns().is_empty());
    }

    #[test]
    fn column_lookup_is_case_insensitive() {
        let mut schema = Schema::new();
        schema.set_single_table_info(TableInfo::new("countries", vec![ColumnInfo::new("region")]));

        assert!(schema.find_column("REGION").is_some());
        assert!(schema.find_column("Region").is_some());
        assert!(schema.find_column("regions").is_none());
    }

    #[test]
    fn data_types_collapse_to_completion_categories() {
        assert_eq!(ColumnType::from(&DataType::Integer), ColumnType::Numeric);
        assert_eq!(ColumnType::from(&DataType::Float), ColumnType::Numeric);
        assert_eq!(ColumnType::from(&DataType::DateTime), ColumnType::DateTime);
        assert_eq!(ColumnType::from(&DataType::Boolean), ColumnType::Boolean);
        assert_eq!(ColumnType::from(&DataType::Mixed), ColumnType::String);
        assert_eq!(ColumnType::from(&DataType::Null), ColumnType::String);
    }

    #[test]
    fn snapshot_carries_type_cardinality_and_nullability() {
        let mut column = DataColumn::new("region").with_type(DataType::String);
        column.unique_values = Some(5);
        column.nullable = false;

        let info = ColumnInfo::from_data_column(&column);
        assert_eq!(info.name, "region");
        assert_eq!(info.data_type, ColumnType::String);
        assert_eq!(info.cardinality, Some(5));
        assert!(!info.nullable);
    }

    #[test]
    fn name_only_tables_still_work() {
        let mut schema = Schema::new();
        schema.set_single_table("t", vec!["a".to_string(), "b".to_string()]);

        assert_eq!(schema.get_columns("t"), vec!["a", "b"]);
        assert_eq!(
            schema.find_column("a").map(|c| c.data_type),
            Some(ColumnType::String)
        );
    }
}
