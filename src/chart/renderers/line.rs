use crate::chart::types::*;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
    Frame,
};

pub struct LineRenderer {
    viewport: ChartViewport,
}

impl LineRenderer {
    pub fn new(viewport: ChartViewport) -> Self {
        Self { viewport }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, data: &DataSeries, config: &ChartConfig) {
        // Convert data points to ratatui format
        let chart_data: Vec<(f64, f64)> = data
            .points
            .iter()
            .filter(|p| {
                p.x >= self.viewport.x_min
                    && p.x <= self.viewport.x_max
                    && p.y >= self.viewport.y_min
                    && p.y <= self.viewport.y_max
            })
            .map(|p| (p.x, p.y))
            .collect();

        let datasets = self.create_datasets(data, &chart_data);
        let (x_axis, y_axis) = self.create_axes(data, config);

        let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);

        frame.render_widget(chart, area);
    }

    fn create_datasets<'a>(
        &self,
        data: &DataSeries,
        chart_data: &'a [(f64, f64)],
    ) -> Vec<Dataset<'a>> {
        vec![Dataset::default()
            .name(data.name.clone())
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(chart_data)]
    }

    fn create_axes(&self, data: &DataSeries, config: &ChartConfig) -> (Axis, Axis) {
        // Create X-axis
        let x_axis = Axis::default()
            .title(config.x_axis.clone())
            .style(Style::default().fg(Color::Gray))
            .bounds([self.viewport.x_min, self.viewport.x_max])
            .labels(self.create_x_labels(data));

        // Create Y-axis with smart scaling
        let y_axis = Axis::default()
            .title(config.y_axis.clone())
            .style(Style::default().fg(Color::Gray))
            .bounds([self.viewport.y_min, self.viewport.y_max])
            .labels(self.create_y_labels());

        (x_axis, y_axis)
    }

    fn create_x_labels(&self, data: &DataSeries) -> Vec<String> {
        // Check if we have timestamps
        let has_timestamps = data.points.iter().any(|p| p.timestamp.is_some());

        if has_timestamps {
            self.create_time_labels()
        } else {
            self.create_numeric_labels(self.viewport.x_min, self.viewport.x_max)
        }
    }

    fn create_time_labels(&self) -> Vec<String> {
        let num_labels = 5;
        let x_span = self.viewport.x_max - self.viewport.x_min;
        let step = x_span / (num_labels as f64 - 1.0);

        (0..num_labels)
            .map(|i| {
                let timestamp_secs = self.viewport.x_min + (i as f64) * step;
                let dt = DateTime::<Utc>::from_timestamp(timestamp_secs as i64, 0)
                    .unwrap_or_else(|| Utc::now());
                format!("{}", dt.format("%H:%M:%S"))
            })
            .collect()
    }

    fn create_numeric_labels(&self, min: f64, max: f64) -> Vec<String> {
        let num_labels = 5;
        let step = (max - min) / (num_labels as f64 - 1.0);

        (0..num_labels)
            .map(|i| {
                let value = min + (i as f64) * step;
                if value.abs() > 1000.0 {
                    format!("{:.0}k", value / 1000.0)
                } else if value.abs() < 1.0 {
                    format!("{:.3}", value)
                } else {
                    format!("{:.1}", value)
                }
            })
            .collect()
    }

    fn create_y_labels(&self) -> Vec<String> {
        self.create_numeric_labels(self.viewport.y_min, self.viewport.y_max)
    }

    pub fn viewport_mut(&mut self) -> &mut ChartViewport {
        &mut self.viewport
    }

    pub fn viewport(&self) -> &ChartViewport {
        &self.viewport
    }
}
