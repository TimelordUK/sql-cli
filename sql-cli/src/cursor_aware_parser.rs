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
        
        // Check for method call context first (e.g., "platformOrderId.")
        if let Some((property_name, property_type)) = self.detect_method_call_context(query_before_cursor, cursor_pos) {
            let suggestions = self.get_string_method_suggestions(&property_type, &partial_word);
            return ParseResult {
                suggestions,
                context: format!("MethodCall({}: {})", property_name, property_type),
                partial_word,
            };
        }
        
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
                // Prioritize column names over SQL keywords in WHERE clauses
                let mut suggestions = self.schema.get_columns("trade_deal");
                
                // Only add SQL keywords if no partial word or if partial doesn't match any columns
                let add_keywords = if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    let matching_columns = suggestions.iter()
                        .any(|col| col.to_lowercase().starts_with(&partial_lower));
                    !matching_columns // Only add keywords if no columns match
                } else {
                    true // Add keywords when no partial word
                };
                
                if add_keywords {
                    suggestions.extend(vec![
                        "AND".to_string(),
                        "OR".to_string(),
                        "ORDER BY".to_string(),
                    ]);
                }
                
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

    fn detect_method_call_context(&self, query_before_cursor: &str, cursor_pos: usize) -> Option<(String, String)> {
        // Look for pattern: "propertyName." at the end of the query before cursor
        // This handles cases like "WHERE platformOrderId." or "SELECT COUNT(*) WHERE ticker."
        
        // Find the last dot before cursor
        if let Some(dot_pos) = query_before_cursor.rfind('.') {
            // Extract the word immediately before the dot
            let before_dot = &query_before_cursor[..dot_pos];
            
            // Find the start of the property name (going backwards from dot)
            let mut property_start = dot_pos;
            let chars: Vec<char> = before_dot.chars().collect();
            
            while property_start > 0 {
                let char_pos = property_start - 1;
                if char_pos < chars.len() {
                    let ch = chars[char_pos];
                    if ch.is_alphanumeric() || ch == '_' {
                        property_start -= 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            
            if property_start < dot_pos {
                let property_name = before_dot[property_start..].trim().to_string();
                
                // Check if this property exists in our schema and get its type
                if let Some(property_type) = self.get_property_type(&property_name) {
                    return Some((property_name, property_type));
                }
            }
        }
        
        None
    }

    fn get_property_type(&self, property_name: &str) -> Option<String> {
        // Get property type from schema - for now, we'll use a simple mapping
        // In a more sophisticated implementation, this would query the actual schema
        
        let property_lower = property_name.to_lowercase();
        
        // String properties (most common for Dynamic LINQ operations)
        let string_properties = [
            "platformorderid", "dealid", "externalorderid", "parentorderid",
            "instrumentid", "instrumentname", "instrumenttype", "isin", "cusip",
            "ticker", "exchange", "counterparty", "counterpartyid", "counterpartytype",
            "counterpartycountry", "trader", "portfolio", "strategy", "desk",
            "status", "confirmationstatus", "settlementstatus", "allocationstatus",
            "currency", "side", "producttype", "venue", "clearinghouse", "prime",
            "comments"
        ];
        
        // Numeric properties  
        let numeric_properties = [
            "price", "quantity", "notional", "commission", "accrual", "netamount"
        ];
        
        // DateTime properties
        let datetime_properties = [
            "tradedate", "settlementdate", "createddate", "modifieddate"
        ];
        
        if string_properties.contains(&property_lower.as_str()) {
            Some("string".to_string())
        } else if numeric_properties.contains(&property_lower.as_str()) {
            Some("numeric".to_string())
        } else if datetime_properties.contains(&property_lower.as_str()) {
            Some("datetime".to_string())
        } else {
            // Default to string for unknown properties
            Some("string".to_string())
        }
    }

    fn get_string_method_suggestions(&self, property_type: &str, partial_word: &Option<String>) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match property_type {
            "string" => {
                // Common Dynamic LINQ string methods
                let string_methods = vec![
                    "Contains(\"\")",
                    "StartsWith(\"\")",
                    "EndsWith(\"\")",
                    "IndexOf(\"\")",
                    "Substring(0, 5)",
                    "ToLower()",
                    "ToUpper()",
                    "Trim()",
                    "Replace(\"\", \"\")",
                    "Length",
                ];
                
                if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    for method in string_methods {
                        if method.to_lowercase().starts_with(&partial_lower) {
                            suggestions.push(method.to_string());
                        }
                    }
                } else {
                    suggestions.extend(string_methods.into_iter().map(|s| s.to_string()));
                }
            },
            "numeric" => {
                let numeric_methods = vec![
                    "ToString()",
                    // Could add math methods here
                ];
                suggestions.extend(numeric_methods.into_iter().map(|s| s.to_string()));
            },
            "datetime" => {
                let datetime_methods = vec![
                    "Year",
                    "Month", 
                    "Day",
                    "ToString(\"yyyy-MM-dd\")",
                    "AddDays(1)",
                ];
                suggestions.extend(datetime_methods.into_iter().map(|s| s.to_string()));
            },
            _ => {
                // Default to string methods
                suggestions.push("ToString()".to_string());
            }
        }
        
        suggestions
    }
}