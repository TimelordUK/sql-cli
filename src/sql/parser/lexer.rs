//! SQL Lexer - Tokenization of SQL queries
//!
//! This module handles the conversion of raw SQL text into tokens
//! that can be consumed by the parser.

/// Lexer mode - controls whether comments are preserved or skipped
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerMode {
    /// Standard mode - skip comments (current default behavior)
    SkipComments,
    /// Preserve mode - tokenize comments as tokens
    PreserveComments,
}

impl Default for LexerMode {
    fn default() -> Self {
        LexerMode::SkipComments
    }
}

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
    ILike, // Case-insensitive LIKE (PostgreSQL)
    Is,
    Null,
    OrderBy,
    GroupBy,
    Having,
    Qualify,
    As,
    Asc,
    Desc,
    Limit,
    Offset,
    Into,      // INTO keyword for temporary tables
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
    Exclude,   // EXCLUDE keyword (for SELECT * EXCLUDE)
    // Note: REPLACE is NOT a keyword - it's handled as a function name
    // to avoid conflicting with the REPLACE() string function

    // PIVOT/UNPIVOT keywords
    Pivot,   // PIVOT keyword for row-to-column transformation
    Unpivot, // UNPIVOT keyword for column-to-row transformation
    For,     // FOR keyword (used in PIVOT: FOR column IN (...))

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

    // Special CTE keywords
    Web,  // WEB (for WEB CTEs)
    File, // FILE (for FILE CTEs — filesystem metadata)

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

    // Comments (preserved for formatting)
    LineComment(String),  // -- comment text (without the -- prefix)
    BlockComment(String), // /* comment text */ (without delimiters)

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
            "ILIKE" => Some(Token::ILike),
            "IS" => Some(Token::Is),
            "NULL" => Some(Token::Null),
            "ORDER" => Some(Token::OrderBy),
            "GROUP" => Some(Token::GroupBy),
            "HAVING" => Some(Token::Having),
            "QUALIFY" => Some(Token::Qualify),
            "AS" => Some(Token::As),
            "ASC" => Some(Token::Asc),
            "DESC" => Some(Token::Desc),
            "LIMIT" => Some(Token::Limit),
            "OFFSET" => Some(Token::Offset),
            "INTO" => Some(Token::Into),
            "DISTINCT" => Some(Token::Distinct),
            "EXCLUDE" => Some(Token::Exclude),
            "PIVOT" => Some(Token::Pivot),
            "UNPIVOT" => Some(Token::Unpivot),
            "FOR" => Some(Token::For),
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
            "FILE" => Some(Token::File),
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
    /// Returns the keyword as it would appear in SQL (uppercase)
    pub fn as_keyword_str(&self) -> Option<&'static str> {
        match self {
            Token::Select => Some("SELECT"),
            Token::From => Some("FROM"),
            Token::Where => Some("WHERE"),
            Token::With => Some("WITH"),
            Token::And => Some("AND"),
            Token::Or => Some("OR"),
            Token::In => Some("IN"),
            Token::Not => Some("NOT"),
            Token::Between => Some("BETWEEN"),
            Token::Like => Some("LIKE"),
            Token::ILike => Some("ILIKE"),
            Token::Is => Some("IS"),
            Token::Null => Some("NULL"),
            Token::OrderBy => Some("ORDER BY"),
            Token::GroupBy => Some("GROUP BY"),
            Token::Having => Some("HAVING"),
            Token::Qualify => Some("QUALIFY"),
            Token::As => Some("AS"),
            Token::Asc => Some("ASC"),
            Token::Desc => Some("DESC"),
            Token::Limit => Some("LIMIT"),
            Token::Offset => Some("OFFSET"),
            Token::Into => Some("INTO"),
            Token::Distinct => Some("DISTINCT"),
            Token::Exclude => Some("EXCLUDE"),
            Token::Pivot => Some("PIVOT"),
            Token::Unpivot => Some("UNPIVOT"),
            Token::For => Some("FOR"),
            Token::Case => Some("CASE"),
            Token::When => Some("WHEN"),
            Token::Then => Some("THEN"),
            Token::Else => Some("ELSE"),
            Token::End => Some("END"),
            Token::Join => Some("JOIN"),
            Token::Inner => Some("INNER"),
            Token::Left => Some("LEFT"),
            Token::Right => Some("RIGHT"),
            Token::Full => Some("FULL"),
            Token::Cross => Some("CROSS"),
            Token::On => Some("ON"),
            Token::Union => Some("UNION"),
            Token::Intersect => Some("INTERSECT"),
            Token::Except => Some("EXCEPT"),
            Token::Over => Some("OVER"),
            Token::Partition => Some("PARTITION"),
            Token::By => Some("BY"),
            Token::Rows => Some("ROWS"),
            Token::Range => Some("RANGE"),
            Token::Preceding => Some("PRECEDING"),
            Token::Following => Some("FOLLOWING"),
            Token::Current => Some("CURRENT"),
            Token::Row => Some("ROW"),
            Token::Unbounded => Some("UNBOUNDED"),
            Token::DateTime => Some("DATETIME"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    mode: LexerMode,
}

impl Lexer {
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self::with_mode(input, LexerMode::default())
    }

    /// Create a new lexer with specified mode
    #[must_use]
    pub fn with_mode(input: &str, mode: LexerMode) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current = chars.first().copied();
        Self {
            input: chars,
            position: 0,
            current_char: current,
            mode,
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

    /// Read a line comment and return its content (without the -- prefix)
    fn read_line_comment(&mut self) -> String {
        let mut result = String::new();

        // Skip '--'
        self.advance();
        self.advance();

        // Read until end of line or EOF
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                self.advance(); // consume the newline
                break;
            }
            result.push(ch);
            self.advance();
        }

        result
    }

    /// Read a block comment and return its content (without /* */ delimiters)
    fn read_block_comment(&mut self) -> String {
        let mut result = String::new();

        // Skip '/*'
        self.advance();
        self.advance();

        // Read until we find '*/'
        while let Some(ch) = self.current_char {
            if ch == '*' && self.peek(1) == Some('/') {
                self.advance(); // skip '*'
                self.advance(); // skip '/'
                break;
            }
            result.push(ch);
            self.advance();
        }

        result
    }

    /// Skip whitespace and comments (for backwards compatibility with parser)
    /// This is the old behavior that discards comments
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

    /// Get next token while preserving comments as tokens
    /// This is the new behavior for comment-aware formatting
    pub fn next_token_with_comments(&mut self) -> Token {
        // Only skip whitespace, NOT comments
        self.skip_whitespace();

        match self.current_char {
            None => Token::Eof,
            // Handle comments as tokens
            Some('-') if self.peek(1) == Some('-') => {
                let comment_text = self.read_line_comment();
                Token::LineComment(comment_text)
            }
            Some('/') if self.peek(1) == Some('*') => {
                let comment_text = self.read_block_comment();
                Token::BlockComment(comment_text)
            }
            Some('*') => {
                self.advance();
                Token::Star
            }
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('/') => {
                // Regular division (comment case handled above)
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
                let ident_val = self.read_string();
                Token::QuotedIdentifier(ident_val)
            }
            Some('$') => {
                if self.peek_string(6) == "$JSON$" {
                    let json_content = self.read_json_block();
                    Token::JsonBlock(json_content)
                } else {
                    let ident = self.read_identifier();
                    Token::Identifier(ident)
                }
            }
            Some('\'') => {
                let string_val = self.read_string();
                Token::StringLiteral(string_val)
            }
            Some('-') if self.peek(1).is_some_and(char::is_numeric) => {
                self.advance();
                let num = self.read_number();
                Token::NumberLiteral(format!("-{num}"))
            }
            Some('-') => {
                self.advance();
                Token::Minus
            }
            Some(ch) if ch.is_numeric() => {
                let num = self.read_number();
                Token::NumberLiteral(num)
            }
            Some('#') => {
                self.advance();
                let table_name = self.read_identifier();
                if table_name.is_empty() {
                    Token::Identifier("#".to_string())
                } else {
                    Token::Identifier(format!("#{}", table_name))
                }
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                // Handle multi-word keywords like GROUP BY and ORDER BY
                match ident.to_uppercase().as_str() {
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
                    _ => Token::from_keyword(&ident).unwrap_or_else(|| Token::Identifier(ident)),
                }
            }
            Some(ch) => {
                self.advance();
                Token::Identifier(ch.to_string())
            }
        }
    }

    /// Get next token - dispatches based on lexer mode
    pub fn next_token(&mut self) -> Token {
        match self.mode {
            LexerMode::SkipComments => self.next_token_skip_comments(),
            LexerMode::PreserveComments => self.next_token_with_comments(),
        }
    }

    /// Get next token skipping comments (original behavior)
    fn next_token_skip_comments(&mut self) -> Token {
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
            Some('#') => {
                // Temporary table identifier: #tablename
                self.advance(); // consume #
                let table_name = self.read_identifier();
                if table_name.is_empty() {
                    // Just # by itself
                    Token::Identifier("#".to_string())
                } else {
                    // #tablename
                    Token::Identifier(format!("#{}", table_name))
                }
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
                    "ILIKE" => Token::ILike,
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
                    "QUALIFY" => Token::Qualify,
                    "AS" => Token::As,
                    "ASC" => Token::Asc,
                    "DESC" => Token::Desc,
                    "LIMIT" => Token::Limit,
                    "OFFSET" => Token::Offset,
                    "INTO" => Token::Into,
                    "DATETIME" => Token::DateTime,
                    "CASE" => Token::Case,
                    "WHEN" => Token::When,
                    "THEN" => Token::Then,
                    "ELSE" => Token::Else,
                    "END" => Token::End,
                    "DISTINCT" => Token::Distinct,
                    "EXCLUDE" => Token::Exclude,
                    "PIVOT" => Token::Pivot,
                    "UNPIVOT" => Token::Unpivot,
                    "FOR" => Token::For,
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
                    // Special CTE keywords
                    "WEB" => Token::Web,
                    "FILE" => Token::File,
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

    /// Tokenize all tokens including comments
    /// This is useful for formatting tools that need to preserve comments
    pub fn tokenize_all_with_comments(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token_with_comments();
            if matches!(token, Token::Eof) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_comment_tokenization() {
        let sql = "SELECT col1, -- this is a comment\ncol2 FROM table";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize_all_with_comments();

        // Find the comment token
        let comment_token = tokens.iter().find(|t| matches!(t, Token::LineComment(_)));
        assert!(comment_token.is_some(), "Should find line comment token");

        if let Some(Token::LineComment(text)) = comment_token {
            assert_eq!(text.trim(), "this is a comment");
        }
    }

    #[test]
    fn test_block_comment_tokenization() {
        let sql = "SELECT /* block comment */ col1 FROM table";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize_all_with_comments();

        // Find the comment token
        let comment_token = tokens.iter().find(|t| matches!(t, Token::BlockComment(_)));
        assert!(comment_token.is_some(), "Should find block comment token");

        if let Some(Token::BlockComment(text)) = comment_token {
            assert_eq!(text.trim(), "block comment");
        }
    }

    #[test]
    fn test_multiple_comments() {
        let sql = "-- First comment\nSELECT col1, /* inline */ col2\n-- Second comment\nFROM table";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize_all_with_comments();

        let line_comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::LineComment(_)))
            .collect();
        let block_comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::BlockComment(_)))
            .collect();

        assert_eq!(line_comments.len(), 2, "Should find 2 line comments");
        assert_eq!(block_comments.len(), 1, "Should find 1 block comment");
    }

    #[test]
    fn test_backwards_compatibility() {
        // Test that next_token() still skips comments
        let sql = "SELECT -- comment\ncol1 FROM table";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize_all();

        // Should NOT contain any comment tokens
        let has_comments = tokens
            .iter()
            .any(|t| matches!(t, Token::LineComment(_) | Token::BlockComment(_)));
        assert!(
            !has_comments,
            "next_token() should skip comments for backwards compatibility"
        );

        // Should still parse correctly
        assert!(tokens.iter().any(|t| matches!(t, Token::Select)));
        assert!(tokens.iter().any(|t| matches!(t, Token::From)));
    }

    // ===== Dual-Mode Lexer Tests (Phase 1) =====

    #[test]
    fn test_lexer_mode_skip_comments() {
        let sql = "SELECT id -- comment\nFROM table";

        // SkipComments mode (default)
        let mut lexer = Lexer::with_mode(sql, LexerMode::SkipComments);

        assert_eq!(lexer.next_token(), Token::Select);
        assert_eq!(lexer.next_token(), Token::Identifier("id".into()));
        // Comment should be skipped
        assert_eq!(lexer.next_token(), Token::From);
        assert_eq!(lexer.next_token(), Token::Identifier("table".into()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_lexer_mode_preserve_comments() {
        let sql = "SELECT id -- comment\nFROM table";

        // PreserveComments mode
        let mut lexer = Lexer::with_mode(sql, LexerMode::PreserveComments);

        assert_eq!(lexer.next_token(), Token::Select);
        assert_eq!(lexer.next_token(), Token::Identifier("id".into()));

        // Comment should be preserved as a token
        let comment_tok = lexer.next_token();
        assert!(matches!(comment_tok, Token::LineComment(_)));
        if let Token::LineComment(text) = comment_tok {
            assert_eq!(text.trim(), "comment");
        }

        assert_eq!(lexer.next_token(), Token::From);
        assert_eq!(lexer.next_token(), Token::Identifier("table".into()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_lexer_mode_default_is_skip() {
        let sql = "SELECT id -- comment\nFROM table";

        // Default (using new()) should skip comments
        let mut lexer = Lexer::new(sql);

        let mut tok_count = 0;
        loop {
            let tok = lexer.next_token();
            if matches!(tok, Token::Eof) {
                break;
            }
            // Should never see a comment token
            assert!(!matches!(
                tok,
                Token::LineComment(_) | Token::BlockComment(_)
            ));
            tok_count += 1;
        }

        // SELECT, id, FROM, table = 4 tokens (no comment)
        assert_eq!(tok_count, 4);
    }

    #[test]
    fn test_lexer_mode_block_comments() {
        let sql = "SELECT /* block */ id FROM table";

        // Skip mode
        let mut lexer_skip = Lexer::with_mode(sql, LexerMode::SkipComments);
        assert_eq!(lexer_skip.next_token(), Token::Select);
        assert_eq!(lexer_skip.next_token(), Token::Identifier("id".into()));
        assert_eq!(lexer_skip.next_token(), Token::From);

        // Preserve mode
        let mut lexer_preserve = Lexer::with_mode(sql, LexerMode::PreserveComments);
        assert_eq!(lexer_preserve.next_token(), Token::Select);

        let comment_tok = lexer_preserve.next_token();
        assert!(matches!(comment_tok, Token::BlockComment(_)));
        if let Token::BlockComment(text) = comment_tok {
            assert_eq!(text.trim(), "block");
        }

        assert_eq!(lexer_preserve.next_token(), Token::Identifier("id".into()));
    }

    #[test]
    fn test_lexer_mode_mixed_comments() {
        let sql = "-- leading\nSELECT /* inline */ id -- trailing\nFROM table";

        let mut lexer = Lexer::with_mode(sql, LexerMode::PreserveComments);

        // leading comment
        assert!(matches!(lexer.next_token(), Token::LineComment(_)));

        // SELECT
        assert_eq!(lexer.next_token(), Token::Select);

        // inline block comment
        assert!(matches!(lexer.next_token(), Token::BlockComment(_)));

        // id
        assert_eq!(lexer.next_token(), Token::Identifier("id".into()));

        // trailing comment
        assert!(matches!(lexer.next_token(), Token::LineComment(_)));

        // FROM table
        assert_eq!(lexer.next_token(), Token::From);
        assert_eq!(lexer.next_token(), Token::Identifier("table".into()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_pivot_keywords() {
        let sql = "PIVOT (MAX(amount) FOR month IN (val1, val2)) UNPIVOT";
        let mut lexer = Lexer::new(sql);

        // Test individual token recognition
        assert_eq!(
            lexer.next_token(),
            Token::Pivot,
            "First token should be PIVOT"
        );
        assert_eq!(lexer.next_token(), Token::LeftParen);
        assert!(matches!(lexer.next_token(), Token::Identifier(_))); // MAX
        assert_eq!(lexer.next_token(), Token::LeftParen);
        assert!(matches!(lexer.next_token(), Token::Identifier(_))); // amount
        assert_eq!(lexer.next_token(), Token::RightParen);
        assert_eq!(lexer.next_token(), Token::For, "Should tokenize FOR");
        assert!(matches!(lexer.next_token(), Token::Identifier(_))); // month
        assert_eq!(lexer.next_token(), Token::In, "Should tokenize IN");
        assert_eq!(lexer.next_token(), Token::LeftParen);
        assert!(matches!(lexer.next_token(), Token::Identifier(_))); // val1
        assert_eq!(lexer.next_token(), Token::Comma);
        assert!(matches!(lexer.next_token(), Token::Identifier(_))); // val2
        assert_eq!(lexer.next_token(), Token::RightParen);
        assert_eq!(lexer.next_token(), Token::RightParen);
        assert_eq!(
            lexer.next_token(),
            Token::Unpivot,
            "Should tokenize UNPIVOT"
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
