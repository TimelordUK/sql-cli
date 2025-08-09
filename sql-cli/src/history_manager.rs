use crate::history::{CommandHistory, HistoryMatch};

/// Manages history search and navigation
pub struct HistoryManager {
    pub search_query: String,
    pub matches: Vec<HistoryMatch>,
    pub selected_index: usize,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            matches: Vec::new(),
            selected_index: 0,
        }
    }

    /// Update matches based on current search query
    pub fn update_matches(&mut self, history: &CommandHistory) {
        if self.search_query.is_empty() {
            // Show all history entries when no search query
            self.matches = history
                .get_session_entries()
                .iter()
                .map(|entry| HistoryMatch {
                    entry: entry.clone(),
                    score: 100,
                    indices: Vec::new(),
                })
                .collect();
        } else {
            // Perform fuzzy search
            self.matches = history.search(&self.search_query);
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.matches.len() && !self.matches.is_empty() {
            self.selected_index = self.matches.len() - 1;
        }
    }

    /// Navigate to next match
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.matches.len();
        }
    }

    /// Navigate to previous match
    pub fn previous_match(&mut self) {
        if !self.matches.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.matches.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get currently selected match
    pub fn get_selected(&self) -> Option<&HistoryMatch> {
        if self.selected_index < self.matches.len() {
            self.matches.get(self.selected_index)
        } else {
            None
        }
    }

    /// Clear search and reset state
    pub fn clear(&mut self) {
        self.search_query.clear();
        self.matches.clear();
        self.selected_index = 0;
    }

    /// Set search query and update matches
    pub fn set_search(&mut self, query: String, history: &CommandHistory) {
        self.search_query = query;
        self.update_matches(history);
    }

    /// Get visible range of matches for rendering
    pub fn get_visible_range(&self, height: usize) -> (usize, usize) {
        if self.matches.is_empty() {
            return (0, 0);
        }

        let total = self.matches.len();
        let half_height = height / 2;

        // Calculate start index to keep selected item centered
        let start = if self.selected_index <= half_height {
            0
        } else if self.selected_index + half_height >= total {
            total.saturating_sub(height)
        } else {
            self.selected_index - half_height
        };

        let end = (start + height).min(total);
        (start, end)
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
