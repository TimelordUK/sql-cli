use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use serde_json::Value;

/// Handles search and filter operations on data
pub struct SearchFilter;

impl SearchFilter {
    /// Perform a regex search on data and return matching positions
    pub fn perform_search(data: &[Vec<String>], pattern: &str) -> Result<Vec<(usize, usize)>> {
        let mut matches = Vec::new();
        let regex = Regex::new(pattern)?;

        for (row_idx, row) in data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if regex.is_match(cell) {
                    matches.push((row_idx, col_idx));
                }
            }
        }

        Ok(matches)
    }

    /// Apply a regex filter to JSON data
    pub fn apply_regex_filter(data: &[Value], pattern: &str) -> Result<Vec<Value>> {
        let regex = Regex::new(pattern)?;
        let mut filtered = Vec::new();

        for item in data {
            if let Some(obj) = item.as_object() {
                let mut matches = false;
                for (_key, value) in obj {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::from("null"),
                        _ => value.to_string(),
                    };

                    if regex.is_match(&value_str) {
                        matches = true;
                        break;
                    }
                }

                if matches {
                    filtered.push(item.clone());
                }
            }
        }

        Ok(filtered)
    }

    /// Apply fuzzy filter to data and return matching indices
    pub fn apply_fuzzy_filter(data: &[Value], pattern: &str, score_threshold: i64) -> Vec<usize> {
        let matcher = SkimMatcherV2::default();
        let mut filtered_indices = Vec::new();

        for (idx, item) in data.iter().enumerate() {
            if let Some(obj) = item.as_object() {
                let mut best_score = 0i64;

                for (_key, value) in obj {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::from("null"),
                        _ => value.to_string(),
                    };

                    if let Some(score) = matcher.fuzzy_match(&value_str, pattern) {
                        best_score = best_score.max(score);
                    }
                }

                if best_score > score_threshold {
                    filtered_indices.push(idx);
                }
            }
        }

        filtered_indices
    }

    /// Find columns matching a search pattern
    pub fn find_matching_columns(headers: &[&str], pattern: &str) -> Vec<(usize, String)> {
        let pattern_lower = pattern.to_lowercase();
        let mut matching = Vec::new();

        for (idx, &header) in headers.iter().enumerate() {
            if header.to_lowercase().contains(&pattern_lower) {
                matching.push((idx, header.to_string()));
            }
        }

        matching
    }

    /// Navigate to next search match
    pub fn next_match(matches: &[(usize, usize)], current_index: usize) -> Option<(usize, usize)> {
        if matches.is_empty() {
            return None;
        }

        let next_index = (current_index + 1) % matches.len();
        Some(matches[next_index])
    }

    /// Navigate to previous search match
    pub fn previous_match(
        matches: &[(usize, usize)],
        current_index: usize,
    ) -> Option<(usize, usize)> {
        if matches.is_empty() {
            return None;
        }

        let prev_index = if current_index == 0 {
            matches.len() - 1
        } else {
            current_index - 1
        };

        Some(matches[prev_index])
    }
}
