// Keep chrono imports for the parser implementation

// Re-exports for backward compatibility - these serve as both imports and re-exports
pub use super::parser::ast::{
    CTEType, Comment, Condition, DataFormat, FrameBound, FrameUnit, HttpMethod, IntoTable,
    JoinClause, JoinCondition, JoinOperator, JoinType, LogicalOp, OrderByColumn, SelectItem,
    SelectStatement, SetOperation, SingleJoinCondition, SortDirection, SqlExpression,
    TableFunction, TableSource, WebCTESpec, WhenBranch, WhereClause, WindowFrame, WindowSpec, CTE,
};
pub use super::parser::legacy::{ParseContext, ParseState, Schema, SqlParser, SqlToken, TableInfo};
pub use super::parser::lexer::{Lexer, LexerMode, Token};
pub use super::parser::ParserConfig;

// Re-export formatting functions for backward compatibility
pub use super::parser::formatter::{format_ast_tree, format_sql_pretty, format_sql_pretty_compact};

// New AST-based formatter
pub use super::parser::ast_formatter::{format_sql_ast, format_sql_ast_with_config, FormatConfig};

// Import the new expression modules
use super::parser::expressions::arithmetic::{
    parse_additive as parse_additive_expr, parse_multiplicative as parse_multiplicative_expr,
    ParseArithmetic,
};
use super::parser::expressions::case::{parse_case_expression as parse_case_expr, ParseCase};
use super::parser::expressions::comparison::{
    parse_comparison as parse_comparison_expr, parse_in_operator, ParseComparison,
};
use super::parser::expressions::logical::{
    parse_logical_and as parse_logical_and_expr, parse_logical_or as parse_logical_or_expr,
    ParseLogical,
};
use super::parser::expressions::primary::{
    parse_primary as parse_primary_expr, ParsePrimary, PrimaryExpressionContext,
};
use super::parser::expressions::ExpressionParser;

// Import function registry to check for function existence
use crate::sql::functions::{FunctionCategory, FunctionRegistry};
use crate::sql::generators::GeneratorRegistry;
use std::sync::Arc;

// Import Web CTE parser
use super::parser::web_cte_parser::WebCteParser;

/// Parser mode - controls whether comments are preserved in AST
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserMode {
    /// Standard parsing - skip comments (current behavior, backward compatible)
    Standard,
    /// Preserve comments in AST (opt-in for formatters)
    PreserveComments,
}

impl Default for ParserMode {
    fn default() -> Self {
        ParserMode::Standard
    }
}

pub struct Parser {
    lexer: Lexer,
    pub current_token: Token,    // Made public for web_cte_parser access
    in_method_args: bool,        // Track if we're parsing method arguments
    columns: Vec<String>,        // Known column names for context-aware parsing
    paren_depth: i32,            // Track parentheses nesting depth
    paren_depth_stack: Vec<i32>, // Stack to save/restore paren depth for nested contexts
    _config: ParserConfig,       // Parser configuration including case sensitivity
    debug_trace: bool,           // Enable detailed token-by-token trace
    trace_depth: usize,          // Track recursion depth for indented trace
    function_registry: Arc<FunctionRegistry>, // Function registry for validation
    generator_registry: Arc<GeneratorRegistry>, // Generator registry for table functions
    mode: ParserMode,            // Parser mode for comment preservation
}

impl Parser {
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self::with_mode(input, ParserMode::default())
    }

    /// Create a new parser with explicit mode for comment preservation
    #[must_use]
    pub fn with_mode(input: &str, mode: ParserMode) -> Self {
        // Choose lexer mode based on parser mode
        let lexer_mode = match mode {
            ParserMode::Standard => LexerMode::SkipComments,
            ParserMode::PreserveComments => LexerMode::PreserveComments,
        };

        let mut lexer = Lexer::with_mode(input, lexer_mode);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            in_method_args: false,
            columns: Vec::new(),
            paren_depth: 0,
            paren_depth_stack: Vec::new(),
            _config: ParserConfig::default(),
            debug_trace: false,
            trace_depth: 0,
            function_registry: Arc::new(FunctionRegistry::new()),
            generator_registry: Arc::new(GeneratorRegistry::new()),
            mode,
        }
    }

    #[must_use]
    pub fn with_config(input: &str, config: ParserConfig) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            in_method_args: false,
            columns: Vec::new(),
            paren_depth: 0,
            paren_depth_stack: Vec::new(),
            _config: config,
            debug_trace: false,
            trace_depth: 0,
            function_registry: Arc::new(FunctionRegistry::new()),
            generator_registry: Arc::new(GeneratorRegistry::new()),
            mode: ParserMode::default(),
        }
    }

    #[must_use]
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }

    #[must_use]
    pub fn with_debug_trace(mut self, enabled: bool) -> Self {
        self.debug_trace = enabled;
        self
    }

    #[must_use]
    pub fn with_function_registry(mut self, registry: Arc<FunctionRegistry>) -> Self {
        self.function_registry = registry;
        self
    }

    #[must_use]
    pub fn with_generator_registry(mut self, registry: Arc<GeneratorRegistry>) -> Self {
        self.generator_registry = registry;
        self
    }

    fn trace_enter(&mut self, context: &str) {
        if self.debug_trace {
            let indent = "  ".repeat(self.trace_depth);
            eprintln!("{}→ {} | Token: {:?}", indent, context, self.current_token);
            self.trace_depth += 1;
        }
    }

    fn trace_exit(&mut self, context: &str, result: &Result<impl std::fmt::Debug, String>) {
        if self.debug_trace {
            self.trace_depth = self.trace_depth.saturating_sub(1);
            let indent = "  ".repeat(self.trace_depth);
            match result {
                Ok(val) => eprintln!("{}← {} ✓ | Result: {:?}", indent, context, val),
                Err(e) => eprintln!("{}← {} ✗ | Error: {}", indent, context, e),
            }
        }
    }

    fn trace_token(&self, action: &str) {
        if self.debug_trace {
            let indent = "  ".repeat(self.trace_depth);
            eprintln!("{}  {} | Token: {:?}", indent, action, self.current_token);
        }
    }

    #[allow(dead_code)]
    fn peek_token(&self) -> Option<Token> {
        // Alternative peek that returns owned token
        let mut temp_lexer = self.lexer.clone();
        let next_token = temp_lexer.next_token();
        if matches!(next_token, Token::Eof) {
            None
        } else {
            Some(next_token)
        }
    }

    /// Check if current token is one of the reserved keywords that should stop parsing
    /// Check if an identifier string is a reserved keyword (for backward compatibility)
    /// This is used when the lexer hasn't properly tokenized keywords and they come through
    /// as Token::Identifier instead of their proper token types
    fn is_identifier_reserved(id: &str) -> bool {
        let id_upper = id.to_uppercase();
        matches!(
            id_upper.as_str(),
            "ORDER" | "HAVING" | "LIMIT" | "OFFSET" | "UNION" | "INTERSECT" | "EXCEPT"
        )
    }

    /// Get comparison operator string representation (for autocomplete context)
    const COMPARISON_OPERATORS: [&'static str; 6] = [" > ", " < ", " >= ", " <= ", " = ", " != "];

    pub fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.trace_token(&format!("Consuming expected {:?}", expected));
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            // Track parentheses depth
            self.update_paren_depth(&expected)?;

            self.current_token = self.lexer.next_token();
            Ok(())
        } else {
            // Provide better error messages for common cases
            let error_msg = match (&expected, &self.current_token) {
                (Token::RightParen, Token::Eof) if self.paren_depth > 0 => {
                    format!(
                        "Unclosed parenthesis - missing {} closing parenthes{}",
                        self.paren_depth,
                        if self.paren_depth == 1 { "is" } else { "es" }
                    )
                }
                (Token::RightParen, _) if self.paren_depth > 0 => {
                    format!(
                        "Expected closing parenthesis but found {:?} (currently {} unclosed parenthes{})",
                        self.current_token,
                        self.paren_depth,
                        if self.paren_depth == 1 { "is" } else { "es" }
                    )
                }
                _ => format!("Expected {:?}, found {:?}", expected, self.current_token),
            };
            Err(error_msg)
        }
    }

    pub fn advance(&mut self) {
        // Track parentheses depth when advancing
        match &self.current_token {
            Token::LeftParen => self.paren_depth += 1,
            Token::RightParen => {
                self.paren_depth -= 1;
                // Note: We don't check for < 0 here because advance() is used
                // in contexts where we're not necessarily expecting a right paren
            }
            _ => {}
        }
        let old_token = self.current_token.clone();
        self.current_token = self.lexer.next_token();
        if self.debug_trace {
            let indent = "  ".repeat(self.trace_depth);
            eprintln!(
                "{}  Advanced: {:?} → {:?}",
                indent, old_token, self.current_token
            );
        }
    }

    /// Collect all leading comments before a SQL construct
    /// This consumes comment tokens and returns them as a Vec<Comment>
    fn collect_leading_comments(&mut self) -> Vec<Comment> {
        let mut comments = Vec::new();
        loop {
            match &self.current_token {
                Token::LineComment(text) => {
                    comments.push(Comment::line(text.clone()));
                    self.advance();
                }
                Token::BlockComment(text) => {
                    comments.push(Comment::block(text.clone()));
                    self.advance();
                }
                _ => break,
            }
        }
        comments
    }

    /// Collect a trailing inline comment (on the same line)
    /// This consumes a single comment token if present
    fn collect_trailing_comment(&mut self) -> Option<Comment> {
        match &self.current_token {
            Token::LineComment(text) => {
                let comment = Some(Comment::line(text.clone()));
                self.advance();
                comment
            }
            Token::BlockComment(text) => {
                let comment = Some(Comment::block(text.clone()));
                self.advance();
                comment
            }
            _ => None,
        }
    }

    fn push_paren_depth(&mut self) {
        self.paren_depth_stack.push(self.paren_depth);
        self.paren_depth = 0;
    }

    fn pop_paren_depth(&mut self) {
        if let Some(depth) = self.paren_depth_stack.pop() {
            // Ignore the internal depth - just restore the saved value
            self.paren_depth = depth;
        }
    }

    pub fn parse(&mut self) -> Result<SelectStatement, String> {
        self.trace_enter("parse");

        // Collect leading comments FIRST (before checking for WITH or SELECT)
        // This allows comments before WITH clauses to be preserved
        let leading_comments = if self.mode == ParserMode::PreserveComments {
            self.collect_leading_comments()
        } else {
            vec![]
        };

        // Now check for WITH clause (after consuming comments)
        let result = if matches!(self.current_token, Token::With) {
            let mut stmt = self.parse_with_clause()?;
            // Attach the leading comments we collected
            stmt.leading_comments = leading_comments;
            stmt
        } else {
            // For SELECT without WITH, pass comments to inner parser
            let stmt = self.parse_select_statement_with_comments_public(leading_comments)?;
            self.check_balanced_parentheses()?;
            stmt
        };

        self.trace_exit("parse", &Ok(&result));
        Ok(result)
    }

    /// Public wrapper that accepts pre-collected comments and checks parens
    fn parse_select_statement_with_comments_public(
        &mut self,
        comments: Vec<Comment>,
    ) -> Result<SelectStatement, String> {
        self.parse_select_statement_with_comments(comments)
    }

    fn parse_with_clause(&mut self) -> Result<SelectStatement, String> {
        self.consume(Token::With)?;
        let ctes = self.parse_cte_list()?;

        // Parse the main SELECT statement - use inner version since we're already tracking parens
        let mut main_query = self.parse_select_statement_inner_no_comments()?;
        main_query.ctes = ctes;

        // Check for balanced parentheses at the end of parsing
        self.check_balanced_parentheses()?;

        Ok(main_query)
    }

    fn parse_with_clause_inner(&mut self) -> Result<SelectStatement, String> {
        self.consume(Token::With)?;
        let ctes = self.parse_cte_list()?;

        // Parse the main SELECT statement (without parenthesis checking for subqueries)
        let mut main_query = self.parse_select_statement_inner()?;
        main_query.ctes = ctes;

        Ok(main_query)
    }

    // Helper function to parse CTE list - eliminates duplication
    fn parse_cte_list(&mut self) -> Result<Vec<CTE>, String> {
        let mut ctes = Vec::new();

        // Parse CTEs
        loop {
            // Check for WEB keyword for each CTE (can be different for each one)
            let is_web = if matches!(&self.current_token, Token::Web) {
                self.trace_token("Found WEB keyword for CTE");
                self.advance();
                true
            } else {
                false
            };

            // Parse CTE name
            let name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => {
                    return Err(format!(
                        "Expected CTE name after {}",
                        if is_web { "WEB" } else { "WITH or comma" }
                    ))
                }
            };
            self.advance();

            // Optional column list: WITH t(col1, col2) AS ...
            let column_list = if matches!(self.current_token, Token::LeftParen) {
                self.advance();
                let cols = self.parse_identifier_list()?;
                self.consume(Token::RightParen)?;
                Some(cols)
            } else {
                None
            };

            // Expect AS
            self.consume(Token::As)?;

            let cte_type = if is_web {
                // Expect opening parenthesis for WEB CTE
                self.consume(Token::LeftParen)?;
                // Parse WEB CTE specification using dedicated parser
                let web_spec = WebCteParser::parse(self)?;
                // Consume closing parenthesis for WEB CTE
                self.consume(Token::RightParen)?;
                CTEType::Web(web_spec)
            } else {
                // For standard CTEs, push depth BEFORE consuming opening paren
                // This ensures the paren is counted in the inner context
                self.push_paren_depth();
                // Now consume opening parenthesis
                self.consume(Token::LeftParen)?;
                let query = self.parse_select_statement_inner()?;
                // Expect closing parenthesis while still in CTE context
                self.consume(Token::RightParen)?;
                // Now pop to restore outer depth after consuming both parens
                self.pop_paren_depth();
                CTEType::Standard(query)
            };

            ctes.push(CTE {
                name,
                column_list,
                cte_type,
            });

            // Check for more CTEs
            if !matches!(self.current_token, Token::Comma) {
                break;
            }
            self.advance();
        }

        Ok(ctes)
    }

    /// Helper function to parse an optional table alias (with or without AS keyword)
    fn parse_optional_alias(&mut self) -> Result<Option<String>, String> {
        if matches!(self.current_token, Token::As) {
            self.advance();
            match &self.current_token {
                Token::Identifier(name) => {
                    let alias = name.clone();
                    self.advance();
                    Ok(Some(alias))
                }
                token => {
                    // Check if it's a reserved keyword - provide helpful error
                    if let Some(keyword) = token.as_keyword_str() {
                        Err(format!(
                            "Reserved keyword '{}' cannot be used as column alias. Use a different name or quote it with double quotes: \"{}\"",
                            keyword,
                            keyword.to_lowercase()
                        ))
                    } else {
                        Err("Expected alias name after AS".to_string())
                    }
                }
            }
        } else if let Token::Identifier(name) = &self.current_token {
            // AS is optional for table aliases
            let alias = name.clone();
            self.advance();
            Ok(Some(alias))
        } else {
            Ok(None)
        }
    }

    /// Helper function to check if an identifier is valid (quoted or regular)
    fn is_valid_identifier(name: &str) -> bool {
        if name.starts_with('"') && name.ends_with('"') {
            // Quoted identifier - always valid
            true
        } else {
            // Regular identifier - check if it's alphanumeric or underscore
            name.chars().all(|c| c.is_alphanumeric() || c == '_')
        }
    }

    /// Helper function to update parentheses depth tracking
    fn update_paren_depth(&mut self, token: &Token) -> Result<(), String> {
        match token {
            Token::LeftParen => self.paren_depth += 1,
            Token::RightParen => {
                self.paren_depth -= 1;
                // Check for extra closing parenthesis
                if self.paren_depth < 0 {
                    return Err(
                        "Unexpected closing parenthesis - no matching opening parenthesis"
                            .to_string(),
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper function to parse comma-separated argument list
    fn parse_argument_list(&mut self) -> Result<Vec<SqlExpression>, String> {
        let mut args = Vec::new();

        if !matches!(self.current_token, Token::RightParen) {
            loop {
                args.push(self.parse_expression()?);

                if matches!(self.current_token, Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(args)
    }

    /// Helper function to check for balanced parentheses at the end of parsing
    fn check_balanced_parentheses(&self) -> Result<(), String> {
        if self.paren_depth > 0 {
            Err(format!(
                "Unclosed parenthesis - missing {} closing parenthes{}",
                self.paren_depth,
                if self.paren_depth == 1 { "is" } else { "es" }
            ))
        } else if self.paren_depth < 0 {
            Err("Extra closing parenthesis found - no matching opening parenthesis".to_string())
        } else {
            Ok(())
        }
    }

    /// Check if an expression contains aggregate functions (COUNT, SUM, AVG, etc.)
    /// This is used to detect unsupported patterns in HAVING clause
    fn contains_aggregate_function(expr: &SqlExpression) -> bool {
        match expr {
            SqlExpression::FunctionCall { name, args, .. } => {
                // Check if this is an aggregate function
                let upper_name = name.to_uppercase();
                let is_aggregate = matches!(
                    upper_name.as_str(),
                    "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "GROUP_CONCAT" | "STRING_AGG"
                );

                // If this is an aggregate, return true
                // Otherwise, recursively check arguments
                is_aggregate || args.iter().any(Self::contains_aggregate_function)
            }
            // Recursively check nested expressions
            SqlExpression::BinaryOp { left, right, .. } => {
                Self::contains_aggregate_function(left) || Self::contains_aggregate_function(right)
            }
            SqlExpression::Not { expr } => Self::contains_aggregate_function(expr),
            SqlExpression::MethodCall { args, .. } => {
                args.iter().any(Self::contains_aggregate_function)
            }
            SqlExpression::ChainedMethodCall { base, args, .. } => {
                Self::contains_aggregate_function(base)
                    || args.iter().any(Self::contains_aggregate_function)
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                when_branches.iter().any(|branch| {
                    Self::contains_aggregate_function(&branch.condition)
                        || Self::contains_aggregate_function(&branch.result)
                }) || else_branch
                    .as_ref()
                    .map_or(false, |e| Self::contains_aggregate_function(e))
            }
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => {
                Self::contains_aggregate_function(expr)
                    || when_branches.iter().any(|branch| {
                        Self::contains_aggregate_function(&branch.value)
                            || Self::contains_aggregate_function(&branch.result)
                    })
                    || else_branch
                        .as_ref()
                        .map_or(false, |e| Self::contains_aggregate_function(e))
            }
            SqlExpression::ScalarSubquery { query } => {
                // Subqueries can have their own aggregates, but that's fine
                // We're only checking the outer HAVING clause
                query
                    .having
                    .as_ref()
                    .map_or(false, |h| Self::contains_aggregate_function(h))
            }
            // Leaf nodes - no aggregates
            SqlExpression::Column(_)
            | SqlExpression::StringLiteral(_)
            | SqlExpression::NumberLiteral(_)
            | SqlExpression::BooleanLiteral(_)
            | SqlExpression::Null
            | SqlExpression::DateTimeConstructor { .. }
            | SqlExpression::DateTimeToday { .. } => false,

            // Window functions contain aggregates by definition
            SqlExpression::WindowFunction { .. } => true,

            // Between has three parts to check
            SqlExpression::Between { expr, lower, upper } => {
                Self::contains_aggregate_function(expr)
                    || Self::contains_aggregate_function(lower)
                    || Self::contains_aggregate_function(upper)
            }

            // IN list - check expr and all values
            SqlExpression::InList { expr, values } | SqlExpression::NotInList { expr, values } => {
                Self::contains_aggregate_function(expr)
                    || values.iter().any(Self::contains_aggregate_function)
            }

            // IN subquery - check expr and subquery
            SqlExpression::InSubquery { expr, subquery }
            | SqlExpression::NotInSubquery { expr, subquery } => {
                Self::contains_aggregate_function(expr)
                    || subquery
                        .having
                        .as_ref()
                        .map_or(false, |h| Self::contains_aggregate_function(h))
            }

            // UNNEST - check column expression
            SqlExpression::Unnest { column, .. } => Self::contains_aggregate_function(column),
        }
    }

    fn parse_select_statement(&mut self) -> Result<SelectStatement, String> {
        self.trace_enter("parse_select_statement");
        let result = self.parse_select_statement_inner()?;

        // Check for balanced parentheses at the end of parsing
        self.check_balanced_parentheses()?;

        Ok(result)
    }

    fn parse_select_statement_inner(&mut self) -> Result<SelectStatement, String> {
        // Collect leading comments ONLY in PreserveComments mode
        let leading_comments = if self.mode == ParserMode::PreserveComments {
            self.collect_leading_comments()
        } else {
            vec![]
        };

        self.parse_select_statement_with_comments(leading_comments)
    }

    /// Parse SELECT statement without collecting leading comments
    /// Used when comments were already collected (e.g., before WITH clause)
    fn parse_select_statement_inner_no_comments(&mut self) -> Result<SelectStatement, String> {
        self.parse_select_statement_with_comments(vec![])
    }

    /// Core SELECT parsing logic - takes pre-collected comments
    fn parse_select_statement_with_comments(
        &mut self,
        leading_comments: Vec<Comment>,
    ) -> Result<SelectStatement, String> {
        self.consume(Token::Select)?;

        // Check for DISTINCT keyword
        let distinct = if matches!(self.current_token, Token::Distinct) {
            self.advance();
            true
        } else {
            false
        };

        // Parse SELECT items (supports computed expressions)
        let select_items = self.parse_select_items()?;

        // Create legacy columns vector for backward compatibility
        let columns = select_items
            .iter()
            .map(|item| match item {
                SelectItem::Star { .. } => "*".to_string(),
                SelectItem::Column {
                    column: col_ref, ..
                } => col_ref.name.clone(),
                SelectItem::Expression { alias, .. } => alias.clone(),
            })
            .collect();

        // Parse INTO clause (for temporary tables) - comes immediately after SELECT items
        let into_table = if matches!(self.current_token, Token::Into) {
            self.advance();
            Some(self.parse_into_clause()?)
        } else {
            None
        };

        // Parse FROM clause - can be a table name, subquery, or table function
        let (from_table, from_subquery, from_function, from_alias) = if matches!(
            self.current_token,
            Token::From
        ) {
            self.advance();

            // Check for table function like RANGE()
            if let Token::Identifier(name) = &self.current_token.clone() {
                // Check if this is a table function by consulting the registry
                // We need to lookahead to see if there's a parenthesis to distinguish
                // between a function call and a table with the same name
                let has_paren = self.peek_token() == Some(Token::LeftParen);
                if self.debug_trace {
                    eprintln!(
                        "  Checking {} for table function, has_paren={}",
                        name, has_paren
                    );
                }

                // Check if it's a known table function or generator
                // In FROM clause context, prioritize generators over scalar functions
                let is_table_function = if has_paren {
                    // First check generator registry (for FROM clause context)
                    if self.debug_trace {
                        eprintln!("  Checking generator registry for {}", name.to_uppercase());
                    }
                    if let Some(_gen) = self.generator_registry.get(&name.to_uppercase()) {
                        if self.debug_trace {
                            eprintln!("  Found {} in generator registry", name);
                        }
                        self.trace_token(&format!("Found generator: {}", name));
                        true
                    } else {
                        // Then check if it's a table function in the function registry
                        if let Some(func) = self.function_registry.get(&name.to_uppercase()) {
                            let sig = func.signature();
                            let is_table_fn = sig.category == FunctionCategory::TableFunction;
                            if self.debug_trace {
                                eprintln!(
                                    "  Found {} in function registry, is_table_function={}",
                                    name, is_table_fn
                                );
                            }
                            if is_table_fn {
                                self.trace_token(&format!(
                                    "Found table function in function registry: {}",
                                    name
                                ));
                            }
                            is_table_fn
                        } else {
                            if self.debug_trace {
                                eprintln!("  {} not found in either registry", name);
                                self.trace_token(&format!(
                                    "Not found as generator or table function: {}",
                                    name
                                ));
                            }
                            false
                        }
                    }
                } else {
                    if self.debug_trace {
                        eprintln!("  No parenthesis after {}, treating as table", name);
                    }
                    false
                };

                if is_table_function {
                    // Parse table function
                    let function_name = name.clone();
                    self.advance(); // Skip function name

                    // Parse arguments
                    self.consume(Token::LeftParen)?;
                    let args = self.parse_argument_list()?;
                    self.consume(Token::RightParen)?;

                    // Optional alias
                    let alias = if matches!(self.current_token, Token::As) {
                        self.advance();
                        match &self.current_token {
                            Token::Identifier(name) => {
                                let alias = name.clone();
                                self.advance();
                                Some(alias)
                            }
                            token => {
                                if let Some(keyword) = token.as_keyword_str() {
                                    return Err(format!(
                                            "Reserved keyword '{}' cannot be used as column alias. Use a different name or quote it with double quotes: \"{}\"",
                                            keyword,
                                            keyword.to_lowercase()
                                        ));
                                } else {
                                    return Err("Expected alias name after AS".to_string());
                                }
                            }
                        }
                    } else if let Token::Identifier(name) = &self.current_token {
                        let alias = name.clone();
                        self.advance();
                        Some(alias)
                    } else {
                        None
                    };

                    (
                        None,
                        None,
                        Some(TableFunction::Generator {
                            name: function_name,
                            args,
                        }),
                        alias,
                    )
                } else {
                    // Not a RANGE, SPLIT, or generator function, so it's a regular table name
                    let table_name = name.clone();
                    self.advance();

                    // Check for optional alias
                    let alias = self.parse_optional_alias()?;

                    (Some(table_name), None, None, alias)
                }
            } else if matches!(self.current_token, Token::LeftParen) {
                // Check for subquery: FROM (SELECT ...) or FROM (WITH ... SELECT ...)
                self.advance();

                // Parse the subquery - it might start with WITH
                let subquery = if matches!(self.current_token, Token::With) {
                    self.parse_with_clause_inner()?
                } else {
                    self.parse_select_statement_inner()?
                };

                self.consume(Token::RightParen)?;

                // Subqueries must have an alias
                let alias = if matches!(self.current_token, Token::As) {
                    self.advance();
                    match &self.current_token {
                        Token::Identifier(name) => {
                            let alias = name.clone();
                            self.advance();
                            alias
                        }
                        token => {
                            if let Some(keyword) = token.as_keyword_str() {
                                return Err(format!(
                                        "Reserved keyword '{}' cannot be used as subquery alias. Use a different name or quote it with double quotes: \"{}\"",
                                        keyword,
                                        keyword.to_lowercase()
                                    ));
                            } else {
                                return Err("Expected alias name after AS".to_string());
                            }
                        }
                    }
                } else {
                    // AS is optional, but alias is required
                    match &self.current_token {
                        Token::Identifier(name) => {
                            let alias = name.clone();
                            self.advance();
                            alias
                        }
                        _ => {
                            return Err(
                                "Subquery in FROM must have an alias (e.g., AS t)".to_string()
                            )
                        }
                    }
                };

                (None, Some(Box::new(subquery)), None, Some(alias))
            } else {
                // Regular table name
                match &self.current_token {
                    Token::Identifier(table) => {
                        let table_name = table.clone();
                        self.advance();

                        // Check for optional alias
                        let alias = self.parse_optional_alias()?;

                        (Some(table_name), None, None, alias)
                    }
                    Token::QuotedIdentifier(table) => {
                        // Handle quoted table names
                        let table_name = table.clone();
                        self.advance();

                        // Check for optional alias
                        let alias = self.parse_optional_alias()?;

                        (Some(table_name), None, None, alias)
                    }
                    _ => return Err("Expected table name or subquery after FROM".to_string()),
                }
            }
        } else {
            (None, None, None, None)
        };

        // Parse JOIN clauses
        let mut joins = Vec::new();
        while self.is_join_token() {
            joins.push(self.parse_join_clause()?);
        }

        let where_clause = if matches!(self.current_token, Token::Where) {
            self.advance();
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let group_by = if matches!(self.current_token, Token::GroupBy) {
            self.advance();
            // Parse expressions instead of just identifiers for GROUP BY
            // This allows GROUP BY TIME_BUCKET(...), CASE ..., etc.
            Some(self.parse_expression_list()?)
        } else {
            None
        };

        // Parse HAVING clause (must come after GROUP BY)
        let having = if matches!(self.current_token, Token::Having) {
            if group_by.is_none() {
                return Err("HAVING clause requires GROUP BY".to_string());
            }
            self.advance();
            let having_expr = self.parse_expression()?;

            // Check if HAVING contains aggregate functions (not supported - use aliases instead)
            if Self::contains_aggregate_function(&having_expr) {
                return Err(
                    "HAVING clause with aggregate functions is not supported. \
                    Use an alias in SELECT for the aggregate and reference it in HAVING.\n\
                    Example: SELECT trader, COUNT(*) as trade_count FROM trades GROUP BY trader HAVING trade_count > 1"
                    .to_string()
                );
            }

            Some(having_expr)
        } else {
            None
        };

        // Parse ORDER BY clause (comes after GROUP BY and HAVING)
        let order_by = if matches!(self.current_token, Token::OrderBy) {
            self.trace_token("Found OrderBy token");
            self.advance();
            Some(self.parse_order_by_list()?)
        } else if let Token::Identifier(s) = &self.current_token {
            // This shouldn't happen if the lexer properly tokenizes ORDER BY
            // But keeping as fallback for compatibility
            if Self::is_identifier_reserved(s) && s.to_uppercase() == "ORDER" {
                self.trace_token("Warning: ORDER as identifier instead of OrderBy token");
                self.advance(); // consume ORDER
                if matches!(&self.current_token, Token::By) {
                    self.advance(); // consume BY
                    Some(self.parse_order_by_list()?)
                } else {
                    return Err("Expected BY after ORDER".to_string());
                }
            } else {
                None
            }
        } else {
            None
        };

        // Parse LIMIT clause
        let limit = if matches!(self.current_token, Token::Limit) {
            self.advance();
            match &self.current_token {
                Token::NumberLiteral(num) => {
                    let limit_val = num
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid LIMIT value: {num}"))?;
                    self.advance();
                    Some(limit_val)
                }
                _ => return Err("Expected number after LIMIT".to_string()),
            }
        } else {
            None
        };

        // Parse OFFSET clause
        let offset = if matches!(self.current_token, Token::Offset) {
            self.advance();
            match &self.current_token {
                Token::NumberLiteral(num) => {
                    let offset_val = num
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid OFFSET value: {num}"))?;
                    self.advance();
                    Some(offset_val)
                }
                _ => return Err("Expected number after OFFSET".to_string()),
            }
        } else {
            None
        };

        // Parse INTO clause (alternative position - SQL Server also supports INTO after all clauses)
        // This handles: SELECT * FROM table WHERE x > 5 INTO #temp
        // If INTO was already parsed after SELECT, this will be None (can't have two INTOs)
        let into_table = if into_table.is_none() && matches!(self.current_token, Token::Into) {
            self.advance();
            Some(self.parse_into_clause()?)
        } else {
            into_table // Keep the one from after SELECT if it exists
        };

        // Parse UNION/INTERSECT/EXCEPT operations
        let set_operations = self.parse_set_operations()?;

        // Collect trailing comment ONLY in PreserveComments mode
        let trailing_comment = if self.mode == ParserMode::PreserveComments {
            self.collect_trailing_comment()
        } else {
            None
        };

        Ok(SelectStatement {
            distinct,
            columns,
            select_items,
            from_table,
            from_subquery,
            from_function,
            from_alias,
            joins,
            where_clause,
            order_by,
            group_by,
            having,
            limit,
            offset,
            ctes: Vec::new(), // Will be populated by WITH clause parser
            into_table,
            set_operations,
            leading_comments,
            trailing_comment,
        })
    }

    /// Parse UNION/INTERSECT/EXCEPT operations
    /// Returns a vector of (operation, select_statement) pairs
    fn parse_set_operations(
        &mut self,
    ) -> Result<Vec<(SetOperation, Box<SelectStatement>)>, String> {
        let mut operations = Vec::new();

        while matches!(
            self.current_token,
            Token::Union | Token::Intersect | Token::Except
        ) {
            // Determine the operation type
            let operation = match &self.current_token {
                Token::Union => {
                    self.advance();
                    // Check for ALL keyword
                    if let Token::Identifier(id) = &self.current_token {
                        if id.to_uppercase() == "ALL" {
                            self.advance();
                            SetOperation::UnionAll
                        } else {
                            SetOperation::Union
                        }
                    } else {
                        SetOperation::Union
                    }
                }
                Token::Intersect => {
                    self.advance();
                    SetOperation::Intersect
                }
                Token::Except => {
                    self.advance();
                    SetOperation::Except
                }
                _ => unreachable!(),
            };

            // Parse the next SELECT statement
            let next_select = self.parse_select_statement_inner()?;

            operations.push((operation, Box::new(next_select)));
        }

        Ok(operations)
    }

    /// Parse SELECT items that support computed expressions with aliases
    fn parse_select_items(&mut self) -> Result<Vec<SelectItem>, String> {
        let mut items = Vec::new();

        loop {
            // Check for qualified star (table.*) or unqualified star (*)
            // First check if we have identifier.* pattern
            if let Token::Identifier(name) = &self.current_token.clone() {
                // Peek ahead to check for .* pattern
                let saved_pos = self.lexer.clone();
                let saved_token = self.current_token.clone();
                let table_name = name.clone();

                self.advance();

                if matches!(self.current_token, Token::Dot) {
                    self.advance();
                    if matches!(self.current_token, Token::Star) {
                        // This is table.* pattern
                        items.push(SelectItem::Star {
                            table_prefix: Some(table_name),
                            leading_comments: vec![],
                            trailing_comment: None,
                        });
                        self.advance();

                        // Continue to next item or end
                        if matches!(self.current_token, Token::Comma) {
                            self.advance();
                            continue;
                        } else {
                            break;
                        }
                    }
                }

                // Not table.*, restore position and continue with normal parsing
                self.lexer = saved_pos;
                self.current_token = saved_token;
            }

            // Check for unqualified *
            if matches!(self.current_token, Token::Star) {
                items.push(SelectItem::Star {
                    table_prefix: None,
                    leading_comments: vec![],
                    trailing_comment: None,
                });
                self.advance();
            } else {
                // Parse expression or column
                let expr = self.parse_comparison()?; // Use comparison to support IS NULL and other comparisons

                // Check for AS alias
                let alias = if matches!(self.current_token, Token::As) {
                    self.advance();
                    match &self.current_token {
                        Token::Identifier(alias_name) => {
                            let alias = alias_name.clone();
                            self.advance();
                            alias
                        }
                        Token::QuotedIdentifier(alias_name) => {
                            let alias = alias_name.clone();
                            self.advance();
                            alias
                        }
                        token => {
                            if let Some(keyword) = token.as_keyword_str() {
                                return Err(format!(
                                    "Reserved keyword '{}' cannot be used as column alias. Use a different name or quote it with double quotes: \"{}\"",
                                    keyword,
                                    keyword.to_lowercase()
                                ));
                            } else {
                                return Err("Expected alias name after AS".to_string());
                            }
                        }
                    }
                } else {
                    // Generate default alias based on expression
                    match &expr {
                        SqlExpression::Column(col_ref) => col_ref.name.clone(),
                        _ => format!("expr_{}", items.len() + 1), // Default alias for computed expressions
                    }
                };

                // Create SelectItem based on expression type
                let item = match expr {
                    SqlExpression::Column(col_ref) if alias == col_ref.name => {
                        // Simple column reference without alias
                        SelectItem::Column {
                            column: col_ref,
                            leading_comments: vec![],
                            trailing_comment: None,
                        }
                    }
                    _ => {
                        // Computed expression or column with different alias
                        SelectItem::Expression {
                            expr,
                            alias,
                            leading_comments: vec![],
                            trailing_comment: None,
                        }
                    }
                };

                items.push(item);
            }

            // Check for comma to continue
            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(items)
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, String> {
        let mut identifiers = Vec::new();

        loop {
            match &self.current_token {
                Token::Identifier(id) => {
                    // Check if this is a reserved keyword that should stop identifier parsing
                    if Self::is_identifier_reserved(id) {
                        // Stop parsing identifiers if we hit a reserved keyword
                        break;
                    }
                    identifiers.push(id.clone());
                    self.advance();
                }
                Token::QuotedIdentifier(id) => {
                    // Handle quoted identifiers like "Customer Id"
                    identifiers.push(id.clone());
                    self.advance();
                }
                _ => {
                    // Stop parsing if we hit any other token type
                    break;
                }
            }

            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        if identifiers.is_empty() {
            return Err("Expected at least one identifier".to_string());
        }

        Ok(identifiers)
    }

    fn parse_window_spec(&mut self) -> Result<WindowSpec, String> {
        let mut partition_by = Vec::new();
        let mut order_by = Vec::new();

        // Check for PARTITION BY
        if matches!(self.current_token, Token::Partition) {
            self.advance(); // consume PARTITION
            if !matches!(self.current_token, Token::By) {
                return Err("Expected BY after PARTITION".to_string());
            }
            self.advance(); // consume BY

            // Parse partition columns
            partition_by = self.parse_identifier_list()?;
        }

        // Check for ORDER BY
        if matches!(self.current_token, Token::OrderBy) {
            self.advance(); // consume ORDER BY (as single token)
            order_by = self.parse_order_by_list()?;
        } else if let Token::Identifier(s) = &self.current_token {
            if Self::is_identifier_reserved(s) && s.to_uppercase() == "ORDER" {
                // Handle ORDER BY as two tokens
                self.advance(); // consume ORDER
                if !matches!(self.current_token, Token::By) {
                    return Err("Expected BY after ORDER".to_string());
                }
                self.advance(); // consume BY
                order_by = self.parse_order_by_list()?;
            }
        }

        // Parse optional window frame (ROWS/RANGE BETWEEN ... AND ...)
        let frame = self.parse_window_frame()?;

        Ok(WindowSpec {
            partition_by,
            order_by,
            frame,
        })
    }

    fn parse_order_by_list(&mut self) -> Result<Vec<OrderByColumn>, String> {
        let mut order_columns = Vec::new();

        loop {
            let column = match &self.current_token {
                Token::Identifier(id) => {
                    let col = id.clone();
                    self.advance();

                    // Check for qualified column name (table.column)
                    if matches!(self.current_token, Token::Dot) {
                        self.advance();
                        match &self.current_token {
                            Token::Identifier(col_name) => {
                                let mut qualified = col;
                                qualified.push('.');
                                qualified.push_str(col_name);
                                self.advance();
                                qualified
                            }
                            _ => return Err("Expected column name after '.'".to_string()),
                        }
                    } else {
                        col
                    }
                }
                Token::QuotedIdentifier(id) => {
                    let col = id.clone();
                    self.advance();
                    col
                }
                Token::NumberLiteral(num) if self.columns.iter().any(|col| col == num) => {
                    // Support numeric column names like "202204"
                    let col = num.clone();
                    self.advance();
                    col
                }
                // Handle window keywords that can be column names
                Token::Row => {
                    self.advance();
                    "row".to_string()
                }
                Token::Rows => {
                    self.advance();
                    "rows".to_string()
                }
                Token::Range => {
                    self.advance();
                    "range".to_string()
                }
                _ => return Err("Expected column name in ORDER BY".to_string()),
            };

            // Check for ASC/DESC
            let direction = match &self.current_token {
                Token::Asc => {
                    self.advance();
                    SortDirection::Asc
                }
                Token::Desc => {
                    self.advance();
                    SortDirection::Desc
                }
                _ => SortDirection::Asc, // Default to ASC if not specified
            };

            order_columns.push(OrderByColumn { column, direction });

            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(order_columns)
    }

    /// Parse INTO clause for temporary tables
    /// Syntax: INTO #table_name
    fn parse_into_clause(&mut self) -> Result<IntoTable, String> {
        // Expect an identifier starting with #
        let name = match &self.current_token {
            Token::Identifier(id) if id.starts_with('#') => {
                let table_name = id.clone();
                self.advance();
                table_name
            }
            Token::Identifier(id) => {
                return Err(format!(
                    "Temporary table name must start with #, got: {}",
                    id
                ));
            }
            _ => {
                return Err(
                    "Expected temporary table name (starting with #) after INTO".to_string()
                );
            }
        };

        Ok(IntoTable { name })
    }

    fn parse_window_frame(&mut self) -> Result<Option<WindowFrame>, String> {
        // Check for ROWS or RANGE keyword
        let unit = match &self.current_token {
            Token::Rows => {
                self.advance();
                FrameUnit::Rows
            }
            Token::Identifier(id) if id.to_uppercase() == "RANGE" => {
                // RANGE as window frame unit
                self.advance();
                FrameUnit::Range
            }
            _ => return Ok(None), // No window frame specified
        };

        // Check for BETWEEN or just a single bound
        let (start, end) = if let Token::Between = &self.current_token {
            self.advance(); // consume BETWEEN
                            // Parse start bound
            let start = self.parse_frame_bound()?;

            // Expect AND
            if !matches!(&self.current_token, Token::And) {
                return Err("Expected AND after window frame start bound".to_string());
            }
            self.advance();

            // Parse end bound
            let end = self.parse_frame_bound()?;
            (start, Some(end))
        } else {
            // Single bound (e.g., "ROWS 5 PRECEDING")
            let bound = self.parse_frame_bound()?;
            (bound, None)
        };

        Ok(Some(WindowFrame { unit, start, end }))
    }

    fn parse_frame_bound(&mut self) -> Result<FrameBound, String> {
        match &self.current_token {
            Token::Unbounded => {
                self.advance();
                match &self.current_token {
                    Token::Preceding => {
                        self.advance();
                        Ok(FrameBound::UnboundedPreceding)
                    }
                    Token::Following => {
                        self.advance();
                        Ok(FrameBound::UnboundedFollowing)
                    }
                    _ => Err("Expected PRECEDING or FOLLOWING after UNBOUNDED".to_string()),
                }
            }
            Token::Current => {
                self.advance();
                if matches!(&self.current_token, Token::Row) {
                    self.advance();
                    return Ok(FrameBound::CurrentRow);
                }
                Err("Expected ROW after CURRENT".to_string())
            }
            Token::NumberLiteral(num) => {
                let n: i64 = num
                    .parse()
                    .map_err(|_| "Invalid number in window frame".to_string())?;
                self.advance();
                match &self.current_token {
                    Token::Preceding => {
                        self.advance();
                        Ok(FrameBound::Preceding(n))
                    }
                    Token::Following => {
                        self.advance();
                        Ok(FrameBound::Following(n))
                    }
                    _ => Err("Expected PRECEDING or FOLLOWING after number".to_string()),
                }
            }
            _ => Err("Invalid window frame bound".to_string()),
        }
    }

    fn parse_where_clause(&mut self) -> Result<WhereClause, String> {
        // Parse the entire WHERE clause as a single expression tree
        // The logical operators (AND/OR) are now handled within parse_expression
        let expr = self.parse_expression()?;

        // Check for unexpected closing parenthesis
        if matches!(self.current_token, Token::RightParen) && self.paren_depth <= 0 {
            return Err(
                "Unexpected closing parenthesis - no matching opening parenthesis".to_string(),
            );
        }

        // Create a single condition with the entire expression
        let conditions = vec![Condition {
            expr,
            connector: None,
        }];

        Ok(WhereClause { conditions })
    }

    fn parse_expression(&mut self) -> Result<SqlExpression, String> {
        self.trace_enter("parse_expression");
        // Start with logical OR as the lowest precedence operator
        // The hierarchy is: OR -> AND -> comparison -> additive -> multiplicative -> primary
        let mut left = self.parse_logical_or()?;

        // Handle IN operator (not preceded by NOT)
        // This uses the modular comparison module
        left = parse_in_operator(self, left)?;

        let result = Ok(left);
        self.trace_exit("parse_expression", &result);
        result
    }

    fn parse_comparison(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular comparison expression parser
        parse_comparison_expr(self)
    }

    fn parse_additive(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular arithmetic expression parser
        parse_additive_expr(self)
    }

    fn parse_multiplicative(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular arithmetic expression parser
        parse_multiplicative_expr(self)
    }

    fn parse_logical_or(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular logical expression parser
        parse_logical_or_expr(self)
    }

    fn parse_logical_and(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular logical expression parser
        parse_logical_and_expr(self)
    }

    fn parse_case_expression(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular CASE expression parser
        parse_case_expr(self)
    }

    fn parse_primary(&mut self) -> Result<SqlExpression, String> {
        // Use the new modular primary expression parser
        // Clone the necessary data to avoid borrowing issues
        let columns = self.columns.clone();
        let in_method_args = self.in_method_args;
        let ctx = PrimaryExpressionContext {
            columns: &columns,
            in_method_args,
        };
        parse_primary_expr(self, &ctx)
    }

    // Keep the old implementation temporarily for reference (will be removed)
    fn parse_method_args(&mut self) -> Result<Vec<SqlExpression>, String> {
        // Set flag to indicate we're parsing method arguments
        self.in_method_args = true;

        let args = self.parse_argument_list()?;

        // Clear the flag
        self.in_method_args = false;

        Ok(args)
    }

    fn parse_function_args(&mut self) -> Result<(Vec<SqlExpression>, bool), String> {
        let mut args = Vec::new();
        let mut has_distinct = false;

        if !matches!(self.current_token, Token::RightParen) {
            // Check if first argument starts with DISTINCT
            if matches!(self.current_token, Token::Distinct) {
                self.advance(); // consume DISTINCT
                has_distinct = true;
            }

            // Parse the expression (either after DISTINCT or directly)
            args.push(self.parse_additive()?);

            // Parse any remaining arguments (DISTINCT only applies to first arg for aggregates)
            while matches!(self.current_token, Token::Comma) {
                self.advance();
                args.push(self.parse_additive()?);
            }
        }

        Ok((args, has_distinct))
    }

    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String> {
        let mut expressions = Vec::new();

        loop {
            expressions.push(self.parse_expression()?);

            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(expressions)
    }

    #[must_use]
    pub fn get_position(&self) -> usize {
        self.lexer.get_position()
    }

    // Check if current token is a JOIN-related token
    fn is_join_token(&self) -> bool {
        matches!(
            self.current_token,
            Token::Join | Token::Inner | Token::Left | Token::Right | Token::Full | Token::Cross
        )
    }

    // Parse a JOIN clause
    fn parse_join_clause(&mut self) -> Result<JoinClause, String> {
        // Determine join type
        let join_type = match &self.current_token {
            Token::Join => {
                self.advance();
                JoinType::Inner // Default JOIN is INNER JOIN
            }
            Token::Inner => {
                self.advance();
                if !matches!(self.current_token, Token::Join) {
                    return Err("Expected JOIN after INNER".to_string());
                }
                self.advance();
                JoinType::Inner
            }
            Token::Left => {
                self.advance();
                // Handle optional OUTER keyword
                if matches!(self.current_token, Token::Outer) {
                    self.advance();
                }
                if !matches!(self.current_token, Token::Join) {
                    return Err("Expected JOIN after LEFT".to_string());
                }
                self.advance();
                JoinType::Left
            }
            Token::Right => {
                self.advance();
                // Handle optional OUTER keyword
                if matches!(self.current_token, Token::Outer) {
                    self.advance();
                }
                if !matches!(self.current_token, Token::Join) {
                    return Err("Expected JOIN after RIGHT".to_string());
                }
                self.advance();
                JoinType::Right
            }
            Token::Full => {
                self.advance();
                // Handle optional OUTER keyword
                if matches!(self.current_token, Token::Outer) {
                    self.advance();
                }
                if !matches!(self.current_token, Token::Join) {
                    return Err("Expected JOIN after FULL".to_string());
                }
                self.advance();
                JoinType::Full
            }
            Token::Cross => {
                self.advance();
                if !matches!(self.current_token, Token::Join) {
                    return Err("Expected JOIN after CROSS".to_string());
                }
                self.advance();
                JoinType::Cross
            }
            _ => return Err("Expected JOIN keyword".to_string()),
        };

        // Parse the table being joined
        let (table, alias) = self.parse_join_table_source()?;

        // Parse ON condition (required for all joins except CROSS JOIN)
        let condition = if join_type == JoinType::Cross {
            // CROSS JOIN doesn't have ON condition - create empty condition
            JoinCondition { conditions: vec![] }
        } else {
            if !matches!(self.current_token, Token::On) {
                return Err("Expected ON keyword after JOIN table".to_string());
            }
            self.advance();
            self.parse_join_condition()?
        };

        Ok(JoinClause {
            join_type,
            table,
            alias,
            condition,
        })
    }

    fn parse_join_table_source(&mut self) -> Result<(TableSource, Option<String>), String> {
        let table = match &self.current_token {
            Token::Identifier(name) => {
                let table_name = name.clone();
                self.advance();
                TableSource::Table(table_name)
            }
            Token::LeftParen => {
                // Subquery as table source
                self.advance();
                let subquery = self.parse_select_statement_inner()?;
                if !matches!(self.current_token, Token::RightParen) {
                    return Err("Expected ')' after subquery".to_string());
                }
                self.advance();

                // Subqueries must have an alias
                let alias = match &self.current_token {
                    Token::Identifier(alias_name) => {
                        let alias = alias_name.clone();
                        self.advance();
                        alias
                    }
                    Token::As => {
                        self.advance();
                        match &self.current_token {
                            Token::Identifier(alias_name) => {
                                let alias = alias_name.clone();
                                self.advance();
                                alias
                            }
                            _ => return Err("Expected alias after AS keyword".to_string()),
                        }
                    }
                    _ => return Err("Subqueries must have an alias".to_string()),
                };

                return Ok((
                    TableSource::DerivedTable {
                        query: Box::new(subquery),
                        alias: alias.clone(),
                    },
                    Some(alias),
                ));
            }
            _ => return Err("Expected table name or subquery in JOIN clause".to_string()),
        };

        // Check for optional alias
        let alias = match &self.current_token {
            Token::Identifier(alias_name) => {
                let alias = alias_name.clone();
                self.advance();
                Some(alias)
            }
            Token::As => {
                self.advance();
                match &self.current_token {
                    Token::Identifier(alias_name) => {
                        let alias = alias_name.clone();
                        self.advance();
                        Some(alias)
                    }
                    _ => return Err("Expected alias after AS keyword".to_string()),
                }
            }
            _ => None,
        };

        Ok((table, alias))
    }

    fn parse_join_condition(&mut self) -> Result<JoinCondition, String> {
        let mut conditions = Vec::new();

        // Parse first condition
        conditions.push(self.parse_single_join_condition()?);

        // Parse additional conditions connected by AND
        while matches!(self.current_token, Token::And) {
            self.advance(); // consume AND
            conditions.push(self.parse_single_join_condition()?);
        }

        Ok(JoinCondition { conditions })
    }

    fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
        // Parse left side as additive expression (stops before comparison operators)
        // This allows the comparison operator to be explicitly parsed by this function
        let left_expr = self.parse_additive()?;

        // Parse operator
        let operator = match &self.current_token {
            Token::Equal => JoinOperator::Equal,
            Token::NotEqual => JoinOperator::NotEqual,
            Token::LessThan => JoinOperator::LessThan,
            Token::LessThanOrEqual => JoinOperator::LessThanOrEqual,
            Token::GreaterThan => JoinOperator::GreaterThan,
            Token::GreaterThanOrEqual => JoinOperator::GreaterThanOrEqual,
            _ => return Err("Expected comparison operator in JOIN condition".to_string()),
        };
        self.advance();

        // Parse right side as additive expression (stops before comparison operators)
        let right_expr = self.parse_additive()?;

        Ok(SingleJoinCondition {
            left_expr,
            operator,
            right_expr,
        })
    }

    fn parse_column_reference(&mut self) -> Result<String, String> {
        match &self.current_token {
            Token::Identifier(name) => {
                let mut column_ref = name.clone();
                self.advance();

                // Check for table.column notation
                if matches!(self.current_token, Token::Dot) {
                    self.advance();
                    match &self.current_token {
                        Token::Identifier(col_name) => {
                            column_ref.push('.');
                            column_ref.push_str(col_name);
                            self.advance();
                        }
                        _ => return Err("Expected column name after '.'".to_string()),
                    }
                }

                Ok(column_ref)
            }
            _ => Err("Expected column reference".to_string()),
        }
    }
}

// Context detection for cursor position
#[derive(Debug, Clone)]
pub enum CursorContext {
    SelectClause,
    FromClause,
    WhereClause,
    OrderByClause,
    AfterColumn(String),
    AfterLogicalOp(LogicalOp),
    AfterComparisonOp(String, String), // column_name, operator
    InMethodCall(String, String),      // object, method
    InExpression,
    Unknown,
}

/// Safe UTF-8 string slicing that ensures we don't slice in the middle of a character
fn safe_slice_to(s: &str, pos: usize) -> &str {
    if pos >= s.len() {
        return s;
    }

    // Find the nearest valid character boundary at or before pos
    let mut safe_pos = pos;
    while safe_pos > 0 && !s.is_char_boundary(safe_pos) {
        safe_pos -= 1;
    }

    &s[..safe_pos]
}

/// Safe UTF-8 string slicing from a position to the end
fn safe_slice_from(s: &str, pos: usize) -> &str {
    if pos >= s.len() {
        return "";
    }

    // Find the nearest valid character boundary at or after pos
    let mut safe_pos = pos;
    while safe_pos < s.len() && !s.is_char_boundary(safe_pos) {
        safe_pos += 1;
    }

    &s[safe_pos..]
}

#[must_use]
pub fn detect_cursor_context(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    let truncated = safe_slice_to(query, cursor_pos);
    let mut parser = Parser::new(truncated);

    // Try to parse as much as possible
    if let Ok(stmt) = parser.parse() {
        let (ctx, partial) = analyze_statement(&stmt, truncated, cursor_pos);
        #[cfg(test)]
        println!("analyze_statement returned: {ctx:?}, {partial:?} for query: '{truncated}'");
        (ctx, partial)
    } else {
        // Partial parse - analyze what we have
        let (ctx, partial) = analyze_partial(truncated, cursor_pos);
        #[cfg(test)]
        println!("analyze_partial returned: {ctx:?}, {partial:?} for query: '{truncated}'");
        (ctx, partial)
    }
}

#[must_use]
pub fn tokenize_query(query: &str) -> Vec<String> {
    let mut lexer = Lexer::new(query);
    let tokens = lexer.tokenize_all();
    tokens.iter().map(|t| format!("{t:?}")).collect()
}

#[must_use]
/// Helper function to find the start of a quoted string searching backwards
fn find_quote_start(bytes: &[u8], mut pos: usize) -> Option<usize> {
    // Skip the closing quote and search backwards
    if pos > 0 {
        pos -= 1;
        while pos > 0 {
            if bytes[pos] == b'"' {
                // Check if it's not an escaped quote
                if pos == 0 || bytes[pos - 1] != b'\\' {
                    return Some(pos);
                }
            }
            pos -= 1;
        }
        // Check position 0 separately
        if bytes[0] == b'"' {
            return Some(0);
        }
    }
    None
}

/// Helper function to handle method call context after validation
fn handle_method_call_context(col_name: &str, after_dot: &str) -> (CursorContext, Option<String>) {
    // Check if there's a partial method name after the dot
    let partial_method = if after_dot.is_empty() {
        None
    } else if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(after_dot.to_string())
    } else {
        None
    };

    // For AfterColumn context, strip quotes if present for consistency
    let col_name_for_context =
        if col_name.starts_with('"') && col_name.ends_with('"') && col_name.len() > 2 {
            col_name[1..col_name.len() - 1].to_string()
        } else {
            col_name.to_string()
        };

    (
        CursorContext::AfterColumn(col_name_for_context),
        partial_method,
    )
}

/// Helper function to check if we're after a comparison operator
fn check_after_comparison_operator(query: &str) -> Option<(CursorContext, Option<String>)> {
    for op in &Parser::COMPARISON_OPERATORS {
        if let Some(op_pos) = query.rfind(op) {
            let before_op = safe_slice_to(query, op_pos);
            let after_op_start = op_pos + op.len();
            let after_op = if after_op_start < query.len() {
                &query[after_op_start..]
            } else {
                ""
            };

            // Check if we have a column name before the operator
            if let Some(col_name) = before_op.split_whitespace().last() {
                if col_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    // Check if we're at or near the end of the query
                    let after_op_trimmed = after_op.trim();
                    if after_op_trimmed.is_empty()
                        || (after_op_trimmed
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_')
                            && !after_op_trimmed.contains('('))
                    {
                        let partial = if after_op_trimmed.is_empty() {
                            None
                        } else {
                            Some(after_op_trimmed.to_string())
                        };
                        return Some((
                            CursorContext::AfterComparisonOp(
                                col_name.to_string(),
                                op.trim().to_string(),
                            ),
                            partial,
                        ));
                    }
                }
            }
        }
    }
    None
}

fn analyze_statement(
    stmt: &SelectStatement,
    query: &str,
    _cursor_pos: usize,
) -> (CursorContext, Option<String>) {
    // First check for method call context (e.g., "columnName." or "columnName.Con")
    let trimmed = query.trim();

    // Check if we're after a comparison operator (e.g., "createdDate > ")
    if let Some(result) = check_after_comparison_operator(query) {
        return result;
    }

    // First check if we're after AND/OR - this takes precedence
    // Helper function to check if string ends with a logical operator
    let ends_with_logical_op = |s: &str| -> bool {
        let s_upper = s.to_uppercase();
        s_upper.ends_with(" AND") || s_upper.ends_with(" OR")
    };

    if ends_with_logical_op(trimmed) {
        // Don't check for method context if we're clearly after a logical operator
    } else {
        // Look for the last dot in the query
        if let Some(dot_pos) = trimmed.rfind('.') {
            // Check if we're after a column name and dot
            let before_dot = safe_slice_to(trimmed, dot_pos);
            let after_dot_start = dot_pos + 1;
            let after_dot = if after_dot_start < trimmed.len() {
                &trimmed[after_dot_start..]
            } else {
                ""
            };

            // Check if the part after dot looks like an incomplete method call
            // (not a complete method call like "Contains(...)")
            if !after_dot.contains('(') {
                // Try to extract the column name - could be quoted or regular
                let col_name = if before_dot.ends_with('"') {
                    // Handle quoted identifier - search backwards for matching opening quote
                    let bytes = before_dot.as_bytes();
                    let pos = before_dot.len() - 1; // Position of closing quote

                    find_quote_start(bytes, pos).map(|start| safe_slice_from(before_dot, start))
                } else {
                    // Regular identifier - get the last word, handling parentheses
                    // Strip all leading parentheses
                    before_dot
                        .split_whitespace()
                        .last()
                        .map(|word| word.trim_start_matches('('))
                };

                if let Some(col_name) = col_name {
                    // For quoted identifiers, keep the quotes, for regular identifiers check validity
                    let is_valid = Parser::is_valid_identifier(col_name);

                    if is_valid {
                        return handle_method_call_context(col_name, after_dot);
                    }
                }
            }
        }
    }

    // Check if we're in WHERE clause
    if let Some(where_clause) = &stmt.where_clause {
        // Check if query ends with AND/OR (with or without trailing space/partial)
        let trimmed_upper = trimmed.to_uppercase();
        if trimmed_upper.ends_with(" AND") || trimmed_upper.ends_with(" OR") {
            let op = if trimmed_upper.ends_with(" AND") {
                LogicalOp::And
            } else {
                LogicalOp::Or
            };
            return (CursorContext::AfterLogicalOp(op), None);
        }

        // Check if we have AND/OR followed by a partial word
        let query_upper = query.to_uppercase();
        if let Some(and_pos) = query_upper.rfind(" AND ") {
            let after_and = safe_slice_from(query, and_pos + 5);
            let partial = extract_partial_at_end(after_and);
            if partial.is_some() {
                return (CursorContext::AfterLogicalOp(LogicalOp::And), partial);
            }
        }

        if let Some(or_pos) = query_upper.rfind(" OR ") {
            let after_or = safe_slice_from(query, or_pos + 4);
            let partial = extract_partial_at_end(after_or);
            if partial.is_some() {
                return (CursorContext::AfterLogicalOp(LogicalOp::Or), partial);
            }
        }

        if let Some(last_condition) = where_clause.conditions.last() {
            if let Some(connector) = &last_condition.connector {
                // We're after AND/OR
                return (
                    CursorContext::AfterLogicalOp(connector.clone()),
                    extract_partial_at_end(query),
                );
            }
        }
        // We're in WHERE clause but not after AND/OR
        return (CursorContext::WhereClause, extract_partial_at_end(query));
    }

    // Check if we're after ORDER BY
    let query_upper = query.to_uppercase();
    if query_upper.ends_with(" ORDER BY") {
        return (CursorContext::OrderByClause, None);
    }

    // Check other contexts based on what's in the statement
    if stmt.order_by.is_some() {
        return (CursorContext::OrderByClause, extract_partial_at_end(query));
    }

    if stmt.from_table.is_some() && stmt.where_clause.is_none() && stmt.order_by.is_none() {
        return (CursorContext::FromClause, extract_partial_at_end(query));
    }

    if !stmt.columns.is_empty() && stmt.from_table.is_none() {
        return (CursorContext::SelectClause, extract_partial_at_end(query));
    }

    (CursorContext::Unknown, None)
}

/// Helper function to find the last occurrence of a token type in the token stream
fn find_last_token(tokens: &[(usize, usize, Token)], target: &Token) -> Option<usize> {
    tokens
        .iter()
        .rposition(|(_, _, t)| t == target)
        .map(|idx| tokens[idx].0)
}

/// Helper function to find the last occurrence of any matching token
fn find_last_matching_token<F>(
    tokens: &[(usize, usize, Token)],
    predicate: F,
) -> Option<(usize, &Token)>
where
    F: Fn(&Token) -> bool,
{
    tokens
        .iter()
        .rposition(|(_, _, t)| predicate(t))
        .map(|idx| (tokens[idx].0, &tokens[idx].2))
}

/// Helper function to check if we're in a specific clause based on tokens
fn is_in_clause(
    tokens: &[(usize, usize, Token)],
    clause_token: Token,
    exclude_tokens: &[Token],
) -> bool {
    // Find the last occurrence of the clause token
    if let Some(clause_pos) = find_last_token(tokens, &clause_token) {
        // Check if any exclude tokens appear after it
        for (pos, _, token) in tokens.iter() {
            if *pos > clause_pos && exclude_tokens.contains(token) {
                return false;
            }
        }
        return true;
    }
    false
}

fn analyze_partial(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    // Tokenize the query up to cursor position
    let mut lexer = Lexer::new(query);
    let tokens = lexer.tokenize_all_with_positions();

    let trimmed = query.trim();

    #[cfg(test)]
    {
        if trimmed.contains("\"Last Name\"") {
            eprintln!("DEBUG analyze_partial: query='{query}', trimmed='{trimmed}'");
        }
    }

    // Check if we're after a comparison operator (e.g., "createdDate > ")
    if let Some(result) = check_after_comparison_operator(query) {
        return result;
    }

    // Look for the last dot in the query (method call context) - check this FIRST
    // before AND/OR detection to properly handle cases like "AND (Country."
    if let Some(dot_pos) = trimmed.rfind('.') {
        #[cfg(test)]
        {
            if trimmed.contains("\"Last Name\"") {
                eprintln!("DEBUG: Found dot at position {dot_pos}");
            }
        }
        // Check if we're after a column name and dot
        let before_dot = &trimmed[..dot_pos];
        let after_dot = &trimmed[dot_pos + 1..];

        // Check if the part after dot looks like an incomplete method call
        // (not a complete method call like "Contains(...)")
        if !after_dot.contains('(') {
            // Try to extract the column name before the dot
            // It could be a quoted identifier like "Last Name" or a regular identifier
            let col_name = if before_dot.ends_with('"') {
                // Handle quoted identifier - search backwards for matching opening quote
                let bytes = before_dot.as_bytes();
                let pos = before_dot.len() - 1; // Position of closing quote

                #[cfg(test)]
                {
                    if trimmed.contains("\"Last Name\"") {
                        eprintln!("DEBUG: before_dot='{before_dot}', looking for opening quote");
                    }
                }

                let found_start = find_quote_start(bytes, pos);

                if let Some(start) = found_start {
                    // Extract the full quoted identifier including quotes
                    let result = safe_slice_from(before_dot, start);
                    #[cfg(test)]
                    {
                        if trimmed.contains("\"Last Name\"") {
                            eprintln!("DEBUG: Extracted quoted identifier: '{result}'");
                        }
                    }
                    Some(result)
                } else {
                    #[cfg(test)]
                    {
                        if trimmed.contains("\"Last Name\"") {
                            eprintln!("DEBUG: No opening quote found!");
                        }
                    }
                    None
                }
            } else {
                // Regular identifier - get the last word, handling parentheses
                // Strip all leading parentheses
                before_dot
                    .split_whitespace()
                    .last()
                    .map(|word| word.trim_start_matches('('))
            };

            if let Some(col_name) = col_name {
                #[cfg(test)]
                {
                    if trimmed.contains("\"Last Name\"") {
                        eprintln!("DEBUG: col_name = '{col_name}'");
                    }
                }

                // For quoted identifiers, keep the quotes, for regular identifiers check validity
                let is_valid = Parser::is_valid_identifier(col_name);

                #[cfg(test)]
                {
                    if trimmed.contains("\"Last Name\"") {
                        eprintln!("DEBUG: is_valid = {is_valid}");
                    }
                }

                if is_valid {
                    return handle_method_call_context(col_name, after_dot);
                }
            }
        }
    }

    // Check if we're after AND/OR using tokens - but only after checking for method calls
    if let Some((pos, token)) =
        find_last_matching_token(&tokens, |t| matches!(t, Token::And | Token::Or))
    {
        // Check if cursor is after the logical operator
        let token_end_pos = if matches!(token, Token::And) {
            pos + 3 // "AND" is 3 characters
        } else {
            pos + 2 // "OR" is 2 characters
        };

        if cursor_pos > token_end_pos {
            // Extract any partial word after the operator
            let after_op = safe_slice_from(query, token_end_pos + 1); // +1 for the space
            let partial = extract_partial_at_end(after_op);
            let op = if matches!(token, Token::And) {
                LogicalOp::And
            } else {
                LogicalOp::Or
            };
            return (CursorContext::AfterLogicalOp(op), partial);
        }
    }

    // Check if the last token is AND or OR (handles case where it's at the very end)
    if let Some((_, _, last_token)) = tokens.last() {
        if matches!(last_token, Token::And | Token::Or) {
            let op = if matches!(last_token, Token::And) {
                LogicalOp::And
            } else {
                LogicalOp::Or
            };
            return (CursorContext::AfterLogicalOp(op), None);
        }
    }

    // Check if we're in ORDER BY clause using tokens
    if let Some(order_pos) = find_last_token(&tokens, &Token::OrderBy) {
        // Check if there's a BY token after ORDER
        let has_by = tokens
            .iter()
            .any(|(pos, _, t)| *pos > order_pos && matches!(t, Token::By));
        if has_by
            || tokens
                .last()
                .map_or(false, |(_, _, t)| matches!(t, Token::OrderBy))
        {
            return (CursorContext::OrderByClause, extract_partial_at_end(query));
        }
    }

    // Check if we're in WHERE clause using tokens
    if is_in_clause(&tokens, Token::Where, &[Token::OrderBy, Token::GroupBy]) {
        return (CursorContext::WhereClause, extract_partial_at_end(query));
    }

    // Check if we're in FROM clause using tokens
    if is_in_clause(
        &tokens,
        Token::From,
        &[Token::Where, Token::OrderBy, Token::GroupBy],
    ) {
        return (CursorContext::FromClause, extract_partial_at_end(query));
    }

    // Check if we're in SELECT clause using tokens
    if find_last_token(&tokens, &Token::Select).is_some()
        && find_last_token(&tokens, &Token::From).is_none()
    {
        return (CursorContext::SelectClause, extract_partial_at_end(query));
    }

    (CursorContext::Unknown, None)
}

fn extract_partial_at_end(query: &str) -> Option<String> {
    let trimmed = query.trim();

    // First check if the last word itself starts with a quote (unclosed quoted identifier being typed)
    if let Some(last_word) = trimmed.split_whitespace().last() {
        if last_word.starts_with('"') && !last_word.ends_with('"') {
            // This is an unclosed quoted identifier like "Cust
            return Some(last_word.to_string());
        }
    }

    // Regular identifier extraction
    let last_word = trimmed.split_whitespace().last()?;

    // Check if it's a partial identifier (not a keyword or operator)
    // First check if it's alphanumeric (potential identifier)
    if last_word.chars().all(|c| c.is_alphanumeric() || c == '_') {
        // Use lexer to determine if it's a keyword or identifier
        if !is_sql_keyword(last_word) {
            Some(last_word.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

// Implement the ParsePrimary trait for Parser to use the modular expression parsing
impl ParsePrimary for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        self.advance();
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.consume(expected)
    }

    fn parse_case_expression(&mut self) -> Result<SqlExpression, String> {
        self.parse_case_expression()
    }

    fn parse_function_args(&mut self) -> Result<(Vec<SqlExpression>, bool), String> {
        self.parse_function_args()
    }

    fn parse_window_spec(&mut self) -> Result<WindowSpec, String> {
        self.parse_window_spec()
    }

    fn parse_logical_or(&mut self) -> Result<SqlExpression, String> {
        self.parse_logical_or()
    }

    fn parse_comparison(&mut self) -> Result<SqlExpression, String> {
        self.parse_comparison()
    }

    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String> {
        self.parse_expression_list()
    }

    fn parse_subquery(&mut self) -> Result<SelectStatement, String> {
        // Parse subquery without parenthesis balance validation
        if matches!(self.current_token, Token::With) {
            self.parse_with_clause_inner()
        } else {
            self.parse_select_statement_inner()
        }
    }
}

// Implement the ExpressionParser trait for Parser to use the modular expression parsing
impl ExpressionParser for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        // Call the main advance method directly to avoid recursion
        match &self.current_token {
            Token::LeftParen => self.paren_depth += 1,
            Token::RightParen => {
                self.paren_depth -= 1;
            }
            _ => {}
        }
        self.current_token = self.lexer.next_token();
    }

    fn peek(&self) -> Option<&Token> {
        // We can't return a reference to a token from a temporary lexer,
        // so we need a different approach. For now, let's use a workaround
        // that checks the next token type without consuming it.
        // This is a limitation of the current design.
        // A proper fix would be to store the peeked token in the Parser struct.
        None // TODO: Implement proper lookahead
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        // Call the main consume method to avoid recursion
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.update_paren_depth(&expected)?;
            self.current_token = self.lexer.next_token();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token
            ))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        if let Token::Identifier(id) = &self.current_token {
            let id = id.clone();
            self.advance();
            Ok(id)
        } else {
            Err(format!(
                "Expected identifier, found {:?}",
                self.current_token
            ))
        }
    }
}

// Implement the ParseArithmetic trait for Parser to use the modular arithmetic parsing
impl ParseArithmetic for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        self.advance();
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.consume(expected)
    }

    fn parse_primary(&mut self) -> Result<SqlExpression, String> {
        self.parse_primary()
    }

    fn parse_multiplicative(&mut self) -> Result<SqlExpression, String> {
        self.parse_multiplicative()
    }

    fn parse_method_args(&mut self) -> Result<Vec<SqlExpression>, String> {
        self.parse_method_args()
    }
}

// Implement the ParseComparison trait for Parser to use the modular comparison parsing
impl ParseComparison for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        self.advance();
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.consume(expected)
    }

    fn parse_primary(&mut self) -> Result<SqlExpression, String> {
        self.parse_primary()
    }

    fn parse_additive(&mut self) -> Result<SqlExpression, String> {
        self.parse_additive()
    }

    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String> {
        self.parse_expression_list()
    }

    fn parse_subquery(&mut self) -> Result<SelectStatement, String> {
        // Parse subquery without parenthesis balance validation
        if matches!(self.current_token, Token::With) {
            self.parse_with_clause_inner()
        } else {
            self.parse_select_statement_inner()
        }
    }
}

// Implement the ParseLogical trait for Parser to use the modular logical parsing
impl ParseLogical for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        self.advance();
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.consume(expected)
    }

    fn parse_logical_and(&mut self) -> Result<SqlExpression, String> {
        self.parse_logical_and()
    }

    fn parse_base_logical_expression(&mut self) -> Result<SqlExpression, String> {
        // This is the base for logical AND - it should parse comparison expressions
        // to avoid infinite recursion with parse_expression
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<SqlExpression, String> {
        self.parse_comparison()
    }

    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String> {
        self.parse_expression_list()
    }
}

// Implement the ParseCase trait for Parser to use the modular CASE parsing
impl ParseCase for Parser {
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn advance(&mut self) {
        self.advance();
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        self.consume(expected)
    }

    fn parse_expression(&mut self) -> Result<SqlExpression, String> {
        self.parse_expression()
    }
}

fn is_sql_keyword(word: &str) -> bool {
    // Use the lexer to check if this word produces a keyword token
    let mut lexer = Lexer::new(word);
    let token = lexer.next_token();

    // Check if it's a keyword token (not an identifier)
    !matches!(token, Token::Identifier(_) | Token::Eof)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that Parser::new() defaults to Standard mode (backward compatible)
    #[test]
    fn test_parser_mode_default_is_standard() {
        let sql = "-- Leading comment\nSELECT * FROM users";
        let mut parser = Parser::new(sql);
        let stmt = parser.parse().unwrap();

        // In Standard mode, comments should be empty
        assert!(stmt.leading_comments.is_empty());
        assert!(stmt.trailing_comment.is_none());
    }

    /// Test that PreserveComments mode collects leading comments
    #[test]
    fn test_parser_mode_preserve_leading_comments() {
        let sql = "-- Important query\n-- Author: Alice\nSELECT id, name FROM users";
        let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
        let stmt = parser.parse().unwrap();

        // Should have 2 leading comments
        assert_eq!(stmt.leading_comments.len(), 2);
        assert!(stmt.leading_comments[0].is_line_comment);
        assert!(stmt.leading_comments[0].text.contains("Important query"));
        assert!(stmt.leading_comments[1].text.contains("Author: Alice"));
    }

    /// Test that PreserveComments mode collects trailing comments
    #[test]
    fn test_parser_mode_preserve_trailing_comment() {
        let sql = "SELECT * FROM users -- Fetch all users";
        let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
        let stmt = parser.parse().unwrap();

        // Should have trailing comment
        assert!(stmt.trailing_comment.is_some());
        let comment = stmt.trailing_comment.unwrap();
        assert!(comment.is_line_comment);
        assert!(comment.text.contains("Fetch all users"));
    }

    /// Test that PreserveComments mode handles block comments
    #[test]
    fn test_parser_mode_preserve_block_comments() {
        let sql = "/* Query explanation */\nSELECT * FROM users";
        let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
        let stmt = parser.parse().unwrap();

        // Should have leading block comment
        assert_eq!(stmt.leading_comments.len(), 1);
        assert!(!stmt.leading_comments[0].is_line_comment); // It's a block comment
        assert!(stmt.leading_comments[0].text.contains("Query explanation"));
    }

    /// Test that PreserveComments mode collects both leading and trailing
    #[test]
    fn test_parser_mode_preserve_both_comments() {
        let sql = "-- Leading\nSELECT * FROM users -- Trailing";
        let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
        let stmt = parser.parse().unwrap();

        // Should have both
        assert_eq!(stmt.leading_comments.len(), 1);
        assert!(stmt.leading_comments[0].text.contains("Leading"));
        assert!(stmt.trailing_comment.is_some());
        assert!(stmt.trailing_comment.unwrap().text.contains("Trailing"));
    }

    /// Test that Standard mode has zero performance overhead (no comment parsing)
    #[test]
    fn test_parser_mode_standard_ignores_comments() {
        let sql = "-- Comment 1\n/* Comment 2 */\nSELECT * FROM users -- Comment 3";
        let mut parser = Parser::with_mode(sql, ParserMode::Standard);
        let stmt = parser.parse().unwrap();

        // Comments should be completely ignored
        assert!(stmt.leading_comments.is_empty());
        assert!(stmt.trailing_comment.is_none());

        // But query should still parse correctly
        assert_eq!(stmt.select_items.len(), 1);
        assert_eq!(stmt.from_table, Some("users".to_string()));
    }

    /// Test backward compatibility - existing code using Parser::new() unchanged
    #[test]
    fn test_parser_backward_compatibility() {
        let sql = "SELECT id, name FROM users WHERE active = true";

        // Old way (still works, defaults to Standard mode)
        let mut parser1 = Parser::new(sql);
        let stmt1 = parser1.parse().unwrap();

        // Explicit Standard mode (same behavior)
        let mut parser2 = Parser::with_mode(sql, ParserMode::Standard);
        let stmt2 = parser2.parse().unwrap();

        // Both should produce identical ASTs (comments are empty in both)
        assert_eq!(stmt1.select_items.len(), stmt2.select_items.len());
        assert_eq!(stmt1.from_table, stmt2.from_table);
        assert_eq!(stmt1.where_clause.is_some(), stmt2.where_clause.is_some());
        assert!(stmt1.leading_comments.is_empty());
        assert!(stmt2.leading_comments.is_empty());
    }
}
