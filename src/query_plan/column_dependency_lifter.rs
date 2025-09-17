use crate::sql::parser::ast::{
    SelectStatement, SelectItem, SqlExpression, OrderByColumn, CTE, CTEType
};
use std::collections::{HashMap, HashSet};

/// Column Dependency Lifter - Rewrites queries to handle column alias dependencies
///
/// This preprocessor automatically generates CTEs when a computed column is referenced
/// in the same SELECT statement (e.g., in window functions, GROUP BY, or ORDER BY).
///
/// Example transformation:
/// ```sql
/// -- Input (illegal - can't reference 'root' alias in PARTITION BY):
/// SELECT
///     CASE WHEN CONTAINS(id, '|') THEN SUBSTRING_AFTER(id, '|', 1) ELSE id END AS root,
///     ROW_NUMBER() OVER (PARTITION BY root) as rn
/// FROM table
///
/// -- Output (legal - CTE provides 'root' for use):
/// WITH __lifted_cols AS (
///     SELECT
///         *,
///         CASE WHEN CONTAINS(id, '|') THEN SUBSTRING_AFTER(id, '|', 1) ELSE id END AS root
///     FROM table
/// )
/// SELECT
///     root,
///     ROW_NUMBER() OVER (PARTITION BY root) as rn
/// FROM __lifted_cols
/// ```
pub struct ColumnDependencyLifter {
    cte_counter: usize,
}

impl ColumnDependencyLifter {
    pub fn new() -> Self {
        Self { cte_counter: 0 }
    }

    /// Check if a statement needs column dependency lifting and rewrite if necessary
    pub fn lift_dependencies(&mut self, statement: &mut SelectStatement) -> bool {
        // Find all column aliases defined in SELECT
        let column_aliases = self.extract_column_aliases(statement);

        // Find all references to these aliases in window functions, GROUP BY, etc.
        let dependencies = self.find_alias_dependencies(statement, &column_aliases);

        if dependencies.is_empty() {
            return false;
        }

        // Generate a CTE to compute the dependent columns
        let lifted_cte = self.generate_lifted_cte(statement, &dependencies);

        // Rewrite the main query to use the CTE
        self.rewrite_query_with_cte(statement, lifted_cte, &dependencies);

        true
    }

    /// Extract all column aliases from SELECT items
    fn extract_column_aliases(&self, statement: &SelectStatement) -> HashMap<String, SqlExpression> {
        let mut aliases = HashMap::new();

        for item in &statement.select_items {
            match item {
                SelectItem::Expression { expr, alias } => {
                    aliases.insert(alias.clone(), expr.clone());
                }
                SelectItem::Column(name) => {
                    // Simple columns might be aliased implicitly
                    if let SqlExpression::Column(col) = SqlExpression::Column(name.clone()) {
                        if col != name {
                            aliases.insert(name.clone(), SqlExpression::Column(col));
                        }
                    }
                }
                _ => {}
            }
        }

        aliases
    }

    /// Find dependencies on column aliases in window functions, GROUP BY, etc.
    fn find_alias_dependencies(
        &self,
        statement: &SelectStatement,
        aliases: &HashMap<String, SqlExpression>
    ) -> HashSet<String> {
        let mut dependencies = HashSet::new();

        // Check window functions in SELECT items
        for item in &statement.select_items {
            if let SelectItem::Expression { expr, .. } = item {
                self.find_deps_in_expression(expr, aliases, &mut dependencies);
            }
        }

        // Check GROUP BY
        if let Some(group_by) = &statement.group_by {
            for col in group_by {
                if aliases.contains_key(col) {
                    dependencies.insert(col.clone());
                }
            }
        }

        // Check ORDER BY
        if let Some(order_by) = &statement.order_by {
            for order_col in order_by {
                if let SqlExpression::Column(col) = &order_col.column {
                    if aliases.contains_key(col) {
                        dependencies.insert(col.clone());
                    }
                }
            }
        }

        // Check HAVING clause
        if let Some(having) = &statement.having {
            self.find_deps_in_expression(having, aliases, &mut dependencies);
        }

        dependencies
    }

    /// Recursively find dependencies in an expression
    fn find_deps_in_expression(
        &self,
        expr: &SqlExpression,
        aliases: &HashMap<String, SqlExpression>,
        dependencies: &mut HashSet<String>
    ) {
        match expr {
            SqlExpression::Window { func: _, args, partition_by, order_by, .. } => {
                // Check args
                for arg in args {
                    self.find_deps_in_expression(arg, aliases, dependencies);
                }

                // Check PARTITION BY
                if let Some(partition) = partition_by {
                    for col in partition {
                        if aliases.contains_key(col) {
                            dependencies.insert(col.clone());
                        }
                    }
                }

                // Check ORDER BY
                if let Some(order) = order_by {
                    for order_col in order {
                        if let SqlExpression::Column(col) = &order_col.column {
                            if aliases.contains_key(col) {
                                dependencies.insert(col.clone());
                            }
                        }
                    }
                }
            }
            SqlExpression::Column(col) => {
                // Check if this column is actually an alias
                if aliases.contains_key(col) {
                    dependencies.insert(col.clone());
                }
            }
            SqlExpression::FunctionCall { args, .. } => {
                for arg in args {
                    self.find_deps_in_expression(arg, aliases, dependencies);
                }
            }
            SqlExpression::BinaryOp { left, right, .. } => {
                self.find_deps_in_expression(left, aliases, dependencies);
                self.find_deps_in_expression(right, aliases, dependencies);
            }
            SqlExpression::Case { when_clauses, else_clause } => {
                for (cond, result) in when_clauses {
                    self.find_deps_in_expression(cond, aliases, dependencies);
                    self.find_deps_in_expression(result, aliases, dependencies);
                }
                if let Some(else_expr) = else_clause {
                    self.find_deps_in_expression(else_expr, aliases, dependencies);
                }
            }
            _ => {}
        }
    }

    /// Generate a CTE that computes the dependent columns
    fn generate_lifted_cte(
        &mut self,
        statement: &SelectStatement,
        dependencies: &HashSet<String>
    ) -> CTE {
        self.cte_counter += 1;
        let cte_name = format!("__lifted_{}", self.cte_counter);

        // Build SELECT items for the CTE
        let mut cte_select_items = vec![SelectItem::Star]; // Include all original columns

        // Add computed columns that are dependencies
        for item in &statement.select_items {
            if let SelectItem::Expression { expr, alias } = item {
                if dependencies.contains(alias) {
                    cte_select_items.push(SelectItem::Expression {
                        expr: expr.clone(),
                        alias: alias.clone(),
                    });
                }
            }
        }

        // Build the CTE query
        let cte_query = SelectStatement {
            distinct: false,
            columns: vec!["*".to_string()], // Legacy field
            select_items: cte_select_items,
            from_table: statement.from_table.clone(),
            from_subquery: statement.from_subquery.clone(),
            from_function: statement.from_function.clone(),
            from_alias: statement.from_alias.clone(),
            joins: statement.joins.clone(),
            where_clause: statement.where_clause.clone(),
            order_by: None, // Don't need ordering in the CTE
            group_by: None, // GROUP BY goes in outer query
            having: None,   // HAVING goes in outer query
            limit: None,
            offset: None,
            ctes: vec![], // No nested CTEs in the generated one
        };

        CTE {
            name: cte_name,
            cte_type: CTEType::Standard(cte_query),
        }
    }

    /// Rewrite the main query to use the generated CTE
    fn rewrite_query_with_cte(
        &self,
        statement: &mut SelectStatement,
        cte: CTE,
        dependencies: &HashSet<String>
    ) {
        let cte_name = cte.name.clone();

        // Remove the computed expressions from SELECT items if they're dependencies
        let mut new_select_items = Vec::new();
        for item in &statement.select_items {
            match item {
                SelectItem::Expression { expr: _, alias } if dependencies.contains(alias) => {
                    // Replace with simple column reference
                    new_select_items.push(SelectItem::Column(alias.clone()));
                }
                _ => {
                    new_select_items.push(item.clone());
                }
            }
        }

        // Update the statement
        statement.select_items = new_select_items;
        statement.from_table = Some(cte_name);
        statement.from_subquery = None;
        statement.from_function = None;
        // Keep the original alias if there was one

        // Add the CTE to the statement
        statement.ctes.insert(0, cte);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_dependency_lifting() {
        let mut lifter = ColumnDependencyLifter::new();

        // Create a query that uses an alias in PARTITION BY
        let mut statement = SelectStatement {
            select_items: vec![
                SelectItem::Expression {
                    expr: SqlExpression::Case {
                        when_clauses: vec![(
                            SqlExpression::FunctionCall {
                                name: "CONTAINS".to_string(),
                                args: vec![
                                    SqlExpression::Column("id".to_string()),
                                    SqlExpression::String("|".to_string()),
                                ],
                            },
                            SqlExpression::FunctionCall {
                                name: "SUBSTRING_AFTER".to_string(),
                                args: vec![
                                    SqlExpression::Column("id".to_string()),
                                    SqlExpression::String("|".to_string()),
                                    SqlExpression::Integer(1),
                                ],
                            },
                        )],
                        else_clause: Some(Box::new(SqlExpression::Column("id".to_string()))),
                    },
                    alias: "root".to_string(),
                },
                SelectItem::Expression {
                    expr: SqlExpression::Window {
                        func: "ROW_NUMBER".to_string(),
                        args: vec![],
                        partition_by: Some(vec!["root".to_string()]),
                        order_by: None,
                        frame: None,
                    },
                    alias: "rn".to_string(),
                },
            ],
            from_table: Some("test_table".to_string()),
            ..Default::default()
        };

        // Apply lifting
        let lifted = lifter.lift_dependencies(&mut statement);

        assert!(lifted);
        assert_eq!(statement.ctes.len(), 1);
        assert!(statement.ctes[0].name.starts_with("__lifted_"));
        assert_eq!(statement.from_table, Some(statement.ctes[0].name.clone()));
    }
}