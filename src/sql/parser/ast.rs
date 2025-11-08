//! Abstract Syntax Tree (AST) definitions for SQL queries
//!
//! This module contains all the data structures that represent
//! the parsed SQL query structure.

// ===== Comment Types =====

/// Represents a SQL comment (line or block)
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// The comment text (without delimiters like -- or /* */)
    pub text: String,
    /// True for line comments (--), false for block comments (/* */)
    pub is_line_comment: bool,
}

impl Comment {
    /// Create a new line comment
    pub fn line(text: String) -> Self {
        Self {
            text,
            is_line_comment: true,
        }
    }

    /// Create a new block comment
    pub fn block(text: String) -> Self {
        Self {
            text,
            is_line_comment: false,
        }
    }
}

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

impl SortDirection {
    pub fn as_u8(&self) -> u8 {
        match self {
            SortDirection::Asc => 0,
            SortDirection::Desc => 1,
        }
    }
}

/// Legacy structure - kept for backward compatibility
/// New code should use OrderByItem
#[derive(Debug, Clone)]
pub struct OrderByColumn {
    pub column: String,
    pub direction: SortDirection,
}

/// Modern ORDER BY item that supports expressions
#[derive(Debug, Clone)]
pub struct OrderByItem {
    pub expr: SqlExpression,
    pub direction: SortDirection,
}

impl OrderByItem {
    /// Create from a simple column name (for backward compatibility)
    pub fn from_column_name(name: String, direction: SortDirection) -> Self {
        Self {
            expr: SqlExpression::Column(ColumnRef {
                name,
                quote_style: QuoteStyle::None,
                table_prefix: None,
            }),
            direction,
        }
    }

    /// Create from an expression
    pub fn from_expression(expr: SqlExpression, direction: SortDirection) -> Self {
        Self { expr, direction }
    }
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameUnit {
    Rows,
    Range,
}

impl FrameUnit {
    pub fn as_u8(&self) -> u8 {
        match self {
            FrameUnit::Rows => 0,
            FrameUnit::Range => 1,
        }
    }
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
    pub order_by: Vec<OrderByItem>,
    pub frame: Option<WindowFrame>, // Optional window frame
}

impl WindowSpec {
    /// Compute a fast hash for cache key purposes
    /// Much faster than format!("{:?}", spec) used previously
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash partition_by columns
        for col in &self.partition_by {
            col.hash(&mut hasher);
        }

        // Hash order_by items (just the column names for simplicity)
        for item in &self.order_by {
            // For ORDER BY, we typically just have column references
            // Hash a string representation for simplicity
            format!("{:?}", item.expr).hash(&mut hasher);
            item.direction.as_u8().hash(&mut hasher);
        }

        // Hash frame specification
        if let Some(ref frame) = self.frame {
            frame.unit.as_u8().hash(&mut hasher);
            format!("{:?}", frame.start).hash(&mut hasher);
            if let Some(ref end) = frame.end {
                format!("{:?}", end).hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

// ===== SELECT Statement Types =====

/// Set operation type for combining SELECT statements
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperation {
    /// UNION ALL - combines results without deduplication
    UnionAll,
    /// UNION - combines results with deduplication (not yet implemented)
    Union,
    /// INTERSECT - returns common rows (not yet implemented)
    Intersect,
    /// EXCEPT - returns rows from left not in right (not yet implemented)
    Except,
}

/// Represents a SELECT item - either a simple column or a computed expression with alias
#[derive(Debug, Clone)]
pub enum SelectItem {
    /// Simple column reference: "`column_name`"
    Column {
        column: ColumnRef,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
    /// Computed expression with alias: "expr AS alias"
    Expression {
        expr: SqlExpression,
        alias: String,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
    /// Star selector: "*" or "table.*"
    Star {
        table_prefix: Option<String>, // e.g., Some("p") for "p.*"
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
    /// Star with EXCLUDE: "* EXCLUDE (col1, col2)"
    StarExclude {
        table_prefix: Option<String>,
        excluded_columns: Vec<String>,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
}

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub distinct: bool,                // SELECT DISTINCT flag
    pub columns: Vec<String>,          // Keep for backward compatibility, will be deprecated
    pub select_items: Vec<SelectItem>, // New field for computed expressions

    // Modern unified FROM source (preferred)
    pub from_source: Option<TableSource>, // Unified FROM source (table, subquery, function, PIVOT, etc.)

    // Legacy FROM fields (deprecated but kept for compatibility during migration)
    #[deprecated(note = "Use from_source instead")]
    pub from_table: Option<String>,
    #[deprecated(note = "Use from_source instead")]
    pub from_subquery: Option<Box<SelectStatement>>, // Subquery in FROM clause
    #[deprecated(note = "Use from_source instead")]
    pub from_function: Option<TableFunction>, // Table function like RANGE() in FROM clause
    #[deprecated(note = "Use from_source instead")]
    pub from_alias: Option<String>, // Alias for subquery (AS name)

    pub joins: Vec<JoinClause>, // JOIN clauses
    pub where_clause: Option<WhereClause>,
    pub order_by: Option<Vec<OrderByItem>>, // Supports expressions: columns, aggregates, CASE, etc.
    pub group_by: Option<Vec<SqlExpression>>, // Changed from Vec<String> to support expressions
    pub having: Option<SqlExpression>,      // HAVING clause for post-aggregation filtering
    pub qualify: Option<SqlExpression>, // QUALIFY clause for window function filtering (Snowflake-style)
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub ctes: Vec<CTE>,                // Common Table Expressions (WITH clause)
    pub into_table: Option<IntoTable>, // INTO clause for temporary tables
    pub set_operations: Vec<(SetOperation, Box<SelectStatement>)>, // UNION/INTERSECT/EXCEPT operations

    // Comment preservation
    pub leading_comments: Vec<Comment>, // Comments before the SELECT keyword
    pub trailing_comment: Option<Comment>, // Trailing comment at end of statement
}

impl Default for SelectStatement {
    fn default() -> Self {
        SelectStatement {
            distinct: false,
            columns: Vec::new(),
            select_items: Vec::new(),
            from_source: None,
            #[allow(deprecated)]
            from_table: None,
            #[allow(deprecated)]
            from_subquery: None,
            #[allow(deprecated)]
            from_function: None,
            #[allow(deprecated)]
            from_alias: None,
            joins: Vec::new(),
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            qualify: None,
            limit: None,
            offset: None,
            ctes: Vec::new(),
            into_table: None,
            set_operations: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }
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
    pub template_vars: Vec<TemplateVar>, // Template variables for injection from temp tables
}

/// Template variable for injecting temp table data into WEB CTEs
#[derive(Debug, Clone)]
pub struct TemplateVar {
    pub placeholder: String,    // e.g., "${#instruments}"
    pub table_name: String,     // e.g., "#instruments"
    pub column: Option<String>, // e.g., Some("symbol") for ${#instruments.symbol}
    pub index: Option<usize>,   // e.g., Some(0) for ${#instruments[0]}
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

/// PIVOT aggregate specification
/// Example: MAX(AmountEaten)
#[derive(Debug, Clone)]
pub struct PivotAggregate {
    pub function: String, // e.g., "MAX", "SUM", "MIN", "AVG", "COUNT"
    pub column: String,   // e.g., "AmountEaten"
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
    /// PIVOT operation - transforms rows into columns
    /// Example: PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle'))
    Pivot {
        source: Box<TableSource>,  // The input table/subquery to pivot
        aggregate: PivotAggregate, // The aggregate function to apply
        pivot_column: String,      // Column whose values become new columns
        pivot_values: Vec<String>, // Specific values to pivot (becomes column names)
        alias: Option<String>,     // Optional alias for the pivoted result
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
    pub left_expr: SqlExpression, // Expression from left table (can be column, function call, etc.)
    pub operator: JoinOperator,   // Join operator
    pub right_expr: SqlExpression, // Expression from right table (can be column, function call, etc.)
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
