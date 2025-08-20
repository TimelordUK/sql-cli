/// Advanced CSV loader with string interning and memory optimization
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use anyhow::Result;
use csv;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// String interner for efficient memory usage with repeated strings
#[derive(Debug, Clone)]
pub struct StringInterner {
    strings: HashMap<String, Arc<String>>,
    usage_count: HashMap<Arc<String>, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            usage_count: HashMap::new(),
        }
    }

    /// Intern a string and return a reference-counted pointer
    pub fn intern(&mut self, s: &str) -> Arc<String> {
        if let Some(rc_str) = self.strings.get(s) {
            let rc = rc_str.clone();
            *self.usage_count.entry(rc.clone()).or_insert(0) += 1;
            rc
        } else {
            let rc_str = Arc::new(s.to_string());
            self.strings.insert(s.to_string(), rc_str.clone());
            self.usage_count.insert(rc_str.clone(), 1);
            rc_str
        }
    }

    /// Get statistics about interned strings
    pub fn stats(&self) -> InternerStats {
        let total_strings = self.strings.len();
        let total_references: usize = self.usage_count.values().sum();
        let memory_saved = self.calculate_memory_saved();

        InternerStats {
            unique_strings: total_strings,
            total_references,
            memory_saved_bytes: memory_saved,
        }
    }

    fn calculate_memory_saved(&self) -> usize {
        let mut saved = 0;
        for (rc_str, count) in &self.usage_count {
            if *count > 1 {
                // Each additional reference saves the string size
                saved += rc_str.len() * (*count - 1);
            }
        }
        saved
    }
}

#[derive(Debug)]
pub struct InternerStats {
    pub unique_strings: usize,
    pub total_references: usize,
    pub memory_saved_bytes: usize,
}

/// Column analysis results for determining interning strategy
#[derive(Debug)]
struct ColumnAnalysis {
    index: usize,
    name: String,
    cardinality: usize,
    sample_size: usize,
    unique_ratio: f64,
    is_categorical: bool,
    avg_string_length: usize,
}

pub struct AdvancedCsvLoader {
    sample_size: usize,
    cardinality_threshold: f64, // Ratio threshold for considering a column categorical
    interners: HashMap<usize, StringInterner>, // Column index -> interner
}

impl AdvancedCsvLoader {
    pub fn new() -> Self {
        Self {
            sample_size: 1000,          // Sample first 1000 rows for analysis
            cardinality_threshold: 0.5, // If < 50% unique values, consider categorical
            interners: HashMap::new(),
        }
    }

    /// Analyze columns to determine which should use string interning
    fn analyze_columns(&mut self, path: &Path) -> Result<Vec<ColumnAnalysis>> {
        info!("Analyzing CSV columns for optimization strategies");

        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        let headers = reader.headers()?.clone();

        // Initialize tracking for each column
        let num_columns = headers.len();
        let mut unique_values: Vec<HashSet<String>> = vec![HashSet::new(); num_columns];
        let mut total_lengths: Vec<usize> = vec![0; num_columns];
        let mut string_counts: Vec<usize> = vec![0; num_columns];

        // Sample rows to analyze cardinality
        let mut row_count = 0;
        for result in reader.records() {
            if row_count >= self.sample_size {
                break;
            }

            let record = result?;
            for (col_idx, field) in record.iter().enumerate() {
                if col_idx < num_columns {
                    // Only track non-numeric strings
                    if !field.is_empty() && field.parse::<f64>().is_err() {
                        unique_values[col_idx].insert(field.to_string());
                        total_lengths[col_idx] += field.len();
                        string_counts[col_idx] += 1;
                    }
                }
            }
            row_count += 1;
        }

        // Build analysis for each column
        let mut analyses = Vec::new();
        for (idx, header) in headers.iter().enumerate() {
            let cardinality = unique_values[idx].len();
            let unique_ratio = if row_count > 0 {
                cardinality as f64 / row_count as f64
            } else {
                1.0
            };

            let avg_length = if string_counts[idx] > 0 {
                total_lengths[idx] / string_counts[idx]
            } else {
                0
            };

            // Consider categorical if low cardinality or common patterns
            let is_categorical = unique_ratio < self.cardinality_threshold
                || Self::is_likely_categorical(&header, cardinality, avg_length);

            analyses.push(ColumnAnalysis {
                index: idx,
                name: header.to_string(),
                cardinality,
                sample_size: row_count,
                unique_ratio,
                is_categorical,
                avg_string_length: avg_length,
            });

            if is_categorical {
                debug!(
                    "Column '{}' marked for interning: {} unique values in {} samples (ratio: {:.2})",
                    header, cardinality, row_count, unique_ratio
                );
                self.interners.insert(idx, StringInterner::new());
            }
        }

        Ok(analyses)
    }

    /// Heuristic to identify likely categorical columns by name and characteristics
    fn is_likely_categorical(name: &str, cardinality: usize, avg_length: usize) -> bool {
        let name_lower = name.to_lowercase();

        // Common categorical column patterns
        let categorical_patterns = [
            "status",
            "state",
            "type",
            "category",
            "class",
            "group",
            "country",
            "region",
            "city",
            "currency",
            "side",
            "book",
            "desk",
            "trader",
            "portfolio",
            "strategy",
            "exchange",
            "venue",
            "counterparty",
            "product",
            "instrument",
        ];

        for pattern in &categorical_patterns {
            if name_lower.contains(pattern) {
                return true;
            }
        }

        // Boolean-like columns
        if name_lower.starts_with("is_") || name_lower.starts_with("has_") {
            return true;
        }

        // Low cardinality with short strings often indicates categories
        cardinality < 100 && avg_length < 50
    }

    /// Load CSV with advanced optimizations
    pub fn load_csv_optimized<P: AsRef<Path>>(
        &mut self,
        path: P,
        table_name: &str,
    ) -> Result<DataTable> {
        let path = path.as_ref();
        info!(
            "Advanced CSV load: Loading {} with optimizations",
            path.display()
        );

        // Track memory before loading
        crate::utils::memory_tracker::track_memory("advanced_csv_start");

        // Analyze columns first
        let analyses = self.analyze_columns(path)?;
        let categorical_columns: HashSet<usize> = analyses
            .iter()
            .filter(|a| a.is_categorical)
            .map(|a| a.index)
            .collect();

        info!(
            "Column analysis complete: {} of {} columns will use string interning",
            categorical_columns.len(),
            analyses.len()
        );

        // Now load the actual data
        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        let headers = reader.headers()?.clone();

        let mut table = DataTable::new(table_name);
        for header in headers.iter() {
            table.add_column(DataColumn::new(header.to_string()));
        }

        crate::utils::memory_tracker::track_memory("advanced_csv_headers");

        // Pre-allocate with estimated capacity if possible
        let file_size = std::fs::metadata(path)?.len();
        let estimated_rows = (file_size / 100) as usize; // Rough estimate
        table.reserve_rows(estimated_rows.min(1_000_000)); // Cap at 1M for safety

        // Read rows with optimizations
        let mut row_count = 0;
        for result in reader.records() {
            let record = result?;
            let mut values = Vec::with_capacity(headers.len());

            for (idx, field) in record.iter().enumerate() {
                let value = if field.is_empty() {
                    DataValue::Null
                } else if let Ok(b) = field.parse::<bool>() {
                    DataValue::Boolean(b)
                } else if let Ok(i) = field.parse::<i64>() {
                    DataValue::Integer(i)
                } else if let Ok(f) = field.parse::<f64>() {
                    DataValue::Float(f)
                } else {
                    // Check if this column should use interning
                    if categorical_columns.contains(&idx) {
                        if let Some(interner) = self.interners.get_mut(&idx) {
                            // Use interned string
                            DataValue::InternedString(interner.intern(field))
                        } else {
                            DataValue::String(field.to_string())
                        }
                    } else if field.contains('-') && field.len() >= 8 && field.len() <= 30 {
                        DataValue::DateTime(field.to_string())
                    } else {
                        DataValue::String(field.to_string())
                    }
                };
                values.push(value);
            }

            table
                .add_row(DataRow::new(values))
                .map_err(|e| anyhow::anyhow!(e))?;
            row_count += 1;

            // Track memory periodically
            if row_count % 10000 == 0 {
                crate::utils::memory_tracker::track_memory(&format!(
                    "advanced_csv_{}rows",
                    row_count
                ));
                debug!("Loaded {} rows...", row_count);
            }
        }

        // Shrink vectors to fit actual data
        table.shrink_to_fit();

        // Infer column types
        table.infer_column_types();

        crate::utils::memory_tracker::track_memory("advanced_csv_complete");

        // Report statistics
        let mut total_saved = 0;
        for (col_idx, interner) in &self.interners {
            let stats = interner.stats();
            if stats.memory_saved_bytes > 0 {
                debug!(
                    "Column {} ('{}'): {} unique strings, {} references, saved {} KB",
                    col_idx,
                    headers.get(*col_idx).unwrap_or(&"?"),
                    stats.unique_strings,
                    stats.total_references,
                    stats.memory_saved_bytes / 1024
                );
            }
            total_saved += stats.memory_saved_bytes;
        }

        info!(
            "Advanced CSV load complete: {} rows, {} columns, ~{} MB (saved {} KB via interning)",
            table.row_count(),
            table.column_count(),
            table.estimate_memory_size() / 1024 / 1024,
            total_saved / 1024
        );

        Ok(table)
    }

    /// Get interner statistics for debugging
    pub fn get_interner_stats(&self) -> HashMap<usize, InternerStats> {
        self.interners
            .iter()
            .map(|(idx, interner)| (*idx, interner.stats()))
            .collect()
    }
}

impl Default for AdvancedCsvLoader {
    fn default() -> Self {
        Self::new()
    }
}
