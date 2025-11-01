//! QUALIFY to WHERE clause transformer
//!
//! This transformer converts QUALIFY clauses (Snowflake-style window function filtering)
//! into WHERE clauses that work with lifted window functions.
//!
//! # Problem
//!
//! Users want to filter on window function results without writing subqueries:
//! ```sql
//! SELECT region, sales_amount,
//!        ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
//! FROM sales
//! QUALIFY rn <= 3  -- Cleaner than WHERE in a subquery!
//! ```
//!
//! # Solution
//!
//! The transformer works in conjunction with ExpressionLifter:
//! 1. ExpressionLifter runs first, lifting window functions to CTE
//! 2. QualifyToWhereTransformer runs second, converting QUALIFY → WHERE
//!
//! # Algorithm
//!
//! 1. Check if statement has a QUALIFY clause
//! 2. If yes, move it to WHERE clause
//! 3. If WHERE already exists, combine with AND
//! 4. Set qualify field to None
//!
//! # Example Transformation
//!
//! After ExpressionLifter:
//! ```sql
//! WITH _lifted AS (
//!     SELECT region, sales_amount,
//!            ROW_NUMBER() OVER (...) AS rn
//!     FROM sales
//! )
//! SELECT * FROM _lifted
//! QUALIFY rn <= 3
//! ```
//!
//! After QualifyToWhereTransformer:
//! ```sql
//! WITH _lifted AS (
//!     SELECT region, sales_amount,
//!            ROW_NUMBER() OVER (...) AS rn
//!     FROM sales
//! )
//! SELECT * FROM _lifted
//! WHERE rn <= 3
//! ```

use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{Condition, LogicalOp, SelectStatement, WhereClause};
use anyhow::Result;
use tracing::debug;

/// Transformer that converts QUALIFY clauses to WHERE clauses
pub struct QualifyToWhereTransformer;

impl QualifyToWhereTransformer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualifyToWhereTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for QualifyToWhereTransformer {
    fn name(&self) -> &str {
        "QualifyToWhereTransformer"
    }

    fn description(&self) -> &str {
        "Converts QUALIFY clauses to WHERE clauses after window function lifting"
    }

    fn transform(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        // Only process if there's a QUALIFY clause
        if stmt.qualify.is_none() {
            return Ok(stmt);
        }

        let qualify_expr = stmt.qualify.take().unwrap();

        debug!("Converting QUALIFY clause to WHERE");

        // If there's already a WHERE clause, combine them with AND
        if let Some(mut where_clause) = stmt.where_clause.take() {
            debug!("Combining QUALIFY with existing WHERE using AND");

            // Add AND connector to the last condition in existing WHERE
            if let Some(last_condition) = where_clause.conditions.last_mut() {
                last_condition.connector = Some(LogicalOp::And);
            }

            // Add the QUALIFY expression as a new condition
            where_clause.conditions.push(Condition {
                expr: qualify_expr,
                connector: None,
            });

            stmt.where_clause = Some(where_clause);
        } else {
            // No existing WHERE clause, create new one
            debug!("Creating new WHERE clause from QUALIFY");

            stmt.where_clause = Some(WhereClause {
                conditions: vec![Condition {
                    expr: qualify_expr,
                    connector: None,
                }],
            });
        }

        Ok(stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::{ColumnRef, QuoteStyle, SqlExpression};

    #[test]
    fn test_qualify_to_where_simple() {
        let mut stmt = SelectStatement::default();

        // Add QUALIFY: rn <= 3
        stmt.qualify = Some(SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("rn".to_string()))),
            op: "<=".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
        });

        let mut transformer = QualifyToWhereTransformer::new();
        let result = transformer.transform(stmt).unwrap();

        // QUALIFY should be None
        assert!(result.qualify.is_none());

        // WHERE should exist
        assert!(result.where_clause.is_some());

        let where_clause = result.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 1);
    }

    #[test]
    fn test_qualify_combined_with_existing_where() {
        let mut stmt = SelectStatement::default();

        // Add existing WHERE: region = 'North'
        stmt.where_clause = Some(WhereClause {
            conditions: vec![Condition {
                expr: SqlExpression::BinaryOp {
                    left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                        "region".to_string(),
                    ))),
                    op: "=".to_string(),
                    right: Box::new(SqlExpression::StringLiteral("North".to_string())),
                },
                connector: None,
            }],
        });

        // Add QUALIFY: rn <= 3
        stmt.qualify = Some(SqlExpression::BinaryOp {
            left: Box::new(SqlExpression::Column(ColumnRef::unquoted("rn".to_string()))),
            op: "<=".to_string(),
            right: Box::new(SqlExpression::NumberLiteral("3".to_string())),
        });

        let mut transformer = QualifyToWhereTransformer::new();
        let result = transformer.transform(stmt).unwrap();

        // QUALIFY should be None
        assert!(result.qualify.is_none());

        // WHERE should have 2 conditions connected by AND
        let where_clause = result.where_clause.unwrap();
        assert_eq!(where_clause.conditions.len(), 2);
        assert!(matches!(
            where_clause.conditions[0].connector,
            Some(LogicalOp::And)
        ));
    }
}
