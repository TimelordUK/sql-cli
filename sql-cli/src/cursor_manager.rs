/// Manages all cursor and navigation operations
/// This extracts cursor logic from the monolithic enhanced_tui.rs
pub struct CursorManager {
    /// Current cursor position in the input (byte offset)
    input_cursor_position: usize,

    /// Visual cursor position for rendering (col, row)
    _visual_cursor: (usize, usize),

    /// Table navigation position (row, col)
    table_cursor: (usize, usize),

    /// Horizontal scroll offset for wide tables
    horizontal_scroll: u16,

    /// Vertical scroll offset for long results
    vertical_scroll: usize,
}

impl CursorManager {
    pub fn new() -> Self {
        Self {
            input_cursor_position: 0,
            _visual_cursor: (0, 0),
            table_cursor: (0, 0),
            horizontal_scroll: 0,
            vertical_scroll: 0,
        }
    }

    // ========== Input Cursor Methods ==========

    /// Move cursor forward by one word
    pub fn move_word_forward(&mut self, text: &str) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let mut pos = self.input_cursor_position;

        // Skip current word
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.input_cursor_position = pos;
        pos
    }

    /// Move cursor backward by one word
    pub fn move_word_backward(&mut self, text: &str) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let mut pos = self.input_cursor_position;

        if pos == 0 {
            return 0;
        }

        pos -= 1;

        // Skip whitespace
        while pos > 0 && chars[pos].is_whitespace() {
            pos -= 1;
        }

        // Skip to beginning of word
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.input_cursor_position = pos;
        pos
    }

    /// Move cursor to beginning of line
    pub fn move_to_line_start(&mut self) -> usize {
        self.input_cursor_position = 0;
        0
    }

    /// Move cursor to end of line
    pub fn move_to_line_end(&mut self, text: &str) -> usize {
        self.input_cursor_position = text.len();
        text.len()
    }

    /// Move cursor left by one character
    pub fn move_left(&mut self) -> usize {
        if self.input_cursor_position > 0 {
            self.input_cursor_position -= 1;
        }
        self.input_cursor_position
    }

    /// Move cursor right by one character
    pub fn move_right(&mut self, text: &str) -> usize {
        if self.input_cursor_position < text.len() {
            self.input_cursor_position += 1;
        }
        self.input_cursor_position
    }

    /// Set cursor position directly
    pub fn set_position(&mut self, pos: usize) {
        self.input_cursor_position = pos;
    }

    /// Get current cursor position
    pub fn position(&self) -> usize {
        self.input_cursor_position
    }

    // ========== Table Navigation Methods ==========

    /// Move selection up in table
    pub fn move_table_up(&mut self) -> (usize, usize) {
        if self.table_cursor.0 > 0 {
            self.table_cursor.0 -= 1;
        }
        self.table_cursor
    }

    /// Move selection down in table
    pub fn move_table_down(&mut self, max_rows: usize) -> (usize, usize) {
        if self.table_cursor.0 < max_rows.saturating_sub(1) {
            self.table_cursor.0 += 1;
        }
        self.table_cursor
    }

    /// Move selection left in table
    pub fn move_table_left(&mut self) -> (usize, usize) {
        if self.table_cursor.1 > 0 {
            self.table_cursor.1 -= 1;
        }
        self.table_cursor
    }

    /// Move selection right in table
    pub fn move_table_right(&mut self, max_cols: usize) -> (usize, usize) {
        if self.table_cursor.1 < max_cols.saturating_sub(1) {
            self.table_cursor.1 += 1;
        }
        self.table_cursor
    }

    /// Move to first row
    pub fn move_table_home(&mut self) -> (usize, usize) {
        self.table_cursor.0 = 0;
        self.table_cursor
    }

    /// Move to last row
    pub fn move_table_end(&mut self, max_rows: usize) -> (usize, usize) {
        self.table_cursor.0 = max_rows.saturating_sub(1);
        self.table_cursor
    }

    /// Page up in table
    pub fn page_up(&mut self, page_size: usize) -> (usize, usize) {
        self.table_cursor.0 = self.table_cursor.0.saturating_sub(page_size);
        self.table_cursor
    }

    /// Page down in table
    pub fn page_down(&mut self, page_size: usize, max_rows: usize) -> (usize, usize) {
        self.table_cursor.0 = (self.table_cursor.0 + page_size).min(max_rows.saturating_sub(1));
        self.table_cursor
    }

    /// Get current table cursor position
    pub fn table_position(&self) -> (usize, usize) {
        self.table_cursor
    }

    /// Reset table cursor to origin
    pub fn reset_table_cursor(&mut self) {
        self.table_cursor = (0, 0);
    }

    // ========== Scroll Management Methods ==========

    /// Update horizontal scroll based on cursor position and viewport
    pub fn update_horizontal_scroll(&mut self, cursor_col: usize, viewport_width: u16) {
        let cursor_x = cursor_col as u16;

        // Scroll right if cursor is beyond viewport
        if cursor_x >= self.horizontal_scroll + viewport_width {
            self.horizontal_scroll = cursor_x.saturating_sub(viewport_width - 1);
        }

        // Scroll left if cursor is before viewport
        if cursor_x < self.horizontal_scroll {
            self.horizontal_scroll = cursor_x;
        }
    }

    /// Update vertical scroll based on cursor position and viewport
    pub fn update_vertical_scroll(&mut self, cursor_row: usize, viewport_height: usize) {
        // Scroll down if cursor is below viewport
        if cursor_row >= self.vertical_scroll + viewport_height {
            self.vertical_scroll = cursor_row.saturating_sub(viewport_height - 1);
        }

        // Scroll up if cursor is above viewport
        if cursor_row < self.vertical_scroll {
            self.vertical_scroll = cursor_row;
        }
    }

    /// Get current scroll offsets
    pub fn scroll_offsets(&self) -> (u16, usize) {
        (self.horizontal_scroll, self.vertical_scroll)
    }

    /// Set scroll offsets directly
    pub fn set_scroll_offsets(&mut self, horizontal: u16, vertical: usize) {
        self.horizontal_scroll = horizontal;
        self.vertical_scroll = vertical;
    }

    // ========== Token/Word Utilities ==========

    /// Find word boundaries at current position
    pub fn get_word_at_cursor(&self, text: &str) -> Option<(usize, usize, String)> {
        if text.is_empty() || self.input_cursor_position > text.len() {
            return None;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut start = self.input_cursor_position;
        let mut end = self.input_cursor_position;

        // If at whitespace, no word
        if start < chars.len() && chars[start].is_whitespace() {
            return None;
        }

        // Find word start
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }

        // Find word end
        while end < chars.len() && !chars[end].is_whitespace() {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();
        Some((start, end, word))
    }

    /// Get partial word before cursor (for completion)
    pub fn get_partial_word_before_cursor(&self, text: &str) -> Option<String> {
        if self.input_cursor_position == 0 {
            return None;
        }

        let before_cursor = &text[..self.input_cursor_position];
        let last_space = before_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);

        if last_space < self.input_cursor_position {
            Some(before_cursor[last_space..].to_string())
        } else {
            None
        }
    }
}

/// Extension trait to integrate CursorManager with Buffer
pub trait CursorBuffer {
    fn cursor_manager(&self) -> &CursorManager;
    fn cursor_manager_mut(&mut self) -> &mut CursorManager;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_navigation() {
        let mut cm = CursorManager::new();
        let text = "SELECT * FROM table WHERE id = 1";

        // Start at beginning
        assert_eq!(cm.position(), 0);

        // Move forward by word
        cm.move_word_forward(text);
        assert_eq!(cm.position(), 7); // After "SELECT "

        cm.move_word_forward(text);
        assert_eq!(cm.position(), 9); // After "* "

        // Move backward by word
        cm.move_word_backward(text);
        assert_eq!(cm.position(), 7); // Back to "*"

        cm.move_word_backward(text);
        assert_eq!(cm.position(), 0); // Back to "SELECT"
    }

    #[test]
    fn test_table_navigation() {
        let mut cm = CursorManager::new();

        // Test movement within bounds
        cm.move_table_down(10);
        assert_eq!(cm.table_position(), (1, 0));

        cm.move_table_right(5);
        assert_eq!(cm.table_position(), (1, 1));

        // Test boundary conditions
        cm.move_table_end(10);
        assert_eq!(cm.table_position(), (9, 1));

        cm.move_table_home();
        assert_eq!(cm.table_position(), (0, 1));
    }

    #[test]
    fn test_scroll_management() {
        let mut cm = CursorManager::new();

        // Test horizontal scroll
        cm.update_horizontal_scroll(100, 80);
        assert_eq!(cm.scroll_offsets().0, 21); // 100 - 80 + 1

        // Test vertical scroll
        cm.update_vertical_scroll(50, 20);
        assert_eq!(cm.scroll_offsets().1, 31); // 50 - 20 + 1
    }

    #[test]
    fn test_word_extraction() {
        let mut cm = CursorManager::new();
        let text = "SELECT column FROM table";

        // Position at "column"
        cm.set_position(7);
        let word = cm.get_word_at_cursor(text);
        assert_eq!(word, Some((7, 13, "column".to_string())));

        // Position at space
        cm.set_position(6);
        let word = cm.get_word_at_cursor(text);
        assert_eq!(word, None);

        // Partial word for completion
        cm.set_position(10); // Middle of "column"
        let partial = cm.get_partial_word_before_cursor(text);
        assert_eq!(partial, Some("col".to_string()));
    }
}
