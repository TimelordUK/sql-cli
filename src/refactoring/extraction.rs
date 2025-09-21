// CTE Extraction and Hoisting Tools
// Analyzes SQL and suggests/performs CTE extractions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionSuggestion {
    pub expression: String,
    pub reason: ExtractionReason,
    pub suggested_cte_name: String,
    pub cte_query: String,
    pub replacement: String,
    pub complexity_score: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExtractionReason {
    ComplexCalculation,
    RepeatedExpression,
    WindowFunction,
    Subquery,
    StringManipulation,
    CaseStatement,
    AggregateInWhere,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CTEChain {
    pub ctes: Vec<CTEDefinition>,
    pub main_query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CTEDefinition {
    pub name: String,
    pub query: String,
    pub dependencies: Vec<String>,
    pub columns: Vec<String>,
}

/// Analyzes a query for potential CTE extractions
pub struct ExtractionAnalyzer;

impl ExtractionAnalyzer {
    /// Analyze a SQL query for extraction opportunities
    pub fn analyze(sql: &str) -> Vec<ExtractionSuggestion> {
        let mut suggestions = Vec::new();

        // Pattern 1: Complex calculations (multiplication, division, functions)
        if sql.contains(" * ") || sql.contains(" / ") {
            if let Some(expr) = Self::find_complex_calculation(sql) {
                suggestions.push(ExtractionSuggestion {
                    expression: expr.clone(),
                    reason: ExtractionReason::ComplexCalculation,
                    suggested_cte_name: "calculated".to_string(),
                    cte_query: Self::generate_cte_for_calculation(&expr),
                    replacement: "calculated_value".to_string(),
                    complexity_score: 10,
                });
            }
        }

        // Pattern 2: CASE statements in WHERE or complex CASE
        if sql.to_uppercase().contains("CASE WHEN") {
            if let Some(case_expr) = Self::find_case_statement(sql) {
                suggestions.push(ExtractionSuggestion {
                    expression: case_expr.clone(),
                    reason: ExtractionReason::CaseStatement,
                    suggested_cte_name: "categorized".to_string(),
                    cte_query: Self::generate_cte_for_case(&case_expr),
                    replacement: "category".to_string(),
                    complexity_score: 15,
                });
            }
        }

        // Pattern 3: String manipulation (SUBSTRING, CONTAINS, etc.)
        if sql.contains("SUBSTRING") || sql.contains("CONTAINS") {
            if let Some(str_expr) = Self::find_string_manipulation(sql) {
                suggestions.push(ExtractionSuggestion {
                    expression: str_expr.clone(),
                    reason: ExtractionReason::StringManipulation,
                    suggested_cte_name: "parsed".to_string(),
                    cte_query: Self::generate_cte_for_string(&str_expr),
                    replacement: "parsed_value".to_string(),
                    complexity_score: 12,
                });
            }
        }

        // Pattern 4: Window functions that could be pre-computed
        if sql.contains("OVER (") {
            if let Some(window_expr) = Self::find_window_function(sql) {
                suggestions.push(ExtractionSuggestion {
                    expression: window_expr.clone(),
                    reason: ExtractionReason::WindowFunction,
                    suggested_cte_name: "windowed".to_string(),
                    cte_query: Self::generate_cte_for_window(&window_expr),
                    replacement: "window_result".to_string(),
                    complexity_score: 20,
                });
            }
        }

        // Sort by complexity score (higher = more beneficial to extract)
        suggestions.sort_by_key(|s| std::cmp::Reverse(s.complexity_score));

        suggestions
    }

    fn find_complex_calculation(sql: &str) -> Option<String> {
        // Simplified pattern matching - in real implementation would use parser
        if sql.contains("price * quantity") {
            return Some("price * quantity".to_string());
        }
        if sql.contains("amount * rate") {
            return Some("amount * rate".to_string());
        }
        None
    }

    fn find_case_statement(sql: &str) -> Option<String> {
        // Find CASE...END blocks
        let upper = sql.to_uppercase();
        if let Some(start) = upper.find("CASE") {
            if let Some(end) = upper[start..].find("END") {
                return Some(sql[start..start + end + 3].to_string());
            }
        }
        None
    }

    fn find_string_manipulation(sql: &str) -> Option<String> {
        // Find string functions
        if sql.contains("SUBSTRING_AFTER") {
            // Extract the full function call
            if let Some(start) = sql.find("SUBSTRING_AFTER") {
                if let Some(end) = Self::find_matching_paren(&sql[start..]) {
                    return Some(sql[start..start + end + 1].to_string());
                }
            }
        }
        None
    }

    fn find_window_function(sql: &str) -> Option<String> {
        // Find window function expressions
        if let Some(start) = sql.find("ROW_NUMBER()") {
            if let Some(over_pos) = sql[start..].find("OVER") {
                if let Some(end) = Self::find_matching_paren(&sql[start + over_pos + 4..]) {
                    return Some(sql[start..start + over_pos + 5 + end].to_string());
                }
            }
        }
        None
    }

    fn find_matching_paren(s: &str) -> Option<usize> {
        let mut depth = 0;
        let mut in_paren = false;

        for (i, ch) in s.char_indices() {
            match ch {
                '(' => {
                    depth += 1;
                    in_paren = true;
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 && in_paren {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn generate_cte_for_calculation(expr: &str) -> String {
        format!("SELECT *, {} as calculated_value FROM source_table", expr)
    }

    fn generate_cte_for_case(expr: &str) -> String {
        format!("SELECT *, {} as category FROM source_table", expr)
    }

    fn generate_cte_for_string(expr: &str) -> String {
        format!("SELECT *, {} as parsed_value FROM source_table", expr)
    }

    fn generate_cte_for_window(expr: &str) -> String {
        format!("SELECT *, {} as window_result FROM source_table", expr)
    }
}

/// Optimizes CTE chains by analyzing dependencies and suggesting combinations
pub struct CTEOptimizer;

impl CTEOptimizer {
    /// Analyze a CTE chain and suggest optimizations
    pub fn optimize_chain(chain: &CTEChain) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for CTEs that could be combined
        for i in 0..chain.ctes.len() {
            for j in i + 1..chain.ctes.len() {
                if Self::can_combine(&chain.ctes[i], &chain.ctes[j]) {
                    suggestions.push(format!(
                        "CTEs '{}' and '{}' could be combined to reduce complexity",
                        chain.ctes[i].name, chain.ctes[j].name
                    ));
                }
            }
        }

        // Check for unused CTEs
        let used_ctes = Self::find_used_ctes(&chain.main_query, &chain.ctes);
        for cte in &chain.ctes {
            if !used_ctes.contains(&cte.name) {
                suggestions.push(format!("CTE '{}' appears to be unused", cte.name));
            }
        }

        // Check for linear chains that could be flattened
        if Self::is_linear_chain(&chain.ctes) {
            suggestions.push("This linear CTE chain could potentially be flattened".to_string());
        }

        suggestions
    }

    fn can_combine(cte1: &CTEDefinition, cte2: &CTEDefinition) -> bool {
        // Simple heuristic: if one depends on the other and doesn't add much complexity
        cte1.dependencies.contains(&cte2.name) || cte2.dependencies.contains(&cte1.name)
    }

    fn find_used_ctes(query: &str, ctes: &[CTEDefinition]) -> HashSet<String> {
        let mut used = HashSet::new();
        for cte in ctes {
            if query.contains(&cte.name) {
                used.insert(cte.name.clone());
            }
        }
        used
    }

    fn is_linear_chain(ctes: &[CTEDefinition]) -> bool {
        // Check if each CTE depends only on the previous one
        for i in 1..ctes.len() {
            if ctes[i].dependencies.len() != 1 {
                return false;
            }
            if !ctes[i].dependencies.contains(&ctes[i - 1].name) {
                return false;
            }
        }
        true
    }
}

/// Generates SQL transformation suggestions
pub fn suggest_extraction(sql: &str) -> Result<serde_json::Value> {
    let suggestions = ExtractionAnalyzer::analyze(sql);

    Ok(serde_json::json!({
        "original": sql,
        "suggestions": suggestions,
        "recommendation": if !suggestions.is_empty() {
            format!("Consider extracting {} expressions to CTEs", suggestions.len())
        } else {
            "No extraction opportunities found".to_string()
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_detection() {
        let sql = "SELECT * FROM orders WHERE price * quantity > 1000";
        let suggestions = ExtractionAnalyzer::analyze(sql);

        assert!(!suggestions.is_empty());
        assert_eq!(
            suggestions[0].reason as u32,
            ExtractionReason::ComplexCalculation as u32
        );
    }

    #[test]
    fn test_case_extraction() {
        let sql = "SELECT CASE WHEN age <= 20 THEN 'young' ELSE 'old' END FROM users";
        let suggestions = ExtractionAnalyzer::analyze(sql);

        assert!(suggestions
            .iter()
            .any(|s| matches!(s.reason, ExtractionReason::CaseStatement)));
    }
}
