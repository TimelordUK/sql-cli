//! Regression tests for P7: multi-condition join ON-clauses used to evaluate an
//! extra condition's operands by *syntactic position* (left operand -> left
//! table, right operand -> right table), ignoring the alias each operand was
//! actually qualified with. So `b.price < a.price` (right-table column written
//! first) was silently evaluated as `a.price < b.price` and returned rows that
//! violated the predicate. See docs/SQL_PARITY.md :: P7.
//!
//! These are *self-consistency* tests: `b.price < a.price` and `a.price > b.price`
//! are the same predicate, so they must return identical result sets. That
//! property needs no reference engine, so it runs in plain `cargo test` — the
//! gap the corpus (DuckDB differential, Parity CI job) couldn't cover locally.
//!
//! The RIGHT-join test at the bottom pins P8: multi-condition RIGHT JOIN used to
//! reuse the LEFT builder with the tables swapped, which inverted the `a.*`/`b.*`
//! column labels and NULLed the wrong side. See docs/SQL_PARITY.md :: P8.

use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::execution::{ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

/// A tiny `trades` base table so a base-table self-join (P4) resolves.
fn trades_table() -> DataTable {
    let mut table = DataTable::new("trades");
    table.add_column(DataColumn::new("symbol").with_type(DataType::String));
    table.add_column(DataColumn::new("price").with_type(DataType::Integer));
    for (symbol, price) in [("A", 10), ("A", 20), ("A", 30), ("B", 5), ("B", 15)] {
        let _ = table.add_row(DataRow {
            values: vec![
                DataValue::String(symbol.to_string()),
                DataValue::Integer(price),
            ],
        });
    }
    table
}

/// Run a query and return its `(ap, bp)` rows sorted, treating NULL bp as `None`.
fn ap_bp_rows(sql: &str) -> Vec<(i64, Option<i64>)> {
    let context = &mut ExecutionContext::new(Arc::new(trades_table()));
    let executor = StatementExecutor::new();
    let mut parser = Parser::new(sql);
    let stmt = parser
        .parse()
        .unwrap_or_else(|e| panic!("parse failed for `{sql}`: {e}"));
    let result = executor
        .execute(stmt, context)
        .unwrap_or_else(|e| panic!("exec failed for `{sql}`: {e}"));

    let view = &result.dataview;
    let src = view.source();
    let ap_idx = src.get_column_index("ap").expect("ap column");
    let bp_idx = src.get_column_index("bp").expect("bp column");

    let as_int = |v: &DataValue| -> Option<i64> {
        match v {
            DataValue::Integer(i) => Some(*i),
            DataValue::Float(f) => Some(*f as i64),
            DataValue::Null => None,
            other => panic!("unexpected price value: {other:?}"),
        }
    };

    let mut rows: Vec<(i64, Option<i64>)> = (0..view.row_count())
        .map(|r| {
            let ap = as_int(&src.get_value(r, ap_idx).expect("ap value"))
                .expect("ap is never NULL on the outer side");
            let bp = as_int(&src.get_value(r, bp_idx).expect("bp value"));
            (ap, bp)
        })
        .collect();
    rows.sort();
    rows
}

#[test]
fn inner_join_operand_order_is_symmetric() {
    // Same predicate, operands written in opposite order — must agree.
    let right_first = ap_bp_rows(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a JOIN trades b ON a.symbol = b.symbol AND b.price < a.price",
    );
    let left_first = ap_bp_rows(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a JOIN trades b ON a.symbol = b.symbol AND a.price > b.price",
    );

    assert_eq!(
        right_first, left_first,
        "`b.price < a.price` and `a.price > b.price` are the same predicate and must return the same rows"
    );

    // Expected set (per trades_table): A(20>10), A(30>10), A(30>20), B(15>5).
    assert_eq!(
        right_first,
        vec![
            (15, Some(5)),
            (20, Some(10)),
            (30, Some(10)),
            (30, Some(20))
        ]
    );

    // Every surviving row must actually satisfy `bp < ap`.
    for (ap, bp) in &right_first {
        let bp = bp.expect("INNER join never yields NULL bp");
        assert!(
            bp < *ap,
            "row (ap={ap}, bp={bp}) violates b.price < a.price"
        );
    }
}

#[test]
fn left_join_operand_order_is_symmetric() {
    // The LEFT path had the same positional bug; unmatched left rows keep NULL bp.
    let right_first = ap_bp_rows(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a LEFT JOIN trades b ON a.symbol = b.symbol AND b.price < a.price",
    );
    let left_first = ap_bp_rows(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a LEFT JOIN trades b ON a.symbol = b.symbol AND a.price > b.price",
    );

    assert_eq!(
        right_first, left_first,
        "LEFT JOIN must also be independent of ON-operand order"
    );

    // Every left row survives; the per-symbol minimum (A=10, B=5) has no smaller
    // mate, so those emit NULL bp. A=30 matches two rows (10 and 20), so there are
    // 4 matched rows + 2 NULL rows = 6. Matched rows must satisfy bp < ap.
    assert_eq!(
        right_first.len(),
        6,
        "4 matched rows + 2 unmatched (NULL) left rows"
    );
    for (ap, bp) in &right_first {
        if let Some(bp) = bp {
            assert!(
                *bp < *ap,
                "matched row (ap={ap}, bp={bp}) violates b.price < a.price"
            );
        }
    }
    // Exactly the two per-symbol minima are unmatched (NULL bp).
    let nulls = right_first.iter().filter(|(_, bp)| bp.is_none()).count();
    assert_eq!(
        nulls, 2,
        "the two per-symbol minimum-price rows have no smaller mate"
    );
}

/// Like `ap_bp_rows`, but the outer (kept) side is `bp` — a RIGHT JOIN keeps every
/// `b` row and NULL-fills `ap`, so `ap` may be NULL and `bp` never is.
fn ap_bp_rows_right(sql: &str) -> Vec<(Option<i64>, i64)> {
    let context = &mut ExecutionContext::new(Arc::new(trades_table()));
    let executor = StatementExecutor::new();
    let mut parser = Parser::new(sql);
    let stmt = parser
        .parse()
        .unwrap_or_else(|e| panic!("parse failed for `{sql}`: {e}"));
    let result = executor
        .execute(stmt, context)
        .unwrap_or_else(|e| panic!("exec failed for `{sql}`: {e}"));

    let view = &result.dataview;
    let src = view.source();
    let ap_idx = src.get_column_index("ap").expect("ap column");
    let bp_idx = src.get_column_index("bp").expect("bp column");

    let as_int = |v: &DataValue| -> Option<i64> {
        match v {
            DataValue::Integer(i) => Some(*i),
            DataValue::Float(f) => Some(*f as i64),
            DataValue::Null => None,
            other => panic!("unexpected price value: {other:?}"),
        }
    };

    let mut rows: Vec<(Option<i64>, i64)> = (0..view.row_count())
        .map(|r| {
            let ap = as_int(&src.get_value(r, ap_idx).expect("ap value"));
            let bp = as_int(&src.get_value(r, bp_idx).expect("bp value"))
                .expect("bp is never NULL on the RIGHT outer side");
            (ap, bp)
        })
        .collect();
    rows.sort();
    rows
}

#[test]
fn right_join_multi_condition_labels_correct_side() {
    // P8: `a RIGHT JOIN b ON a.symbol = b.symbol AND a.price < b.price` keeps every
    // `b` row. The `a.*` values must land under `ap` and `b.*` under `bp` (not the
    // reverse), and unmatched rows must NULL the `a` (ap) side, not `b`.
    let right_first = ap_bp_rows_right(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a RIGHT JOIN trades b ON a.symbol = b.symbol AND a.price < b.price",
    );
    // Same predicate, operands written joined-side first — must agree (P7 holds
    // through the RIGHT path too).
    let joined_first = ap_bp_rows_right(
        "SELECT a.price AS ap, b.price AS bp \
         FROM trades a RIGHT JOIN trades b ON a.symbol = b.symbol AND b.price > a.price",
    );
    assert_eq!(
        right_first, joined_first,
        "RIGHT JOIN must be independent of ON-operand order"
    );

    // Expected (per trades_table): every b row kept; ap = a.price where a.price <
    // b.price, else NULL. b=(A,10)->NULL, b=(A,20)->10, b=(A,30)->{10,20},
    // b=(B,5)->NULL, b=(B,15)->5.
    assert_eq!(
        right_first,
        vec![
            (None, 5),
            (None, 10),
            (Some(5), 15),
            (Some(10), 20),
            (Some(10), 30),
            (Some(20), 30),
        ]
    );

    // The kept (bp) side is never NULL (enforced by the helper); only ap NULLs.
    // If the labels were swapped (the P8 bug) the NULLs would appear under bp.
    let ap_nulls = right_first.iter().filter(|(ap, _)| ap.is_none()).count();
    assert_eq!(ap_nulls, 2, "two b rows have no smaller-priced a mate");

    // Every matched row satisfies ap < bp — impossible if the values were swapped.
    for (ap, bp) in &right_first {
        if let Some(ap) = ap {
            assert!(
                *ap < *bp,
                "matched row (ap={ap}, bp={bp}) violates a.price < b.price — labels swapped?"
            );
        }
    }
}
