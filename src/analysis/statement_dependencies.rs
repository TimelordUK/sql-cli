// Statement dependency analysis for --execute-statement feature
// Analyzes SQL scripts to compute minimal execution plans based on temp table dependencies

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info};

use crate::sql::recursive_parser::{Parser, SelectStatement, SqlExpression, TableSource};

/// Information about tables referenced by a statement
#[derive(Debug, Clone, PartialEq)]
pub struct TableReferences {
    /// Tables that this statement reads from (FROM, JOIN clauses)
    pub reads: Vec<String>,
    /// Tables that this statement writes to (CREATE TEMP TABLE, INSERT, etc.)
    pub writes: Vec<String>,
}

impl TableReferences {
    fn new() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    fn add_read(&mut self, table: String) {
        if !self.reads.contains(&table) {
            self.reads.push(table);
        }
    }

    fn add_write(&mut self, table: String) {
        if !self.writes.contains(&table) {
            self.writes.push(table);
        }
    }
}

/// A parsed SQL statement with its dependencies
#[derive(Debug, Clone)]
pub struct DependencyStatement {
    /// Statement number (1-based for user display)
    pub number: usize,
    /// The SQL text
    pub sql: String,
    /// Tables this statement references
    pub references: TableReferences,
    /// Whether this is a temp table creation
    pub creates_temp_table: bool,
}

/// Execution plan for a target statement
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Statement numbers to execute in order (1-based)
    pub statements_to_execute: Vec<usize>,
    /// Statement numbers that will be skipped
    pub statements_to_skip: Vec<usize>,
    /// The target statement number
    pub target_statement: usize,
    /// Dependency graph as adjacency list (for debugging)
    /// statement_number -> [dependent_statement_numbers]
    pub dependency_graph: HashMap<usize, Vec<usize>>,
}

impl ExecutionPlan {
    /// Create a formatted debug trace of the execution plan
    pub fn format_debug_trace(&self, statements: &[DependencyStatement]) -> String {
        let mut output = Vec::new();

        output.push("=== Execution Plan Debug Trace ===\n".to_string());
        output.push(format!("Target Statement: #{}\n", self.target_statement));
        output.push(format!(
            "Statements to Execute: {:?}\n",
            self.statements_to_execute
        ));
        output.push(format!(
            "Statements to Skip: {:?}\n\n",
            self.statements_to_skip
        ));

        output.push("--- Dependency Graph ---\n".to_string());
        for stmt_num in &self.statements_to_execute {
            if let Some(stmt) = statements.iter().find(|s| s.number == *stmt_num) {
                output.push(format!("\nStatement #{}: ", stmt_num));
                if stmt.creates_temp_table {
                    output.push("[TEMP TABLE] ".to_string());
                }
                output.push(format!("\n  Reads: {:?}", stmt.references.reads));
                output.push(format!("\n  Writes: {:?}", stmt.references.writes));

                if let Some(deps) = self.dependency_graph.get(stmt_num) {
                    if !deps.is_empty() {
                        output.push(format!("\n  Depends on: {:?}", deps));
                    }
                }
                output.push("\n  SQL: ".to_string());
                output.push(
                    stmt.sql
                        .lines()
                        .map(|line| format!("    {}", line))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }

        output.push("\n\n--- Skipped Statements ---\n".to_string());
        for stmt_num in &self.statements_to_skip {
            if let Some(stmt) = statements.iter().find(|s| s.number == *stmt_num) {
                output.push(format!("\nStatement #{}: [SKIPPED]\n", stmt_num));
                output.push(format!("  Reads: {:?}\n", stmt.references.reads));
                output.push(format!("  Writes: {:?}\n", stmt.references.writes));
            }
        }

        output.join("")
    }
}

/// Dependency analyzer for SQL scripts
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Analyze a list of SQL statements and extract their dependencies
    pub fn analyze_statements(statements: &[String]) -> Result<Vec<DependencyStatement>> {
        let mut analyzed = Vec::new();

        for (idx, sql) in statements.iter().enumerate() {
            let number = idx + 1; // 1-based numbering for user display

            // Check if this creates a temp table
            let creates_temp_table = Self::is_temp_table_creation(sql);

            // Extract the SELECT portion for parsing
            let select_sql = if creates_temp_table {
                // For CREATE TEMP TABLE, extract the SELECT part after "AS"
                Self::extract_select_from_create_temp_table(sql)?
            } else {
                sql.clone()
            };

            // Parse the SELECT statement
            let mut parser = Parser::new(&select_sql);
            let ast = parser
                .parse()
                .map_err(|e| anyhow!("Failed to parse statement {}: {}", number, e))?;

            // Extract table references
            let references = Self::extract_table_references(&ast, sql)?;

            analyzed.push(DependencyStatement {
                number,
                sql: sql.clone(),
                references,
                creates_temp_table,
            });
        }

        Ok(analyzed)
    }

    /// Extract table references from a parsed AST
    fn extract_table_references(ast: &SelectStatement, sql: &str) -> Result<TableReferences> {
        let mut refs = TableReferences::new();

        // Check for CREATE TEMP TABLE (write operation)
        if Self::is_temp_table_creation(sql) {
            if let Some(table_name) = Self::extract_temp_table_name(sql) {
                refs.add_write(table_name);
            }
        }

        // Extract FROM clause tables (read operations)
        if let Some(table) = &ast.from_table {
            refs.add_read(table.clone());
        }

        // Extract from subquery if present
        if let Some(subquery) = &ast.from_subquery {
            let subquery_refs = Self::extract_table_references(subquery, "")?;
            for table in subquery_refs.reads {
                refs.add_read(table);
            }
        }

        // Extract from function if present (table functions like RANGE)
        if let Some(_function) = &ast.from_function {
            // Table functions don't reference existing tables, so nothing to extract
        }

        // Extract from JOINs
        for join in &ast.joins {
            Self::extract_from_table_source(&join.table, &mut refs)?;
        }

        // Extract from CTEs
        for cte in &ast.ctes {
            match &cte.cte_type {
                crate::sql::parser::ast::CTEType::Standard(stmt) => {
                    let cte_refs = Self::extract_table_references(stmt, "")?;
                    for table in cte_refs.reads {
                        refs.add_read(table);
                    }
                }
                _ => {} // WEB CTEs don't have dependencies on local tables
            }
        }

        // Extract from WHERE clause (for subqueries)
        if let Some(where_clause) = &ast.where_clause {
            for condition in &where_clause.conditions {
                Self::extract_from_expression(&condition.expr, &mut refs)?;
            }
        }

        Ok(refs)
    }

    /// Extract table references from a table source (handles subqueries, table functions, etc.)
    fn extract_from_table_source(
        table_source: &TableSource,
        refs: &mut TableReferences,
    ) -> Result<()> {
        match table_source {
            TableSource::Table(name) => {
                refs.add_read(name.clone());
            }
            TableSource::DerivedTable { query, .. } => {
                let subquery_refs = Self::extract_table_references(query, "")?;
                for table in subquery_refs.reads {
                    refs.add_read(table);
                }
            }
        }
        Ok(())
    }

    /// Extract table references from expressions (for subqueries in WHERE, etc.)
    fn extract_from_expression(expr: &SqlExpression, refs: &mut TableReferences) -> Result<()> {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                let subquery_refs = Self::extract_table_references(query, "")?;
                for table in subquery_refs.reads {
                    refs.add_read(table);
                }
            }
            SqlExpression::InSubquery {
                expr: inner_expr,
                subquery,
            } => {
                Self::extract_from_expression(inner_expr, refs)?;
                let subquery_refs = Self::extract_table_references(subquery, "")?;
                for table in subquery_refs.reads {
                    refs.add_read(table);
                }
            }
            SqlExpression::NotInSubquery {
                expr: inner_expr,
                subquery,
            } => {
                Self::extract_from_expression(inner_expr, refs)?;
                let subquery_refs = Self::extract_table_references(subquery, "")?;
                for table in subquery_refs.reads {
                    refs.add_read(table);
                }
            }
            SqlExpression::BinaryOp { left, right, .. } => {
                Self::extract_from_expression(left, refs)?;
                Self::extract_from_expression(right, refs)?;
            }
            SqlExpression::FunctionCall { args, .. } => {
                for arg in args {
                    Self::extract_from_expression(arg, refs)?;
                }
            }
            SqlExpression::WindowFunction { args, .. } => {
                for arg in args {
                    Self::extract_from_expression(arg, refs)?;
                }
            }
            SqlExpression::MethodCall { args, .. } => {
                for arg in args {
                    Self::extract_from_expression(arg, refs)?;
                }
            }
            SqlExpression::ChainedMethodCall { base, args, .. } => {
                Self::extract_from_expression(base, refs)?;
                for arg in args {
                    Self::extract_from_expression(arg, refs)?;
                }
            }
            _ => {} // Other expression types don't contain table references
        }
        Ok(())
    }

    /// Check if SQL creates a temp table
    fn is_temp_table_creation(sql: &str) -> bool {
        let sql_lower = sql.to_lowercase();
        sql_lower.contains("create temp table")
            || sql_lower.contains("create temporary table")
            || sql_lower.contains("into #") // SELECT ... INTO #temp_table syntax (can have newline before)
    }

    /// Extract the SELECT portion from a CREATE TEMP TABLE statement
    /// Example: "CREATE TEMP TABLE foo AS SELECT * FROM bar" -> "SELECT * FROM bar"
    /// Also handles: "SELECT * INTO #foo FROM bar" -> "SELECT * FROM bar"
    fn extract_select_from_create_temp_table(sql: &str) -> Result<String> {
        let sql_lower = sql.to_lowercase();

        // Check if this is SELECT ... INTO #temp syntax
        if sql_lower.contains("into #") {
            // For SELECT INTO, we need to remove the "INTO #table_name" part
            // Pattern: SELECT ... INTO #table_name FROM ...
            // We want: SELECT ... FROM ...

            let into_pos = sql_lower
                .find("into #")
                .ok_or_else(|| anyhow!("Could not find INTO # in statement"))?;

            // Find the start of the line containing INTO (to handle newlines)
            let before_into = &sql_lower[..into_pos];
            let line_start = before_into.rfind('\n').map(|p| p + 1).unwrap_or(0);

            // Skip any whitespace before INTO on the same line
            let into_line_start = sql_lower[line_start..into_pos]
                .trim_start()
                .is_empty()
                .then_some(line_start)
                .unwrap_or(into_pos);

            // Find the end of the table name (next whitespace after INTO #)
            let after_into = &sql_lower[into_pos + 6..]; // skip "into #" (6 chars)
            let table_end = after_into
                .find(char::is_whitespace)
                .ok_or_else(|| anyhow!("Could not find end of table name in SELECT INTO"))?;

            let from_start_in_remaining = into_pos + 6 + table_end;

            // Reconstruct: SELECT ... (before INTO line) + (after table name)
            let select_part = sql[..into_line_start].trim();
            let from_part = sql[from_start_in_remaining..].trim();

            Ok(format!("{} {}", select_part, from_part))
        } else {
            // Original CREATE TEMP TABLE AS SELECT syntax
            // Find the position of "AS" keyword
            let as_pos = sql_lower
                .find(" as ")
                .ok_or_else(|| anyhow!("CREATE TEMP TABLE statement must have AS keyword"))?;

            // Extract everything after "AS"
            let select_start = as_pos + 4; // " as " is 4 characters
            let select_sql = sql[select_start..].trim();

            if select_sql.is_empty() {
                return Err(anyhow!(
                    "CREATE TEMP TABLE statement has no SELECT after AS"
                ));
            }

            Ok(select_sql.to_string())
        }
    }

    /// Extract temp table name from CREATE TEMP TABLE statement or SELECT INTO
    fn extract_temp_table_name(sql: &str) -> Option<String> {
        let sql_lower = sql.to_lowercase();

        // Check for SELECT ... INTO #table syntax first
        if sql_lower.contains("into #") {
            let into_pos = sql_lower.find("into #")?;
            let after_into = &sql[into_pos + 6..]; // skip "into #" (6 chars)

            // Extract table name (everything until next whitespace)
            let table_name = after_into
                .split_whitespace()
                .next()?
                .trim_end_matches(|c: char| c == '(' || c == ';');

            return Some(format!("#{}", table_name)); // Include the # prefix
        }

        // Find "create temp table" or "create temporary table"
        let start_pattern = if sql_lower.contains("create temp table") {
            "create temp table"
        } else if sql_lower.contains("create temporary table") {
            "create temporary table"
        } else {
            return None;
        };

        // Find the position after the pattern
        let start_idx = sql_lower.find(start_pattern)? + start_pattern.len();

        // Extract the table name (next word after the pattern)
        let remaining = sql[start_idx..].trim();
        let table_name = remaining
            .split_whitespace()
            .next()?
            .trim_end_matches(|c: char| c == '(' || c == ';');

        Some(table_name.to_string())
    }

    /// Compute execution plan for a target statement
    /// Returns the minimal set of statements needed to execute the target
    pub fn compute_execution_plan(
        statements: &[DependencyStatement],
        target_statement_number: usize,
    ) -> Result<ExecutionPlan> {
        if target_statement_number == 0 || target_statement_number > statements.len() {
            return Err(anyhow!(
                "Invalid target statement number: {}. Must be 1-{}",
                target_statement_number,
                statements.len()
            ));
        }

        info!(
            "Computing execution plan for statement #{}",
            target_statement_number
        );

        // Build dependency graph: statement_number -> [depends_on_statement_numbers]
        let mut dependency_graph: HashMap<usize, Vec<usize>> = HashMap::new();

        // Build a map of table_name -> statement_number that creates it
        let mut table_creators: HashMap<String, usize> = HashMap::new();

        for stmt in statements {
            // Register temp tables this statement creates
            for table in &stmt.references.writes {
                table_creators.insert(table.clone(), stmt.number);
            }

            // Find dependencies for tables this statement reads
            let mut depends_on = Vec::new();
            for table in &stmt.references.reads {
                // Look for the latest statement that creates this table (before current statement)
                for candidate in statements {
                    if candidate.number >= stmt.number {
                        break; // Only look at earlier statements
                    }
                    if candidate.references.writes.contains(table) {
                        if !depends_on.contains(&candidate.number) {
                            depends_on.push(candidate.number);
                        }
                    }
                }
            }

            if !depends_on.is_empty() {
                dependency_graph.insert(stmt.number, depends_on);
            }
        }

        debug!("Dependency graph: {:?}", dependency_graph);

        // Compute transitive dependencies using BFS
        let mut to_execute = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(target_statement_number);

        while let Some(stmt_num) = queue.pop_front() {
            if to_execute.insert(stmt_num) {
                // First time seeing this statement, add its dependencies to queue
                if let Some(deps) = dependency_graph.get(&stmt_num) {
                    for &dep in deps {
                        queue.push_back(dep);
                    }
                }
            }
        }

        // Sort statements to execute in order
        let mut statements_to_execute: Vec<usize> = to_execute.into_iter().collect();
        statements_to_execute.sort_unstable();

        // Compute skipped statements
        let statements_to_skip: Vec<usize> = (1..=statements.len())
            .filter(|n| !statements_to_execute.contains(n))
            .collect();

        info!(
            "Execution plan: execute {:?}, skip {:?}",
            statements_to_execute, statements_to_skip
        );

        Ok(ExecutionPlan {
            statements_to_execute,
            statements_to_skip,
            target_statement: target_statement_number,
            dependency_graph,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_temp_table_name() {
        let sql1 = "CREATE TEMP TABLE foo AS SELECT * FROM bar";
        assert_eq!(
            DependencyAnalyzer::extract_temp_table_name(sql1),
            Some("foo".to_string())
        );

        let sql2 = "CREATE TEMPORARY TABLE my_temp AS SELECT 1";
        assert_eq!(
            DependencyAnalyzer::extract_temp_table_name(sql2),
            Some("my_temp".to_string())
        );

        let sql3 = "SELECT * FROM baz";
        assert_eq!(DependencyAnalyzer::extract_temp_table_name(sql3), None);
    }

    #[test]
    fn test_simple_dependency() {
        let statements = vec![
            "CREATE TEMP TABLE raw_data AS SELECT * FROM sales".to_string(),
            "SELECT COUNT(*) FROM customers".to_string(),
            "SELECT * FROM raw_data WHERE amount > 100".to_string(),
        ];

        let analyzed = DependencyAnalyzer::analyze_statements(&statements).unwrap();
        assert_eq!(analyzed.len(), 3);

        // Statement 1: creates raw_data, reads sales
        assert_eq!(analyzed[0].references.writes, vec!["raw_data"]);
        assert_eq!(analyzed[0].references.reads, vec!["sales"]);

        // Statement 2: reads customers (independent)
        assert_eq!(analyzed[1].references.reads, vec!["customers"]);
        assert!(analyzed[1].references.writes.is_empty());

        // Statement 3: reads raw_data
        assert_eq!(analyzed[2].references.reads, vec!["raw_data"]);
    }

    #[test]
    fn test_execution_plan() {
        let statements = vec![
            "CREATE TEMP TABLE raw_data AS SELECT * FROM sales".to_string(),
            "SELECT COUNT(*) FROM customers".to_string(),
            "SELECT * FROM raw_data WHERE amount > 100".to_string(),
        ];

        let analyzed = DependencyAnalyzer::analyze_statements(&statements).unwrap();
        let plan = DependencyAnalyzer::compute_execution_plan(&analyzed, 3).unwrap();

        // Should execute statement 1 (creates raw_data) and 3 (target)
        // Should skip statement 2 (independent)
        assert_eq!(plan.statements_to_execute, vec![1, 3]);
        assert_eq!(plan.statements_to_skip, vec![2]);
        assert_eq!(plan.target_statement, 3);
    }

    #[test]
    fn test_transitive_dependencies() {
        let statements = vec![
            "CREATE TEMP TABLE t1 AS SELECT * FROM base".to_string(),
            "CREATE TEMP TABLE t2 AS SELECT * FROM t1".to_string(),
            "CREATE TEMP TABLE t3 AS SELECT * FROM t2".to_string(),
            "SELECT * FROM unrelated".to_string(),
            "SELECT * FROM t3".to_string(),
        ];

        let analyzed = DependencyAnalyzer::analyze_statements(&statements).unwrap();
        let plan = DependencyAnalyzer::compute_execution_plan(&analyzed, 5).unwrap();

        // Should execute 1 -> 2 -> 3 -> 5 (transitive chain)
        // Should skip 4 (unrelated)
        assert_eq!(plan.statements_to_execute, vec![1, 2, 3, 5]);
        assert_eq!(plan.statements_to_skip, vec![4]);
    }

    #[test]
    fn test_invalid_statement_number() {
        let statements = vec!["SELECT 1".to_string()];
        let analyzed = DependencyAnalyzer::analyze_statements(&statements).unwrap();

        // Test statement number 0
        assert!(DependencyAnalyzer::compute_execution_plan(&analyzed, 0).is_err());

        // Test statement number > len
        assert!(DependencyAnalyzer::compute_execution_plan(&analyzed, 5).is_err());
    }
}
