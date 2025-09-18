use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    pub query_parse_time: Duration,
    pub query_plan_time: Duration,
    pub query_execute_time: Duration,
    pub total_time: Duration,

    pub rows_processed: usize,
    pub rows_returned: usize,
    pub rows_per_second: f64,

    pub peak_memory_bytes: Option<usize>,
    pub allocations_count: Option<usize>,

    pub table_scans: usize,
    pub expressions_evaluated: usize,
    pub functions_called: usize,
}

impl BenchmarkMetrics {
    pub fn new() -> Self {
        BenchmarkMetrics {
            query_parse_time: Duration::ZERO,
            query_plan_time: Duration::ZERO,
            query_execute_time: Duration::ZERO,
            total_time: Duration::ZERO,
            rows_processed: 0,
            rows_returned: 0,
            rows_per_second: 0.0,
            peak_memory_bytes: None,
            allocations_count: None,
            table_scans: 0,
            expressions_evaluated: 0,
            functions_called: 0,
        }
    }

    pub fn calculate_throughput(&mut self) {
        if self.total_time.as_secs_f64() > 0.0 {
            self.rows_per_second = self.rows_processed as f64 / self.total_time.as_secs_f64();
        }
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{:.3},{:.3},{:.3},{:.3},{},{},{:.0}",
            self.query_parse_time.as_secs_f64() * 1000.0,
            self.query_plan_time.as_secs_f64() * 1000.0,
            self.query_execute_time.as_secs_f64() * 1000.0,
            self.total_time.as_secs_f64() * 1000.0,
            self.rows_processed,
            self.rows_returned,
            self.rows_per_second
        )
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub query_name: String,
    pub query_category: String,
    pub table_name: String,
    pub row_count: usize,
    pub metrics: BenchmarkMetrics,
    pub error: Option<String>,
}

impl BenchmarkResult {
    pub fn new(
        query_name: String,
        query_category: String,
        table_name: String,
        row_count: usize,
    ) -> Self {
        BenchmarkResult {
            query_name,
            query_category,
            table_name,
            row_count,
            metrics: BenchmarkMetrics::new(),
            error: None,
        }
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.query_name,
            self.table_name,
            self.row_count,
            self.metrics.to_csv_row(),
            self.error.as_ref().unwrap_or(&"OK".to_string())
        )
    }
}

pub struct MetricsCollector {
    start: Option<Instant>,
    phase_start: Option<Instant>,
    current_metrics: BenchmarkMetrics,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            start: None,
            phase_start: None,
            current_metrics: BenchmarkMetrics::new(),
        }
    }

    pub fn start_total(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn start_phase(&mut self) {
        self.phase_start = Some(Instant::now());
    }

    pub fn end_parse_phase(&mut self) {
        if let Some(start) = self.phase_start {
            self.current_metrics.query_parse_time = start.elapsed();
        }
    }

    pub fn end_plan_phase(&mut self) {
        if let Some(start) = self.phase_start {
            self.current_metrics.query_plan_time = start.elapsed();
        }
    }

    pub fn end_execute_phase(&mut self) {
        if let Some(start) = self.phase_start {
            self.current_metrics.query_execute_time = start.elapsed();
        }
    }

    pub fn end_total(&mut self) {
        if let Some(start) = self.start {
            self.current_metrics.total_time = start.elapsed();
        }
    }

    pub fn set_rows(&mut self, processed: usize, returned: usize) {
        self.current_metrics.rows_processed = processed;
        self.current_metrics.rows_returned = returned;
        self.current_metrics.calculate_throughput();
    }

    pub fn get_metrics(&self) -> BenchmarkMetrics {
        self.current_metrics.clone()
    }
}
