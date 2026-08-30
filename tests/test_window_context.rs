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

// --- P24: RANGE frames follow peer groups, ROWS follows physical rows ---

use sql_cli::sql::parser::ast::{FrameBound, FrameUnit, WindowFrame, WindowSpec};

/// Scores with deliberate ties, mirroring `data/null_edges.csv`:
/// sorted ascending they are 10, 20, 30, 50, 50, 70, 70, 90.
fn tied_scores_view() -> Arc<DataView> {
    let mut table = DataTable::new("scores");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("score"));

    for (id, score) in [
        (1, 50),
        (2, 50),
        (4, 70),
        (5, 30),
        (6, 70),
        (7, 90),
        (8, 10),
        (9, 20),
    ] {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(id),
                DataValue::Integer(score),
            ]))
            .unwrap();
    }

    Arc::new(DataView::new(Arc::new(table)))
}

fn order_by_score() -> Vec<OrderByItem> {
    vec![OrderByItem {
        expr: SqlExpression::Column(ColumnRef {
            name: "score".to_string(),
            quote_style: QuoteStyle::None,
            table_prefix: None,
        }),
        direction: SortDirection::Asc,
    }]
}

fn spec_with_frame(frame: Option<WindowFrame>) -> WindowSpec {
    WindowSpec {
        partition_by: vec![],
        order_by: order_by_score(),
        frame,
    }
}

/// Running SUM per source row index, in the fixture's row order.
fn running_sums(spec: WindowSpec) -> Vec<i64> {
    let context = WindowContext::new_with_spec(tied_scores_view(), spec).unwrap();
    (0..8)
        .map(|row| match context.get_frame_sum(row, "score") {
            Some(DataValue::Integer(n)) => n,
            other => panic!("row {row}: expected an integer sum, got {other:?}"),
        })
        .collect()
}

#[test]
fn range_frame_includes_every_peer_at_the_current_value() {
    // RANGE UNBOUNDED PRECEDING .. CURRENT ROW: the two 50s both see 160
    // (10+20+30+50+50), and the two 70s both see 300.
    let sums = running_sums(spec_with_frame(Some(WindowFrame {
        unit: FrameUnit::Range,
        start: FrameBound::UnboundedPreceding,
        end: Some(FrameBound::CurrentRow),
    })));

    // rows in fixture order: id 1,2,4,5,6,7,8,9 / score 50,50,70,30,70,90,10,20
    assert_eq!(sums, vec![160, 160, 300, 60, 300, 390, 10, 30]);
}

#[test]
fn rows_frame_still_counts_physical_rows() {
    // The same query with ROWS must NOT merge the ties: the first 50 sees only
    // itself (110), the second sees both (160). This is the behaviour RANGE was
    // wrongly inheriting.
    let sums = running_sums(spec_with_frame(Some(WindowFrame {
        unit: FrameUnit::Rows,
        start: FrameBound::UnboundedPreceding,
        end: Some(FrameBound::CurrentRow),
    })));

    assert_eq!(sums, vec![110, 160, 230, 60, 300, 390, 10, 30]);
}

#[test]
fn current_row_as_a_start_bound_opens_at_the_first_peer() {
    // RANGE CURRENT ROW .. UNBOUNDED FOLLOWING: both 50s see 50+50+70+70+90.
    let sums = running_sums(spec_with_frame(Some(WindowFrame {
        unit: FrameUnit::Range,
        start: FrameBound::CurrentRow,
        end: Some(FrameBound::UnboundedFollowing),
    })));

    // Descending totals from each peer group's first row (total is 390):
    //   10 -> 390, 20 -> 380, 30 -> 360, 50/50 -> 330, 70/70 -> 230, 90 -> 90
    assert_eq!(sums, vec![330, 330, 230, 360, 230, 90, 390, 380]);
}

#[test]
fn a_range_frame_with_a_numeric_offset_is_rejected_not_guessed() {
    // Value-based offsets are unimplemented. Erroring is the point: silently
    // answering as ROWS is precisely the P24 defect.
    let result = WindowContext::new_with_spec(
        tied_scores_view(),
        spec_with_frame(Some(WindowFrame {
            unit: FrameUnit::Range,
            start: FrameBound::Preceding(1),
            end: Some(FrameBound::CurrentRow),
        })),
    );
    let err = match result {
        Ok(_) => panic!("RANGE with a numeric offset should be rejected"),
        Err(e) => e.to_string(),
    };

    assert!(
        err.contains("RANGE frames with a numeric offset"),
        "unexpected error: {err}"
    );

    // The same offset under ROWS is positional and must still work.
    WindowContext::new_with_spec(
        tied_scores_view(),
        spec_with_frame(Some(WindowFrame {
            unit: FrameUnit::Rows,
            start: FrameBound::Preceding(1),
            end: Some(FrameBound::CurrentRow),
        })),
    )
    .expect("ROWS with a numeric offset should remain supported");
}
