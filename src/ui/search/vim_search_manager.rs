use crate::data::data_view::DataView;
use crate::ui::viewport_manager::ViewportManager;
use tracing::{debug, error, info, warn};

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

                // Log current state before moving
                info!(target: "vim_search", 
                    "=== 'n' KEY PRESSED - BEFORE NAVIGATION ===");
                info!(target: "vim_search", 
                    "Current match index: {}/{}, Pattern: '{}'", 
                    *current_index + 1, matches.len(), pattern);
                info!(target: "vim_search", 
                    "Current viewport - rows: {:?}, cols: {:?}", 
                    viewport.get_viewport_rows(), viewport.viewport_cols());
                info!(target: "vim_search", 
                    "Current crosshair position: row={}, col={}", 
                    viewport.get_crosshair_row(), viewport.get_crosshair_col());

                // Wrap around to beginning
                *current_index = (*current_index + 1) % matches.len();
                let match_item = matches[*current_index].clone();

                info!(target: "vim_search", 
                    "=== NEXT MATCH DETAILS ===");
                info!(target: "vim_search", 
                    "Match {}/{}: row={}, visual_col={}, stored_value='{}'", 
                    *current_index + 1, matches.len(),
                    match_item.row, match_item.col, match_item.value);

                // Double-check: Does this value actually contain our pattern?
                if !match_item
                    .value
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
                {
                    error!(target: "vim_search",
                        "CRITICAL ERROR: Match value '{}' does NOT contain search pattern '{}'!",
                        match_item.value, pattern);
                    error!(target: "vim_search",
                        "This indicates the search index is corrupted or stale!");
                }

                // Log what we expect to find at this position
                info!(target: "vim_search", 
                    "Expected: Cell at row {} col {} should contain substring '{}'", 
                    match_item.row, match_item.col, pattern);

                // Verify the stored match actually contains the pattern
                let stored_contains = match_item
                    .value
                    .to_lowercase()
                    .contains(&pattern.to_lowercase());
                if !stored_contains {
                    warn!(target: "vim_search",
                        "CRITICAL: Stored match '{}' does NOT contain pattern '{}'!",
                        match_item.value, pattern);
                } else {
                    info!(target: "vim_search",
                        "✓ Stored match '{}' contains pattern '{}'",
                        match_item.value, pattern);
                }

                Some(match_item)
            }
            _ => {
                debug!(target: "vim_search", "next_match called but not in navigation mode");
                None
            }
        };

        // Then navigate to it if we have a match
        if let Some(ref match_item) = match_to_navigate {
            info!(target: "vim_search", 
                "=== NAVIGATING TO MATCH ===");
            self.navigate_to_match(match_item, viewport);

            // Log state after navigation
            info!(target: "vim_search", 
                "=== AFTER NAVIGATION ===");
            info!(target: "vim_search", 
                "New viewport - rows: {:?}, cols: {:?}", 
                viewport.get_viewport_rows(), viewport.viewport_cols());
            info!(target: "vim_search", 
                "New crosshair position: row={}, col={}", 
                viewport.get_crosshair_row(), viewport.get_crosshair_col());
            info!(target: "vim_search", 
                "Crosshair should be at: row={}, col={} (visual coordinates)", 
                match_item.row, match_item.col);
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

        info!(target: "vim_search", 
            "=== FIND_MATCHES CALLED ===");
        info!(target: "vim_search", 
            "Pattern passed in: '{}', pattern_lower: '{}', case_sensitive: {}", 
            pattern, pattern_lower, self.case_sensitive);

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
                            // IMPORTANT: The enum_idx is the position in row.values array,
                            // which corresponds to the position in display_columns.
                            // Since we're searching in visual/display order, we use enum_idx directly
                            // as the visual column index for the viewport to understand.

                            // Map enum_idx back to the actual DataTable column for debugging
                            let actual_col = if enum_idx < display_columns.len() {
                                display_columns[enum_idx]
                            } else {
                                enum_idx // Fallback, shouldn't happen
                            };

                            info!(target: "vim_search", 
                                "Found first match in row {} at visual col {} (DataTable col {}, value '{}')", 
                                row_idx, enum_idx, actual_col, value_str);

                            // Extra validation - log if we find "Futures Trading"
                            if value_str.contains("Futures Trading") {
                                warn!(target: "vim_search",
                                    "SUSPICIOUS: Found 'Futures Trading' as a match for pattern '{}' (search_value='{}', pattern_lower='{}')",
                                    pattern, search_value, pattern_lower);
                            }

                            first_match_in_row = Some(SearchMatch {
                                row: row_idx,
                                col: enum_idx, // This is the visual column index in display order
                                value: value_str,
                            });
                        } else {
                            debug!(target: "vim_search", 
                                "Skipping additional match in row {} at visual col {} (enum_idx {}): '{}'", 
                                row_idx, enum_idx, enum_idx, value_str);
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
        info!(target: "vim_search", 
            "=== NAVIGATE_TO_MATCH START ===");
        info!(target: "vim_search", 
            "Target match: row={} (absolute), col={} (visual), value='{}'", 
            match_item.row, match_item.col, match_item.value);

        // Get terminal dimensions to preserve width
        let terminal_width = viewport.get_terminal_width();
        let terminal_height = viewport.get_terminal_height();
        info!(target: "vim_search",
            "Terminal dimensions: width={}, height={}",
            terminal_width, terminal_height);

        // Get current viewport state BEFORE any changes
        let viewport_rows = viewport.get_viewport_rows();
        let viewport_cols = viewport.viewport_cols();
        let viewport_height = viewport_rows.end - viewport_rows.start;
        let viewport_width = viewport_cols.end - viewport_cols.start;

        info!(target: "vim_search",
            "Current viewport BEFORE changes:");
        info!(target: "vim_search",
            "  Rows: {:?} (height={})", viewport_rows, viewport_height);
        info!(target: "vim_search",
            "  Cols: {:?} (width={})", viewport_cols, viewport_width);
        info!(target: "vim_search",
            "  Current crosshair: row={}, col={}",
            viewport.get_crosshair_row(), viewport.get_crosshair_col());

        // ALWAYS center the match in the viewport for predictable behavior
        // The match should appear at viewport position (height/2, width/2)
        let new_row_start = match_item.row.saturating_sub(viewport_height / 2);
        info!(target: "vim_search", 
            "Centering row {} in viewport (height={}), new viewport start row={}", 
            match_item.row, viewport_height, new_row_start);

        // For columns, we can't just divide by 2 because columns have variable widths
        // Instead, try to position the match column reasonably in view
        // Start by trying to show a few columns before the match if possible
        let new_col_start = match_item.col.saturating_sub(3); // Show 3 columns before if possible
        info!(target: "vim_search", 
            "Positioning column {} in viewport, new viewport start col={}", 
            match_item.col, new_col_start);

        // Log what we're about to do
        info!(target: "vim_search",
            "=== VIEWPORT UPDATE ===");
        info!(target: "vim_search",
            "Will call set_viewport with: row_start={}, col_start={}, width={}, height={}",
            new_row_start, new_col_start, terminal_width, terminal_height);

        // Update viewport with preserved terminal dimensions
        viewport.set_viewport(
            new_row_start,
            new_col_start,
            terminal_width, // Use actual terminal width, not column count!
            terminal_height as u16,
        );

        // Get the updated viewport state
        let final_viewport_rows = viewport.get_viewport_rows();
        let final_viewport_cols = viewport.viewport_cols();

        info!(target: "vim_search", 
            "Viewport AFTER set_viewport: rows {:?}, cols {:?}", 
            final_viewport_rows, final_viewport_cols);

        // CRITICAL: Check if our target column is actually in the viewport!
        if match_item.col < final_viewport_cols.start || match_item.col >= final_viewport_cols.end {
            error!(target: "vim_search",
                "CRITICAL ERROR: Target column {} is NOT in viewport {:?} after set_viewport!",
                match_item.col, final_viewport_cols);
            error!(target: "vim_search",
                "We asked for col_start={}, but viewport gave us {:?}",
                new_col_start, final_viewport_cols);
        }

        // Set the crosshair to the ABSOLUTE position of the match
        // The viewport manager uses absolute coordinates internally
        info!(target: "vim_search",
            "=== CROSSHAIR POSITIONING ===");
        info!(target: "vim_search",
            "Setting crosshair to ABSOLUTE position: row={}, col={}",
            match_item.row, match_item.col);

        viewport.set_crosshair(match_item.row, match_item.col);

        // Verify the match is centered in the viewport
        let center_row =
            final_viewport_rows.start + (final_viewport_rows.end - final_viewport_rows.start) / 2;
        let center_col =
            final_viewport_cols.start + (final_viewport_cols.end - final_viewport_cols.start) / 2;

        info!(target: "vim_search",
            "Viewport center is at: row={}, col={}",
            center_row, center_col);
        info!(target: "vim_search",
            "Match is at: row={}, col={}",
            match_item.row, match_item.col);
        info!(target: "vim_search",
            "Distance from center: row_diff={}, col_diff={}",
            (match_item.row as i32 - center_row as i32).abs(),
            (match_item.col as i32 - center_col as i32).abs());

        // Get the viewport-relative position for verification
        if let Some((vp_row, vp_col)) = viewport.get_crosshair_viewport_position() {
            info!(target: "vim_search",
                "Crosshair appears at viewport position: ({}, {})",
                vp_row, vp_col);
            info!(target: "vim_search",
                "Viewport dimensions: {} rows x {} cols",
                final_viewport_rows.end - final_viewport_rows.start,
                final_viewport_cols.end - final_viewport_cols.start);
            info!(target: "vim_search",
                "Expected center position: ({}, {})",
                (final_viewport_rows.end - final_viewport_rows.start) / 2,
                (final_viewport_cols.end - final_viewport_cols.start) / 2);
        } else {
            error!(target: "vim_search",
                "CRITICAL: Crosshair is NOT visible in viewport after centering!");
        }

        // Verify the match is actually visible in the viewport after scrolling
        info!(target: "vim_search",
            "=== VERIFICATION ===");

        if match_item.row < final_viewport_rows.start || match_item.row >= final_viewport_rows.end {
            error!(target: "vim_search", 
                "ERROR: Match row {} is OUTSIDE viewport {:?} after scrolling!", 
                match_item.row, final_viewport_rows);
        } else {
            info!(target: "vim_search",
                "✓ Match row {} is within viewport {:?}",
                match_item.row, final_viewport_rows);
        }

        if match_item.col < final_viewport_cols.start || match_item.col >= final_viewport_cols.end {
            error!(target: "vim_search", 
                "ERROR: Match column {} is OUTSIDE viewport {:?} after scrolling!", 
                match_item.col, final_viewport_cols);
        } else {
            info!(target: "vim_search",
                "✓ Match column {} is within viewport {:?}",
                match_item.col, final_viewport_cols);
        }

        // Final summary
        info!(target: "vim_search", 
            "=== NAVIGATE_TO_MATCH COMPLETE ===");
        info!(target: "vim_search",
            "Match at absolute ({}, {}), crosshair at ({}, {}), viewport rows {:?} cols {:?}", 
            match_item.row, match_item.col,
            viewport.get_crosshair_row(), viewport.get_crosshair_col(),
            final_viewport_rows, final_viewport_cols);
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
