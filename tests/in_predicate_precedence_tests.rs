//! Regression tests for P29/P30 — `IN` must bind at the comparison level, so it
//! composes with the surrounding boolean operators.
//!
//! `IN` used to be applied at the top of `parse_expression`, *after* the whole
//! OR/AND hierarchy had been parsed. That produced two different failures
//! depending on operand order:
//!
//!   `WHERE a = 1 AND b IN (..)` parsed as `InList { expr: (a = 1 AND b), .. }`
//!       — "is this boolean one of the list values?" — false for every row, so
//!       the query silently returned NOTHING.
//!   `WHERE b IN (..) AND a = 1` left `AND a = 1` unconsumed, which was silently
//!       discarded until P13 stage 1 and a hard error after it.
//!
//! These live in `cargo test` as well as the DuckDB corpus because the corpus
//! only runs in the Parity CI job and needs a reference engine; P30 returned a
//! plausible wrong answer (an empty result reads as "no matching data"), so it
//! deserves a check that runs everywhere.

use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::execution::{ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

/// Rows chosen so that each predicate in a compound WHERE selects a *different*
/// set: `team = 'alpha'` takes ids 1-3, `score IN (50, 70)` takes ids 1, 2, 4, 6.
/// The intersection is ids 1 and 2, so a result of 3 rows means the `team` test
/// was dropped, 4 rows means the `IN` was dropped, and 0 rows is the P30 shape.
fn scores_table() -> DataTable {
    let mut table = DataTable::new("scores");
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("team").with_type(DataType::String));
    table.add_column(DataColumn::new("score").with_type(DataType::Integer));

    let rows: Vec<(i64, &str, Option<i64>)> = vec![
        (1, "alpha", Some(50)),
        (2, "alpha", Some(50)),
        (3, "alpha", None),
        (4, "beta", Some(70)),
        (5, "beta", Some(30)),
        (6, "beta", Some(70)),
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

/// Run `sql` against the fixture and return the surviving ids, in order.
fn ids(sql: &str) -> Vec<i64> {
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

    (0..view.row_count())
        .map(|r| {
            view.get_cell_value(r, id_col)
                .expect("null id")
                .parse::<i64>()
                .expect("id not an integer")
        })
        .collect()
}

#[test]
fn condition_before_in_list_applies_both() {
    // P30: this parsed fine and returned zero rows.
    assert_eq!(
        ids("SELECT id FROM scores WHERE team = 'alpha' AND score IN (50, 70) ORDER BY id"),
        vec![1, 2],
        "an empty result is the P30 signature: IN swallowed the AND expression \
         and asked whether a boolean was one of the list values"
    );
}

#[test]
fn condition_after_in_list_applies_both() {
    // P29: the `AND team = ...` was discarded, then (post-P13) a parse error.
    assert_eq!(
        ids("SELECT id FROM scores WHERE score IN (50, 70) AND team = 'alpha' ORDER BY id"),
        vec![1, 2],
        "4 ids here means the trailing AND was dropped and only the IN applied"
    );
}

#[test]
fn in_list_composes_with_or_in_both_operand_orders() {
    // The defect was in how IN bound against the whole hierarchy, so OR needs
    // pinning too — a fix reaching only parse_logical_and would pass the AND
    // tests above and still break these.
    // score IN (50) selects {1, 2}; team = 'beta' selects {4, 5, 6}; union is all
    // but id 3, whose NULL score matches neither.
    let expected = vec![1, 2, 4, 5, 6];
    assert_eq!(
        ids("SELECT id FROM scores WHERE score IN (50) OR team = 'beta' ORDER BY id"),
        expected
    );
    assert_eq!(
        ids("SELECT id FROM scores WHERE team = 'beta' OR score IN (50) ORDER BY id"),
        expected
    );
}

#[test]
fn in_list_in_the_middle_of_a_chain() {
    // Needs parse_comparison to consume the IN *and* hand control back to the
    // AND loop. The old top-level placement could not express this at all.
    assert_eq!(
        ids("SELECT id FROM scores WHERE team = 'alpha' AND score IN (50, 70) AND id < 2 ORDER BY id"),
        vec![1]
    );
}

#[test]
fn not_in_still_composes() {
    // Control: NOT IN was always handled inside parse_comparison and was already
    // correct. It is what showed the fix belonged there. This must not regress.
    //
    // NOT IN (70) selects {1, 2, 5} — id 3's NULL score is excluded, which is
    // correct here and separately tracked as P19 for the general case — and
    // team = 'beta' selects {4, 5, 6}, so only id 5 satisfies both. Picking 70
    // rather than 50 keeps the two predicates disagreeing, so a dropped
    // conjunct changes the answer.
    assert_eq!(
        ids("SELECT id FROM scores WHERE score NOT IN (70) AND team = 'beta' ORDER BY id"),
        vec![5]
    );
}

#[test]
fn in_list_alone_is_unchanged() {
    // Control: `IN` with no surrounding boolean always worked, including the
    // exclusion of the NULL-scored id 3 against a list holding no NULL.
    assert_eq!(
        ids("SELECT id FROM scores WHERE score IN (50, 70) ORDER BY id"),
        vec![1, 2, 4, 6]
    );
}
