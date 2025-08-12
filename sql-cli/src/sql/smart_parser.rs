use crate::parser::{ParseState, Schema};

#[derive(Debug, Clone)]
pub struct SmartSqlParser {
    schema: Schema,
}

#[derive(Debug, Clone)]
pub struct ParseContext {
    pub cursor_position: usize,
    pub tokens_before_cursor: Vec<SqlToken>,
    pub partial_token_at_cursor: Option<String>,
    pub tokens_after_cursor: Vec<SqlToken>,
    pub current_state: ParseState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlToken {
    Keyword(String),    // SELECT, FROM, WHERE, etc.
    Identifier(String), // column names, table names
    Operator(String),   // =, >, <, etc.
    String(String),     // 'quoted strings'
    Number(String),     // 123, 45.67
    Comma,
    Incomplete(String), // partial token at cursor
}

impl SmartSqlParser {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
        }
    }

    pub fn get_completion_suggestions(&self, query: &str, cursor_pos: usize) -> Vec<String> {
        let context = self.parse_with_cursor(query, cursor_pos);

        match context.current_state {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => self.get_column_suggestions(&context),
            ParseState::InColumnList => self.get_column_or_from_suggestions(&context),
            ParseState::AfterFrom => self.get_table_suggestions(&context),
            ParseState::AfterTable => vec!["WHERE".to_string(), "ORDER BY".to_string()],
            ParseState::InWhere => self.get_where_suggestions(&context),
            ParseState::InOrderBy => self.get_orderby_suggestions(&context),
            _ => vec![],
        }
    }

    fn parse_with_cursor(&self, query: &str, cursor_pos: usize) -> ParseContext {
        let cursor_pos = cursor_pos.min(query.len());

        // Split query at cursor
        let before_cursor = &query[..cursor_pos];
        let after_cursor = &query[cursor_pos..];

        // Tokenize the parts
        let tokens_before = self.tokenize(before_cursor);
        let tokens_after = self.tokenize(after_cursor);

        // Find partial token at cursor
        let partial_token = self.extract_partial_token_at_cursor(query, cursor_pos);

        // Determine current parse state
        let state = self.determine_parse_state(&tokens_before, &partial_token);

        ParseContext {
            cursor_position: cursor_pos,
            tokens_before_cursor: tokens_before,
            partial_token_at_cursor: partial_token,
            tokens_after_cursor: tokens_after,
            current_state: state,
        }
    }

    fn tokenize(&self, text: &str) -> Vec<SqlToken> {
        let mut tokens = Vec::new();
        let mut chars = text.char_indices().peekable();
        let mut current_token = String::new();

        while let Some((_i, ch)) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    if !current_token.is_empty() {
                        tokens.push(self.classify_token(&current_token));
                        current_token.clear();
                    }
                }
                ',' => {
                    if !current_token.is_empty() {
                        tokens.push(self.classify_token(&current_token));
                        current_token.clear();
                    }
                    tokens.push(SqlToken::Comma);
                }
                '\'' => {
                    // Handle quoted strings
                    let mut string_content = String::new();
                    while let Some((_, next_ch)) = chars.next() {
                        if next_ch == '\'' {
                            break;
                        }
                        string_content.push(next_ch);
                    }
                    tokens.push(SqlToken::String(string_content));
                }
                '=' | '>' | '<' | '!' => {
                    if !current_token.is_empty() {
                        tokens.push(self.classify_token(&current_token));
                        current_token.clear();
                    }

                    let mut operator = ch.to_string();
                    if let Some((_, '=')) = chars.peek() {
                        chars.next();
                        operator.push('=');
                    }
                    tokens.push(SqlToken::Operator(operator));
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(self.classify_token(&current_token));
        }

        tokens
    }

    fn classify_token(&self, token: &str) -> SqlToken {
        let upper_token = token.to_uppercase();
        match upper_token.as_str() {
            "SELECT" | "FROM" | "WHERE" | "ORDER" | "BY" | "AND" | "OR" | "GROUP" | "HAVING"
            | "LIMIT" | "OFFSET" | "ASC" | "DESC" => SqlToken::Keyword(upper_token),
            _ => {
                if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    SqlToken::Number(token.to_string())
                } else {
                    SqlToken::Identifier(token.to_string())
                }
            }
        }
    }

    fn extract_partial_token_at_cursor(&self, query: &str, cursor_pos: usize) -> Option<String> {
        if cursor_pos == 0 || cursor_pos > query.len() {
            return None;
        }

        let chars: Vec<char> = query.chars().collect();

        // Find start of current word
        let mut start = cursor_pos;
        while start > 0 && chars[start - 1].is_alphanumeric() {
            start -= 1;
        }

        // Find end of current word
        let mut end = cursor_pos;
        while end < chars.len() && chars[end].is_alphanumeric() {
            end += 1;
        }

        if start < end {
            let partial: String = chars[start..cursor_pos].iter().collect();
            if !partial.is_empty() {
                Some(partial)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn determine_parse_state(
        &self,
        tokens: &[SqlToken],
        partial_token: &Option<String>,
    ) -> ParseState {
        if tokens.is_empty() && partial_token.is_none() {
            return ParseState::Start;
        }

        let mut state = ParseState::Start;
        let mut i = 0;

        while i < tokens.len() {
            match &tokens[i] {
                SqlToken::Keyword(kw) if kw == "SELECT" => {
                    state = ParseState::AfterSelect;
                }
                SqlToken::Keyword(kw) if kw == "FROM" => {
                    state = ParseState::AfterFrom;
                }
                SqlToken::Keyword(kw) if kw == "WHERE" => {
                    state = ParseState::InWhere;
                }
                SqlToken::Keyword(kw) if kw == "ORDER" => {
                    // Check if next token is "BY"
                    if i + 1 < tokens.len() {
                        if let SqlToken::Keyword(next_kw) = &tokens[i + 1] {
                            if next_kw == "BY" {
                                state = ParseState::InOrderBy;
                                i += 1; // Skip the "BY" token
                            }
                        }
                    }
                }
                SqlToken::Identifier(_) => match state {
                    ParseState::AfterSelect => state = ParseState::InColumnList,
                    ParseState::AfterFrom => state = ParseState::AfterTable,
                    _ => {}
                },
                SqlToken::Comma => match state {
                    ParseState::InColumnList => state = ParseState::InColumnList,
                    _ => {}
                },
                _ => {}
            }
            i += 1;
        }

        state
    }

    fn get_column_suggestions(&self, context: &ParseContext) -> Vec<String> {
        let mut columns = self.schema.get_columns("trade_deal");
        columns.push("*".to_string());

        self.filter_suggestions(columns, &context.partial_token_at_cursor)
    }

    fn get_column_or_from_suggestions(&self, context: &ParseContext) -> Vec<String> {
        let mut suggestions = self.schema.get_columns("trade_deal");
        suggestions.push("FROM".to_string());

        self.filter_suggestions(suggestions, &context.partial_token_at_cursor)
    }

    fn get_table_suggestions(&self, context: &ParseContext) -> Vec<String> {
        let tables = vec!["trade_deal".to_string(), "instrument".to_string()];
        self.filter_suggestions(tables, &context.partial_token_at_cursor)
    }

    fn get_where_suggestions(&self, context: &ParseContext) -> Vec<String> {
        let mut suggestions = self.schema.get_columns("trade_deal");
        suggestions.extend(vec![
            "AND".to_string(),
            "OR".to_string(),
            "ORDER BY".to_string(),
        ]);

        self.filter_suggestions(suggestions, &context.partial_token_at_cursor)
    }

    fn get_orderby_suggestions(&self, context: &ParseContext) -> Vec<String> {
        let mut suggestions = self.schema.get_columns("trade_deal");
        suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);

        self.filter_suggestions(suggestions, &context.partial_token_at_cursor)
    }

    fn filter_suggestions(
        &self,
        suggestions: Vec<String>,
        partial: &Option<String>,
    ) -> Vec<String> {
        if let Some(partial_text) = partial {
            suggestions
                .into_iter()
                .filter(|s| s.to_lowercase().starts_with(&partial_text.to_lowercase()))
                .collect()
        } else {
            suggestions
        }
    }
}
