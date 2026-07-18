use crate::sql::parser::ast::{SelectStatement, SqlExpression};
use crate::sql::parser::walk;

/// INTO Clause Remover - Removes INTO clause from AST for execution
///
/// This module implements AST rewriting to remove the INTO clause from
/// SELECT statements. The INTO clause is used to store query results in
/// temporary tables, but the query executor doesn't understand this syntax.
///
/// The removal is done at the AST level (not via regex) to ensure correctness
/// and maintainability. The caller is responsible for capturing the INTO table
/// information before removal and storing the results after execution.
///
/// Example transformation:
/// ```sql
/// -- Input:
/// SELECT col1, col2 INTO #temp FROM table WHERE x > 5
///
/// -- Output (for execution):
/// SELECT col1, col2 FROM table WHERE x > 5
/// ```
pub struct IntoClauseRemover;

impl IntoClauseRemover {
    /// Remove INTO clause from statement and all nested subqueries
    ///
    /// This creates a new statement with `into_table` set to None.
    /// The original statement is not modified.
    ///
    /// # Arguments
    /// * `statement` - The SELECT statement to process
    ///
    /// # Returns
    /// A new statement with INTO clause removed from all levels
    pub fn remove_into_clause(statement: SelectStatement) -> SelectStatement {
        Self::remove_from_statement(statement)
    }

    /// Recursively remove INTO clause from a statement and its subqueries
    fn remove_from_statement(mut statement: SelectStatement) -> SelectStatement {
        // Remove the INTO clause from this statement
        statement.into_table = None;

        // Remove from subquery in FROM clause
        statement.map_from_subquery(Self::remove_from_statement);

        // Remove from JOIN subqueries
        statement.joins = statement
            .joins
            .into_iter()
            .map(|mut join| {
                if let crate::sql::parser::ast::TableSource::DerivedTable { query, alias } =
                    join.table
                {
                    join.table = crate::sql::parser::ast::TableSource::DerivedTable {
                        query: Box::new(Self::remove_from_statement(*query)),
                        alias,
                    };
                }
                join
            })
            .collect();

        // Remove from scalar subqueries and other expression subqueries
        statement.select_items = statement
            .select_items
            .into_iter()
            .map(|item| Self::remove_from_select_item(item))
            .collect();

        // Remove from WHERE clause subqueries
        if let Some(mut where_clause) = statement.where_clause.take() {
            for condition in &mut where_clause.conditions {
                condition.expr = Self::remove_from_expression(condition.expr.clone());
            }
            statement.where_clause = Some(where_clause);
        }

        // Remove from set operation queries (UNION, INTERSECT, EXCEPT)
        statement.set_operations = statement
            .set_operations
            .into_iter()
            .map(|(op, query)| (op, Box::new(Self::remove_from_statement(*query))))
            .collect();

        statement
    }

    /// Remove INTO from SELECT items (handles subqueries in expressions)
    fn remove_from_select_item(
        item: crate::sql::parser::ast::SelectItem,
    ) -> crate::sql::parser::ast::SelectItem {
        match item {
            crate::sql::parser::ast::SelectItem::Expression {
                expr,
                alias,
                leading_comments,
                trailing_comment,
            } => crate::sql::parser::ast::SelectItem::Expression {
                expr: Self::remove_from_expression(expr),
                alias,
                leading_comments,
                trailing_comment,
            },
            other => other,
        }
    }

    /// Remove INTO from expressions (handles subqueries)
    ///
    /// The only real rule here is about subqueries: every nested
    /// `SelectStatement` needs its `into_table` cleared. Everything else is
    /// plain traversal, delegated to [`walk::map_children`].
    ///
    /// The subquery arms must stay explicit. `map_children` treats a subquery
    /// statement as a **scope boundary** and deliberately does not descend into
    /// it — correct for the alias expanders, but exactly what this transformer
    /// has to do. Delegating them would silently stop INTO being removed from
    /// nested queries.
    fn remove_from_expression(expr: SqlExpression) -> SqlExpression {
        match expr {
            SqlExpression::ScalarSubquery { query } => SqlExpression::ScalarSubquery {
                query: Box::new(Self::remove_from_statement(*query)),
            },
            SqlExpression::InSubquery { expr, subquery } => SqlExpression::InSubquery {
                expr: Box::new(Self::remove_from_expression(*expr)),
                subquery: Box::new(Self::remove_from_statement(*subquery)),
            },
            SqlExpression::NotInSubquery { expr, subquery } => SqlExpression::NotInSubquery {
                expr: Box::new(Self::remove_from_expression(*expr)),
                subquery: Box::new(Self::remove_from_statement(*subquery)),
            },
            // Previously missing: the tuple forms fell into the catch-all, so
            // `WHERE (a, b) IN (SELECT ... INTO #t ...)` kept its INTO clause.
            SqlExpression::InSubqueryTuple { exprs, subquery } => SqlExpression::InSubqueryTuple {
                exprs: exprs
                    .into_iter()
                    .map(Self::remove_from_expression)
                    .collect(),
                subquery: Box::new(Self::remove_from_statement(*subquery)),
            },
            SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
                SqlExpression::NotInSubqueryTuple {
                    exprs: exprs
                        .into_iter()
                        .map(Self::remove_from_expression)
                        .collect(),
                    subquery: Box::new(Self::remove_from_statement(*subquery)),
                }
            }
            other => walk::map_children(other, Self::remove_from_expression),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::IntoTable;

    /// Regression for the walk.rs migration: the tuple subquery forms used to
    /// fall into the hand-rolled catch-all, so an INTO inside
    /// `(a, b) IN (SELECT ...)` was never removed and would reach the executor.
    ///
    /// Parsed rather than hand-built so the AST is one the parser actually
    /// produces (see R4 in docs/ENGINE_REFACTORING.md).
    #[test]
    fn removes_into_inside_tuple_subquery() {
        use crate::sql::recursive_parser::Parser;

        let stmt = Parser::new("SELECT a FROM t WHERE (a, b) IN (SELECT x, y FROM u INTO #inner)")
            .parse()
            .expect("query should parse");

        // Precondition: the parser really did put an INTO on the inner query.
        let inner_into = |s: &SelectStatement| match &s.where_clause {
            Some(w) => match &w.conditions[0].expr {
                SqlExpression::InSubqueryTuple { subquery, .. } => subquery.into_table.clone(),
                other => panic!("expected a tuple IN subquery, got {other:?}"),
            },
            None => panic!("expected a where clause"),
        };
        assert!(
            inner_into(&stmt).is_some(),
            "test is meaningless unless the inner query starts with an INTO"
        );

        let result = IntoClauseRemover::remove_into_clause(stmt);
        assert!(
            inner_into(&result).is_none(),
            "INTO must be removed from inside a tuple subquery"
        );
    }

    #[test]
    fn test_remove_simple_into() {
        let stmt = SelectStatement {
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
            into_table: Some(IntoTable {
                name: "#temp".to_string(),
            }),
            set_operations: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = IntoClauseRemover::remove_into_clause(stmt);
        assert!(result.into_table.is_none());
        assert_eq!(result.from_table, Some("table1".to_string()));
    }

    #[test]
    fn test_remove_into_from_subquery() {
        let subquery = SelectStatement {
            distinct: false,
            columns: vec![],
            select_items: vec![],
            from_source: None,
            #[allow(deprecated)]
            from_table: Some("inner_table".to_string()),
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
            into_table: Some(IntoTable {
                name: "#inner_temp".to_string(),
            }),
            set_operations: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let stmt = SelectStatement {
            distinct: false,
            columns: vec![],
            select_items: vec![],
            from_source: None,
            #[allow(deprecated)]
            from_table: None,
            #[allow(deprecated)]
            from_subquery: Some(Box::new(subquery)),
            #[allow(deprecated)]
            from_function: None,
            #[allow(deprecated)]
            from_alias: Some("subq".to_string()),
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            qualify: None,
            limit: None,
            offset: None,
            ctes: vec![],
            into_table: Some(IntoTable {
                name: "#outer_temp".to_string(),
            }),
            set_operations: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = IntoClauseRemover::remove_into_clause(stmt);

        // Both outer and inner INTO should be removed
        assert!(result.into_table.is_none());
        assert!(result.from_subquery.as_ref().unwrap().into_table.is_none());
    }
}
