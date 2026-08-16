//! Regression tests for turning a `DataView` back into a `DataTable`
//! (parity P28 / P31).
//!
//! A `DataView` carries three things the source table does not: the WHERE
//! filter (`visible_rows`), the SELECT projection (`visible_columns`) and the
//! LIMIT/OFFSET window. Every consumer that materializes a result — staging a
//! temp table via `SELECT ... INTO`, applying the `--limit` flag — has to
//! respect all three. Reaching past the view to `source()` silently
//! reintroduces rows and columns the query excluded.
//!
//! `materialize_view` already honoured the filter and the projection; it
//! iterated `visible_row_indices()`, which is the *pre-limit* set, so a LIMIT
//! was dropped.

use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use sql_cli::execution::{ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

/// 5 rows, 3 columns; 3 rows have `score > 40`.
fn scores_table() -> DataTable {
    let mut table = DataTable::new("scores");
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("team").with_type(DataType::String));
    table.add_column(DataColumn::new("score").with_type(DataType::Integer));
    for (id, team, score) in [
        (1, "alpha", 50),
        (2, "alpha", 10),
        (3, "beta", 70),
        (4, "beta", 20),
        (5, "gamma", 90),
    ] {
        let _ = table.add_row(DataRow {
            values: vec![
                DataValue::Integer(id),
                DataValue::String(team.to_string()),
                DataValue::Integer(score),
            ],
        });
    }
    table
}

fn view_for(sql: &str) -> DataView {
    let mut context = ExecutionContext::new(Arc::new(scores_table()));
    let executor = StatementExecutor::new();
    let stmt = Parser::new(sql)
        .parse()
        .unwrap_or_else(|e| panic!("parse failed for `{sql}`: {e}"));
    executor
        .execute(stmt, &mut context)
        .unwrap_or_else(|e| panic!("exec failed for `{sql}`: {e}"))
        .dataview
}

fn materialize(sql: &str) -> DataTable {
    QueryEngine::new()
        .materialize_view(view_for(sql))
        .expect("materialize")
}

#[test]
fn materialize_view_honours_the_where_clause() {
    let table = materialize("SELECT id, score FROM scores WHERE score > 40");
    assert_eq!(
        table.row_count(),
        3,
        "materializing must not reintroduce filtered-out rows"
    );
}

#[test]
fn materialize_view_honours_the_select_list() {
    let table = materialize("SELECT id, score FROM scores WHERE score > 40");
    let names: Vec<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["id", "score"],
        "materializing must keep the projection, not the source's columns"
    );
}

#[test]
fn materialize_view_honours_limit() {
    // The bug: `visible_row_indices()` is the pre-limit set, so all 5 rows
    // were copied.
    let table = materialize("SELECT id FROM scores LIMIT 2");
    assert_eq!(table.row_count(), 2, "LIMIT must survive materialization");
}

#[test]
fn materialize_view_honours_limit_after_a_filter() {
    let table = materialize("SELECT id FROM scores WHERE score > 40 LIMIT 2");
    assert_eq!(table.row_count(), 2);
}

#[test]
fn materialize_view_keeps_order_by() {
    let table = materialize("SELECT id, score FROM scores ORDER BY score DESC");
    let scores: Vec<i64> = (0..table.row_count())
        .map(|i| match &table.rows[i].values[1] {
            DataValue::Integer(n) => *n,
            other => panic!("expected an integer score, got {other:?}"),
        })
        .collect();
    assert_eq!(scores, vec![90, 70, 50, 20, 10]);
}

#[test]
fn materialize_view_of_an_unrestricted_query_is_the_whole_table() {
    // Control: the case that was always correct must stay correct.
    let table = materialize("SELECT * FROM scores");
    assert_eq!(table.row_count(), 5);
    assert_eq!(table.columns.len(), 3);
}

#[test]
fn windowed_row_indices_applies_offset_and_limit() {
    let view = DataView::new(Arc::new(scores_table())).with_limit(2, 1);
    assert_eq!(view.visible_row_indices(), &[0, 1, 2, 3, 4]);
    assert_eq!(
        view.windowed_row_indices(),
        &[1, 2],
        "the windowed set is what the consumer actually sees"
    );
    assert_eq!(view.row_count(), view.windowed_row_indices().len());
}

#[test]
fn windowed_row_indices_clamps_an_oversized_window() {
    let view = DataView::new(Arc::new(scores_table())).with_limit(100, 3);
    assert_eq!(view.windowed_row_indices(), &[3, 4]);

    let past_the_end = DataView::new(Arc::new(scores_table())).with_limit(2, 99);
    assert!(past_the_end.windowed_row_indices().is_empty());
}

#[test]
fn with_max_rows_takes_the_tighter_window() {
    let base = DataView::new(Arc::new(scores_table()));
    assert_eq!(base.clone().with_max_rows(3).row_count(), 3);

    // A wider display limit must not widen an existing SQL LIMIT.
    let limited = DataView::new(Arc::new(scores_table())).with_limit(2, 0);
    assert_eq!(limited.with_max_rows(5).row_count(), 2);
}

#[test]
fn with_max_rows_preserves_the_offset() {
    let view = DataView::new(Arc::new(scores_table()))
        .with_limit(4, 1)
        .with_max_rows(2);
    assert_eq!(view.windowed_row_indices(), &[1, 2]);
}
