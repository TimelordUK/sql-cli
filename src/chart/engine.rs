use crate::chart::types::*;
use crate::data::data_view::DataView;
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};

pub struct ChartEngine {
    data_view: DataView,
}

impl ChartEngine {
    pub fn new(data_view: DataView) -> Self {
        Self { data_view }
    }

    pub fn execute_chart_query(&mut self, config: &ChartConfig) -> Result<DataSeries> {
        // The SQL query has already been executed in chart_main.rs
        // Just extract chart data from the already-filtered data_view
        self.extract_chart_data(&self.data_view, config)
    }

    fn extract_chart_data(&self, data: &DataView, config: &ChartConfig) -> Result<DataSeries> {
        let headers = data.column_names();

        // Find column indices
        let x_col_idx = headers
            .iter()
            .position(|h| h == &config.x_axis)
            .ok_or_else(|| anyhow!("X-axis column '{}' not found", config.x_axis))?;
        let y_col_idx = headers
            .iter()
            .position(|h| h == &config.y_axis)
            .ok_or_else(|| anyhow!("Y-axis column '{}' not found", config.y_axis))?;

        let mut points = Vec::new();
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        // Convert data to chart points
        for row_idx in 0..data.row_count() {
            let x_value_str = data.get_cell_value(row_idx, x_col_idx);
            let y_value_str = data.get_cell_value(row_idx, y_col_idx);

            if let (Some(x_str), Some(y_str)) = (x_value_str, y_value_str) {
                let (x_float, timestamp) = self.convert_str_to_float_with_time(&x_str)?;
                let y_float = self.convert_str_to_float(&y_str)?;

                // Skip invalid data points
                if x_float.is_finite() && y_float.is_finite() {
                    points.push(DataPoint {
                        x: x_float,
                        y: y_float,
                        timestamp,
                        label: None,
                    });

                    x_min = x_min.min(x_float);
                    x_max = x_max.max(x_float);
                    y_min = y_min.min(y_float);
                    y_max = y_max.max(y_float);
                }
            }
        }

        if points.is_empty() {
            return Err(anyhow!("No valid data points found"));
        }

        Ok(DataSeries {
            name: format!("{} vs {}", config.y_axis, config.x_axis),
            points,
            x_range: (x_min, x_max),
            y_range: (y_min, y_max),
        })
    }

    fn convert_str_to_float_with_time(&self, s: &str) -> Result<(f64, Option<DateTime<Utc>>)> {
        // Try to parse as timestamp first - handle multiple formats
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            let utc_dt = dt.with_timezone(&Utc);
            // Convert to seconds since epoch for plotting
            let timestamp_secs = utc_dt.timestamp() as f64;
            Ok((timestamp_secs, Some(utc_dt)))
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
            // Handle format like "2025-08-12T09:00:00" (no timezone)
            let utc_dt = dt.and_utc();
            let timestamp_secs = utc_dt.timestamp() as f64;
            Ok((timestamp_secs, Some(utc_dt)))
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%f") {
            // Handle format like "2025-08-12T09:00:18.030000" (with microseconds, no timezone)
            let utc_dt = dt.and_utc();
            let timestamp_secs = utc_dt.timestamp() as f64;
            Ok((timestamp_secs, Some(utc_dt)))
        } else if let Ok(f) = s.parse::<f64>() {
            Ok((f, None))
        } else {
            Err(anyhow!("Cannot convert '{}' to numeric value", s))
        }
    }

    fn convert_str_to_float(&self, s: &str) -> Result<f64> {
        s.parse::<f64>()
            .map_err(|_| anyhow!("Cannot convert '{}' to numeric value", s))
    }
}
