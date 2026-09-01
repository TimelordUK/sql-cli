use crate::data::csv_fixes::quote_if_needed;
use crate::parser::{ParseState, Schema, TableInfo};
use crate::recursive_parser::{detect_cursor_context, CursorContext, LogicalOp};
use crate::sql::completion_token::{find_completion_token, CompletionToken};

#[derive(Debug, Clone)]
pub struct CursorAwareParser {
    schema: Schema,
}

#[derive(Debug)]
pub struct ParseResult {
    pub suggestions: Vec<String>,
    pub context: String,
    pub partial_word: Option<String>,
    /// Byte offset in the query where an accepted suggestion should be spliced
    /// in; it replaces `query[replace_start..cursor_pos]`. The parser owns this
    /// because only it knows whether the text before the cursor is a column
    /// reference (`name.com` - replace all of it) or a method call on a column
    /// (`price.Con` - replace only `Con`).
    pub replace_start: usize,
}

/// Strip the surrounding quotes from a quoted identifier so it can be compared
/// against what the user typed.
fn strip_identifier_quotes(suggestion: &str) -> &str {
    suggestion
        .strip_prefix('"')
        .map_or(suggestion, |rest| rest.strip_suffix('"').unwrap_or(rest))
}

impl Default for CursorAwareParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorAwareParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
        }
    }

    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = schema;
    }

    pub fn update_single_table(&mut self, table_name: String, columns: Vec<String>) {
        self.schema.set_single_table(&table_name, columns);
    }

    /// Replace the schema with a fully-typed snapshot of the loaded table.
    /// Preferred over [`Self::update_single_table`], which can only say
    /// "string" about every column.
    pub fn update_single_table_info(&mut self, table: TableInfo) {
        self.schema.set_single_table_info(table);
    }

    #[must_use]
    pub fn get_table_columns(&self, table_name: &str) -> Vec<String> {
        self.schema.get_columns(table_name)
    }

    #[must_use]
    pub fn get_completions(&self, query: &str, cursor_pos: usize) -> ParseResult {
        // Use the recursive parser for better context detection
        let (cursor_context, partial_word) = detect_cursor_context(query, cursor_pos);

        // If we didn't get a partial word from recursive parser, try our own extraction
        let partial_word = partial_word.or_else(|| self.extract_word_at_cursor(query, cursor_pos));

        let default_table = self
            .schema
            .get_first_table_name()
            .unwrap_or("trade_deal".to_string());

        // The identifier the cursor sits in, scanned with quote- and
        // dot-awareness. It is the single source of truth for both what we
        // filter against and which span an accepted suggestion replaces.
        let token = find_completion_token(query, cursor_pos);

        // `name.com` is ambiguous: a column literally called `name.common`, or
        // a method call on a column called `name`. Columns win when the text
        // actually prefixes one, because completion is the only way to
        // discover a dotted column name - methods stay reachable on any column
        // whose name really exists.
        if let Some(result) = self.complete_dotted_column(token.as_ref(), &default_table) {
            return result;
        }

        let (suggestions, context_str) = match &cursor_context {
            CursorContext::SelectClause => {
                // Apply quote_if_needed to column names
                let mut cols = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();
                cols.push("*".to_string());

                // Add math functions
                cols.extend(vec![
                    "ROUND(".to_string(),
                    "ABS(".to_string(),
                    "FLOOR(".to_string(),
                    "CEILING(".to_string(),
                    "CEIL(".to_string(),
                    "MOD(".to_string(),
                    "QUOTIENT(".to_string(),
                    "POWER(".to_string(),
                    "POW(".to_string(),
                    "SQRT(".to_string(),
                    "EXP(".to_string(),
                    "LN(".to_string(),
                    "LOG(".to_string(),
                    "LOG10(".to_string(),
                    "PI(".to_string(),
                    "TEXTJOIN(".to_string(),
                    "DATEDIFF(".to_string(),
                    "DATEADD(".to_string(),
                    "NOW(".to_string(),
                    "TODAY(".to_string(),
                ]);

                // NOTE: We intentionally do NOT filter out already selected columns
                // Users may want to select the same column multiple times, especially
                // when using it in computed expressions like: SELECT q * p as notional, q
                // Duplicate handling should be done at query execution time if needed

                (cols, "SelectClause".to_string())
            }
            CursorContext::FromClause => {
                let tables = self.schema.get_table_names();
                (tables, "FromClause".to_string())
            }
            CursorContext::WhereClause | CursorContext::AfterLogicalOp(_) => {
                // We're in WHERE clause or after AND/OR - suggest columns
                let mut suggestions = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();

                // Add math functions that can be used in WHERE
                suggestions.extend(vec![
                    "ROUND(".to_string(),
                    "ABS(".to_string(),
                    "FLOOR(".to_string(),
                    "CEILING(".to_string(),
                    "CEIL(".to_string(),
                    "MOD(".to_string(),
                    "QUOTIENT(".to_string(),
                    "POWER(".to_string(),
                    "POW(".to_string(),
                    "SQRT(".to_string(),
                    "EXP(".to_string(),
                    "LN(".to_string(),
                    "LOG(".to_string(),
                    "LOG10(".to_string(),
                    "PI(".to_string(),
                    "TEXTJOIN(".to_string(),
                    "DATEDIFF(".to_string(),
                    "DATEADD(".to_string(),
                    "NOW(".to_string(),
                    "TODAY(".to_string(),
                ]);

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
                    .get_property_type(col_name)
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
                    "boolean" => vec!["true".to_string(), "false".to_string()],
                    _ => vec![],
                };
                (suggestions, format!("AfterComparison({col_name} {op})"))
            }
            CursorContext::InMethodCall(obj, method) => {
                let property_type = self.get_property_type(obj).unwrap_or("string".to_string());
                let suggestions = self.get_string_method_suggestions(&property_type, &partial_word);
                (suggestions, format!("InMethodCall({obj}.{method})"))
            }
            CursorContext::InExpression => {
                // Generic expression context - could be anywhere
                let mut suggestions = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();

                // Add math functions
                suggestions.extend(vec![
                    "ROUND(".to_string(),
                    "ABS(".to_string(),
                    "FLOOR(".to_string(),
                    "CEILING(".to_string(),
                    "CEIL(".to_string(),
                    "MOD(".to_string(),
                    "QUOTIENT(".to_string(),
                    "POWER(".to_string(),
                    "POW(".to_string(),
                    "SQRT(".to_string(),
                    "EXP(".to_string(),
                    "LN(".to_string(),
                    "LOG(".to_string(),
                    "LOG10(".to_string(),
                    "PI(".to_string(),
                    "AND".to_string(),
                    "OR".to_string(),
                ]);
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
                    // Apply quote_if_needed to column names
                    suggestions.extend(
                        self.schema
                            .get_columns(&default_table)
                            .into_iter()
                            .map(|col| quote_if_needed(&col)),
                    );
                }

                // Always add ASC/DESC options
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                (suggestions, "OrderByClause".to_string())
            }
            CursorContext::Unknown => {
                // Fall back to original heuristic parser
                // Ensure we slice at a valid UTF-8 character boundary
                let safe_cursor_pos = self.find_safe_boundary(query, cursor_pos.min(query.len()));
                let query_before_cursor = &query[..safe_cursor_pos];
                let context = self.determine_context(query_before_cursor);
                let suggestions = self.get_suggestions_for_context(&context, &partial_word, query);
                return ParseResult {
                    suggestions,
                    context: format!("{context:?} (partial: {partial_word:?})"),
                    partial_word,
                    replace_start: token.as_ref().map_or(cursor_pos, |t| t.start),
                };
            }
        };

        let is_method_context = matches!(
            cursor_context,
            CursorContext::AfterColumn(_) | CursorContext::InMethodCall(_, _)
        );
        let is_value_context = matches!(cursor_context, CursorContext::AfterComparisonOp(_, _));

        // Method suggestions arrive pre-filtered against the partial method
        // name; everything else is filtered here. Suggestions may be quoted
        // (`"name.common"`) while the user typed either `na` or `"na`, so both
        // sides are compared with quotes stripped.
        let mut final_suggestions = suggestions;
        if !is_method_context && !is_value_context {
            if let Some(needle) = token
                .as_ref()
                .map(CompletionToken::unquoted)
                .filter(|n| !n.is_empty())
            {
                let needle = needle.to_lowercase();
                final_suggestions.retain(|s| {
                    strip_identifier_quotes(s)
                        .to_lowercase()
                        .starts_with(&needle)
                });
            }
        }

        // Method names replace only the segment after the dot (`price.Con` ->
        // `price.Contains('')`); everything else replaces the whole identifier.
        // After a quoted column the dot terminates the token, so the partial
        // method is already the whole token (`"name.common".Star` -> `Star`).
        let replace_start = if is_method_context {
            token.as_ref().map_or(cursor_pos, |t| {
                t.last_segment().map_or(t.start, |(start, _)| start)
            })
        } else {
            token.as_ref().map_or(cursor_pos, |t| t.start)
        };

        ParseResult {
            suggestions: final_suggestions,
            context: format!("{context_str} (partial: {partial_word:?})"),
            partial_word,
            replace_start,
        }
    }

    /// Suggest real column names when the text at the cursor prefixes one.
    ///
    /// Only dotted text reaches here: undotted prefixes are already handled by
    /// the clause contexts, whereas a dotted prefix would otherwise be read as
    /// a method call and the column would be unreachable by completion.
    fn complete_dotted_column(
        &self,
        token: Option<&CompletionToken>,
        table: &str,
    ) -> Option<ParseResult> {
        let token = token?;
        let needle = token.unquoted();
        if !needle.contains('.') {
            return None;
        }

        let needle_lower = needle.to_lowercase();
        let matches: Vec<String> = self
            .schema
            .get_columns(table)
            .into_iter()
            .filter(|col| col.to_lowercase().starts_with(&needle_lower))
            .map(|col| quote_if_needed(&col))
            .collect();

        // No column by that name - leave it to the method-call handling.
        if matches.is_empty() {
            return None;
        }

        Some(ParseResult {
            suggestions: matches,
            context: format!("DottedColumn (partial: {needle:?})"),
            partial_word: Some(token.text.clone()),
            replace_start: token.start,
        })
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
            if partial.is_empty() {
                None
            } else {
                Some(partial)
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
            .is_some_and(|w| *w == "AND" || *w == "OR");

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
        let default_table = self
            .schema
            .get_first_table_name()
            .unwrap_or("trade_deal".to_string());

        let mut suggestions = match context {
            ParseState::Start => vec!["SELECT".to_string()],
            ParseState::AfterSelect => {
                let mut cols = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();
                cols.push("*".to_string());
                cols
            }
            ParseState::InColumnList => {
                let mut cols = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();
                cols.push("FROM".to_string());
                cols
            }
            ParseState::AfterFrom => self.schema.get_table_names(),
            ParseState::AfterTable => {
                vec!["WHERE".to_string(), "ORDER BY".to_string()]
            }
            ParseState::InWhere => {
                // Prioritize column names over SQL keywords in WHERE clauses
                let mut suggestions = self
                    .schema
                    .get_columns(&default_table)
                    .into_iter()
                    .map(|col| quote_if_needed(&col))
                    .collect::<Vec<_>>();

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
                    suggestions.extend(
                        self.schema
                            .get_columns(&default_table)
                            .into_iter()
                            .map(|col| quote_if_needed(&col)),
                    );
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

    /// The completion category of a column, from the loaded schema.
    ///
    /// Before T2 this was a hardcoded list of trade-desk column names with
    /// `else => "string"`, so on any other dataset every column was a string:
    /// numeric columns were offered `Contains('')` and date columns never got
    /// `DateTime(`. `None` now means genuinely unknown - no file loaded yet,
    /// or text that is not a column at all - and callers still fall back to
    /// string methods there, which is the safe default for an unknown name.
    fn get_property_type(&self, property_name: &str) -> Option<String> {
        self.schema
            .find_column(property_name)
            .map(|column| column.data_type.as_str().to_string())
    }

    /// Find a safe UTF-8 character boundary at or before the given position
    fn find_safe_boundary(&self, s: &str, pos: usize) -> usize {
        if pos >= s.len() {
            return s.len();
        }

        // If already at a valid boundary, return it
        if s.is_char_boundary(pos) {
            return pos;
        }

        // Find the nearest valid character boundary before pos
        let mut safe_pos = pos;
        while safe_pos > 0 && !s.is_char_boundary(safe_pos) {
            safe_pos -= 1;
        }
        safe_pos
    }

    #[cfg(test)]
    #[must_use]
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
                // Format: methods with parameters include ('') with cursor placement hint
                // Methods without parameters include () for consistency
                let string_methods = vec![
                    "Contains('')",
                    "StartsWith('')",
                    "EndsWith('')",
                    "IndexOf('')",
                    "Substring(0, 5)",
                    "ToLower()",
                    "ToUpper()",
                    "Trim()",
                    "TrimStart()",
                    "TrimEnd()",
                    "IsNullOrEmpty()",
                    "Replace('', '')",
                    "Length()", // Changed from "Length" to "Length()"
                ];

                if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    for method in string_methods {
                        if method.to_lowercase().starts_with(&partial_lower) {
                            suggestions.push(method.to_string());
                        }
                    }
                } else {
                    suggestions.extend(
                        string_methods
                            .into_iter()
                            .map(std::string::ToString::to_string),
                    );
                }
            }
            "numeric" | "integer" | "float" | "decimal" => {
                // Numeric columns can use string methods via type coercion in the tree walker
                let numeric_string_methods = vec![
                    "Contains('')",
                    "StartsWith('')",
                    "EndsWith('')",
                    "ToString()",
                    "Length()", // Changed from "Length" to "Length()"
                                // Could add math methods here in the future
                ];

                if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    for method in numeric_string_methods {
                        if method.to_lowercase().starts_with(&partial_lower) {
                            suggestions.push(method.to_string());
                        }
                    }
                } else {
                    suggestions.extend(
                        numeric_string_methods
                            .into_iter()
                            .map(std::string::ToString::to_string),
                    );
                }
            }
            "datetime" => {
                // DateTime columns can use both datetime-specific and string methods
                let datetime_methods = vec![
                    "Year()",  // Changed from "Year" to "Year()"
                    "Month()", // Changed from "Month" to "Month()"
                    "Day()",   // Changed from "Day" to "Day()"
                    "ToString(\"yyyy-MM-dd\")",
                    "AddDays(1)",
                    // String methods via coercion
                    "Contains('')",
                    "StartsWith('')",
                    "EndsWith('')",
                    "Length()", // Changed from "Length" to "Length()"
                ];

                if let Some(partial) = partial_word {
                    let partial_lower = partial.to_lowercase();
                    for method in datetime_methods {
                        if method.to_lowercase().starts_with(&partial_lower) {
                            suggestions.push(method.to_string());
                        }
                    }
                } else {
                    suggestions.extend(
                        datetime_methods
                            .into_iter()
                            .map(std::string::ToString::to_string),
                    );
                }
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

    use crate::parser::ColumnInfo;
    use crate::parser::ColumnType;

    /// The trade_deal schema these tests were written against. It used to be
    /// the *default* schema, which is exactly what T2 removed - a parser with
    /// no file loaded now knows no columns, so the fixture has to say so.
    fn create_test_parser() -> CursorAwareParser {
        let mut parser = CursorAwareParser::new();
        parser.update_single_table_info(TableInfo::new(
            "trade_deal",
            crate::config::schema_config::get_full_trade_deal_columns()
                .into_iter()
                .map(|name| {
                    let column_type = match name.to_lowercase().as_str() {
                        "price" | "quantity" | "notional" | "commission" | "netamount" => {
                            ColumnType::Numeric
                        }
                        "tradedate" | "settlementdate" | "createddate" | "confirmationdate" => {
                            ColumnType::DateTime
                        }
                        _ => ColumnType::String,
                    };
                    ColumnInfo::new(name).with_type(column_type)
                })
                .collect(),
        ));
        parser
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

        // Already selected Company, but we SHOULD still suggest it
        // Users may want to select the same column multiple times
        // e.g., for computed expressions like: SELECT q * p as total, q
        let query = "SELECT Company, ";
        let result = parser.get_completions(query, query.len());

        assert!(
            result.suggestions.iter().any(|s| s == "Company"),
            "Should still suggest Company even though already selected"
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
