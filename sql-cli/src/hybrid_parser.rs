use crate::cursor_aware_parser::CursorAwareParser;
use crate::recursive_parser::{detect_cursor_context, tokenize_query, CursorContext, LogicalOp};

#[derive(Clone)]
pub struct HybridParser {
    parser: CursorAwareParser,
}

impl std::fmt::Debug for HybridParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridParser")
            .field("parser", &"<CursorAwareParser>")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct HybridResult {
    pub suggestions: Vec<String>,
    pub context: String,
    pub parser_used: String,
    pub recursive_context: String,
    pub cursor_position: usize,
    pub query_complexity: String,
}

impl HybridParser {
    pub fn new() -> Self {
        Self {
            parser: CursorAwareParser::new(),
        }
    }

    pub fn update_single_table(&mut self, table_name: String, columns: Vec<String>) {
        self.parser.update_single_table(table_name, columns);
    }

    pub fn get_completions(&self, query: &str, cursor_pos: usize) -> HybridResult {
        // Use the improved parser with recursive descent for context detection
        let result = self.parser.get_completions(query, cursor_pos);

        // Get recursive parser context for debugging
        let (cursor_context, _) = detect_cursor_context(query, cursor_pos);
        let recursive_context = match cursor_context {
            CursorContext::SelectClause => "SelectClause",
            CursorContext::FromClause => "FromClause",
            CursorContext::WhereClause => "WhereClause",
            CursorContext::OrderByClause => "OrderByClause",
            CursorContext::AfterColumn(_) => "AfterColumn",
            CursorContext::AfterLogicalOp(LogicalOp::And) => "AfterAND",
            CursorContext::AfterLogicalOp(LogicalOp::Or) => "AfterOR",
            CursorContext::AfterComparisonOp(_, _) => "AfterComparisonOp",
            CursorContext::InMethodCall(_, _) => "InMethodCall",
            CursorContext::InExpression => "InExpression",
            CursorContext::Unknown => "Unknown",
        };

        HybridResult {
            suggestions: result.suggestions,
            context: result.context.clone(),
            parser_used: "RecursiveDescent".to_string(),
            recursive_context: recursive_context.to_string(),
            cursor_position: cursor_pos,
            query_complexity: self.analyze_query_complexity(query),
        }
    }

    fn analyze_query_complexity(&self, query: &str) -> String {
        let mut complexity_factors = Vec::new();

        // Count logical operators
        let logical_ops = query.to_uppercase().matches(" AND ").count()
            + query.to_uppercase().matches(" OR ").count();
        if logical_ops > 0 {
            complexity_factors.push(format!("{}x logical", logical_ops));
        }

        // Count method calls
        let method_calls = query.matches('.').count();
        if method_calls > 0 {
            complexity_factors.push(format!("{}x methods", method_calls));
        }

        // Count parentheses depth
        let paren_depth = self.max_paren_depth(query);
        if paren_depth > 1 {
            complexity_factors.push(format!("{}lvl nested", paren_depth));
        }

        // Count subqueries (simplified)
        if query.to_uppercase().contains("SELECT")
            && query.to_uppercase().matches("SELECT").count() > 1
        {
            complexity_factors.push("subquery".to_string());
        }

        if complexity_factors.is_empty() {
            "simple".to_string()
        } else {
            complexity_factors.join(", ")
        }
    }

    fn max_paren_depth(&self, query: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth: usize = 0;

        for ch in query.chars() {
            match ch {
                '(' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                ')' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        max_depth
    }

    pub fn debug_tree(&self, query: &str) -> String {
        // We now use recursive descent parser, which doesn't have a tree visualization yet
        "Recursive descent parser - AST visualization not implemented".to_string()
    }

    pub fn get_detailed_debug_info(&self, query: &str, cursor_pos: usize) -> String {
        let result = self.get_completions(query, cursor_pos);

        let char_at_cursor = if cursor_pos < query.len() {
            format!("'{}'", query.chars().nth(cursor_pos).unwrap_or(' '))
        } else {
            "EOF".to_string()
        };

        // Get tokenized output
        let tokens = tokenize_query(query);
        let tokenized_output = if tokens.is_empty() {
            "  (no tokens)".to_string()
        } else {
            tokens
                .iter()
                .enumerate()
                .map(|(i, t)| format!("  [{}] {}", i, t))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let ast_tree = self.debug_tree(query);

        // Extract partial word from context string
        let partial_word_info = if result.context.contains("(partial:") {
            // Extract the partial word from the context string
            if let Some(start) = result.context.find("(partial: ") {
                let substr = &result.context[start + 10..];
                if let Some(end) = substr.find(')') {
                    substr[..end].to_string()
                } else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            }
        } else {
            "None".to_string()
        };

        format!(
            "========== PARSER DEBUG ==========\n\
Query: '{}'\n\
Query Length: {}\n\
Cursor: {} {}\n\
Partial Word: {}\n\
Complexity: {}\n\
Parser Used: {}\n\
Parser Type: Recursive Descent\n\
\n\
TOKENIZED OUTPUT:\n{}\n\
\n\
CONTEXT: {}\n\
RECURSIVE PARSER CONTEXT: {}\n\
\n\
SUGGESTIONS ({}):\n{}\n\
\n\
AST TREE:\n{}\n\
==================================",
            query,
            query.len(),
            cursor_pos,
            char_at_cursor,
            partial_word_info,
            result.query_complexity,
            result.parser_used,
            tokenized_output,
            result.context,
            result.recursive_context,
            result.suggestions.len(),
            if result.suggestions.is_empty() {
                "  (no suggestions)".to_string()
            } else {
                result
                    .suggestions
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("  {}: {}", i + 1, s))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            ast_tree
        )
    }
}
