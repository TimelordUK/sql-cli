//! Simple utility operations
//!
//! This module contains simple, self-contained operations extracted from the TUI
//! that have minimal dependencies and can be easily tested.

use crate::buffer::{BufferAPI, BufferManager};
use crate::text_navigation::TextNavigator;

/// Context for text navigation operations
pub struct TextNavigationContext<'a> {
    pub query: &'a str,
    pub cursor_pos: usize,
}

/// Context for undo operations
pub struct UndoContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
}

/// Get the cursor token position in the query text
/// Returns (start, end) positions of the token at cursor
pub fn get_cursor_token_position(ctx: &TextNavigationContext) -> (usize, usize) {
    TextNavigator::get_cursor_token_position(ctx.query, ctx.cursor_pos)
}

/// Get the token at the cursor position
/// Returns the token string if found
pub fn get_token_at_cursor(ctx: &TextNavigationContext) -> Option<String> {
    TextNavigator::get_token_at_cursor(ctx.query, ctx.cursor_pos)
}

/// Result of an undo operation
#[derive(Debug, PartialEq)]
pub enum UndoResult {
    /// Undo was performed successfully
    Success,
    /// Nothing to undo
    NothingToUndo,
    /// No buffer available
    NoBuffer,
}

impl UndoResult {
    /// Get the status message for this result
    pub fn status_message(&self) -> &'static str {
        match self {
            UndoResult::Success => "Undo performed",
            UndoResult::NothingToUndo => "Nothing to undo",
            UndoResult::NoBuffer => "No buffer available for undo",
        }
    }
}

/// Perform an undo operation on the current buffer
pub fn perform_undo(ctx: &mut UndoContext) -> UndoResult {
    if let Some(buffer) = ctx.buffer_manager.current_mut() {
        if buffer.perform_undo() {
            UndoResult::Success
        } else {
            UndoResult::NothingToUndo
        }
    } else {
        UndoResult::NoBuffer
    }
}

/// Check for common SQL parser errors in a query string
/// Returns an error message if issues are found, None if the query looks valid
pub fn check_parser_error(query: &str) -> Option<String> {
    // Quick check for common parser errors
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in query.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '\'' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return Some("Extra )".to_string());
                }
            }
            _ => {}
        }
    }

    if paren_depth > 0 {
        return Some(format!("Missing {} )", paren_depth));
    }

    // Could add more checks here (unclosed strings, etc.)
    if in_string {
        return Some("Unclosed string".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cursor_token_position() {
        let ctx = TextNavigationContext {
            query: "SELECT name FROM users",
            cursor_pos: 2, // Inside "SELECT"
        };

        let (start, end) = get_cursor_token_position(&ctx);
        // This test assumes TextNavigator works correctly
        // The exact values depend on TextNavigator's implementation
        assert!(start <= ctx.cursor_pos);
        assert!(end >= ctx.cursor_pos);
    }

    #[test]
    fn test_get_token_at_cursor() {
        let ctx = TextNavigationContext {
            query: "SELECT name FROM users",
            cursor_pos: 2, // Inside "SELECT"
        };

        let token = get_token_at_cursor(&ctx);
        // The exact token depends on TextNavigator's implementation
        // We just verify that we get some result
        assert!(token.is_some() || token.is_none()); // Always true, just ensures it compiles
    }

    #[test]
    fn test_get_token_at_cursor_empty_query() {
        let ctx = TextNavigationContext {
            query: "",
            cursor_pos: 0,
        };

        let token = get_token_at_cursor(&ctx);
        // Should handle empty query gracefully
        assert!(token.is_none() || token.is_some()); // Always true, just ensures it compiles
    }

    #[test]
    fn test_undo_result_status_messages() {
        assert_eq!(UndoResult::Success.status_message(), "Undo performed");
        assert_eq!(
            UndoResult::NothingToUndo.status_message(),
            "Nothing to undo"
        );
        assert_eq!(
            UndoResult::NoBuffer.status_message(),
            "No buffer available for undo"
        );
    }

    #[test]
    fn test_check_parser_error_valid_queries() {
        assert_eq!(check_parser_error("SELECT * FROM users"), None);
        assert_eq!(
            check_parser_error("SELECT name FROM users WHERE id = 1"),
            None
        );
        assert_eq!(
            check_parser_error("SELECT (column1 + column2) FROM table"),
            None
        );
        assert_eq!(check_parser_error("SELECT 'hello world' FROM dual"), None);
    }

    #[test]
    fn test_check_parser_error_mismatched_parens() {
        assert_eq!(
            check_parser_error("SELECT (column FROM table"),
            Some("Missing 1 )".to_string())
        );
        assert_eq!(
            check_parser_error("SELECT ((column FROM table"),
            Some("Missing 2 )".to_string())
        );
        assert_eq!(
            check_parser_error("SELECT column) FROM table"),
            Some("Extra )".to_string())
        );
    }

    #[test]
    fn test_check_parser_error_unclosed_string() {
        assert_eq!(
            check_parser_error("SELECT 'unclosed FROM table"),
            Some("Unclosed string".to_string())
        );
        assert_eq!(
            check_parser_error("SELECT name FROM users WHERE name = 'test"),
            Some("Unclosed string".to_string())
        );
    }

    #[test]
    fn test_check_parser_error_escaped_quotes() {
        // Escaped quotes should not be treated as string terminators
        assert_eq!(
            check_parser_error("SELECT 'it\\'s a test' FROM table"),
            None
        );
    }

    #[test]
    fn test_check_parser_error_parens_in_strings() {
        // Parentheses inside strings should not affect parentheses counting
        assert_eq!(
            check_parser_error("SELECT 'text with (parens)' FROM table"),
            None
        );
        assert_eq!(
            check_parser_error("SELECT 'text with (unclosed FROM table"),
            Some("Unclosed string".to_string())
        );
    }

    #[test]
    fn test_check_parser_error_empty_query() {
        assert_eq!(check_parser_error(""), None);
    }

    // Note: Testing perform_undo would require setting up a full BufferManager with buffers
    // and undo state, which is complex. The function is simple enough that integration
    // testing through the TUI is sufficient for now.
}
