//! Query Rewriter Module
//!
//! This module analyzes SQL queries and suggests/performs transformations
//! to make them compatible with the SQL engine's capabilities.
//!
//! Main transformations:
//! - Hoist expressions from aggregate/window functions into CTEs
//! - Convert complex expressions to simpler forms
//! - Identify patterns that need rewriting

use crate::sql::parser::ast::{CTEType, SelectStatement, SqlExpression, CTE};
use serde::{Deserialize, Serialize};

/// Represents a suggested rewrite for a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteSuggestion {
    /// Type of rewrite needed
    pub rewrite_type: RewriteType,
    /// Location in original query (if available)
    pub location: Option<String>,
    /// Description of the issue
    pub issue: String,
    /// Suggested fix
    pub suggestion: String,
    /// The rewritten SQL (if automatic rewrite is possible)
    pub rewritten_sql: Option<String>,
    /// CTE that could be added to fix the issue
    pub suggested_cte: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewriteType {
    /// Expression in aggregate function needs hoisting
    AggregateExpressionHoist,
    /// Expression in window function needs hoisting
    WindowExpressionHoist,
    /// Complex WHERE clause expression needs simplification
    WhereExpressionHoist,
    /// LAG/LEAD with expression needs hoisting
    LagLeadExpressionHoist,
    /// Complex JOIN condition needs simplification
    JoinConditionHoist,
    /// Nested aggregate functions
    NestedAggregateHoist,
}

/// Analyzes a query and returns rewrite suggestions
pub struct QueryRewriter {
    suggestions: Vec<RewriteSuggestion>,
}

impl QueryRewriter {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
        }
    }

    /// Analyze a query and return suggestions
    pub fn analyze(&mut self, stmt: &SelectStatement) -> Vec<RewriteSuggestion> {
        self.suggestions.clear();

        // Analyze SELECT items for complex expressions
        self.analyze_select_items(stmt);

        // Analyze WHERE clause
        if let Some(where_clause) = &stmt.where_clause {
            self.analyze_where_clause(where_clause);
        }

        // Analyze GROUP BY
        if let Some(group_by) = &stmt.group_by {
            self.analyze_group_by(group_by);
        }

        // Analyze existing CTEs for issues
        for cte in &stmt.ctes {
            self.analyze_cte(cte);
        }

        self.suggestions.clone()
    }

    /// Check SELECT items for expressions that need hoisting
    fn analyze_select_items(&mut self, stmt: &SelectStatement) {
        for item in &stmt.select_items {
            if let crate::sql::parser::ast::SelectItem::Expression { expr, alias } = item {
                self.check_expression_for_hoisting(expr, Some(alias));
            }
        }
    }

    /// Check if an expression needs hoisting
    fn check_expression_for_hoisting(&mut self, expr: &SqlExpression, context: Option<&str>) {
        match expr {
            SqlExpression::WindowFunction { name, args, .. } => {
                // Check if window function has complex expressions
                for arg in args {
                    if self.is_complex_expression(arg) {
                        self.suggestions.push(RewriteSuggestion {
                            rewrite_type: RewriteType::WindowExpressionHoist,
                            location: context.map(|s| s.to_string()),
                            issue: format!("Window function {} contains complex expression", name),
                            suggestion: "Hoist the expression to a CTE and reference the column"
                                .to_string(),
                            rewritten_sql: None,
                            suggested_cte: Some(self.generate_hoist_cte(arg, "expr_cte")),
                        });
                    }
                }
            }
            SqlExpression::FunctionCall { name, args, .. } => {
                // Check if it's an aggregate function with complex expression
                if self.is_aggregate_function(name) {
                    for arg in args {
                        if self.is_complex_expression(arg) {
                            self.suggestions.push(RewriteSuggestion {
                                rewrite_type: RewriteType::AggregateExpressionHoist,
                                location: context.map(|s| s.to_string()),
                                issue: format!("Aggregate function {} contains expression: {:?}", name, arg),
                                suggestion: "Create a CTE with the calculated expression, then aggregate the result column".to_string(),
                                rewritten_sql: None,
                                suggested_cte: Some(self.generate_hoist_cte(arg, "calc_cte")),
                            });
                        }
                    }
                }

                // Check for LAG/LEAD with expressions
                if name == "LAG" || name == "LEAD" {
                    if let Some(first_arg) = args.first() {
                        if self.is_complex_expression(first_arg) {
                            self.suggestions.push(RewriteSuggestion {
                                rewrite_type: RewriteType::LagLeadExpressionHoist,
                                location: context.map(|s| s.to_string()),
                                issue: format!("{} function contains expression instead of column reference", name),
                                suggestion: format!("Calculate expression in a CTE, then apply {} to the result column", name),
                                rewritten_sql: None,
                                suggested_cte: Some(self.generate_hoist_cte(first_arg, "lag_lead_cte")),
                            });
                        }
                    }
                }
            }
            SqlExpression::BinaryOp { left, right, .. } => {
                // Recursively check both sides
                self.check_expression_for_hoisting(left, context);
                self.check_expression_for_hoisting(right, context);
            }
            _ => {}
        }
    }

    /// Check if an expression is complex (not just a column reference)
    fn is_complex_expression(&self, expr: &SqlExpression) -> bool {
        !matches!(
            expr,
            SqlExpression::Column(_)
                | SqlExpression::NumberLiteral(_)
                | SqlExpression::StringLiteral(_)
        )
    }

    /// Check if a function is an aggregate function
    fn is_aggregate_function(&self, name: &str) -> bool {
        matches!(
            name.to_uppercase().as_str(),
            "SUM" | "AVG" | "COUNT" | "MIN" | "MAX" | "STDDEV" | "VARIANCE" | "MEDIAN"
        )
    }

    /// Generate a suggested CTE for hoisting an expression
    fn generate_hoist_cte(&self, expr: &SqlExpression, cte_name: &str) -> String {
        let expr_str = self.expression_to_sql(expr);
        format!(
            "{} AS (\n    SELECT \n        *,\n        {} AS calculated_value\n    FROM previous_table\n)",
            cte_name, expr_str
        )
    }

    /// Convert an expression to SQL string
    fn expression_to_sql(&self, expr: &SqlExpression) -> String {
        match expr {
            SqlExpression::Column(col_ref) => col_ref.to_sql(),
            SqlExpression::BinaryOp { left, right, op } => {
                format!(
                    "{} {} {}",
                    self.expression_to_sql(left),
                    op,
                    self.expression_to_sql(right)
                )
            }
            SqlExpression::NumberLiteral(n) => n.clone(),
            SqlExpression::StringLiteral(s) => format!("'{}'", s),
            SqlExpression::FunctionCall { name, args, .. } => {
                let arg_strs: Vec<String> =
                    args.iter().map(|a| self.expression_to_sql(a)).collect();
                format!("{}({})", name, arg_strs.join(", "))
            }
            _ => format!("{:?}", expr), // Fallback for complex types
        }
    }

    fn analyze_where_clause(&mut self, _where_clause: &crate::sql::parser::ast::WhereClause) {
        // TODO: Analyze WHERE clause for complex expressions
    }

    fn analyze_group_by(&mut self, _group_by: &[SqlExpression]) {
        // TODO: Analyze GROUP BY for complex expressions
    }

    fn analyze_cte(&mut self, cte: &CTE) {
        // Recursively analyze CTEs
        if let CTEType::Standard(query) = &cte.cte_type {
            let mut sub_rewriter = QueryRewriter::new();
            sub_rewriter.analyze(query);
            for mut suggestion in sub_rewriter.suggestions {
                // Prepend CTE name to location
                suggestion.location = Some(format!(
                    "CTE '{}': {}",
                    cte.name,
                    suggestion.location.unwrap_or_default()
                ));
                self.suggestions.push(suggestion);
            }
        }
    }

    /// Attempt to automatically rewrite a query
    pub fn rewrite(&self, stmt: &SelectStatement) -> Option<SelectStatement> {
        // This would implement actual rewriting logic
        // For now, we just analyze and suggest
        None
    }
}

/// JSON output for CLI integration
#[derive(Debug, Serialize, Deserialize)]
pub struct RewriteAnalysis {
    pub success: bool,
    pub suggestions: Vec<RewriteSuggestion>,
    pub can_auto_rewrite: bool,
    pub rewritten_query: Option<String>,
}

impl RewriteAnalysis {
    pub fn from_suggestions(suggestions: Vec<RewriteSuggestion>) -> Self {
        let can_auto_rewrite = suggestions.iter().any(|s| s.rewritten_sql.is_some());
        Self {
            success: true,
            suggestions,
            can_auto_rewrite,
            rewritten_query: None,
        }
    }
}
