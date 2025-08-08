use crate::api_client::QueryResponse;
use crate::buffer::{Buffer, BufferAPI};
use std::collections::HashMap;

/// Manages all data operations, separated from UI concerns
/// This is the data layer that the enhanced_tui should delegate to
pub struct DataManager {
    /// Cache of column widths for each buffer
    column_widths_cache: HashMap<String, Vec<u16>>,
    
    /// Filter state for each buffer
    filter_cache: HashMap<String, FilterState>,
    
    /// Search state for each buffer
    search_cache: HashMap<String, SearchState>,
}

#[derive(Clone, Debug)]
pub struct FilterState {
    pub active: bool,
    pub filter_text: String,
    pub filtered_indices: Vec<usize>,
    pub case_insensitive: bool,
}

#[derive(Clone, Debug)]
pub struct SearchState {
    pub active: bool,
    pub search_text: String,
    pub matches: Vec<(usize, usize)>, // (row, col) positions
    pub current_match: usize,
}

impl DataManager {
    pub fn new() -> Self {
        Self {
            column_widths_cache: HashMap::new(),
            filter_cache: HashMap::new(),
            search_cache: HashMap::new(),
        }
    }
    
    // ========== Column Width Calculations ==========
    
    /// Calculate optimal column widths for display
    pub fn calculate_column_widths(
        &mut self,
        buffer_id: &str,
        results: &QueryResponse,
        max_width: u16,
    ) -> Vec<u16> {
        // Check cache first
        if let Some(cached) = self.column_widths_cache.get(buffer_id) {
            return cached.clone();
        }
        
        if results.columns.is_empty() {
            return vec![];
        }
        
        let mut widths = vec![0u16; results.columns.len()];
        
        // Start with column header widths
        for (i, col) in results.columns.iter().enumerate() {
            widths[i] = col.len() as u16;
        }
        
        // Sample first 100 rows for width calculation
        let sample_size = results.rows.len().min(100);
        for row in results.rows.iter().take(sample_size) {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    let cell_width = self.measure_cell_width(cell);
                    widths[i] = widths[i].max(cell_width);
                }
            }
        }
        
        // Apply constraints
        let total_width: u16 = widths.iter().sum();
        if total_width > max_width {
            self.distribute_width_proportionally(&mut widths, max_width);
        }
        
        // Enforce minimum and maximum widths
        for width in &mut widths {
            *width = (*width).max(4).min(50);
        }
        
        // Cache the result
        self.column_widths_cache.insert(buffer_id.to_string(), widths.clone());
        
        widths
    }
    
    fn measure_cell_width(&self, cell: &str) -> u16 {
        // Handle multi-line cells
        cell.lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16
    }
    
    fn distribute_width_proportionally(&self, widths: &mut [u16], max_total: u16) {
        let total: u16 = widths.iter().sum();
        if total == 0 {
            return;
        }
        
        let scale = max_total as f32 / total as f32;
        for width in widths.iter_mut() {
            *width = ((*width as f32 * scale).floor() as u16).max(4);
        }
    }
    
    // ========== Filtering Operations ==========
    
    /// Apply filter to results
    pub fn apply_filter(
        &mut self,
        buffer_id: &str,
        results: &QueryResponse,
        filter_text: &str,
        case_insensitive: bool,
    ) -> Vec<usize> {
        if filter_text.is_empty() {
            self.filter_cache.remove(buffer_id);
            return (0..results.rows.len()).collect();
        }
        
        let filter = if case_insensitive {
            filter_text.to_lowercase()
        } else {
            filter_text.to_string()
        };
        
        let mut filtered_indices = Vec::new();
        
        for (idx, row) in results.rows.iter().enumerate() {
            let row_text = row.join(" ");
            let compare_text = if case_insensitive {
                row_text.to_lowercase()
            } else {
                row_text
            };
            
            if compare_text.contains(&filter) {
                filtered_indices.push(idx);
            }
        }
        
        // Cache the filter state
        self.filter_cache.insert(
            buffer_id.to_string(),
            FilterState {
                active: true,
                filter_text: filter_text.to_string(),
                filtered_indices: filtered_indices.clone(),
                case_insensitive,
            },
        );
        
        filtered_indices
    }
    
    /// Clear filter for a buffer
    pub fn clear_filter(&mut self, buffer_id: &str) {
        self.filter_cache.remove(buffer_id);
    }
    
    /// Get current filter state
    pub fn get_filter_state(&self, buffer_id: &str) -> Option<&FilterState> {
        self.filter_cache.get(buffer_id)
    }
    
    // ========== Search Operations ==========
    
    /// Search for text in results
    pub fn search_in_results(
        &mut self,
        buffer_id: &str,
        results: &QueryResponse,
        search_text: &str,
        case_insensitive: bool,
    ) -> Vec<(usize, usize)> {
        if search_text.is_empty() {
            self.search_cache.remove(buffer_id);
            return vec![];
        }
        
        let search = if case_insensitive {
            search_text.to_lowercase()
        } else {
            search_text.to_string()
        };
        
        let mut matches = Vec::new();
        
        for (row_idx, row) in results.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let compare_text = if case_insensitive {
                    cell.to_lowercase()
                } else {
                    cell.to_string()
                };
                
                if compare_text.contains(&search) {
                    matches.push((row_idx, col_idx));
                }
            }
        }
        
        // Cache the search state
        self.search_cache.insert(
            buffer_id.to_string(),
            SearchState {
                active: true,
                search_text: search_text.to_string(),
                matches: matches.clone(),
                current_match: 0,
            },
        );
        
        matches
    }
    
    /// Navigate to next search match
    pub fn next_search_match(&mut self, buffer_id: &str) -> Option<(usize, usize)> {
        if let Some(state) = self.search_cache.get_mut(buffer_id) {
            if !state.matches.is_empty() {
                state.current_match = (state.current_match + 1) % state.matches.len();
                return Some(state.matches[state.current_match]);
            }
        }
        None
    }
    
    /// Navigate to previous search match
    pub fn prev_search_match(&mut self, buffer_id: &str) -> Option<(usize, usize)> {
        if let Some(state) = self.search_cache.get_mut(buffer_id) {
            if !state.matches.is_empty() {
                state.current_match = if state.current_match == 0 {
                    state.matches.len() - 1
                } else {
                    state.current_match - 1
                };
                return Some(state.matches[state.current_match]);
            }
        }
        None
    }
    
    /// Clear search for a buffer
    pub fn clear_search(&mut self, buffer_id: &str) {
        self.search_cache.remove(buffer_id);
    }
    
    /// Get current search state
    pub fn get_search_state(&self, buffer_id: &str) -> Option<&SearchState> {
        self.search_cache.get(buffer_id)
    }
    
    // ========== Data Transformation ==========
    
    /// Truncate string for display
    pub fn truncate_for_display(text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            return text.to_string();
        }
        
        if max_width <= 3 {
            return ".".repeat(max_width.min(text.len()));
        }
        
        let truncated = &text[..max_width - 3];
        format!("{}...", truncated)
    }
    
    /// Format cell value for display
    pub fn format_cell_value(value: &str, width: usize, align_right: bool) -> String {
        let truncated = Self::truncate_for_display(value, width);
        
        if align_right {
            format!("{:>width$}", truncated, width = width)
        } else {
            format!("{:<width$}", truncated, width = width)
        }
    }
    
    /// Check if column contains numeric data
    pub fn is_numeric_column(results: &QueryResponse, col_idx: usize) -> bool {
        if col_idx >= results.columns.len() || results.rows.is_empty() {
            return false;
        }
        
        // Sample first 10 non-empty values
        let mut numeric_count = 0;
        let mut sample_count = 0;
        
        for row in results.rows.iter().take(20) {
            if col_idx < row.len() && !row[col_idx].trim().is_empty() {
                if row[col_idx].parse::<f64>().is_ok() {
                    numeric_count += 1;
                }
                sample_count += 1;
                if sample_count >= 10 {
                    break;
                }
            }
        }
        
        // Consider numeric if >70% of samples are numbers
        sample_count > 0 && (numeric_count as f32 / sample_count as f32) > 0.7
    }
    
    // ========== Statistics ==========
    
    /// Calculate basic statistics for results
    pub fn calculate_stats(results: &QueryResponse) -> DataStats {
        DataStats {
            total_rows: results.rows.len(),
            total_columns: results.columns.len(),
            memory_size: Self::estimate_memory_size(results),
        }
    }
    
    fn estimate_memory_size(results: &QueryResponse) -> usize {
        let mut size = 0;
        
        // Column headers
        for col in &results.columns {
            size += col.len();
        }
        
        // Row data
        for row in &results.rows {
            for cell in row {
                size += cell.len();
            }
        }
        
        size
    }
}

#[derive(Debug, Clone)]
pub struct DataStats {
    pub total_rows: usize,
    pub total_columns: usize,
    pub memory_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_results() -> QueryResponse {
        QueryResponse {
            columns: vec!["id".to_string(), "name".to_string(), "value".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string(), "100".to_string()],
                vec!["2".to_string(), "Bob".to_string(), "200".to_string()],
                vec!["3".to_string(), "Charlie".to_string(), "300".to_string()],
            ],
        }
    }
    
    #[test]
    fn test_column_width_calculation() {
        let mut dm = DataManager::new();
        let results = create_test_results();
        
        let widths = dm.calculate_column_widths("test", &results, 100);
        
        assert_eq!(widths.len(), 3);
        assert!(widths[0] >= 2); // "id"
        assert!(widths[1] >= 7); // "Charlie"
        assert!(widths[2] >= 5); // "value"
    }
    
    #[test]
    fn test_filtering() {
        let mut dm = DataManager::new();
        let results = create_test_results();
        
        // Filter for "Alice"
        let indices = dm.apply_filter("test", &results, "Alice", false);
        assert_eq!(indices, vec![0]);
        
        // Case insensitive filter
        let indices = dm.apply_filter("test", &results, "alice", true);
        assert_eq!(indices, vec![0]);
        
        // Filter for "0" (appears in multiple rows)
        let indices = dm.apply_filter("test", &results, "0", false);
        assert_eq!(indices, vec![0, 1, 2]);
    }
    
    #[test]
    fn test_search() {
        let mut dm = DataManager::new();
        let results = create_test_results();
        
        // Search for "Bob"
        let matches = dm.search_in_results("test", &results, "Bob", false);
        assert_eq!(matches, vec![(1, 1)]);
        
        // Search for "00" (appears in values)
        let matches = dm.search_in_results("test", &results, "00", false);
        assert_eq!(matches, vec![(0, 2), (1, 2), (2, 2)]);
        
        // Test navigation
        dm.next_search_match("test");
        let state = dm.get_search_state("test").unwrap();
        assert_eq!(state.current_match, 1);
    }
    
    #[test]
    fn test_numeric_detection() {
        let results = create_test_results();
        
        assert!(!DataManager::is_numeric_column(&results, 1)); // "name" column
        assert!(DataManager::is_numeric_column(&results, 2)); // "value" column
    }
}