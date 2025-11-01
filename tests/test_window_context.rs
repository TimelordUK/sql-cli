use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::sql::parser::ast::{ColumnRef, OrderByItem, QuoteStyle, SqlExpression};
use sql_cli::sql::recursive_parser::SortDirection;
use sql_cli::sql::window_context::WindowContext;
use std::sync::Arc;

#[test]
fn test_window_context_single_partition() {
    // Create test data
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("value"));

    // Add rows: [1,10], [2,20], [3,30], [4,40], [5,50]
    for i in 1..=5 {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(i),
                DataValue::Integer(i * 10),
            ]))
            .unwrap();
    }

    let view = DataView::new(Arc::new(table));

    // Create window context with ORDER BY id
    let context = WindowContext::new(
        Arc::new(view),
        vec![], // No partition
        vec![OrderByItem {
            expr: SqlExpression::Column(ColumnRef {
                name: "id".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            }),
            direction: SortDirection::Asc,
        }],
    )
    .unwrap();

    // Test LAG functionality
    assert_eq!(
        context.get_offset_value(0, -1, "value"),
        None, // No previous row
    );

    assert_eq!(
        context.get_offset_value(1, -1, "value"),
        Some(DataValue::Integer(10)), // Previous row has value 10
    );

    assert_eq!(
        context.get_offset_value(2, -1, "value"),
        Some(DataValue::Integer(20)), // Previous row has value 20
    );

    // Test LEAD functionality
    assert_eq!(
        context.get_offset_value(0, 1, "value"),
        Some(DataValue::Integer(20)), // Next row has value 20
    );

    assert_eq!(
        context.get_offset_value(4, 1, "value"),
        None, // No next row
    );

    // Test ROW_NUMBER
    assert_eq!(context.get_row_number(0), 1);
    assert_eq!(context.get_row_number(1), 2);
    assert_eq!(context.get_row_number(4), 5);
}

#[test]
fn test_window_context_with_partitions() {
    // Create test data with categories
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("value"));

    // Category A: [A,1,100], [A,2,200], [A,3,300]
    // Category B: [B,1,10], [B,2,20]
    let data = vec![
        ("A", 1, 100),
        ("A", 2, 200),
        ("A", 3, 300),
        ("B", 1, 10),
        ("B", 2, 20),
    ];

    for (cat, id, val) in data {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(cat.to_string()),
                DataValue::Integer(id),
                DataValue::Integer(val),
            ]))
            .unwrap();
    }

    let view = DataView::new(Arc::new(table));

    // Create window context with PARTITION BY category, ORDER BY id
    let context = WindowContext::new(
        Arc::new(view),
        vec!["category".to_string()],
        vec![OrderByItem {
            expr: SqlExpression::Column(ColumnRef {
                name: "id".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            }),
            direction: SortDirection::Asc,
        }],
    )
    .unwrap();

    // Test that we have 2 partitions
    assert_eq!(context.partition_count(), 2);

    // Test ROW_NUMBER restarts for each partition
    assert_eq!(context.get_row_number(0), 1); // First row in partition A
    assert_eq!(context.get_row_number(1), 2); // Second row in partition A
    assert_eq!(context.get_row_number(3), 1); // First row in partition B
    assert_eq!(context.get_row_number(4), 2); // Second row in partition B

    // Test LAG within partitions
    assert_eq!(
        context.get_offset_value(0, -1, "value"),
        None, // No previous in partition A
    );

    assert_eq!(
        context.get_offset_value(1, -1, "value"),
        Some(DataValue::Integer(100)), // Previous in partition A
    );

    assert_eq!(
        context.get_offset_value(3, -1, "value"),
        None, // No previous in partition B (different partition)
    );

    // Test FIRST_VALUE/LAST_VALUE
    assert_eq!(
        context.get_first_value(1, "value"), // Any row in partition A
        Some(DataValue::Integer(100)),       // First value in partition A
    );

    assert_eq!(
        context.get_last_value(1, "value"), // Any row in partition A
        Some(DataValue::Integer(300)),      // Last value in partition A
    );

    assert_eq!(
        context.get_first_value(4, "value"), // Any row in partition B
        Some(DataValue::Integer(10)),        // First value in partition B
    );
}

#[test]
fn test_window_context_order_by_desc() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("value"));

    // Add rows: [30], [10], [20], [50], [40]
    let values = vec![30, 10, 20, 50, 40];
    for v in values {
        table
            .add_row(DataRow::new(vec![DataValue::Integer(v)]))
            .unwrap();
    }

    let view = DataView::new(Arc::new(table));

    // Create window context with ORDER BY value DESC
    let context = WindowContext::new(
        Arc::new(view),
        vec![],
        vec![OrderByItem {
            expr: SqlExpression::Column(ColumnRef {
                name: "value".to_string(),
                quote_style: QuoteStyle::None,
                table_prefix: None,
            }),
            direction: SortDirection::Desc,
        }],
    )
    .unwrap();

    // After ordering DESC: [50], [40], [30], [20], [10]
    // Original indices:     [3],  [4],  [0],  [2],  [1]

    // Test ROW_NUMBER reflects the DESC order
    assert_eq!(context.get_row_number(3), 1); // value=50 is first
    assert_eq!(context.get_row_number(4), 2); // value=40 is second
    assert_eq!(context.get_row_number(0), 3); // value=30 is third
    assert_eq!(context.get_row_number(2), 4); // value=20 is fourth
    assert_eq!(context.get_row_number(1), 5); // value=10 is fifth

    // Test LAG gets previous in DESC order
    assert_eq!(
        context.get_offset_value(4, -1, "value"), // 40's previous
        Some(DataValue::Integer(50)),             // Previous in DESC order is 50
    );

    assert_eq!(
        context.get_offset_value(0, -1, "value"), // 30's previous
        Some(DataValue::Integer(40)),             // Previous in DESC order is 40
    );
}
