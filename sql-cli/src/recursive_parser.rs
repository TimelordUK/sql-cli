use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    And,
    Or,
    In,
    OrderBy,
    GroupBy,
    Having,
    
    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    Star,
    
    // Operators
    Dot,
    Comma,
    LeftParen,
    RightParen,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    
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
                Token::Equals
            }
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::LessThanEquals
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::NotEquals
                } else {
                    Token::LessThan
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::GreaterThanEquals
                } else {
                    Token::GreaterThan
                }
            }
            Some('!') if self.peek(1) == Some('=') => {
                self.advance();
                self.advance();
                Token::NotEquals
            }
            Some('"') | Some('\'') => {
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
}

// AST Nodes
#[derive(Debug, Clone)]
pub enum SqlExpression {
    Column(String),
    StringLiteral(String),
    NumberLiteral(String),
    MethodCall {
        object: String,
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

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub columns: Vec<String>,
    pub from_table: Option<String>,
    pub where_clause: Option<WhereClause>,
    pub order_by: Option<Vec<String>>,
    pub group_by: Option<Vec<String>>,
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }
    
    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.current_token = self.lexer.next_token();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token))
        }
    }
    
    fn advance(&mut self) {
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
            if let Token::Identifier(table) = &self.current_token {
                let table_name = table.clone();
                self.advance();
                Some(table_name)
            } else {
                return Err("Expected table name after FROM".to_string());
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
            Some(self.parse_identifier_list()?)
        } else {
            None
        };
        
        let group_by = if matches!(self.current_token, Token::GroupBy) {
            self.advance();
            Some(self.parse_identifier_list()?)
        } else {
            None
        };
        
        Ok(SelectStatement {
            columns,
            from_table,
            where_clause,
            order_by,
            group_by,
        })
    }
    
    fn parse_select_list(&mut self) -> Result<Vec<String>, String> {
        let mut columns = Vec::new();
        
        if matches!(self.current_token, Token::Star) {
            columns.push("*".to_string());
            self.advance();
        } else {
            loop {
                if let Token::Identifier(col) = &self.current_token {
                    columns.push(col.clone());
                    self.advance();
                } else {
                    return Err("Expected column name".to_string());
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
            if let Token::Identifier(id) = &self.current_token {
                identifiers.push(id.clone());
                self.advance();
            } else {
                return Err("Expected identifier".to_string());
            }
            
            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(identifiers)
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
                _ => None,
            };
            
            conditions.push(Condition { expr, connector: connector.clone() });
            
            if connector.is_none() {
                break;
            }
        }
        
        Ok(WhereClause { conditions })
    }
    
    fn parse_expression(&mut self) -> Result<SqlExpression, String> {
        let mut left = self.parse_primary()?;
        
        // Handle method calls
        if matches!(self.current_token, Token::Dot) {
            self.advance();
            if let Token::Identifier(method) = &self.current_token {
                let method_name = method.clone();
                self.advance();
                
                if matches!(self.current_token, Token::LeftParen) {
                    self.advance();
                    let args = self.parse_method_args()?;
                    self.consume(Token::RightParen)?;
                    
                    if let SqlExpression::Column(obj) = left {
                        left = SqlExpression::MethodCall {
                            object: obj,
                            method: method_name,
                            args,
                        };
                    }
                }
            }
        }
        
        // Handle binary operators
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
        
        Ok(left)
    }
    
    fn parse_primary(&mut self) -> Result<SqlExpression, String> {
        match &self.current_token {
            Token::Identifier(id) => {
                let expr = SqlExpression::Column(id.clone());
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
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }
    
    fn parse_method_args(&mut self) -> Result<Vec<SqlExpression>, String> {
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
            Token::Equals => Some("=".to_string()),
            Token::NotEquals => Some("!=".to_string()),
            Token::LessThan => Some("<".to_string()),
            Token::GreaterThan => Some(">".to_string()),
            Token::LessThanEquals => Some("<=".to_string()),
            Token::GreaterThanEquals => Some(">=".to_string()),
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
    AfterColumn(String),
    AfterLogicalOp(LogicalOp),
    InMethodCall(String, String), // object, method
    InExpression,
    Unknown,
}

pub fn detect_cursor_context(query: &str, cursor_pos: usize) -> (CursorContext, Option<String>) {
    let truncated = &query[..cursor_pos];
    let mut parser = Parser::new(truncated);
    
    // Try to parse as much as possible
    match parser.parse() {
        Ok(stmt) => {
            let (ctx, partial) = analyze_statement(&stmt, truncated, cursor_pos);
            #[cfg(test)]
            println!("analyze_statement returned: {:?}, {:?} for query: '{}'", ctx, partial, truncated);
            (ctx, partial)
        },
        Err(_) => {
            // Partial parse - analyze what we have
            let (ctx, partial) = analyze_partial(truncated, cursor_pos);
            #[cfg(test)]
            println!("analyze_partial returned: {:?}, {:?} for query: '{}'", ctx, partial, truncated);
            (ctx, partial)
        }
    }
}

pub fn tokenize_query(query: &str) -> Vec<String> {
    let mut lexer = Lexer::new(query);
    let tokens = lexer.tokenize_all();
    tokens.iter().map(|t| format!("{:?}", t)).collect()
}

fn analyze_statement(stmt: &SelectStatement, query: &str, _cursor_pos: usize) -> (CursorContext, Option<String>) {
    // First check for method call context (e.g., "columnName." or "columnName.Con")
    let trimmed = query.trim();
    
    // First check if we're after AND/OR - this takes precedence
    if trimmed.to_uppercase().ends_with(" AND") || trimmed.to_uppercase().ends_with(" OR") ||
       trimmed.to_uppercase().ends_with(" AND ") || trimmed.to_uppercase().ends_with(" OR ") {
        // Don't check for method context if we're clearly after a logical operator
    } else {
        // Look for the last dot in the query
        if let Some(dot_pos) = trimmed.rfind('.') {
            // Check if we're after a column name and dot
            let before_dot = &trimmed[..dot_pos];
            let after_dot = &trimmed[dot_pos + 1..];
            
            // Check if the part after dot looks like an incomplete method call
            // (not a complete method call like "Contains(...)")
            if !after_dot.contains('(') {
                if let Some(col_name) = before_dot.split_whitespace().last() {
                    if col_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        // We're in a method call context
                        // Check if there's a partial method name after the dot
                        let partial_method = if after_dot.is_empty() {
                            None
                        } else if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            Some(after_dot.to_string())
                        } else {
                            None
                        };
                        
                        return (CursorContext::AfterColumn(col_name.to_string()), partial_method);
                    }
                }
            }
        }
    }
    
    // Check if we're in WHERE clause
    if let Some(where_clause) = &stmt.where_clause {
        if let Some(last_condition) = where_clause.conditions.last() {
            if let Some(connector) = &last_condition.connector {
                // We're after AND/OR
                return (CursorContext::AfterLogicalOp(connector.clone()), extract_partial_at_end(query));
            }
        }
        // We're in WHERE clause but not after AND/OR
        return (CursorContext::WhereClause, extract_partial_at_end(query));
    }
    
    // Check other contexts based on what's in the statement
    if stmt.from_table.is_some() && stmt.where_clause.is_none() {
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
    
    // First check if we're after AND/OR - this takes precedence
    if trimmed.to_uppercase().ends_with(" AND") || trimmed.to_uppercase().ends_with(" OR") ||
       trimmed.to_uppercase().ends_with(" AND ") || trimmed.to_uppercase().ends_with(" OR ") {
        // Don't check for method context if we're clearly after a logical operator
    } else {
        // Look for the last dot in the query
        if let Some(dot_pos) = trimmed.rfind('.') {
            // Check if we're after a column name and dot
            let before_dot = &trimmed[..dot_pos];
            let after_dot = &trimmed[dot_pos + 1..];
            
            // Check if the part after dot looks like an incomplete method call
            // (not a complete method call like "Contains(...)")
            if !after_dot.contains('(') {
                if let Some(col_name) = before_dot.split_whitespace().last() {
                    if col_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        // We're in a method call context
                        // Check if there's a partial method name after the dot
                        let partial_method = if after_dot.is_empty() {
                            None
                        } else if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            Some(after_dot.to_string())
                        } else {
                            None
                        };
                        
                        return (CursorContext::AfterColumn(col_name.to_string()), partial_method);
                    }
                }
            }
        }
    }
    
    // Simple heuristics for partial queries
    if let Some(and_pos) = upper.rfind(" AND ") {
        if and_pos + 5 >= cursor_pos {
            return (CursorContext::AfterLogicalOp(LogicalOp::And), extract_partial_at_end(query));
        }
    }
    
    if let Some(or_pos) = upper.rfind(" OR ") {
        if or_pos + 4 >= cursor_pos {
            return (CursorContext::AfterLogicalOp(LogicalOp::Or), extract_partial_at_end(query));
        }
    }
    
    if upper.contains("WHERE") && !upper.contains("ORDER") && !upper.contains("GROUP") {
        return (CursorContext::WhereClause, extract_partial_at_end(query));
    }
    
    if upper.contains("FROM") && !upper.contains("WHERE") {
        return (CursorContext::FromClause, extract_partial_at_end(query));
    }
    
    if upper.contains("SELECT") && !upper.contains("FROM") {
        return (CursorContext::SelectClause, extract_partial_at_end(query));
    }
    
    (CursorContext::Unknown, None)
}

fn extract_partial_at_end(query: &str) -> Option<String> {
    let trimmed = query.trim();
    let last_word = trimmed.split_whitespace().last()?;
    
    // Check if it's a partial identifier (not a keyword or operator)
    if last_word.chars().all(|c| c.is_alphanumeric() || c == '_') &&
       !is_sql_keyword(last_word) {
        Some(last_word.to_string())
    } else {
        None
    }
}

fn is_sql_keyword(word: &str) -> bool {
    matches!(word.to_uppercase().as_str(),
        "SELECT" | "FROM" | "WHERE" | "AND" | "OR" | "IN" | 
        "ORDER" | "BY" | "GROUP" | "HAVING" | "ASC" | "DESC"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    fn test_cursor_context_after_and() {
        let query = "SELECT * FROM trade_deal WHERE status = 'active' AND ";
        let (context, partial) = detect_cursor_context(query, query.len());
        
        assert!(matches!(context, CursorContext::AfterLogicalOp(LogicalOp::And)));
        assert_eq!(partial, None);
    }
    
    #[test]
    fn test_cursor_context_with_partial() {
        let query = "SELECT * FROM trade_deal WHERE status = 'active' AND p";
        let (context, partial) = detect_cursor_context(query, query.len());
        
        assert!(matches!(context, CursorContext::AfterLogicalOp(LogicalOp::And)));
        assert_eq!(partial, Some("p".to_string()));
    }
}