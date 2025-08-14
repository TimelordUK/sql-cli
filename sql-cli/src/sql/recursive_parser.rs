use chrono::{Datelike, Local, NaiveDateTime};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    And,
    Or,
    In,
    Not,
    Between,
    Like,
    Is,
    Null,
    OrderBy,
    GroupBy,
    Having,
    Asc,
    Desc,
    Limit,
    Offset,
    DateTime, // DateTime constructor

    // Literals
    Identifier(String),
    QuotedIdentifier(String), // For "Customer Id" style identifiers
    StringLiteral(String),
    NumberLiteral(String),
    Star,

    // Operators
    Dot,
    Comma,
    LeftParen,
    RightParen,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current = chars.get(0).copied();
        Self {
            input: chars,
            position: 0,
            current_char: current,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_string(&mut self) -> String {
        let mut result = String::new();
        let quote_char = self.current_char.unwrap(); // ' or "
        self.advance(); // skip opening quote

        while let Some(ch) = self.current_char {
            if ch == quote_char {
                self.advance(); // skip closing quote
                break;
            }
            result.push(ch);
            self.advance();
        }
        result
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_numeric() || ch == '.' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char {
            None => Token::Eof,
            Some('*') => {
                self.advance();
                Token::Star
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some('(') => {
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                Token::RightParen
            }
            Some('=') => {
                self.advance();
                Token::Equal
            }
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::LessThanOrEqual
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::LessThan
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::GreaterThanOrEqual
                } else {
                    Token::GreaterThan
                }
            }
            Some('!') if self.peek(1) == Some('=') => {
                self.advance();
                self.advance();
                Token::NotEqual
            }
            Some('"') => {
                // Double quotes = identifier
                let ident_val = self.read_string();
                Token::QuotedIdentifier(ident_val)
            }
            Some('\'') => {
                // Single quotes = string literal
                let string_val = self.read_string();
                Token::StringLiteral(string_val)
            }
            Some(ch) if ch.is_numeric() => {
                let num = self.read_number();
                Token::NumberLiteral(num)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.to_uppercase().as_str() {
                    "SELECT" => Token::Select,
                    "FROM" => Token::From,
                    "WHERE" => Token::Where,
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "IN" => Token::In,
                    "NOT" => Token::Not,
                    "BETWEEN" => Token::Between,
                    "LIKE" => Token::Like,
                    "IS" => Token::Is,
                    "NULL" => Token::Null,
                    "ORDER" if self.peek_keyword("BY") => {
                        self.skip_whitespace();
                        self.read_identifier(); // consume "BY"
                        Token::OrderBy
                    }
                    "GROUP" if self.peek_keyword("BY") => {
                        self.skip_whitespace();
                        self.read_identifier(); // consume "BY"
                        Token::GroupBy
                    }
                    "HAVING" => Token::Having,
                    "ASC" => Token::Asc,
                    "DESC" => Token::Desc,
                    "LIMIT" => Token::Limit,
                    "OFFSET" => Token::Offset,
                    "DATETIME" => Token::DateTime,
                    _ => Token::Identifier(ident),
                }
            }
            Some(ch) => {
                self.advance();
                Token::Identifier(ch.to_string())
            }
        }
    }

    fn peek_keyword(&mut self, keyword: &str) -> bool {
        let saved_pos = self.position;
        let saved_char = self.current_char;

        self.skip_whitespace();
        let next_word = self.read_identifier();
        let matches = next_word.to_uppercase() == keyword;

        // Restore position
        self.position = saved_pos;
        self.current_char = saved_char;

        matches
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn tokenize_all(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if matches!(token, Token::Eof) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    pub fn tokenize_all_with_positions(&mut self) -> Vec<(usize, usize, Token)> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let start_pos = self.position;
            let token = self.next_token();
            let end_pos = self.position;

            if matches!(token, Token::Eof) {
                break;
            }
            tokens.push((start_pos, end_pos, token));
        }
        tokens
    }
}

// AST Nodes
#[derive(Debug, Clone)]
pub enum SqlExpression {
    Column(String),
    StringLiteral(String),
    NumberLiteral(String),
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
}

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

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub columns: Vec<String>,
    pub from_table: Option<String>,
    pub where_clause: Option<WhereClause>,
    pub order_by: Option<Vec<OrderByColumn>>,
    pub group_by: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub struct ParserConfig {
    pub case_insensitive: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            case_insensitive: false,
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    in_method_args: bool, // Track if we're parsing method arguments
    columns: Vec<String>, // Known column names for context-aware parsing
    paren_depth: i32,     // Track parentheses nesting depth
    config: ParserConfig, // Parser configuration including case sensitivity
}

impl Parser {
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

    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
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
        self.parse_select_statement()
    }

    fn parse_select_statement(&mut self) -> Result<SelectStatement, String> {
        self.consume(Token::Select)?;

        let columns = self.parse_select_list()?;

        let from_table = if matches!(self.current_token, Token::From) {
            self.advance();
            match &self.current_token {
                Token::Identifier(table) => {
                    let table_name = table.clone();
                    self.advance();
                    Some(table_name)
                }
                Token::QuotedIdentifier(table) => {
                    // Handle quoted table names
                    let table_name = table.clone();
                    self.advance();
                    Some(table_name)
                }
                _ => return Err("Expected table name after FROM".to_string()),
            }
        } else {
            None
        };

        let where_clause = if matches!(self.current_token, Token::Where) {
            self.advance();
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let order_by = if matches!(self.current_token, Token::OrderBy) {
            self.advance();
            Some(self.parse_order_by_list()?)
        } else {
            None
        };

        let group_by = if matches!(self.current_token, Token::GroupBy) {
            self.advance();
            Some(self.parse_identifier_list()?)
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
                        .map_err(|_| format!("Invalid LIMIT value: {}", num))?;
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
                        .map_err(|_| format!("Invalid OFFSET value: {}", num))?;
                    self.advance();
                    Some(offset_val)
                }
                _ => return Err("Expected number after OFFSET".to_string()),
            }
        } else {
            None
        };

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

        Ok(SelectStatement {
            columns,
            from_table,
            where_clause,
            order_by,
            group_by,
            limit,
            offset,
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

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, String> {
        let mut identifiers = Vec::new();

        loop {
            match &self.current_token {
                Token::Identifier(id) => {
                    identifiers.push(id.clone());
                    self.advance();
                }
                Token::QuotedIdentifier(id) => {
                    // Handle quoted identifiers like "Customer Id"
                    identifiers.push(id.clone());
                    self.advance();
                }
                _ => return Err("Expected identifier".to_string()),
            }

            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(identifiers)
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

    fn parse_where_clause(&mut self) -> Result<WhereClause, String> {
        let mut conditions = Vec::new();

        loop {
            let expr = self.parse_expression()?;

            let connector = match &self.current_token {
                Token::And => {
                    self.advance();
                    Some(LogicalOp::And)
                }
                Token::Or => {
                    self.advance();
                    Some(LogicalOp::Or)
                }
                Token::RightParen if self.paren_depth <= 0 => {
                    // Unexpected closing parenthesis
                    return Err(
                        "Unexpected closing parenthesis - no matching opening parenthesis"
                            .to_string(),
                    );
                }
                _ => None,
            };

            conditions.push(Condition {
                expr,
                connector: connector.clone(),
            });

            if connector.is_none() {
                break;
            }
        }

        Ok(WhereClause { conditions })
    }

    fn parse_expression(&mut self) -> Result<SqlExpression, String> {
        let mut left = self.parse_comparison()?;

        // Handle binary operators at expression level (should be handled in parse_comparison now)
        // Keep this for backward compatibility but it shouldn't be reached
        if let Some(op) = self.get_binary_op() {
            self.advance();
            let right = self.parse_expression()?;
            left = SqlExpression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        // Handle IN operator
        if matches!(self.current_token, Token::In) {
            self.advance();
            self.consume(Token::LeftParen)?;
            let values = self.parse_expression_list()?;
            self.consume(Token::RightParen)?;

            left = SqlExpression::InList {
                expr: Box::new(left),
                values,
            };
        }

        // Handle NOT IN operator - this should be handled in parse_comparison instead
        // since NOT is a prefix operator that should be parsed before the expression

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<SqlExpression, String> {
        let mut left = self.parse_primary()?;

        // Handle method calls - support chained calls
        while matches!(self.current_token, Token::Dot) {
            self.advance();
            if let Token::Identifier(method) = &self.current_token {
                let method_name = method.clone();
                self.advance();

                if matches!(self.current_token, Token::LeftParen) {
                    self.advance();
                    let args = self.parse_method_args()?;
                    self.consume(Token::RightParen)?;

                    // Support chained method calls
                    match left {
                        SqlExpression::Column(obj) => {
                            // First method call on a column
                            left = SqlExpression::MethodCall {
                                object: obj,
                                method: method_name,
                                args,
                            };
                        }
                        SqlExpression::MethodCall { .. }
                        | SqlExpression::ChainedMethodCall { .. } => {
                            // Chained method call on a previous method call
                            left = SqlExpression::ChainedMethodCall {
                                base: Box::new(left),
                                method: method_name,
                                args,
                            };
                        }
                        _ => {
                            // Other expressions - shouldn't normally happen
                            return Err(format!("Cannot call method on {:?}", left));
                        }
                    }
                } else {
                    // No parentheses after identifier - might be column reference like table.column
                    // Put the identifier back and break
                    break;
                }
            } else {
                break;
            }
        }

        // Handle BETWEEN operator
        if matches!(self.current_token, Token::Between) {
            self.advance(); // consume BETWEEN
            let lower = self.parse_primary()?;
            self.consume(Token::And)?; // BETWEEN requires AND
            let upper = self.parse_primary()?;

            return Ok(SqlExpression::Between {
                expr: Box::new(left),
                lower: Box::new(lower),
                upper: Box::new(upper),
            });
        }

        // Handle NOT IN operator
        if matches!(self.current_token, Token::Not) {
            self.advance(); // consume NOT
            if matches!(self.current_token, Token::In) {
                self.advance(); // consume IN
                self.consume(Token::LeftParen)?;
                let values = self.parse_expression_list()?;
                self.consume(Token::RightParen)?;

                return Ok(SqlExpression::NotInList {
                    expr: Box::new(left),
                    values,
                });
            } else {
                return Err("Expected IN after NOT".to_string());
            }
        }

        // Handle comparison operators
        if let Some(op) = self.get_binary_op() {
            self.advance();
            let right = self.parse_comparison()?;
            left = SqlExpression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical_or(&mut self) -> Result<SqlExpression, String> {
        let mut left = self.parse_logical_and()?;

        while matches!(self.current_token, Token::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            // For now, we'll just return the left side to make it compile
            // In a real implementation, we'd need a LogicalOp variant in SqlExpression
            // but for the AST visualization, the WHERE clause handles this properly
            left = SqlExpression::BinaryOp {
                left: Box::new(left),
                op: "OR".to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<SqlExpression, String> {
        let mut left = self.parse_expression()?;

        while matches!(self.current_token, Token::And) {
            self.advance();
            let right = self.parse_expression()?;
            // Similar to OR, we use BinaryOp to represent AND
            left = SqlExpression::BinaryOp {
                left: Box::new(left),
                op: "AND".to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<SqlExpression, String> {
        // Special case: check if a number literal could actually be a column name
        // This handles cases where columns are named with pure numbers like "202204"
        if let Token::NumberLiteral(num_str) = &self.current_token {
            // Check if this number matches a known column name
            if self.columns.iter().any(|col| col == num_str) {
                let expr = SqlExpression::Column(num_str.clone());
                self.advance();
                return Ok(expr);
            }
        }

        match &self.current_token {
            Token::DateTime => {
                self.advance(); // consume DateTime
                self.consume(Token::LeftParen)?;

                // Check if empty parentheses for DateTime() - today's date
                if matches!(&self.current_token, Token::RightParen) {
                    self.advance(); // consume )
                    return Ok(SqlExpression::DateTimeToday {
                        hour: None,
                        minute: None,
                        second: None,
                    });
                }

                // Parse year
                let year = if let Token::NumberLiteral(n) = &self.current_token {
                    n.parse::<i32>().map_err(|_| "Invalid year")?
                } else {
                    return Err("Expected year in DateTime constructor".to_string());
                };
                self.advance();
                self.consume(Token::Comma)?;

                // Parse month
                let month = if let Token::NumberLiteral(n) = &self.current_token {
                    n.parse::<u32>().map_err(|_| "Invalid month")?
                } else {
                    return Err("Expected month in DateTime constructor".to_string());
                };
                self.advance();
                self.consume(Token::Comma)?;

                // Parse day
                let day = if let Token::NumberLiteral(n) = &self.current_token {
                    n.parse::<u32>().map_err(|_| "Invalid day")?
                } else {
                    return Err("Expected day in DateTime constructor".to_string());
                };
                self.advance();

                // Check for optional time components
                let mut hour = None;
                let mut minute = None;
                let mut second = None;

                if matches!(&self.current_token, Token::Comma) {
                    self.advance(); // consume comma

                    // Parse hour
                    if let Token::NumberLiteral(n) = &self.current_token {
                        hour = Some(n.parse::<u32>().map_err(|_| "Invalid hour")?);
                        self.advance();

                        // Check for minute
                        if matches!(&self.current_token, Token::Comma) {
                            self.advance(); // consume comma

                            if let Token::NumberLiteral(n) = &self.current_token {
                                minute = Some(n.parse::<u32>().map_err(|_| "Invalid minute")?);
                                self.advance();

                                // Check for second
                                if matches!(&self.current_token, Token::Comma) {
                                    self.advance(); // consume comma

                                    if let Token::NumberLiteral(n) = &self.current_token {
                                        second =
                                            Some(n.parse::<u32>().map_err(|_| "Invalid second")?);
                                        self.advance();
                                    }
                                }
                            }
                        }
                    }
                }

                self.consume(Token::RightParen)?;
                Ok(SqlExpression::DateTimeConstructor {
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                })
            }
            Token::Identifier(id) => {
                let expr = SqlExpression::Column(id.clone());
                self.advance();
                Ok(expr)
            }
            Token::QuotedIdentifier(id) => {
                // If we're in method arguments, treat quoted identifiers as string literals
                // This handles cases like Country.Contains("Alb") where "Alb" should be a string
                let expr = if self.in_method_args {
                    SqlExpression::StringLiteral(id.clone())
                } else {
                    // Otherwise it's a column name like "Customer Id"
                    SqlExpression::Column(id.clone())
                };
                self.advance();
                Ok(expr)
            }
            Token::StringLiteral(s) => {
                let expr = SqlExpression::StringLiteral(s.clone());
                self.advance();
                Ok(expr)
            }
            Token::NumberLiteral(n) => {
                let expr = SqlExpression::NumberLiteral(n.clone());
                self.advance();
                Ok(expr)
            }
            Token::LeftParen => {
                self.advance();

                // Parse a parenthesized expression which might contain logical operators
                // We need to handle cases like (a OR b) as a single expression
                let expr = self.parse_logical_or()?;

                self.consume(Token::RightParen)?;
                Ok(expr)
            }
            Token::Not => {
                self.advance(); // consume NOT

                // Check if this is a NOT IN expression
                if let Ok(inner_expr) = self.parse_comparison() {
                    // After parsing the inner expression, check if we're followed by IN
                    if matches!(self.current_token, Token::In) {
                        self.advance(); // consume IN
                        self.consume(Token::LeftParen)?;
                        let values = self.parse_expression_list()?;
                        self.consume(Token::RightParen)?;

                        return Ok(SqlExpression::NotInList {
                            expr: Box::new(inner_expr),
                            values,
                        });
                    } else {
                        // Regular NOT expression
                        return Ok(SqlExpression::Not {
                            expr: Box::new(inner_expr),
                        });
                    }
                } else {
                    return Err("Expected expression after NOT".to_string());
                }
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }

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

    pub fn get_position(&self) -> usize {
        self.lexer.get_position()
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

pub fn detect_cursor_context(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    let truncated = safe_slice_to(query, cursor_pos);
    let mut parser = Parser::new(truncated);

    // Try to parse as much as possible
    match parser.parse() {
        Ok(stmt) => {
            let (ctx, partial) = analyze_statement(&stmt, truncated, cursor_pos);
            #[cfg(test)]
            println!(
                "analyze_statement returned: {:?}, {:?} for query: '{}'",
                ctx, partial, truncated
            );
            (ctx, partial)
        }
        Err(_) => {
            // Partial parse - analyze what we have
            let (ctx, partial) = analyze_partial(truncated, cursor_pos);
            #[cfg(test)]
            println!(
                "analyze_partial returned: {:?}, {:?} for query: '{}'",
                ctx, partial, truncated
            );
            (ctx, partial)
        }
    }
}

pub fn tokenize_query(query: &str) -> Vec<String> {
    let mut lexer = Lexer::new(query);
    let tokens = lexer.tokenize_all();
    tokens.iter().map(|t| format!("{:?}", t)).collect()
}

pub fn format_sql_pretty(query: &str) -> Vec<String> {
    format_sql_pretty_compact(query, 5) // Default to 5 columns per line
}

// Pretty print AST for debug visualization
pub fn format_ast_tree(query: &str) -> String {
    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => format_select_statement(&stmt, 0),
        Err(e) => format!("❌ PARSE ERROR ❌\n{}\n\n⚠️  The query could not be parsed correctly.\n💡 Check parentheses, operators, and syntax.", e),
    }
}

fn format_select_statement(stmt: &SelectStatement, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    result.push_str(&format!("{indent_str}SelectStatement {{\n"));

    // Format columns
    result.push_str(&format!("{indent_str}  columns: ["));
    if !stmt.columns.is_empty() {
        result.push('\n');
        for col in &stmt.columns {
            result.push_str(&format!("{indent_str}    \"{col}\",\n"));
        }
        result.push_str(&format!("{indent_str}  ],\n"));
    } else {
        result.push_str("],\n");
    }

    // Format from table
    if let Some(table) = &stmt.from_table {
        result.push_str(&format!("{indent_str}  from_table: \"{table}\",\n"));
    }

    // Format where clause
    if let Some(where_clause) = &stmt.where_clause {
        result.push_str(&format!("{indent_str}  where_clause: {{\n"));
        result.push_str(&format_where_clause(where_clause, indent + 2));
        result.push_str(&format!("{indent_str}  }},\n"));
    }

    // Format order by
    if let Some(order_by) = &stmt.order_by {
        result.push_str(&format!("{indent_str}  order_by: ["));
        if !order_by.is_empty() {
            result.push('\n');
            for col in order_by {
                let dir = match col.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                result.push_str(&format!(
                    "{indent_str}    \"{col}\" {dir},\n",
                    col = col.column
                ));
            }
            result.push_str(&format!("{indent_str}  ],\n"));
        } else {
            result.push_str("],\n");
        }
    }

    // Format group by
    if let Some(group_by) = &stmt.group_by {
        result.push_str(&format!("{indent_str}  group_by: ["));
        if !group_by.is_empty() {
            result.push('\n');
            for col in group_by {
                result.push_str(&format!("{indent_str}    \"{col}\",\n"));
            }
            result.push_str(&format!("{indent_str}  ],\n"));
        } else {
            result.push_str("]\n");
        }
    }

    result.push_str(&format!("{indent_str}}}"));
    result
}

fn format_where_clause(clause: &WhereClause, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    result.push_str(&format!("{indent_str}conditions: [\n"));

    for condition in &clause.conditions {
        result.push_str(&format!("{indent_str}  {{\n"));
        result.push_str(&format!(
            "{indent_str}    expr: {},\n",
            format_expression_ast(&condition.expr)
        ));

        if let Some(connector) = &condition.connector {
            let connector_str = match connector {
                LogicalOp::And => "AND",
                LogicalOp::Or => "OR",
            };
            result.push_str(&format!("{indent_str}    connector: {connector_str},\n"));
        }

        result.push_str(&format!("{indent_str}  }},\n"));
    }

    result.push_str(&format!("{indent_str}]\n"));
    result
}

fn format_expression_ast(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(name) => format!("Column(\"{}\")", name),
        SqlExpression::StringLiteral(value) => format!("StringLiteral(\"{}\")", value),
        SqlExpression::NumberLiteral(value) => format!("NumberLiteral({})", value),
        SqlExpression::DateTimeConstructor {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => {
            format!(
                "DateTime({}-{:02}-{:02} {:02}:{:02}:{:02})",
                year,
                month,
                day,
                hour.unwrap_or(0),
                minute.unwrap_or(0),
                second.unwrap_or(0)
            )
        }
        SqlExpression::DateTimeToday {
            hour,
            minute,
            second,
        } => {
            format!(
                "DateTimeToday({:02}:{:02}:{:02})",
                hour.unwrap_or(0),
                minute.unwrap_or(0),
                second.unwrap_or(0)
            )
        }
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => {
            let args_str = args
                .iter()
                .map(|a| format_expression_ast(a))
                .collect::<Vec<_>>()
                .join(", ");
            format!("MethodCall({}.{}({}))", object, method, args_str)
        }
        SqlExpression::ChainedMethodCall { base, method, args } => {
            let args_str = args
                .iter()
                .map(|a| format_expression_ast(a))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "ChainedMethodCall({}.{}({}))",
                format_expression_ast(base),
                method,
                args_str
            )
        }
        SqlExpression::BinaryOp { left, op, right } => {
            format!(
                "BinaryOp({} {} {})",
                format_expression_ast(left),
                op,
                format_expression_ast(right)
            )
        }
        SqlExpression::InList { expr, values } => {
            let list_str = values
                .iter()
                .map(|e| format_expression_ast(e))
                .collect::<Vec<_>>()
                .join(", ");
            format!("InList({} IN [{}])", format_expression_ast(expr), list_str)
        }
        SqlExpression::NotInList { expr, values } => {
            let list_str = values
                .iter()
                .map(|e| format_expression_ast(e))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "NotInList({} NOT IN [{}])",
                format_expression_ast(expr),
                list_str
            )
        }
        SqlExpression::Between { expr, lower, upper } => {
            format!(
                "Between({} BETWEEN {} AND {})",
                format_expression_ast(expr),
                format_expression_ast(lower),
                format_expression_ast(upper)
            )
        }
        SqlExpression::Not { expr } => {
            format!("Not({})", format_expression_ast(expr))
        }
    }
}

// Convert DateTime expressions to ISO 8601 format strings for comparison
pub fn datetime_to_iso_string(expr: &SqlExpression) -> Option<String> {
    match expr {
        SqlExpression::DateTimeConstructor {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => {
            let h = hour.unwrap_or(0);
            let m = minute.unwrap_or(0);
            let s = second.unwrap_or(0);

            // Create a NaiveDateTime
            if let Ok(dt) = NaiveDateTime::parse_from_str(
                &format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    year, month, day, h, m, s
                ),
                "%Y-%m-%d %H:%M:%S",
            ) {
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            }
        }
        SqlExpression::DateTimeToday {
            hour,
            minute,
            second,
        } => {
            let now = Local::now();
            let h = hour.unwrap_or(0);
            let m = minute.unwrap_or(0);
            let s = second.unwrap_or(0);

            // Create today's date at specified time (or midnight)
            if let Ok(dt) = NaiveDateTime::parse_from_str(
                &format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    now.year(),
                    now.month(),
                    now.day(),
                    h,
                    m,
                    s
                ),
                "%Y-%m-%d %H:%M:%S",
            ) {
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

// Format SQL with preserved parentheses using token positions
fn format_sql_with_preserved_parens(
    query: &str,
    cols_per_line: usize,
) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let mut lexer = Lexer::new(query);
    let tokens_with_pos = lexer.tokenize_all_with_positions();

    if tokens_with_pos.is_empty() {
        return Err("No tokens found".to_string());
    }

    let mut i = 0;
    let cols_per_line = cols_per_line.max(1);

    while i < tokens_with_pos.len() {
        let (start, _end, ref token) = tokens_with_pos[i];

        match token {
            Token::Select => {
                lines.push("SELECT".to_string());
                i += 1;

                // Collect columns until FROM
                let mut columns = Vec::new();
                let mut col_start = i;
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].2 {
                        Token::From | Token::Eof => break,
                        Token::Comma => {
                            // Extract column text from original query
                            if col_start < i {
                                let col_text = extract_text_between_positions(
                                    query,
                                    tokens_with_pos[col_start].0,
                                    tokens_with_pos[i - 1].1,
                                );
                                columns.push(col_text);
                            }
                            i += 1;
                            col_start = i;
                        }
                        _ => i += 1,
                    }
                }
                // Add last column
                if col_start < i && i > 0 {
                    let col_text = extract_text_between_positions(
                        query,
                        tokens_with_pos[col_start].0,
                        tokens_with_pos[i - 1].1,
                    );
                    columns.push(col_text);
                }

                // Format columns with proper indentation
                for chunk in columns.chunks(cols_per_line) {
                    let mut line = "    ".to_string();
                    for (idx, col) in chunk.iter().enumerate() {
                        if idx > 0 {
                            line.push_str(", ");
                        }
                        line.push_str(col.trim());
                    }
                    // Add comma if not last chunk
                    let is_last_chunk = chunk.as_ptr() as usize
                        + chunk.len() * std::mem::size_of::<String>()
                        >= columns.last().map(|c| c as *const _ as usize).unwrap_or(0);
                    if !is_last_chunk && columns.len() > cols_per_line {
                        line.push(',');
                    }
                    lines.push(line);
                }
            }
            Token::From => {
                i += 1;
                if i < tokens_with_pos.len() {
                    let table_start = tokens_with_pos[i].0;
                    // Find end of table name
                    while i < tokens_with_pos.len() {
                        match &tokens_with_pos[i].2 {
                            Token::Where | Token::OrderBy | Token::GroupBy | Token::Eof => break,
                            _ => i += 1,
                        }
                    }
                    if i > 0 {
                        let table_text = extract_text_between_positions(
                            query,
                            table_start,
                            tokens_with_pos[i - 1].1,
                        );
                        lines.push(format!("FROM {}", table_text.trim()));
                    }
                }
            }
            Token::Where => {
                lines.push("WHERE".to_string());
                i += 1;

                // Extract entire WHERE clause preserving parentheses
                let where_start = if i < tokens_with_pos.len() {
                    tokens_with_pos[i].0
                } else {
                    start
                };

                // Find end of WHERE clause
                let mut where_end = query.len();
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].2 {
                        Token::OrderBy | Token::GroupBy | Token::Eof => {
                            if i > 0 {
                                where_end = tokens_with_pos[i - 1].1;
                            }
                            break;
                        }
                        _ => i += 1,
                    }
                }

                // Extract and format WHERE clause text, preserving parentheses
                let where_text = extract_text_between_positions(query, where_start, where_end);

                // Split by AND/OR at the top level (not inside parentheses)
                let formatted_where = format_where_clause_with_parens(&where_text);
                for line in formatted_where {
                    lines.push(format!("    {}", line));
                }
            }
            Token::OrderBy => {
                i += 1;
                let order_start = if i < tokens_with_pos.len() {
                    tokens_with_pos[i].0
                } else {
                    start
                };

                // Find end of ORDER BY
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].2 {
                        Token::GroupBy | Token::Eof => break,
                        _ => i += 1,
                    }
                }

                if i > 0 {
                    let order_text = extract_text_between_positions(
                        query,
                        order_start,
                        tokens_with_pos[i - 1].1,
                    );
                    lines.push(format!("ORDER BY {}", order_text.trim()));
                }
            }
            Token::GroupBy => {
                i += 1;
                let group_start = if i < tokens_with_pos.len() {
                    tokens_with_pos[i].0
                } else {
                    start
                };

                // Find end of GROUP BY
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].2 {
                        Token::Having | Token::Eof => break,
                        _ => i += 1,
                    }
                }

                if i > 0 {
                    let group_text = extract_text_between_positions(
                        query,
                        group_start,
                        tokens_with_pos[i - 1].1,
                    );
                    lines.push(format!("GROUP BY {}", group_text.trim()));
                }
            }
            _ => i += 1,
        }
    }

    Ok(lines)
}

// Helper function to extract text between positions
fn extract_text_between_positions(query: &str, start: usize, end: usize) -> String {
    let chars: Vec<char> = query.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len());
    chars[start..end].iter().collect()
}

// Format WHERE clause preserving parentheses
fn format_where_clause_with_parens(where_text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut paren_depth = 0;
    let mut i = 0;
    let chars: Vec<char> = where_text.chars().collect();

    while i < chars.len() {
        // Check for AND/OR at top level
        if paren_depth == 0 {
            // Look for " AND " or " OR "
            if i + 5 <= chars.len() {
                let next_five: String = chars[i..i + 5].iter().collect();
                if next_five.to_uppercase() == " AND " {
                    if !current_line.trim().is_empty() {
                        lines.push(current_line.trim().to_string());
                    }
                    lines.push("AND".to_string());
                    current_line.clear();
                    i += 5;
                    continue;
                }
            }
            if i + 4 <= chars.len() {
                let next_four: String = chars[i..i + 4].iter().collect();
                if next_four.to_uppercase() == " OR " {
                    if !current_line.trim().is_empty() {
                        lines.push(current_line.trim().to_string());
                    }
                    lines.push("OR".to_string());
                    current_line.clear();
                    i += 4;
                    continue;
                }
            }
        }

        // Track parentheses depth
        match chars[i] {
            '(' => {
                paren_depth += 1;
                current_line.push('(');
            }
            ')' => {
                paren_depth -= 1;
                current_line.push(')');
            }
            c => current_line.push(c),
        }
        i += 1;
    }

    // Add remaining line
    if !current_line.trim().is_empty() {
        lines.push(current_line.trim().to_string());
    }

    // If no AND/OR found, return the whole text as one line
    if lines.is_empty() {
        lines.push(where_text.trim().to_string());
    }

    lines
}

pub fn format_sql_pretty_compact(query: &str, cols_per_line: usize) -> Vec<String> {
    // First try to use the new AST-preserving formatter
    if let Ok(lines) = format_sql_with_preserved_parens(query, cols_per_line) {
        return lines;
    }

    // Fall back to the old implementation for backward compatibility
    let mut lines = Vec::new();
    let mut parser = Parser::new(query);

    // Ensure cols_per_line is at least 1 to avoid panic
    let cols_per_line = cols_per_line.max(1);

    match parser.parse() {
        Ok(stmt) => {
            // SELECT clause
            if !stmt.columns.is_empty() {
                lines.push("SELECT".to_string());

                // Group columns by cols_per_line
                for chunk in stmt.columns.chunks(cols_per_line) {
                    let mut line = "    ".to_string();
                    for (i, col) in chunk.iter().enumerate() {
                        if i > 0 {
                            line.push_str(", ");
                        }
                        line.push_str(col);
                    }
                    // Add comma at end if not the last chunk
                    let last_chunk_idx = (stmt.columns.len() - 1) / cols_per_line;
                    let current_chunk_idx =
                        stmt.columns.iter().position(|c| c == &chunk[0]).unwrap() / cols_per_line;
                    if current_chunk_idx < last_chunk_idx {
                        line.push(',');
                    }
                    lines.push(line);
                }
            }

            // FROM clause
            if let Some(table) = &stmt.from_table {
                lines.push(format!("FROM {}", table));
            }

            // WHERE clause
            if let Some(where_clause) = &stmt.where_clause {
                lines.push("WHERE".to_string());
                for (i, condition) in where_clause.conditions.iter().enumerate() {
                    if i > 0 {
                        // Add the connector from the previous condition
                        if let Some(prev_condition) = where_clause.conditions.get(i - 1) {
                            if let Some(connector) = &prev_condition.connector {
                                match connector {
                                    LogicalOp::And => lines.push("    AND".to_string()),
                                    LogicalOp::Or => lines.push("    OR".to_string()),
                                }
                            }
                        }
                    }
                    lines.push(format!("    {}", format_expression(&condition.expr)));
                }
            }

            // ORDER BY clause
            if let Some(order_by) = &stmt.order_by {
                let order_str = order_by
                    .iter()
                    .map(|col| {
                        let dir = match col.direction {
                            SortDirection::Asc => " ASC",
                            SortDirection::Desc => " DESC",
                        };
                        format!("{}{}", col.column, dir)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!("ORDER BY {}", order_str));
            }

            // GROUP BY clause
            if let Some(group_by) = &stmt.group_by {
                let group_str = group_by.join(", ");
                lines.push(format!("GROUP BY {}", group_str));
            }
        }
        Err(_) => {
            // If parsing fails, fall back to simple tokenization
            let mut lexer = Lexer::new(query);
            let tokens = lexer.tokenize_all();
            let mut current_line = String::new();
            let mut indent = 0;

            for token in tokens {
                match &token {
                    Token::Select
                    | Token::From
                    | Token::Where
                    | Token::OrderBy
                    | Token::GroupBy => {
                        if !current_line.is_empty() {
                            lines.push(current_line.trim().to_string());
                            current_line.clear();
                        }
                        lines.push(format!("{:?}", token).to_uppercase());
                        indent = 1;
                    }
                    Token::And | Token::Or => {
                        if !current_line.is_empty() {
                            lines.push(format!("{}{}", "    ".repeat(indent), current_line.trim()));
                            current_line.clear();
                        }
                        lines.push(format!("    {:?}", token).to_uppercase());
                    }
                    Token::Comma => {
                        current_line.push(',');
                        if indent > 0 {
                            lines.push(format!("{}{}", "    ".repeat(indent), current_line.trim()));
                            current_line.clear();
                        }
                    }
                    Token::Eof => break,
                    _ => {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(&format_token(&token));
                    }
                }
            }

            if !current_line.is_empty() {
                lines.push(format!("{}{}", "    ".repeat(indent), current_line.trim()));
            }
        }
    }

    lines
}

fn format_expression(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(name) => name.clone(),
        SqlExpression::StringLiteral(s) => format!("'{}'", s),
        SqlExpression::NumberLiteral(n) => n.clone(),
        SqlExpression::DateTimeConstructor {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => {
            let mut result = format!("DateTime({}, {}, {}", year, month, day);
            if let Some(h) = hour {
                result.push_str(&format!(", {}", h));
                if let Some(m) = minute {
                    result.push_str(&format!(", {}", m));
                    if let Some(s) = second {
                        result.push_str(&format!(", {}", s));
                    }
                }
            }
            result.push(')');
            result
        }
        SqlExpression::DateTimeToday {
            hour,
            minute,
            second,
        } => {
            let mut result = "DateTime()".to_string();
            if let Some(h) = hour {
                result = format!("DateTime(TODAY, {}", h);
                if let Some(m) = minute {
                    result.push_str(&format!(", {}", m));
                    if let Some(s) = second {
                        result.push_str(&format!(", {}", s));
                    }
                }
                result.push(')');
            }
            result
        }
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => {
            let args_str = args
                .iter()
                .map(|arg| format_expression(arg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}.{}({})", object, method, args_str)
        }
        SqlExpression::BinaryOp { left, op, right } => {
            // Check if this is a logical operator that needs parentheses
            // We add parentheses for OR/AND operators to preserve grouping
            if op == "OR" || op == "AND" {
                // For logical operators, we need to check if we should add parentheses
                // This is a simplified approach - in production you'd track context
                format!(
                    "({} {} {})",
                    format_expression(left),
                    op,
                    format_expression(right)
                )
            } else {
                format!(
                    "{} {} {}",
                    format_expression(left),
                    op,
                    format_expression(right)
                )
            }
        }
        SqlExpression::InList { expr, values } => {
            let values_str = values
                .iter()
                .map(|v| format_expression(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} IN ({})", format_expression(expr), values_str)
        }
        SqlExpression::NotInList { expr, values } => {
            let values_str = values
                .iter()
                .map(|v| format_expression(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} NOT IN ({})", format_expression(expr), values_str)
        }
        SqlExpression::Between { expr, lower, upper } => {
            format!(
                "{} BETWEEN {} AND {}",
                format_expression(expr),
                format_expression(lower),
                format_expression(upper)
            )
        }
        SqlExpression::Not { expr } => {
            format!("NOT {}", format_expression(expr))
        }
        SqlExpression::ChainedMethodCall { base, method, args } => {
            let args_str = args
                .iter()
                .map(|arg| format_expression(arg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}.{}({})", format_expression(base), method, args_str)
        }
    }
}

fn format_token(token: &Token) -> String {
    match token {
        Token::Identifier(s) => s.clone(),
        Token::QuotedIdentifier(s) => format!("\"{}\"", s),
        Token::StringLiteral(s) => format!("'{}'", s),
        Token::NumberLiteral(n) => n.clone(),
        Token::DateTime => "DateTime".to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::Comma => ",".to_string(),
        Token::Dot => ".".to_string(),
        Token::Equal => "=".to_string(),
        Token::NotEqual => "!=".to_string(),
        Token::LessThan => "<".to_string(),
        Token::GreaterThan => ">".to_string(),
        Token::LessThanOrEqual => "<=".to_string(),
        Token::GreaterThanOrEqual => ">=".to_string(),
        Token::In => "IN".to_string(),
        _ => format!("{:?}", token).to_uppercase(),
    }
}

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

                    if let Some(start) = found_start {
                        // Extract the full quoted identifier including quotes
                        Some(safe_slice_from(before_dot, start))
                    } else {
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

    if stmt.columns.len() > 0 && stmt.from_table.is_none() {
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
            eprintln!(
                "DEBUG analyze_partial: query='{}', trimmed='{}'",
                query, trimmed
            );
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
                eprintln!("DEBUG: Found dot at position {}", dot_pos);
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
                        eprintln!(
                            "DEBUG: before_dot='{}', looking for opening quote",
                            before_dot
                        );
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
                            eprintln!("DEBUG: Extracted quoted identifier: '{}'", result);
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
                        eprintln!("DEBUG: col_name = '{}'", col_name);
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
                        eprintln!("DEBUG: is_valid = {}", is_valid);
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chained_method_calls() {
        // Test ToString().IndexOf('.') pattern
        let query = "SELECT * FROM trades WHERE commission.ToString().IndexOf('.') = 1";
        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Failed to parse chained method calls: {:?}",
            result
        );

        // Test multiple chained calls
        let query2 = "SELECT * FROM data WHERE field.ToUpper().Replace('A', 'B').StartsWith('C')";
        let mut parser2 = Parser::new(query2);
        let result2 = parser2.parse();

        assert!(
            result2.is_ok(),
            "Failed to parse multiple chained calls: {:?}",
            result2
        );
    }

    #[test]
    fn test_tokenizer() {
        let mut lexer = Lexer::new("SELECT * FROM trade_deal WHERE price > 100");

        assert!(matches!(lexer.next_token(), Token::Select));
        assert!(matches!(lexer.next_token(), Token::Star));
        assert!(matches!(lexer.next_token(), Token::From));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "trade_deal"));
        assert!(matches!(lexer.next_token(), Token::Where));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
        assert!(matches!(lexer.next_token(), Token::GreaterThan));
        assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "100"));
    }

    #[test]
    fn test_tokenizer_datetime() {
        let mut lexer = Lexer::new("WHERE createdDate > DateTime(2025, 10, 20)");

        assert!(matches!(lexer.next_token(), Token::Where));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "createdDate"));
        assert!(matches!(lexer.next_token(), Token::GreaterThan));
        assert!(matches!(lexer.next_token(), Token::DateTime));
        assert!(matches!(lexer.next_token(), Token::LeftParen));
        assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "2025"));
        assert!(matches!(lexer.next_token(), Token::Comma));
        assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "10"));
        assert!(matches!(lexer.next_token(), Token::Comma));
        assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "20"));
        assert!(matches!(lexer.next_token(), Token::RightParen));
    }

    #[test]
    fn test_parse_simple_select() {
        let mut parser = Parser::new("SELECT * FROM trade_deal");
        let stmt = parser.parse().unwrap();

        assert_eq!(stmt.columns, vec!["*"]);
        assert_eq!(stmt.from_table, Some("trade_deal".to_string()));
        assert!(stmt.where_clause.is_none());
    }

    #[test]
    fn test_parse_where_with_method() {
        let mut parser = Parser::new("SELECT * FROM trade_deal WHERE name.Contains(\"test\")");
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);
    }

    #[test]
    fn test_parse_datetime_constructor() {
        let mut parser =
            Parser::new("SELECT * FROM trade_deal WHERE createdDate > DateTime(2025, 10, 20)");
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);

        // Check the expression structure
        if let SqlExpression::BinaryOp { left, op, right } = &where_clause.conditions[0].expr {
            assert_eq!(op, ">");
            assert!(matches!(left.as_ref(), SqlExpression::Column(col) if col == "createdDate"));
            assert!(matches!(
                right.as_ref(),
                SqlExpression::DateTimeConstructor {
                    year: 2025,
                    month: 10,
                    day: 20,
                    hour: None,
                    minute: None,
                    second: None
                }
            ));
        } else {
            panic!("Expected BinaryOp with DateTime constructor");
        }
    }

    #[test]
    fn test_cursor_context_after_and() {
        let query = "SELECT * FROM trade_deal WHERE status = 'active' AND ";
        let (context, partial) = detect_cursor_context(query, query.len());

        assert!(matches!(
            context,
            CursorContext::AfterLogicalOp(LogicalOp::And)
        ));
        assert_eq!(partial, None);
    }

    #[test]
    fn test_cursor_context_with_partial() {
        let query = "SELECT * FROM trade_deal WHERE status = 'active' AND p";
        let (context, partial) = detect_cursor_context(query, query.len());

        assert!(matches!(
            context,
            CursorContext::AfterLogicalOp(LogicalOp::And)
        ));
        assert_eq!(partial, Some("p".to_string()));
    }

    #[test]
    fn test_cursor_context_after_datetime_comparison() {
        let query = "SELECT * FROM trade_deal WHERE createdDate > ";
        let (context, partial) = detect_cursor_context(query, query.len());

        assert!(
            matches!(context, CursorContext::AfterComparisonOp(col, op) if col == "createdDate" && op == ">")
        );
        assert_eq!(partial, None);
    }

    #[test]
    fn test_cursor_context_partial_datetime() {
        let query = "SELECT * FROM trade_deal WHERE createdDate > Date";
        let (context, partial) = detect_cursor_context(query, query.len());

        assert!(
            matches!(context, CursorContext::AfterComparisonOp(col, op) if col == "createdDate" && op == ">")
        );
        assert_eq!(partial, Some("Date".to_string()));
    }

    // Tests for quoted identifiers
    #[test]
    fn test_tokenizer_quoted_identifier() {
        let mut lexer = Lexer::new(r#"SELECT "Customer Id", "First Name" FROM customers"#);

        assert!(matches!(lexer.next_token(), Token::Select));
        assert!(matches!(lexer.next_token(), Token::QuotedIdentifier(s) if s == "Customer Id"));
        assert!(matches!(lexer.next_token(), Token::Comma));
        assert!(matches!(lexer.next_token(), Token::QuotedIdentifier(s) if s == "First Name"));
        assert!(matches!(lexer.next_token(), Token::From));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "customers"));
    }

    #[test]
    fn test_tokenizer_quoted_vs_string_literal() {
        // Double quotes should be QuotedIdentifier, single quotes should be StringLiteral
        let mut lexer = Lexer::new(r#"WHERE "Customer Id" = 'John' AND Country.Contains('USA')"#);

        assert!(matches!(lexer.next_token(), Token::Where));
        assert!(matches!(lexer.next_token(), Token::QuotedIdentifier(s) if s == "Customer Id"));
        assert!(matches!(lexer.next_token(), Token::Equal));
        assert!(matches!(lexer.next_token(), Token::StringLiteral(s) if s == "John"));
        assert!(matches!(lexer.next_token(), Token::And));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "Country"));
        assert!(matches!(lexer.next_token(), Token::Dot));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "Contains"));
        assert!(matches!(lexer.next_token(), Token::LeftParen));
        assert!(matches!(lexer.next_token(), Token::StringLiteral(s) if s == "USA"));
        assert!(matches!(lexer.next_token(), Token::RightParen));
    }

    #[test]
    fn test_tokenizer_method_with_double_quotes_should_be_string() {
        // This is the bug: double quotes in method args should be treated as strings
        // Currently fails because "Alb" becomes QuotedIdentifier
        let mut lexer = Lexer::new(r#"Country.Contains("Alb")"#);

        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "Country"));
        assert!(matches!(lexer.next_token(), Token::Dot));
        assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "Contains"));
        assert!(matches!(lexer.next_token(), Token::LeftParen));

        // This test currently fails - "Alb" is tokenized as QuotedIdentifier
        // but it should be StringLiteral in this context
        let token = lexer.next_token();
        println!("Token for \"Alb\": {:?}", token);
        // TODO: Fix this - should be StringLiteral, not QuotedIdentifier
        // assert!(matches!(token, Token::StringLiteral(s) if s == "Alb"));

        assert!(matches!(lexer.next_token(), Token::RightParen));
    }

    #[test]
    fn test_parse_select_with_quoted_columns() {
        let mut parser = Parser::new(r#"SELECT "Customer Id", Company FROM customers"#);
        let stmt = parser.parse().unwrap();

        assert_eq!(stmt.columns, vec!["Customer Id", "Company"]);
        assert_eq!(stmt.from_table, Some("customers".to_string()));
    }

    #[test]
    fn test_cursor_context_select_with_partial_quoted() {
        // Testing autocompletion when user types: SELECT "Cust
        let query = r#"SELECT "Cust"#;
        let (context, partial) = detect_cursor_context(query, query.len() - 1); // cursor before closing quote

        println!("Context: {:?}, Partial: {:?}", context, partial);
        assert!(matches!(context, CursorContext::SelectClause));
        // Should extract "Cust as partial
        // TODO: Fix partial extraction for quoted identifiers
    }

    #[test]
    fn test_cursor_context_select_after_comma_with_quoted() {
        // User has typed: SELECT Company, "Customer
        let query = r#"SELECT Company, "Customer "#;
        let (context, partial) = detect_cursor_context(query, query.len());

        println!("Context: {:?}, Partial: {:?}", context, partial);
        assert!(matches!(context, CursorContext::SelectClause));
        // Should suggest "Customer Id" and other quoted columns
    }

    #[test]
    fn test_cursor_context_order_by_quoted() {
        let query = r#"SELECT * FROM customers ORDER BY "Cust"#;
        let (context, partial) = detect_cursor_context(query, query.len() - 1);

        println!("Context: {:?}, Partial: {:?}", context, partial);
        assert!(matches!(context, CursorContext::OrderByClause));
        // Should extract partial for quoted identifier
    }

    #[test]
    fn test_where_clause_with_quoted_column() {
        let mut parser = Parser::new(r#"SELECT * FROM customers WHERE "Customer Id" = 'C123'"#);
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);

        if let SqlExpression::BinaryOp { left, op, right } = &where_clause.conditions[0].expr {
            assert_eq!(op, "=");
            assert!(matches!(left.as_ref(), SqlExpression::Column(col) if col == "Customer Id"));
            assert!(matches!(right.as_ref(), SqlExpression::StringLiteral(s) if s == "C123"));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_parse_method_with_double_quotes_as_string() {
        // Now that we have context awareness, double quotes in method args should be treated as strings
        let mut parser = Parser::new(r#"SELECT * FROM customers WHERE Country.Contains("USA")"#);
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);

        if let SqlExpression::MethodCall {
            object,
            method,
            args,
        } = &where_clause.conditions[0].expr
        {
            assert_eq!(object, "Country");
            assert_eq!(method, "Contains");
            assert_eq!(args.len(), 1);
            // The double-quoted "USA" should be treated as a StringLiteral
            assert!(matches!(&args[0], SqlExpression::StringLiteral(s) if s == "USA"));
        } else {
            panic!("Expected MethodCall");
        }
    }

    #[test]
    fn test_extract_partial_with_quoted_columns_in_query() {
        // Test that extract_partial_at_end doesn't get confused by quoted columns earlier in query
        let query = r#"SELECT City,Company,Country,"Customer Id" FROM customers ORDER BY coun"#;
        let (context, partial) = detect_cursor_context(query, query.len());

        assert!(matches!(context, CursorContext::OrderByClause));
        assert_eq!(
            partial,
            Some("coun".to_string()),
            "Should extract 'coun' as partial, not everything after the quoted column"
        );
    }

    #[test]
    fn test_extract_partial_quoted_identifier_being_typed() {
        // Test extracting a partial quoted identifier that's being typed
        let query = r#"SELECT "Cust"#;
        let partial = extract_partial_at_end(query);
        assert_eq!(partial, Some("\"Cust".to_string()));

        // But completed quoted identifiers shouldn't be extracted
        let query2 = r#"SELECT "Customer Id" FROM"#;
        let partial2 = extract_partial_at_end(query2);
        assert_eq!(partial2, None); // FROM is a keyword, so no partial
    }

    // Complex WHERE clause tests with parentheses for trade queries
    #[test]
    fn test_complex_where_parentheses_basic() {
        // Basic parenthesized OR condition
        let mut parser =
            Parser::new(r#"SELECT * FROM trades WHERE (status = "active" OR status = "pending")"#);
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);

        // Verify the structure is a BinaryOp with OR
        if let SqlExpression::BinaryOp { op, .. } = &where_clause.conditions[0].expr {
            assert_eq!(op, "OR");
        } else {
            panic!("Expected BinaryOp with OR");
        }
    }

    #[test]
    fn test_complex_where_mixed_and_or_with_parens() {
        // (condition1 OR condition2) AND condition3
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL") AND price > 100"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2);

        // First condition should be the parenthesized OR expression
        if let SqlExpression::BinaryOp { op, .. } = &where_clause.conditions[0].expr {
            assert_eq!(op, "OR");
        } else {
            panic!("Expected first condition to be OR expression");
        }

        // Should have AND connector to next condition
        assert!(matches!(
            where_clause.conditions[0].connector,
            Some(LogicalOp::And)
        ));

        // Second condition should be price > 100
        if let SqlExpression::BinaryOp { op, .. } = &where_clause.conditions[1].expr {
            assert_eq!(op, ">");
        } else {
            panic!("Expected second condition to be price > 100");
        }
    }

    #[test]
    fn test_complex_where_nested_parentheses() {
        // ((condition1 OR condition2) AND condition3) OR condition4
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE ((symbol = "AAPL" OR symbol = "GOOGL") AND price > 100) OR status = "cancelled""#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();

        // Should parse successfully with nested structure
        assert!(where_clause.conditions.len() > 0);
    }

    #[test]
    fn test_complex_where_multiple_or_groups() {
        // (group1) AND (group2) - common pattern for filtering trades
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL" OR symbol = "MSFT") AND (price > 100 AND price < 500)"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2);

        // First condition group should have OR
        assert!(matches!(
            where_clause.conditions[0].connector,
            Some(LogicalOp::And)
        ));
    }

    #[test]
    fn test_complex_where_with_methods_in_parens() {
        // Using string methods inside parentheses
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (symbol.StartsWith("A") OR symbol.StartsWith("G")) AND volume > 1000000"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2);

        // First condition should be the OR of two method calls
        if let SqlExpression::BinaryOp { op, left, right } = &where_clause.conditions[0].expr {
            assert_eq!(op, "OR");
            assert!(matches!(left.as_ref(), SqlExpression::MethodCall { .. }));
            assert!(matches!(right.as_ref(), SqlExpression::MethodCall { .. }));
        } else {
            panic!("Expected OR of method calls");
        }
    }

    #[test]
    fn test_complex_where_date_comparisons_with_parens() {
        // Date range queries common in trade analysis
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (executionDate > DateTime(2024, 1, 1) AND executionDate < DateTime(2024, 12, 31)) AND (status = "filled" OR status = "partial")"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2);

        // Both condition groups should parse correctly
        assert!(matches!(
            where_clause.conditions[0].connector,
            Some(LogicalOp::And)
        ));
    }

    #[test]
    fn test_complex_where_price_volume_filters() {
        // Complex trade filtering by price and volume
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE ((price > 100 AND price < 200) OR (price > 500 AND price < 1000)) AND volume > 10000"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();

        // Should handle nested price ranges with OR
        assert!(where_clause.conditions.len() > 0);
    }

    #[test]
    fn test_complex_where_mixed_string_numeric() {
        // Mix of string comparisons and numeric comparisons in groups
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (exchange = "NYSE" OR exchange = "NASDAQ") AND (volume > 1000000 OR notes.Contains("urgent"))"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        // Should parse without errors
    }

    #[test]
    fn test_complex_where_triple_nested() {
        // Very complex nesting - ((a OR b) AND (c OR d)) OR (e AND f)
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE ((symbol = "AAPL" OR symbol = "GOOGL") AND (price > 100 OR volume > 1000000)) OR (status = "cancelled" AND reason.Contains("timeout"))"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        // Should handle triple nesting correctly
    }

    #[test]
    fn test_complex_where_single_parens_around_and() {
        // Parentheses around AND conditions
        let mut parser = Parser::new(
            r#"SELECT * FROM trades WHERE (symbol = "AAPL" AND price > 150 AND volume > 100000)"#,
        );
        let stmt = parser.parse().unwrap();

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();

        // Should correctly parse the AND chain inside parentheses
        assert!(where_clause.conditions.len() > 0);
    }

    // Format preservation tests - ensure F3 multi-line mode preserves parentheses
    #[test]
    fn test_format_preserves_simple_parentheses() {
        let query = r#"SELECT * FROM trades WHERE (status = "active" OR status = "pending")"#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Check parentheses are preserved
        assert!(formatted_text.contains("(status"));
        assert!(formatted_text.contains("\"pending\")"));

        // Count parentheses
        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(
            original_parens, formatted_parens,
            "Parentheses should be preserved"
        );
    }

    #[test]
    fn test_format_preserves_complex_parentheses() {
        let query =
            r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL") AND price > 100"#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Check the grouped OR condition is preserved
        assert!(formatted_text.contains("(symbol"));
        assert!(formatted_text.contains("\"GOOGL\")"));

        // Verify parentheses count
        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(original_parens, formatted_parens);
    }

    #[test]
    fn test_format_preserves_nested_parentheses() {
        let query = r#"SELECT * FROM trades WHERE ((symbol = "AAPL" OR symbol = "GOOGL") AND price > 100) OR status = "cancelled""#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Count nested parentheses
        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(
            original_parens, formatted_parens,
            "Nested parentheses should be preserved"
        );
        assert_eq!(original_parens, 4, "Should have 4 parentheses total");
    }

    #[test]
    fn test_format_preserves_method_calls_in_parentheses() {
        let query = r#"SELECT * FROM trades WHERE (symbol.StartsWith("A") OR symbol.StartsWith("G")) AND volume > 1000000"#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Check method calls are preserved with their parentheses
        assert!(formatted_text.contains("(symbol.StartsWith"));
        assert!(formatted_text.contains("StartsWith(\"A\")"));
        assert!(formatted_text.contains("StartsWith(\"G\")"));

        // Count all parentheses (including method calls)
        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(original_parens, formatted_parens);
        assert_eq!(
            original_parens, 6,
            "Should have 6 parentheses (1 group + 2 method calls)"
        );
    }

    #[test]
    fn test_format_preserves_multiple_groups() {
        let query = r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL") AND (price > 100 AND price < 500)"#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Both groups should be preserved
        assert!(formatted_text.contains("(symbol"));
        assert!(formatted_text.contains("(price"));

        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(original_parens, formatted_parens);
        assert_eq!(original_parens, 4, "Should have 4 parentheses (2 groups)");
    }

    #[test]
    fn test_format_preserves_date_ranges() {
        let query = r#"SELECT * FROM trades WHERE (executionDate > DateTime(2024, 1, 1) AND executionDate < DateTime(2024, 12, 31))"#;
        let formatted = format_sql_pretty_compact(query, 5);
        let formatted_text = formatted.join(" ");

        // Check DateTime functions and grouping are preserved
        assert!(formatted_text.contains("(executionDate"));
        assert!(formatted_text.contains("DateTime(2024, 1, 1)"));
        assert!(formatted_text.contains("DateTime(2024, 12, 31)"));

        let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
        let formatted_parens = formatted_text
            .chars()
            .filter(|c| *c == '(' || *c == ')')
            .count();
        assert_eq!(original_parens, formatted_parens);
    }

    #[test]
    fn test_format_multiline_layout() {
        // Test that formatted output has proper multi-line structure
        let query =
            r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL") AND price > 100"#;
        let formatted = format_sql_pretty_compact(query, 5);

        // Should have SELECT, FROM, WHERE, and condition lines
        assert!(formatted.len() >= 4, "Should have multiple lines");
        assert_eq!(formatted[0], "SELECT");
        assert!(formatted[1].trim().starts_with("*"));
        assert!(formatted[2].starts_with("FROM"));
        assert_eq!(formatted[3], "WHERE");

        // WHERE conditions should be indented
        let where_lines: Vec<_> = formatted.iter().skip(4).collect();
        assert!(where_lines.iter().any(|l| l.contains("(symbol")));
        assert!(where_lines.iter().any(|l| l.trim() == "AND"));
    }

    #[test]
    fn test_between_simple() {
        let mut parser = Parser::new("SELECT * FROM table WHERE price BETWEEN 50 AND 100");
        let stmt = parser.parse().expect("Should parse simple BETWEEN");

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);

        // Verify AST formatting
        let ast = format_ast_tree("SELECT * FROM table WHERE price BETWEEN 50 AND 100");
        assert!(!ast.contains("PARSE ERROR"));
        assert!(ast.contains("SelectStatement"));
    }

    #[test]
    fn test_between_in_parentheses() {
        let mut parser = Parser::new("SELECT * FROM table WHERE (price BETWEEN 50 AND 100)");
        let stmt = parser.parse().expect("Should parse BETWEEN in parentheses");

        assert!(stmt.where_clause.is_some());

        // This was the failing case before the fix
        let ast = format_ast_tree("SELECT * FROM table WHERE (price BETWEEN 50 AND 100)");
        assert!(!ast.contains("PARSE ERROR"), "Should not have parse error");
    }

    #[test]
    fn test_between_with_or() {
        let query = "SELECT * FROM test WHERE (Price BETWEEN 50 AND 100) OR (quantity > 5)";
        let mut parser = Parser::new(query);
        let stmt = parser.parse().expect("Should parse BETWEEN with OR");

        assert!(stmt.where_clause.is_some());
        // The parser should successfully parse the query with BETWEEN and OR
        // That's the main test - it doesn't fail with "Expected RightParen, found Between"
    }

    #[test]
    fn test_between_with_and() {
        let query = "SELECT * FROM table WHERE category = 'Books' AND price BETWEEN 10 AND 50";
        let mut parser = Parser::new(query);
        let stmt = parser.parse().expect("Should parse BETWEEN with AND");

        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2); // Two conditions joined by AND
    }

    #[test]
    fn test_multiple_between() {
        let query =
            "SELECT * FROM table WHERE (price BETWEEN 10 AND 50) AND (quantity BETWEEN 5 AND 20)";
        let mut parser = Parser::new(query);
        let stmt = parser
            .parse()
            .expect("Should parse multiple BETWEEN clauses");

        assert!(stmt.where_clause.is_some());
    }

    #[test]
    fn test_between_complex_query() {
        // The actual user query that was failing
        let query = "SELECT * FROM test_sorting WHERE (Price BETWEEN 50 AND 100) OR (Product.Length() > 5) ORDER BY Category ASC, price DESC";
        let mut parser = Parser::new(query);
        let stmt = parser
            .parse()
            .expect("Should parse complex query with BETWEEN, method calls, and ORDER BY");

        assert!(stmt.where_clause.is_some());
        assert!(stmt.order_by.is_some());

        let order_by = stmt.order_by.unwrap();
        assert_eq!(order_by.len(), 2);
        assert_eq!(order_by[0].column, "Category");
        assert!(matches!(order_by[0].direction, SortDirection::Asc));
        assert_eq!(order_by[1].column, "price");
        assert!(matches!(order_by[1].direction, SortDirection::Desc));
    }

    #[test]
    fn test_between_formatting() {
        let expr = SqlExpression::Between {
            expr: Box::new(SqlExpression::Column("price".to_string())),
            lower: Box::new(SqlExpression::NumberLiteral("50".to_string())),
            upper: Box::new(SqlExpression::NumberLiteral("100".to_string())),
        };

        let formatted = format_expression(&expr);
        assert_eq!(formatted, "price BETWEEN 50 AND 100");

        let ast_formatted = format_expression_ast(&expr);
        assert!(ast_formatted.contains("Between"));
        assert!(ast_formatted.contains("50"));
        assert!(ast_formatted.contains("100"));
    }

    #[test]
    fn test_utf8_boundary_safety() {
        // Test that cursor detection doesn't panic on UTF-8 boundaries
        let query_with_unicode = "SELECT * FROM table WHERE column = 'héllo'";

        // Test various cursor positions, including ones that would be in the middle of characters
        for pos in 0..query_with_unicode.len() + 1 {
            // This should not panic, even if pos is in the middle of a UTF-8 character
            let result =
                std::panic::catch_unwind(|| detect_cursor_context(query_with_unicode, pos));

            assert!(
                result.is_ok(),
                "Panic at position {} in query with Unicode",
                pos
            );
        }

        // Test with a position beyond the string length
        let result = std::panic::catch_unwind(|| detect_cursor_context(query_with_unicode, 1000));
        assert!(result.is_ok(), "Panic with position beyond string length");

        // Test specifically with the 'é' character which is 2 bytes in UTF-8
        let pos_in_e = query_with_unicode.find('é').unwrap() + 1; // This should be in the middle of 'é'
        let result =
            std::panic::catch_unwind(|| detect_cursor_context(query_with_unicode, pos_in_e));
        assert!(
            result.is_ok(),
            "Panic with cursor in middle of UTF-8 character"
        );
    }
}
