// Subquery execution handler
// Walks the AST to evaluate subqueries and replace them with their results

use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::query_engine::QueryEngine;
use crate::sql::parser::ast::{Condition, SelectItem, SelectStatement, SqlExpression, WhereClause};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info};

/// Result of executing a subquery
#[derive(Debug, Clone)]
pub enum SubqueryResult {
    /// Scalar subquery returned a single value
    Scalar(DataValue),
    /// IN subquery returned a set of values
    ValueSet(HashSet<DataValue>),
    /// Subquery returned multiple rows/columns (for future use)
    Table(Arc<DataView>),
}

/// Executes subqueries within a SQL statement
pub struct SubqueryExecutor {
    query_engine: QueryEngine,
    source_table: Arc<DataTable>,
    /// Cache of executed subqueries to avoid re-execution
    cache: HashMap<String, SubqueryResult>,
    /// CTE context for resolving CTE references in subqueries
    cte_context: HashMap<String, Arc<DataView>>,
}

impl SubqueryExecutor {
    /// Create a new subquery executor
    pub fn new(query_engine: QueryEngine, source_table: Arc<DataTable>) -> Self {
        Self {
            query_engine,
            source_table,
            cache: HashMap::new(),
            cte_context: HashMap::new(),
        }
    }

    /// Create a new subquery executor with CTE context
    pub fn with_cte_context(
        query_engine: QueryEngine,
        source_table: Arc<DataTable>,
        cte_context: HashMap<String, Arc<DataView>>,
    ) -> Self {
        Self {
            query_engine,
            source_table,
            cache: HashMap::new(),
            cte_context,
        }
    }

    /// Execute all subqueries in a statement and return a modified statement
    /// with subqueries replaced by their results
    pub fn execute_subqueries(&mut self, statement: &SelectStatement) -> Result<SelectStatement> {
        info!("SubqueryExecutor: Starting subquery execution pass");
        info!(
            "SubqueryExecutor: Available CTEs: {:?}",
            self.cte_context.keys().collect::<Vec<_>>()
        );

        // Clone the statement to modify
        let mut modified_statement = statement.clone();

        // Process WHERE clause if present
        if let Some(ref where_clause) = statement.where_clause {
            debug!("SubqueryExecutor: Processing WHERE clause for subqueries");
            let mut new_conditions = Vec::new();
            for condition in &where_clause.conditions {
                new_conditions.push(Condition {
                    expr: self.process_expression(&condition.expr)?,
                    connector: condition.connector.clone(),
                });
            }
            modified_statement.where_clause = Some(WhereClause {
                conditions: new_conditions,
            });
        }

        // Process SELECT items
        let mut new_select_items = Vec::new();
        for item in &statement.select_items {
            match item {
                SelectItem::Column(col) => {
                    new_select_items.push(SelectItem::Column(col.clone()));
                }
                SelectItem::Expression { expr, alias } => {
                    new_select_items.push(SelectItem::Expression {
                        expr: self.process_expression(expr)?,
                        alias: alias.clone(),
                    });
                }
                SelectItem::Star => {
                    new_select_items.push(SelectItem::Star);
                }
            }
        }
        modified_statement.select_items = new_select_items;

        // Process HAVING clause if present
        if let Some(ref having) = statement.having {
            debug!("SubqueryExecutor: Processing HAVING clause for subqueries");
            modified_statement.having = Some(self.process_expression(having)?);
        }

        debug!("SubqueryExecutor: Subquery execution complete");
        Ok(modified_statement)
    }

    /// Process an expression, executing any subqueries and replacing them with results
    fn process_expression(&mut self, expr: &SqlExpression) -> Result<SqlExpression> {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                debug!("SubqueryExecutor: Executing scalar subquery");
                let result = self.execute_scalar_subquery(query)?;
                Ok(result)
            }

            SqlExpression::InSubquery { expr, subquery } => {
                debug!("SubqueryExecutor: Executing IN subquery");
                let values = self.execute_in_subquery(subquery)?;

                // Replace with InList containing the actual values
                Ok(SqlExpression::InList {
                    expr: Box::new(self.process_expression(expr)?),
                    values: values
                        .into_iter()
                        .map(|v| match v {
                            DataValue::Null => SqlExpression::Null,
                            DataValue::Integer(i) => SqlExpression::NumberLiteral(i.to_string()),
                            DataValue::Float(f) => SqlExpression::NumberLiteral(f.to_string()),
                            DataValue::String(s) => SqlExpression::StringLiteral(s),
                            DataValue::InternedString(s) => {
                                SqlExpression::StringLiteral(s.to_string())
                            }
                            DataValue::Boolean(b) => SqlExpression::BooleanLiteral(b),
                            DataValue::DateTime(dt) => SqlExpression::StringLiteral(dt),
                        })
                        .collect(),
                })
            }

            SqlExpression::NotInSubquery { expr, subquery } => {
                debug!("SubqueryExecutor: Executing NOT IN subquery");
                let values = self.execute_in_subquery(subquery)?;

                // Replace with NotInList containing the actual values
                Ok(SqlExpression::NotInList {
                    expr: Box::new(self.process_expression(expr)?),
                    values: values
                        .into_iter()
                        .map(|v| match v {
                            DataValue::Null => SqlExpression::Null,
                            DataValue::Integer(i) => SqlExpression::NumberLiteral(i.to_string()),
                            DataValue::Float(f) => SqlExpression::NumberLiteral(f.to_string()),
                            DataValue::String(s) => SqlExpression::StringLiteral(s),
                            DataValue::InternedString(s) => {
                                SqlExpression::StringLiteral(s.to_string())
                            }
                            DataValue::Boolean(b) => SqlExpression::BooleanLiteral(b),
                            DataValue::DateTime(dt) => SqlExpression::StringLiteral(dt),
                        })
                        .collect(),
                })
            }

            // Process nested expressions
            SqlExpression::BinaryOp { left, op, right } => Ok(SqlExpression::BinaryOp {
                left: Box::new(self.process_expression(left)?),
                op: op.clone(),
                right: Box::new(self.process_expression(right)?),
            }),

            // Note: UnaryOp doesn't exist in the current AST, handle negation differently
            // This case might need to be removed or adapted based on actual AST structure
            SqlExpression::Between { expr, lower, upper } => Ok(SqlExpression::Between {
                expr: Box::new(self.process_expression(expr)?),
                lower: Box::new(self.process_expression(lower)?),
                upper: Box::new(self.process_expression(upper)?),
            }),

            SqlExpression::InList { expr, values } => Ok(SqlExpression::InList {
                expr: Box::new(self.process_expression(expr)?),
                values: values
                    .iter()
                    .map(|v| self.process_expression(v))
                    .collect::<Result<Vec<_>>>()?,
            }),

            SqlExpression::NotInList { expr, values } => Ok(SqlExpression::NotInList {
                expr: Box::new(self.process_expression(expr)?),
                values: values
                    .iter()
                    .map(|v| self.process_expression(v))
                    .collect::<Result<Vec<_>>>()?,
            }),

            // CaseWhen doesn't exist in current AST, skip for now
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => Ok(SqlExpression::FunctionCall {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| self.process_expression(a))
                    .collect::<Result<Vec<_>>>()?,
                distinct: *distinct,
            }),

            // Pass through expressions that don't contain subqueries
            _ => Ok(expr.clone()),
        }
    }

    /// Execute a scalar subquery and return a single value
    fn execute_scalar_subquery(&mut self, query: &SelectStatement) -> Result<SqlExpression> {
        let cache_key = format!("scalar:{:?}", query);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("SubqueryExecutor: Using cached scalar subquery result");
            if let SubqueryResult::Scalar(value) = cached {
                return Ok(self.datavalue_to_expression(value.clone()));
            }
        }

        info!("SubqueryExecutor: Executing scalar subquery");

        // Execute the subquery using execute_statement_with_cte_context
        let result_view = self.query_engine.execute_statement_with_cte_context(
            self.source_table.clone(),
            query.clone(),
            &self.cte_context,
        )?;

        // Scalar subquery must return exactly one row and one column
        if result_view.row_count() != 1 {
            return Err(anyhow!(
                "Scalar subquery returned {} rows, expected exactly 1",
                result_view.row_count()
            ));
        }

        if result_view.column_count() != 1 {
            return Err(anyhow!(
                "Scalar subquery returned {} columns, expected exactly 1",
                result_view.column_count()
            ));
        }

        // Get the single value
        let value = if let Some(row) = result_view.get_row(0) {
            row.values.get(0).cloned().unwrap_or(DataValue::Null)
        } else {
            DataValue::Null
        };

        // Cache the result
        self.cache
            .insert(cache_key, SubqueryResult::Scalar(value.clone()));

        Ok(self.datavalue_to_expression(value))
    }

    /// Execute an IN subquery and return a set of values
    fn execute_in_subquery(&mut self, query: &SelectStatement) -> Result<Vec<DataValue>> {
        let cache_key = format!("in:{:?}", query);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("SubqueryExecutor: Using cached IN subquery result");
            if let SubqueryResult::ValueSet(values) = cached {
                return Ok(values.iter().cloned().collect());
            }
        }

        info!("SubqueryExecutor: Executing IN subquery");
        debug!(
            "SubqueryExecutor: Available CTEs in context: {:?}",
            self.cte_context.keys().collect::<Vec<_>>()
        );
        debug!("SubqueryExecutor: Subquery: {:?}", query);

        // Execute the subquery using execute_statement_with_cte_context
        let result_view = self.query_engine.execute_statement_with_cte_context(
            self.source_table.clone(),
            query.clone(),
            &self.cte_context,
        )?;

        debug!(
            "SubqueryExecutor: IN subquery returned {} rows",
            result_view.row_count()
        );

        // IN subquery must return exactly one column
        if result_view.column_count() != 1 {
            return Err(anyhow!(
                "IN subquery returned {} columns, expected exactly 1",
                result_view.column_count()
            ));
        }

        // Collect all values from the first column
        let mut values = HashSet::new();
        for row_idx in 0..result_view.row_count() {
            if let Some(row) = result_view.get_row(row_idx) {
                if let Some(value) = row.values.get(0) {
                    values.insert(value.clone());
                }
            }
        }

        // Cache the result
        self.cache
            .insert(cache_key, SubqueryResult::ValueSet(values.clone()));

        Ok(values.into_iter().collect())
    }

    /// Convert a DataValue to a SqlExpression
    fn datavalue_to_expression(&self, value: DataValue) -> SqlExpression {
        match value {
            DataValue::Null => SqlExpression::Null,
            DataValue::Integer(i) => SqlExpression::NumberLiteral(i.to_string()),
            DataValue::Float(f) => SqlExpression::NumberLiteral(f.to_string()),
            DataValue::String(s) => SqlExpression::StringLiteral(s),
            DataValue::InternedString(s) => SqlExpression::StringLiteral(s.to_string()),
            DataValue::Boolean(b) => SqlExpression::BooleanLiteral(b),
            DataValue::DateTime(dt) => SqlExpression::StringLiteral(dt),
        }
    }
}
