use crate::sql::parser::ast::SelectStatement;

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
        if let Some(subquery) = statement.from_subquery.take() {
            statement.from_subquery = Some(Box::new(Self::remove_from_statement(*subquery)));
        }

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
            crate::sql::parser::ast::SelectItem::Expression { expr, alias } => {
                crate::sql::parser::ast::SelectItem::Expression {
                    expr: Self::remove_from_expression(expr),
                    alias,
                }
            }
            other => other,
        }
    }

    /// Remove INTO from expressions (handles subqueries)
    fn remove_from_expression(
        expr: crate::sql::parser::ast::SqlExpression,
    ) -> crate::sql::parser::ast::SqlExpression {
        use crate::sql::parser::ast::SqlExpression;

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
            SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
                left: Box::new(Self::remove_from_expression(*left)),
                op,
                right: Box::new(Self::remove_from_expression(*right)),
            },
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => SqlExpression::FunctionCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| Self::remove_from_expression(arg))
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
                        condition: Box::new(Self::remove_from_expression(*branch.condition)),
                        result: Box::new(Self::remove_from_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(Self::remove_from_expression(*e))),
            },
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => SqlExpression::SimpleCaseExpression {
                expr: Box::new(Self::remove_from_expression(*expr)),
                when_branches: when_branches
                    .into_iter()
                    .map(|branch| crate::sql::parser::ast::SimpleWhenBranch {
                        value: Box::new(Self::remove_from_expression(*branch.value)),
                        result: Box::new(Self::remove_from_expression(*branch.result)),
                    })
                    .collect(),
                else_branch: else_branch.map(|e| Box::new(Self::remove_from_expression(*e))),
            },
            SqlExpression::InList { expr, values } => SqlExpression::InList {
                expr: Box::new(Self::remove_from_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|e| Self::remove_from_expression(e))
                    .collect(),
            },
            SqlExpression::NotInList { expr, values } => SqlExpression::NotInList {
                expr: Box::new(Self::remove_from_expression(*expr)),
                values: values
                    .into_iter()
                    .map(|e| Self::remove_from_expression(e))
                    .collect(),
            },
            SqlExpression::Between { expr, lower, upper } => SqlExpression::Between {
                expr: Box::new(Self::remove_from_expression(*expr)),
                lower: Box::new(Self::remove_from_expression(*lower)),
                upper: Box::new(Self::remove_from_expression(*upper)),
            },
            SqlExpression::Not { expr } => SqlExpression::Not {
                expr: Box::new(Self::remove_from_expression(*expr)),
            },
            // Terminal expressions don't contain subqueries
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::IntoTable;

    #[test]
    fn test_remove_simple_into() {
        let stmt = SelectStatement {
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
            into_table: Some(IntoTable {
                name: "#temp".to_string(),
            }),
            set_operations: vec![],
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
            from_table: Some("inner_table".to_string()),
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
            into_table: Some(IntoTable {
                name: "#inner_temp".to_string(),
            }),
            set_operations: vec![],
        };

        let stmt = SelectStatement {
            distinct: false,
            columns: vec![],
            select_items: vec![],
            from_table: None,
            from_subquery: Some(Box::new(subquery)),
            from_function: None,
            from_alias: Some("subq".to_string()),
            joins: vec![],
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            limit: None,
            offset: None,
            ctes: vec![],
            into_table: Some(IntoTable {
                name: "#outer_temp".to_string(),
            }),
            set_operations: vec![],
        };

        let result = IntoClauseRemover::remove_into_clause(stmt);

        // Both outer and inner INTO should be removed
        assert!(result.into_table.is_none());
        assert!(result.from_subquery.as_ref().unwrap().into_table.is_none());
    }
}
