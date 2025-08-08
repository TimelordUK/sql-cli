use std::collections::HashSet;

/// Manages tab completion for SQL queries
/// Extracted from the monolithic enhanced_tui.rs
#[derive(Debug, Clone)]
pub struct CompletionManager {
    /// Current completion suggestions
    suggestions: Vec<String>,

    /// Current index in suggestions list
    current_index: usize,

    /// Last query we generated suggestions for
    last_query: String,

    /// Last cursor position for suggestions
    last_cursor_pos: usize,

    /// Available table names for completion
    table_names: HashSet<String>,

    /// Available column names per table
    column_names: std::collections::HashMap<String, Vec<String>>,
}

impl CompletionManager {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            current_index: 0,
            last_query: String::new(),
            last_cursor_pos: 0,
            table_names: HashSet::new(),
            column_names: std::collections::HashMap::new(),
        }
    }

    /// Reset completion state
    pub fn reset(&mut self) {
        self.suggestions.clear();
        self.current_index = 0;
        self.last_query.clear();
        self.last_cursor_pos = 0;
    }

    /// Check if we have active suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    /// Get current suggestions
    pub fn suggestions(&self) -> &[String] {
        &self.suggestions
    }

    /// Get current suggestion index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the currently selected suggestion
    pub fn current_suggestion(&self) -> Option<&str> {
        if self.suggestions.is_empty() {
            None
        } else {
            Some(&self.suggestions[self.current_index])
        }
    }

    /// Move to next suggestion
    pub fn next_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.current_index = (self.current_index + 1) % self.suggestions.len();
        }
    }

    /// Move to previous suggestion
    pub fn prev_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.suggestions.len() - 1
            } else {
                self.current_index - 1
            };
        }
    }

    /// Update available table names
    pub fn set_table_names(&mut self, tables: HashSet<String>) {
        self.table_names = tables;
    }

    /// Update column names for a table
    pub fn set_column_names(&mut self, table: String, columns: Vec<String>) {
        self.column_names.insert(table, columns);
    }

    /// Generate suggestions for a partial word at cursor position
    pub fn generate_suggestions(
        &mut self,
        query: &str,
        cursor_pos: usize,
        partial_word: &str,
    ) -> bool {
        // Check if we already have suggestions for this position
        if query == self.last_query && cursor_pos == self.last_cursor_pos {
            return self.has_suggestions();
        }

        // Store query state
        self.last_query = query.to_string();
        self.last_cursor_pos = cursor_pos;
        self.suggestions.clear();
        self.current_index = 0;

        // Determine context for completion
        let context = self.analyze_context(query, cursor_pos);

        // Generate suggestions based on context
        match context {
            CompletionContext::TableName => {
                self.suggest_tables(partial_word);
            }
            CompletionContext::ColumnName(table) => {
                self.suggest_columns(&table, partial_word);
            }
            CompletionContext::Keyword => {
                self.suggest_keywords(partial_word);
            }
            CompletionContext::Unknown => {
                // Try all categories
                self.suggest_keywords(partial_word);
                self.suggest_tables(partial_word);
            }
        }

        self.has_suggestions()
    }

    /// Analyze query context at cursor position
    fn analyze_context(&self, query: &str, cursor_pos: usize) -> CompletionContext {
        let before_cursor = &query[..cursor_pos.min(query.len())];
        let lower = before_cursor.to_lowercase();

        // Simple heuristics for context detection
        if lower.ends_with("from ") || lower.ends_with("join ") {
            CompletionContext::TableName
        } else if lower.contains("select ") && !lower.contains(" from") {
            // In SELECT clause, suggest columns
            // Try to find table name in FROM clause if it exists
            if let Some(table) = self.extract_table_from_query(query) {
                CompletionContext::ColumnName(table)
            } else {
                CompletionContext::Keyword
            }
        } else if lower.ends_with("where ") || lower.ends_with("and ") || lower.ends_with("or ") {
            // In WHERE clause, suggest columns
            if let Some(table) = self.extract_table_from_query(query) {
                CompletionContext::ColumnName(table)
            } else {
                CompletionContext::Unknown
            }
        } else {
            CompletionContext::Keyword
        }
    }

    /// Extract table name from query (simple version)
    fn extract_table_from_query(&self, query: &str) -> Option<String> {
        let lower = query.to_lowercase();
        if let Some(from_pos) = lower.find(" from ") {
            let after_from = &query[from_pos + 6..];
            let table_name = after_from
                .split_whitespace()
                .next()
                .map(|s| s.trim_end_matches(',').trim_end_matches(';'))?;

            // Check if it's a known table
            if self.table_names.contains(table_name) {
                Some(table_name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Suggest table names
    fn suggest_tables(&mut self, partial: &str) {
        let lower_partial = partial.to_lowercase();
        let mut suggestions: Vec<String> = self
            .table_names
            .iter()
            .filter(|t| t.to_lowercase().starts_with(&lower_partial))
            .cloned()
            .collect();

        suggestions.sort();
        self.suggestions = suggestions;
    }

    /// Suggest column names for a table
    fn suggest_columns(&mut self, table: &str, partial: &str) {
        if let Some(columns) = self.column_names.get(table) {
            let lower_partial = partial.to_lowercase();
            let mut suggestions: Vec<String> = columns
                .iter()
                .filter(|c| c.to_lowercase().starts_with(&lower_partial))
                .cloned()
                .collect();

            suggestions.sort();
            self.suggestions = suggestions;
        }
    }

    /// Suggest SQL keywords
    fn suggest_keywords(&mut self, partial: &str) {
        const SQL_KEYWORDS: &[&str] = &[
            "SELECT",
            "FROM",
            "WHERE",
            "GROUP BY",
            "ORDER BY",
            "HAVING",
            "JOIN",
            "LEFT JOIN",
            "RIGHT JOIN",
            "INNER JOIN",
            "OUTER JOIN",
            "ON",
            "AND",
            "OR",
            "NOT",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "AS",
            "DISTINCT",
            "COUNT",
            "SUM",
            "AVG",
            "MIN",
            "MAX",
            "INSERT",
            "UPDATE",
            "DELETE",
            "CREATE",
            "DROP",
            "ALTER",
            "TABLE",
            "INDEX",
            "VIEW",
            "UNION",
            "ALL",
            "LIMIT",
            "OFFSET",
        ];

        let lower_partial = partial.to_lowercase();
        let mut suggestions: Vec<String> = SQL_KEYWORDS
            .iter()
            .filter(|k| k.to_lowercase().starts_with(&lower_partial))
            .map(|k| k.to_string())
            .collect();

        suggestions.sort();
        self.suggestions = suggestions;
    }
}

/// Context for completion
#[derive(Debug, Clone)]
enum CompletionContext {
    TableName,
    ColumnName(String), // Table name
    Keyword,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_manager_creation() {
        let cm = CompletionManager::new();
        assert!(!cm.has_suggestions());
        assert_eq!(cm.current_index(), 0);
    }

    #[test]
    fn test_suggestion_navigation() {
        let mut cm = CompletionManager::new();
        cm.suggestions = vec![
            "SELECT".to_string(),
            "FROM".to_string(),
            "WHERE".to_string(),
        ];

        assert_eq!(cm.current_suggestion(), Some("SELECT"));

        cm.next_suggestion();
        assert_eq!(cm.current_suggestion(), Some("FROM"));

        cm.next_suggestion();
        assert_eq!(cm.current_suggestion(), Some("WHERE"));

        cm.next_suggestion();
        assert_eq!(cm.current_suggestion(), Some("SELECT")); // Wraps around

        cm.prev_suggestion();
        assert_eq!(cm.current_suggestion(), Some("WHERE"));
    }

    #[test]
    fn test_keyword_suggestions() {
        let mut cm = CompletionManager::new();
        cm.generate_suggestions("SEL", 3, "SEL");

        assert!(cm.has_suggestions());
        assert!(cm.suggestions().contains(&"SELECT".to_string()));
    }

    #[test]
    fn test_table_suggestions() {
        let mut cm = CompletionManager::new();
        let mut tables = HashSet::new();
        tables.insert("users".to_string());
        tables.insert("orders".to_string());
        tables.insert("products".to_string());
        cm.set_table_names(tables);

        // The query is "SELECT * FROM u" and cursor is after "u"
        // This should trigger table name completion
        cm.generate_suggestions("SELECT * FROM ", 14, "");

        assert!(cm.has_suggestions());
        assert!(cm.suggestions().contains(&"users".to_string()));
    }
}
