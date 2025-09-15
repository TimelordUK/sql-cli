use crate::sql::parser::ast::{
    SelectStatement, SqlExpression, WhereClause, CTE, CTEType, SelectItem
};
use crate::query_plan::{WorkUnit, WorkUnitType, WorkUnitExpression, QueryPlan};
use std::collections::HashSet;

/// Expression lifter that identifies and lifts non-supported WHERE expressions to CTEs
pub struct ExpressionLifter {
    /// Counter for generating unique CTE names
    cte_counter: usize,

    /// Set of function names that need to be lifted
    liftable_functions: HashSet<String>,
}

impl ExpressionLifter {
    /// Create a new expression lifter
    pub fn new() -> Self {
        let mut liftable_functions = HashSet::new();

        // Add window functions that need lifting
        liftable_functions.insert("ROW_NUMBER".to_string());
        liftable_functions.insert("RANK".to_string());
        liftable_functions.insert("DENSE_RANK".to_string());
        liftable_functions.insert("LAG".to_string());
        liftable_functions.insert("LEAD".to_string());
        liftable_functions.insert("FIRST_VALUE".to_string());
        liftable_functions.insert("LAST_VALUE".to_string());
        liftable_functions.insert("NTH_VALUE".to_string());

        // Add aggregate functions that might need lifting in certain contexts
        liftable_functions.insert("PERCENTILE_CONT".to_string());
        liftable_functions.insert("PERCENTILE_DISC".to_string());

        ExpressionLifter {
            cte_counter: 0,
            liftable_functions,
        }
    }

    /// Generate a unique CTE name
    fn next_cte_name(&mut self) -> String {
        self.cte_counter += 1;
        format!("__lifted_{}", self.cte_counter)
    }

    /// Check if an expression needs to be lifted
    pub fn needs_lifting(&self, expr: &SqlExpression) -> bool {
        match expr {
            SqlExpression::WindowFunction { .. } => true,

            SqlExpression::FunctionCall { name, .. } => {
                self.liftable_functions.contains(&name.to_uppercase())
            }

            SqlExpression::BinaryOp { left, right, .. } => {
                self.needs_lifting(left) || self.needs_lifting(right)
            }

            SqlExpression::Not { expr } => self.needs_lifting(expr),

            SqlExpression::InList { expr, values } => {
                self.needs_lifting(expr) || values.iter().any(|v| self.needs_lifting(v))
            }

            SqlExpression::NotInList { expr, values } => {
                self.needs_lifting(expr) || values.iter().any(|v| self.needs_lifting(v))
            }

            SqlExpression::Between { expr, lower, upper } => {
                self.needs_lifting(expr) || self.needs_lifting(lower) || self.needs_lifting(upper)
            }

            SqlExpression::CaseExpression { when_branches, else_branch } => {
                when_branches.iter().any(|branch| {
                    self.needs_lifting(&branch.condition) || self.needs_lifting(&branch.result)
                }) || else_branch.as_ref().map_or(false, |e| self.needs_lifting(e))
            }

            SqlExpression::SimpleCaseExpression { expr, when_branches, else_branch } => {
                self.needs_lifting(expr) ||
                when_branches.iter().any(|branch| {
                    self.needs_lifting(&branch.value) || self.needs_lifting(&branch.result)
                }) || else_branch.as_ref().map_or(false, |e| self.needs_lifting(e))
            }

            _ => false,
        }
    }

    /// Analyze WHERE clause and identify expressions to lift
    pub fn analyze_where_clause(&mut self, where_clause: &WhereClause) -> Vec<LiftableExpression> {
        let mut liftable = Vec::new();

        // Analyze each condition in the WHERE clause
        for condition in &where_clause.conditions {
            if self.needs_lifting(&condition.expr) {
                liftable.push(LiftableExpression {
                    expression: condition.expr.clone(),
                    suggested_name: self.next_cte_name(),
                    dependencies: Vec::new(), // TODO: Analyze dependencies
                });
            }
        }

        liftable
    }

    /// Lift expressions from a SELECT statement
    pub fn lift_expressions(&mut self, stmt: &mut SelectStatement) -> Vec<CTE> {
        let mut lifted_ctes = Vec::new();

        // Check WHERE clause for liftable expressions
        if let Some(ref where_clause) = stmt.where_clause {
            let liftable = self.analyze_where_clause(where_clause);

            for lift_expr in liftable {
                // Create a CTE that includes the lifted expression as a computed column
                let cte_select = SelectStatement {
                    distinct: false,
                    columns: vec!["*".to_string()],
                    select_items: vec![
                        SelectItem::Star,
                        SelectItem::Expression {
                            expr: lift_expr.expression.clone(),
                            alias: "lifted_value".to_string(),
                        }
                    ],
                    from_table: stmt.from_table.clone(),
                    from_subquery: stmt.from_subquery.clone(),
                    from_function: stmt.from_function.clone(),
                    from_alias: stmt.from_alias.clone(),
                    joins: stmt.joins.clone(),
                    where_clause: None, // Move simpler parts of WHERE here if possible
                    order_by: None,
                    group_by: None,
                    having: None,
                    limit: None,
                    offset: None,
                    ctes: Vec::new(),
                };

                let cte = CTE {
                    name: lift_expr.suggested_name.clone(),
                    column_list: None,
                    cte_type: CTEType::Standard(cte_select),
                };

                lifted_ctes.push(cte);

                // Update the main query to reference the CTE
                stmt.from_table = Some(lift_expr.suggested_name);

                // Replace the complex WHERE expression with a simple column reference
                use crate::sql::parser::ast::{Condition, LogicalOp};
                stmt.where_clause = Some(WhereClause {
                    conditions: vec![Condition {
                        expr: SqlExpression::Column("lifted_value".to_string()),
                        connector: None,
                    }],
                });
            }
        }

        // Add lifted CTEs to the statement
        stmt.ctes.extend(lifted_ctes.clone());

        lifted_ctes
    }

    /// Create work units for lifted expressions
    pub fn create_work_units_for_lifted(
        &mut self,
        lifted_ctes: &[CTE],
        plan: &mut QueryPlan,
    ) -> Vec<String> {
        let mut cte_ids = Vec::new();

        for cte in lifted_ctes {
            let unit_id = format!("cte_{}", cte.name);

            let work_unit = WorkUnit {
                id: unit_id.clone(),
                work_type: WorkUnitType::CTE,
                expression: match &cte.cte_type {
                    CTEType::Standard(select) => {
                        WorkUnitExpression::Select(select.clone())
                    }
                    CTEType::Web(_) => {
                        WorkUnitExpression::Custom("WEB CTE".to_string())
                    }
                },
                dependencies: Vec::new(), // CTEs typically don't depend on each other initially
                parallelizable: true,     // CTEs can often be computed in parallel
                cost_estimate: None,
            };

            plan.add_unit(work_unit);
            cte_ids.push(unit_id);
        }

        cte_ids
    }
}

/// Represents an expression that can be lifted to a CTE
#[derive(Debug)]
pub struct LiftableExpression {
    /// The expression to lift
    pub expression: SqlExpression,

    /// Suggested name for the CTE
    pub suggested_name: String,

    /// Dependencies on other CTEs or tables
    pub dependencies: Vec<String>,
}

/// Analyze dependencies between expressions
pub fn analyze_dependencies(expr: &SqlExpression) -> HashSet<String> {
    let mut deps = HashSet::new();

    match expr {
        SqlExpression::Column(col) => {
            deps.insert(col.clone());
        }

        SqlExpression::FunctionCall { args, .. } => {
            for arg in args {
                deps.extend(analyze_dependencies(arg));
            }
        }

        SqlExpression::WindowFunction { args, window_spec, .. } => {
            for arg in args {
                deps.extend(analyze_dependencies(arg));
            }

            // Add partition and order columns as dependencies
            for col in &window_spec.partition_by {
                deps.insert(col.clone());
            }

            for order_col in &window_spec.order_by {
                deps.insert(order_col.column.clone());
            }
        }

        SqlExpression::BinaryOp { left, right, .. } => {
            deps.extend(analyze_dependencies(left));
            deps.extend(analyze_dependencies(right));
        }

        SqlExpression::CaseExpression { when_branches, else_branch } => {
            for branch in when_branches {
                deps.extend(analyze_dependencies(&branch.condition));
                deps.extend(analyze_dependencies(&branch.result));
            }

            if let Some(else_expr) = else_branch {
                deps.extend(analyze_dependencies(else_expr));
            }
        }

        SqlExpression::SimpleCaseExpression { expr, when_branches, else_branch } => {
            deps.extend(analyze_dependencies(expr));

            for branch in when_branches {
                deps.extend(analyze_dependencies(&branch.value));
                deps.extend(analyze_dependencies(&branch.result));
            }

            if let Some(else_expr) = else_branch {
                deps.extend(analyze_dependencies(else_expr));
            }
        }

        _ => {}
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_lifting_window_function() {
        let lifter = ExpressionLifter::new();

        let window_expr = SqlExpression::WindowFunction {
            name: "ROW_NUMBER".to_string(),
            args: vec![],
            window_spec: crate::sql::parser::ast::WindowSpec {
                partition_by: vec![],
                order_by: vec![],
                frame: None,
            },
        };

        assert!(lifter.needs_lifting(&window_expr));
    }

    #[test]
    fn test_needs_lifting_simple_expression() {
        let lifter = ExpressionLifter::new();

        let simple_expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column("col1".to_string())),
            op: "=".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("42".to_string())),
        };

        assert!(!lifter.needs_lifting(&simple_expr));
    }

    #[test]
    fn test_analyze_dependencies() {
        let expr = SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column("col1".to_string())),
            op: "+".to_string(),
            right: Box::new(SqlExpression::Column("col2".to_string())),
        };

        let deps = analyze_dependencies(&expr);
        assert!(deps.contains("col1"));
        assert!(deps.contains("col2"));
        assert_eq!(deps.len(), 2);
    }
}