// SQL Refactoring Tools for IDE Integration
// Provides programmatic transformations of SQL queries

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod banding;
pub mod conditional_agg;
pub mod extraction;

pub use banding::{CaseCondition, CaseGenerator, CaseStyle};

#[derive(Debug, Serialize, Deserialize)]
pub struct RefactoringResult {
    pub original: String,
    pub transformed: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BandingConfig {
    pub column: String,
    pub bands: Vec<Band>,
    pub else_label: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Band {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub label: String,
}

impl BandingConfig {
    /// Parse bands from string format like "0-10,11-20,21-30,30+"
    pub fn from_string(column: &str, bands_str: &str) -> Result<Self> {
        let mut bands = Vec::new();

        for band_str in bands_str.split(',') {
            let band_str = band_str.trim();

            if band_str.ends_with('+') {
                // Handle "30+" format
                let min_str = band_str.trim_end_matches('+');
                let min = min_str.parse::<f64>()?;
                bands.push(Band {
                    min: Some(min),
                    max: None,
                    label: band_str.to_string(),
                });
            } else if band_str.contains('-') {
                // Handle "0-10" format
                let parts: Vec<&str> = band_str.split('-').collect();
                if parts.len() == 2 {
                    let min = parts[0].parse::<f64>()?;
                    let max = parts[1].parse::<f64>()?;
                    bands.push(Band {
                        min: Some(min),
                        max: Some(max),
                        label: band_str.to_string(),
                    });
                }
            }
        }

        Ok(BandingConfig {
            column: column.to_string(),
            bands,
            else_label: None,
            alias: Some(format!("{}_band", column)),
        })
    }

    /// Generate SQL CASE statement for banding
    pub fn to_sql(&self) -> String {
        let mut sql = String::from("CASE");

        for band in &self.bands {
            sql.push('\n');
            if let Some(min) = band.min {
                if let Some(max) = band.max {
                    if band.min == Some(0.0) || self.bands.iter().position(|b| b == band) == Some(0)
                    {
                        sql.push_str(&format!(
                            "    WHEN {} <= {} THEN '{}'",
                            self.column, max, band.label
                        ));
                    } else {
                        sql.push_str(&format!(
                            "    WHEN {} > {} AND {} <= {} THEN '{}'",
                            self.column, min, self.column, max, band.label
                        ));
                    }
                } else {
                    sql.push_str(&format!(
                        "    WHEN {} > {} THEN '{}'",
                        self.column, min, band.label
                    ));
                }
            } else if let Some(max) = band.max {
                sql.push_str(&format!(
                    "    WHEN {} <= {} THEN '{}'",
                    self.column, max, band.label
                ));
            }
        }

        if let Some(else_label) = &self.else_label {
            sql.push_str(&format!("\n    ELSE '{}'", else_label));
        }

        sql.push_str("\nEND");

        if let Some(alias) = &self.alias {
            sql.push_str(&format!(" AS {}", alias));
        }

        sql
    }
}

/// Generate automatic bands for a numeric range
pub fn generate_auto_bands(min: f64, max: f64, num_buckets: usize) -> Vec<Band> {
    let range = max - min;
    let bucket_size = range / num_buckets as f64;

    let mut bands = Vec::new();

    for i in 0..num_buckets {
        let band_min = min + (i as f64 * bucket_size);
        let band_max = if i == num_buckets - 1 {
            max
        } else {
            min + ((i + 1) as f64 * bucket_size)
        };

        let label = if i == num_buckets - 1 {
            format!("{:.0}+", band_min)
        } else {
            format!("{:.0}-{:.0}", band_min, band_max)
        };

        bands.push(Band {
            min: if i == 0 { None } else { Some(band_min) },
            max: if i == num_buckets - 1 {
                None
            } else {
                Some(band_max)
            },
            label,
        });
    }

    bands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banding_generation() {
        let config = BandingConfig::from_string("age", "0-10,11-20,21-30,30+").unwrap();
        let sql = config.to_sql();

        assert!(sql.contains("WHEN age <= 10 THEN '0-10'"));
        assert!(sql.contains("WHEN age > 30 THEN '30+'"));
        assert!(sql.contains("AS age_band"));
    }

    #[test]
    fn test_auto_bands() {
        let bands = generate_auto_bands(0.0, 100.0, 4);
        assert_eq!(bands.len(), 4);
        assert_eq!(bands[0].label, "0-25");
        assert_eq!(bands[3].label, "75+");
    }
}
