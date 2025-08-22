use crate::data::data_view::DataView;
use crate::ui::viewport_manager::ViewportManager;
use tracing::{debug, info, warn};

/// Represents a search match in the data
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub row: usize,
    pub col: usize,
    pub value: String,
}

/// State for vim-like search mode
#[derive(Debug, Clone)]
pub enum VimSearchState {
    /// Not in search mode
    Inactive,
    /// Typing search pattern (/ mode)
    Typing { pattern: String },
    /// Search confirmed, navigating matches (after Enter)
    Navigating {
        pattern: String,
        matches: Vec<SearchMatch>,
        current_index: usize,
    },
}

/// Manages vim-like forward search behavior
pub struct VimSearchManager {
    state: VimSearchState,
    case_sensitive: bool,
    last_search_pattern: Option<String>,
}

impl VimSearchManager {
    pub fn new() -> Self {
        Self {
            state: VimSearchState::Inactive,
            case_sensitive: false,
            last_search_pattern: None,
        }
    }

    /// Start search mode (when / is pressed)
    pub fn start_search(&mut self) {
        info!(target: "vim_search", "Starting vim search mode");
        self.state = VimSearchState::Typing {
            pattern: String::new(),
        };
    }

    /// Update search pattern and find first match dynamically
    pub fn update_pattern(
        &mut self,
        pattern: String,
        dataview: &DataView,
        viewport: &mut ViewportManager,
    ) -> Option<SearchMatch> {
        debug!(target: "vim_search", "Updating pattern to: '{}'", pattern);

        // Update state to typing mode with new pattern
        self.state = VimSearchState::Typing {
            pattern: pattern.clone(),
        };

        if pattern.is_empty() {
            return None;
        }

        // Find all matches
        let matches = self.find_matches(&pattern, dataview);

        if let Some(first_match) = matches.first() {
            debug!(target: "vim_search", 
                "Found {} matches, navigating to first at ({}, {})", 
                matches.len(), first_match.row, first_match.col);

            // Navigate to first match and ensure it's visible
            self.navigate_to_match(first_match, viewport);
            Some(first_match.clone())
        } else {
            debug!(target: "vim_search", "No matches found for pattern: '{}'", pattern);
            None
        }
    }

    /// Confirm search (when Enter is pressed) - enter navigation mode
    pub fn confirm_search(&mut self, dataview: &DataView, viewport: &mut ViewportManager) -> bool {
        match &self.state {
            VimSearchState::Typing { pattern } => {
                if pattern.is_empty() {
                    info!(target: "vim_search", "Empty pattern, canceling search");
                    self.cancel_search();
                    return false;
                }

                let pattern = pattern.clone();
                let matches = self.find_matches(&pattern, dataview);

                if matches.is_empty() {
                    warn!(target: "vim_search", "No matches found for pattern: '{}'", pattern);
                    self.cancel_search();
                    return false;
                }

                info!(target: "vim_search", 
                    "Confirming search with {} matches for pattern: '{}'", 
                    matches.len(), pattern);

                // Navigate to first match
                if let Some(first_match) = matches.first() {
                    self.navigate_to_match(first_match, viewport);
                }

                // Enter navigation mode
                self.state = VimSearchState::Navigating {
                    pattern: pattern.clone(),
                    matches,
                    current_index: 0,
                };
                self.last_search_pattern = Some(pattern);
                true
            }
            _ => {
                warn!(target: "vim_search", "confirm_search called in wrong state: {:?}", self.state);
                false
            }
        }
    }

    /// Navigate to next match (n key)
    pub fn next_match(&mut self, viewport: &mut ViewportManager) -> Option<SearchMatch> {
        // First, update the index and get the match
        let match_to_navigate = match &mut self.state {
            VimSearchState::Navigating {
                matches,
                current_index,
                pattern,
            } => {
                if matches.is_empty() {
                    return None;
                }

                // Wrap around to beginning
                *current_index = (*current_index + 1) % matches.len();
                let match_item = matches[*current_index].clone();

                info!(target: "vim_search", 
                    "Navigating to next match {}/{} at ({}, {})", 
                    *current_index + 1, matches.len(), match_item.row, match_item.col);

                Some(match_item)
            }
            _ => {
                debug!(target: "vim_search", "next_match called but not in navigation mode");
                None
            }
        };

        // Then navigate to it if we have a match
        if let Some(ref match_item) = match_to_navigate {
            self.navigate_to_match(match_item, viewport);
        }

        match_to_navigate
    }

    /// Navigate to previous match (N key)
    pub fn previous_match(&mut self, viewport: &mut ViewportManager) -> Option<SearchMatch> {
        // First, update the index and get the match
        let match_to_navigate = match &mut self.state {
            VimSearchState::Navigating {
                matches,
                current_index,
                pattern,
            } => {
                if matches.is_empty() {
                    return None;
                }

                // Wrap around to end
                *current_index = if *current_index == 0 {
                    matches.len() - 1
                } else {
                    *current_index - 1
                };

                let match_item = matches[*current_index].clone();

                info!(target: "vim_search", 
                    "Navigating to previous match {}/{} at ({}, {})", 
                    *current_index + 1, matches.len(), match_item.row, match_item.col);

                Some(match_item)
            }
            _ => {
                debug!(target: "vim_search", "previous_match called but not in navigation mode");
                None
            }
        };

        // Then navigate to it if we have a match
        if let Some(ref match_item) = match_to_navigate {
            self.navigate_to_match(match_item, viewport);
        }

        match_to_navigate
    }

    /// Cancel search and return to normal mode
    pub fn cancel_search(&mut self) {
        info!(target: "vim_search", "Canceling search, returning to inactive state");
        self.state = VimSearchState::Inactive;
    }

    /// Exit navigation mode but keep search pattern for later
    pub fn exit_navigation(&mut self) {
        if let VimSearchState::Navigating { pattern, .. } = &self.state {
            self.last_search_pattern = Some(pattern.clone());
        }
        self.state = VimSearchState::Inactive;
    }

    /// Resume search with last pattern (for repeating search with /)
    pub fn resume_last_search(
        &mut self,
        dataview: &DataView,
        viewport: &mut ViewportManager,
    ) -> bool {
        if let Some(pattern) = &self.last_search_pattern {
            let pattern = pattern.clone();
            let matches = self.find_matches(&pattern, dataview);

            if !matches.is_empty() {
                info!(target: "vim_search", 
                    "Resuming search with pattern '{}', found {} matches", 
                    pattern, matches.len());

                // Navigate to first match
                if let Some(first_match) = matches.first() {
                    self.navigate_to_match(first_match, viewport);
                }

                self.state = VimSearchState::Navigating {
                    pattern,
                    matches,
                    current_index: 0,
                };
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if currently in search mode
    pub fn is_active(&self) -> bool {
        !matches!(self.state, VimSearchState::Inactive)
    }

    /// Check if in typing mode
    pub fn is_typing(&self) -> bool {
        matches!(self.state, VimSearchState::Typing { .. })
    }

    /// Check if in navigation mode
    pub fn is_navigating(&self) -> bool {
        matches!(self.state, VimSearchState::Navigating { .. })
    }

    /// Get current pattern
    pub fn get_pattern(&self) -> Option<String> {
        match &self.state {
            VimSearchState::Typing { pattern } => Some(pattern.clone()),
            VimSearchState::Navigating { pattern, .. } => Some(pattern.clone()),
            VimSearchState::Inactive => None,
        }
    }

    /// Get current match info for status display
    pub fn get_match_info(&self) -> Option<(usize, usize)> {
        match &self.state {
            VimSearchState::Navigating {
                matches,
                current_index,
                ..
            } => Some((*current_index + 1, matches.len())),
            _ => None,
        }
    }

    /// Find all matches in the dataview
    fn find_matches(&self, pattern: &str, dataview: &DataView) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        let pattern_lower = if !self.case_sensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        debug!(target: "vim_search", 
            "Searching for '{}' (case_sensitive: {})", 
            pattern, self.case_sensitive);

        // Search through all visible data
        for row_idx in 0..dataview.row_count() {
            if let Some(row) = dataview.get_row(row_idx) {
                for (col_idx, value) in row.values.iter().enumerate() {
                    let value_str = value.to_string();
                    let search_value = if !self.case_sensitive {
                        value_str.to_lowercase()
                    } else {
                        value_str.clone()
                    };

                    if search_value.contains(&pattern_lower) {
                        debug!(target: "vim_search", 
                            "Found match at ({}, {}): '{}'", 
                            row_idx, col_idx, value_str);
                        matches.push(SearchMatch {
                            row: row_idx,
                            col: col_idx,
                            value: value_str,
                        });
                    }
                }
            }
        }

        debug!(target: "vim_search", "Found {} total matches", matches.len());
        matches
    }

    /// Navigate viewport to ensure match is visible and set crosshair
    fn navigate_to_match(&self, match_item: &SearchMatch, viewport: &mut ViewportManager) {
        debug!(target: "vim_search", 
            "Navigating to match at row={}, col={}", 
            match_item.row, match_item.col);

        // Ensure row is visible
        let viewport_rows = viewport.get_viewport_rows();
        let viewport_height = viewport_rows.end - viewport_rows.start;

        // If row is not visible, scroll to center it
        if match_item.row < viewport_rows.start || match_item.row >= viewport_rows.end {
            // Calculate new viewport start to center the match
            let new_start = match_item.row.saturating_sub(viewport_height / 2);
            let viewport_cols = viewport.viewport_cols();
            let col_width = (viewport_cols.end - viewport_cols.start) as u16;
            viewport.set_viewport(
                new_start,
                viewport_cols.start,
                col_width,
                viewport_height as u16,
            );
            debug!(target: "vim_search", 
                "Scrolled viewport to show row {}, new viewport: {:?}", 
                match_item.row, viewport.get_viewport_rows());
        }

        // Ensure column is visible
        let viewport_cols = viewport.viewport_cols();
        if match_item.col < viewport_cols.start || match_item.col >= viewport_cols.end {
            // Scroll horizontally to show the column
            let col_width = viewport_cols.end - viewport_cols.start;
            let new_col_start = match_item.col.saturating_sub(col_width / 4);
            let viewport_rows_for_set = viewport.get_viewport_rows();
            viewport.set_viewport(
                viewport_rows_for_set.start,
                new_col_start,
                col_width as u16,
                viewport_height as u16,
            );
            debug!(target: "vim_search", 
                "Scrolled viewport to show column {}, new viewport: {:?}", 
                match_item.col, viewport.viewport_cols());
        }

        // Set crosshair position to the match
        // IMPORTANT: The crosshair uses ABSOLUTE coordinates, not viewport-relative
        // The ViewportManager internally handles the conversion when needed
        viewport.set_crosshair(match_item.row, match_item.col);

        // Verify the match is actually visible in the viewport after scrolling
        let viewport_rows = viewport.get_viewport_rows();
        if match_item.row < viewport_rows.start || match_item.row >= viewport_rows.end {
            warn!(target: "vim_search", 
                "Match row {} is outside viewport {:?} after scrolling", 
                match_item.row, viewport_rows);
        }

        info!(target: "vim_search", 
            "Positioned crosshair at absolute ({}, {}) for match", 
            match_item.row, match_item.col);
    }

    /// Set case sensitivity for search
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
        debug!(target: "vim_search", "Case sensitivity set to: {}", case_sensitive);
    }
}
