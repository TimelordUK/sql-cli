use crate::recursive_parser::{Lexer, Token};

/// Manages text navigation and token-based movement
/// Extracted from the monolithic enhanced_tui.rs
pub struct TextNavigator;

impl TextNavigator {
    /// Get the cursor's position in terms of tokens (current_token, total_tokens)
    pub fn get_cursor_token_position(query: &str, cursor_pos: usize) -> (usize, usize) {
        if query.is_empty() {
            return (0, 0);
        }

        // Use lexer to tokenize the query
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();

        if tokens.is_empty() {
            return (0, 0);
        }

        // Special case: cursor at position 0 is always before the first token
        if cursor_pos == 0 {
            return (0, tokens.len());
        }

        // Find which token the cursor is in
        let mut current_token = 0;
        for (i, (start, end, _)) in tokens.iter().enumerate() {
            if cursor_pos >= *start && cursor_pos <= *end {
                current_token = i + 1;
                break;
            } else if cursor_pos < *start {
                // Cursor is between tokens
                current_token = i;
                break;
            }
        }

        // If cursor is after all tokens
        if current_token == 0 && cursor_pos > 0 {
            current_token = tokens.len();
        }

        (current_token, tokens.len())
    }

    /// Get the token at the cursor position
    pub fn get_token_at_cursor(query: &str, cursor_pos: usize) -> Option<String> {
        if query.is_empty() {
            return None;
        }

        // Use lexer to tokenize the query
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();

        // Find the token at cursor position
        for (start, end, token) in &tokens {
            if cursor_pos >= *start && cursor_pos <= *end {
                // Format token nicely
                let token_str = Self::format_token(token);
                return Some(token_str.to_string());
            }
        }

        None
    }

    /// Calculate the target position for jumping to the previous token
    pub fn calculate_prev_token_position(query: &str, cursor_pos: usize) -> Option<usize> {
        if cursor_pos == 0 {
            return None;
        }

        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();

        // Find current token position
        let mut in_token = false;
        let mut current_token_start = 0;
        for (start, end, _) in &tokens {
            if cursor_pos > *start && cursor_pos <= *end {
                in_token = true;
                current_token_start = *start;
                break;
            }
        }

        // Find the previous token start
        let target_pos = if in_token && cursor_pos > current_token_start {
            // If we're in the middle of a token, go to its start
            current_token_start
        } else {
            // Otherwise, find the previous token
            let mut prev_start = 0;
            for (start, _, _) in tokens.iter().rev() {
                if *start < cursor_pos {
                    prev_start = *start;
                    break;
                }
            }
            prev_start
        };

        if target_pos < cursor_pos {
            Some(target_pos)
        } else {
            None
        }
    }

    /// Calculate the target position for jumping to the next token
    pub fn calculate_next_token_position(query: &str, cursor_pos: usize) -> Option<usize> {
        let query_len = query.len();
        if cursor_pos >= query_len {
            return None;
        }

        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();

        // Find current token position
        let mut in_token = false;
        let mut current_token_end = query_len;
        for (start, end, _) in &tokens {
            if cursor_pos >= *start && cursor_pos < *end {
                in_token = true;
                current_token_end = *end;
                break;
            }
        }

        // Find the next token start
        let target_pos = if in_token && cursor_pos < current_token_end {
            // If we're in a token, go to the start of the next token
            let mut next_start = query_len;
            for (start, _, _) in &tokens {
                if *start > current_token_end {
                    next_start = *start;
                    break;
                }
            }
            next_start
        } else {
            // Otherwise, find the next token from current position
            let mut next_start = query_len;
            for (start, _, _) in &tokens {
                if *start > cursor_pos {
                    next_start = *start;
                    break;
                }
            }
            next_start
        };

        if target_pos > cursor_pos && target_pos <= query_len {
            Some(target_pos)
        } else {
            None
        }
    }

    /// Format a token for display
    fn format_token(token: &Token) -> &str {
        match token {
            Token::Select => "SELECT",
            Token::From => "FROM",
            Token::Where => "WHERE",
            Token::GroupBy => "GROUP BY",
            Token::OrderBy => "ORDER BY",
            Token::Having => "HAVING",
            Token::Asc => "ASC",
            Token::Desc => "DESC",
            Token::And => "AND",
            Token::Or => "OR",
            Token::In => "IN",
            Token::DateTime => "DateTime",
            Token::Identifier(s) => s,
            Token::QuotedIdentifier(s) => s,
            Token::StringLiteral(s) => s,
            Token::NumberLiteral(s) => s,
            Token::Star => "*",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::Equal => "=",
            Token::NotEqual => "!=",
            Token::LessThan => "<",
            Token::LessThanOrEqual => "<=",
            Token::GreaterThan => ">",
            Token::GreaterThanOrEqual => ">=",
            Token::Like => "LIKE",
            Token::Not => "NOT",
            Token::Is => "IS",
            Token::Null => "NULL",
            Token::Between => "BETWEEN",
            Token::Limit => "LIMIT",
            Token::Offset => "OFFSET",
            Token::Eof => "EOF",
        }
    }
}

/// Text editing utilities
pub struct TextEditor;

impl TextEditor {
    /// Kill text from beginning of line to cursor position
    /// Returns (killed_text, remaining_text)
    pub fn kill_line_backward(text: &str, cursor_pos: usize) -> Option<(String, String)> {
        if cursor_pos == 0 {
            return None;
        }

        let killed_text = text.chars().take(cursor_pos).collect::<String>();
        let remaining_text = text.chars().skip(cursor_pos).collect::<String>();

        Some((killed_text, remaining_text))
    }

    /// Kill text from cursor position to end of line
    /// Returns (killed_text, remaining_text)
    pub fn kill_line_forward(text: &str, cursor_pos: usize) -> Option<(String, String)> {
        if cursor_pos >= text.len() {
            return None;
        }

        let remaining_text = text.chars().take(cursor_pos).collect::<String>();
        let killed_text = text.chars().skip(cursor_pos).collect::<String>();

        Some((killed_text, remaining_text))
    }

    /// Delete word backward from cursor position
    /// Returns (deleted_text, remaining_text, new_cursor_pos)
    pub fn delete_word_backward(text: &str, cursor_pos: usize) -> Option<(String, String, usize)> {
        if cursor_pos == 0 {
            return None;
        }

        let before_cursor = &text[..cursor_pos];
        let after_cursor = &text[cursor_pos..];

        // Find word boundary, including leading whitespace before the word
        let mut word_start = before_cursor.len();
        let mut chars = before_cursor.chars().rev().peekable();

        // Step 1: Skip trailing whitespace (if any)
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                word_start -= ch.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        // Step 2: Skip the word itself
        while let Some(&ch) = chars.peek() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            word_start -= ch.len_utf8();
            chars.next();
        }

        // Step 3: Include any whitespace before the word (so deleting at a word boundary includes the space)
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                word_start -= ch.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        let deleted_text = text[word_start..cursor_pos].to_string();
        let remaining_text = format!("{}{}", &text[..word_start], after_cursor);

        Some((deleted_text, remaining_text, word_start))
    }

    /// Delete word forward from cursor position
    /// Returns (deleted_text, remaining_text)
    pub fn delete_word_forward(text: &str, cursor_pos: usize) -> Option<(String, String)> {
        if cursor_pos >= text.len() {
            return None;
        }

        let before_cursor = &text[..cursor_pos];
        let after_cursor = &text[cursor_pos..];

        // Find word boundary
        let mut chars = after_cursor.chars();
        let mut word_end = 0;

        // Skip any non-alphanumeric chars at the beginning
        while let Some(ch) = chars.next() {
            word_end += ch.len_utf8();
            if ch.is_alphanumeric() || ch == '_' {
                // Found start of word, now skip the rest of it
                while let Some(ch) = chars.next() {
                    if !ch.is_alphanumeric() && ch != '_' {
                        break;
                    }
                    word_end += ch.len_utf8();
                }
                break;
            }
        }

        let deleted_text = text[cursor_pos..cursor_pos + word_end].to_string();
        let remaining_text = format!("{}{}", before_cursor, &after_cursor[word_end..]);

        Some((deleted_text, remaining_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_token_position() {
        let query = "SELECT * FROM users WHERE id = 1";

        // Cursor at beginning
        assert_eq!(TextNavigator::get_cursor_token_position(query, 0), (0, 8));

        // Cursor in SELECT
        assert_eq!(TextNavigator::get_cursor_token_position(query, 3), (1, 8));

        // Cursor after SELECT
        assert_eq!(TextNavigator::get_cursor_token_position(query, 7), (2, 8));
    }

    #[test]
    fn test_kill_line_backward() {
        let text = "SELECT * FROM users";

        // Kill from middle
        let result = TextEditor::kill_line_backward(text, 8);
        assert_eq!(
            result,
            Some(("SELECT *".to_string(), " FROM users".to_string()))
        );

        // Kill from beginning (no-op)
        let result = TextEditor::kill_line_backward(text, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_word_backward() {
        let text = "SELECT * FROM users";

        // Delete "FROM"
        let result = TextEditor::delete_word_backward(text, 13);
        assert_eq!(
            result,
            Some((" FROM".to_string(), "SELECT * users".to_string(), 8))
        );
    }
}
