use crate::csv_fixes::quote_if_needed;
use crate::parser::{ParseState, Schema};
use crate::recursive_parser::{detect_cursor_context, CursorContext, LogicalOp};

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

    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = schema;
    }

    pub fn update_single_table(&mut self, table_name: String, columns: Vec<String>) {
        self.schema.set_single_table(table_name, columns);
    }

    pub fn get_completions(&self, query: &str, cursor_pos: usize) -> ParseResult {
        // Use the recursive parser for better context detection
        let (cursor_context, partial_word) = detect_cursor_context(query, cursor_pos);

        // If we didn't get a partial word from recursive parser, try our own extraction
        let partial_word = partial_word.or_else(|| self.extract_word_at_cursor(query, cursor_pos));

        let default_table = self.schema.get_first_table_name().unwrap_or("trade_deal");

        let (suggestions, context_str) = match &cursor_context {
            CursorContext::SelectClause => {
                // get_columns already applies quote_if_needed, so don't double-quote
                let mut cols = self.schema.get_columns(default_table);
                cols.push("*".to_string());

                // Filter out already selected columns
                // But don't include the partial word at cursor as "selected"
                let extract_pos = if let Some(ref partial) = partial_word {
                    cursor_pos.saturating_sub(partial.len())
                } else {
                    cursor_pos
                };
                let selected_columns = self.extract_selected_columns(query, extract_pos);
                cols = cols
                    .into_iter()
                    .filter(|col| {
                        // Check if this column is already selected (case-insensitive)
                        !selected_columns.iter().any(|selected| {
                            // Strip quotes from both for comparison if needed
                            let col_clean =
                                if col.starts_with('"') && col.ends_with('"') && col.len() > 2 {
                                    &col[1..col.len() - 1]
                                } else {
                                    col
                                };
                            let selected_clean = if selected.starts_with('"')
                                && selected.ends_with('"')
                                && selected.len() > 2
                            {
                                &selected[1..selected.len() - 1]
                            } else {
                                selected
                            };
                            col_clean.eq_ignore_ascii_case(selected_clean)
                        })
                    })
                    .collect();

                (cols, "SelectClause".to_string())
            }
            CursorContext::FromClause => {
                let tables = self.schema.get_table_names();
                (tables, "FromClause".to_string())
            }
            CursorContext::WhereClause | CursorContext::AfterLogicalOp(_) => {
                // We're in WHERE clause or after AND/OR - suggest columns
                let mut suggestions = self.schema.get_columns(default_table);

                // Only add SQL keywords if no partial word or if partial doesn't match any columns
                let add_keywords = if let Some(ref partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    !suggestions
                        .iter()
                        .any(|col| col.to_lowercase().starts_with(&partial_lower))
                } else {
                    true
                };

                if add_keywords {
                    suggestions.extend(vec![
                        "AND".to_string(),
                        "OR".to_string(),
                        "IN".to_string(),
                        "ORDER BY".to_string(),
                    ]);
                }

                let ctx = match &cursor_context {
                    CursorContext::AfterLogicalOp(LogicalOp::And) => "AfterAND",
                    CursorContext::AfterLogicalOp(LogicalOp::Or) => "AfterOR",
                    _ => "WhereClause",
                };
                (suggestions, ctx.to_string())
            }
            CursorContext::AfterColumn(col_name) => {
                // We're after a column and possibly a dot (method call context)
                let property_type = self
                    .get_property_type(col_name)
                    .unwrap_or("string".to_string());
                let suggestions = self.get_string_method_suggestions(&property_type, &partial_word);
                (suggestions, "AfterColumn".to_string())
            }
            CursorContext::AfterComparisonOp(col_name, op) => {
                // We're after a comparison operator - suggest based on column type
                let property_type = self
                    .get_property_type(&col_name)
                    .unwrap_or("string".to_string());
                let suggestions = match property_type.as_str() {
                    "datetime" => {
                        // For datetime columns, suggest DateTime constructor
                        let mut suggestions = vec!["DateTime(".to_string()];
                        // Also suggest common date patterns
                        suggestions.extend(vec![
                            "DateTime.Today".to_string(),
                            "DateTime.Now".to_string(),
                        ]);
                        suggestions
                    }
                    "string" => {
                        // For strings, suggest string literals
                        vec!["''".to_string()]
                    }
                    "numeric" => {
                        // For numbers, no specific suggestions
                        vec![]
                    }
                    _ => vec![],
                };
                (suggestions, format!("AfterComparison({} {})", col_name, op))
            }
            CursorContext::InMethodCall(obj, method) => {
                let property_type = self.get_property_type(obj).unwrap_or("string".to_string());
                let suggestions = self.get_string_method_suggestions(&property_type, &partial_word);
                (suggestions, format!("InMethodCall({}.{})", obj, method))
            }
            CursorContext::InExpression => {
                // Generic expression context - could be anywhere
                let mut suggestions = self.schema.get_columns(default_table);
                suggestions.extend(vec!["AND".to_string(), "OR".to_string()]);
                (suggestions, "InExpression".to_string())
            }
            CursorContext::OrderByClause => {
                // We're in ORDER BY clause - suggest selected columns if explicit, otherwise all columns
                let mut suggestions = Vec::new();

                // Extract selected columns from the query
                let selected_columns = self.extract_selected_columns(query, query.len());

                // If we have explicitly selected columns (not SELECT *), use those
                if !selected_columns.is_empty() && !selected_columns.contains(&"*".to_string()) {
                    suggestions.extend(selected_columns);
                } else {
                    // Fallback to all columns if SELECT * or no columns detected
                    // get_columns already applies quote_if_needed
                    suggestions.extend(self.schema.get_columns(default_table));
                }

                // Always add ASC/DESC options
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                (suggestions, "OrderByClause".to_string())
            }
            CursorContext::Unknown => {
                // Fall back to original heuristic parser
                let query_before_cursor = &query[..cursor_pos.min(query.len())];
                let context = self.determine_context(query_before_cursor);
                let suggestions = self.get_suggestions_for_context(&context, &partial_word, query);
                return ParseResult {
                    suggestions,
                    context: format!("{:?} (partial: {:?})", context, partial_word),
                    partial_word,
                };
            }
        };

        // Filter by partial word if present (but not for method suggestions as they're already filtered)
        let mut final_suggestions = suggestions;
        let is_method_context = matches!(
            cursor_context,
            CursorContext::AfterColumn(_)
                | CursorContext::InMethodCall(_, _)
                | CursorContext::AfterComparisonOp(_, _)
        );

        if let Some(ref partial) = partial_word {
            if !is_method_context {
                // Only filter non-method suggestions
                final_suggestions.retain(|suggestion| {
                    // Check if we're dealing with a partial quoted identifier
                    if partial.starts_with('"') {
                        // User is typing a quoted identifier like "customer
                        let partial_without_quote = &partial[1..]; // Remove the opening quote

                        // Check if suggestion is a quoted identifier that matches
                        if suggestion.starts_with('"')
                            && suggestion.ends_with('"')
                            && suggestion.len() > 2
                        {
                            // Full quoted identifier like "Customer Id"
                            let suggestion_without_quotes = &suggestion[1..suggestion.len() - 1];
                            suggestion_without_quotes
                                .to_lowercase()
                                .starts_with(&partial_without_quote.to_lowercase())
                        } else if suggestion.starts_with('"') && suggestion.len() > 1 {
                            // Partial quoted identifier (shouldn't happen in suggestions but handle it)
                            let suggestion_without_quote = &suggestion[1..];
                            suggestion_without_quote
                                .to_lowercase()
                                .starts_with(&partial_without_quote.to_lowercase())
                        } else {
                            // Also check non-quoted suggestions that might need quotes
                            suggestion
                                .to_lowercase()
                                .starts_with(&partial_without_quote.to_lowercase())
                        }
                    } else {
                        // Normal non-quoted partial (e.g., "customer")
                        // Handle quoted column names - check if the suggestion starts with a quote
                        let suggestion_to_check = if suggestion.starts_with('"')
                            && suggestion.ends_with('"')
                            && suggestion.len() > 2
                        {
                            // Remove both quotes for comparison (e.g., "Customer Id" -> "Customer Id")
                            &suggestion[1..suggestion.len() - 1]
                        } else if suggestion.starts_with('"') && suggestion.len() > 1 {
                            // Malformed quoted identifier - just strip opening quote
                            &suggestion[1..]
                        } else {
                            suggestion
                        };

                        // Now compare the cleaned suggestion with the partial
                        suggestion_to_check
                            .to_lowercase()
                            .starts_with(&partial.to_lowercase())
                    }
                });
            }
        }

        ParseResult {
            suggestions: final_suggestions,
            context: format!("{} (partial: {:?})", context_str, partial_word),
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

        // Handle both cases: cursor in middle of word or at end of word
        if start < end {
            // Extract partial word up to cursor
            let partial: String = chars[start..cursor_pos.min(end)].iter().collect();
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

        // Check if we're at the end after a logical operator (AND/OR)
        // This indicates we should be expecting a new column/condition
        let trimmed = query_before_cursor.trim();
        // Removed debug output to avoid corrupting TUI

        // Check various ways AND/OR might appear at the end
        let upper_trimmed = trimmed.to_uppercase();
        let ends_with_and_or = upper_trimmed.ends_with(" AND") || 
                               upper_trimmed.ends_with(" OR") ||
                               upper_trimmed.ends_with(" AND ") ||  // With trailing space
                               upper_trimmed.ends_with(" OR "); // With trailing space

        // Also check if the last word is AND/OR
        let words_check: Vec<&str> = query_upper.split_whitespace().collect();
        let last_word_is_and_or = words_check
            .last()
            .map(|w| *w == "AND" || *w == "OR")
            .unwrap_or(false);

        if ends_with_and_or || last_word_is_and_or {
            // After AND/OR, we're expecting a new column in WHERE context
            if query_upper.contains("WHERE") {
                // Detected AND/OR at end, return InWhere for column suggestions
                return ParseState::InWhere;
            }
        }

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
                "AND" | "OR" => {
                    // AND/OR continue the current WHERE context
                    if last_keyword == "WHERE" {
                        last_keyword_idx = Some(i);
                        last_keyword = "WHERE"; // Stay in WHERE context
                    }
                }
                "IN" => {
                    // IN continues WHERE context
                    if last_keyword == "WHERE" {
                        last_keyword_idx = Some(i);
                        last_keyword = "WHERE";
                    }
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
                if query_upper.contains("SELECT")
                    && query_upper.contains("FROM")
                    && query_upper.contains("WHERE")
                {
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

    fn get_suggestions_for_context(
        &self,
        context: &ParseState,
        partial_word: &Option<String>,
        query: &str,
    ) -> Vec<String> {
        let default_table = self.schema.get_first_table_name().unwrap_or("trade_deal");

        let mut suggestions = match context {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => {
                let mut cols = self.schema.get_columns(default_table);
                cols.push("*".to_string());
                cols
            }
            ParseState::InColumnList => {
                let mut cols = self.schema.get_columns(default_table);
                cols.push("FROM".to_string());
                cols
            }
            ParseState::AfterFrom => self.schema.get_table_names(),
            ParseState::AfterTable => {
                vec!["WHERE".to_string(), "ORDER BY".to_string()]
            }
            ParseState::InWhere => {
                // Prioritize column names over SQL keywords in WHERE clauses
                let mut suggestions = self.schema.get_columns(default_table);

                // Only add SQL keywords if no partial word or if partial doesn't match any columns
                let add_keywords = if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    let matching_columns = suggestions
                        .iter()
                        .any(|col| col.to_lowercase().starts_with(&partial_lower));
                    !matching_columns // Only add keywords if no columns match
                } else {
                    true // Add keywords when no partial word
                };

                if add_keywords {
                    suggestions.extend(vec![
                        "AND".to_string(),
                        "OR".to_string(),
                        "IN".to_string(),
                        "ORDER BY".to_string(),
                    ]);
                }

                suggestions
            }
            ParseState::InOrderBy => {
                let mut suggestions = Vec::new();

                // Extract selected columns from the query
                let selected_columns = self.extract_selected_columns(query, query.len());

                // If we have explicitly selected columns (not SELECT *), use those
                if !selected_columns.is_empty() && !selected_columns.contains(&"*".to_string()) {
                    suggestions.extend(selected_columns);
                } else {
                    // Fallback to all columns if SELECT * or no columns detected
                    suggestions.extend(self.schema.get_columns(default_table));
                }

                // Always add ASC/DESC options
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                suggestions
            }
            _ => vec![],
        };

        // Filter by partial word if present
        if let Some(partial) = partial_word {
            suggestions.retain(|suggestion| {
                suggestion
                    .to_lowercase()
                    .starts_with(&partial.to_lowercase())
            });
        }

        suggestions
    }

    fn extract_selected_columns(&self, query: &str, cursor_pos: usize) -> Vec<String> {
        // Extract columns that have already been selected in the current SELECT clause
        let mut selected_columns = Vec::new();

        // Find the SELECT keyword position
        let query_upper = query.to_uppercase();
        if let Some(select_pos) = query_upper.find("SELECT") {
            // Find the FROM keyword or cursor position, whichever comes first
            let end_pos = query_upper
                .find("FROM")
                .unwrap_or(cursor_pos)
                .min(cursor_pos);

            // Extract the SELECT clause
            if select_pos + 6 < end_pos {
                let select_clause = &query[(select_pos + 6)..end_pos];

                // Split by commas and extract column names
                for part in select_clause.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        // Extract just the column name (handle cases like "column AS alias")
                        let col_name = if trimmed.starts_with('"') {
                            // Handle quoted identifiers - find the closing quote
                            if let Some(close_quote_pos) = trimmed[1..].find('"') {
                                // Include both quotes
                                &trimmed[..close_quote_pos + 2]
                            } else {
                                // Malformed quoted identifier, take what we have
                                trimmed
                            }
                        } else {
                            // For unquoted identifiers, stop at first whitespace
                            if let Some(space_pos) = trimmed.find(char::is_whitespace) {
                                &trimmed[..space_pos]
                            } else {
                                trimmed
                            }
                        };

                        // Preserve the original case of the column name
                        selected_columns.push(col_name.to_string());
                    }
                }
            }
        }

        selected_columns
    }

    fn detect_method_call_context(
        &self,
        query_before_cursor: &str,
        cursor_pos: usize,
    ) -> Option<(String, String)> {
        // Look for pattern: "propertyName." at the end of the query before cursor
        // This handles cases like "WHERE platformOrderId." or "SELECT COUNT(*) WHERE ticker."
        // But NOT cases like "WHERE prop.Contains('x') AND " where we've moved past the method call

        // Find the last dot before cursor
        if let Some(dot_pos) = query_before_cursor.rfind('.') {
            // Check if cursor is close to the dot - if there's too much text after the dot,
            // we're probably not in method call context anymore
            let text_after_dot = &query_before_cursor[dot_pos + 1..];

            // If there's significant text after the dot that looks like a completed method call,
            // we're probably not in method call context
            if text_after_dot.contains(')')
                && (text_after_dot.contains(" AND ")
                    || text_after_dot.contains(" OR ")
                    || text_after_dot.trim().ends_with(" AND")
                    || text_after_dot.trim().ends_with(" OR"))
            {
                return None; // We've completed the method call and moved on
            }

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
            "platformorderid",
            "dealid",
            "externalorderid",
            "parentorderid",
            "instrumentid",
            "instrumentname",
            "instrumenttype",
            "isin",
            "cusip",
            "ticker",
            "exchange",
            "counterparty",
            "counterpartyid",
            "counterpartytype",
            "counterpartycountry",
            "trader",
            "portfolio",
            "strategy",
            "desk",
            "status",
            "confirmationstatus",
            "settlementstatus",
            "allocationstatus",
            "currency",
            "side",
            "producttype",
            "venue",
            "clearinghouse",
            "prime",
            "comments",
            "book",
            "source",
            "sourcesystem",
        ];

        // Numeric properties
        let numeric_properties = [
            "price",
            "quantity",
            "notional",
            "commission",
            "accrual",
            "netamount",
            "accruedinterest",
            "grossamount",
            "settlementamount",
            "fees",
            "tax",
        ];

        // DateTime properties
        let datetime_properties = [
            "tradedate",
            "settlementdate",
            "createddate",
            "modifieddate",
            "valuedate",
            "maturitydate",
            "confirmationdate",
            "executiondate",
            "lastmodifieddate",
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

    #[cfg(test)]
    pub fn test_extract_selected_columns(&self, query: &str, cursor_pos: usize) -> Vec<String> {
        self.extract_selected_columns(query, cursor_pos)
    }

    fn get_string_method_suggestions(
        &self,
        property_type: &str,
        partial_word: &Option<String>,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match property_type {
            "string" => {
                // Common Dynamic LINQ string methods
                let string_methods = vec![
                    "Contains('')",
                    "StartsWith('')",
                    "EndsWith('')",
                    "IndexOf('')",
                    "Substring(0, 5)",
                    "ToLower()",
                    "ToUpper()",
                    "IsNullOrEmpty()",
                    "Trim()",
                    "Replace('', '')",
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
            }
            "numeric" => {
                let numeric_methods = vec![
                    "ToString()",
                    // Could add math methods here
                ];
                suggestions.extend(numeric_methods.into_iter().map(|s| s.to_string()));
            }
            "datetime" => {
                let datetime_methods = vec![
                    "Year",
                    "Month",
                    "Day",
                    "ToString(\"yyyy-MM-dd\")",
                    "AddDays(1)",
                ];
                suggestions.extend(datetime_methods.into_iter().map(|s| s.to_string()));
            }
            _ => {
                // Default to string methods
                suggestions.push("ToString()".to_string());
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_parser() -> CursorAwareParser {
        CursorAwareParser::new()
    }

    #[test]
    fn test_basic_select_completion() {
        let parser = create_test_parser();

        // At the beginning
        let result = parser.get_completions("", 0);
        println!("Context for empty query: {}", result.context);
        assert_eq!(result.suggestions, vec!["SELECT"]);
        assert!(result.context.contains("Start") || result.context.contains("Unknown"));

        // After SELECT
        let result = parser.get_completions("SELECT ", 7);
        println!("Context for 'SELECT ': {}", result.context);
        assert!(result.suggestions.contains(&"*".to_string()));
        assert!(result.suggestions.contains(&"dealId".to_string()));
        assert!(result.context.contains("AfterSelect") || result.context.contains("SelectClause"));
    }

    #[test]
    fn test_where_clause_completion() {
        let parser = create_test_parser();

        // After WHERE
        let query = "SELECT * FROM trade_deal WHERE ";
        let result = parser.get_completions(query, query.len());
        println!("Context for WHERE clause: {}", result.context);
        assert!(result.suggestions.contains(&"dealId".to_string()));
        assert!(result.suggestions.contains(&"platformOrderId".to_string()));
        assert!(result.context.contains("InWhere") || result.context.contains("WhereClause"));
    }

    #[test]
    fn test_method_call_detection() {
        let parser = create_test_parser();

        // After column name with dot
        let query = "SELECT * FROM trade_deal WHERE platformOrderId.";
        let result = parser.get_completions(query, query.len());
        println!("Context for method call: {}", result.context);
        println!("Suggestions: {:?}", result.suggestions);
        assert!(result.suggestions.contains(&"Contains('')".to_string()));
        assert!(result.suggestions.contains(&"StartsWith('')".to_string()));
        assert!(result.context.contains("MethodCall") || result.context.contains("AfterColumn"));
    }

    #[test]
    fn test_and_operator_context() {
        let parser = create_test_parser();

        // After completed method call and AND
        let query = "SELECT * FROM trade_deal WHERE allocationStatus.Contains(\"All\") AND ";
        let result = parser.get_completions(query, query.len());
        println!("Context after AND: {}", result.context);
        assert!(result.suggestions.contains(&"dealId".to_string()));
        assert!(result.suggestions.contains(&"platformOrderId".to_string()));
        assert!(
            result.context.contains("InWhere")
                || result.context.contains("AfterAND")
                || result.context.contains("WhereClause")
        );
        assert!(!result.context.contains("MethodCall"));
    }

    #[test]
    fn test_and_operator_with_partial_word() {
        let parser = create_test_parser();

        // After AND with partial column name
        let query = "SELECT * FROM trade_deal WHERE allocationStatus.Contains(\"All\") AND p";
        let result = parser.get_completions(query, query.len());

        // Should suggest columns starting with 'p'
        assert!(result.suggestions.contains(&"platformOrderId".to_string()));
        assert!(result.suggestions.contains(&"price".to_string()));
        assert!(result.suggestions.contains(&"portfolio".to_string()));

        // Should NOT suggest columns that don't start with 'p'
        assert!(!result.suggestions.contains(&"dealId".to_string()));
        assert!(!result.suggestions.contains(&"quantity".to_string()));

        // Should be in WHERE context, not MethodCall
        assert!(
            result.context.contains("InWhere")
                || result.context.contains("WhereClause")
                || result.context.contains("AfterAND")
        );
        assert!(!result.context.contains("MethodCall"));

        // Should have detected partial word
        assert!(result.context.contains("(partial: Some(\"p\"))"));
    }

    #[test]
    fn test_or_operator_context() {
        let parser = create_test_parser();

        // After OR
        let query = "SELECT * FROM trade_deal WHERE price > 100 OR ";
        let result = parser.get_completions(query, query.len());
        println!("Context after OR: {}", result.context);
        assert!(result.suggestions.contains(&"dealId".to_string()));
        assert!(
            result.context.contains("InWhere")
                || result.context.contains("AfterOR")
                || result.context.contains("WhereClause")
        );
    }

    #[test]
    fn test_partial_word_extraction() {
        let parser = create_test_parser();

        // Test various partial word scenarios
        assert_eq!(
            parser.extract_word_at_cursor("SELECT deal", 11),
            Some("deal".to_string())
        );
        assert_eq!(
            parser.extract_word_at_cursor("WHERE p", 7),
            Some("p".to_string())
        );
        assert_eq!(
            parser.extract_word_at_cursor("AND platf", 9),
            Some("platf".to_string())
        );

        // Edge cases
        assert_eq!(parser.extract_word_at_cursor("", 0), None);
        assert_eq!(parser.extract_word_at_cursor("SELECT ", 7), None);
    }

    #[test]
    fn test_complex_query_with_multiple_conditions() {
        let parser = create_test_parser();

        // Complex query with multiple ANDs
        let query = "SELECT * FROM trade_deal WHERE platformOrderId.StartsWith(\"ABC\") AND price > 100 AND ";
        let result = parser.get_completions(query, query.len());
        println!("Context for complex query: {}", result.context);
        assert!(result.suggestions.contains(&"dealId".to_string()));
        assert!(
            result.context.contains("InWhere")
                || result.context.contains("AfterAND")
                || result.context.contains("WhereClause")
        );
        assert!(!result.context.contains("MethodCall"));
    }

    #[test]
    fn test_in_clause_support() {
        let parser = create_test_parser();

        // After IN
        let query = "SELECT * FROM trade_deal WHERE status IN ";
        let result = parser.get_completions(query, query.len());
        println!("Context after IN: {}", result.context);
        // IN clause support - should suggest opening parenthesis or values
        assert!(
            result.context.contains("InWhere")
                || result.context.contains("WhereClause")
                || result.context.contains("Unknown")
        );
    }

    #[test]
    fn test_partial_method_name_completion() {
        let parser = create_test_parser();

        // Partial method name after dot
        let query = "SELECT * FROM trade_deal WHERE instrumentName.Con";
        let result = parser.get_completions(query, query.len());
        println!("Context for partial method: {}", result.context);
        println!("Suggestions: {:?}", result.suggestions);

        // Should be in method call context with partial word "Con"
        assert!(result.context.contains("MethodCall") || result.context.contains("AfterColumn"));
        assert!(result.context.contains("(partial: Some(\"Con\"))"));

        // Should suggest methods starting with "Con"
        assert!(result.suggestions.contains(&"Contains('')".to_string()));
        assert!(!result.suggestions.contains(&"StartsWith('')".to_string())); // Doesn't start with "Con"
    }

    #[test]
    fn test_partial_matching_quoted_identifier() {
        let parser = CursorAwareParser::new();
        // Set up schema with "Customer Id" column
        let mut parser = parser;
        parser.update_single_table(
            "customers".to_string(),
            vec![
                "Index".to_string(),
                "Customer Id".to_string(), // Store without quotes
                "First Name".to_string(),  // Store without quotes
                "Company".to_string(),
            ],
        );

        // Test that "customer" partial matches "Customer Id"
        let query = "SELECT customer";
        let result = parser.get_completions(query, query.len());

        // Should suggest "Customer Id" (quoted)
        assert!(
            result.suggestions.iter().any(|s| s == "\"Customer Id\""),
            "Should suggest quoted Customer Id for partial \"customer\". Got: {:?}",
            result.suggestions
        );
    }

    #[test]
    fn test_case_preservation_in_order_by() {
        let parser = CursorAwareParser::new();
        let mut parser = parser;
        parser.update_single_table(
            "customers".to_string(),
            vec!["Company".to_string(), "Country".to_string()],
        );

        // Test that ORDER BY preserves case from SELECT
        let query = "SELECT Company, Country FROM customers ORDER BY Com";
        let result = parser.get_completions(query, query.len());

        // Should suggest "Company" with proper case
        assert!(
            result.suggestions.iter().any(|s| s == "Company"),
            "Should preserve case in ORDER BY suggestions. Got: {:?}",
            result.suggestions
        );
    }

    #[test]
    fn test_extract_selected_columns_preserves_case() {
        let parser = CursorAwareParser::new();

        let query = "SELECT Company, Country FROM customers";
        let columns = parser.test_extract_selected_columns(query, query.len());

        assert_eq!(columns, vec!["Company", "Country"]);
        assert_ne!(
            columns,
            vec!["company", "country"],
            "Should preserve original case"
        );
    }

    #[test]
    fn test_filtering_already_selected_columns() {
        let parser = CursorAwareParser::new();
        let mut parser = parser;
        parser.update_single_table(
            "customers".to_string(),
            vec![
                "Company".to_string(),
                "Country".to_string(),
                "Customer Id".to_string(),
            ],
        );

        // Already selected Company, should not suggest it again
        let query = "SELECT Company, ";
        let result = parser.get_completions(query, query.len());

        assert!(
            !result.suggestions.iter().any(|s| s == "Company"),
            "Should not suggest already selected Company"
        );
        assert!(
            result.suggestions.iter().any(|s| s == "Country"),
            "Should suggest Country"
        );
        assert!(
            result.suggestions.iter().any(|s| s == "\"Customer Id\""),
            "Should suggest Customer Id"
        );
    }

    #[test]
    fn test_order_by_completion_with_quoted_columns() {
        let parser = CursorAwareParser::new();
        let mut parser = parser;
        parser.update_single_table(
            "customers".to_string(),
            vec![
                "City".to_string(),
                "Company".to_string(),
                "Country".to_string(),
                "Customer Id".to_string(),
            ],
        );

        // Test ORDER BY completion after query with quoted columns
        let query = r#"SELECT City,Company,Country,"Customer Id" FROM customers ORDER BY coun"#;
        let result = parser.get_completions(query, query.len());

        // Should get the partial word right
        assert_eq!(
            result.partial_word,
            Some("coun".to_string()),
            "Should extract 'coun' as partial, not something weird"
        );

        // Should suggest Country
        assert!(
            result.suggestions.iter().any(|s| s == "Country"),
            "Should suggest Country for partial 'coun'. Got: {:?}",
            result.suggestions
        );
    }

    #[test]
    fn test_order_by_quoted_partial_completion() {
        let parser = CursorAwareParser::new();
        let mut parser = parser;
        parser.update_single_table(
            "customers".to_string(),
            vec![
                "City".to_string(),
                "Company".to_string(),
                "Country".to_string(),
                "Customer Id".to_string(),
            ],
        );

        // Test ORDER BY completion with partial quoted identifier
        let query =
            r#"select City,Company,Country,"Customer Id" from customers order by City, "Customer"#;
        let result = parser.get_completions(query, query.len());

        // The partial word should be "Customer
        assert_eq!(
            result.partial_word,
            Some("\"Customer".to_string()),
            "Should extract '\"Customer' as partial"
        );

        // Should suggest "Customer Id" with proper quotes
        assert!(
            result.suggestions.iter().any(|s| s == "\"Customer Id\""),
            "Should suggest properly quoted 'Customer Id' for partial '\"Customer'. Got: {:?}",
            result.suggestions
        );

        // Should NOT have truncated suggestions like "Customer
        assert!(
            !result.suggestions.iter().any(|s| s == "\"Customer"),
            "Should not have truncated suggestion '\"Customer'. Got: {:?}",
            result.suggestions
        );
    }
}
