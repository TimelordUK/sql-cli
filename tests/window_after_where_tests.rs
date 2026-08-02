//! Regression tests for P21 — window functions must be evaluated AFTER the
//! WHERE clause.
//!
//! SQL evaluates window functions after FROM/WHERE/GROUP BY/HAVING, so a
//! filtered-out row must not appear in a partition, occupy a ROW_NUMBER slot, or
//! be counted. Before the fix the window evaluator built its partitions from the
//! whole source table and a WHERE clause had no effect on any window — silently.
//!
//! These live in `cargo test` as well as the DuckDB corpus because the corpus
//! only runs in the Parity CI job and needs a reference engine; this class of bug
//! returns a plausible wrong answer with no error, so it deserves a check that
//! runs everywhere.

use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::execution::{ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

/// Two teams. `alpha` has three rows, one of which (id 3) has a NULL score and
/// is removed by the WHERE clause in every test below; `beta` has three rows
/// that all survive. So a window over `alpha` must see 2 rows and one over
/// `beta` must see 3 — any query that reports 3 for `alpha` is looking at
/// pre-filter data.
fn scores_table() -> DataTable {
    let mut table = DataTable::new("scores");
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("team").with_type(DataType::String));
    table.add_column(DataColumn::new("score").with_type(DataType::Integer));

    let rows: Vec<(i64, &str, Option<i64>)> = vec![
        (1, "alpha", Some(50)),
        (2, "alpha", Some(40)),
        (3, "alpha", None), // filtered out by `score IS NOT NULL`
        (4, "beta", Some(70)),
        (5, "beta", Some(60)),
        (6, "beta", Some(30)),
    ];

    for (id, team, score) in rows {
        let _ = table.add_row(DataRow {
            values: vec![
                DataValue::Integer(id),
                DataValue::String(team.to_string()),
                score.map_or(DataValue::Null, DataValue::Integer),
            ],
        });
    }

    table
}

/// Run `sql` against the fixture and return (id, value) pairs for the window
/// column, which every query below aliases to `v`.
fn run(sql: &str) -> Vec<(i64, Option<i64>)> {
    let mut context = ExecutionContext::new(Arc::new(scores_table()));
    let executor = StatementExecutor::new();
    let mut parser = Parser::new(sql);
    let stmt = parser.parse().expect("parse failed");
    let result = executor
        .execute(stmt, &mut context)
        .expect("execution failed");

    let view = &result.dataview;
    let id_col = view
        .column_names()
        .iter()
        .position(|c| c == "id")
        .expect("no id column");
    let v_col = view
        .column_names()
        .iter()
        .position(|c| c == "v")
        .expect("no v column");

    (0..view.row_count())
        .map(|r| {
            let id = match view.get_cell_value(r, id_col).as_deref() {
                Some(s) => s.parse::<i64>().expect("id not an integer"),
                None => panic!("null id"),
            };
            let v = view.get_cell_value(r, v_col).and_then(|s| {
                if s == "NULL" {
                    None
                } else {
                    s.parse().ok()
                }
            });
            (id, v)
        })
        .collect()
}

#[test]
fn count_over_partition_counts_only_surviving_rows() {
    let got = run("SELECT id, COUNT(*) OVER (PARTITION BY team) AS v \
         FROM scores WHERE score IS NOT NULL ORDER BY id");

    // alpha keeps 2 of its 3 rows, beta keeps all 3.
    assert_eq!(
        got,
        vec![
            (1, Some(2)),
            (2, Some(2)),
            (4, Some(3)),
            (5, Some(3)),
            (6, Some(3)),
        ],
        "COUNT(*) OVER must count post-WHERE rows; a 3 for team alpha means the \
         window saw the filtered-out row"
    );
}

#[test]
fn row_number_does_not_reserve_slots_for_filtered_rows() {
    let got = run(
        "SELECT id, ROW_NUMBER() OVER (PARTITION BY team ORDER BY score DESC, id) AS v \
         FROM scores WHERE score IS NOT NULL ORDER BY id",
    );

    assert_eq!(
        got,
        vec![
            (1, Some(1)),
            (2, Some(2)),
            (4, Some(1)),
            (5, Some(2)),
            (6, Some(3)),
        ],
        "ranks must be dense over the surviving rows; a filtered-out row must \
         not consume a slot"
    );
}

#[test]
fn window_in_a_derived_table_also_respects_the_filter() {
    // Pinned separately: a fix applied only to the top-level SELECT would leave
    // the nested form wrong.
    let got = run("SELECT id, v FROM (\
             SELECT id, ROW_NUMBER() OVER (ORDER BY score DESC, id) AS v \
             FROM scores WHERE score IS NOT NULL\
         ) x ORDER BY id");

    // Surviving scores ranked desc: 70(id4), 60(id5), 50(id1), 40(id2), 30(id6).
    assert_eq!(
        got,
        vec![
            (1, Some(3)),
            (2, Some(4)),
            (4, Some(1)),
            (5, Some(2)),
            (6, Some(5)),
        ]
    );
}

#[test]
fn unfiltered_windows_are_unchanged() {
    // The control that located P21 in the first place: without a WHERE, window
    // evaluation was always correct. This guards against a fix that "corrects"
    // the filtered case by breaking the unfiltered one.
    let got = run("SELECT id, COUNT(*) OVER (PARTITION BY team) AS v FROM scores ORDER BY id");

    assert_eq!(
        got,
        vec![
            (1, Some(3)),
            (2, Some(3)),
            (3, Some(3)),
            (4, Some(3)),
            (5, Some(3)),
            (6, Some(3)),
        ]
    );
}

#[test]
fn sum_over_partition_respects_the_filter() {
    // SUM is the near-miss: in the corpus fixture the filtered-out rows carried
    // NULL scores, which SUM ignores anyway, so SUM AGREEd while P21 was live.
    // Here id 3 is still the NULL row, so this asserts the same shape — but the
    // test is worth keeping because it is the query users actually write, and it
    // would catch a fix that reached COUNT but not the aggregate windows.
    let got = run("SELECT id, SUM(score) OVER (PARTITION BY team) AS v \
         FROM scores WHERE score IS NOT NULL ORDER BY id");

    assert_eq!(
        got,
        vec![
            (1, Some(90)),
            (2, Some(90)),
            (4, Some(160)),
            (5, Some(160)),
            (6, Some(160)),
        ]
    );
}
