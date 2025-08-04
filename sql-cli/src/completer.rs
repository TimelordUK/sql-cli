use reedline::{Completer, Span, Suggestion};
use std::sync::{Arc, Mutex};

use crate::parser::{SqlParser, Schema, ParseState};

pub struct SqlCompleter {
    parser: Arc<Mutex<SqlParser>>,
    schema: Schema,
}

impl SqlCompleter {
    pub fn new() -> Self {
        Self {
            parser: Arc::new(Mutex::new(SqlParser::new())),
            schema: Schema::new(),
        }
    }
}

impl Completer for SqlCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let input = &line[..pos];
        
        let mut parser = self.parser.lock().unwrap();
        let context = parser.get_completion_context(input);
        let suggestions = context.get_suggestions(&self.schema);
        
        let start_pos = if let Some(partial) = &context.partial_word {
            pos.saturating_sub(partial.len())
        } else {
            pos
        };
        
        suggestions
            .into_iter()
            .map(|value| {
                let description = match context.state {
                    ParseState::AfterSelect | ParseState::InColumnList => Some("column".to_string()),
                    ParseState::AfterFrom => Some("table".to_string()),
                    ParseState::InWhere => Some("condition".to_string()),
                    ParseState::InOrderBy => Some("order".to_string()),
                    _ => None,
                };
                
                Suggestion {
                    value: value.clone(),
                    description,
                    extra: None,
                    span: Span {
                        start: start_pos,
                        end: pos,
                    },
                    style: None,
                    append_whitespace: match context.state {
                        ParseState::Start | ParseState::AfterFrom => true,
                        _ => {
                            matches!(
                                value.as_str(),
                                "SELECT" | "FROM" | "WHERE" | "ORDER BY" | "AND" | "OR" | "ASC" | "DESC"
                            )
                        }
                    },
                }
            })
            .collect()
    }
}