//! Abstract Syntax Tree (AST) definitions for SQL queries
//!
//! This module contains all the data structures that represent
//! the parsed SQL query structure.

// ===== Expression Types =====

/// Quote style for identifiers (column names, table names, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuoteStyle {
    /// No quotes needed (valid unquoted identifier)
    None,
    /// Double quotes: "Customer Id"
    DoubleQuotes,
    /// SQL Server style brackets: [Customer Id]
    Brackets,
}

/// Column reference with optional quoting information and table prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnRef {
    pub name: String,
    pub quote_style: QuoteStyle,
    /// Optional table/alias prefix (e.g., "messages" in "messages.field_name")
    pub table_prefix: Option<String>,
}

impl ColumnRef {
    /// Create an unquoted column reference
    pub fn unquoted(name: String) -> Self {
        Self {
            name,
            quote_style: QuoteStyle::None,
            table_prefix: None,
        }
    }

    /// Create a double-quoted column reference
    pub fn quoted(name: String) -> Self {
        Self {
            name,
            quote_style: QuoteStyle::DoubleQuotes,
            table_prefix: None,
        }
    }

    /// Create a qualified column reference (table.column)
    pub fn qualified(table: String, name: String) -> Self {
        Self {
            name,
            quote_style: QuoteStyle::None,
            table_prefix: Some(table),
        }
    }

    /// Get the full qualified string representation
    pub fn to_qualified_string(&self) -> String {
        match &self.table_prefix {
            Some(table) => format!("{}.{}", table, self.name),
            None => self.name.clone(),
        }
    }

    /// Create a bracket-quoted column reference
    pub fn bracketed(name: String) -> Self {
        Self {
            name,
            quote_style: QuoteStyle::Brackets,
            table_prefix: None,
        }
    }

    /// Format the column reference with appropriate quoting
    pub fn to_sql(&self) -> String {
        let column_part = match self.quote_style {
            QuoteStyle::None => self.name.clone(),
            QuoteStyle::DoubleQuotes => format!("\"{}\"", self.name),
            QuoteStyle::Brackets => format!("[{}]", self.name),
        };

        match &self.table_prefix {
            Some(table) => format!("{}.{}", table, column_part),
            None => column_part,
        }
    }
}

impl PartialEq<str> for ColumnRef {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<&str> for ColumnRef {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

impl std::fmt::Display for ColumnRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

#[derive(Debug, Clone)]
pub enum SqlExpression {
    Column(ColumnRef),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
    Null, // NULL literal
    DateTimeConstructor {
        year: i32,
        month: u32,
        day: u32,
        hour: Option<u32>,
        minute: Option<u32>,
        second: Option<u32>,
    },
    DateTimeToday {
        hour: Option<u32>,
        minute: Option<u32>,
        second: Option<u32>,
    },
    MethodCall {
        object: String,
        method: String,
        args: Vec<SqlExpression>,
    },
    ChainedMethodCall {
        base: Box<SqlExpression>,
        method: String,
        args: Vec<SqlExpression>,
    },
    FunctionCall {
        name: String,
        args: Vec<SqlExpression>,
        distinct: bool, // For COUNT(DISTINCT col), SUM(DISTINCT col), etc.
    },
    WindowFunction {
        name: String,
        args: Vec<SqlExpression>,
        window_spec: WindowSpec,
    },
    BinaryOp {
        left: Box<SqlExpression>,
        op: String,
        right: Box<SqlExpression>,
    },
    InList {
        expr: Box<SqlExpression>,
        values: Vec<SqlExpression>,
    },
    NotInList {
        expr: Box<SqlExpression>,
        values: Vec<SqlExpression>,
    },
    Between {
        expr: Box<SqlExpression>,
        lower: Box<SqlExpression>,
        upper: Box<SqlExpression>,
    },
    Not {
        expr: Box<SqlExpression>,
    },
    CaseExpression {
        when_branches: Vec<WhenBranch>,
        else_branch: Option<Box<SqlExpression>>,
    },
    SimpleCaseExpression {
        expr: Box<SqlExpression>,
        when_branches: Vec<SimpleWhenBranch>,
        else_branch: Option<Box<SqlExpression>>,
    },
    /// Scalar subquery that returns a single value
    /// Used in expressions like: WHERE col = (SELECT MAX(id) FROM table)
    ScalarSubquery {
        query: Box<SelectStatement>,
    },
    /// IN subquery that returns multiple values
    /// Used in expressions like: WHERE col IN (SELECT id FROM table WHERE ...)
    InSubquery {
        expr: Box<SqlExpression>,
        subquery: Box<SelectStatement>,
    },
    /// UNNEST - Row expansion function that splits delimited strings
    /// Used like: SELECT UNNEST(accounts, '|') AS account FROM fix_trades
    /// Causes row multiplication - one input row becomes N output rows
    Unnest {
        column: Box<SqlExpression>,
        delimiter: String,
    },
    /// NOT IN subquery
    /// Used in expressions like: WHERE col NOT IN (SELECT id FROM table WHERE ...)
    NotInSubquery {
        expr: Box<SqlExpression>,
        subquery: Box<SelectStatement>,
    },
}

#[derive(Debug, Clone)]
pub struct WhenBranch {
    pub condition: Box<SqlExpression>,
    pub result: Box<SqlExpression>,
}

#[derive(Debug, Clone)]
pub struct SimpleWhenBranch {
    pub value: Box<SqlExpression>,
    pub result: Box<SqlExpression>,
}

// ===== WHERE Clause Types =====

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub expr: SqlExpression,
    pub connector: Option<LogicalOp>, // AND/OR connecting to next condition
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

// ===== ORDER BY Types =====

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct OrderByColumn {
    pub column: String,
    pub direction: SortDirection,
}

// ===== Window Function Types =====

/// Window frame bounds
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    UnboundedPreceding,
    CurrentRow,
    Preceding(i64),
    Following(i64),
    UnboundedFollowing,
}

/// Window frame unit (ROWS or RANGE)
#[derive(Debug, Clone, PartialEq)]
pub enum FrameUnit {
    Rows,
    Range,
}

/// Window frame specification
#[derive(Debug, Clone)]
pub struct WindowFrame {
    pub unit: FrameUnit,
    pub start: FrameBound,
    pub end: Option<FrameBound>, // None means CURRENT ROW
}

#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub partition_by: Vec<String>,
    pub order_by: Vec<OrderByColumn>,
    pub frame: Option<WindowFrame>, // Optional window frame
}

// ===== SELECT Statement Types =====

/// Represents a SELECT item - either a simple column or a computed expression with alias
#[derive(Debug, Clone)]
pub enum SelectItem {
    /// Simple column reference: "`column_name`"
    Column(ColumnRef),
    /// Computed expression with alias: "expr AS alias"
    Expression { expr: SqlExpression, alias: String },
    /// Star selector: "*"
    Star,
}

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub distinct: bool,                // SELECT DISTINCT flag
    pub columns: Vec<String>,          // Keep for backward compatibility, will be deprecated
    pub select_items: Vec<SelectItem>, // New field for computed expressions
    pub from_table: Option<String>,
    pub from_subquery: Option<Box<SelectStatement>>, // Subquery in FROM clause
    pub from_function: Option<TableFunction>,        // Table function like RANGE() in FROM clause
    pub from_alias: Option<String>,                  // Alias for subquery (AS name)
    pub joins: Vec<JoinClause>,                      // JOIN clauses
    pub where_clause: Option<WhereClause>,
    pub order_by: Option<Vec<OrderByColumn>>,
    pub group_by: Option<Vec<SqlExpression>>, // Changed from Vec<String> to support expressions
    pub having: Option<SqlExpression>,        // HAVING clause for post-aggregation filtering
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub ctes: Vec<CTE>,                // Common Table Expressions (WITH clause)
    pub into_table: Option<IntoTable>, // INTO clause for temporary tables
}

/// INTO clause for creating temporary tables
#[derive(Debug, Clone, PartialEq)]
pub struct IntoTable {
    /// Name of the temporary table (must start with #)
    pub name: String,
}

// ===== Table and Join Types =====

/// Table function that generates virtual tables
#[derive(Debug, Clone)]
pub enum TableFunction {
    Generator {
        name: String,
        args: Vec<SqlExpression>,
    },
}

/// Common Table Expression (CTE) structure
#[derive(Debug, Clone)]
pub struct CTE {
    pub name: String,
    pub column_list: Option<Vec<String>>, // Optional column list: WITH t(col1, col2) AS ...
    pub cte_type: CTEType,
}

/// Type of CTE - standard SQL or WEB fetch
#[derive(Debug, Clone)]
pub enum CTEType {
    Standard(SelectStatement),
    Web(WebCTESpec),
}

/// Specification for WEB CTEs
#[derive(Debug, Clone)]
pub struct WebCTESpec {
    pub url: String,
    pub format: Option<DataFormat>,        // CSV, JSON, or auto-detect
    pub headers: Vec<(String, String)>,    // HTTP headers
    pub cache_seconds: Option<u64>,        // Cache duration
    pub method: Option<HttpMethod>,        // HTTP method (GET, POST, etc.)
    pub body: Option<String>,              // Request body for POST/PUT
    pub json_path: Option<String>, // JSON path to extract (e.g., "Result" for {Result: [...]})
    pub form_files: Vec<(String, String)>, // Multipart form files: (field_name, file_path)
    pub form_fields: Vec<(String, String)>, // Multipart form fields: (field_name, value)
}

/// HTTP methods for WEB CTEs
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

/// Data format for WEB CTEs
#[derive(Debug, Clone)]
pub enum DataFormat {
    CSV,
    JSON,
    Auto, // Auto-detect from Content-Type or extension
}

/// Table source - either a file/table name or a derived table (subquery/CTE)
#[derive(Debug, Clone)]
pub enum TableSource {
    Table(String), // Regular table from CSV/JSON
    DerivedTable {
        // Both CTE and subquery
        query: Box<SelectStatement>,
        alias: String, // Required alias for subqueries
    },
}

/// Join type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// Join operator for join conditions
#[derive(Debug, Clone, PartialEq)]
pub enum JoinOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

/// Single join condition
#[derive(Debug, Clone)]
pub struct SingleJoinCondition {
    pub left_column: String, // Column from left table (can include table prefix)
    pub operator: JoinOperator, // Join operator
    pub right_column: String, // Column from right table (can include table prefix)
}

/// Join condition - can be multiple conditions connected by AND
#[derive(Debug, Clone)]
pub struct JoinCondition {
    pub conditions: Vec<SingleJoinCondition>, // Multiple conditions connected by AND
}

/// Join clause structure
#[derive(Debug, Clone)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableSource,       // The table being joined
    pub alias: Option<String>,    // Optional alias for the joined table
    pub condition: JoinCondition, // ON condition(s)
}
