// Keep chrono imports for the parser implementation

// Re-exports for backward compatibility - these serve as both imports and re-exports
pub use super::parser::ast::{
    CTEType, Condition, DataFormat, FrameBound, FrameUnit, JoinClause, JoinCondition, JoinOperator,
    JoinType, LogicalOp, OrderByColumn, SelectItem, SelectStatement, SortDirection, SqlExpression,
    TableFunction, TableSource, WebCTESpec, WhenBranch, WhereClause, WindowFrame, WindowSpec, CTE,
};
pub use super::parser::legacy::{ParseContext, ParseState, Schema, SqlParser, SqlToken, TableInfo};
pub use super::parser::lexer::{Lexer, Token};
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
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    in_method_args: bool, // Track if we're parsing method arguments
    columns: Vec<String>, // Known column names for context-aware parsing
    paren_depth: i32,     // Track parentheses nesting depth
    #[allow(dead_code)]
    config: ParserConfig, // Parser configuration including case sensitivity
}

impl Parser {
    #[must_use]
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            in_method_args: false,
            columns: Vec::new(),
            paren_depth: 0,
            config: ParserConfig::default(),
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
            config,
        }
    }

    #[must_use]
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
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

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            // Track parentheses depth
            match &expected {
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

    fn advance(&mut self) {
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
        self.current_token = self.lexer.next_token();
    }

    pub fn parse(&mut self) -> Result<SelectStatement, String> {
        // Check for WITH clause at the beginning
        if matches!(self.current_token, Token::With) {
            self.parse_with_clause()
        } else {
            self.parse_select_statement()
        }
    }

    fn parse_with_clause(&mut self) -> Result<SelectStatement, String> {
        self.consume(Token::With)?;

        let mut ctes = Vec::new();

        // Parse CTEs
        loop {
            // Check for WEB keyword for each CTE (can be different for each one)
            let is_web = if let Token::Identifier(id) = &self.current_token {
                if id.to_uppercase() == "WEB" {
                    self.advance();
                    true
                } else {
                    false
                }
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

            // Expect opening parenthesis
            self.consume(Token::LeftParen)?;

            let cte_type = if is_web {
                // Parse WEB CTE specification
                let web_spec = self.parse_web_cte_spec()?;
                CTEType::Web(web_spec)
            } else {
                // Parse the CTE query (inner version that doesn't check parentheses)
                let query = self.parse_select_statement_inner()?;
                CTEType::Standard(query)
            };

            // Expect closing parenthesis
            self.consume(Token::RightParen)?;

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

        // Parse the main SELECT statement (with parenthesis checking)
        let mut main_query = self.parse_select_statement()?;
        main_query.ctes = ctes;

        Ok(main_query)
    }

    fn parse_web_cte_spec(&mut self) -> Result<WebCTESpec, String> {
        // Expect URL keyword
        if let Token::Identifier(id) = &self.current_token {
            if id.to_uppercase() != "URL" {
                return Err("Expected URL keyword in WEB CTE".to_string());
            }
        } else {
            return Err("Expected URL keyword in WEB CTE".to_string());
        }
        self.advance();

        // Parse URL string
        let url = match &self.current_token {
            Token::StringLiteral(url) => url.clone(),
            _ => return Err("Expected URL string after URL keyword".to_string()),
        };
        self.advance();

        // Parse optional clauses
        let mut format = None;
        let mut headers = Vec::new();
        let mut cache_seconds = None;

        // Parse optional clauses until we hit the closing parenthesis
        while !matches!(self.current_token, Token::RightParen)
            && !matches!(self.current_token, Token::Eof)
        {
            if let Token::Identifier(id) = &self.current_token {
                match id.to_uppercase().as_str() {
                    "FORMAT" => {
                        self.advance();
                        format = Some(self.parse_data_format()?);
                    }
                    "CACHE" => {
                        self.advance();
                        cache_seconds = Some(self.parse_cache_duration()?);
                    }
                    "HEADERS" => {
                        self.advance();
                        headers = self.parse_headers()?;
                    }
                    _ => {
                        return Err(format!(
                            "Unexpected keyword '{}' in WEB CTE specification",
                            id
                        ));
                    }
                }
            } else {
                break;
            }
        }

        Ok(WebCTESpec {
            url,
            format,
            headers,
            cache_seconds,
        })
    }

    fn parse_data_format(&mut self) -> Result<DataFormat, String> {
        if let Token::Identifier(id) = &self.current_token {
            let format = match id.to_uppercase().as_str() {
                "CSV" => DataFormat::CSV,
                "JSON" => DataFormat::JSON,
                "AUTO" => DataFormat::Auto,
                _ => return Err(format!("Unknown data format: {}", id)),
            };
            self.advance();
            Ok(format)
        } else {
            Err("Expected data format (CSV, JSON, or AUTO)".to_string())
        }
    }

    fn parse_cache_duration(&mut self) -> Result<u64, String> {
        match &self.current_token {
            Token::NumberLiteral(n) => {
                let duration = n
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid cache duration: {}", n))?;
                self.advance();
                Ok(duration)
            }
            _ => Err("Expected number for cache duration".to_string()),
        }
    }

    fn parse_headers(&mut self) -> Result<Vec<(String, String)>, String> {
        self.consume(Token::LeftParen)?;

        let mut headers = Vec::new();

        loop {
            // Parse header name
            let key = match &self.current_token {
                Token::Identifier(id) => id.clone(),
                Token::StringLiteral(s) => s.clone(),
                _ => return Err("Expected header name".to_string()),
            };
            self.advance();

            // Expect : (colon) for header key-value separator
            if !matches!(self.current_token, Token::Colon) {
                // For backwards compatibility, also accept =
                if matches!(self.current_token, Token::Equal) {
                    self.advance();
                } else {
                    return Err("Expected ':' or '=' after header name".to_string());
                }
            } else {
                self.advance(); // consume the colon
            }

            // Parse header value
            let value = match &self.current_token {
                Token::StringLiteral(s) => s.clone(),
                _ => return Err("Expected header value as string".to_string()),
            };
            self.advance();

            headers.push((key, value));

            // Check for more headers
            if !matches!(self.current_token, Token::Comma) {
                break;
            }
            self.advance();
        }

        self.consume(Token::RightParen)?;
        Ok(headers)
    }

    fn parse_with_clause_inner(&mut self) -> Result<SelectStatement, String> {
        self.consume(Token::With)?;

        let mut ctes = Vec::new();

        // Parse CTEs
        loop {
            // Check for WEB keyword for each CTE (can be different for each one)
            let is_web = if let Token::Identifier(id) = &self.current_token {
                if id.to_uppercase() == "WEB" {
                    self.advance();
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Parse CTE name
            let name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err("Expected CTE name after WITH or comma".to_string()),
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

            // Expect opening parenthesis
            self.consume(Token::LeftParen)?;

            let cte_type = if is_web {
                // Parse WEB CTE specification
                let web_spec = self.parse_web_cte_spec()?;
                CTEType::Web(web_spec)
            } else {
                // Parse the CTE query (inner version that doesn't check parentheses)
                let query = self.parse_select_statement_inner()?;
                CTEType::Standard(query)
            };

            // Expect closing parenthesis
            self.consume(Token::RightParen)?;

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

        // Parse the main SELECT statement (without parenthesis checking for subqueries)
        let mut main_query = self.parse_select_statement_inner()?;
        main_query.ctes = ctes;

        Ok(main_query)
    }

    fn parse_select_statement(&mut self) -> Result<SelectStatement, String> {
        let result = self.parse_select_statement_inner()?;

        // Check for balanced parentheses at the end of parsing
        if self.paren_depth > 0 {
            return Err(format!(
                "Unclosed parenthesis - missing {} closing parenthes{}",
                self.paren_depth,
                if self.paren_depth == 1 { "is" } else { "es" }
            ));
        } else if self.paren_depth < 0 {
            return Err(
                "Extra closing parenthesis found - no matching opening parenthesis".to_string(),
            );
        }

        Ok(result)
    }

    fn parse_select_statement_inner(&mut self) -> Result<SelectStatement, String> {
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
                SelectItem::Star => "*".to_string(),
                SelectItem::Column(col_ref) => col_ref.name.clone(),
                SelectItem::Expression { alias, .. } => alias.clone(),
            })
            .collect();

        // Parse FROM clause - can be a table name, subquery, or table function
        let (from_table, from_subquery, from_function, from_alias) =
            if matches!(self.current_token, Token::From) {
                self.advance();

                // Check for table function like RANGE()
                if let Token::Identifier(name) = &self.current_token.clone() {
                    if name.to_uppercase() == "RANGE" {
                        self.advance();
                        // Parse RANGE function
                        self.consume(Token::LeftParen)?;

                        // Parse start expression
                        let start = self.parse_expression()?;
                        self.consume(Token::Comma)?;

                        // Parse end expression
                        let end = self.parse_expression()?;

                        // Parse optional step
                        let step = if matches!(self.current_token, Token::Comma) {
                            self.advance();
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };

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
                                _ => return Err("Expected alias name after AS".to_string()),
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
                            Some(TableFunction::Range { start, end, step }),
                            alias,
                        )
                    } else if name.to_uppercase() == "SPLIT" {
                        // Parse SPLIT(text[, delimiter])
                        self.advance(); // Skip "SPLIT"
                        self.consume(Token::LeftParen)?;

                        let text = self.parse_expression()?;

                        let delimiter = if matches!(self.current_token, Token::Comma) {
                            self.advance();
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };

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
                                _ => return Err("Expected alias name after AS".to_string()),
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
                            Some(TableFunction::Split { text, delimiter }),
                            alias,
                        )
                    } else if name.to_uppercase().starts_with("GENERATE_")
                        || name.to_uppercase().starts_with("RANDOM_")
                        || name.to_uppercase() == "FIBONACCI"
                        || name.to_uppercase() == "PRIME_FACTORS"
                        || name.to_uppercase() == "COLLATZ"
                        || name.to_uppercase() == "PASCAL_TRIANGLE"
                        || name.to_uppercase() == "TRIANGULAR"
                        || name.to_uppercase() == "SQUARES"
                        || name.to_uppercase() == "FACTORIALS"
                    {
                        // Parse generator function
                        let generator_name = name.clone();
                        self.advance(); // Skip generator name

                        // Parse arguments
                        self.consume(Token::LeftParen)?;
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
                                _ => return Err("Expected alias name after AS".to_string()),
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
                                name: generator_name,
                                args,
                            }),
                            alias,
                        )
                    } else {
                        // Not a RANGE, SPLIT, or generator function, so it's a regular table name
                        let table_name = name.clone();
                        self.advance();

                        // Check for optional alias
                        let alias = if matches!(self.current_token, Token::As) {
                            self.advance();
                            match &self.current_token {
                                Token::Identifier(name) => {
                                    let alias = name.clone();
                                    self.advance();
                                    Some(alias)
                                }
                                _ => return Err("Expected alias name after AS".to_string()),
                            }
                        } else if let Token::Identifier(name) = &self.current_token {
                            // AS is optional for table aliases
                            let alias = name.clone();
                            self.advance();
                            Some(alias)
                        } else {
                            None
                        };

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
                            _ => return Err("Expected alias name after AS".to_string()),
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
                            let alias = if matches!(self.current_token, Token::As) {
                                self.advance();
                                match &self.current_token {
                                    Token::Identifier(name) => {
                                        let alias = name.clone();
                                        self.advance();
                                        Some(alias)
                                    }
                                    _ => return Err("Expected alias name after AS".to_string()),
                                }
                            } else if let Token::Identifier(name) = &self.current_token {
                                // AS is optional for table aliases
                                let alias = name.clone();
                                self.advance();
                                Some(alias)
                            } else {
                                None
                            };

                            (Some(table_name), None, None, alias)
                        }
                        Token::QuotedIdentifier(table) => {
                            // Handle quoted table names
                            let table_name = table.clone();
                            self.advance();

                            // Check for optional alias
                            let alias = if matches!(self.current_token, Token::As) {
                                self.advance();
                                match &self.current_token {
                                    Token::Identifier(name) => {
                                        let alias = name.clone();
                                        self.advance();
                                        Some(alias)
                                    }
                                    _ => return Err("Expected alias name after AS".to_string()),
                                }
                            } else if let Token::Identifier(name) = &self.current_token {
                                // AS is optional for table aliases
                                let alias = name.clone();
                                self.advance();
                                Some(alias)
                            } else {
                                None
                            };

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
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse ORDER BY clause (comes after GROUP BY and HAVING)
        let order_by = if matches!(self.current_token, Token::OrderBy) {
            self.advance();
            Some(self.parse_order_by_list()?)
        } else if let Token::Identifier(s) = &self.current_token {
            if s.to_uppercase() == "ORDER" {
                // Handle ORDER BY as two separate tokens
                self.advance(); // consume ORDER
                if matches!(&self.current_token, Token::Identifier(by_token) if by_token.to_uppercase() == "BY")
                {
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
        })
    }

    fn parse_select_list(&mut self) -> Result<Vec<String>, String> {
        let mut columns = Vec::new();

        if matches!(self.current_token, Token::Star) {
            columns.push("*".to_string());
            self.advance();
        } else {
            loop {
                match &self.current_token {
                    Token::Identifier(col) => {
                        columns.push(col.clone());
                        self.advance();
                    }
                    Token::QuotedIdentifier(col) => {
                        // Handle quoted column names like "Customer Id"
                        columns.push(col.clone());
                        self.advance();
                    }
                    _ => return Err("Expected column name".to_string()),
                }

                if matches!(self.current_token, Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(columns)
    }

    /// Parse SELECT items that support computed expressions with aliases
    fn parse_select_items(&mut self) -> Result<Vec<SelectItem>, String> {
        let mut items = Vec::new();

        loop {
            // Check for * only at the beginning of a select item
            // After a comma, * could be either SELECT * or part of multiplication
            if matches!(self.current_token, Token::Star) {
                // Determine if this is SELECT * or multiplication
                // SELECT * is only valid:
                // 1. As the first item in SELECT
                // 2. Right after a comma (but not if followed by something that makes it multiplication)

                // For now, treat Star as SELECT * only if we're at the start or just after a comma
                // and the star is not immediately followed by something that would make it multiplication
                items.push(SelectItem::Star);
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
                        _ => return Err("Expected alias name after AS".to_string()),
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
                        SelectItem::Column(col_ref)
                    }
                    _ => {
                        // Computed expression or column with different alias
                        SelectItem::Expression { expr, alias }
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
                    let id_upper = id.to_uppercase();
                    if matches!(
                        id_upper.as_str(),
                        "ORDER" | "HAVING" | "LIMIT" | "OFFSET" | "UNION" | "INTERSECT" | "EXCEPT"
                    ) {
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
            if s.to_uppercase() == "ORDER" {
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
                    col
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

    fn parse_window_frame(&mut self) -> Result<Option<WindowFrame>, String> {
        // Check for ROWS or RANGE keyword
        let unit = match &self.current_token {
            Token::Identifier(id) if id.to_uppercase() == "ROWS" => {
                self.advance();
                FrameUnit::Rows
            }
            Token::Identifier(id) if id.to_uppercase() == "RANGE" => {
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
            Token::Identifier(id) if id.to_uppercase() == "UNBOUNDED" => {
                self.advance();
                match &self.current_token {
                    Token::Identifier(id) if id.to_uppercase() == "PRECEDING" => {
                        self.advance();
                        Ok(FrameBound::UnboundedPreceding)
                    }
                    Token::Identifier(id) if id.to_uppercase() == "FOLLOWING" => {
                        self.advance();
                        Ok(FrameBound::UnboundedFollowing)
                    }
                    _ => Err("Expected PRECEDING or FOLLOWING after UNBOUNDED".to_string()),
                }
            }
            Token::Identifier(id) if id.to_uppercase() == "CURRENT" => {
                self.advance();
                if let Token::Identifier(id) = &self.current_token {
                    if id.to_uppercase() == "ROW" {
                        self.advance();
                        return Ok(FrameBound::CurrentRow);
                    }
                }
                Err("Expected ROW after CURRENT".to_string())
            }
            Token::NumberLiteral(num) => {
                let n: i64 = num
                    .parse()
                    .map_err(|_| "Invalid number in window frame".to_string())?;
                self.advance();
                match &self.current_token {
                    Token::Identifier(id) if id.to_uppercase() == "PRECEDING" => {
                        self.advance();
                        Ok(FrameBound::Preceding(n))
                    }
                    Token::Identifier(id) if id.to_uppercase() == "FOLLOWING" => {
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
        // Start with logical OR as the lowest precedence operator
        // The hierarchy is: OR -> AND -> comparison -> additive -> multiplicative -> primary
        let mut left = self.parse_logical_or()?;

        // Handle IN operator (not preceded by NOT)
        // This uses the modular comparison module
        left = parse_in_operator(self, left)?;

        Ok(left)
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
        let mut args = Vec::new();

        // Set flag to indicate we're parsing method arguments
        self.in_method_args = true;

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

    fn get_binary_op(&self) -> Option<String> {
        match &self.current_token {
            Token::Equal => Some("=".to_string()),
            Token::NotEqual => Some("!=".to_string()),
            Token::LessThan => Some("<".to_string()),
            Token::GreaterThan => Some(">".to_string()),
            Token::LessThanOrEqual => Some("<=".to_string()),
            Token::GreaterThanOrEqual => Some(">=".to_string()),
            Token::Like => Some("LIKE".to_string()),
            _ => None,
        }
    }

    fn get_arithmetic_op(&self) -> Option<String> {
        match &self.current_token {
            Token::Plus => Some("+".to_string()),
            Token::Minus => Some("-".to_string()),
            Token::Star => Some("*".to_string()), // Multiplication (context-sensitive)
            Token::Divide => Some("/".to_string()),
            Token::Modulo => Some("%".to_string()),
            _ => None,
        }
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
            // CROSS JOIN doesn't have ON condition
            JoinCondition {
                left_column: String::new(),
                operator: JoinOperator::Equal,
                right_column: String::new(),
            }
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
        // Parse left column (can include table prefix)
        let left_column = self.parse_column_reference()?;

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

        // Parse right column (can include table prefix)
        let right_column = self.parse_column_reference()?;

        Ok(JoinCondition {
            left_column,
            operator,
            right_column,
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
fn analyze_statement(
    stmt: &SelectStatement,
    query: &str,
    _cursor_pos: usize,
) -> (CursorContext, Option<String>) {
    // First check for method call context (e.g., "columnName." or "columnName.Con")
    let trimmed = query.trim();

    // Check if we're after a comparison operator (e.g., "createdDate > ")
    let comparison_ops = [" > ", " < ", " >= ", " <= ", " = ", " != "];
    for op in &comparison_ops {
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
                        return (
                            CursorContext::AfterComparisonOp(
                                col_name.to_string(),
                                op.trim().to_string(),
                            ),
                            partial,
                        );
                    }
                }
            }
        }
    }

    // First check if we're after AND/OR - this takes precedence
    if trimmed.to_uppercase().ends_with(" AND")
        || trimmed.to_uppercase().ends_with(" OR")
        || trimmed.to_uppercase().ends_with(" AND ")
        || trimmed.to_uppercase().ends_with(" OR ")
    {
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
                    let mut pos = before_dot.len() - 1; // Position of closing quote
                    let mut found_start = None;

                    // Skip the closing quote and search backwards
                    if pos > 0 {
                        pos -= 1;
                        while pos > 0 {
                            if bytes[pos] == b'"' {
                                // Check if it's not an escaped quote
                                if pos == 0 || bytes[pos - 1] != b'\\' {
                                    found_start = Some(pos);
                                    break;
                                }
                            }
                            pos -= 1;
                        }
                        // Check position 0 separately
                        if found_start.is_none() && bytes[0] == b'"' {
                            found_start = Some(0);
                        }
                    }

                    found_start.map(|start| safe_slice_from(before_dot, start))
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
                    let is_valid = if col_name.starts_with('"') && col_name.ends_with('"') {
                        // Quoted identifier - always valid
                        true
                    } else {
                        // Regular identifier - check if it's alphanumeric or underscore
                        col_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    };

                    if is_valid {
                        // We're in a method call context
                        // Check if there's a partial method name after the dot
                        let partial_method = if after_dot.is_empty() {
                            None
                        } else if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            Some(after_dot.to_string())
                        } else {
                            None
                        };

                        // For AfterColumn context, strip quotes if present for consistency
                        let col_name_for_context = if col_name.starts_with('"')
                            && col_name.ends_with('"')
                            && col_name.len() > 2
                        {
                            col_name[1..col_name.len() - 1].to_string()
                        } else {
                            col_name.to_string()
                        };

                        return (
                            CursorContext::AfterColumn(col_name_for_context),
                            partial_method,
                        );
                    }
                }
            }
        }
    }

    // Check if we're in WHERE clause
    if let Some(where_clause) = &stmt.where_clause {
        // Check if query ends with AND/OR (with or without trailing space/partial)
        if trimmed.to_uppercase().ends_with(" AND") || trimmed.to_uppercase().ends_with(" OR") {
            let op = if trimmed.to_uppercase().ends_with(" AND") {
                LogicalOp::And
            } else {
                LogicalOp::Or
            };
            return (CursorContext::AfterLogicalOp(op), None);
        }

        // Check if we have AND/OR followed by a partial word
        if let Some(and_pos) = query.to_uppercase().rfind(" AND ") {
            let after_and = safe_slice_from(query, and_pos + 5);
            let partial = extract_partial_at_end(after_and);
            if partial.is_some() {
                return (CursorContext::AfterLogicalOp(LogicalOp::And), partial);
            }
        }

        if let Some(or_pos) = query.to_uppercase().rfind(" OR ") {
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
    if query.to_uppercase().ends_with(" ORDER BY ") || query.to_uppercase().ends_with(" ORDER BY") {
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

fn analyze_partial(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    let upper = query.to_uppercase();

    // Check for method call context first (e.g., "columnName." or "columnName.Con")
    let trimmed = query.trim();

    #[cfg(test)]
    {
        if trimmed.contains("\"Last Name\"") {
            eprintln!("DEBUG analyze_partial: query='{query}', trimmed='{trimmed}'");
        }
    }

    // Check if we're after a comparison operator (e.g., "createdDate > ")
    let comparison_ops = [" > ", " < ", " >= ", " <= ", " = ", " != "];
    for op in &comparison_ops {
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
                    // Check if we're at or near the end of the query (allowing for some whitespace)
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
                        return (
                            CursorContext::AfterComparisonOp(
                                col_name.to_string(),
                                op.trim().to_string(),
                            ),
                            partial,
                        );
                    }
                }
            }
        }
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
                let mut pos = before_dot.len() - 1; // Position of closing quote
                let mut found_start = None;

                #[cfg(test)]
                {
                    if trimmed.contains("\"Last Name\"") {
                        eprintln!("DEBUG: before_dot='{before_dot}', looking for opening quote");
                    }
                }

                // Skip the closing quote and search backwards
                if pos > 0 {
                    pos -= 1;
                    while pos > 0 {
                        if bytes[pos] == b'"' {
                            // Check if it's not an escaped quote
                            if pos == 0 || bytes[pos - 1] != b'\\' {
                                found_start = Some(pos);
                                break;
                            }
                        }
                        pos -= 1;
                    }
                    // Check position 0 separately
                    if found_start.is_none() && bytes[0] == b'"' {
                        found_start = Some(0);
                    }
                }

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
                let is_valid = if col_name.starts_with('"') && col_name.ends_with('"') {
                    // Quoted identifier - always valid
                    true
                } else {
                    // Regular identifier - check if it's alphanumeric or underscore
                    col_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                };

                #[cfg(test)]
                {
                    if trimmed.contains("\"Last Name\"") {
                        eprintln!("DEBUG: is_valid = {is_valid}");
                    }
                }

                if is_valid {
                    // We're in a method call context
                    // Check if there's a partial method name after the dot
                    let partial_method = if after_dot.is_empty() {
                        None
                    } else if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        Some(after_dot.to_string())
                    } else {
                        None
                    };

                    // For AfterColumn context, strip quotes if present for consistency
                    let col_name_for_context = if col_name.starts_with('"')
                        && col_name.ends_with('"')
                        && col_name.len() > 2
                    {
                        col_name[1..col_name.len() - 1].to_string()
                    } else {
                        col_name.to_string()
                    };

                    return (
                        CursorContext::AfterColumn(col_name_for_context),
                        partial_method,
                    );
                }
            }
        }
    }

    // Check if we're after AND/OR - but only after checking for method calls
    if let Some(and_pos) = upper.rfind(" AND ") {
        // Check if cursor is after AND
        if cursor_pos >= and_pos + 5 {
            // Extract any partial word after AND
            let after_and = safe_slice_from(query, and_pos + 5);
            let partial = extract_partial_at_end(after_and);
            return (CursorContext::AfterLogicalOp(LogicalOp::And), partial);
        }
    }

    if let Some(or_pos) = upper.rfind(" OR ") {
        // Check if cursor is after OR
        if cursor_pos >= or_pos + 4 {
            // Extract any partial word after OR
            let after_or = safe_slice_from(query, or_pos + 4);
            let partial = extract_partial_at_end(after_or);
            return (CursorContext::AfterLogicalOp(LogicalOp::Or), partial);
        }
    }

    // Handle case where AND/OR is at the very end
    if trimmed.to_uppercase().ends_with(" AND") || trimmed.to_uppercase().ends_with(" OR") {
        let op = if trimmed.to_uppercase().ends_with(" AND") {
            LogicalOp::And
        } else {
            LogicalOp::Or
        };
        return (CursorContext::AfterLogicalOp(op), None);
    }

    // Check if we're after ORDER BY
    if upper.ends_with(" ORDER BY ") || upper.ends_with(" ORDER BY") || upper.contains("ORDER BY ")
    {
        return (CursorContext::OrderByClause, extract_partial_at_end(query));
    }

    if upper.contains("WHERE") && !upper.contains("ORDER") && !upper.contains("GROUP") {
        return (CursorContext::WhereClause, extract_partial_at_end(query));
    }

    if upper.contains("FROM") && !upper.contains("WHERE") && !upper.contains("ORDER") {
        return (CursorContext::FromClause, extract_partial_at_end(query));
    }

    if upper.contains("SELECT") && !upper.contains("FROM") {
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
    if last_word.chars().all(|c| c.is_alphanumeric() || c == '_') && !is_sql_keyword(last_word) {
        Some(last_word.to_string())
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
            match &expected {
                Token::LeftParen => self.paren_depth += 1,
                Token::RightParen => {
                    self.paren_depth -= 1;
                    if self.paren_depth < 0 {
                        return Err(
                            "Unexpected closing parenthesis - no matching opening parenthesis"
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
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
    matches!(
        word.to_uppercase().as_str(),
        "SELECT"
            | "FROM"
            | "WHERE"
            | "AND"
            | "OR"
            | "IN"
            | "ORDER"
            | "BY"
            | "GROUP"
            | "HAVING"
            | "ASC"
            | "DESC"
            | "DISTINCT"
    )
}
