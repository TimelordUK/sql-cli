use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

// Compile regex patterns once and reuse them
static DATE_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_date_patterns() -> &'static Vec<Regex> {
    DATE_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap(), // YYYY-MM-DD
            Regex::new(r"^\d{2}/\d{2}/\d{4}").unwrap(), // MM/DD/YYYY
            Regex::new(r"^\d{2}-\d{2}-\d{4}").unwrap(), // DD-MM-YYYY
            Regex::new(r"^\d{4}/\d{2}/\d{2}").unwrap(), // YYYY/MM/DD
        ]
    })
}

/// Analyzes data for statistics, column widths, and other metrics
/// Extracted from the monolithic enhanced_tui.rs
pub struct DataAnalyzer {
    /// Cached column statistics
    column_stats: HashMap<String, ColumnStatistics>,

    /// Cached optimal column widths
    column_widths: Vec<usize>,
}

/// Statistics for a single column
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub column_name: String,
    pub data_type: ColumnType,
    pub total_values: usize,
    pub non_null_values: usize,
    pub null_values: usize,
    pub unique_values: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub avg_value: Option<f64>,
    pub sum_value: Option<f64>,
    pub median_value: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub frequency_map: Option<std::collections::BTreeMap<String, usize>>,
}

/// Detected column type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColumnType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    Mixed,
    Unknown,
}

impl DataAnalyzer {
    pub fn new() -> Self {
        Self {
            column_stats: HashMap::new(),
            column_widths: Vec::new(),
        }
    }

    /// Calculate statistics for a specific column
    pub fn calculate_column_statistics(
        &mut self,
        column_name: &str,
        values: &[&str],
    ) -> ColumnStatistics {
        let mut stats = ColumnStatistics {
            column_name: column_name.to_string(),
            data_type: ColumnType::Unknown,
            total_values: values.len(),
            non_null_values: 0,
            null_values: 0,
            unique_values: 0,
            min_value: None,
            max_value: None,
            avg_value: None,
            sum_value: None,
            median_value: None,
            min_length: None,
            max_length: None,
            frequency_map: None,
        };

        if values.is_empty() {
            return stats;
        }

        // Collect unique values more efficiently - use references when possible
        let mut unique = std::collections::HashSet::new();
        let mut numeric_values = Vec::new();
        let mut min_str: Option<&str> = None;
        let mut max_str: Option<&str> = None;
        let mut lengths = Vec::new();

        for value in values {
            if value.is_empty() {
                stats.null_values += 1;
            } else {
                stats.non_null_values += 1;

                // Use string reference for unique count
                unique.insert(*value);
                lengths.push(value.len());

                // Track min/max strings without cloning
                match min_str {
                    None => min_str = Some(value),
                    Some(min) if value < &min => min_str = Some(value),
                    _ => {}
                }
                match max_str {
                    None => max_str = Some(value),
                    Some(max) if value > &max => max_str = Some(value),
                    _ => {}
                }

                // Try to parse as number
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                }
            }
        }

        stats.unique_values = unique.len();

        // Determine data type
        stats.data_type = self.detect_column_type(values);

        // Calculate type-specific statistics
        match stats.data_type {
            ColumnType::Integer | ColumnType::Float => {
                if !numeric_values.is_empty() {
                    let sum: f64 = numeric_values.iter().sum();
                    stats.sum_value = Some(sum);
                    stats.avg_value = Some(sum / numeric_values.len() as f64);

                    // Calculate median
                    numeric_values
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let mid = numeric_values.len() / 2;
                    stats.median_value = if numeric_values.len() % 2 == 0 {
                        Some((numeric_values[mid - 1] + numeric_values[mid]) / 2.0)
                    } else {
                        Some(numeric_values[mid])
                    };

                    let min = numeric_values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = numeric_values
                        .iter()
                        .cloned()
                        .fold(f64::NEG_INFINITY, f64::max);
                    stats.min_value = Some(min.to_string());
                    stats.max_value = Some(max.to_string());
                }
            }
            _ => {
                // String statistics - use the min/max we already found without cloning
                stats.min_value = min_str.map(|s| s.to_string());
                stats.max_value = max_str.map(|s| s.to_string());
            }
        }

        // Build frequency map for columns with reasonable unique count
        const MAX_UNIQUE_FOR_FREQUENCY: usize = 100;
        if stats.unique_values <= MAX_UNIQUE_FOR_FREQUENCY && stats.unique_values > 0 {
            let mut freq_map = std::collections::BTreeMap::new();
            for value in values {
                if !value.is_empty() {
                    *freq_map.entry(value.to_string()).or_insert(0) += 1;
                }
            }
            stats.frequency_map = Some(freq_map);
        }

        // Length statistics
        if !lengths.is_empty() {
            stats.min_length = lengths.iter().min().copied();
            stats.max_length = lengths.iter().max().copied();
        }

        // Cache the statistics
        self.column_stats
            .insert(column_name.to_string(), stats.clone());

        stats
    }

    /// Detect the type of a column based on its values
    pub fn detect_column_type(&self, values: &[&str]) -> ColumnType {
        if values.is_empty() {
            return ColumnType::Unknown;
        }

        let mut type_counts = HashMap::new();

        // Early exit optimization: check first few values
        // If they're all the same type, check if the rest matches
        let first_type = self.detect_single_value_type(values[0]);
        let mut all_same = true;

        for (i, value) in values.iter().filter(|v| !v.is_empty()).enumerate() {
            let detected_type = self.detect_single_value_type(value);

            if i < 10 && detected_type != first_type {
                all_same = false;
            }

            *type_counts.entry(detected_type).or_insert(0) += 1;

            // Early exit if we have enough samples and they're mixed
            if i > 100 && type_counts.len() > 1 && !all_same {
                break;
            }
        }

        // If we have multiple types, it's mixed
        if type_counts.len() > 1 {
            // But if >90% are one type, use that type
            let total: usize = type_counts.values().sum();
            for (col_type, count) in type_counts.iter() {
                if *count as f64 / total as f64 > 0.9 {
                    return col_type.clone();
                }
            }
            ColumnType::Mixed
        } else if let Some((col_type, _)) = type_counts.into_iter().next() {
            col_type
        } else {
            ColumnType::Unknown
        }
    }

    /// Detect type of a single value
    fn detect_single_value_type(&self, value: &str) -> ColumnType {
        if value.is_empty() {
            return ColumnType::Unknown;
        }

        // Check in order of likelihood/performance
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            ColumnType::Boolean
        } else if value.parse::<i64>().is_ok() {
            ColumnType::Integer
        } else if value.parse::<f64>().is_ok() {
            ColumnType::Float
        } else if self.looks_like_date_fast(value) {
            ColumnType::Date
        } else {
            ColumnType::String
        }
    }

    /// Check if a string looks like a date using pre-compiled regex patterns
    fn looks_like_date_fast(&self, value: &str) -> bool {
        // Simple heuristics for date detection
        if value.len() < 8 || value.len() > 30 {
            return false;
        }

        // Use pre-compiled regex patterns
        let patterns = get_date_patterns();
        for pattern in patterns {
            if pattern.is_match(value) {
                return true;
            }
        }

        false
    }

    /// Check if a string looks like a date
    fn looks_like_date(&self, value: &str) -> bool {
        // Simple heuristics for date detection
        if value.len() < 8 || value.len() > 30 {
            return false;
        }

        // Check for common date patterns
        let date_patterns = [
            r"\d{4}-\d{2}-\d{2}", // YYYY-MM-DD
            r"\d{2}/\d{2}/\d{4}", // MM/DD/YYYY
            r"\d{2}-\d{2}-\d{4}", // DD-MM-YYYY
            r"\d{4}/\d{2}/\d{2}", // YYYY/MM/DD
        ];

        for pattern in &date_patterns {
            if regex::Regex::new(pattern)
                .map(|re| re.is_match(value))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Calculate optimal column widths for display
    pub fn calculate_optimal_column_widths(
        &mut self,
        data: &[Value],
        max_sample_rows: usize,
    ) -> Vec<usize> {
        if data.is_empty() {
            return Vec::new();
        }

        // Get headers from first row
        let headers: Vec<String> = if let Some(first_row) = data.first() {
            if let Some(obj) = first_row.as_object() {
                obj.keys().map(|k| k.to_string()).collect()
            } else {
                return Vec::new();
            }
        } else {
            return Vec::new();
        };

        let mut widths = vec![0; headers.len()];

        // Start with header widths
        for (i, header) in headers.iter().enumerate() {
            widths[i] = header.len();
        }

        // Sample rows for width calculation
        let total_rows = data.len();
        let rows_to_check: Vec<usize> = if total_rows <= max_sample_rows {
            (0..total_rows).collect()
        } else {
            // Sample evenly distributed rows
            let step = total_rows / max_sample_rows;
            (0..max_sample_rows)
                .map(|i| (i * step).min(total_rows - 1))
                .collect()
        };

        // Check sampled rows
        for &row_idx in &rows_to_check {
            if let Some(row) = data.get(row_idx) {
                if let Some(obj) = row.as_object() {
                    for (i, header) in headers.iter().enumerate() {
                        if let Some(value) = obj.get(header) {
                            let display_len = self.get_value_display_length(value);
                            widths[i] = widths[i].max(display_len);
                        }
                    }
                }
            }
        }

        // Apply constraints
        for width in &mut widths {
            *width = (*width).min(50).max(3); // Min 3, max 50 characters
        }

        self.column_widths = widths.clone();
        widths
    }

    /// Get display length of a JSON value
    fn get_value_display_length(&self, value: &Value) -> usize {
        match value {
            Value::String(s) => s.len(),
            Value::Number(n) => n.to_string().len(),
            Value::Bool(b) => b.to_string().len(),
            Value::Null => 4, // "null"
            Value::Array(a) => format!("[{} items]", a.len()).len(),
            Value::Object(o) => format!("{{{} fields}}", o.len()).len(),
        }
    }

    /// Get cached column statistics
    pub fn get_column_statistics(&self, column_name: &str) -> Option<&ColumnStatistics> {
        self.column_stats.get(column_name)
    }

    /// Get cached column widths
    pub fn get_column_widths(&self) -> &[usize] {
        &self.column_widths
    }

    /// Clear all cached data
    pub fn clear_cache(&mut self) {
        self.column_stats.clear();
        self.column_widths.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_column_type_detection() {
        let analyzer = DataAnalyzer::new();

        // Integer column
        let int_values = vec!["1", "2", "3", "4", "5"];
        assert_eq!(
            analyzer.detect_column_type(&int_values),
            ColumnType::Integer
        );

        // Float column
        let float_values = vec!["1.5", "2.7", "3.14", "4.0", "5.5"];
        assert_eq!(
            analyzer.detect_column_type(&float_values),
            ColumnType::Float
        );

        // String column
        let string_values = vec!["alice", "bob", "charlie", "david"];
        assert_eq!(
            analyzer.detect_column_type(&string_values),
            ColumnType::String
        );

        // Boolean column
        let bool_values = vec!["true", "false", "TRUE", "FALSE"];
        assert_eq!(
            analyzer.detect_column_type(&bool_values),
            ColumnType::Boolean
        );
    }

    #[test]
    fn test_column_statistics() {
        let mut analyzer = DataAnalyzer::new();

        let values = vec!["10", "20", "30", "40", "50", ""];

        let stats = analyzer.calculate_column_statistics("test_column", &values);

        assert_eq!(stats.total_values, 6);
        assert_eq!(stats.non_null_values, 5);
        assert_eq!(stats.null_values, 1);
        assert_eq!(stats.unique_values, 5);
        assert_eq!(stats.data_type, ColumnType::Integer);
        assert_eq!(stats.avg_value, Some(30.0));
        assert_eq!(stats.sum_value, Some(150.0));
        assert_eq!(stats.median_value, Some(30.0));
        assert_eq!(stats.min_value, Some("10".to_string()));
        assert_eq!(stats.max_value, Some("50".to_string()));
        assert!(stats.frequency_map.is_some());
    }

    #[test]
    fn test_optimal_column_widths() {
        let mut analyzer = DataAnalyzer::new();

        let data = vec![
            json!({"name": "Alice", "age": 30, "city": "New York"}),
            json!({"name": "Bob", "age": 25, "city": "Los Angeles"}),
            json!({"name": "Charlie", "age": 35, "city": "SF"}),
        ];

        let widths = analyzer.calculate_optimal_column_widths(&data, 100);

        assert_eq!(widths.len(), 3);
        // The keys are sorted alphabetically: age, city, name
        // So widths[0] is for "age", widths[1] is for "city", widths[2] is for "name"
        assert!(widths[2] >= 7); // "Charlie" is 7 chars (name column)
        assert!(widths[0] >= 3); // "age" header is 3 chars
        assert!(widths[1] >= 11); // "Los Angeles" is 11 chars (city column)
    }
}
