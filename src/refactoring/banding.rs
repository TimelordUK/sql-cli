// Banding and bucketing refactoring tools

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use super::{Band, BandingConfig};

#[derive(Debug, Serialize, Deserialize)]
pub enum CaseStyle {
    Range,  // Numeric ranges like 0-10, 11-20
    Values, // Specific values like 'A', 'B', 'C'
    Custom, // Custom conditions
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseGenerator {
    pub column: String,
    pub style: CaseStyle,
    pub conditions: Vec<CaseCondition>,
    pub else_clause: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseCondition {
    pub condition: String,
    pub label: String,
}

impl CaseGenerator {
    /// Create a generator from range specifications
    pub fn from_ranges(column: &str, ranges: &str, labels: Option<Vec<String>>) -> Result<Self> {
        let config = BandingConfig::from_string(column, ranges)?;
        let mut conditions = Vec::new();

        for (i, band) in config.bands.iter().enumerate() {
            let label = if let Some(ref labels) = labels {
                labels.get(i).unwrap_or(&band.label).clone()
            } else {
                band.label.clone()
            };

            let condition = if let Some(min) = band.min {
                if let Some(max) = band.max {
                    if i == 0 {
                        format!("{} <= {}", column, max)
                    } else {
                        format!("{} > {} AND {} <= {}", column, min, column, max)
                    }
                } else {
                    format!("{} > {}", column, min)
                }
            } else if let Some(max) = band.max {
                format!("{} <= {}", column, max)
            } else {
                continue;
            };

            conditions.push(CaseCondition { condition, label });
        }

        Ok(CaseGenerator {
            column: column.to_string(),
            style: CaseStyle::Range,
            conditions,
            else_clause: config.else_label,
            alias: config.alias,
        })
    }

    /// Create a generator from specific values
    pub fn from_values(column: &str, value_mappings: Vec<(String, String)>) -> Self {
        let conditions = value_mappings
            .into_iter()
            .map(|(value, label)| CaseCondition {
                condition: format!("{} = '{}'", column, value),
                label,
            })
            .collect();

        CaseGenerator {
            column: column.to_string(),
            style: CaseStyle::Values,
            conditions,
            else_clause: Some("Other".to_string()),
            alias: Some(format!("{}_category", column)),
        }
    }

    /// Generate the SQL CASE statement
    pub fn to_sql(&self) -> String {
        let mut sql = String::from("CASE");

        for condition in &self.conditions {
            sql.push_str(&format!(
                "\n    WHEN {} THEN '{}'",
                condition.condition, condition.label
            ));
        }

        if let Some(ref else_clause) = self.else_clause {
            sql.push_str(&format!("\n    ELSE '{}'", else_clause));
        }

        sql.push_str("\nEND");

        if let Some(ref alias) = self.alias {
            sql.push_str(&format!(" AS {}", alias));
        }

        sql
    }

    /// Generate a pretty-printed version with indentation
    pub fn to_sql_formatted(&self, indent: usize) -> String {
        let indent_str = " ".repeat(indent);
        let mut sql = format!("{}CASE", indent_str);

        for condition in &self.conditions {
            sql.push_str(&format!(
                "\n{}    WHEN {} THEN '{}'",
                indent_str, condition.condition, condition.label
            ));
        }

        if let Some(ref else_clause) = self.else_clause {
            sql.push_str(&format!("\n{}    ELSE '{}'", indent_str, else_clause));
        }

        sql.push_str(&format!("\n{}END", indent_str));

        if let Some(ref alias) = self.alias {
            sql.push_str(&format!(" AS {}", alias));
        }

        sql
    }
}

impl BandingConfig {
    /// Generate percentile-based bands
    pub fn from_percentiles(column: &str, percentiles: Vec<f64>) -> Self {
        let mut bands = Vec::new();

        for i in 0..percentiles.len() {
            let label = if i == 0 {
                format!("P0-P{}", (percentiles[i] * 100.0) as i32)
            } else if i == percentiles.len() - 1 {
                format!("P{}+", (percentiles[i - 1] * 100.0) as i32)
            } else {
                format!(
                    "P{}-P{}",
                    (percentiles[i - 1] * 100.0) as i32,
                    (percentiles[i] * 100.0) as i32
                )
            };

            bands.push(Band {
                min: if i == 0 {
                    None
                } else {
                    Some(percentiles[i - 1])
                },
                max: if i == percentiles.len() - 1 {
                    None
                } else {
                    Some(percentiles[i])
                },
                label,
            });
        }

        BandingConfig {
            column: column.to_string(),
            bands,
            else_label: None,
            alias: Some(format!("{}_percentile", column)),
        }
    }

    /// Generate equal-width bands
    pub fn from_equal_width(
        column: &str,
        num_bands: usize,
        min_val: f64,
        max_val: f64,
        labels: Option<Vec<String>>,
    ) -> Self {
        let width = (max_val - min_val) / num_bands as f64;
        let mut bands = Vec::new();

        for i in 0..num_bands {
            let band_min = min_val + (i as f64 * width);
            let band_max = if i == num_bands - 1 {
                max_val
            } else {
                min_val + ((i + 1) as f64 * width)
            };

            let label = if let Some(ref labels) = labels {
                labels.get(i).map(|s| s.clone()).unwrap_or_else(|| {
                    if i == num_bands - 1 {
                        format!("{:.0}+", band_min)
                    } else {
                        format!("{:.0}-{:.0}", band_min, band_max)
                    }
                })
            } else {
                if i == num_bands - 1 {
                    format!("{:.0}+", band_min)
                } else {
                    format!("{:.0}-{:.0}", band_min, band_max)
                }
            };

            bands.push(Band {
                min: if i == 0 { None } else { Some(band_min) },
                max: if i == num_bands - 1 {
                    None
                } else {
                    Some(band_max)
                },
                label,
            });
        }

        BandingConfig {
            column: column.to_string(),
            bands,
            else_label: None,
            alias: Some(format!("{}_band", column)),
        }
    }
}
