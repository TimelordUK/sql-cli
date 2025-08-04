use crate::parser::{Schema, ParseState};

#[derive(Debug, Clone)]
pub struct CursorAwareParser {
    schema: Schema,
}

#[derive(Debug)]
pub struct ParseResult {
    pub suggestions: Vec<String>,
    pub context: String,
    pub partial_word: Option<String>,
}

impl CursorAwareParser {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
        }
    }
    
    pub fn get_completions(&self, query: &str, cursor_pos: usize) -> ParseResult {
        // Extract the word being typed at cursor position
        let partial_word = self.extract_word_at_cursor(query, cursor_pos);
        
        // Parse the query up to the cursor position
        let query_before_cursor = &query[..cursor_pos.min(query.len())];
        let context = self.determine_context(query_before_cursor);
        
        let suggestions = self.get_suggestions_for_context(&context, &partial_word);
        
        ParseResult {
            suggestions,
            context: format!("{:?}", context),
            partial_word,
        }
    }
    
    fn extract_word_at_cursor(&self, query: &str, cursor_pos: usize) -> Option<String> {
        if cursor_pos == 0 || cursor_pos > query.len() {
            return None;
        }
        
        let chars: Vec<char> = query.chars().collect();
        
        // Find word boundaries around cursor
        let mut start = cursor_pos;
        let mut end = cursor_pos;
        
        // Move start backward to beginning of word
        while start > 0 && Self::is_word_char(chars.get(start - 1).copied().unwrap_or(' ')) {
            start -= 1;
        }
        
        // Move end forward to end of word
        while end < chars.len() && Self::is_word_char(chars.get(end).copied().unwrap_or(' ')) {
            end += 1;
        }
        
        if start < cursor_pos {
            // Extract partial word up to cursor
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
    
    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }
    
    fn determine_context(&self, query_before_cursor: &str) -> ParseState {
        let query_upper = query_before_cursor.to_uppercase();
        let words: Vec<&str> = query_upper.split_whitespace().collect();
        
        if words.is_empty() {
            return ParseState::Start;
        }
        
        // Find the last complete SQL keyword
        let mut last_keyword_idx = None;
        let mut last_keyword = "";
        
        for (i, word) in words.iter().enumerate() {
            match *word {
                "SELECT" => {
                    last_keyword_idx = Some(i);
                    last_keyword = "SELECT";
                }
                "FROM" => {
                    last_keyword_idx = Some(i);
                    last_keyword = "FROM";
                }
                "WHERE" => {
                    last_keyword_idx = Some(i);
                    last_keyword = "WHERE";
                }
                "ORDER" => {
                    // Check if followed by BY
                    if i + 1 < words.len() && words[i + 1] == "BY" {
                        last_keyword_idx = Some(i);
                        last_keyword = "ORDER BY";
                    }
                }
                _ => {}
            }
        }
        
        match last_keyword {
            "SELECT" => {
                if let Some(idx) = last_keyword_idx {
                    // Count tokens after SELECT
                    let tokens_after_select = words.len() - idx - 1;
                    if tokens_after_select == 0 {
                        ParseState::AfterSelect
                    } else {
                        // Check if we've seen FROM yet
                        if words[(idx + 1)..].contains(&"FROM") {
                            ParseState::AfterTable // We're past the FROM clause
                        } else {
                            ParseState::InColumnList
                        }
                    }
                } else {
                    ParseState::AfterSelect
                }
            }
            "FROM" => {
                if let Some(idx) = last_keyword_idx {
                    let tokens_after_from = words.len() - idx - 1;
                    if tokens_after_from == 0 {
                        ParseState::AfterFrom
                    } else {
                        ParseState::AfterTable
                    }
                } else {
                    ParseState::AfterFrom
                }
            }
            "WHERE" => ParseState::InWhere,
            "ORDER BY" => ParseState::InOrderBy,
            _ => {
                // No clear keyword found, try to infer from context
                if query_upper.contains("SELECT") && query_upper.contains("FROM") && query_upper.contains("WHERE") {
                    ParseState::InWhere
                } else if query_upper.contains("SELECT") && query_upper.contains("FROM") {
                    ParseState::AfterTable
                } else if query_upper.contains("SELECT") {
                    ParseState::InColumnList
                } else {
                    ParseState::Start
                }
            }
        }
    }
    
    fn get_suggestions_for_context(&self, context: &ParseState, partial_word: &Option<String>) -> Vec<String> {
        let mut suggestions = match context {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => {
                let mut cols = self.schema.get_columns("trade_deal");
                cols.push("*".to_string());
                cols
            }
            ParseState::InColumnList => {
                let mut cols = self.schema.get_columns("trade_deal");
                cols.push("FROM".to_string());
                cols
            }
            ParseState::AfterFrom => {
                vec!["trade_deal".to_string(), "instrument".to_string()]
            }
            ParseState::AfterTable => {
                vec!["WHERE".to_string(), "ORDER BY".to_string()]
            }
            ParseState::InWhere => {
                let mut suggestions = self.schema.get_columns("trade_deal");
                suggestions.extend(vec![
                    "AND".to_string(),
                    "OR".to_string(),
                    "ORDER BY".to_string(),
                ]);
                suggestions
            }
            ParseState::InOrderBy => {
                let mut suggestions = self.schema.get_columns("trade_deal");
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                suggestions
            }
            _ => vec![],
        };
        
        // Filter by partial word if present
        if let Some(partial) = partial_word {
            suggestions.retain(|suggestion| {
                suggestion.to_lowercase().starts_with(&partial.to_lowercase())
            });
        }
        
        suggestions
    }
}