/// Cursor and text manipulation operations for SQL input
/// These operations use the lexer to understand SQL syntax and provide
/// intelligent cursor movement and text editing
use crate::recursive_parser::Lexer;

pub struct CursorOperations;

impl CursorOperations {
    /// Move cursor to the previous word boundary
    pub fn find_word_boundary_backward(text: &str, cursor_pos: usize) -> usize {
        if cursor_pos == 0 {
            return 0;
        }

        // Use lexer to tokenize and find word boundaries
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize_all_with_positions();

        // Find the token boundary before the cursor
        let mut target_pos = 0;
        for (start, end, _) in tokens.iter().rev() {
            if *end <= cursor_pos {
                // If we're at the start of a token, go to the previous one
                if *end == cursor_pos && start < &cursor_pos {
                    target_pos = *start;
                } else {
                    // Otherwise go to the start of this token
                    for (s, e, _) in tokens.iter().rev() {
                        if *e <= cursor_pos && *s < cursor_pos {
                            target_pos = *s;
                            break;
                        }
                    }
                }
                break;
            }
        }

        target_pos
    }

    /// Move cursor to the next word boundary
    pub fn find_word_boundary_forward(text: &str, cursor_pos: usize) -> usize {
        // Use lexer to tokenize
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize_all_with_positions();

        // Find the next token boundary after cursor
        for (start, _, _) in &tokens {
            if *start > cursor_pos {
                return *start;
            }
        }

        // If no token found, go to end
        text.len()
    }

    /// Delete from cursor to previous word boundary
    pub fn delete_word_backward(text: &str, cursor_pos: usize) -> (String, usize) {
        if cursor_pos == 0 {
            return (text.to_string(), cursor_pos);
        }

        let word_start = Self::find_word_boundary_backward(text, cursor_pos);

        // Delete from word_start to cursor_pos
        let mut new_text = String::new();
        new_text.push_str(&text[..word_start]);
        new_text.push_str(&text[cursor_pos..]);

        (new_text, word_start)
    }

    /// Delete from cursor to next word boundary
    pub fn delete_word_forward(text: &str, cursor_pos: usize) -> (String, usize) {
        if cursor_pos >= text.len() {
            return (text.to_string(), cursor_pos);
        }

        let word_end = Self::find_word_boundary_forward(text, cursor_pos);

        // Delete from cursor_pos to word_end
        let mut new_text = String::new();
        new_text.push_str(&text[..cursor_pos]);
        new_text.push_str(&text[word_end..]);

        (new_text, cursor_pos)
    }

    /// Kill line from cursor to end
    pub fn kill_line(text: &str, cursor_pos: usize) -> (String, String) {
        let killed = text[cursor_pos..].to_string();
        let new_text = text[..cursor_pos].to_string();
        (new_text, killed)
    }

    /// Kill line from start to cursor
    pub fn kill_line_backward(text: &str, cursor_pos: usize) -> (String, String, usize) {
        let killed = text[..cursor_pos].to_string();
        let new_text = text[cursor_pos..].to_string();
        (new_text, killed, 0) // New cursor position is 0
    }

    /// Jump to previous SQL token
    pub fn jump_to_prev_token(text: &str, cursor_pos: usize) -> usize {
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize_all_with_positions();

        // Find the previous significant token (skip whitespace/punctuation)
        let mut target_pos = cursor_pos;
        for (start, _, _) in tokens.iter().rev() {
            if *start < cursor_pos {
                target_pos = *start;
                break;
            }
        }

        target_pos
    }

    /// Jump to next SQL token
    pub fn jump_to_next_token(text: &str, cursor_pos: usize) -> usize {
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize_all_with_positions();

        // Find the next significant token
        for (start, _, _) in &tokens {
            if *start > cursor_pos {
                return *start;
            }
        }

        text.len()
    }

    /// Find position of matching bracket/parenthesis
    pub fn find_matching_bracket(text: &str, cursor_pos: usize) -> Option<usize> {
        let chars: Vec<char> = text.chars().collect();
        if cursor_pos >= chars.len() {
            return None;
        }

        let ch = chars[cursor_pos];
        let (open, close, direction) = match ch {
            '(' => ('(', ')', 1),
            ')' => ('(', ')', -1),
            '[' => ('[', ']', 1),
            ']' => ('[', ']', -1),
            '{' => ('{', '}', 1),
            '}' => ('{', '}', -1),
            _ => return None,
        };

        let mut count = 1;
        let mut pos = cursor_pos as isize;

        while count > 0 {
            pos += direction;
            if pos < 0 || pos >= chars.len() as isize {
                return None;
            }

            let current = chars[pos as usize];
            if current == open {
                count += if direction > 0 { 1 } else { -1 };
            } else if current == close {
                count -= if direction > 0 { 1 } else { -1 };
            }
        }

        Some(pos as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundary_backward() {
        let text = "SELECT * FROM users WHERE id = 1";
        assert_eq!(CursorOperations::find_word_boundary_backward(text, 14), 9); // FROM -> *
        assert_eq!(CursorOperations::find_word_boundary_backward(text, 7), 0); // SELECT start
    }

    #[test]
    fn test_delete_word_backward() {
        let text = "SELECT * FROM users";
        let (new_text, cursor) = CursorOperations::delete_word_backward(text, 19); // At end
        assert_eq!(new_text, "SELECT * FROM ");
        assert_eq!(cursor, 14);
    }

    #[test]
    fn test_kill_line() {
        let text = "SELECT * FROM users WHERE id = 1";
        let (new_text, killed) = CursorOperations::kill_line(text, 19); // After "users"
        assert_eq!(new_text, "SELECT * FROM users");
        assert_eq!(killed, " WHERE id = 1");
    }

    #[test]
    fn test_matching_bracket() {
        let text = "SELECT * FROM (SELECT id FROM users)";
        assert_eq!(CursorOperations::find_matching_bracket(text, 14), Some(35)); // ( -> )
        assert_eq!(CursorOperations::find_matching_bracket(text, 35), Some(14));
        // ) -> (
    }
}
