use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub title: String,
    pub x_axis: String,
    pub y_axis: String,
    pub chart_type: ChartType,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ChartType {
    #[default]
    Line,
    Scatter,
    Bar,
    Candlestick,
}

#[derive(Debug, Clone)]
pub struct DataSeries {
    pub name: String,
    pub points: Vec<DataPoint>,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChartViewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub zoom_level: f64,
}

impl ChartViewport {
    pub fn new(x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        Self {
            x_min: x_range.0,
            x_max: x_range.1,
            y_min: y_range.0,
            y_max: y_range.1,
            zoom_level: 1.0,
        }
    }

    pub fn auto_scale(&mut self, data: &DataSeries) {
        self.x_min = data.x_range.0;
        self.x_max = data.x_range.1;

        // Smart Y-axis scaling with padding
        let y_span = data.y_range.1 - data.y_range.0;
        let padding = y_span * 0.1; // 10% padding
        self.y_min = data.y_range.0 - padding;
        self.y_max = data.y_range.1 + padding;
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        let x_span = self.x_max - self.x_min;
        let y_span = self.y_max - self.y_min;

        self.x_min += dx * x_span * 0.1;
        self.x_max += dx * x_span * 0.1;
        self.y_min += dy * y_span * 0.1;
        self.y_max += dy * y_span * 0.1;
    }

    pub fn zoom(&mut self, factor: f64, center_x: f64, center_y: f64) {
        let x_span = self.x_max - self.x_min;
        let y_span = self.y_max - self.y_min;

        let new_x_span = x_span / factor;
        let new_y_span = y_span / factor;

        self.x_min = center_x - new_x_span / 2.0;
        self.x_max = center_x + new_x_span / 2.0;
        self.y_min = center_y - new_y_span / 2.0;
        self.y_max = center_y + new_y_span / 2.0;

        self.zoom_level *= factor;
    }
}
