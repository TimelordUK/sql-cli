/// Text processing utilities extracted from enhanced_tui
/// Contains cursor management, word extraction, and text manipulation functions

/// Extract a partial word at the cursor position in a query string
/// Used for completion and search functionality
pub fn extract_partial_word_at_cursor(query: &str, cursor_pos: usize) -> Option<String> {
    if cursor_pos == 0 || cursor_pos > query.len() {
        return None;
    }

    let chars: Vec<char> = query.chars().collect();
    let mut start = cursor_pos;
    let end = cursor_pos;

    // Check if we might be in a quoted identifier
    let mut in_quote = false;

    // Find start of word (go backward)
    while start > 0 {
        let prev_char = chars[start - 1];
        if prev_char == '"' {
            // Found a quote, include it and stop
            start -= 1;
            in_quote = true;
            break;
        } else if prev_char.is_alphanumeric() || prev_char == '_' || (prev_char == ' ' && in_quote)
        {
            start -= 1;
        } else {
            break;
        }
    }

    // Convert back to byte positions
    let start_byte = chars[..start].iter().map(|c| c.len_utf8()).sum();
    let end_byte = chars[..end].iter().map(|c| c.len_utf8()).sum();

    if start_byte < end_byte {
        Some(query[start_byte..end_byte].to_string())
    } else {
        None
    }
}

/// Get the token at cursor position in SQL text
pub fn get_token_at_cursor(sql_text: &str, cursor_pos: usize) -> Option<String> {
    if sql_text.is_empty() || cursor_pos > sql_text.len() {
        return None;
    }

    let chars: Vec<char> = sql_text.chars().collect();
    if cursor_pos > chars.len() {
        return None;
    }

    // Find word boundaries
    let mut start = cursor_pos;
    let mut end = cursor_pos;

    // Move start backward to beginning of word
    while start > 0 {
        let idx = start - 1;
        if idx < chars.len() && (chars[idx].is_alphanumeric() || chars[idx] == '_') {
            start -= 1;
        } else {
            break;
        }
    }

    // Move end forward to end of word
    while end < chars.len() {
        if chars[end].is_alphanumeric() || chars[end] == '_' {
            end += 1;
        } else {
            break;
        }
    }

    if start < end {
        let token: String = chars[start..end].iter().collect();
        Some(token)
    } else {
        None
    }
}

/// Calculate the cursor position within a token for syntax highlighting
pub fn get_cursor_token_position(sql_text: &str, cursor_pos: usize) -> (usize, usize) {
    if let Some(token) = get_token_at_cursor(sql_text, cursor_pos) {
        // Find where this token starts in the text
        let before_cursor = &sql_text[..cursor_pos.min(sql_text.len())];
        if let Some(rev_pos) = before_cursor.rfind(&token) {
            let token_start = rev_pos;
            let pos_in_token = cursor_pos.saturating_sub(token_start);
            return (token_start, pos_in_token);
        }
    }
    (cursor_pos, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_partial_word() {
        assert_eq!(
            extract_partial_word_at_cursor("SELECT coun", 11),
            Some("coun".to_string())
        );

        assert_eq!(
            extract_partial_word_at_cursor("SELECT \"quoted col", 18),
            Some("\"quoted col".to_string())
        );

        assert_eq!(extract_partial_word_at_cursor("", 0), None);
    }

    #[test]
    fn test_get_token_at_cursor() {
        assert_eq!(
            get_token_at_cursor("SELECT column_name FROM", 10),
            Some("column_name".to_string())
        );

        assert_eq!(
            get_token_at_cursor("WHERE id = 123", 7),
            Some("id".to_string())
        );
    }
}
