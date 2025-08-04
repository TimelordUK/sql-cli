use tree_sitter::{Language, Parser, Query, QueryCursor, Tree, Node};
use tree_sitter_sql;

pub struct TreeSitterSqlParser {
    parser: Parser,
    language: Language,
}

impl Clone for TreeSitterSqlParser {
    fn clone(&self) -> Self {
        Self::new().expect("Failed to clone TreeSitterSqlParser")
    }
}

#[derive(Debug, Clone)]
pub struct SqlContext {
    pub node_type: String,
    pub parent_type: Option<String>,
    pub grandparent_type: Option<String>,
    pub is_in_where_clause: bool,
    pub is_after_logical_operator: bool,
    pub is_in_method_call: bool,
    pub current_column: Option<String>,
    pub available_columns: Vec<String>,
}

impl TreeSitterSqlParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let language = tree_sitter_sql::language();
        let mut parser = Parser::new();
        parser.set_language(language)?;
        
        Ok(Self {
            parser,
            language,
        })
    }
    
    pub fn parse_sql(&mut self, sql: &str) -> Result<Tree, Box<dyn std::error::Error>> {
        let tree = self.parser.parse(sql, None)
            .ok_or("Failed to parse SQL")?;
        Ok(tree)
    }
    
    pub fn get_context_at_cursor(&mut self, sql: &str, cursor_pos: usize) -> Result<SqlContext, Box<dyn std::error::Error>> {
        let tree = self.parse_sql(sql)?;
        let root_node = tree.root_node();
        
        // Find the node at cursor position
        let cursor_node = self.find_node_at_position(&root_node, cursor_pos);
        
        // Analyze the context
        let context = self.analyze_context(&cursor_node, sql);
        
        Ok(context)
    }
    
    fn find_node_at_position<'a>(&self, node: &Node<'a>, position: usize) -> Node<'a> {
        // Convert byte position to point (row, col)
        let current = *node;
        
        // Walk down the tree to find the most specific node at the cursor position
        for child in current.children(&mut current.walk()) {
            if child.start_byte() <= position && position <= child.end_byte() {
                return self.find_node_at_position(&child, position);
            }
        }
        
        current
    }
    
    fn analyze_context(&self, node: &Node, sql: &str) -> SqlContext {
        let node_type = node.kind().to_string();
        let parent = node.parent();
        let parent_type = parent.as_ref().map(|p| p.kind().to_string());
        let grandparent_type = parent.as_ref()
            .and_then(|p| p.parent())
            .map(|gp| gp.kind().to_string());
        
        // Determine context based on node hierarchy
        let is_in_where_clause = self.is_ancestor_of_type(node, "where_clause");
        let is_after_logical_operator = self.is_after_logical_op(node, sql);
        let is_in_method_call = node_type == "function_call" || 
                               parent_type.as_ref().map_or(false, |t| t == "function_call");
        
        // Extract current column if we're in a column context
        let current_column = self.extract_current_column(node, sql);
        
        // Get available columns (this would come from schema)
        let available_columns = self.get_available_columns();
        
        SqlContext {
            node_type,
            parent_type,
            grandparent_type,
            is_in_where_clause,
            is_after_logical_operator,
            is_in_method_call,
            current_column,
            available_columns,
        }
    }
    
    fn is_ancestor_of_type(&self, node: &Node, ancestor_type: &str) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == ancestor_type {
                return true;
            }
            current = parent.parent();
        }
        false
    }
    
    fn is_after_logical_op(&self, node: &Node, sql: &str) -> bool {
        // Look for AND/OR in the preceding text
        if let Some(parent) = node.parent() {
            let start_byte = parent.start_byte();
            let node_start = node.start_byte();
            
            if start_byte < node_start {
                let preceding_text = &sql[start_byte..node_start];
                let upper_text = preceding_text.to_uppercase();
                return upper_text.trim().ends_with(" AND") || 
                       upper_text.trim().ends_with(" OR");
            }
        }
        false
    }
    
    fn extract_current_column(&self, node: &Node, sql: &str) -> Option<String> {
        // Extract column name if we're in a column context
        if node.kind() == "identifier" || node.kind() == "column_reference" {
            let text = &sql[node.start_byte()..node.end_byte()];
            Some(text.to_string())
        } else {
            None
        }
    }
    
    fn get_available_columns(&self) -> Vec<String> {
        // This would integrate with your existing schema
        vec![
            "dealId".to_string(),
            "platformOrderId".to_string(),
            "allocationStatus".to_string(),
            "counterparty".to_string(),
            "price".to_string(),
            "quantity".to_string(),
            // ... etc
        ]
    }
    
    pub fn get_completion_suggestions(&mut self, sql: &str, cursor_pos: usize) -> Vec<String> {
        match self.get_context_at_cursor(sql, cursor_pos) {
            Ok(context) => self.suggestions_for_context(&context),
            Err(_) => {
                // Fallback to simple suggestions if parsing fails
                vec!["SELECT".to_string(), "FROM".to_string(), "WHERE".to_string()]
            }
        }
    }
    
    fn suggestions_for_context(&self, context: &SqlContext) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match context.node_type.as_str() {
            "identifier" | "column_reference" if context.is_in_where_clause => {
                // We're typing a column name in WHERE clause
                suggestions.extend(context.available_columns.clone());
            }
            "function_call" if context.is_in_method_call => {
                // We're in a method call - suggest string methods
                suggestions.extend(vec![
                    "Contains(\"\")".to_string(),
                    "StartsWith(\"\")".to_string(),
                    "EndsWith(\"\")".to_string(),
                ]);
            }
            _ if context.is_after_logical_operator => {
                // After AND/OR - suggest columns
                suggestions.extend(context.available_columns.clone());
            }
            _ if context.is_in_where_clause => {
                // General WHERE clause suggestions
                suggestions.extend(context.available_columns.clone());
                suggestions.extend(vec![
                    "AND".to_string(),
                    "OR".to_string(),
                    "ORDER BY".to_string(),
                ]);
            }
            _ => {
                // Default suggestions
                suggestions.extend(vec![
                    "SELECT".to_string(),
                    "FROM".to_string(),
                    "WHERE".to_string(),
                ]);
            }
        }
        
        suggestions
    }
    
    pub fn debug_tree(&mut self, sql: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tree = self.parse_sql(sql)?;
        Ok(tree.root_node().to_sexp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = TreeSitterSqlParser::new().unwrap();
        let sql = "SELECT * FROM trade_deal WHERE price > 100";
        let tree = parser.parse_sql(sql).unwrap();
        
        assert!(!tree.root_node().has_error());
    }
    
    #[test]
    fn test_context_detection() {
        let mut parser = TreeSitterSqlParser::new().unwrap();
        let sql = "SELECT * FROM trade_deal WHERE allocationStatus.Contains('All') AND ";
        let cursor_pos = sql.len(); // At the end
        
        let context = parser.get_context_at_cursor(sql, cursor_pos).unwrap();
        assert!(context.is_in_where_clause);
        assert!(context.is_after_logical_operator);
    }
}