//! Search and filter operations
//!
//! This module contains search and filter operations extracted from the monolithic TUI
//! to improve maintainability and testability. The search system is complex with multiple
//! modes and state dependencies, so we start with extracting coordination logic.

use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::data::data_view::DataView;
use crate::widgets::search_modes_widget::SearchMode;
use std::sync::Arc;

/// Context for search operations
/// Provides the minimal interface needed for search operations
pub struct SearchContext<'a> {
    pub state_container: &'a Arc<AppStateContainer>,
    pub buffer: &'a mut dyn BufferAPI,
    pub current_data: Option<&'a DataView>,
}

/// Result of a search operation
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub matches_found: usize,
    pub first_match: Option<(usize, usize)>, // (row, col)
    pub status_message: String,
}

/// Result of search action execution
#[derive(Debug, Clone, PartialEq)]
pub enum SearchActionResult {
    /// Search was executed successfully
    Success(SearchResult),
    /// Search was started but needs follow-up action
    InProgress(String),
    /// Search failed
    Error(String),
}

/// Execute a search action based on mode and pattern
/// This coordinates the search process but delegates the actual searching
pub fn execute_search_action(
    mode: SearchMode,
    pattern: String,
    ctx: &mut SearchContext,
) -> SearchActionResult {
    match mode {
        SearchMode::Search => {
            // Set search pattern in state container
            ctx.state_container.start_search(pattern.clone());
            ctx.buffer.set_search_pattern(pattern);

            // Perform the actual search (this will need to be extracted separately)
            let search_result = perform_search_with_context(ctx);

            match search_result {
                Ok(result) => SearchActionResult::Success(result),
                Err(e) => SearchActionResult::Error(e),
            }
        }
        SearchMode::Filter => {
            SearchActionResult::InProgress(format!("Filter mode with pattern: {}", pattern))
        }
        SearchMode::FuzzyFilter => {
            SearchActionResult::InProgress(format!("Fuzzy filter mode with pattern: {}", pattern))
        }
        SearchMode::ColumnSearch => {
            SearchActionResult::InProgress(format!("Column search mode with pattern: {}", pattern))
        }
    }
}

/// Perform search with the given context
/// This is where the actual search logic will eventually be extracted
fn perform_search_with_context(ctx: &mut SearchContext) -> Result<SearchResult, String> {
    if let Some(dataview) = ctx.current_data {
        // Convert DataView rows to Vec<Vec<String>> for search
        let data: Vec<Vec<String>> = (0..dataview.row_count())
            .filter_map(|i| dataview.get_row(i))
            .map(|row| row.values.iter().map(|v| v.to_string()).collect())
            .collect();

        // Perform search using AppStateContainer
        let matches = ctx.state_container.perform_search(&data);

        // Convert matches to our format
        let buffer_matches: Vec<(usize, usize)> = matches
            .iter()
            .map(|(row, col, _, _)| (*row, *col))
            .collect();

        if !buffer_matches.is_empty() {
            let first_match = buffer_matches[0];

            // Update buffer state
            ctx.buffer.set_search_matches(buffer_matches.clone());
            ctx.buffer.set_search_match_index(0);
            ctx.buffer.set_current_match(Some(first_match));

            Ok(SearchResult {
                matches_found: buffer_matches.len(),
                first_match: Some(first_match),
                status_message: format!("Found {} matches", buffer_matches.len()),
            })
        } else {
            ctx.buffer.set_search_matches(Vec::new());

            Ok(SearchResult {
                matches_found: 0,
                first_match: None,
                status_message: "No matches found".to_string(),
            })
        }
    } else {
        Err("No data available for search".to_string())
    }
}

/// Apply search result to UI state
pub fn apply_search_result(result: &SearchResult, ctx: &mut SearchContext) {
    if let Some((row, _col)) = result.first_match {
        ctx.state_container.set_table_selected_row(Some(row));
    }

    ctx.buffer.set_status_message(result.status_message.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_action_result_types() {
        // Test that our result types work correctly
        let success = SearchActionResult::Success(SearchResult {
            matches_found: 5,
            first_match: Some((0, 1)),
            status_message: "Found 5 matches".to_string(),
        });

        match success {
            SearchActionResult::Success(result) => {
                assert_eq!(result.matches_found, 5);
                assert_eq!(result.first_match, Some((0, 1)));
                assert_eq!(result.status_message, "Found 5 matches");
            }
            _ => panic!("Expected Success result"),
        }

        let in_progress = SearchActionResult::InProgress("Filtering...".to_string());
        match in_progress {
            SearchActionResult::InProgress(msg) => {
                assert_eq!(msg, "Filtering...");
            }
            _ => panic!("Expected InProgress result"),
        }

        let error = SearchActionResult::Error("No data".to_string());
        match error {
            SearchActionResult::Error(msg) => {
                assert_eq!(msg, "No data");
            }
            _ => panic!("Expected Error result"),
        }
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            matches_found: 0,
            first_match: None,
            status_message: "No matches found".to_string(),
        };

        assert_eq!(result.matches_found, 0);
        assert_eq!(result.first_match, None);
        assert_eq!(result.status_message, "No matches found");
    }
}
