//! SQL Lexer - Tokenization of SQL queries
//!
//! This module handles the conversion of raw SQL text into tokens
//! that can be consumed by the parser.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    With, // WITH clause for CTEs
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
    As,
    Asc,
    Desc,
    Limit,
    Offset,
    DateTime,  // DateTime constructor
    Case,      // CASE expression
    When,      // WHEN clause
    Then,      // THEN clause
    Else,      // ELSE clause
    End,       // END keyword
    Distinct,  // DISTINCT keyword for aggregate functions
    Over,      // OVER keyword for window functions
    Partition, // PARTITION keyword for window functions
    By,        // BY keyword (used with PARTITION BY, ORDER BY)

    // Window frame keywords
    Rows,      // ROWS frame type
    Range,     // RANGE frame type
    Unbounded, // UNBOUNDED for frame bounds
    Preceding, // PRECEDING for frame bounds
    Following, // FOLLOWING for frame bounds
    Current,   // CURRENT for CURRENT ROW
    Row,       // ROW for CURRENT ROW

    // Set operation keywords
    Union,     // UNION
    Intersect, // INTERSECT
    Except,    // EXCEPT

    // Special CTE keyword
    Web, // WEB (for WEB CTEs)

    // Row expansion functions
    Unnest, // UNNEST (for expanding delimited strings into rows)

    // JOIN keywords
    Join,  // JOIN keyword
    Inner, // INNER JOIN
    Left,  // LEFT JOIN
    Right, // RIGHT JOIN
    Full,  // FULL JOIN
    Outer, // OUTER keyword (LEFT OUTER, RIGHT OUTER, FULL OUTER)
    On,    // ON keyword for join conditions
    Cross, // CROSS JOIN

    // Literals
    Identifier(String),
    QuotedIdentifier(String), // For "Customer Id" style identifiers
    StringLiteral(String),
    JsonBlock(String), // For $JSON$...$ JSON$ delimited blocks
    NumberLiteral(String),
    Star,

    // Operators
    Dot,
    Comma,
    Colon,
    LeftParen,
    RightParen,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    // Arithmetic operators
    Plus,
    Minus,
    Divide,
    Modulo,

    // String operators
    Concat, // || for string concatenation

    // Special
    Eof,
}

impl Token {
    /// Check if a string is a SQL keyword and return corresponding token
    pub fn from_keyword(s: &str) -> Option<Token> {
        match s.to_uppercase().as_str() {
            "SELECT" => Some(Token::Select),
            "FROM" => Some(Token::From),
            "WHERE" => Some(Token::Where),
            "WITH" => Some(Token::With),
            "AND" => Some(Token::And),
            "OR" => Some(Token::Or),
            "IN" => Some(Token::In),
            "NOT" => Some(Token::Not),
            "BETWEEN" => Some(Token::Between),
            "LIKE" => Some(Token::Like),
            "IS" => Some(Token::Is),
            "NULL" => Some(Token::Null),
            "ORDER" => Some(Token::OrderBy),
            "GROUP" => Some(Token::GroupBy),
            "HAVING" => Some(Token::Having),
            "AS" => Some(Token::As),
            "ASC" => Some(Token::Asc),
            "DESC" => Some(Token::Desc),
            "LIMIT" => Some(Token::Limit),
            "OFFSET" => Some(Token::Offset),
            "DISTINCT" => Some(Token::Distinct),
            "CASE" => Some(Token::Case),
            "WHEN" => Some(Token::When),
            "THEN" => Some(Token::Then),
            "ELSE" => Some(Token::Else),
            "END" => Some(Token::End),
            "OVER" => Some(Token::Over),
            "PARTITION" => Some(Token::Partition),
            "BY" => Some(Token::By),
            "ROWS" => Some(Token::Rows),
            "RANGE" => Some(Token::Range),
            "UNBOUNDED" => Some(Token::Unbounded),
            "PRECEDING" => Some(Token::Preceding),
            "FOLLOWING" => Some(Token::Following),
            "CURRENT" => Some(Token::Current),
            "ROW" => Some(Token::Row),
            "UNION" => Some(Token::Union),
            "INTERSECT" => Some(Token::Intersect),
            "EXCEPT" => Some(Token::Except),
            "WEB" => Some(Token::Web),
            "UNNEST" => Some(Token::Unnest),
            "JOIN" => Some(Token::Join),
            "INNER" => Some(Token::Inner),
            "LEFT" => Some(Token::Left),
            "RIGHT" => Some(Token::Right),
            "FULL" => Some(Token::Full),
            "OUTER" => Some(Token::Outer),
            "ON" => Some(Token::On),
            "CROSS" => Some(Token::Cross),
            _ => None,
        }
    }

    /// Check if token is a logical operator
    pub fn is_logical_operator(&self) -> bool {
        matches!(self, Token::And | Token::Or)
    }

    /// Check if token is a join type
    pub fn is_join_type(&self) -> bool {
        matches!(
            self,
            Token::Inner | Token::Left | Token::Right | Token::Full | Token::Cross
        )
    }

    /// Check if token ends a clause
    pub fn is_clause_terminator(&self) -> bool {
        matches!(
            self,
            Token::OrderBy
                | Token::GroupBy
                | Token::Having
                | Token::Limit
                | Token::Offset
                | Token::Union
                | Token::Intersect
                | Token::Except
        )
    }

    /// Get the string representation of a keyword token
    pub fn as_keyword_str(&self) -> Option<&'static str> {
        match self {
            Token::Select => Some("SELECT"),
            Token::From => Some("FROM"),
            Token::Where => Some("WHERE"),
            Token::With => Some("WITH"),
            Token::And => Some("AND"),
            Token::Or => Some("OR"),
            Token::OrderBy => Some("ORDER BY"),
            Token::GroupBy => Some("GROUP BY"),
            Token::Having => Some("HAVING"),
            // Add more as needed
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    #[must_use]
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current = chars.first().copied();
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

    /// Peek ahead n characters and return as a string
    fn peek_string(&self, n: usize) -> String {
        let mut result = String::new();
        for i in 0..n {
            if let Some(ch) = self.input.get(self.position + i) {
                result.push(*ch);
            } else {
                break;
            }
        }
        result
    }

    /// Read a JSON block delimited by $JSON$...$JSON$
    /// Consumes the opening delimiter and reads until closing $JSON$
    fn read_json_block(&mut self) -> String {
        let mut result = String::new();

        // Skip opening $JSON$
        for _ in 0..6 {
            self.advance();
        }

        // Read until we find closing $JSON$
        while let Some(ch) = self.current_char {
            // Check if we're at the closing delimiter
            if ch == '$' && self.peek_string(6) == "$JSON$" {
                // Skip closing $JSON$
                for _ in 0..6 {
                    self.advance();
                }
                break;
            }
            result.push(ch);
            self.advance();
        }

        result
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

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(ch) = self.current_char {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Check for comments
            match self.current_char {
                Some('-') if self.peek(1) == Some('-') => {
                    // Single-line comment: skip until end of line
                    self.advance(); // skip first '-'
                    self.advance(); // skip second '-'
                    while let Some(ch) = self.current_char {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                Some('/') if self.peek(1) == Some('*') => {
                    // Multi-line comment: skip until */
                    self.advance(); // skip '/'
                    self.advance(); // skip '*'
                    while let Some(ch) = self.current_char {
                        if ch == '*' && self.peek(1) == Some('/') {
                            self.advance(); // skip '*'
                            self.advance(); // skip '/'
                            break;
                        }
                        self.advance();
                    }
                }
                _ => {
                    // No more comments or whitespace
                    break;
                }
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
        let has_e = false;

        // Read the main number part (including decimal point)
        while let Some(ch) = self.current_char {
            if !has_e && (ch.is_numeric() || ch == '.') {
                result.push(ch);
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !has_e && !result.is_empty() {
                // Handle scientific notation
                result.push(ch);
                self.advance();
                let _ = has_e; // We don't allow multiple 'e' characters, so break after this

                // Check for optional sign after 'e'
                if let Some(sign) = self.current_char {
                    if sign == '+' || sign == '-' {
                        result.push(sign);
                        self.advance();
                    }
                }

                // Read exponent digits
                while let Some(digit) = self.current_char {
                    if digit.is_numeric() {
                        result.push(digit);
                        self.advance();
                    } else {
                        break;
                    }
                }
                break; // Done reading the number
            } else {
                break;
            }
        }
        result
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        match self.current_char {
            None => Token::Eof,
            Some('*') => {
                self.advance();
                // Context-sensitive: could be SELECT * or multiplication
                // The parser will distinguish based on context
                Token::Star // We'll handle multiplication in parser
            }
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('/') => {
                // Check if this is a comment start
                if self.peek(1) == Some('*') {
                    // This shouldn't happen as comments are skipped above,
                    // but handle it just in case
                    self.skip_whitespace_and_comments();
                    return self.next_token();
                }
                self.advance();
                Token::Divide
            }
            Some('%') => {
                self.advance();
                Token::Modulo
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(':') => {
                self.advance();
                Token::Colon
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
            Some('|') if self.peek(1) == Some('|') => {
                self.advance();
                self.advance();
                Token::Concat
            }
            Some('"') => {
                // Double quotes = identifier
                let ident_val = self.read_string();
                Token::QuotedIdentifier(ident_val)
            }
            Some('$') => {
                // Check if this is $JSON$ delimiter
                if self.peek_string(6) == "$JSON$" {
                    let json_content = self.read_json_block();
                    Token::JsonBlock(json_content)
                } else {
                    // Not a JSON block, could be part of identifier or parameter
                    // For now, treat as identifier start
                    let ident = self.read_identifier();
                    Token::Identifier(ident)
                }
            }
            Some('\'') => {
                // Single quotes = string literal
                let string_val = self.read_string();
                Token::StringLiteral(string_val)
            }
            Some('-') if self.peek(1) == Some('-') => {
                // This is a comment, skip it and get next token
                self.skip_whitespace_and_comments();
                self.next_token()
            }
            Some('-') if self.peek(1).is_some_and(char::is_numeric) => {
                // Handle negative numbers
                self.advance(); // skip '-'
                let num = self.read_number();
                Token::NumberLiteral(format!("-{num}"))
            }
            Some('-') => {
                // Handle subtraction operator
                self.advance();
                Token::Minus
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
                    "WITH" => Token::With,
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
                    "AS" => Token::As,
                    "ASC" => Token::Asc,
                    "DESC" => Token::Desc,
                    "LIMIT" => Token::Limit,
                    "OFFSET" => Token::Offset,
                    "DATETIME" => Token::DateTime,
                    "CASE" => Token::Case,
                    "WHEN" => Token::When,
                    "THEN" => Token::Then,
                    "ELSE" => Token::Else,
                    "END" => Token::End,
                    "DISTINCT" => Token::Distinct,
                    "OVER" => Token::Over,
                    "PARTITION" => Token::Partition,
                    "BY" => Token::By,
                    // Window frame keywords
                    "ROWS" => Token::Rows,
                    // Note: RANGE is context-sensitive - it's both a window frame keyword and a table function
                    // We'll handle this in the parser based on context
                    "UNBOUNDED" => Token::Unbounded,
                    "PRECEDING" => Token::Preceding,
                    "FOLLOWING" => Token::Following,
                    "CURRENT" => Token::Current,
                    "ROW" => Token::Row,
                    // Set operation keywords
                    "UNION" => Token::Union,
                    "INTERSECT" => Token::Intersect,
                    "EXCEPT" => Token::Except,
                    // Special CTE keyword
                    "WEB" => Token::Web,
                    // Row expansion functions
                    "UNNEST" => Token::Unnest,
                    // JOIN keywords
                    "JOIN" => Token::Join,
                    "INNER" => Token::Inner,
                    "LEFT" => Token::Left,
                    "RIGHT" => Token::Right,
                    "FULL" => Token::Full,
                    "OUTER" => Token::Outer,
                    "ON" => Token::On,
                    "CROSS" => Token::Cross,
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

        self.skip_whitespace_and_comments();
        let next_word = self.read_identifier();
        let matches = next_word.to_uppercase() == keyword;

        // Restore position
        self.position = saved_pos;
        self.current_char = saved_char;

        matches
    }

    #[must_use]
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
            self.skip_whitespace_and_comments();
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
