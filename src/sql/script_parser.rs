// Script parser for handling multi-statement SQL scripts with GO separator
// Similar to SQL Server's batch execution model

use anyhow::Result;

/// Directives that can be attached to a script statement
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptDirective {
    /// Skip execution of this statement
    Skip,
}

/// Type of script statement
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptStatementType {
    /// Regular SQL query
    Query(String),
    /// EXIT statement - stops script execution
    /// Optional exit code (defaults to 0 for success)
    Exit(Option<i32>),
}

/// A parsed script statement with optional directives
#[derive(Debug, Clone)]
pub struct ScriptStatement {
    /// The type of statement (Query or Exit)
    pub statement_type: ScriptStatementType,
    /// Directives attached to this statement (from comments above it)
    pub directives: Vec<ScriptDirective>,
}

impl ScriptStatement {
    /// Check if this statement should be skipped
    pub fn should_skip(&self) -> bool {
        self.directives.contains(&ScriptDirective::Skip)
    }

    /// Check if this is an EXIT statement
    pub fn is_exit(&self) -> bool {
        matches!(self.statement_type, ScriptStatementType::Exit(_))
    }

    /// Get exit code if this is an EXIT statement
    pub fn get_exit_code(&self) -> Option<i32> {
        match &self.statement_type {
            ScriptStatementType::Exit(code) => Some(code.unwrap_or(0)),
            _ => None,
        }
    }

    /// Get the SQL query if this is a query statement
    pub fn get_query(&self) -> Option<&str> {
        match &self.statement_type {
            ScriptStatementType::Query(sql) => Some(sql),
            ScriptStatementType::Exit(_) => None,
        }
    }
}

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

    /// Parse directives from comment lines
    /// Looks for patterns like: -- [SKIP], -- [TODO], etc.
    fn parse_directives(comment_lines: &[String]) -> Vec<ScriptDirective> {
        let mut directives = Vec::new();

        for line in comment_lines {
            let trimmed = line.trim();
            if !trimmed.starts_with("--") {
                continue;
            }

            let comment_content = trimmed.strip_prefix("--").unwrap().trim();

            // Check for directive patterns: [SKIP], [IGNORE]
            if comment_content.eq_ignore_ascii_case("[skip]")
                || comment_content.eq_ignore_ascii_case("[ignore]")
            {
                directives.push(ScriptDirective::Skip);
            }
        }

        directives
    }

    /// Split a `GO` batch into individual statements on top-level `;`.
    ///
    /// `GO` remains the batch separator it has always been — nothing about
    /// existing scripts changes shape. This only recovers statements that were
    /// previously glued together into one string, where the parser would parse
    /// the first and **silently discard the rest** (P13). `prime_numbers.sql`
    /// had a whole `SELECT` that never ran for exactly this reason.
    ///
    /// Quote- and comment-aware, so a `;` inside a string literal, a quoted
    /// identifier, a line comment or a block comment does not split. Doubled
    /// quotes (`'O''Brien'`) are handled as the escape they are rather than as
    /// a close-then-reopen.
    ///
    /// Statement *scope* is unaffected: the script executor builds one
    /// `ExecutionContext` for the whole file, so `SELECT ... INTO #tmp` stays
    /// visible to later statements whether they are separated by `;` or `GO`.
    fn split_on_semicolons(batch: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = String::new();
        let mut chars = batch.chars().peekable();
        let mut in_single = false;
        let mut in_double = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        while let Some(ch) = chars.next() {
            if in_line_comment {
                current.push(ch);
                if ch == '\n' {
                    in_line_comment = false;
                }
                continue;
            }
            if in_block_comment {
                current.push(ch);
                if ch == '*' && chars.peek() == Some(&'/') {
                    current.push(chars.next().unwrap());
                    in_block_comment = false;
                }
                continue;
            }
            if in_single || in_double {
                let quote = if in_single { '\'' } else { '"' };
                current.push(ch);
                if ch == quote {
                    if chars.peek() == Some(&quote) {
                        current.push(chars.next().unwrap()); // escaped quote
                    } else if in_single {
                        in_single = false;
                    } else {
                        in_double = false;
                    }
                }
                continue;
            }

            match ch {
                '\'' => {
                    in_single = true;
                    current.push(ch);
                }
                '"' => {
                    in_double = true;
                    current.push(ch);
                }
                '-' if chars.peek() == Some(&'-') => {
                    in_line_comment = true;
                    current.push(ch);
                }
                '/' if chars.peek() == Some(&'*') => {
                    in_block_comment = true;
                    current.push(ch);
                }
                ';' => {
                    let stmt = current.trim().to_string();
                    if !stmt.is_empty() {
                        out.push(stmt);
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        let last = current.trim().to_string();
        if !last.is_empty() {
            out.push(last);
        }
        out
    }

    /// True if the text holds more than one top-level statement, i.e. it needs
    /// the script executor even though it has no `GO` separator.
    ///
    /// Without this a `;`-separated file routes to the single-query path, which
    /// parses the whole thing as one statement — historically running only the
    /// first and silently dropping the rest (P13).
    #[must_use]
    pub fn is_multi_statement(sql: &str) -> bool {
        Self::split_on_semicolons(sql)
            .iter()
            .filter(|s| !Self::is_comment_only(s))
            .count()
            > 1
    }

    /// Turn one accumulated batch into zero or more `ScriptStatement`s.
    /// Batch-level directives (e.g. `-- [SKIP]`) apply to every statement in it.
    fn push_batch(batch: &str, pending_comments: &[String], statements: &mut Vec<ScriptStatement>) {
        let batch = batch.trim();
        if batch.is_empty() || Self::is_comment_only(batch) {
            return;
        }

        let directives = Self::parse_directives(pending_comments);

        for stmt in Self::split_on_semicolons(batch) {
            if Self::is_comment_only(&stmt) {
                continue;
            }
            let statement_type =
                Self::parse_exit_statement(&stmt).unwrap_or(ScriptStatementType::Query(stmt));
            statements.push(ScriptStatement {
                statement_type,
                directives: directives.clone(),
            });
        }
    }

    /// Parse the script into ScriptStatements with directives
    /// GO must be on its own line (case-insensitive)
    pub fn parse_script_statements(&self) -> Vec<ScriptStatement> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut pending_comments = Vec::new();

        for line in self.content.lines() {
            let trimmed = line.trim();

            // Check if this line is just "GO" (case-insensitive)
            if trimmed.eq_ignore_ascii_case("go") {
                Self::push_batch(&current_statement, &pending_comments, &mut statements);
                current_statement.clear();
                pending_comments.clear();
            } else if trimmed.starts_with("--") {
                // This is a comment line - save it for directive parsing
                pending_comments.push(line.to_string());
                // Also add to current statement
                if !current_statement.is_empty() {
                    current_statement.push('\n');
                }
                current_statement.push_str(line);
            } else {
                // Regular line - add to current statement
                if !current_statement.is_empty() {
                    current_statement.push('\n');
                }
                current_statement.push_str(line);
            }
        }

        // Don't forget the last batch if there's no trailing GO
        Self::push_batch(&current_statement, &pending_comments, &mut statements);

        statements
    }

    /// Try to parse an EXIT statement with optional exit code
    /// Supports: EXIT, EXIT;, EXIT 0, EXIT 1;, etc.
    /// Strips comments before checking
    fn parse_exit_statement(statement: &str) -> Option<ScriptStatementType> {
        // Extract non-comment content
        let mut non_comment_lines = Vec::new();
        for line in statement.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                non_comment_lines.push(trimmed);
            }
        }

        if non_comment_lines.is_empty() {
            return None;
        }

        // Join non-comment lines and check if it's EXIT
        let content = non_comment_lines.join(" ");
        let trimmed = content.trim().trim_end_matches(';').trim();

        if trimmed.eq_ignore_ascii_case("exit") {
            return Some(ScriptStatementType::Exit(None));
        }

        // Check for EXIT with a number: EXIT 0, EXIT 1, etc.
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 2 && parts[0].eq_ignore_ascii_case("exit") {
            if let Ok(code) = parts[1].parse::<i32>() {
                return Some(ScriptStatementType::Exit(Some(code)));
            }
        }

        None
    }

    /// Parse the script into individual SQL statements (legacy method)
    /// GO must be on its own line (case-insensitive)
    /// Returns a vector of SQL statements to execute
    pub fn parse_statements(&self) -> Vec<String> {
        self.parse_script_statements()
            .into_iter()
            .filter_map(|stmt| match stmt.statement_type {
                ScriptStatementType::Query(sql) => Some(sql),
                ScriptStatementType::Exit(_) => None,
            })
            .collect()
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
