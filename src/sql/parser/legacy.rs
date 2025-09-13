//! Legacy parser types for backward compatibility
//!
//! These types were previously defined in parser.rs and are needed
//! by various parts of the codebase.

use crate::config::schema_config;

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

#[derive(Debug, Clone)]
pub struct Schema {
    tables: Vec<TableInfo>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<String>,
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

impl Schema {
    #[must_use]
    pub fn new() -> Self {
        // Use the complete column list from schema_config
        let trade_deal_columns = schema_config::get_full_trade_deal_columns();

        Self {
            tables: vec![
                TableInfo {
                    name: "trade_deal".to_string(),
                    columns: trade_deal_columns,
                },
                TableInfo {
                    name: "test".to_string(),
                    columns: vec![
                        "id".to_string(),
                        "name".to_string(),
                        "value".to_string(),
                        "timestamp".to_string(),
                    ],
                },
            ],
        }
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.iter().map(|t| t.name.clone()).collect()
    }

    pub fn get_columns_for_table(&self, table_name: &str) -> Vec<String> {
        self.tables
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(table_name))
            .map(|t| t.columns.clone())
            .unwrap_or_default()
    }

    pub fn get_all_columns(&self) -> Vec<String> {
        let mut all_columns = Vec::new();
        for table in &self.tables {
            all_columns.extend(table.columns.clone());
        }
        all_columns.sort();
        all_columns.dedup();
        all_columns
    }

    // Legacy compatibility methods
    pub fn set_single_table(&mut self, table_name: &str, columns: Vec<String>) {
        self.tables.clear();
        self.tables.push(TableInfo {
            name: table_name.to_string(),
            columns,
        });
    }

    pub fn get_columns(&self, table_name: &str) -> Vec<String> {
        self.get_columns_for_table(table_name)
    }

    pub fn get_first_table_name(&self) -> Option<String> {
        self.tables.first().map(|t| t.name.clone())
    }

    pub fn add_table(&mut self, name: String, columns: Vec<String>) {
        // Remove table if it already exists
        self.tables.retain(|t| t.name != name);
        self.tables.push(TableInfo { name, columns });
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables
            .iter()
            .any(|t| t.name.eq_ignore_ascii_case(table_name))
    }
}
