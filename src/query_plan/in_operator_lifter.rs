use crate::sql::parser::ast::{
    CTEType, Condition, LogicalOp, SelectItem, SelectStatement, SqlExpression, WhereClause, CTE,
};
use std::collections::HashMap;

/// Specialized lifter for IN operator expressions with function calls
pub struct InOperatorLifter {
    /// Counter for generating unique column names
    column_counter: usize,
}

impl InOperatorLifter {
    pub fn new() -> Self {
        InOperatorLifter { column_counter: 0 }
    }

    /// Generate a unique column name for lifted expressions
    fn next_column_name(&mut self) -> String {
        self.column_counter += 1;
        format!("__expr_{}", self.column_counter)
    }

    /// Check if an IN expression needs lifting (has function call on left side)
    pub fn needs_in_lifting(expr: &SqlExpression) -> bool {
        match expr {
            SqlExpression::InList { expr, .. } | SqlExpression::NotInList { expr, .. } => {
                Self::is_complex_expression(expr)
            }
            _ => false,
        }
    }

    /// Check if an expression is complex (not just a column reference)
    fn is_complex_expression(expr: &SqlExpression) -> bool {
        !matches!(expr, SqlExpression::Column(_))
    }

    /// Lift IN expressions with function calls
    pub fn lift_in_expressions(&mut self, stmt: &mut SelectStatement) -> Vec<LiftedInExpression> {
        let mut lifted = Vec::new();

        if let Some(ref mut where_clause) = stmt.where_clause {
            let mut new_conditions = Vec::new();

            for condition in &where_clause.conditions {
                match &condition.expr {
                    SqlExpression::InList { expr, values } if Self::is_complex_expression(expr) => {
                        let column_alias = self.next_column_name();

                        // Record what we're lifting
                        lifted.push(LiftedInExpression {
                            original_expr: expr.as_ref().clone(),
                            alias: column_alias.clone(),
                            values: values.clone(),
                            is_not_in: false,
                        });

                        // Create new simple condition
                        new_conditions.push(Condition {
                            expr: SqlExpression::InList {
                                expr: Box::new(SqlExpression::Column(column_alias)),
                                values: values.clone(),
                            },
                            connector: condition.connector.clone(),
                        });
                    }
                    SqlExpression::NotInList { expr, values }
                        if Self::is_complex_expression(expr) =>
                    {
                        let column_alias = self.next_column_name();

                        // Record what we're lifting
                        lifted.push(LiftedInExpression {
                            original_expr: expr.as_ref().clone(),
                            alias: column_alias.clone(),
                            values: values.clone(),
                            is_not_in: true,
                        });

                        // Create new simple condition
                        new_conditions.push(Condition {
                            expr: SqlExpression::NotInList {
                                expr: Box::new(SqlExpression::Column(column_alias)),
                                values: values.clone(),
                            },
                            connector: condition.connector.clone(),
                        });
                    }
                    _ => {
                        // Keep condition as-is
                        new_conditions.push(condition.clone());
                    }
                }
            }

            // Update WHERE clause with simplified conditions
            where_clause.conditions = new_conditions;
        }

        lifted
    }

    /// Apply lifted expressions to SELECT items
    pub fn apply_lifted_to_select(
        &self,
        stmt: &mut SelectStatement,
        lifted: &[LiftedInExpression],
    ) {
        // Add the computed expressions to the SELECT list
        for lift in lifted {
            stmt.select_items.push(SelectItem::Expression {
                expr: lift.original_expr.clone(),
                alias: lift.alias.clone(),
            });
        }
    }

    /// Create a CTE that includes the lifted expressions
    pub fn create_lifting_cte(
        &self,
        base_table: &str,
        lifted: &[LiftedInExpression],
        cte_name: String,
    ) -> CTE {
        let mut select_items = vec![SelectItem::Star];

        // Add each lifted expression as a computed column
        for lift in lifted {
            select_items.push(SelectItem::Expression {
                expr: lift.original_expr.clone(),
                alias: lift.alias.clone(),
            });
        }

        let cte_select = SelectStatement {
            distinct: false,
            columns: vec!["*".to_string()],
            select_items,
            from_table: Some(base_table.to_string()),
            from_subquery: None,
            from_function: None,
            from_alias: None,
            joins: Vec::new(),
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            limit: None,
            offset: None,
            ctes: Vec::new(),
        };

        CTE {
            name: cte_name,
            column_list: None,
            cte_type: CTEType::Standard(cte_select),
        }
    }

    /// Rewrite a query to lift IN expressions with function calls
    pub fn rewrite_query(&mut self, stmt: &mut SelectStatement) -> bool {
        // Check if we have any IN expressions to lift
        let has_in_to_lift = if let Some(ref where_clause) = stmt.where_clause {
            where_clause
                .conditions
                .iter()
                .any(|c| Self::needs_in_lifting(&c.expr))
        } else {
            false
        };

        if !has_in_to_lift {
            return false;
        }

        // Extract the base table name
        let base_table = match &stmt.from_table {
            Some(table) => table.clone(),
            None => return false, // Can't lift without a FROM clause
        };

        // Lift the IN expressions
        let lifted = self.lift_in_expressions(stmt);

        if lifted.is_empty() {
            return false;
        }

        // Create a CTE with the lifted expressions
        let cte_name = format!("{}_lifted", base_table);
        let cte = self.create_lifting_cte(&base_table, &lifted, cte_name.clone());

        // Add the CTE to the statement
        stmt.ctes.push(cte);

        // Update the FROM clause to use the CTE
        stmt.from_table = Some(cte_name);

        true
    }
}

/// Information about a lifted IN expression
#[derive(Debug, Clone)]
pub struct LiftedInExpression {
    /// The original expression (e.g., LOWER(column))
    pub original_expr: SqlExpression,
    /// The alias for the computed column
    pub alias: String,
    /// The values in the IN list
    pub values: Vec<SqlExpression>,
    /// Whether this was NOT IN (vs IN)
    pub is_not_in: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_in_lifting() {
        // Simple column IN doesn't need lifting
        let simple_in = SqlExpression::InList {
            expr: Box::new(SqlExpression::Column("col".to_string())),
            values: vec![SqlExpression::StringLiteral("a".to_string())],
        };
        assert!(!InOperatorLifter::needs_in_lifting(&simple_in));

        // Function call IN needs lifting
        let func_in = SqlExpression::InList {
            expr: Box::new(SqlExpression::FunctionCall {
                name: "LOWER".to_string(),
                args: vec![SqlExpression::Column("col".to_string())],
                distinct: false,
            }),
            values: vec![SqlExpression::StringLiteral("a".to_string())],
        };
        assert!(InOperatorLifter::needs_in_lifting(&func_in));
    }

    #[test]
    fn test_is_complex_expression() {
        // Column is not complex
        assert!(!InOperatorLifter::is_complex_expression(
            &SqlExpression::Column("col".to_string())
        ));

        // Function call is complex
        assert!(InOperatorLifter::is_complex_expression(
            &SqlExpression::FunctionCall {
                name: "LOWER".to_string(),
                args: vec![SqlExpression::Column("col".to_string())],
                distinct: false,
            }
        ));

        // Binary op is complex
        assert!(InOperatorLifter::is_complex_expression(
            &SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::Column("a".to_string())),
                op: "+".to_string(),
                right: Box::new(SqlExpression::NumberLiteral("1".to_string())),
            }
        ));
    }
}
