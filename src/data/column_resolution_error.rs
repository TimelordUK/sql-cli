//! The error message produced when a qualified column reference (`a.path`)
//! cannot be resolved.
//!
//! It lives in one place because it was previously written out at three call
//! sites, all saying the same misleading thing: *"Table 'a' may not support
//! qualified column names"*. That explanation is essentially never true —
//! qualified names work — and it sends the reader looking for a feature
//! limitation instead of at their query. It cost real debugging time twice
//! from field use. See T9 in `docs/TUI_FEATURES.md` and P40 in
//! `docs/SQL_PARITY.md`.
//!
//! There are two genuinely different failures behind one message, and the
//! reader needs to be told which one they hit:
//!
//! 1. the prefix names nothing in scope — usually a CTE that was defined but
//!    never put in a `FROM` clause;
//! 2. the prefix is in scope but has no such column — usually a typo.

use crate::data::datatable::DataTable;
use anyhow::anyhow;

/// How many column names to list before truncating.
const MAX_LISTED_COLUMNS: usize = 12;

/// Build the error for an unresolvable `prefix.column` reference.
///
/// `prefix` is what the user wrote; `resolved` is that prefix after alias
/// resolution (pass the same value when the caller has no alias map).
#[must_use]
pub fn qualified_column_not_found(
    table: &DataTable,
    prefix: &str,
    resolved: &str,
    column: &str,
) -> anyhow::Error {
    if prefix_is_in_scope(table, prefix, resolved) {
        anyhow!(
            "Column '{}' not found in '{}'. {}",
            column,
            prefix,
            available_columns(table)
        )
    } else {
        anyhow!(
            "Unknown table or alias '{}' in '{}.{}'. {} \
             A CTE has to be named in a FROM clause before its columns can be referenced.",
            prefix,
            prefix,
            column,
            tables_in_scope(table)
        )
    }
}

/// Whether `prefix` plausibly names the table being queried.
///
/// Deliberately generous — a false "in scope" only costs a slightly less
/// pointed message, whereas a false "unknown table" would reintroduce exactly
/// the kind of confident, wrong explanation this module exists to remove.
fn prefix_is_in_scope(table: &DataTable, prefix: &str, resolved: &str) -> bool {
    if table.name.eq_ignore_ascii_case(prefix) || table.name.eq_ignore_ascii_case(resolved) {
        return true;
    }

    // A JOIN or CTE result carries qualified names like `orders.id`; if any
    // column is qualified with this prefix, the prefix is certainly in scope.
    table.columns.iter().any(|c| {
        c.qualified_name.as_deref().is_some_and(|q| {
            q.split_once('.').is_some_and(|(t, _)| {
                t.eq_ignore_ascii_case(prefix) || t.eq_ignore_ascii_case(resolved)
            })
        })
    })
}

fn tables_in_scope(table: &DataTable) -> String {
    if table.name.is_empty() {
        "The query selects from no named table.".to_string()
    } else {
        format!("The query selects from '{}'.", table.name)
    }
}

fn available_columns(table: &DataTable) -> String {
    let names: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    if names.is_empty() {
        return "It has no columns.".to_string();
    }

    let shown = names.len().min(MAX_LISTED_COLUMNS);
    let mut msg = format!("Available columns: {}", names[..shown].join(", "));
    if names.len() > shown {
        msg.push_str(&format!(" (+{} more)", names.len() - shown));
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataTable};

    fn table_named(name: &str, columns: &[&str]) -> DataTable {
        let mut t = DataTable::new(name);
        for c in columns {
            t.add_column(DataColumn::new(*c));
        }
        t
    }

    #[test]
    fn an_out_of_scope_prefix_names_the_prefix_not_a_missing_feature() {
        // The reported case: `WITH a AS (...) SELECT a.path` with no FROM.
        let t = table_named("", &[]);
        let msg = qualified_column_not_found(&t, "a", "a", "path").to_string();

        assert!(msg.contains("Unknown table or alias 'a'"), "{msg}");
        assert!(msg.contains("FROM"), "{msg}");
        // The old message's central claim must not come back.
        assert!(!msg.contains("qualified column names"), "{msg}");
    }

    #[test]
    fn a_prefix_matching_the_table_reports_the_column_and_lists_alternatives() {
        let t = table_named("a", &["path", "id"]);
        let msg = qualified_column_not_found(&t, "a", "a", "nope").to_string();

        assert!(msg.contains("Column 'nope' not found in 'a'"), "{msg}");
        assert!(msg.contains("path"), "{msg}");
        assert!(msg.contains("id"), "{msg}");
        assert!(!msg.contains("Unknown table"), "{msg}");
    }

    #[test]
    fn an_alias_resolving_to_the_table_counts_as_in_scope() {
        // `FROM orders o` — the user wrote `o`, which resolves to `orders`.
        let t = table_named("orders", &["id"]);
        let msg = qualified_column_not_found(&t, "o", "orders", "nope").to_string();

        assert!(msg.contains("Column 'nope' not found in 'o'"), "{msg}");
        assert!(!msg.contains("Unknown table"), "{msg}");
    }

    #[test]
    fn a_qualified_column_name_puts_its_prefix_in_scope() {
        // JOIN results carry `orders.id`, and the table itself is named
        // something else entirely.
        let mut t = table_named("join_result", &["id"]);
        t.columns[0].qualified_name = Some("orders.id".to_string());
        let msg = qualified_column_not_found(&t, "orders", "orders", "nope").to_string();

        assert!(msg.contains("Column 'nope' not found in 'orders'"), "{msg}");
        assert!(!msg.contains("Unknown table"), "{msg}");
    }

    #[test]
    fn a_long_column_list_is_truncated() {
        let names: Vec<String> = (0..30).map(|i| format!("c{i}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let t = table_named("wide", &refs);
        let msg = qualified_column_not_found(&t, "wide", "wide", "nope").to_string();

        assert!(msg.contains("(+18 more)"), "{msg}");
    }
}
