// Script parser for handling multi-statement SQL scripts with GO separator
// Similar to SQL Server's batch execution model

use anyhow::Result;

/// Parses SQL scripts into individual statements using GO as separator
pub struct ScriptParser {
    content: String,
    data_file_hint: Option<String>,
}

impl ScriptParser {
    /// Create a new script parser with the given content
    pub fn new(content: &str) -> Self {
        let data_file_hint = Self::extract_data_file_hint(content);
        Self {
            content: content.to_string(),
            data_file_hint,
        }
    }

    /// Extract data file hint from script comments
    /// Looks for patterns like:
    /// -- #!data: path/to/file.csv
    /// -- #!datafile: path/to/file.csv  
    /// -- #! /path/to/file.csv
    fn extract_data_file_hint(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip non-comment lines
            if !trimmed.starts_with("--") {
                continue;
            }

            // Remove the comment prefix
            let comment_content = trimmed.strip_prefix("--").unwrap().trim();

            // Check for data file hint patterns
            if let Some(path) = comment_content.strip_prefix("#!data:") {
                return Some(path.trim().to_string());
            }
            if let Some(path) = comment_content.strip_prefix("#!datafile:") {
                return Some(path.trim().to_string());
            }
            if let Some(path) = comment_content.strip_prefix("#!") {
                let path = path.trim();
                // Check if it looks like a file path
                if path.contains('.') || path.contains('/') || path.contains('\\') {
                    return Some(path.to_string());
                }
            }
        }
        None
    }

    /// Get the data file hint if present
    pub fn data_file_hint(&self) -> Option<&str> {
        self.data_file_hint.as_deref()
    }

    /// Parse the script into individual SQL statements
    /// GO must be on its own line (case-insensitive)
    /// Returns a vector of SQL statements to execute
    pub fn parse_statements(&self) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();

        for line in self.content.lines() {
            let trimmed = line.trim();

            // Check if this line is just "GO" (case-insensitive)
            if trimmed.eq_ignore_ascii_case("go") {
                // Add the current statement if it's not empty or just comments
                let statement = current_statement.trim().to_string();
                if !statement.is_empty() && !Self::is_comment_only(&statement) {
                    statements.push(statement);
                }
                current_statement.clear();
            } else {
                // Add this line to the current statement
                if !current_statement.is_empty() {
                    current_statement.push('\n');
                }
                current_statement.push_str(line);
            }
        }

        // Don't forget the last statement if there's no trailing GO
        let statement = current_statement.trim().to_string();
        if !statement.is_empty() && !Self::is_comment_only(&statement) {
            statements.push(statement);
        }

        statements
    }

    /// Check if a statement contains only comments (no actual SQL)
    fn is_comment_only(statement: &str) -> bool {
        for line in statement.lines() {
            let trimmed = line.trim();
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            // If we find any non-comment content, it's not comment-only
            return false;
        }
        // All lines were comments or empty
        true
    }

    /// Parse and validate that all statements are valid SQL
    /// Returns the statements or an error if any are invalid
    pub fn parse_and_validate(&self) -> Result<Vec<String>> {
        let statements = self.parse_statements();

        if statements.is_empty() {
            anyhow::bail!("No SQL statements found in script");
        }

        // Basic validation - ensure no statement is just whitespace
        for (i, stmt) in statements.iter().enumerate() {
            if stmt.trim().is_empty() {
                anyhow::bail!("Empty statement at position {}", i + 1);
            }
        }

        Ok(statements)
    }
}

/// Result of executing a single statement in a script
#[derive(Debug)]
pub struct StatementResult {
    pub statement_number: usize,
    pub sql: String,
    pub success: bool,
    pub rows_affected: usize,
    pub error_message: Option<String>,
    pub execution_time_ms: f64,
}

/// Result of executing an entire script
#[derive(Debug)]
pub struct ScriptResult {
    pub total_statements: usize,
    pub successful_statements: usize,
    pub failed_statements: usize,
    pub total_execution_time_ms: f64,
    pub statement_results: Vec<StatementResult>,
}

impl ScriptResult {
    pub fn new() -> Self {
        Self {
            total_statements: 0,
            successful_statements: 0,
            failed_statements: 0,
            total_execution_time_ms: 0.0,
            statement_results: Vec::new(),
        }
    }

    pub fn add_success(&mut self, statement_number: usize, sql: String, rows: usize, time_ms: f64) {
        self.total_statements += 1;
        self.successful_statements += 1;
        self.total_execution_time_ms += time_ms;

        self.statement_results.push(StatementResult {
            statement_number,
            sql,
            success: true,
            rows_affected: rows,
            error_message: None,
            execution_time_ms: time_ms,
        });
    }

    pub fn add_failure(
        &mut self,
        statement_number: usize,
        sql: String,
        error: String,
        time_ms: f64,
    ) {
        self.total_statements += 1;
        self.failed_statements += 1;
        self.total_execution_time_ms += time_ms;

        self.statement_results.push(StatementResult {
            statement_number,
            sql,
            success: false,
            rows_affected: 0,
            error_message: Some(error),
            execution_time_ms: time_ms,
        });
    }

    pub fn all_successful(&self) -> bool {
        self.failed_statements == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_statement() {
        let script = "SELECT * FROM users";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], "SELECT * FROM users");
    }

    #[test]
    fn test_parse_multiple_statements_with_go() {
        let script = r"
SELECT * FROM users
GO
SELECT * FROM orders
GO
SELECT * FROM products
";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 3);
        assert_eq!(statements[0].trim(), "SELECT * FROM users");
        assert_eq!(statements[1].trim(), "SELECT * FROM orders");
        assert_eq!(statements[2].trim(), "SELECT * FROM products");
    }

    #[test]
    fn test_go_case_insensitive() {
        let script = r"
SELECT 1
go
SELECT 2
Go
SELECT 3
GO
";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 3);
    }

    #[test]
    fn test_go_in_string_not_separator() {
        let script = r"
SELECT 'This string contains GO but should not split' as test
GO
SELECT 'Another statement' as test2
";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("GO but should not split"));
    }

    #[test]
    fn test_multiline_statements() {
        let script = r"
SELECT 
    id,
    name,
    email
FROM users
WHERE active = true
GO
SELECT COUNT(*) 
FROM orders
";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("WHERE active = true"));
    }

    #[test]
    fn test_empty_statements_filtered() {
        let script = r"
GO
SELECT 1
GO
GO
SELECT 2
GO
";
        let parser = ScriptParser::new(script);
        let statements = parser.parse_statements();

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].trim(), "SELECT 1");
        assert_eq!(statements[1].trim(), "SELECT 2");
    }
}
