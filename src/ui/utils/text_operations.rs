// Pure text manipulation functions with no TUI dependencies
// These functions take text and cursor position, return results

/// Result of a text operation that modifies the text
#[derive(Debug, Clone)]
pub struct TextOperationResult {
    /// The new text after the operation
    pub new_text: String,
    /// The new cursor position after the operation
    pub new_cursor_position: usize,
    /// Text that was deleted/killed (for kill ring)
    pub killed_text: Option<String>,
    /// Description of what happened
    pub description: String,
}

/// Result of a cursor movement operation
#[derive(Debug, Clone)]
pub struct CursorMovementResult {
    /// The new cursor position
    pub new_position: usize,
    /// The word or token that was jumped over
    pub jumped_text: Option<String>,
}

// ========== Pure Text Manipulation Functions ==========

/// Kill text from cursor to end of line (Ctrl+K behavior)
#[must_use]
pub fn kill_line(text: &str, cursor_position: usize) -> TextOperationResult {
    let text_len = text.len();

    if cursor_position >= text_len {
        // Cursor at end, nothing to kill
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: cursor_position,
            killed_text: None,
            description: "Nothing to kill".to_string(),
        };
    }

    // Find the end of the current line
    let line_end = text[cursor_position..]
        .find('\n')
        .map_or(text_len, |pos| cursor_position + pos);

    let killed = text[cursor_position..line_end].to_string();
    let mut new_text = String::with_capacity(text_len);
    new_text.push_str(&text[..cursor_position]);

    // If we're killing up to a newline, keep the newline
    if line_end < text_len && text.chars().nth(line_end) == Some('\n') {
        new_text.push('\n');
        new_text.push_str(&text[line_end + 1..]);
    } else {
        new_text.push_str(&text[line_end..]);
    }

    let killed_len = killed.len();
    TextOperationResult {
        new_text,
        new_cursor_position: cursor_position,
        killed_text: Some(killed),
        description: format!("Killed {killed_len} characters"),
    }
}

/// Kill text from beginning of line to cursor (Ctrl+U behavior)
#[must_use]
pub fn kill_line_backward(text: &str, cursor_position: usize) -> TextOperationResult {
    if cursor_position == 0 {
        // Cursor at start, nothing to kill
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: 0,
            killed_text: None,
            description: "Nothing to kill".to_string(),
        };
    }

    // Find the start of the current line
    let line_start = text[..cursor_position].rfind('\n').map_or(0, |pos| pos + 1);

    let killed = text[line_start..cursor_position].to_string();
    let mut new_text = String::with_capacity(text.len());
    new_text.push_str(&text[..line_start]);
    new_text.push_str(&text[cursor_position..]);

    let killed_len = killed.len();
    TextOperationResult {
        new_text,
        new_cursor_position: line_start,
        killed_text: Some(killed),
        description: format!("Killed {killed_len} characters backward"),
    }
}

/// Delete word backward from cursor (Ctrl+W behavior)
#[must_use]
pub fn delete_word_backward(text: &str, cursor_position: usize) -> TextOperationResult {
    if cursor_position == 0 {
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: 0,
            killed_text: None,
            description: "At beginning of text".to_string(),
        };
    }

    // Skip any trailing whitespace
    let mut pos = cursor_position;
    while pos > 0 && text.chars().nth(pos - 1).is_some_and(char::is_whitespace) {
        pos -= 1;
    }

    // Find the start of the word
    let word_start = if pos == 0 {
        0
    } else {
        let mut start = pos;
        while start > 0 && !text.chars().nth(start - 1).is_some_and(char::is_whitespace) {
            start -= 1;
        }
        start
    };

    let killed = text[word_start..cursor_position].to_string();
    let mut new_text = String::with_capacity(text.len());
    new_text.push_str(&text[..word_start]);
    new_text.push_str(&text[cursor_position..]);

    let killed_trimmed = killed.trim().to_string();
    TextOperationResult {
        new_text,
        new_cursor_position: word_start,
        killed_text: Some(killed),
        description: format!("Deleted word: '{killed_trimmed}'"),
    }
}

/// Delete word forward from cursor (Alt+D behavior)
#[must_use]
pub fn delete_word_forward(text: &str, cursor_position: usize) -> TextOperationResult {
    let text_len = text.len();
    if cursor_position >= text_len {
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: cursor_position,
            killed_text: None,
            description: "At end of text".to_string(),
        };
    }

    // Skip any leading whitespace
    let mut pos = cursor_position;
    while pos < text_len && text.chars().nth(pos).is_some_and(char::is_whitespace) {
        pos += 1;
    }

    // Find the end of the word
    let word_end = if pos >= text_len {
        text_len
    } else {
        let mut end = pos;
        while end < text_len && !text.chars().nth(end).is_some_and(char::is_whitespace) {
            end += 1;
        }
        end
    };

    let killed = text[cursor_position..word_end].to_string();
    let mut new_text = String::with_capacity(text.len());
    new_text.push_str(&text[..cursor_position]);
    new_text.push_str(&text[word_end..]);

    let killed_trimmed = killed.trim().to_string();
    TextOperationResult {
        new_text,
        new_cursor_position: cursor_position,
        killed_text: Some(killed),
        description: format!("Deleted word: '{killed_trimmed}'"),
    }
}

// ========== Pure Cursor Movement Functions ==========

/// Move cursor backward one word (Ctrl+Left or Alt+B)
#[must_use]
pub fn move_word_backward(text: &str, cursor_position: usize) -> CursorMovementResult {
    if cursor_position == 0 {
        return CursorMovementResult {
            new_position: 0,
            jumped_text: None,
        };
    }

    // Skip any trailing whitespace
    let mut pos = cursor_position;
    while pos > 0 && text.chars().nth(pos - 1).is_some_and(char::is_whitespace) {
        pos -= 1;
    }

    // Find the start of the word
    let word_start = if pos == 0 {
        0
    } else {
        let mut start = pos;
        while start > 0 && !text.chars().nth(start - 1).is_some_and(char::is_whitespace) {
            start -= 1;
        }
        start
    };

    let jumped = if word_start < cursor_position {
        Some(text[word_start..cursor_position].to_string())
    } else {
        None
    };

    CursorMovementResult {
        new_position: word_start,
        jumped_text: jumped,
    }
}

/// Move cursor forward one word (Ctrl+Right or Alt+F)
#[must_use]
pub fn move_word_forward(text: &str, cursor_position: usize) -> CursorMovementResult {
    let text_len = text.len();
    if cursor_position >= text_len {
        return CursorMovementResult {
            new_position: cursor_position,
            jumped_text: None,
        };
    }

    // Skip current word
    let mut pos = cursor_position;
    while pos < text_len && !text.chars().nth(pos).is_some_and(char::is_whitespace) {
        pos += 1;
    }

    // Skip whitespace
    while pos < text_len && text.chars().nth(pos).is_some_and(char::is_whitespace) {
        pos += 1;
    }

    let jumped = if pos > cursor_position {
        Some(text[cursor_position..pos].to_string())
    } else {
        None
    };

    CursorMovementResult {
        new_position: pos,
        jumped_text: jumped,
    }
}

/// Jump to previous SQL token (more sophisticated than word)
#[must_use]
pub fn jump_to_prev_token(text: &str, cursor_position: usize) -> CursorMovementResult {
    if cursor_position == 0 {
        return CursorMovementResult {
            new_position: 0,
            jumped_text: None,
        };
    }

    // SQL tokens include: keywords, identifiers, operators, literals
    // For now, implement similar to word but can be enhanced for SQL
    let mut pos = cursor_position;

    // Skip any trailing whitespace or operators
    while pos > 0 {
        let ch = text.chars().nth(pos - 1);
        if let Some(c) = ch {
            if c.is_whitespace() || "(),;=<>!+-*/".contains(c) {
                pos -= 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Find the start of the token
    let token_start = if pos == 0 {
        0
    } else {
        let mut start = pos;
        while start > 0 {
            let ch = text.chars().nth(start - 1);
            if let Some(c) = ch {
                if !c.is_whitespace() && !"(),;=<>!+-*/".contains(c) {
                    start -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        start
    };

    let jumped = if token_start < cursor_position {
        Some(text[token_start..cursor_position].to_string())
    } else {
        None
    };

    CursorMovementResult {
        new_position: token_start,
        jumped_text: jumped,
    }
}

/// Jump to next SQL token
#[must_use]
pub fn jump_to_next_token(text: &str, cursor_position: usize) -> CursorMovementResult {
    let text_len = text.len();
    if cursor_position >= text_len {
        return CursorMovementResult {
            new_position: cursor_position,
            jumped_text: None,
        };
    }

    let mut pos = cursor_position;

    // Skip current token
    while pos < text_len {
        let ch = text.chars().nth(pos);
        if let Some(c) = ch {
            if !c.is_whitespace() && !"(),;=<>!+-*/".contains(c) {
                pos += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Skip whitespace and operators to next token
    while pos < text_len {
        let ch = text.chars().nth(pos);
        if let Some(c) = ch {
            if c.is_whitespace() || "(),;=<>!+-*/".contains(c) {
                pos += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let jumped = if pos > cursor_position {
        Some(text[cursor_position..pos].to_string())
    } else {
        None
    };

    CursorMovementResult {
        new_position: pos,
        jumped_text: jumped,
    }
}

// ========== Helper Functions ==========

/// Clear all text (simple helper)
#[must_use]
pub fn clear_text() -> TextOperationResult {
    TextOperationResult {
        new_text: String::new(),
        new_cursor_position: 0,
        killed_text: None,
        description: "Cleared all text".to_string(),
    }
}

/// Insert character at cursor position
#[must_use]
pub fn insert_char(text: &str, cursor_position: usize, ch: char) -> TextOperationResult {
    let mut new_text = String::with_capacity(text.len() + 1);
    new_text.push_str(&text[..cursor_position.min(text.len())]);
    new_text.push(ch);
    if cursor_position < text.len() {
        new_text.push_str(&text[cursor_position..]);
    }

    TextOperationResult {
        new_text,
        new_cursor_position: cursor_position + 1,
        killed_text: None,
        description: format!("Inserted '{ch}'"),
    }
}

/// Delete character at cursor position (Delete key)
#[must_use]
pub fn delete_char(text: &str, cursor_position: usize) -> TextOperationResult {
    if cursor_position >= text.len() {
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: cursor_position,
            killed_text: None,
            description: "Nothing to delete".to_string(),
        };
    }

    let deleted = text.chars().nth(cursor_position).unwrap();
    let mut new_text = String::with_capacity(text.len() - 1);
    new_text.push_str(&text[..cursor_position]);
    new_text.push_str(&text[cursor_position + 1..]);

    TextOperationResult {
        new_text,
        new_cursor_position: cursor_position,
        killed_text: Some(deleted.to_string()),
        description: format!("Deleted '{deleted}'"),
    }
}

/// Delete character before cursor (Backspace)
#[must_use]
pub fn backspace(text: &str, cursor_position: usize) -> TextOperationResult {
    if cursor_position == 0 {
        return TextOperationResult {
            new_text: text.to_string(),
            new_cursor_position: 0,
            killed_text: None,
            description: "At beginning".to_string(),
        };
    }

    let deleted = text.chars().nth(cursor_position - 1).unwrap();
    let mut new_text = String::with_capacity(text.len() - 1);
    new_text.push_str(&text[..cursor_position - 1]);
    new_text.push_str(&text[cursor_position..]);

    TextOperationResult {
        new_text,
        new_cursor_position: cursor_position - 1,
        killed_text: Some(deleted.to_string()),
        description: format!("Deleted '{deleted}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_line() {
        let text = "SELECT * FROM table WHERE id = 1";
        let result = kill_line(text, 7);
        assert_eq!(result.new_text, "SELECT ");
        assert_eq!(
            result.killed_text,
            Some("* FROM table WHERE id = 1".to_string())
        );
        assert_eq!(result.new_cursor_position, 7);
    }

    #[test]
    fn test_kill_line_backward() {
        let text = "SELECT * FROM table";
        let result = kill_line_backward(text, 7);
        assert_eq!(result.new_text, "* FROM table");
        assert_eq!(result.killed_text, Some("SELECT ".to_string()));
        assert_eq!(result.new_cursor_position, 0);
    }

    #[test]
    fn test_delete_word_backward() {
        let text = "SELECT * FROM table";
        let result = delete_word_backward(text, 13); // After "FROM"
        assert_eq!(result.new_text, "SELECT *  table");
        assert_eq!(result.killed_text, Some("FROM".to_string()));
        assert_eq!(result.new_cursor_position, 9);
    }

    #[test]
    fn test_move_word_forward() {
        let text = "SELECT * FROM table";
        let result = move_word_forward(text, 0);
        assert_eq!(result.new_position, 7); // After "SELECT "

        let result2 = move_word_forward(text, 7);
        assert_eq!(result2.new_position, 9); // After "* "
    }

    #[test]
    fn test_move_word_backward() {
        let text = "SELECT * FROM table";
        let result = move_word_backward(text, 13); // From end of "FROM"
        assert_eq!(result.new_position, 9); // Start of "FROM"

        let result2 = move_word_backward(text, 9);
        assert_eq!(result2.new_position, 7); // Start of "*"
    }

    #[test]
    fn test_jump_to_next_token() {
        let text = "SELECT id, name FROM users WHERE id = 1";
        let result = jump_to_next_token(text, 0);
        assert_eq!(result.new_position, 7); // After "SELECT "

        let result2 = jump_to_next_token(text, 7);
        assert_eq!(result2.new_position, 11); // After "id, " (skips comma and space)
    }

    #[test]
    fn test_insert_and_delete() {
        let text = "SELECT";
        let result = insert_char(text, 6, ' ');
        assert_eq!(result.new_text, "SELECT ");
        assert_eq!(result.new_cursor_position, 7);

        let result2 = delete_char(&result.new_text, 6);
        assert_eq!(result2.new_text, "SELECT");

        let result3 = backspace(&result.new_text, 7);
        assert_eq!(result3.new_text, "SELECT");
        assert_eq!(result3.new_cursor_position, 6);
    }
}

// ========== SQL-Specific Text Functions ==========

/// Result of applying a completion to text
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// The new text with completion applied
    pub new_text: String,
    /// The new cursor position after completion
    pub new_cursor_position: usize,
    /// Description of what was completed
    pub description: String,
}

/// Splice a completion suggestion over `query[replace_start..cursor_pos]`.
///
/// `replace_start` comes from the parser (see
/// [`crate::sql::cursor_aware_parser::ParseResult::replace_start`]) because
/// only it knows whether the text before the cursor is a column reference to
/// be replaced wholesale or a method name hanging off one. The suggestion is
/// inserted verbatim - quoting is already decided by the suggestion itself, and
/// the span it replaces includes any opening quote the user typed.
#[must_use]
pub fn apply_completion_to_text(
    query: &str,
    cursor_pos: usize,
    replace_start: usize,
    suggestion: &str,
) -> CompletionResult {
    let replace_start = replace_start.min(cursor_pos);
    let before = &query[..replace_start];
    let after_cursor = &query[cursor_pos..];

    let new_query = format!("{before}{suggestion}{after_cursor}");

    // Smart cursor positioning based on function signature
    let new_cursor_pos = if suggestion.ends_with("('')") {
        // Function with parameters - position cursor between the quotes
        // e.g., Contains('|') where | is cursor
        replace_start + suggestion.len() - 2
    } else {
        replace_start + suggestion.len()
    };

    // Better description based on completion type
    let description = if suggestion.ends_with("('')") {
        format!("Completed '{suggestion}' with cursor positioned for parameter input")
    } else if suggestion.ends_with("()") {
        format!("Completed parameterless function '{suggestion}'")
    } else {
        format!("Completed '{suggestion}'")
    };

    CompletionResult {
        new_text: new_query,
        new_cursor_position: new_cursor_pos,
        description,
    }
}
