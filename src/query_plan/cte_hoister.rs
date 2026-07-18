use crate::sql::parser::ast::{
    CTEType, SelectItem, SelectStatement, SqlExpression, WhereClause, CTE,
};
use crate::sql::parser::walk;
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
        statement.map_from_subquery(|subquery| {
            let rewritten_sub = self.hoist_from_statement(subquery);

            // If the subquery has CTEs, hoist them
            for cte in rewritten_sub.ctes.clone() {
                self.add_cte(cte);
            }

            // Return the subquery without its CTEs (they're hoisted)
            SelectStatement {
                ctes: Vec::new(),
                ..rewritten_sub
            }
        });

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
    ///
    /// The only real rule is the subquery arms: recurse into the nested
    /// statement so its CTEs get pulled up to the top level. They stay
    /// explicit because [`walk::map_children`] treats a subquery statement as
    /// a scope boundary -- crossing it is precisely this transformer's job.
    fn hoist_from_expression(&mut self, expr: SqlExpression) -> SqlExpression {
        match expr {
            SqlExpression::ScalarSubquery { query } => SqlExpression::ScalarSubquery {
                query: Box::new(self.hoist_from_statement(*query)),
            },
            SqlExpression::InSubquery { expr, subquery } => SqlExpression::InSubquery {
                expr: Box::new(self.hoist_from_expression(*expr)),
                subquery: Box::new(self.hoist_from_statement(*subquery)),
            },
            SqlExpression::NotInSubquery { expr, subquery } => SqlExpression::NotInSubquery {
                expr: Box::new(self.hoist_from_expression(*expr)),
                subquery: Box::new(self.hoist_from_statement(*subquery)),
            },
            // Defensive, not a demonstrable fix: the tuple forms fell into the
            // old catch-all, but the parser currently rejects WITH anywhere in
            // expression position ("Tuple IN requires a subquery on the right"),
            // so no input reaches these arms today. Kept for symmetry with the
            // other subquery arms, which are equally unreachable for the same
            // reason.
            SqlExpression::InSubqueryTuple { exprs, subquery } => SqlExpression::InSubqueryTuple {
                exprs: exprs
                    .into_iter()
                    .map(|e| self.hoist_from_expression(e))
                    .collect(),
                subquery: Box::new(self.hoist_from_statement(*subquery)),
            },
            SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
                SqlExpression::NotInSubqueryTuple {
                    exprs: exprs
                        .into_iter()
                        .map(|e| self.hoist_from_expression(e))
                        .collect(),
                    subquery: Box::new(self.hoist_from_statement(*subquery)),
                }
            }
            other => walk::map_children(other, |e| self.hoist_from_expression(e)),
        }
    }

    /// Recursively hoist from a WHERE clause
    fn hoist_from_where_clause(&mut self, where_clause: &mut WhereClause) {
        for condition in &mut where_clause.conditions {
            condition.expr = self.hoist_from_expression(condition.expr.clone());
        }
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
    ///
    /// The only real rule is the subquery arms: descend into the nested
    /// statement and look for CTE references there. Those must stay explicit
    /// because [`walk::visit_children`] treats a subquery statement as a scope
    /// boundary and will not enter it. Everything else is plain traversal.
    fn find_cte_refs_in_expression(&self, expr: &SqlExpression, deps: &mut HashSet<String>) {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                self.find_cte_references(query, deps);
            }
            SqlExpression::InSubquery { expr, subquery }
            | SqlExpression::NotInSubquery { expr, subquery } => {
                self.find_cte_refs_in_expression(expr, deps);
                self.find_cte_references(subquery, deps);
            }
            // Tuple forms were missing from the old catch-all. Unlike the
            // hoisting path these ARE reachable: the subquery need not contain
            // a WITH, only a reference to an already-hoisted CTE.
            SqlExpression::InSubqueryTuple { exprs, subquery }
            | SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
                for e in exprs {
                    self.find_cte_refs_in_expression(e, deps);
                }
                self.find_cte_references(subquery, deps);
            }
            other => {
                walk::visit_children(other, |child| self.find_cte_refs_in_expression(child, deps))
            }
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

    /// Regression: hoisting a derived table must rewrite the `from_source`
    /// copy, not only the legacy `from_subquery`.
    ///
    /// The parser fills both with clones of the same subquery, and the executor
    /// reads `from_source`. Rewriting one alone left a stale pre-hoist copy in
    /// the field that actually gets executed.
    ///
    /// This currently produces the right answer either way — the stale copy is
    /// a self-contained statement that still carries its own CTEs — so the
    /// desync is latent, not a live wrong-results bug. The assertion pins the
    /// two representations together before the correlated-subquery work starts
    /// depending on `from_source` being authoritative.
    #[test]
    fn test_derived_table_hoisting_updates_from_source() {
        use crate::sql::parser::ast::TableSource;
        use crate::sql::recursive_parser::Parser;

        let mut parser = Parser::new(
            "SELECT symbol FROM (WITH x AS (SELECT symbol FROM trades) SELECT symbol FROM x) sub",
        );
        let stmt = parser.parse().expect("query should parse");

        let hoisted = CTEHoister::hoist_ctes(stmt);

        // The inner CTE was lifted to the top level.
        assert_eq!(hoisted.ctes.len(), 1, "inner CTE should be hoisted");
        assert_eq!(hoisted.ctes[0].name, "x");

        // Both representations must show the CTE-stripped subquery. Before the
        // fix, from_source still held the original with `ctes.len() == 1`.
        #[allow(deprecated)]
        let legacy = hoisted
            .from_subquery
            .as_ref()
            .expect("from_subquery should be present");
        assert!(legacy.ctes.is_empty(), "legacy copy should be stripped");

        match hoisted.from_source {
            Some(TableSource::DerivedTable {
                ref query,
                ref alias,
            }) => {
                assert!(
                    query.ctes.is_empty(),
                    "from_source holds a stale pre-hoist subquery with {} CTE(s)",
                    query.ctes.len()
                );
                assert_eq!(alias, "sub", "derived-table alias must survive the rewrite");
            }
            ref other => panic!("expected a DerivedTable from_source, got {other:?}"),
        }
    }

    #[test]
    fn test_simple_cte_hoisting() {
        // Test that a simple nested CTE gets hoisted
        let inner_query = SelectStatement {
            distinct: false,
            columns: vec!["col1".to_string()],
            select_items: vec![],
            from_source: None,
            #[allow(deprecated)]
            from_table: Some("table1".to_string()),
            #[allow(deprecated)]
            from_subquery: None,
            #[allow(deprecated)]
            from_function: None,
            #[allow(deprecated)]
            from_alias: None,
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            qualify: None,
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
            from_source: None,
            #[allow(deprecated)]
            from_subquery: Some(Box::new(SelectStatement {
                distinct: false,
                columns: vec![],
                select_items: vec![],
                ctes: vec![CTE {
                    name: "inner".to_string(),
                    column_list: None,
                    cte_type: CTEType::Standard(inner_query),
                }],
                from_source: None,
                #[allow(deprecated)]
                from_table: Some("inner".to_string()),
                #[allow(deprecated)]
                from_subquery: None,
                #[allow(deprecated)]
                from_function: None,
                #[allow(deprecated)]
                from_alias: None,
                joins: vec![],
                where_clause: None,
                order_by: None,
                group_by: None,
                having: None,
                qualify: None,
                limit: None,
                offset: None,
                into_table: None,
                set_operations: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            })),
            #[allow(deprecated)]
            from_table: None,
            #[allow(deprecated)]
            from_function: None,
            #[allow(deprecated)]
            from_alias: None,
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            qualify: None,
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
