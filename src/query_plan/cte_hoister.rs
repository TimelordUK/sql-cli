use crate::sql::parser::ast::{
    CTEType, Condition, SelectItem, SelectStatement, SqlExpression, WhereClause, CTE,
};
use std::collections::{HashMap, HashSet};

/// CTE Hoister - Analyzes and rewrites nested CTEs
///
/// This module implements automatic CTE hoisting to transform nested WITH clauses
/// into a flat list of CTEs at the query's top level. This enables natural nested
/// query writing while maintaining compatibility with SQL execution.
///
/// Example transformation:
/// ```sql
/// -- Input (nested):
/// SELECT * FROM (
///   WITH inner_cte AS (SELECT ...)
///   SELECT * FROM inner_cte
/// )
///
/// -- Output (hoisted):
/// WITH inner_cte AS (SELECT ...)
/// SELECT * FROM inner_cte
/// ```
pub struct CTEHoister {
    hoisted_ctes: Vec<CTE>,
    _cte_counter: usize,
    dependency_graph: HashMap<String, HashSet<String>>,
}

impl CTEHoister {
    pub fn new() -> Self {
        Self {
            hoisted_ctes: Vec::new(),
            _cte_counter: 0,
            dependency_graph: HashMap::new(),
        }
    }

    /// Hoist all nested CTEs to the top level
    pub fn hoist_ctes(mut statement: SelectStatement) -> SelectStatement {
        let mut hoister = CTEHoister::new();

        // First collect any existing top-level CTEs
        for cte in statement.ctes.drain(..) {
            hoister.add_cte(cte);
        }

        // Then recursively hoist from the main statement
        let rewritten = hoister.hoist_from_statement(statement);

        // Build final statement with all hoisted CTEs
        SelectStatement {
            ctes: hoister.get_ordered_ctes(),
            ..rewritten
        }
    }

    /// Recursively hoist CTEs from a SELECT statement
    fn hoist_from_statement(&mut self, mut statement: SelectStatement) -> SelectStatement {
        // Hoist from subquery in FROM clause
        if let Some(subquery) = statement.from_subquery.take() {
            let rewritten_sub = self.hoist_from_statement(*subquery);

            // If the subquery has CTEs, hoist them
            for cte in rewritten_sub.ctes.clone() {
                self.add_cte(cte);
            }

            // Return the subquery without its CTEs (they're hoisted)
            statement.from_subquery = Some(Box::new(SelectStatement {
                ctes: Vec::new(),
                ..rewritten_sub
            }));
        }

        // Hoist from CTEs in this statement
        let local_ctes = statement.ctes.drain(..).collect::<Vec<_>>();
        for mut cte in local_ctes {
            // First hoist from within this CTE's query if it's a standard CTE
            if let CTEType::Standard(query) = cte.cte_type {
                let hoisted_query = self.hoist_from_statement(query);
                cte.cte_type = CTEType::Standard(hoisted_query);
            }
            // Then add the CTE itself
            self.add_cte(cte);
        }

        // Hoist from expressions in SELECT items
        statement.select_items = statement
            .select_items
            .into_iter()
            .map(|item| self.hoist_from_select_item(item))
            .collect();

        // Hoist from WHERE clause subqueries
        if let Some(where_clause) = &mut statement.where_clause {
            self.hoist_from_where_clause(where_clause);
        }

        // Return the statement without CTEs (they're all hoisted)
        SelectStatement {
            ctes: Vec::new(),
            ..statement
        }
    }

    /// Hoist CTEs from a SELECT item (for subqueries in expressions)
    fn hoist_from_select_item(&mut self, item: SelectItem) -> SelectItem {
        match item {
            SelectItem::Expression {
                expr,
                alias,
                leading_comments,
                trailing_comment,
            } => SelectItem::Expression {
                expr: self.hoist_from_expression(expr),
                alias,
                leading_comments,
                trailing_comment,
            },
            other => other,
        }
    }

    /// Hoist CTEs from an expression
    fn hoist_from_expression(&mut self, expr: SqlExpression) -> SqlExpression {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                let rewritten = self.hoist_from_statement(*query);
                SqlExpression::ScalarSubquery {
                    query: Box::new(rewritten),
                }
            }
            SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
                left: Box::new(self.hoist_from_expression(*left)),
                op,
                right: Box::new(self.hoist_from_expression(*right)),
            },
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => SqlExpression::FunctionCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.hoist_from_expression(arg))
                    .collect(),
                distinct,
            },
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => SqlExpression::CaseExpression {
                when_branches: when_branches
                    .into_iter()
                    .map(|branch| crate::sql::parser::ast::WhenBranch {
                        condition: Box::new(self.hoist_from_expression(*branch.condition)),
                        result: Box::new(self.hoist_from_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(self.hoist_from_expression(*e))),
            },
            SqlExpression::InList { expr, values } => SqlExpression::InList {
                expr: Box::new(self.hoist_from_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|e| self.hoist_from_expression(e))
                    .collect(),
            },
            SqlExpression::NotInList { expr, values } => SqlExpression::NotInList {
                expr: Box::new(self.hoist_from_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|e| self.hoist_from_expression(e))
                    .collect(),
            },
            SqlExpression::InSubquery { expr, subquery } => {
                let rewritten = self.hoist_from_statement(*subquery);
                SqlExpression::InSubquery {
                    expr: Box::new(self.hoist_from_expression(*expr)),
                    subquery: Box::new(rewritten),
                }
            }
            SqlExpression::NotInSubquery { expr, subquery } => {
                let rewritten = self.hoist_from_statement(*subquery);
                SqlExpression::NotInSubquery {
                    expr: Box::new(self.hoist_from_expression(*expr)),
                    subquery: Box::new(rewritten),
                }
            }
            SqlExpression::Between { expr, lower, upper } => SqlExpression::Between {
                expr: Box::new(self.hoist_from_expression(*expr)),
                lower: Box::new(self.hoist_from_expression(*lower)),
                upper: Box::new(self.hoist_from_expression(*upper)),
            },
            SqlExpression::Not { expr } => SqlExpression::Not {
                expr: Box::new(self.hoist_from_expression(*expr)),
            },
            // For other expression types that might contain subqueries
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => SqlExpression::SimpleCaseExpression {
                expr: Box::new(self.hoist_from_expression(*expr)),
                when_branches: when_branches
                    .into_iter()
                    .map(|branch| crate::sql::parser::ast::SimpleWhenBranch {
                        value: Box::new(self.hoist_from_expression(*branch.value)),
                        result: Box::new(self.hoist_from_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(self.hoist_from_expression(*e))),
            },
            // Terminal expressions don't contain subqueries
            other => other,
        }
    }

    /// Hoist CTEs from WHERE clause
    fn hoist_from_where_clause(&mut self, where_clause: &mut WhereClause) {
        for condition in &mut where_clause.conditions {
            condition.expr = self.hoist_from_expression(condition.expr.clone());
        }
    }

    /// Recursively hoist from a condition
    fn hoist_from_condition(&mut self, condition: &mut Condition) {
        condition.expr = self.hoist_from_expression(condition.expr.clone());
    }

    /// Add a CTE to the hoisted collection
    fn add_cte(&mut self, cte: CTE) {
        // Track dependencies for proper ordering
        self.analyze_cte_dependencies(&cte);
        self.hoisted_ctes.push(cte);
    }

    /// Analyze CTE dependencies for proper ordering
    fn analyze_cte_dependencies(&mut self, cte: &CTE) {
        let mut deps = HashSet::new();
        if let CTEType::Standard(query) = &cte.cte_type {
            self.find_cte_references(query, &mut deps);
        }
        self.dependency_graph.insert(cte.name.clone(), deps);
    }

    /// Find all CTE references in a statement
    fn find_cte_references(&self, statement: &SelectStatement, deps: &mut HashSet<String>) {
        // Check if FROM references a CTE
        if let Some(table) = &statement.from_table {
            // Check if this table name is a CTE
            for cte in &self.hoisted_ctes {
                if cte.name == *table {
                    deps.insert(table.clone());
                }
            }
        }

        // Check subquery references
        if let Some(subquery) = &statement.from_subquery {
            self.find_cte_references(subquery, deps);
        }

        // Check JOIN references
        for join in &statement.joins {
            // Check if join table is a CTE
            if let crate::sql::parser::ast::TableSource::Table(table_name) = &join.table {
                for cte in &self.hoisted_ctes {
                    if cte.name == *table_name {
                        deps.insert(table_name.clone());
                    }
                }
            }
        }

        // Check expressions for CTE references
        for item in &statement.select_items {
            if let SelectItem::Expression { expr, .. } = item {
                self.find_cte_refs_in_expression(expr, deps);
            }
        }

        // Check WHERE clause
        if let Some(where_clause) = &statement.where_clause {
            for condition in &where_clause.conditions {
                self.find_cte_refs_in_expression(&condition.expr, deps);
            }
        }
    }

    /// Find CTE references in an expression
    fn find_cte_refs_in_expression(&self, expr: &SqlExpression, deps: &mut HashSet<String>) {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                self.find_cte_references(query, deps);
            }
            SqlExpression::InSubquery { subquery, .. } => {
                self.find_cte_references(subquery, deps);
            }
            SqlExpression::NotInSubquery { subquery, .. } => {
                self.find_cte_references(subquery, deps);
            }
            SqlExpression::FunctionCall { args, .. } => {
                for arg in args {
                    self.find_cte_refs_in_expression(arg, deps);
                }
            }
            SqlExpression::BinaryOp { left, right, .. } => {
                self.find_cte_refs_in_expression(left, deps);
                self.find_cte_refs_in_expression(right, deps);
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                for branch in when_branches {
                    self.find_cte_refs_in_expression(&branch.condition, deps);
                    self.find_cte_refs_in_expression(&branch.result, deps);
                }
                if let Some(else_expr) = else_branch {
                    self.find_cte_refs_in_expression(else_expr, deps);
                }
            }
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => {
                self.find_cte_refs_in_expression(expr, deps);
                for branch in when_branches {
                    self.find_cte_refs_in_expression(&branch.value, deps);
                    self.find_cte_refs_in_expression(&branch.result, deps);
                }
                if let Some(else_expr) = else_branch {
                    self.find_cte_refs_in_expression(else_expr, deps);
                }
            }
            SqlExpression::InList { expr, values } | SqlExpression::NotInList { expr, values } => {
                self.find_cte_refs_in_expression(expr, deps);
                for value in values {
                    self.find_cte_refs_in_expression(value, deps);
                }
            }
            SqlExpression::Between { expr, lower, upper } => {
                self.find_cte_refs_in_expression(expr, deps);
                self.find_cte_refs_in_expression(lower, deps);
                self.find_cte_refs_in_expression(upper, deps);
            }
            SqlExpression::Not { expr } => {
                self.find_cte_refs_in_expression(expr, deps);
            }
            _ => {}
        }
    }

    /// Get CTEs in dependency order
    fn get_ordered_ctes(self) -> Vec<CTE> {
        // Simple topological sort
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        fn visit(
            name: &str,
            graph: &HashMap<String, HashSet<String>>,
            ctes: &[CTE],
            visited: &mut HashSet<String>,
            temp_mark: &mut HashSet<String>,
            result: &mut Vec<CTE>,
        ) {
            if visited.contains(name) {
                return;
            }
            if temp_mark.contains(name) {
                // Circular dependency - for now just continue
                return;
            }

            temp_mark.insert(name.to_string());

            if let Some(deps) = graph.get(name) {
                for dep in deps {
                    visit(dep, graph, ctes, visited, temp_mark, result);
                }
            }

            temp_mark.remove(name);
            visited.insert(name.to_string());

            // Find and add the CTE
            if let Some(cte) = ctes.iter().find(|c| c.name == name) {
                result.push(cte.clone());
            }
        }

        // Visit all CTEs
        for cte in &self.hoisted_ctes {
            visit(
                &cte.name,
                &self.dependency_graph,
                &self.hoisted_ctes,
                &mut visited,
                &mut temp_mark,
                &mut result,
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cte_hoisting() {
        // Test that a simple nested CTE gets hoisted
        let inner_query = SelectStatement {
            distinct: false,
            columns: vec!["col1".to_string()],
            select_items: vec![],
            from_table: Some("table1".to_string()),
            from_subquery: None,
            from_function: None,
            from_alias: None,
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            limit: None,
            offset: None,
            ctes: vec![],
            into_table: None,
            set_operations: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let nested_query = SelectStatement {
            distinct: false,
            columns: vec![],
            select_items: vec![],
            from_subquery: Some(Box::new(SelectStatement {
                distinct: false,
                columns: vec![],
                select_items: vec![],
                ctes: vec![CTE {
                    name: "inner".to_string(),
                    column_list: None,
                    cte_type: CTEType::Standard(inner_query),
                }],
                from_table: Some("inner".to_string()),
                from_subquery: None,
                from_function: None,
                from_alias: None,
                joins: vec![],
                where_clause: None,
                order_by: None,
                group_by: None,
                having: None,
                limit: None,
                offset: None,
                into_table: None,
                set_operations: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            })),
            from_table: None,
            from_function: None,
            from_alias: None,
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            limit: None,
            offset: None,
            ctes: vec![],
            into_table: None,
            set_operations: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = CTEHoister::hoist_ctes(nested_query);

        assert_eq!(result.ctes.len(), 1);
        assert_eq!(result.ctes[0].name, "inner");
        assert!(result.from_subquery.as_ref().unwrap().ctes.is_empty());
    }
}
