use serde_json::Value;
use std::collections::HashMap;

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
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
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
        values: &[String],
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
            min_length: None,
            max_length: None,
        };

        if values.is_empty() {
            return stats;
        }

        // Collect unique values
        let mut unique = std::collections::HashSet::new();
        let mut numeric_values = Vec::new();
        let mut string_values = Vec::new();
        let mut lengths = Vec::new();

        for value in values {
            if value.is_empty() {
                stats.null_values += 1;
            } else {
                stats.non_null_values += 1;
                unique.insert(value.clone());
                lengths.push(value.len());

                // Try to parse as number
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                }
                string_values.push(value.clone());
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
                    stats.avg_value = Some(sum / numeric_values.len() as f64);

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
                // String statistics
                if !string_values.is_empty() {
                    string_values.sort();
                    stats.min_value = string_values.first().cloned();
                    stats.max_value = string_values.last().cloned();
                }
            }
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
    pub fn detect_column_type(&self, values: &[String]) -> ColumnType {
        if values.is_empty() {
            return ColumnType::Unknown;
        }

        let mut type_counts = HashMap::new();

        for value in values.iter().filter(|v| !v.is_empty()) {
            let detected_type = if value.parse::<i64>().is_ok() {
                ColumnType::Integer
            } else if value.parse::<f64>().is_ok() {
                ColumnType::Float
            } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
                ColumnType::Boolean
            } else if self.looks_like_date(value) {
                ColumnType::Date
            } else {
                ColumnType::String
            };

            *type_counts.entry(detected_type).or_insert(0) += 1;
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
        let int_values = vec!["1", "2", "3", "4", "5"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(
            analyzer.detect_column_type(&int_values),
            ColumnType::Integer
        );

        // Float column
        let float_values = vec!["1.5", "2.7", "3.14", "4.0", "5.5"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(
            analyzer.detect_column_type(&float_values),
            ColumnType::Float
        );

        // String column
        let string_values = vec!["alice", "bob", "charlie", "david"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(
            analyzer.detect_column_type(&string_values),
            ColumnType::String
        );

        // Boolean column
        let bool_values = vec!["true", "false", "TRUE", "FALSE"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(
            analyzer.detect_column_type(&bool_values),
            ColumnType::Boolean
        );
    }

    #[test]
    fn test_column_statistics() {
        let mut analyzer = DataAnalyzer::new();

        let values = vec!["10", "20", "30", "40", "50", ""]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let stats = analyzer.calculate_column_statistics("test_column", &values);

        assert_eq!(stats.total_values, 6);
        assert_eq!(stats.non_null_values, 5);
        assert_eq!(stats.null_values, 1);
        assert_eq!(stats.unique_values, 5);
        assert_eq!(stats.data_type, ColumnType::Integer);
        assert_eq!(stats.avg_value, Some(30.0));
        assert_eq!(stats.min_value, Some("10".to_string()));
        assert_eq!(stats.max_value, Some("50".to_string()));
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
