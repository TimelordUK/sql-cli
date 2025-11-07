use crate::query_plan::pipeline::ASTTransformer;
use crate::sql::parser::ast::{
    ColumnRef, Condition, LogicalOp, OrderByItem, PivotAggregate, QuoteStyle, SelectItem,
    SelectStatement, SqlExpression, TableSource, WhenBranch,
};
use anyhow::{anyhow, Result};

/// PIVOT Expander - Transforms PIVOT syntax into standard SQL with CASE expressions
///
/// This transformer converts SQL Server-style PIVOT operations into standard SQL
/// using CASE expressions and GROUP BY clauses.
///
/// Transformation Example:
/// ```sql
/// -- Input (PIVOT syntax):
/// SELECT * FROM food_eaten
/// PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle', 'Apple'))
///
/// -- Output (Standard SQL):
/// SELECT Date,
///     MAX(CASE WHEN FoodName = 'Sammich' THEN AmountEaten ELSE NULL END) AS Sammich,
///     MAX(CASE WHEN FoodName = 'Pickle' THEN AmountEaten ELSE NULL END) AS Pickle,
///     MAX(CASE WHEN FoodName = 'Apple' THEN AmountEaten ELSE NULL END) AS Apple
/// FROM food_eaten
/// GROUP BY Date
/// ```
///
/// The algorithm:
/// 1. Detect PIVOT in FROM clause or JOINs
/// 2. Extract PIVOT specification (aggregate function, pivot column, pivot values)
/// 3. Generate CASE expression for each pivot value
/// 4. Wrap each CASE in the aggregate function
/// 5. Determine GROUP BY columns (all source columns except pivot_column and aggregate_column)
/// 6. Build new SelectStatement with CASE expressions and GROUP BY
pub struct PivotExpander;

impl PivotExpander {
    /// Transform a SELECT statement, expanding any PIVOT operations
    pub fn expand(statement: SelectStatement) -> Result<SelectStatement> {
        // Check if FROM contains a PIVOT
        if let Some(ref from_subquery) = statement.from_subquery {
            // Check if the subquery's FROM is a PIVOT (wrapped subquery pattern)
            return Self::expand_from_subquery(statement);
        }

        // Check if there's a direct PIVOT in FROM (this won't happen with current parser
        // as parser wraps PIVOT in a subquery, but handle it for completeness)
        Ok(statement)
    }

    /// Handle PIVOT in a subquery context
    fn expand_from_subquery(mut statement: SelectStatement) -> Result<SelectStatement> {
        if let Some(subquery) = statement.from_subquery.take() {
            // Recursively process the subquery first
            let processed_subquery = Self::expand(*subquery)?;
            statement.from_subquery = Some(Box::new(processed_subquery));
        }

        Ok(statement)
    }

    /// Expand a PIVOT operation into CASE expressions + GROUP BY
    pub fn expand_pivot(
        source: &TableSource,
        aggregate: &PivotAggregate,
        pivot_column: &str,
        pivot_values: &[String],
        alias: &Option<String>,
    ) -> Result<SelectStatement> {
        // Extract the base source table/subquery
        let (base_table, base_alias, base_subquery) = Self::extract_base_source(source)?;

        // Determine columns for GROUP BY
        // We need all columns from the source except pivot_column and aggregate.column
        let group_by_columns = Self::determine_group_by_columns(
            &base_table,
            &base_alias,
            &base_subquery,
            pivot_column,
            &aggregate.column,
        )?;

        // Build SELECT items
        let mut select_items = Vec::new();

        // Add GROUP BY columns to SELECT
        for col in &group_by_columns {
            select_items.push(SelectItem::Column {
                column: ColumnRef::unquoted(col.clone()),
                leading_comments: Vec::new(),
                trailing_comment: None,
            });
        }

        // Generate CASE expression for each pivot value
        for pivot_value in pivot_values {
            let case_expr = Self::build_pivot_case_expression(
                pivot_column,
                pivot_value,
                &aggregate.column,
                &aggregate.function,
            )?;

            select_items.push(SelectItem::Expression {
                expr: case_expr,
                alias: pivot_value.clone(),
                leading_comments: Vec::new(),
                trailing_comment: None,
            });
        }

        // Build the transformed statement
        let mut result = SelectStatement {
            distinct: false,
            columns: Vec::new(), // Deprecated field
            select_items,
            from_table: base_table,
            from_subquery: base_subquery,
            from_function: None,
            from_alias: base_alias.or_else(|| alias.clone()),
            joins: Vec::new(),
            where_clause: None,
            order_by: None,
            group_by: Some(
                group_by_columns
                    .iter()
                    .map(|col| SqlExpression::Column(ColumnRef::unquoted(col.clone())))
                    .collect(),
            ),
            having: None,
            qualify: None,
            limit: None,
            offset: None,
            ctes: Vec::new(),
            into_table: None,
            set_operations: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        Ok(result)
    }

    /// Extract the base source from a TableSource (table name or subquery)
    fn extract_base_source(
        source: &TableSource,
    ) -> Result<(Option<String>, Option<String>, Option<Box<SelectStatement>>)> {
        match source {
            TableSource::Table(name) => Ok((Some(name.clone()), None, None)),
            TableSource::DerivedTable { query, alias } => {
                Ok((None, Some(alias.clone()), Some(query.clone())))
            }
            TableSource::Pivot { .. } => Err(anyhow!("Nested PIVOT operations are not supported")),
        }
    }

    /// Determine which columns should be in the GROUP BY clause
    /// These are all columns except the pivot_column and the aggregate_column
    fn determine_group_by_columns(
        base_table: &Option<String>,
        base_alias: &Option<String>,
        base_subquery: &Option<Box<SelectStatement>>,
        pivot_column: &str,
        aggregate_column: &str,
    ) -> Result<Vec<String>> {
        // For now, we need to infer columns from the source
        // This is a simplified implementation - in production, you'd want to:
        // 1. Query the data source schema
        // 2. Extract columns from subquery SELECT items
        // 3. Handle qualified column names

        if let Some(subquery) = base_subquery {
            // Extract column names from subquery's SELECT items
            let mut columns = Vec::new();
            for item in &subquery.select_items {
                match item {
                    SelectItem::Column { column, .. } => {
                        let col_name = column.name.clone();
                        if col_name != pivot_column && col_name != aggregate_column {
                            columns.push(col_name);
                        }
                    }
                    SelectItem::Expression { alias, .. } => {
                        if alias != pivot_column && alias != aggregate_column {
                            columns.push(alias.clone());
                        }
                    }
                    SelectItem::Star { .. } => {
                        // Cannot determine columns from *, would need schema info
                        return Err(anyhow!(
                            "PIVOT with SELECT * is not supported. Please specify columns explicitly."
                        ));
                    }
                    SelectItem::StarExclude { .. } => {
                        return Err(anyhow!(
                            "PIVOT with SELECT * EXCLUDE is not supported. Please specify columns explicitly."
                        ));
                    }
                }
            }
            Ok(columns)
        } else {
            // For table sources, we'd need schema information
            // This is a limitation - in production, integrate with schema discovery
            Err(anyhow!(
                "PIVOT on table sources requires explicit column specification. \
                 Use a subquery: SELECT col1, col2, pivot_col, agg_col FROM table"
            ))
        }
    }

    /// Build a CASE expression for a single pivot value
    /// Example: MAX(CASE WHEN FoodName = 'Sammich' THEN AmountEaten ELSE NULL END)
    fn build_pivot_case_expression(
        pivot_column: &str,
        pivot_value: &str,
        aggregate_column: &str,
        aggregate_function: &str,
    ) -> Result<SqlExpression> {
        // Build: CASE WHEN pivot_column = 'pivot_value' THEN aggregate_column ELSE NULL END
        let case_expr = SqlExpression::CaseExpression {
            when_branches: vec![WhenBranch {
                condition: Box::new(SqlExpression::BinaryOp {
                    left: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                        pivot_column.to_string(),
                    ))),
                    op: "=".to_string(),
                    right: Box::new(SqlExpression::StringLiteral(pivot_value.to_string())),
                }),
                result: Box::new(SqlExpression::Column(ColumnRef::unquoted(
                    aggregate_column.to_string(),
                ))),
            }],
            else_branch: Some(Box::new(SqlExpression::Null)),
        };

        // Wrap in aggregate function: aggregate_function(case_expr)
        let aggregated = SqlExpression::FunctionCall {
            name: aggregate_function.to_uppercase(),
            args: vec![case_expr],
            distinct: false,
        };

        Ok(aggregated)
    }
}

impl ASTTransformer for PivotExpander {
    fn name(&self) -> &str {
        "PivotExpander"
    }

    fn description(&self) -> &str {
        "Expands PIVOT operations into CASE expressions with GROUP BY"
    }

    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
        Self::expand(stmt)
    }
}

impl Default for PivotExpander {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pivot_case_expression() {
        let expr =
            PivotExpander::build_pivot_case_expression("FoodName", "Sammich", "AmountEaten", "MAX")
                .unwrap();

        // Verify it's a function call
        match expr {
            SqlExpression::FunctionCall { name, args, .. } => {
                assert_eq!(name, "MAX");
                assert_eq!(args.len(), 1);

                // Verify the CASE expression inside
                match &args[0] {
                    SqlExpression::CaseExpression {
                        when_branches,
                        else_branch,
                    } => {
                        assert_eq!(when_branches.len(), 1);
                        assert!(else_branch.is_some());
                    }
                    _ => panic!("Expected CaseExpression inside function call"),
                }
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_determine_group_by_columns_with_subquery() {
        // Create a simple subquery: SELECT Date, FoodName, AmountEaten FROM food_eaten
        let subquery = SelectStatement {
            distinct: false,
            columns: Vec::new(),
            select_items: vec![
                SelectItem::Column {
                    column: ColumnRef::unquoted("Date".to_string()),
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                },
                SelectItem::Column {
                    column: ColumnRef::unquoted("FoodName".to_string()),
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                },
                SelectItem::Column {
                    column: ColumnRef::unquoted("AmountEaten".to_string()),
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                },
            ],
            from_table: Some("food_eaten".to_string()),
            from_subquery: None,
            from_function: None,
            from_alias: None,
            joins: Vec::new(),
            where_clause: None,
            order_by: None,
            group_by: None,
            having: None,
            qualify: None,
            limit: None,
            offset: None,
            ctes: Vec::new(),
            into_table: None,
            set_operations: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        let columns = PivotExpander::determine_group_by_columns(
            &None,
            &Some("src".to_string()),
            &Some(Box::new(subquery)),
            "FoodName",
            "AmountEaten",
        )
        .unwrap();

        // Should only return "Date" (excluding FoodName and AmountEaten)
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0], "Date");
    }
}
