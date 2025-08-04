use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use crate::parser::Schema;

extern "C" {
    fn tree_sitter_sql() -> Language;
}

pub struct TreeSitterSqlParser {
    parser: Parser,
    schema: Schema,
}

#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub suggestions: Vec<String>,
    pub context: String,
}

impl TreeSitterSqlParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_sql() };
        parser.set_language(&language)?;
        
        Ok(Self {
            parser,
            schema: Schema::new(),
        })
    }
    
    pub fn get_completions(&mut self, query: &str, cursor_pos: usize) -> CompletionResult {
        let tree = self.parser.parse(query, None).unwrap();
        
        // Find node at cursor position
        let cursor_point = self.byte_to_point(query, cursor_pos);
        let node_at_cursor = tree.root_node().descendant_for_point_range(cursor_point, cursor_point);
        
        if let Some(node) = node_at_cursor {
            let context = self.determine_completion_context(&tree, node, query, cursor_pos);
            let suggestions = self.get_suggestions_for_context(&context, query, cursor_pos);
            
            CompletionResult {
                suggestions,
                context: context.clone(),
            }
        } else {
            CompletionResult {
                suggestions: vec!["SELECT".to_string()],
                context: "start".to_string(),
            }
        }
    }
    
    fn byte_to_point(&self, text: &str, byte_pos: usize) -> tree_sitter::Point {
        let mut row = 0;
        let mut column = 0;
        
        for (i, ch) in text.char_indices() {
            if i >= byte_pos {
                break;
            }
            if ch == '\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        
        tree_sitter::Point { row, column }
    }
    
    fn determine_completion_context(&self, tree: &Tree, node: tree_sitter::Node, query: &str, cursor_pos: usize) -> String {
        // Walk up the tree to find SQL context
        let mut current = node;
        
        loop {
            let kind = current.kind();
            
            match kind {
                "select_statement" => {
                    return self.analyze_select_statement(current, query, cursor_pos);
                }
                "column" | "column_list" => {
                    return "column".to_string();
                }
                "from_clause" => {
                    return "table".to_string();
                }
                "where_clause" => {
                    return "where".to_string();
                }
                "order_by_clause" => {
                    return "order_by".to_string();
                }
                _ => {}
            }
            
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }
        
        "unknown".to_string()
    }
    
    fn analyze_select_statement(&self, node: tree_sitter::Node, query: &str, cursor_pos: usize) -> String {
        // Use tree-sitter queries to find specific contexts
        let query_str = r#"
            (select_statement
              (select_clause) @select
              (from_clause)? @from
              (where_clause)? @where
              (order_by_clause)? @order)
        "#;
        
        let language = unsafe { tree_sitter_sql() };
        if let Ok(ts_query) = Query::new(&language, query_str) {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&ts_query, node, query.as_bytes());
            
            for m in matches {
                for capture in m.captures {
                    let start_byte = capture.node.start_byte();
                    let end_byte = capture.node.end_byte();
                    
                    if cursor_pos >= start_byte && cursor_pos <= end_byte {
                        match ts_query.capture_names()[capture.index as usize] {
                            "select" => return "column_list".to_string(),
                            "from" => return "table".to_string(),
                            "where" => return "where_expression".to_string(),
                            "order" => return "order_by".to_string(),
                            _ => {}
                        }
                    }
                }
            }
        }
        
        "select_statement".to_string()
    }
    
    fn get_suggestions_for_context(&self, context: &str, query: &str, cursor_pos: usize) -> Vec<String> {
        let partial_word = self.extract_partial_word(query, cursor_pos);
        
        let suggestions = match context {
            "column_list" | "column" => {
                let mut cols = self.schema.get_columns("trade_deal");
                cols.push("*".to_string());
                cols.push("FROM".to_string());
                cols
            }
            "table" => {
                vec!["trade_deal".to_string(), "instrument".to_string()]
            }
            "where_expression" | "where" => {
                let mut suggestions = self.schema.get_columns("trade_deal");
                suggestions.extend(vec![
                    "AND".to_string(),
                    "OR".to_string(),
                    "ORDER BY".to_string(),
                ]);
                suggestions
            }
            "order_by" => {
                let mut suggestions = self.schema.get_columns("trade_deal");
                suggestions.extend(vec!["ASC".to_string(), "DESC".to_string()]);
                suggestions
            }
            "start" => {
                vec!["SELECT".to_string()]
            }
            _ => {
                vec!["SELECT".to_string()]
            }
        };
        
        // Filter by partial word
        if let Some(partial) = partial_word {
            suggestions
                .into_iter()
                .filter(|s| s.to_lowercase().starts_with(&partial.to_lowercase()))
                .collect()
        } else {
            suggestions
        }
    }
    
    fn extract_partial_word(&self, query: &str, cursor_pos: usize) -> Option<String> {
        if cursor_pos == 0 || cursor_pos > query.len() {
            return None;
        }
        
        let chars: Vec<char> = query.chars().collect();
        
        // Find start of current word
        let mut start = cursor_pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        if start < cursor_pos {
            let partial: String = chars[start..cursor_pos].iter().collect();
            if !partial.is_empty() && partial.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Some(partial)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn validate_query(&mut self, query: &str) -> bool {
        if let Some(tree) = self.parser.parse(query, None) {
            !tree.root_node().has_error()
        } else {
            false
        }
    }
    
    pub fn get_syntax_errors(&mut self, query: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if let Some(tree) = self.parser.parse(query, None) {
            self.collect_errors(tree.root_node(), &mut errors);
        }
        
        errors
    }
    
    fn collect_errors(&self, node: tree_sitter::Node, errors: &mut Vec<String>) {
        if node.is_error() {
            errors.push(format!("Syntax error at position {}", node.start_byte()));
        }
        
        for child in node.children(&mut node.walk()) {
            self.collect_errors(child, errors);
        }
    }
}