//! Tests for ORDER BY arbitrary expressions.
//!
//! Pre-fix the engine rejected anything but `SqlExpression::Column` in ORDER
//! BY with "ORDER BY expressions not yet supported - only simple columns
//! allowed". The fix promotes non-Column expressions as hidden SELECT items
//! (`__hidden_orderby_N`) which get evaluated normally and stripped from
//! output after sort.
//!
//! These tests run through `StatementExecutor` (not `QueryEngine` directly)
//! because the AST transformer pipeline lives there.

use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::execution::context::ExecutionContext;
use sql_cli::execution::statement_executor::StatementExecutor;
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

fn build_users_table() -> Arc<DataTable> {
    let mut table = DataTable::new("users");
    table.add_column(DataColumn::new("user_id"));
    table.add_column(DataColumn::new("name"));

    for (id, name) in [(1, "Daniel"), (2, "Monica"), (3, "Maria"), (4, "James")] {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(id),
                DataValue::String(name.to_string()),
            ]))
            .unwrap();
    }
    Arc::new(table)
}

fn build_ratings_table() -> Arc<DataTable> {
    let mut table = DataTable::new("ratings");
    table.add_column(DataColumn::new("user_id"));
    table.add_column(DataColumn::new("rating"));

    // user 1: 3 ratings, user 2: 3 ratings, user 3: 2 ratings, user 4: 1 rating
    for (uid, rating) in [
        (1, 3),
        (1, 1),
        (1, 4),
        (2, 5),
        (2, 2),
        (2, 4),
        (3, 3),
        (3, 4),
        (4, 2),
    ] {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(uid),
                DataValue::Integer(rating),
            ]))
            .unwrap();
    }
    Arc::new(table)
}

fn run(table: Arc<DataTable>, sql: &str) -> DataView {
    let stmt = Parser::new(sql)
        .parse()
        .unwrap_or_else(|e| panic!("parse failed for `{}`: {}", sql, e));
    let executor = StatementExecutor::new();
    let mut ctx = ExecutionContext::new(table);
    let result = executor
        .execute(stmt, &mut ctx)
        .unwrap_or_else(|e| panic!("execute failed for `{}`: {}", sql, e));
    result.dataview
}

fn column_names(view: &DataView) -> Vec<String> {
    view.column_names()
}

fn integer_column(view: &DataView, idx: usize) -> Vec<i64> {
    (0..view.row_count())
        .map(|i| match &view.get_row(i).unwrap().values[idx] {
            DataValue::Integer(n) => *n,
            other => panic!("expected integer at row {} col {}, got {:?}", i, idx, other),
        })
        .collect()
}

fn string_column(view: &DataView, idx: usize) -> Vec<String> {
    (0..view.row_count())
        .map(|i| match &view.get_row(i).unwrap().values[idx] {
            DataValue::String(s) => s.clone(),
            other => panic!("expected string at row {} col {}, got {:?}", i, idx, other),
        })
        .collect()
}

#[test]
fn order_by_function_call_on_projected_column() {
    let view = run(
        build_users_table(),
        "SELECT name FROM users ORDER BY UPPER(name) DESC",
    );

    // Hidden orderby column must not appear in output
    assert_eq!(column_names(&view), vec!["name"]);
    assert_eq!(
        string_column(&view, 0),
        vec!["Monica", "Maria", "James", "Daniel"]
    );
}

#[test]
fn order_by_arithmetic_with_group_by() {
    let view = run(
        build_ratings_table(),
        "SELECT user_id, COUNT(*) AS n FROM ratings GROUP BY user_id ORDER BY user_id + 10 DESC",
    );

    assert_eq!(column_names(&view), vec!["user_id", "n"]);
    assert_eq!(integer_column(&view, 0), vec![4, 3, 2, 1]);
}

#[test]
fn order_by_aggregate_not_in_select() {
    // COUNT(*) is not in SELECT list. Pre-fix this hit the unsupported-expr
    // error; now it gets promoted as a hidden column and evaluated per group.
    let view = run(
        build_ratings_table(),
        "SELECT user_id FROM ratings GROUP BY user_id ORDER BY COUNT(*) DESC, user_id ASC",
    );

    assert_eq!(column_names(&view), vec!["user_id"]);
    // counts: user 1=3, user 2=3, user 3=2, user 4=1
    // ORDER BY count DESC, user_id ASC → 1, 2, 3, 4
    assert_eq!(integer_column(&view, 0), vec![1, 2, 3, 4]);
}

#[test]
fn order_by_case_expression() {
    // CASE moves user_id=1 to the front (sort key 0); others sort by their value
    let view = run(
        build_users_table(),
        "SELECT user_id, name FROM users \
         ORDER BY CASE user_id WHEN 1 THEN 0 ELSE user_id END",
    );

    assert_eq!(column_names(&view), vec!["user_id", "name"]);
    assert_eq!(integer_column(&view, 0), vec![1, 2, 3, 4]);
}

#[test]
fn computed_expression_on_grouping_key_is_allowed() {
    // Standalone validator regression: SELECT col + N FROM t GROUP BY col was
    // previously rejected with "must appear in GROUP BY clause" — extending
    // the validator to walk column refs unblocks this independently of the
    // ORDER BY change.
    let view = run(
        build_ratings_table(),
        "SELECT user_id + 10 AS shifted FROM ratings GROUP BY user_id",
    );

    let names = column_names(&view);
    assert!(
        names.contains(&"shifted".to_string()),
        "expected 'shifted' column, got {:?}",
        names
    );
}
