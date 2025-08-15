use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

fn create_test_data(rows: usize) -> DataTable {
    let mut table = DataTable::new("test");

    // Add columns
    table.add_column(DataColumn::new("book"));
    table.add_column(DataColumn::new("value"));
    table.add_column(DataColumn::new("status"));

    // Add rows with varying book values
    let book_values = vec![
        "Commodities Trading",
        "Equity Trading",
        "FX Trading",
        "Bond Trading",
        "Derivatives",
        "Options",
        "Futures",
        "ETF Trading",
        "Structured Products",
        "Money Markets",
    ];

    for i in 0..rows {
        let book = book_values[i % book_values.len()].to_string();
        let row = DataRow::new(vec![
            DataValue::String(book),
            DataValue::Integer(i as i64),
            DataValue::String(format!("STATUS_{}", i % 5)),
        ]);
        table.add_row(row).unwrap();
    }

    table
}

fn benchmark_contains_query(c: &mut Criterion) {
    let table_10k = Arc::new(create_test_data(10_000));
    let table_50k = Arc::new(create_test_data(50_000));
    let table_100k = Arc::new(create_test_data(100_000));

    let mut group = c.benchmark_group("where_contains");

    // Test with 10k rows
    group.bench_function("10k_rows", |b| {
        let engine = QueryEngine::new();
        let query = "SELECT * FROM test WHERE book.Contains('comm')";
        b.iter(|| {
            let result = engine.execute(table_10k.clone(), black_box(query));
            assert!(result.is_ok());
        });
    });

    // Test with 50k rows
    group.bench_function("50k_rows", |b| {
        let engine = QueryEngine::new();
        let query = "SELECT * FROM test WHERE book.Contains('comm')";
        b.iter(|| {
            let result = engine.execute(table_50k.clone(), black_box(query));
            assert!(result.is_ok());
        });
    });

    // Test with 100k rows
    group.bench_function("100k_rows", |b| {
        let engine = QueryEngine::new();
        let query = "SELECT * FROM test WHERE book.Contains('comm')";
        b.iter(|| {
            let result = engine.execute(table_100k.clone(), black_box(query));
            assert!(result.is_ok());
        });
    });

    group.finish();
}

fn benchmark_simple_comparison(c: &mut Criterion) {
    let table_100k = Arc::new(create_test_data(100_000));

    let mut group = c.benchmark_group("where_comparison");

    // Simple equality comparison
    group.bench_function("equality", |b| {
        let engine = QueryEngine::new();
        let query = "SELECT * FROM test WHERE status = 'STATUS_1'";
        b.iter(|| {
            let result = engine.execute(table_100k.clone(), black_box(query));
            assert!(result.is_ok());
        });
    });

    // Numeric comparison
    group.bench_function("numeric_gt", |b| {
        let engine = QueryEngine::new();
        let query = "SELECT * FROM test WHERE value > 50000";
        b.iter(|| {
            let result = engine.execute(table_100k.clone(), black_box(query));
            assert!(result.is_ok());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_contains_query,
    benchmark_simple_comparison
);
criterion_main!(benches);
