use crate::benchmarks::data_generator::BenchmarkDataGenerator;
use crate::benchmarks::metrics::{BenchmarkResult, MetricsCollector};
use crate::benchmarks::query_suite::{BenchmarkQuery, QuerySuite};
use crate::data::datatable::DataTable;
use crate::data::query_engine::QueryEngine;
use crate::sql::recursive_parser::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

pub struct BenchmarkRunner {
    results: Vec<BenchmarkResult>,
    tables: HashMap<String, DataTable>,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        BenchmarkRunner {
            results: Vec::new(),
            tables: HashMap::new(),
        }
    }

    pub fn prepare_benchmark_data(&mut self, sizes: &[usize]) {
        println!("=== Preparing Benchmark Data ===");
        for &size in sizes {
            println!("Generating tables with {} rows...", size);
            let tables = BenchmarkDataGenerator::generate_all_benchmark_tables(size);

            for (name, table) in tables {
                let key = format!("{}_{}", name, size);
                println!(
                    "  - Generated {} table: {} rows, {} columns",
                    key,
                    table.rows.len(),
                    table.columns.len()
                );
                self.tables.insert(key, table);
            }
        }
        println!("Total tables generated: {}\n", self.tables.len());
    }

    pub fn run_query_benchmark(
        &mut self,
        query: &BenchmarkQuery,
        table: &DataTable,
        table_name: &str,
        row_count: usize,
    ) -> BenchmarkResult {
        let mut result = BenchmarkResult::new(
            query.name.clone(),
            query.category.as_str().to_string(),
            table_name.to_string(),
            row_count,
        );

        let mut collector = MetricsCollector::new();
        collector.start_total();

        collector.start_phase();
        let parse_result = Parser::new(&query.sql).parse();
        collector.end_parse_phase();

        match parse_result {
            Ok(statement) => {
                collector.start_phase();

                let engine = QueryEngine::new();
                let table_arc = Arc::new(table.clone());

                collector.start_phase();
                match engine.execute_statement(table_arc, statement) {
                    Ok(result_view) => {
                        collector.end_execute_phase();
                        collector.set_rows(table.rows.len(), result_view.row_count());
                    }
                    Err(e) => {
                        result.error = Some(format!("Execution error: {}", e));
                    }
                }
            }
            Err(e) => {
                result.error = Some(format!("Parse error: {}", e));
            }
        }

        collector.end_total();
        result.metrics = collector.get_metrics();
        result
    }

    pub fn run_progressive_benchmarks(&mut self, increment: usize, max_rows: usize) {
        println!("=== Running Progressive Benchmarks ===");
        println!("Increment: {} rows, Max: {} rows\n", increment, max_rows);

        let sizes: Vec<usize> = (1..=max_rows / increment).map(|i| i * increment).collect();

        self.prepare_benchmark_data(&sizes);

        let queries = QuerySuite::get_progressive_queries();

        for size in &sizes {
            if let Some(query_sqls) = queries.get(size) {
                println!("Running benchmarks for {} rows:", size);

                let table_key = format!("mixed_{}", size);
                if let Some(table) = self.tables.get(&table_key).cloned() {
                    for (i, sql) in query_sqls.iter().enumerate() {
                        let query = BenchmarkQuery::new(
                            format!("prog_query_{}", i),
                            crate::benchmarks::query_suite::QueryCategory::BasicOperations,
                            sql.clone(),
                            format!("Progressive query {}", i),
                            "mixed",
                        );

                        let result = self.run_query_benchmark(&query, &table, &table_key, *size);

                        println!(
                            "  - {}: {:.2}ms, {} rows/sec",
                            query.name,
                            result.metrics.total_time.as_secs_f64() * 1000.0,
                            result.metrics.rows_per_second as u64
                        );

                        self.results.push(result);
                    }
                }
                println!();
            }
        }
    }

    pub fn run_comprehensive_benchmarks(&mut self, sizes: &[usize]) {
        println!("=== Running Comprehensive Benchmarks ===");
        self.prepare_benchmark_data(sizes);

        let all_queries = QuerySuite::get_all_queries();
        let total_benchmarks = sizes.len() * all_queries.len();
        let mut completed = 0;

        for &size in sizes {
            println!("\n--- Testing with {} rows ---", size);

            for query in &all_queries {
                let table_key = format!("{}_{}", query.table_type, size);

                let table = if query.table_type == "all" {
                    self.tables.get(&format!("mixed_{}", size)).cloned()
                } else {
                    self.tables.get(&table_key).cloned()
                };

                if let Some(table) = table {
                    let result = self.run_query_benchmark(&query, &table, &query.table_type, size);

                    completed += 1;
                    let progress = (completed as f64 / total_benchmarks as f64) * 100.0;

                    if result.error.is_none() {
                        println!(
                            "[{:.1}%] {} ({}): {:.2}ms",
                            progress,
                            query.name,
                            query.category.as_str(),
                            result.metrics.total_time.as_secs_f64() * 1000.0
                        );
                    } else {
                        println!(
                            "[{:.1}%] {} ({}): ERROR - {}",
                            progress,
                            query.name,
                            query.category.as_str(),
                            result.error.as_ref().unwrap()
                        );
                    }

                    self.results.push(result);
                }
            }
        }
    }

    pub fn run_category_benchmarks(
        &mut self,
        category: crate::benchmarks::query_suite::QueryCategory,
        sizes: &[usize],
    ) {
        println!("=== Running {} Benchmarks ===", category.as_str());
        self.prepare_benchmark_data(sizes);

        let queries: Vec<_> = QuerySuite::get_all_queries()
            .into_iter()
            .filter(|q| q.category == category)
            .collect();

        for &size in sizes {
            println!(
                "\nTesting {} queries with {} rows:",
                category.as_str(),
                size
            );

            for query in &queries {
                let table_key = format!("{}_{}", query.table_type, size);
                let table = self
                    .tables
                    .get(&table_key)
                    .cloned()
                    .or_else(|| self.tables.get(&format!("mixed_{}", size)).cloned());

                if let Some(table) = table {
                    let result = self.run_query_benchmark(&query, &table, &query.table_type, size);

                    println!(
                        "  {}: {:.2}ms ({})",
                        query.name,
                        result.metrics.total_time.as_secs_f64() * 1000.0,
                        if result.error.is_none() {
                            "OK"
                        } else {
                            "FAILED"
                        }
                    );

                    self.results.push(result);
                }
            }
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# SQL CLI Benchmark Report\n\n");
        report.push_str(&format!("Generated: {}\n", chrono::Local::now()));
        report.push_str(&format!("Total benchmarks run: {}\n\n", self.results.len()));

        report.push_str("## Summary Statistics\n\n");

        let successful: Vec<_> = self.results.iter().filter(|r| r.error.is_none()).collect();

        if !successful.is_empty() {
            let avg_time: f64 = successful
                .iter()
                .map(|r| r.metrics.total_time.as_secs_f64())
                .sum::<f64>()
                / successful.len() as f64;

            let avg_throughput: f64 = successful
                .iter()
                .map(|r| r.metrics.rows_per_second)
                .sum::<f64>()
                / successful.len() as f64;

            report.push_str(&format!("- Successful: {}\n", successful.len()));
            report.push_str(&format!(
                "- Failed: {}\n",
                self.results.len() - successful.len()
            ));
            report.push_str(&format!(
                "- Average execution time: {:.2}ms\n",
                avg_time * 1000.0
            ));
            report.push_str(&format!(
                "- Average throughput: {:.0} rows/sec\n\n",
                avg_throughput
            ));
        }

        report.push_str("## Results by Category\n\n");

        let categories = vec!["basic", "aggregation", "sorting", "window", "complex"];

        for category in categories {
            let category_results: Vec<_> = self
                .results
                .iter()
                .filter(|r| r.query_category == category && r.error.is_none())
                .collect();

            if !category_results.is_empty() {
                report.push_str(&format!(
                    "### {} Operations\n\n",
                    category.chars().next().unwrap().to_uppercase().to_string() + &category[1..]
                ));

                for result in category_results {
                    report.push_str(&format!(
                        "- {} ({} rows): {:.2}ms, {:.0} rows/sec\n",
                        result.query_name,
                        result.row_count,
                        result.metrics.total_time.as_secs_f64() * 1000.0,
                        result.metrics.rows_per_second
                    ));
                }
                report.push_str("\n");
            }
        }

        report.push_str("## Performance by Data Size\n\n");

        let mut size_map: HashMap<usize, Vec<&BenchmarkResult>> = HashMap::new();
        for result in &self.results {
            if result.error.is_none() {
                size_map
                    .entry(result.row_count)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }

        let mut sizes: Vec<_> = size_map.keys().cloned().collect();
        sizes.sort();

        for size in sizes {
            if let Some(results) = size_map.get(&size) {
                let avg_time: f64 = results
                    .iter()
                    .map(|r| r.metrics.total_time.as_secs_f64())
                    .sum::<f64>()
                    / results.len() as f64;

                report.push_str(&format!(
                    "- {} rows: avg {:.2}ms ({} queries)\n",
                    size,
                    avg_time * 1000.0,
                    results.len()
                ));
            }
        }

        report
    }

    pub fn save_results_csv(&self, filename: &str) -> Result<(), String> {
        let mut file =
            File::create(filename).map_err(|e| format!("Failed to create CSV file: {}", e))?;

        writeln!(file, "query_name,table,row_count,parse_ms,plan_ms,execute_ms,total_ms,rows_processed,rows_returned,rows_per_sec,status")
            .map_err(|e| format!("Failed to write CSV header: {}", e))?;

        for result in &self.results {
            writeln!(file, "{}", result.to_csv_row())
                .map_err(|e| format!("Failed to write CSV row: {}", e))?;
        }

        Ok(())
    }

    pub fn print_summary(&self) {
        println!("\n=== Benchmark Summary ===");

        let successful = self.results.iter().filter(|r| r.error.is_none()).count();
        let failed = self.results.len() - successful;

        println!(
            "Total: {}, Successful: {}, Failed: {}",
            self.results.len(),
            successful,
            failed
        );

        if successful > 0 {
            let total_time: f64 = self
                .results
                .iter()
                .filter(|r| r.error.is_none())
                .map(|r| r.metrics.total_time.as_secs_f64())
                .sum();

            println!("Total benchmark time: {:.2}s", total_time);

            let fastest = self
                .results
                .iter()
                .filter(|r| r.error.is_none())
                .min_by_key(|r| r.metrics.total_time)
                .unwrap();

            let slowest = self
                .results
                .iter()
                .filter(|r| r.error.is_none())
                .max_by_key(|r| r.metrics.total_time)
                .unwrap();

            println!(
                "\nFastest query: {} ({:.2}ms)",
                fastest.query_name,
                fastest.metrics.total_time.as_secs_f64() * 1000.0
            );

            println!(
                "Slowest query: {} ({:.2}ms)",
                slowest.query_name,
                slowest.metrics.total_time.as_secs_f64() * 1000.0
            );
        }

        if failed > 0 {
            println!("\nFailed queries:");
            for result in self.results.iter().filter(|r| r.error.is_some()) {
                println!(
                    "  - {}: {}",
                    result.query_name,
                    result.error.as_ref().unwrap()
                );
            }
        }
    }
}
