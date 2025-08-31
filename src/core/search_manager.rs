//! SearchManager - Encapsulates all search logic for the TUI
//!
//! This module provides a clean separation between search logic and UI rendering.
//! It handles case sensitivity, coordinate mapping, and iteration through matches.

use tracing::warn;

/// Represents a single search match
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    /// Row index in the data (0-based)
    pub row: usize,
    /// Column index in the data (0-based)
    pub column: usize,
    /// The actual value that matched
    pub value: String,
    /// The highlighted portion of the match
    pub highlight_range: (usize, usize),
}

/// Configuration for search behavior
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Whether search is case sensitive
    pub case_sensitive: bool,
    /// Whether to use regex matching
    pub use_regex: bool,
    /// Whether to search only visible columns
    pub visible_columns_only: bool,
    /// Whether to wrap around when reaching the end
    pub wrap_around: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            use_regex: false,
            visible_columns_only: false,
            wrap_around: true,
        }
    }
}

/// Manages search state and provides iteration through matches
pub struct SearchManager {
    /// Current search pattern
    pattern: String,
    /// All matches found
    matches: Vec<SearchMatch>,
    /// Current match index
    current_index: usize,
    /// Search configuration
    config: SearchConfig,
    /// Cached regex (if using regex mode)
    regex: Option<regex::Regex>,
}

impl SearchManager {
    /// Create a new SearchManager with default config
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            matches: Vec::new(),
            current_index: 0,
            config: SearchConfig::default(),
            regex: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: SearchConfig) -> Self {
        Self {
            pattern: String::new(),
            matches: Vec::new(),
            current_index: 0,
            config,
            regex: None,
        }
    }

    /// Update search configuration
    pub fn set_config(&mut self, config: SearchConfig) {
        // Clear regex cache if switching modes
        if !config.use_regex {
            self.regex = None;
        }
        self.config = config;
    }

    /// Set case sensitivity
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.config.case_sensitive = case_sensitive;
        // Re-compile regex if needed
        if self.config.use_regex && !self.pattern.is_empty() {
            self.compile_regex();
        }
    }

    /// Perform a search on the given data
    /// Returns the number of matches found
    pub fn search(
        &mut self,
        pattern: &str,
        data: &[Vec<String>],
        visible_columns: Option<&[usize]>,
    ) -> usize {
        self.pattern = pattern.to_string();
        self.matches.clear();
        self.current_index = 0;

        if pattern.is_empty() {
            return 0;
        }

        // Compile regex if needed
        if self.config.use_regex {
            self.compile_regex();
            if self.regex.is_none() {
                return 0; // Invalid regex
            }
        }

        // Determine which columns to search
        let columns_to_search: Vec<usize> = if self.config.visible_columns_only {
            visible_columns
                .map(|cols| cols.to_vec())
                .unwrap_or_else(|| {
                    // If no visible columns specified, search all
                    if !data.is_empty() {
                        (0..data[0].len()).collect()
                    } else {
                        vec![]
                    }
                })
        } else {
            // Search all columns
            if !data.is_empty() {
                (0..data[0].len()).collect()
            } else {
                vec![]
            }
        };

        // Search through data
        for (row_idx, row) in data.iter().enumerate() {
            for &col_idx in &columns_to_search {
                if col_idx >= row.len() {
                    continue;
                }

                let cell_value = &row[col_idx];
                if let Some(range) = self.matches_pattern(cell_value, pattern) {
                    self.matches.push(SearchMatch {
                        row: row_idx,
                        column: col_idx,
                        value: cell_value.clone(),
                        highlight_range: range,
                    });
                }
            }
        }

        self.matches.len()
    }

    /// Check if a value matches the pattern and return highlight range
    fn matches_pattern(&self, value: &str, pattern: &str) -> Option<(usize, usize)> {
        if self.config.use_regex {
            // Use regex matching
            if let Some(ref regex) = self.regex {
                if let Some(m) = regex.find(value) {
                    return Some((m.start(), m.end()));
                }
            }
        } else {
            // Use substring matching
            let search_value = if self.config.case_sensitive {
                value.to_string()
            } else {
                value.to_lowercase()
            };

            let search_pattern = if self.config.case_sensitive {
                pattern.to_string()
            } else {
                pattern.to_lowercase()
            };

            if let Some(pos) = search_value.find(&search_pattern) {
                return Some((pos, pos + pattern.len()));
            }
        }
        None
    }

    /// Compile regex pattern
    fn compile_regex(&mut self) {
        let pattern = if self.config.case_sensitive {
            self.pattern.clone()
        } else {
            format!("(?i){}", self.pattern)
        };

        match regex::Regex::new(&pattern) {
            Ok(regex) => self.regex = Some(regex),
            Err(e) => {
                warn!("Invalid regex pattern: {}", e);
                self.regex = None;
            }
        }
    }

    /// Get the current match (if any)
    pub fn current_match(&self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            None
        } else {
            self.matches.get(self.current_index)
        }
    }

    /// Move to the next match
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        if self.current_index + 1 < self.matches.len() {
            self.current_index += 1;
        } else if self.config.wrap_around {
            self.current_index = 0;
        }

        self.current_match()
    }

    /// Move to the previous match
    pub fn previous_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
        } else if self.config.wrap_around {
            self.current_index = self.matches.len() - 1;
        }

        self.current_match()
    }

    /// Jump to a specific match index
    pub fn jump_to_match(&mut self, index: usize) -> Option<&SearchMatch> {
        if index < self.matches.len() {
            self.current_index = index;
            self.current_match()
        } else {
            None
        }
    }

    /// Get the first match (useful for initial navigation)
    pub fn first_match(&self) -> Option<&SearchMatch> {
        self.matches.first()
    }

    /// Get all matches
    pub fn all_matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    /// Get the total number of matches
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get current match index (1-based for display)
    pub fn current_match_number(&self) -> usize {
        if self.matches.is_empty() {
            0
        } else {
            self.current_index + 1
        }
    }

    /// Clear all search state
    pub fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
        self.current_index = 0;
        self.regex = None;
    }

    /// Check if there's an active search
    pub fn has_active_search(&self) -> bool {
        !self.pattern.is_empty()
    }

    /// Get the current search pattern
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Calculate scroll offset needed to show a match in viewport
    pub fn calculate_scroll_offset(
        &self,
        match_pos: &SearchMatch,
        viewport_height: usize,
        current_offset: usize,
    ) -> usize {
        let row = match_pos.row;

        // If match is above current view, scroll up to it
        if row < current_offset {
            row
        }
        // If match is below current view, center it
        else if row >= current_offset + viewport_height {
            row.saturating_sub(viewport_height / 2)
        }
        // Match is already visible
        else {
            current_offset
        }
    }

    /// Find the next match from a given position
    pub fn find_next_from(&self, current_row: usize, current_col: usize) -> Option<&SearchMatch> {
        // Find matches after current position
        for match_item in &self.matches {
            if match_item.row > current_row
                || (match_item.row == current_row && match_item.column > current_col)
            {
                return Some(match_item);
            }
        }

        // Wrap around if enabled
        if self.config.wrap_around && !self.matches.is_empty() {
            return self.matches.first();
        }

        None
    }

    /// Find the previous match from a given position
    pub fn find_previous_from(
        &self,
        current_row: usize,
        current_col: usize,
    ) -> Option<&SearchMatch> {
        // Find matches before current position (in reverse)
        for match_item in self.matches.iter().rev() {
            if match_item.row < current_row
                || (match_item.row == current_row && match_item.column < current_col)
            {
                return Some(match_item);
            }
        }

        // Wrap around if enabled
        if self.config.wrap_around && !self.matches.is_empty() {
            return self.matches.last();
        }

        None
    }
}

/// Iterator for search matches
pub struct SearchIterator<'a> {
    manager: &'a SearchManager,
    index: usize,
}

impl<'a> Iterator for SearchIterator<'a> {
    type Item = &'a SearchMatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.manager.matches.len() {
            let result = &self.manager.matches[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl SearchManager {
    /// Get an iterator over all matches
    pub fn iter(&self) -> SearchIterator {
        SearchIterator {
            manager: self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_search() {
        let mut manager = SearchManager::new();
        manager.set_case_sensitive(false);

        let data = vec![
            vec!["Unconfirmed".to_string(), "data1".to_string()],
            vec!["unconfirmed".to_string(), "data2".to_string()],
            vec!["UNCONFIRMED".to_string(), "data3".to_string()],
            vec!["confirmed".to_string(), "data4".to_string()],
        ];

        let count = manager.search("unconfirmed", &data, None);
        assert_eq!(count, 3);

        // All case variations should match
        let matches: Vec<_> = manager.iter().collect();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].row, 0);
        assert_eq!(matches[1].row, 1);
        assert_eq!(matches[2].row, 2);
    }

    #[test]
    fn test_case_sensitive_search() {
        let mut manager = SearchManager::new();
        manager.set_case_sensitive(true);

        let data = vec![
            vec!["Unconfirmed".to_string(), "data1".to_string()],
            vec!["unconfirmed".to_string(), "data2".to_string()],
            vec!["UNCONFIRMED".to_string(), "data3".to_string()],
        ];

        let count = manager.search("Unconfirmed", &data, None);
        assert_eq!(count, 1);

        let first_match = manager.first_match().unwrap();
        assert_eq!(first_match.row, 0);
        assert_eq!(first_match.value, "Unconfirmed");
    }

    #[test]
    fn test_navigation() {
        let mut manager = SearchManager::new();

        let data = vec![
            vec!["apple".to_string(), "banana".to_string()],
            vec!["apple pie".to_string(), "cherry".to_string()],
            vec!["orange".to_string(), "apple juice".to_string()],
        ];

        manager.search("apple", &data, None);
        assert_eq!(manager.match_count(), 3);

        // Test next navigation
        let first = manager.current_match().unwrap();
        assert_eq!((first.row, first.column), (0, 0));

        let second = manager.next_match().unwrap();
        assert_eq!((second.row, second.column), (1, 0));

        let third = manager.next_match().unwrap();
        assert_eq!((third.row, third.column), (2, 1));

        // Test wrap around
        let wrapped = manager.next_match().unwrap();
        assert_eq!((wrapped.row, wrapped.column), (0, 0));

        // Test previous navigation
        let prev = manager.previous_match().unwrap();
        assert_eq!((prev.row, prev.column), (2, 1));
    }

    #[test]
    fn test_visible_columns_filter() {
        let mut config = SearchConfig::default();
        config.visible_columns_only = true;
        let mut manager = SearchManager::with_config(config);

        let data = vec![
            vec![
                "apple".to_string(),
                "hidden".to_string(),
                "banana".to_string(),
            ],
            vec![
                "orange".to_string(),
                "apple".to_string(),
                "cherry".to_string(),
            ],
        ];

        // Search only in columns 0 and 2 (column 1 is hidden)
        let visible = vec![0, 2];
        let count = manager.search("apple", &data, Some(&visible));

        // Should only find apple in column 0 of row 0, not in column 1 of row 1
        assert_eq!(count, 1);
        let match_item = manager.first_match().unwrap();
        assert_eq!(match_item.row, 0);
        assert_eq!(match_item.column, 0);
    }

    #[test]
    fn test_scroll_offset_calculation() {
        let manager = SearchManager::new();

        let match_item = SearchMatch {
            row: 50,
            column: 0,
            value: String::new(),
            highlight_range: (0, 0),
        };

        // Match below viewport - should center
        let offset = manager.calculate_scroll_offset(&match_item, 20, 10);
        assert_eq!(offset, 40); // 50 - 20/2

        // Match above viewport - should scroll to it
        let offset = manager.calculate_scroll_offset(&match_item, 20, 60);
        assert_eq!(offset, 50);

        // Match already visible - keep current offset
        let offset = manager.calculate_scroll_offset(&match_item, 20, 45);
        assert_eq!(offset, 45);
    }

    #[test]
    fn test_find_from_position() {
        let mut manager = SearchManager::new();

        let data = vec![
            vec!["a".to_string(), "b".to_string(), "match".to_string()],
            vec!["match".to_string(), "c".to_string(), "d".to_string()],
            vec!["e".to_string(), "match".to_string(), "f".to_string()],
        ];

        manager.search("match", &data, None);

        // Find next from position (0, 1)
        let next = manager.find_next_from(0, 1).unwrap();
        assert_eq!((next.row, next.column), (0, 2));

        // Find next from position (1, 0)
        let next = manager.find_next_from(1, 0).unwrap();
        assert_eq!((next.row, next.column), (2, 1));

        // Find previous from position (2, 0)
        let prev = manager.find_previous_from(2, 0).unwrap();
        assert_eq!((prev.row, prev.column), (1, 0));
    }
}
