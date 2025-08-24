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

    /// Clear all search state and return to inactive mode
    pub fn clear(&mut self) {
        info!(target: "vim_search", "Clearing all search state");
        self.state = VimSearchState::Inactive;
        self.last_search_pattern = None;
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

    /// Reset to first match (for 'g' key)
    pub fn reset_to_first_match(&mut self, viewport: &mut ViewportManager) -> Option<SearchMatch> {
        match &mut self.state {
            VimSearchState::Navigating {
                matches,
                current_index,
                ..
            } => {
                if matches.is_empty() {
                    return None;
                }

                // Reset to first match
                *current_index = 0;
                let first_match = matches[0].clone();

                info!(target: "vim_search", 
                    "Resetting to first match at ({}, {})", 
                    first_match.row, first_match.col);

                // Navigate to the first match
                self.navigate_to_match(&first_match, viewport);
                Some(first_match)
            }
            _ => {
                debug!(target: "vim_search", "reset_to_first_match called but not in navigation mode");
                None
            }
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

        // Get the display column indices to map enumeration index to actual column index
        let display_columns = dataview.get_display_columns();
        debug!(target: "vim_search", 
            "Display columns mapping: {:?} (count: {})", 
            display_columns, display_columns.len());

        // Search through all visible data
        for row_idx in 0..dataview.row_count() {
            if let Some(row) = dataview.get_row(row_idx) {
                let mut first_match_in_row: Option<SearchMatch> = None;

                // The row.values are in display order
                for (enum_idx, value) in row.values.iter().enumerate() {
                    let value_str = value.to_string();
                    let search_value = if !self.case_sensitive {
                        value_str.to_lowercase()
                    } else {
                        value_str.clone()
                    };

                    if search_value.contains(&pattern_lower) {
                        // For vim-like behavior, we prioritize the first match in each row
                        // This prevents jumping between columns on the same row
                        if first_match_in_row.is_none() {
                            debug!(target: "vim_search", 
                                "Found first match in row {} at col {}: '{}'", 
                                row_idx, enum_idx, value_str);
                            first_match_in_row = Some(SearchMatch {
                                row: row_idx,
                                col: enum_idx, // Use the enumeration index as the visual column index
                                value: value_str,
                            });
                        } else {
                            debug!(target: "vim_search", 
                                "Skipping additional match in row {} at col {}: '{}'", 
                                row_idx, enum_idx, value_str);
                        }
                    }
                }

                // Add the first match from this row if we found one
                if let Some(match_item) = first_match_in_row {
                    matches.push(match_item);
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

    /// Set search state from external search (e.g., SearchModesWidget)
    /// This allows 'n' and 'N' to work after a regular search
    pub fn set_search_state_from_external(
        &mut self,
        pattern: String,
        matches: Vec<(usize, usize)>,
        dataview: &DataView,
    ) {
        info!(target: "vim_search", 
            "Setting search state from external search: pattern='{}', {} matches", 
            pattern, matches.len());

        // Convert matches to SearchMatch format
        let search_matches: Vec<SearchMatch> = matches
            .into_iter()
            .filter_map(|(row, col)| {
                if let Some(row_data) = dataview.get_row(row) {
                    if col < row_data.values.len() {
                        Some(SearchMatch {
                            row,
                            col,
                            value: row_data.values[col].to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !search_matches.is_empty() {
            let match_count = search_matches.len();

            // Set the state to navigating
            self.state = VimSearchState::Navigating {
                pattern: pattern.clone(),
                matches: search_matches,
                current_index: 0,
            };
            self.last_search_pattern = Some(pattern);

            info!(target: "vim_search", 
                "Vim search state updated: {} matches ready for navigation", 
                match_count);
        } else {
            warn!(target: "vim_search", "No valid matches to set in vim search state");
        }
    }
}
