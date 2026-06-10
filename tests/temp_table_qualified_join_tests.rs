//! Regression tests for two temp-table issues:
//!   1. Qualified column references (`SELECT f.col FROM #tmp f`) in the SELECT
//!      projection used to fail against single base/temp tables because the
//!      projection resolver did a qualified-only lookup with no unqualified
//!      fallback (query_engine::resolve_select_columns).
//!   2. Equi-joins between a string key (e.g. extracted from JSON via SUBSTR)
//!      and an integer key silently produced no matches because the hash index
//!      keyed on the exact DataValue variant (hash_join::canonical_join_key).
//!
//! Both go through the real SELECT INTO path so the temp tables are
//! materialized from the filtered view, exactly as script execution does.

use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::execution::{ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

fn run(executor: &StatementExecutor, context: &mut ExecutionContext, sql: &str) {
    let mut parser = Parser::new(sql);
    let stmt = parser
        .parse()
        .unwrap_or_else(|e| panic!("parse failed for `{sql}`: {e}"));
    executor
        .execute(stmt, context)
        .unwrap_or_else(|e| panic!("exec failed for `{sql}`: {e}"));
}

fn sales_table() -> DataTable {
    let mut table = DataTable::new("sales");
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("product").with_type(DataType::String));
    table.add_column(DataColumn::new("quantity").with_type(DataType::Integer));
    for (id, product, qty) in [
        (1, "Widget", 10),
        (2, "Gadget", 5),
        (3, "Doohickey", 15),
        (4, "Whatsit", 20),
    ] {
        let _ = table.add_row(DataRow {
            values: vec![
                DataValue::Integer(id),
                DataValue::String(product.to_string()),
                DataValue::Integer(qty),
            ],
        });
    }
    table
}

#[test]
fn qualified_alias_in_select_resolves_on_temp_and_base_tables() {
    let mut context = ExecutionContext::new(Arc::new(sales_table()));
    let executor = StatementExecutor::new();

    // Materialize a filtered temp table via the real SELECT INTO path.
    run(
        &executor,
        &mut context,
        "SELECT id, product, quantity INTO #hi FROM sales WHERE quantity > 10",
    );

    // Qualified columns against the aliased temp table (the reported failure).
    let mut p = Parser::new("SELECT f.product, f.quantity FROM #hi f");
    let stmt = p.parse().expect("parse");
    let result = executor
        .execute(stmt, &mut context)
        .expect("qualified alias on temp table should resolve, not error");
    assert_eq!(result.dataview.row_count(), 2); // Doohickey + Whatsit
    assert_eq!(result.dataview.column_count(), 2);

    // Same shape against a plain base table with an alias.
    let mut p2 = Parser::new("SELECT s.product FROM sales s");
    let stmt2 = p2.parse().expect("parse");
    let result2 = executor
        .execute(stmt2, &mut context)
        .expect("qualified alias on base table should resolve");
    assert_eq!(result2.dataview.row_count(), 4);
    assert_eq!(result2.dataview.column_count(), 1);
}

#[test]
fn join_coerces_string_key_to_integer_key() {
    let mut context = ExecutionContext::new(Arc::new(sales_table()));
    let executor = StatementExecutor::new();

    // #agents: integer agent_id.
    let mut agents = DataTable::new("#agents");
    agents.add_column(DataColumn::new("agent_id").with_type(DataType::Integer));
    agents.add_column(DataColumn::new("agent_name").with_type(DataType::String));
    for (id, name) in [(1, "Ann"), (2, "Bob"), (3, "Cat")] {
        let _ = agents.add_row(DataRow {
            values: vec![DataValue::Integer(id), DataValue::String(name.to_string())],
        });
    }
    context
        .store_temp_table("#agents".to_string(), Arc::new(agents))
        .expect("store #agents");

    // #builds: agent reference stored as a STRING (as TeamCity/JSON+SUBSTR yields).
    let mut builds = DataTable::new("#builds");
    builds.add_column(DataColumn::new("build_id").with_type(DataType::Integer));
    builds.add_column(DataColumn::new("f_agent_id").with_type(DataType::String));
    for (bid, aref) in [(10, "1"), (11, "2"), (12, "3"), (13, "99")] {
        let _ = builds.add_row(DataRow {
            values: vec![DataValue::Integer(bid), DataValue::String(aref.to_string())],
        });
    }
    context
        .store_temp_table("#builds".to_string(), Arc::new(builds))
        .expect("store #builds");

    let mut p = Parser::new(
        "SELECT build_id, agent_name \
         FROM #builds b \
         LEFT JOIN #agents a ON b.f_agent_id = a.agent_id",
    );
    let stmt = p.parse().expect("parse");
    let result = executor.execute(stmt, &mut context).expect("join exec");

    // 4 builds; "1"/"2"/"3" coerce-match agents, "99" stays unmatched (NULL).
    assert_eq!(result.dataview.row_count(), 4);
    let src = result.dataview.source();
    let name_idx = src
        .get_column_index("agent_name")
        .expect("agent_name column present in join result");
    let matched = (0..result.dataview.row_count())
        .filter_map(|r| src.get_value(r, name_idx))
        .filter(|v| !matches!(v, DataValue::Null))
        .count();
    assert_eq!(matched, 3, "string keys should coerce-match integer keys");
}

#[test]
fn join_does_not_coerce_when_both_keys_are_strings() {
    // When both join columns are strings, numeric coercion is OFF: exact string
    // equality applies, so "07" does not match "7". Casting both sides to
    // strings is the deliberate opt-out of numeric matching.
    let mut context = ExecutionContext::new(Arc::new(sales_table()));
    let executor = StatementExecutor::new();

    let mut agents = DataTable::new("#agents");
    agents.add_column(DataColumn::new("agent_id").with_type(DataType::String));
    agents.add_column(DataColumn::new("agent_name").with_type(DataType::String));
    for (id, name) in [("7", "Ann"), ("8", "Bob")] {
        let _ = agents.add_row(DataRow {
            values: vec![
                DataValue::String(id.to_string()),
                DataValue::String(name.to_string()),
            ],
        });
    }
    context
        .store_temp_table("#agents".to_string(), Arc::new(agents))
        .expect("store #agents");

    let mut builds = DataTable::new("#builds");
    builds.add_column(DataColumn::new("build_id").with_type(DataType::Integer));
    builds.add_column(DataColumn::new("f_agent_id").with_type(DataType::String));
    for (bid, aref) in [(10, "07"), (11, "8")] {
        let _ = builds.add_row(DataRow {
            values: vec![DataValue::Integer(bid), DataValue::String(aref.to_string())],
        });
    }
    context
        .store_temp_table("#builds".to_string(), Arc::new(builds))
        .expect("store #builds");

    let mut p = Parser::new(
        "SELECT build_id, agent_name \
         FROM #builds b \
         LEFT JOIN #agents a ON b.f_agent_id = a.agent_id",
    );
    let stmt = p.parse().expect("parse");
    let result = executor.execute(stmt, &mut context).expect("join exec");

    assert_eq!(result.dataview.row_count(), 2);
    let src = result.dataview.source();
    let name_idx = src
        .get_column_index("agent_name")
        .expect("agent_name column present");
    let matched = (0..result.dataview.row_count())
        .filter_map(|r| src.get_value(r, name_idx))
        .filter(|v| !matches!(v, DataValue::Null))
        .count();
    // Only "8" == "8" matches; "07" != "7" because strings are not coerced.
    assert_eq!(matched, 1, "string vs string must use exact text equality");
}
