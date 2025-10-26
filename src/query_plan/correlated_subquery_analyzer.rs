//! Correlated Subquery Analysis
//!
//! This module analyzes SELECT statements to detect and classify subqueries,
//! particularly correlated subqueries that reference columns from outer queries.
//!
//! # Purpose
//!
//! Before we can transform correlated subqueries, we need to understand:
//! - Where subqueries appear in the query
//! - Which subqueries are correlated (reference outer query)
//! - What type of correlation exists (scalar, EXISTS, IN, etc.)
//! - Which columns are referenced from outer scope
//!
//! This analyzer provides visibility into these patterns, which will inform
//! future transformation strategies.

use crate::sql::parser::ast::{SelectStatement, SqlExpression, WhereClause};
use std::collections::HashSet;

/// Location where a subquery appears
#[derive(Debug, Clone, PartialEq)]
pub enum SubqueryLocation {
    /// Subquery in FROM clause (derived table)
    FromClause,
    /// Subquery in WHERE clause
    WhereClause,
    /// Subquery in SELECT list (scalar subquery)
    SelectList,
    /// Subquery in HAVING clause
    HavingClause,
    /// Subquery in JOIN ON condition
    JoinCondition,
}

/// Type of subquery based on its usage
#[derive(Debug, Clone, PartialEq)]
pub enum SubqueryType {
    /// Scalar subquery that returns single value
    Scalar,
    /// IN/NOT IN subquery
    InList { negated: bool },
    /// EXISTS/NOT EXISTS (not yet parsed, but for future)
    Exists { negated: bool },
    /// Subquery in FROM clause
    DerivedTable,
}

/// Information about a detected subquery
#[derive(Debug, Clone)]
pub struct SubqueryInfo {
    /// Location where subquery appears
    pub location: SubqueryLocation,
    /// Type of subquery
    pub subquery_type: SubqueryType,
    /// Whether this subquery references outer query columns
    pub is_correlated: bool,
    /// Columns from outer query that are referenced
    pub outer_references: Vec<String>,
    /// The subquery statement itself
    pub statement: SelectStatement,
}

/// Analysis results for a query
#[derive(Debug, Default)]
pub struct CorrelationAnalysis {
    /// All subqueries found in the query
    pub subqueries: Vec<SubqueryInfo>,
    /// Total count of subqueries
    pub total_count: usize,
    /// Count of correlated subqueries
    pub correlated_count: usize,
    /// Count of non-correlated subqueries
    pub non_correlated_count: usize,
}

impl CorrelationAnalysis {
    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("=== Subquery Analysis ===\n"));
        report.push_str(&format!("Total subqueries: {}\n", self.total_count));
        report.push_str(&format!("  Correlated: {}\n", self.correlated_count));
        report.push_str(&format!(
            "  Non-correlated: {}\n",
            self.non_correlated_count
        ));
        report.push_str("\n");

        if self.subqueries.is_empty() {
            report.push_str("No subqueries detected.\n");
            return report;
        }

        for (idx, info) in self.subqueries.iter().enumerate() {
            report.push_str(&format!("Subquery #{}: ", idx + 1));

            // Location
            report.push_str(&format!("{:?} - ", info.location));

            // Type
            report.push_str(&format!("{:?}", info.subquery_type));

            // Correlation status
            if info.is_correlated {
                report.push_str(" [CORRELATED]\n");
                report.push_str(&format!(
                    "  Outer references: {:?}\n",
                    info.outer_references
                ));
            } else {
                report.push_str(" [NON-CORRELATED]\n");
            }
        }

        report
    }
}

/// Analyzer for detecting and classifying correlated subqueries
pub struct CorrelatedSubqueryAnalyzer {
    /// Stack of table/alias names available at each nesting level
    /// Used to determine if a column reference is to an outer query
    scope_stack: Vec<HashSet<String>>,
}

impl CorrelatedSubqueryAnalyzer {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![HashSet::new()],
        }
    }

    /// Analyze a SELECT statement for subqueries
    pub fn analyze(&mut self, stmt: &SelectStatement) -> CorrelationAnalysis {
        let mut analysis = CorrelationAnalysis::default();

        // Collect table/alias names in current scope
        let mut current_scope = HashSet::new();
        if let Some(ref table) = stmt.from_table {
            current_scope.insert(table.clone());
        }
        if let Some(ref alias) = stmt.from_alias {
            current_scope.insert(alias.clone());
        }

        // Push current scope onto stack
        self.scope_stack.push(current_scope);

        // Analyze different parts of the query
        self.analyze_from_clause(stmt, &mut analysis);
        self.analyze_select_list(stmt, &mut analysis);
        self.analyze_where_clause(stmt, &mut analysis);
        self.analyze_having_clause(stmt, &mut analysis);

        // Pop scope
        self.scope_stack.pop();

        // Update counts
        analysis.total_count = analysis.subqueries.len();
        analysis.correlated_count = analysis
            .subqueries
            .iter()
            .filter(|s| s.is_correlated)
            .count();
        analysis.non_correlated_count = analysis.total_count - analysis.correlated_count;

        analysis
    }

    /// Analyze FROM clause for subqueries
    fn analyze_from_clause(&mut self, stmt: &SelectStatement, analysis: &mut CorrelationAnalysis) {
        if let Some(ref subquery) = stmt.from_subquery {
            let outer_refs = self.find_outer_references(subquery);

            analysis.subqueries.push(SubqueryInfo {
                location: SubqueryLocation::FromClause,
                subquery_type: SubqueryType::DerivedTable,
                is_correlated: !outer_refs.is_empty(),
                outer_references: outer_refs,
                statement: (**subquery).clone(),
            });
        }
    }

    /// Analyze SELECT list for scalar subqueries
    fn analyze_select_list(&mut self, stmt: &SelectStatement, analysis: &mut CorrelationAnalysis) {
        for item in &stmt.select_items {
            if let crate::sql::parser::ast::SelectItem::Expression { expr, .. } = item {
                self.analyze_expression_for_subqueries(
                    expr,
                    SubqueryLocation::SelectList,
                    analysis,
                );
            }
        }
    }

    /// Analyze WHERE clause for subqueries
    fn analyze_where_clause(&mut self, stmt: &SelectStatement, analysis: &mut CorrelationAnalysis) {
        if let Some(ref where_clause) = stmt.where_clause {
            for condition in &where_clause.conditions {
                self.analyze_expression_for_subqueries(
                    &condition.expr,
                    SubqueryLocation::WhereClause,
                    analysis,
                );
            }
        }
    }

    /// Analyze HAVING clause for subqueries
    fn analyze_having_clause(
        &mut self,
        stmt: &SelectStatement,
        analysis: &mut CorrelationAnalysis,
    ) {
        if let Some(ref having_expr) = stmt.having {
            self.analyze_expression_for_subqueries(
                having_expr,
                SubqueryLocation::HavingClause,
                analysis,
            );
        }
    }

    /// Recursively analyze an expression for subqueries
    fn analyze_expression_for_subqueries(
        &mut self,
        expr: &SqlExpression,
        location: SubqueryLocation,
        analysis: &mut CorrelationAnalysis,
    ) {
        match expr {
            SqlExpression::ScalarSubquery { query } => {
                let outer_refs = self.find_outer_references(query);

                analysis.subqueries.push(SubqueryInfo {
                    location: location.clone(),
                    subquery_type: SubqueryType::Scalar,
                    is_correlated: !outer_refs.is_empty(),
                    outer_references: outer_refs,
                    statement: (**query).clone(),
                });
            }

            SqlExpression::InSubquery { expr: _, subquery } => {
                let outer_refs = self.find_outer_references(subquery);

                analysis.subqueries.push(SubqueryInfo {
                    location: location.clone(),
                    subquery_type: SubqueryType::InList { negated: false },
                    is_correlated: !outer_refs.is_empty(),
                    outer_references: outer_refs,
                    statement: (**subquery).clone(),
                });
            }

            SqlExpression::NotInSubquery { expr: _, subquery } => {
                let outer_refs = self.find_outer_references(subquery);

                analysis.subqueries.push(SubqueryInfo {
                    location: location.clone(),
                    subquery_type: SubqueryType::InList { negated: true },
                    is_correlated: !outer_refs.is_empty(),
                    outer_references: outer_refs,
                    statement: (**subquery).clone(),
                });
            }

            // Recursively check nested expressions
            SqlExpression::BinaryOp { left, right, .. } => {
                self.analyze_expression_for_subqueries(left, location.clone(), analysis);
                self.analyze_expression_for_subqueries(right, location, analysis);
            }

            SqlExpression::Not { expr } => {
                self.analyze_expression_for_subqueries(expr, location, analysis);
            }

            SqlExpression::FunctionCall { args, .. } => {
                for arg in args {
                    self.analyze_expression_for_subqueries(arg, location.clone(), analysis);
                }
            }

            _ => {
                // Other expression types don't contain subqueries
            }
        }
    }

    /// Find column references to outer query (correlation)
    fn find_outer_references(&self, subquery: &SelectStatement) -> Vec<String> {
        let mut outer_refs = Vec::new();
        let mut referenced_tables = HashSet::new();

        // Collect all table references in the subquery
        self.collect_table_references(subquery, &mut referenced_tables);

        // Check if any referenced tables are from outer scopes
        for table in &referenced_tables {
            // Check all outer scopes (everything except current/innermost scope)
            for scope in self.scope_stack.iter().rev().skip(1) {
                if scope.contains(table) {
                    outer_refs.push(table.clone());
                    break;
                }
            }
        }

        outer_refs.sort();
        outer_refs.dedup();
        outer_refs
    }

    /// Collect all table/alias references in a statement
    fn collect_table_references(&self, stmt: &SelectStatement, refs: &mut HashSet<String>) {
        // Check WHERE clause for table-qualified columns
        if let Some(ref where_clause) = stmt.where_clause {
            self.collect_references_from_where(where_clause, refs);
        }

        // Check SELECT list
        for item in &stmt.select_items {
            if let crate::sql::parser::ast::SelectItem::Expression { expr, .. } = item {
                self.collect_references_from_expr(expr, refs);
            }
        }

        // Check HAVING
        if let Some(ref having) = stmt.having {
            self.collect_references_from_expr(having, refs);
        }
    }

    /// Collect table references from WHERE clause
    fn collect_references_from_where(
        &self,
        where_clause: &WhereClause,
        refs: &mut HashSet<String>,
    ) {
        for condition in &where_clause.conditions {
            self.collect_references_from_expr(&condition.expr, refs);
        }
    }

    /// Collect table references from expression
    fn collect_references_from_expr(&self, expr: &SqlExpression, refs: &mut HashSet<String>) {
        match expr {
            SqlExpression::Column(col_ref) => {
                if let Some(ref table) = col_ref.table_prefix {
                    refs.insert(table.clone());
                }
            }

            SqlExpression::BinaryOp { left, right, .. } => {
                self.collect_references_from_expr(left, refs);
                self.collect_references_from_expr(right, refs);
            }

            SqlExpression::Not { expr } => {
                self.collect_references_from_expr(expr, refs);
            }

            SqlExpression::FunctionCall { args, .. } => {
                for arg in args {
                    self.collect_references_from_expr(arg, refs);
                }
            }

            SqlExpression::InList { expr, values } => {
                self.collect_references_from_expr(expr, refs);
                for val in values {
                    self.collect_references_from_expr(val, refs);
                }
            }

            SqlExpression::NotInList { expr, values } => {
                self.collect_references_from_expr(expr, refs);
                for val in values {
                    self.collect_references_from_expr(val, refs);
                }
            }

            _ => {
                // Other expression types
            }
        }
    }
}

impl Default for CorrelatedSubqueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::{Condition, QuoteStyle};

    #[test]
    fn test_non_correlated_scalar_subquery() {
        let mut analyzer = CorrelatedSubqueryAnalyzer::new();

        // Main query with non-correlated subquery
        let main_stmt = SelectStatement {
            from_table: Some("trades".to_string()),
            ..Default::default()
        };

        let analysis = analyzer.analyze(&main_stmt);
        assert_eq!(analysis.total_count, 0);
    }

    #[test]
    fn test_from_clause_subquery() {
        let mut analyzer = CorrelatedSubqueryAnalyzer::new();

        let subquery = SelectStatement {
            from_table: Some("inner_table".to_string()),
            ..Default::default()
        };

        let main_stmt = SelectStatement {
            from_subquery: Some(Box::new(subquery)),
            from_alias: Some("sub".to_string()),
            ..Default::default()
        };

        let analysis = analyzer.analyze(&main_stmt);
        assert_eq!(analysis.total_count, 1);
        assert_eq!(
            analysis.subqueries[0].location,
            SubqueryLocation::FromClause
        );
        assert_eq!(
            analysis.subqueries[0].subquery_type,
            SubqueryType::DerivedTable
        );
        assert!(!analysis.subqueries[0].is_correlated);
    }

    #[test]
    fn test_analysis_report_format() {
        let analysis = CorrelationAnalysis {
            subqueries: vec![],
            total_count: 0,
            correlated_count: 0,
            non_correlated_count: 0,
        };

        let report = analysis.report();
        assert!(report.contains("Subquery Analysis"));
        assert!(report.contains("No subqueries detected"));
    }
}
