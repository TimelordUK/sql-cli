// Helper functions for CSV handling improvements

use std::collections::HashMap;

/// Check if a column name needs quoting (contains spaces or special characters)
pub fn needs_quoting(column_name: &str) -> bool {
    column_name.contains(' ') || 
    column_name.contains('-') || 
    column_name.contains('.') ||
    column_name.contains('(') ||
    column_name.contains(')') ||
    column_name.contains('[') ||
    column_name.contains(']') ||
    column_name.contains('"') ||
    column_name.contains('\'')
}

/// Quote a column name if necessary
pub fn quote_if_needed(column_name: &str) -> String {
    if needs_quoting(column_name) {
        format!("\"{}\"", column_name.replace('"', "\"\""))
    } else {
        column_name.to_string()
    }
}

/// Build a case-insensitive lookup map for column names
/// Maps lowercase column names to their original case versions
pub fn build_column_lookup(headers: &[String]) -> HashMap<String, String> {
    let mut lookup = HashMap::new();
    for header in headers {
        lookup.insert(header.to_lowercase(), header.clone());
    }
    lookup
}

/// Find a column by name (case-insensitive)
pub fn find_column_case_insensitive<'a>(
    obj: &'a serde_json::Map<String, serde_json::Value>,
    column_name: &str,
    lookup: &HashMap<String, String>
) -> Option<(&'a String, &'a serde_json::Value)> {
    // First try exact match
    if let Some(value) = obj.get_key_value(column_name) {
        return Some(value);
    }
    
    // Then try case-insensitive match using lookup
    if let Some(actual_name) = lookup.get(&column_name.to_lowercase()) {
        obj.get_key_value(actual_name)
    } else {
        // Fallback: linear search (for quoted columns)
        let column_unquoted = column_name.trim_matches('"');
        for (key, value) in obj {
            if key == column_unquoted || key.to_lowercase() == column_unquoted.to_lowercase() {
                return Some((key, value));
            }
        }
        None
    }
}

/// Parse a column name that might be quoted
pub fn parse_column_name(column: &str) -> &str {
    column.trim().trim_matches('"').trim_matches('\'')
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_needs_quoting() {
        assert!(!needs_quoting("City"));
        assert!(!needs_quoting("customer_id"));
        assert!(needs_quoting("Phone 1"));
        assert!(needs_quoting("Customer-ID"));
        assert!(needs_quoting("Price ($)"));
    }
    
    #[test]
    fn test_quote_if_needed() {
        assert_eq!(quote_if_needed("City"), "City");
        assert_eq!(quote_if_needed("Phone 1"), "\"Phone 1\"");
        assert_eq!(quote_if_needed("Has\"Quote"), "\"Has\"\"Quote\"");
    }
}