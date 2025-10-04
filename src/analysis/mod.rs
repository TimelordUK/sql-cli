// Analysis module - Provides structured query analysis for IDE/plugin integration
// This enables tools to understand SQL structure without manual text parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::sql::parser::ast::{CTEType, SelectItem, SelectStatement, CTE};

/// Comprehensive query analysis result
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryAnalysis {
    /// Whether the query is syntactically valid
    pub valid: bool,
    /// Type of query: "SELECT", "CTE", etc.
    pub query_type: String,
    /// Whether query contains SELECT *
    pub has_star: bool,
    /// Locations of SELECT * in query
    pub star_locations: Vec<StarLocation>,
    /// Tables referenced in query
    pub tables: Vec<String>,
    /// Columns explicitly referenced
    pub columns: Vec<String>,
    /// CTEs in query
    pub ctes: Vec<CteAnalysis>,
    /// FROM clause information
    pub from_clause: Option<FromClauseInfo>,
    /// WHERE clause information
    pub where_clause: Option<WhereClauseInfo>,
    /// Parse/validation errors
    pub errors: Vec<String>,
}

/// Location of SELECT * in query
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StarLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Context: "main_query", "cte:name", "subquery"
    pub context: String,
}

/// CTE analysis information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CteAnalysis {
    /// CTE name
    pub name: String,
    /// CTE type: "Standard", "WEB", "Recursive"
    pub cte_type: String,
    /// Start line (will be populated when parser tracks positions)
    pub start_line: usize,
    /// End line (will be populated when parser tracks positions)
    pub end_line: usize,
    /// Start byte offset in query
    pub start_offset: usize,
    /// End byte offset in query
    pub end_offset: usize,
    /// Whether this CTE contains SELECT *
    pub has_star: bool,
    /// Columns produced by this CTE (if known)
    pub columns: Vec<String>,
    /// WEB CTE configuration (if applicable)
    pub web_config: Option<WebCteConfig>,
}

/// WEB CTE configuration details
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebCteConfig {
    /// URL endpoint
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Headers (if any)
    pub headers: Vec<(String, String)>,
    /// Format (CSV, JSON, etc.)
    pub format: Option<String>,
}

/// FROM clause analysis
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FromClauseInfo {
    /// Type: "table", "subquery", "function", "cte"
    pub source_type: String,
    /// Table/CTE name (if applicable)
    pub name: Option<String>,
}

/// WHERE clause analysis
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WhereClauseInfo {
    /// Whether WHERE clause is present
    pub present: bool,
    /// Columns referenced in WHERE
    pub columns_referenced: Vec<String>,
}

/// Column expansion result
#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnExpansion {
    /// Original query with SELECT *
    pub original_query: String,
    /// Expanded query with actual column names
    pub expanded_query: String,
    /// Column information
    pub columns: Vec<ColumnInfo>,
    /// Number of columns expanded
    pub expansion_count: usize,
    /// Columns from CTEs (cte_name -> columns)
    pub cte_columns: HashMap<String, Vec<String>>,
}

/// Column information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: String,
}

/// Query context at a specific position
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryContext {
    /// Type: "main_query", "CTE", "subquery"
    pub context_type: String,
    /// CTE name (if in CTE)
    pub cte_name: Option<String>,
    /// CTE index (0-based)
    pub cte_index: Option<usize>,
    /// Query bounds
    pub query_bounds: QueryBounds,
    /// Parent query bounds (if in CTE/subquery)
    pub parent_query_bounds: Option<QueryBounds>,
    /// Whether this can be executed independently
    pub can_execute_independently: bool,
}

/// Query boundary information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryBounds {
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// Start byte offset
    pub start_offset: usize,
    /// End byte offset
    pub end_offset: usize,
}

/// Analyze a SQL query and return structured information
pub fn analyze_query(ast: &SelectStatement, _sql: &str) -> QueryAnalysis {
    let mut analysis = QueryAnalysis {
        valid: true,
        query_type: "SELECT".to_string(),
        has_star: false,
        star_locations: vec![],
        tables: vec![],
        columns: vec![],
        ctes: vec![],
        from_clause: None,
        where_clause: None,
        errors: vec![],
    };

    // Analyze CTEs
    for cte in &ast.ctes {
        analysis.ctes.push(analyze_cte(cte));
    }

    // Check for SELECT * in main query
    for item in &ast.select_items {
        if matches!(item, SelectItem::Star) {
            analysis.has_star = true;
            analysis.star_locations.push(StarLocation {
                line: 1, // TODO: Track actual line when parser supports it
                column: 8,
                context: "main_query".to_string(),
            });
        }
    }

    // Extract table references
    if let Some(ref table) = ast.from_table {
        let table_name: String = table.clone();
        analysis.tables.push(table_name.clone());
        analysis.from_clause = Some(FromClauseInfo {
            source_type: "table".to_string(),
            name: Some(table_name),
        });
    } else if ast.from_subquery.is_some() {
        analysis.from_clause = Some(FromClauseInfo {
            source_type: "subquery".to_string(),
            name: None,
        });
    }

    // Analyze WHERE clause
    if let Some(ref where_clause) = ast.where_clause {
        let mut columns = vec![];
        // TODO: Extract column references from WHERE conditions
        for condition in &where_clause.conditions {
            // Extract column names from condition.expr
            // This is simplified - full implementation would walk the expression tree
            if let Some(col) = extract_column_from_expr(&condition.expr) {
                if !columns.contains(&col) {
                    columns.push(col);
                }
            }
        }

        analysis.where_clause = Some(WhereClauseInfo {
            present: true,
            columns_referenced: columns,
        });
    }

    // Extract explicitly named columns from SELECT
    for item in &ast.select_items {
        if let SelectItem::Column(col_ref) = item {
            if !analysis.columns.contains(&col_ref.name) {
                analysis.columns.push(col_ref.name.clone());
            }
        }
    }

    analysis
}

fn analyze_cte(cte: &CTE) -> CteAnalysis {
    let cte_type_str = match &cte.cte_type {
        CTEType::Standard(_) => "Standard",
        CTEType::Web(_) => "WEB",
    };

    let mut has_star = false;
    let mut web_config = None;

    match &cte.cte_type {
        CTEType::Standard(stmt) => {
            // Check if CTE query has SELECT *
            for item in &stmt.select_items {
                if matches!(item, SelectItem::Star) {
                    has_star = true;
                    break;
                }
            }
        }
        CTEType::Web(web_spec) => {
            let method_str = match &web_spec.method {
                Some(m) => format!("{:?}", m),
                None => "GET".to_string(),
            };
            web_config = Some(WebCteConfig {
                url: web_spec.url.clone(),
                method: method_str,
                headers: web_spec.headers.clone(),
                format: web_spec.format.as_ref().map(|f| format!("{:?}", f)),
            });
        }
    }

    CteAnalysis {
        name: cte.name.clone(),
        cte_type: cte_type_str.to_string(),
        start_line: 1, // TODO: Track when parser supports it
        end_line: 1,   // TODO: Track when parser supports it
        start_offset: 0,
        end_offset: 0,
        has_star,
        columns: vec![], // TODO: Extract column names
        web_config,
    }
}

fn extract_column_from_expr(expr: &crate::sql::parser::ast::SqlExpression) -> Option<String> {
    use crate::sql::parser::ast::SqlExpression;

    match expr {
        SqlExpression::Column(col_ref) => Some(col_ref.name.clone()),
        SqlExpression::BinaryOp { left, right, .. } => {
            // Try left first, then right
            extract_column_from_expr(left).or_else(|| extract_column_from_expr(right))
        }
        SqlExpression::FunctionCall { args, .. } => {
            // Extract from first argument
            args.first().and_then(|arg| extract_column_from_expr(arg))
        }
        _ => None,
    }
}

/// Extract a specific CTE as a standalone query
pub fn extract_cte(ast: &SelectStatement, cte_name: &str) -> Option<String> {
    for cte in &ast.ctes {
        if cte.name == cte_name {
            return Some(format_cte_as_query(cte));
        }
    }
    None
}

fn format_cte_as_query(cte: &CTE) -> String {
    match &cte.cte_type {
        CTEType::Standard(stmt) => {
            // Format the SELECT statement
            // This is simplified - could use the AST formatter
            format_select_statement(stmt)
        }
        CTEType::Web(web_spec) => {
            // Can't execute WEB CTE independently (needs WITH WEB syntax)
            let mut parts = vec![
                format!("WITH WEB {} AS (", cte.name),
                format!("  URL '{}'", web_spec.url),
            ];

            if let Some(ref m) = web_spec.method {
                parts.push(format!("  METHOD {:?}", m));
            }

            if !web_spec.headers.is_empty() {
                parts.push("  HEADERS (".to_string());
                for (k, v) in &web_spec.headers {
                    parts.push(format!("    '{}' = '{}'", k, v));
                }
                parts.push("  )".to_string());
            }

            if let Some(ref b) = web_spec.body {
                parts.push(format!("  BODY '{}'", b));
            }

            if let Some(ref f) = web_spec.format {
                parts.push(format!("  FORMAT {:?}", f));
            }

            parts.push(")".to_string());
            parts.push(format!("SELECT * FROM {}", cte.name));

            parts.join("\n")
        }
    }
}

fn format_select_statement(stmt: &SelectStatement) -> String {
    let mut parts = vec!["SELECT".to_string()];

    // SELECT items
    if stmt.select_items.is_empty() {
        parts.push("  *".to_string());
    } else {
        for (i, item) in stmt.select_items.iter().enumerate() {
            let prefix = if i == 0 { "    " } else { "  , " };
            match item {
                SelectItem::Star => parts.push(format!("{}*", prefix)),
                SelectItem::Column(col) => {
                    parts.push(format!("{}{}", prefix, col.name));
                }
                SelectItem::Expression { expr, alias } => {
                    let expr_str = format_expr(expr);
                    parts.push(format!("{}{} AS {}", prefix, expr_str, alias));
                }
            }
        }
    }

    // FROM
    if let Some(ref table) = stmt.from_table {
        parts.push(format!("FROM {}", table));
    }

    // WHERE
    if let Some(ref where_clause) = stmt.where_clause {
        parts.push("WHERE".to_string());
        for (i, condition) in where_clause.conditions.iter().enumerate() {
            let connector = if i > 0 {
                condition
                    .connector
                    .as_ref()
                    .map(|op| match op {
                        crate::sql::parser::ast::LogicalOp::And => "AND",
                        crate::sql::parser::ast::LogicalOp::Or => "OR",
                    })
                    .unwrap_or("AND")
            } else {
                ""
            };
            let expr_str = format_expr(&condition.expr);
            if i == 0 {
                parts.push(format!("  {}", expr_str));
            } else {
                parts.push(format!("  {} {}", connector, expr_str));
            }
        }
    }

    // LIMIT
    if let Some(limit) = stmt.limit {
        parts.push(format!("LIMIT {}", limit));
    }

    parts.join("\n")
}

fn format_expr(expr: &crate::sql::parser::ast::SqlExpression) -> String {
    use crate::sql::parser::ast::SqlExpression;

    match expr {
        SqlExpression::Column(col) => col.name.clone(),
        SqlExpression::NumberLiteral(n) => n.clone(),
        SqlExpression::StringLiteral(s) => format!("'{}'", s),
        SqlExpression::BooleanLiteral(b) => b.to_string(),
        SqlExpression::Null => "NULL".to_string(),
        SqlExpression::BinaryOp { left, op, right } => {
            format!("{} {} {}", format_expr(left), op, format_expr(right))
        }
        SqlExpression::FunctionCall { name, args, .. } => {
            let arg_strs: Vec<String> = args.iter().map(|a| format_expr(a)).collect();
            format!("{}({})", name, arg_strs.join(", "))
        }
        SqlExpression::Not { expr } => {
            format!("NOT {}", format_expr(expr))
        }
        SqlExpression::CaseExpression { .. } => "CASE...END".to_string(), // Simplified
        SqlExpression::SimpleCaseExpression { .. } => "CASE...END".to_string(), // Simplified
        SqlExpression::ScalarSubquery { .. } => "(SELECT ...)".to_string(), // Simplified
        SqlExpression::InSubquery { .. } => "IN (SELECT ...)".to_string(), // Simplified
        SqlExpression::NotInSubquery { .. } => "NOT IN (SELECT ...)".to_string(), // Simplified
        SqlExpression::InList { .. } => "IN (...)".to_string(),           // Simplified
        SqlExpression::NotInList { .. } => "NOT IN (...)".to_string(),    // Simplified
        SqlExpression::Between { .. } => "BETWEEN...AND...".to_string(),  // Simplified
        _ => "...".to_string(), // Simplified for other cases
    }
}

/// Find query context at a specific line:column position
pub fn find_query_context(ast: &SelectStatement, line: usize, _column: usize) -> QueryContext {
    // Check if position is within a CTE
    for (idx, cte) in ast.ctes.iter().enumerate() {
        // TODO: Use actual line numbers when parser tracks them
        // For now, assume each CTE is ~5 lines
        let cte_start = 1 + (idx * 5);
        let cte_end = cte_start + 4;

        if line >= cte_start && line <= cte_end {
            return QueryContext {
                context_type: "CTE".to_string(),
                cte_name: Some(cte.name.clone()),
                cte_index: Some(idx),
                query_bounds: QueryBounds {
                    start_line: cte_start,
                    end_line: cte_end,
                    start_offset: 0,
                    end_offset: 0,
                },
                parent_query_bounds: Some(QueryBounds {
                    start_line: 1,
                    end_line: 100, // TODO: Track actual end
                    start_offset: 0,
                    end_offset: 0,
                }),
                can_execute_independently: !matches!(cte.cte_type, CTEType::Web(_)),
            };
        }
    }

    // Otherwise, in main query
    QueryContext {
        context_type: "main_query".to_string(),
        cte_name: None,
        cte_index: None,
        query_bounds: QueryBounds {
            start_line: 1,
            end_line: 100, // TODO: Track actual end
            start_offset: 0,
            end_offset: 0,
        },
        parent_query_bounds: None,
        can_execute_independently: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::recursive_parser::Parser;

    #[test]
    fn test_analyze_simple_query() {
        let sql = "SELECT * FROM trades WHERE price > 100";
        let mut parser = Parser::new(sql);
        let ast = parser.parse().unwrap();

        let analysis = analyze_query(&ast, sql);

        assert!(analysis.valid);
        assert_eq!(analysis.query_type, "SELECT");
        assert!(analysis.has_star);
        assert_eq!(analysis.star_locations.len(), 1);
        assert_eq!(analysis.tables, vec!["trades"]);
    }

    #[test]
    fn test_analyze_cte_query() {
        let sql = "WITH trades AS (SELECT * FROM raw_trades) SELECT symbol FROM trades";
        let mut parser = Parser::new(sql);
        let ast = parser.parse().unwrap();

        let analysis = analyze_query(&ast, sql);

        assert!(analysis.valid);
        assert_eq!(analysis.ctes.len(), 1);
        assert_eq!(analysis.ctes[0].name, "trades");
        assert_eq!(analysis.ctes[0].cte_type, "Standard");
        assert!(analysis.ctes[0].has_star);
    }

    #[test]
    fn test_extract_cte() {
        let sql =
            "WITH trades AS (SELECT * FROM raw_trades WHERE price > 100) SELECT * FROM trades";
        let mut parser = Parser::new(sql);
        let ast = parser.parse().unwrap();

        let extracted = extract_cte(&ast, "trades").unwrap();

        assert!(extracted.contains("SELECT"));
        assert!(extracted.contains("raw_trades"));
        assert!(extracted.contains("price > 100"));
    }
}
