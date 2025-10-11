use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::sql::parser::ast::SelectStatement;
use crate::sql::recursive_parser::Parser;
use crate::sql::script_parser::ScriptParser;

/// Represents a single statement in the script with its dependencies
#[derive(Debug, Clone)]
pub struct StatementNode {
    /// 1-based index in the script
    pub index: usize,
    /// The SQL text of this statement
    pub sql: String,
    /// Temporary tables this statement creates (e.g., ["#temp", "#summary"])
    pub creates_tables: Vec<String>,
    /// Tables this statement depends on (both temp tables and base tables)
    pub depends_on_tables: Vec<String>,
    /// The parsed AST (if parsing succeeded)
    pub ast: Option<SelectStatement>,
}

/// Dependency graph for a SQL script (temp table dependencies between statements)
#[derive(Debug)]
pub struct ScriptDependencyGraph {
    /// All statements in the script
    pub statements: Vec<StatementNode>,
    /// Map of table name -> statement index that creates it
    pub table_creators: HashMap<String, usize>,
}

impl ScriptDependencyGraph {
    /// Analyze a SQL script and build the dependency graph
    ///
    /// This parses the entire script, identifies what tables each statement
    /// creates and depends on, and builds a graph of dependencies.
    pub fn analyze(script_content: &str) -> Result<Self> {
        let script_parser = ScriptParser::new(script_content);
        let script_statements = script_parser.parse_script_statements();

        let mut statements = Vec::new();
        let mut table_creators = HashMap::new();

        for (idx, script_stmt) in script_statements.iter().enumerate() {
            let statement_num = idx + 1;

            // Skip EXIT and [SKIP] directives
            if script_stmt.is_exit() || script_stmt.should_skip() {
                continue;
            }

            // Get the SQL query
            let sql = match script_stmt.get_query() {
                Some(s) => s.to_string(),
                None => continue,
            };

            // Parse the statement to extract dependencies
            let mut parser = Parser::new(&sql);
            let ast = parser.parse().ok();

            let mut creates_tables = Vec::new();
            let mut depends_on_tables = Vec::new();

            if let Some(ref stmt) = ast {
                // Check if this statement creates a temp table (INTO clause)
                if let Some(ref into_table) = stmt.into_table {
                    creates_tables.push(into_table.name.clone());
                    table_creators.insert(into_table.name.clone(), statement_num);
                }

                // Check what tables this statement depends on
                if let Some(ref from_table) = stmt.from_table {
                    depends_on_tables.push(from_table.clone());
                }

                // Check JOIN clauses for dependencies
                for join in &stmt.joins {
                    if let crate::sql::parser::ast::TableSource::Table(table_name) = &join.table {
                        depends_on_tables.push(table_name.clone());
                    }
                }

                // TODO: Could also check subqueries, CTEs, etc. for more complete analysis
            }

            statements.push(StatementNode {
                index: statement_num,
                sql,
                creates_tables,
                depends_on_tables,
                ast,
            });
        }

        Ok(Self {
            statements,
            table_creators,
        })
    }

    /// Get the minimal set of statement indices needed to execute a target statement
    ///
    /// Returns statements in execution order (dependencies first, target last)
    pub fn get_dependencies(&self, target_index: usize) -> Result<Vec<usize>> {
        if target_index == 0 || target_index > self.statements.len() {
            anyhow::bail!(
                "Invalid statement index: {}. Script has {} statements.",
                target_index,
                self.statements.len()
            );
        }

        let mut required = HashSet::new();
        let mut to_process = vec![target_index];

        // Traverse dependency graph backwards
        while let Some(stmt_idx) = to_process.pop() {
            if required.contains(&stmt_idx) {
                continue; // Already processed
            }

            required.insert(stmt_idx);

            // Find the statement (0-indexed in Vec, but stmt_idx is 1-indexed)
            if let Some(stmt) = self.statements.iter().find(|s| s.index == stmt_idx) {
                // For each table this statement depends on
                for table in &stmt.depends_on_tables {
                    // If it's a temp table, find who creates it
                    if table.starts_with('#') {
                        if let Some(&creator_idx) = self.table_creators.get(table) {
                            to_process.push(creator_idx);
                        }
                        // If temp table is not found, it will error during execution
                    }
                    // Base tables don't need dependency tracking
                }
            }
        }

        // Convert to sorted vector (execution order)
        let mut result: Vec<usize> = required.into_iter().collect();
        result.sort();

        Ok(result)
    }

    /// Generate a debug report showing the dependency analysis
    pub fn explain_dependencies(&self, target_index: usize) -> Result<String> {
        let deps = self.get_dependencies(target_index)?;

        let mut output = String::new();
        output.push_str("\n=== Dependency Analysis ===\n");
        output.push_str(&format!(
            "Script has {} statements total\n",
            self.statements.len()
        ));
        output.push_str(&format!("Target: Statement {}\n\n", target_index));

        // Show all relevant statements
        for &stmt_idx in &deps {
            if let Some(stmt) = self.statements.iter().find(|s| s.index == stmt_idx) {
                let is_target = stmt_idx == target_index;
                let marker = if is_target { " [TARGET]" } else { "" };

                output.push_str(&format!("Statement {}{}\n", stmt_idx, marker));

                // Show abbreviated SQL (first 60 chars)
                let sql_preview = if stmt.sql.len() > 60 {
                    format!("{}...", &stmt.sql[..60])
                } else {
                    stmt.sql.clone()
                };
                output.push_str(&format!("  SQL: {}\n", sql_preview.replace('\n', " ")));

                if !stmt.creates_tables.is_empty() {
                    output.push_str(&format!("  Creates: {}\n", stmt.creates_tables.join(", ")));
                }

                if !stmt.depends_on_tables.is_empty() {
                    output.push_str(&format!(
                        "  Depends on: {}\n",
                        stmt.depends_on_tables.join(", ")
                    ));
                }

                output.push('\n');
            }
        }

        output.push_str("Execution Plan:\n");
        for &stmt_idx in &deps {
            let marker = if stmt_idx == target_index {
                " ← target"
            } else {
                ""
            };
            output.push_str(&format!("  → Statement {}{}\n", stmt_idx, marker));
        }

        output.push_str(&format!(
            "\nExecuting {} of {} statements...\n",
            deps.len(),
            self.statements.len()
        ));

        Ok(output)
    }

    /// Get a statement by its index
    pub fn get_statement(&self, index: usize) -> Option<&StatementNode> {
        self.statements.iter().find(|s| s.index == index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_dependency_chain() {
        let script = r#"
SELECT * INTO #temp FROM data WHERE value > 100;
GO

SELECT * INTO #summary FROM #temp GROUP BY category;
GO

SELECT * FROM #summary WHERE total > 500;
GO
"#;

        let graph = ScriptDependencyGraph::analyze(script).unwrap();

        // Should have 3 statements
        assert_eq!(graph.statements.len(), 3);

        // Statement 1 creates #temp
        assert_eq!(graph.statements[0].creates_tables, vec!["#temp"]);

        // Statement 2 depends on #temp and creates #summary
        assert!(graph.statements[1]
            .depends_on_tables
            .contains(&"#temp".to_string()));
        assert_eq!(graph.statements[1].creates_tables, vec!["#summary"]);

        // Statement 3 depends on #summary
        assert!(graph.statements[2]
            .depends_on_tables
            .contains(&"#summary".to_string()));

        // Get dependencies for statement 3
        let deps = graph.get_dependencies(3).unwrap();
        assert_eq!(deps, vec![1, 2, 3]); // All three statements needed
    }

    #[test]
    fn test_independent_statements() {
        let script = r#"
SELECT * FROM data1;
GO

SELECT * FROM data2;
GO

SELECT * FROM data3;
GO
"#;

        let graph = ScriptDependencyGraph::analyze(script).unwrap();
        assert_eq!(graph.statements.len(), 3);

        // Statement 3 only needs itself (no temp tables)
        let deps = graph.get_dependencies(3).unwrap();
        assert_eq!(deps, vec![3]);
    }

    #[test]
    fn test_partial_dependency() {
        let script = r#"
SELECT * INTO #temp1 FROM data;
GO

SELECT * INTO #temp2 FROM data;
GO

SELECT * FROM #temp2;
GO
"#;

        let graph = ScriptDependencyGraph::analyze(script).unwrap();

        // Statement 3 only needs statement 2 (creates #temp2)
        let deps = graph.get_dependencies(3).unwrap();
        assert_eq!(deps, vec![2, 3]);
    }

    #[test]
    fn test_explain_output() {
        let script = r#"
SELECT * INTO #temp FROM data;
GO

SELECT * FROM #temp;
GO
"#;

        let graph = ScriptDependencyGraph::analyze(script).unwrap();
        let explanation = graph.explain_dependencies(2).unwrap();

        // Should mention both statements
        assert!(explanation.contains("Statement 1"));
        assert!(explanation.contains("Statement 2"));
        assert!(explanation.contains("[TARGET]"));
        assert!(explanation.contains("Creates: #temp"));
        assert!(explanation.contains("Depends on: #temp"));
    }
}
