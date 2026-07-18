//! Generic traversal helpers for [`SqlExpression`] trees.
//!
//! Before this module every consumer hand-rolled its own `match expr { ... }`
//! over all 24 expression variants, each ending in a `_ => {}` catch-all. The
//! duplication was not harmless: each copy silently skipped whichever variants
//! its author forgot, so a transformer would quietly no-op on `CASE`, method
//! calls, or tuple subqueries rather than fail.
//!
//! The two helpers here are **exhaustive by construction** — neither has a
//! catch-all arm — so adding a variant to `SqlExpression` becomes a compile
//! error in this file instead of a silent miss spread across the codebase.
//!
//! # Direct children only
//!
//! Both helpers visit a node's *direct* children and do not recurse. Callers
//! drive the recursion, which is what lets a transformer intercept the nodes it
//! cares about and delegate everything else:
//!
//! ```ignore
//! fn transform(&self, expr: SqlExpression) -> SqlExpression {
//!     match expr {
//!         SqlExpression::BinaryOp { left, op, right } if op == "ILIKE" => {
//!             /* the one real rule */
//!         }
//!         other => walk::map_children(other, |e| self.transform(e)),
//!     }
//! }
//! ```
//!
//! [`visit_all`] is provided for the common collector case that genuinely wants
//! every node.
//!
//! # Scope boundaries
//!
//! **Subqueries are opaque.** These helpers do not descend into the
//! `SelectStatement` inside `ScalarSubquery`, `InSubquery`, `NotInSubquery`,
//! `InSubqueryTuple` or `NotInSubqueryTuple`, because that statement is a
//! *different query scope*. Descending automatically would be wrong for the
//! alias expanders — a SELECT alias from the outer query must not be expanded
//! inside a subquery that has its own FROM.
//!
//! Same-scope operands of those variants *are* visited: `InSubquery`'s `expr`
//! and `InSubqueryTuple`'s `exprs` belong to the enclosing query, only the
//! `subquery` itself is skipped.
//!
//! A caller that does want to cross the boundary (the CTE hoister, for one)
//! matches those variants explicitly and delegates the rest here.
//!
//! Window specs, by contrast, *are* same-scope: `WindowSpec::order_by` holds
//! real expressions and is descended into. (`partition_by` is `Vec<String>`,
//! so there is nothing to walk.)

use super::ast::{SimpleWhenBranch, SqlExpression, WhenBranch};

/// Rebuild `expr`, replacing each direct child expression with `f(child)`.
///
/// Leaf nodes are returned unchanged. See the module docs for why subqueries
/// are not descended into.
pub fn map_children(
    expr: SqlExpression,
    mut f: impl FnMut(SqlExpression) -> SqlExpression,
) -> SqlExpression {
    match expr {
        // ---- Leaves: nothing to walk ----
        e @ (SqlExpression::Column(_)
        | SqlExpression::StringLiteral(_)
        | SqlExpression::NumberLiteral(_)
        | SqlExpression::BooleanLiteral(_)
        | SqlExpression::Null
        | SqlExpression::DateTimeConstructor { .. }
        | SqlExpression::DateTimeToday { .. }) => e,

        // ---- Scope boundary: the inner statement is deliberately untouched ----
        e @ SqlExpression::ScalarSubquery { .. } => e,

        // ---- Same-scope children ----
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => SqlExpression::MethodCall {
            object,
            method,
            args: args.into_iter().map(&mut f).collect(),
        },

        SqlExpression::ChainedMethodCall { base, method, args } => {
            SqlExpression::ChainedMethodCall {
                base: Box::new(f(*base)),
                method,
                args: args.into_iter().map(&mut f).collect(),
            }
        }

        SqlExpression::FunctionCall {
            name,
            args,
            distinct,
        } => SqlExpression::FunctionCall {
            name,
            args: args.into_iter().map(&mut f).collect(),
            distinct,
        },

        SqlExpression::WindowFunction {
            name,
            args,
            mut window_spec,
        } => {
            let args = args.into_iter().map(&mut f).collect();
            // partition_by is Vec<String>; only order_by carries expressions.
            for item in &mut window_spec.order_by {
                let taken = std::mem::replace(&mut item.expr, SqlExpression::Null);
                item.expr = f(taken);
            }
            SqlExpression::WindowFunction {
                name,
                args,
                window_spec,
            }
        }

        SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
            left: Box::new(f(*left)),
            op,
            right: Box::new(f(*right)),
        },

        SqlExpression::InList { expr, values } => SqlExpression::InList {
            expr: Box::new(f(*expr)),
            values: values.into_iter().map(&mut f).collect(),
        },

        SqlExpression::NotInList { expr, values } => SqlExpression::NotInList {
            expr: Box::new(f(*expr)),
            values: values.into_iter().map(&mut f).collect(),
        },

        SqlExpression::Between { expr, lower, upper } => SqlExpression::Between {
            expr: Box::new(f(*expr)),
            lower: Box::new(f(*lower)),
            upper: Box::new(f(*upper)),
        },

        SqlExpression::Not { expr } => SqlExpression::Not {
            expr: Box::new(f(*expr)),
        },

        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => SqlExpression::CaseExpression {
            when_branches: when_branches
                .into_iter()
                .map(|b| WhenBranch {
                    condition: Box::new(f(*b.condition)),
                    result: Box::new(f(*b.result)),
                })
                .collect(),
            else_branch: else_branch.map(|e| Box::new(f(*e))),
        },

        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => SqlExpression::SimpleCaseExpression {
            expr: Box::new(f(*expr)),
            when_branches: when_branches
                .into_iter()
                .map(|b| SimpleWhenBranch {
                    value: Box::new(f(*b.value)),
                    result: Box::new(f(*b.result)),
                })
                .collect(),
            else_branch: else_branch.map(|e| Box::new(f(*e))),
        },

        SqlExpression::Unnest { column, delimiter } => SqlExpression::Unnest {
            column: Box::new(f(*column)),
            delimiter,
        },

        // ---- Subquery variants: walk the same-scope operand, skip the statement ----
        SqlExpression::InSubquery { expr, subquery } => SqlExpression::InSubquery {
            expr: Box::new(f(*expr)),
            subquery,
        },

        SqlExpression::NotInSubquery { expr, subquery } => SqlExpression::NotInSubquery {
            expr: Box::new(f(*expr)),
            subquery,
        },

        SqlExpression::InSubqueryTuple { exprs, subquery } => SqlExpression::InSubqueryTuple {
            exprs: exprs.into_iter().map(&mut f).collect(),
            subquery,
        },

        SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
            SqlExpression::NotInSubqueryTuple {
                exprs: exprs.into_iter().map(&mut f).collect(),
                subquery,
            }
        }
    }
}

/// Call `f` on each direct child expression of `expr`.
///
/// The borrowing counterpart to [`map_children`], for collectors that only read.
/// Same scope rules apply.
pub fn visit_children<'a>(expr: &'a SqlExpression, mut f: impl FnMut(&'a SqlExpression)) {
    match expr {
        // ---- Leaves: nothing to walk ----
        SqlExpression::Column(_)
        | SqlExpression::StringLiteral(_)
        | SqlExpression::NumberLiteral(_)
        | SqlExpression::BooleanLiteral(_)
        | SqlExpression::Null
        | SqlExpression::DateTimeConstructor { .. }
        | SqlExpression::DateTimeToday { .. } => {}

        // ---- Scope boundary ----
        SqlExpression::ScalarSubquery { .. } => {}

        // ---- Same-scope children ----
        SqlExpression::MethodCall { args, .. } | SqlExpression::FunctionCall { args, .. } => {
            args.iter().for_each(f);
        }

        SqlExpression::ChainedMethodCall { base, args, .. } => {
            f(base);
            args.iter().for_each(f);
        }

        SqlExpression::WindowFunction {
            args, window_spec, ..
        } => {
            args.iter().for_each(&mut f);
            // partition_by is Vec<String>; only order_by carries expressions.
            window_spec.order_by.iter().for_each(|item| f(&item.expr));
        }

        SqlExpression::BinaryOp { left, right, .. } => {
            f(left);
            f(right);
        }

        SqlExpression::InList { expr, values } | SqlExpression::NotInList { expr, values } => {
            f(expr);
            values.iter().for_each(f);
        }

        SqlExpression::Between { expr, lower, upper } => {
            f(expr);
            f(lower);
            f(upper);
        }

        SqlExpression::Not { expr } | SqlExpression::Unnest { column: expr, .. } => f(expr),

        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            for branch in when_branches {
                f(&branch.condition);
                f(&branch.result);
            }
            if let Some(e) = else_branch {
                f(e);
            }
        }

        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => {
            f(expr);
            for branch in when_branches {
                f(&branch.value);
                f(&branch.result);
            }
            if let Some(e) = else_branch {
                f(e);
            }
        }

        // ---- Subquery variants: same-scope operand only ----
        SqlExpression::InSubquery { expr, .. } | SqlExpression::NotInSubquery { expr, .. } => {
            f(expr)
        }

        SqlExpression::InSubqueryTuple { exprs, .. }
        | SqlExpression::NotInSubqueryTuple { exprs, .. } => exprs.iter().for_each(f),
    }
}

/// Call `f` on `expr` and every descendant, pre-order.
///
/// The usual entry point for collectors ("find every column reference",
/// "find every aggregate"). Subquery statements are still not descended into —
/// see the module docs.
pub fn visit_all<'a>(expr: &'a SqlExpression, f: &mut impl FnMut(&'a SqlExpression)) {
    f(expr);
    visit_children(expr, |child| visit_all(child, f));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::ColumnRef;
    use crate::sql::recursive_parser::Parser;

    /// Parse `SELECT <expr> FROM t` and hand back the projected expression.
    fn expr_of(select_expr: &str) -> SqlExpression {
        let sql = format!("SELECT {select_expr} FROM t");
        let stmt = Parser::new(&sql)
            .parse()
            .unwrap_or_else(|e| panic!("{sql} should parse: {e}"));

        stmt.select_items
            .into_iter()
            .find_map(|item| match item {
                crate::sql::parser::ast::SelectItem::Expression { expr, .. } => Some(expr),
                _ => None,
            })
            .expect("expected a projected expression")
    }

    /// Collect every column name reachable from `expr`.
    fn columns(expr: &SqlExpression) -> Vec<String> {
        let mut found = Vec::new();
        visit_all(expr, &mut |e| {
            if let SqlExpression::Column(c) = e {
                found.push(c.name.clone());
            }
        });
        found
    }

    /// Rename every column reference, recursing via `map_children`.
    fn rename_all(expr: SqlExpression, to: &str) -> SqlExpression {
        match expr {
            SqlExpression::Column(_) => SqlExpression::Column(ColumnRef::unquoted(to.to_string())),
            other => map_children(other, |e| rename_all(e, to)),
        }
    }

    #[test]
    fn visit_all_reaches_case_branches() {
        let expr = expr_of("CASE WHEN a > 1 THEN b ELSE c END");
        let mut found = columns(&expr);
        found.sort();
        assert_eq!(found, vec!["a", "b", "c"]);
    }

    #[test]
    fn visit_all_reaches_nested_function_args() {
        let expr = expr_of("UPPER(TRIM(name))");
        assert_eq!(columns(&expr), vec!["name"]);
    }

    #[test]
    fn visit_all_reaches_between_operands() {
        let expr = expr_of("x BETWEEN lo AND hi");
        assert_eq!(columns(&expr), vec!["x", "lo", "hi"]);
    }

    /// The payoff case: `WindowSpec::order_by` holds real expressions, and
    /// every hand-rolled walker in the codebase missed them.
    #[test]
    fn visit_all_reaches_window_order_by() {
        let expr = expr_of("ROW_NUMBER() OVER (ORDER BY created_at)");
        assert!(
            columns(&expr).contains(&"created_at".to_string()),
            "window ORDER BY expressions must be reachable"
        );
    }

    /// Subqueries are a scope boundary: the operand is walked, the inner
    /// statement is not.
    #[test]
    fn walkers_do_not_cross_into_subqueries() {
        let expr = expr_of("(SELECT MAX(inner_col) FROM other)");
        assert!(
            matches!(expr, SqlExpression::ScalarSubquery { .. }),
            "expected a scalar subquery"
        );
        assert!(
            columns(&expr).is_empty(),
            "must not descend into a subquery's own scope"
        );

        // ...but a same-scope operand alongside one still is.
        let stmt = Parser::new("SELECT a FROM t WHERE outer_col IN (SELECT x FROM other)")
            .parse()
            .expect("should parse");
        let cond = &stmt.where_clause.expect("where clause").conditions[0].expr;
        assert_eq!(columns(cond), vec!["outer_col"]);
    }

    #[test]
    fn map_children_rewrites_nested_expressions() {
        let expr = expr_of("CASE WHEN a > 1 THEN UPPER(b) ELSE c END");
        let renamed = rename_all(expr, "z");
        assert_eq!(columns(&renamed), vec!["z", "z", "z"]);
    }

    #[test]
    fn map_children_rewrites_window_order_by() {
        let expr = expr_of("ROW_NUMBER() OVER (ORDER BY created_at)");
        let renamed = rename_all(expr, "z");
        assert_eq!(columns(&renamed), vec!["z"]);
    }

    #[test]
    fn map_children_leaves_leaves_alone() {
        let expr = expr_of("42");
        let mapped = map_children(expr, |_| panic!("a literal has no children"));
        assert!(matches!(mapped, SqlExpression::NumberLiteral(ref n) if n == "42"));
    }
}
