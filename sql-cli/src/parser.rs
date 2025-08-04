
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

#[derive(Debug, Clone)]
pub struct SqlParser {
    pub tokens: Vec<SqlToken>,
    pub current_state: ParseState,
}

impl SqlParser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current_state: ParseState::Start,
        }
    }

    pub fn parse_partial(&mut self, input: &str) -> Result<ParseState, String> {
        self.tokens.clear();
        self.current_state = ParseState::Start;
        
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(ParseState::Start);
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            match self.current_state {
                ParseState::Start => {
                    if word.eq_ignore_ascii_case("select") {
                        self.tokens.push(SqlToken::Select);
                        self.current_state = ParseState::AfterSelect;
                    }
                }
                ParseState::AfterSelect | ParseState::InColumnList => {
                    if word.eq_ignore_ascii_case("from") {
                        self.tokens.push(SqlToken::From);
                        self.current_state = ParseState::AfterFrom;
                    } else {
                        self.tokens.push(SqlToken::Column(word.to_string()));
                        self.current_state = ParseState::InColumnList;
                    }
                }
                ParseState::AfterFrom => {
                    self.tokens.push(SqlToken::Table(word.to_string()));
                    self.current_state = ParseState::AfterTable;
                }
                ParseState::AfterTable => {
                    if word.eq_ignore_ascii_case("where") {
                        self.tokens.push(SqlToken::Where);
                        self.current_state = ParseState::InWhere;
                    } else if word.eq_ignore_ascii_case("order") {
                        if i + 1 < words.len() && words[i + 1].eq_ignore_ascii_case("by") {
                            self.tokens.push(SqlToken::OrderBy);
                            self.current_state = ParseState::InOrderBy;
                        }
                    }
                }
                ParseState::InWhere => {
                    if word.eq_ignore_ascii_case("order") {
                        if i + 1 < words.len() && words[i + 1].eq_ignore_ascii_case("by") {
                            self.tokens.push(SqlToken::OrderBy);
                            self.current_state = ParseState::InOrderBy;
                        }
                    } else {
                        self.tokens.push(SqlToken::Identifier(word.to_string()));
                    }
                }
                ParseState::InOrderBy => {
                    self.tokens.push(SqlToken::Column(word.to_string()));
                }
                _ => {}
            }
        }
        
        Ok(self.current_state.clone())
    }

    pub fn get_completion_context(&mut self, partial_input: &str) -> CompletionContext {
        let _ = self.parse_partial(partial_input);
        
        CompletionContext {
            state: self.current_state.clone(),
            last_token: self.tokens.last().cloned(),
            partial_word: self.extract_partial_word(partial_input),
        }
    }
    
    fn extract_partial_word(&self, input: &str) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.ends_with(' ') {
            None
        } else {
            trimmed.split_whitespace().last().map(|s| s.to_string())
        }
    }
}

#[derive(Debug)]
pub struct CompletionContext {
    pub state: ParseState,
    pub last_token: Option<SqlToken>,
    pub partial_word: Option<String>,
}

impl CompletionContext {
    pub fn get_suggestions(&self, schema: &Schema) -> Vec<String> {
        match self.state {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => {
                let mut suggestions: Vec<String> = schema.get_columns("trade_deal")
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                suggestions.push("*".to_string());
                self.filter_suggestions(suggestions)
            }
            ParseState::InColumnList => {
                let mut suggestions: Vec<String> = schema.get_columns("trade_deal")
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                suggestions.push("FROM".to_string());
                self.filter_suggestions(suggestions)
            }
            ParseState::AfterFrom => {
                let suggestions = vec!["trade_deal".to_string(), "instrument".to_string()];
                self.filter_suggestions(suggestions)
            }
            ParseState::AfterTable => {
                let suggestions = vec!["WHERE".to_string(), "ORDER BY".to_string()];
                self.filter_suggestions(suggestions)
            }
            ParseState::InWhere => {
                let mut suggestions: Vec<String> = schema.get_columns("trade_deal")
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                suggestions.extend(vec![
                    "AND".to_string(),
                    "OR".to_string(),
                    "ORDER BY".to_string(),
                ]);
                self.filter_suggestions(suggestions)
            }
            ParseState::InOrderBy => {
                let mut suggestions: Vec<String> = schema.get_columns("trade_deal")
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                self.filter_suggestions(suggestions)
            }
            _ => vec![],
        }
    }
    
    fn filter_suggestions(&self, suggestions: Vec<String>) -> Vec<String> {
        if let Some(partial) = &self.partial_word {
            suggestions
                .into_iter()
                .filter(|s| s.to_lowercase().starts_with(&partial.to_lowercase()))
                .collect()
        } else {
            suggestions
        }
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

impl Schema {
    pub fn new() -> Self {
        let trade_deal_columns = vec![
            "dealId", "platformOrderId", "tradeDate", "settlementDate", 
            "instrumentId", "quantity", "price", "counterparty",
            "trader", "book", "strategy", "status", "currency",
        ];
        
        Self {
            tables: vec![
                TableInfo {
                    name: "trade_deal".to_string(),
                    columns: trade_deal_columns.iter().map(|&s| s.to_string()).collect(),
                },
                TableInfo {
                    name: "instrument".to_string(),
                    columns: vec!["instrumentId".to_string(), "name".to_string(), "type".to_string()],
                },
            ],
        }
    }
    
    pub fn get_columns(&self, table_name: &str) -> Vec<String> {
        self.tables
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(table_name))
            .map(|t| t.columns.clone())
            .unwrap_or_default()
    }
}