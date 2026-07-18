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
//! **Subqueries are opaque by default.** [`map_children`] and
//! [`visit_children`] do not descend into the `SelectStatement` inside
//! `ScalarSubquery`, `InSubquery`, `NotInSubquery`, `InSubqueryTuple` or
//! `NotInSubqueryTuple`, because that statement is a *different query scope*.
//! Descending automatically would be wrong for the alias expanders — a SELECT
//! alias from the outer query must not be expanded inside a subquery that has
//! its own FROM.
//!
//! Same-scope operands of those variants *are* visited: `InSubquery`'s `expr`
//! and `InSubqueryTuple`'s `exprs` belong to the enclosing query, only the
//! `subquery` itself is skipped.
//!
//! # Crossing the boundary
//!
//! Some transformers legitimately need to cross it — `ILIKE` -> `LIKE` is
//! scope-independent, INTO removal and CTE hoisting have to reach nested
//! statements by definition. Those callers use
//! [`map_children_crossing`] / [`visit_children_crossing`], which take a second
//! closure for the nested statement.
//!
//! Both closures take an explicit `ctx` parameter rather than capturing what
//! they need. That is forced, not stylistic: a transformer whose recursion is
//! `&mut self` (the CTE hoister) cannot hand out two closures that each capture
//! `self` mutably. Threading the state through as `ctx` gives one mutable
//! borrow, split across the two calls by the helper.
//!
//! **The crossing forms are the primitives.** `map_children` is defined as
//! `map_children_crossing` with an identity statement handler, and
//! `visit_children` as `visit_children_crossing` with a no-op one. This is
//! deliberate: it means the set of subquery-bearing variants is written down
//! **exactly once in the codebase**, in this file. A caller that hand-listed
//! those variants itself would compile clean — and silently stop crossing —
//! the day a new one is added (`Exists`, for instance). Here, adding a variant
//! is a compile error in one place.
//!
//! Window specs, by contrast, *are* same-scope: `WindowSpec::order_by` holds
//! real expressions and is descended into. (`partition_by` is `Vec<String>`,
//! so there is nothing to walk.)

use super::ast::{SelectStatement, SimpleWhenBranch, SqlExpression, WhenBranch};

/// Rebuild `expr`, replacing each direct child expression with `f(child)`.
///
/// Leaf nodes are returned unchanged. Subquery statements are a scope boundary
/// and are **not** descended into — see the module docs. Use
/// [`map_children_crossing`] when you need to rewrite them too.
pub fn map_children(
    expr: SqlExpression,
    mut f: impl FnMut(SqlExpression) -> SqlExpression,
) -> SqlExpression {
    // The closure is its own context; the identity statement handler is what
    // makes subqueries opaque. It hands the `Box` straight back, so the opaque
    // path -- which is most callers -- does no work at all for a subquery.
    map_children_crossing(expr, &mut f, |f, e| f(e), |_, stmt| stmt)
}

/// Rebuild `expr`, replacing each direct child expression with `f(ctx, child)`
/// **and** each directly nested subquery statement with `f_stmt(ctx, stmt)`.
///
/// This is the primitive [`map_children`] is built on; it is the only
/// exhaustive match over `SqlExpression` in the rewrite path. Callers that must
/// reach into nested statements (CTE hoisting, INTO removal, scope-independent
/// operator rewrites) use this instead of hand-listing the subquery variants,
/// so a newly added subquery-bearing variant breaks the build here rather than
/// being silently skipped at each call site.
///
/// `ctx` carries whatever mutable state the two closures share — typically the
/// transformer itself. See the module docs for why it is a parameter rather
/// than a capture.
///
/// `f_stmt` takes and returns the `Box`, not the statement, so that the opaque
/// case ([`map_children`], whose handler is `|_, stmt| stmt`) is a passthrough
/// rather than an unbox/realloc of a large struct at every subquery.
pub fn map_children_crossing<C>(
    expr: SqlExpression,
    ctx: &mut C,
    mut f: impl FnMut(&mut C, SqlExpression) -> SqlExpression,
    mut f_stmt: impl FnMut(&mut C, Box<SelectStatement>) -> Box<SelectStatement>,
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

        // ---- Scope boundary: only `f_stmt` may touch the inner statement ----
        SqlExpression::ScalarSubquery { query } => SqlExpression::ScalarSubquery {
            query: f_stmt(ctx, query),
        },

        // ---- Same-scope children ----
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => SqlExpression::MethodCall {
            object,
            method,
            args: args.into_iter().map(|e| f(&mut *ctx, e)).collect(),
        },

        SqlExpression::ChainedMethodCall { base, method, args } => {
            SqlExpression::ChainedMethodCall {
                base: Box::new(f(&mut *ctx, *base)),
                method,
                args: args.into_iter().map(|e| f(&mut *ctx, e)).collect(),
            }
        }

        SqlExpression::FunctionCall {
            name,
            args,
            distinct,
        } => SqlExpression::FunctionCall {
            name,
            args: args.into_iter().map(|e| f(&mut *ctx, e)).collect(),
            distinct,
        },

        SqlExpression::WindowFunction {
            name,
            args,
            mut window_spec,
        } => {
            let args = args.into_iter().map(|e| f(&mut *ctx, e)).collect();
            // partition_by is Vec<String>; only order_by carries expressions.
            for item in &mut window_spec.order_by {
                let taken = std::mem::replace(&mut item.expr, SqlExpression::Null);
                item.expr = f(&mut *ctx, taken);
            }
            SqlExpression::WindowFunction {
                name,
                args,
                window_spec,
            }
        }

        SqlExpression::BinaryOp { left, op, right } => SqlExpression::BinaryOp {
            left: Box::new(f(&mut *ctx, *left)),
            op,
            right: Box::new(f(&mut *ctx, *right)),
        },

        SqlExpression::InList { expr, values } => SqlExpression::InList {
            expr: Box::new(f(&mut *ctx, *expr)),
            values: values.into_iter().map(|e| f(&mut *ctx, e)).collect(),
        },

        SqlExpression::NotInList { expr, values } => SqlExpression::NotInList {
            expr: Box::new(f(&mut *ctx, *expr)),
            values: values.into_iter().map(|e| f(&mut *ctx, e)).collect(),
        },

        SqlExpression::Between { expr, lower, upper } => SqlExpression::Between {
            expr: Box::new(f(&mut *ctx, *expr)),
            lower: Box::new(f(&mut *ctx, *lower)),
            upper: Box::new(f(&mut *ctx, *upper)),
        },

        SqlExpression::Not { expr } => SqlExpression::Not {
            expr: Box::new(f(&mut *ctx, *expr)),
        },

        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => SqlExpression::CaseExpression {
            when_branches: when_branches
                .into_iter()
                .map(|b| WhenBranch {
                    condition: Box::new(f(&mut *ctx, *b.condition)),
                    result: Box::new(f(&mut *ctx, *b.result)),
                })
                .collect(),
            else_branch: else_branch.map(|e| Box::new(f(&mut *ctx, *e))),
        },

        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => SqlExpression::SimpleCaseExpression {
            expr: Box::new(f(&mut *ctx, *expr)),
            when_branches: when_branches
                .into_iter()
                .map(|b| SimpleWhenBranch {
                    value: Box::new(f(&mut *ctx, *b.value)),
                    result: Box::new(f(&mut *ctx, *b.result)),
                })
                .collect(),
            else_branch: else_branch.map(|e| Box::new(f(&mut *ctx, *e))),
        },

        SqlExpression::Unnest { column, delimiter } => SqlExpression::Unnest {
            column: Box::new(f(&mut *ctx, *column)),
            delimiter,
        },

        // ---- Subquery variants: same-scope operand via `f`, statement via `f_stmt` ----
        SqlExpression::InSubquery { expr, subquery } => SqlExpression::InSubquery {
            expr: Box::new(f(&mut *ctx, *expr)),
            subquery: f_stmt(ctx, subquery),
        },

        SqlExpression::NotInSubquery { expr, subquery } => SqlExpression::NotInSubquery {
            expr: Box::new(f(&mut *ctx, *expr)),
            subquery: f_stmt(ctx, subquery),
        },

        SqlExpression::InSubqueryTuple { exprs, subquery } => SqlExpression::InSubqueryTuple {
            exprs: exprs.into_iter().map(|e| f(&mut *ctx, e)).collect(),
            subquery: f_stmt(ctx, subquery),
        },

        SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
            SqlExpression::NotInSubqueryTuple {
                exprs: exprs.into_iter().map(|e| f(&mut *ctx, e)).collect(),
                subquery: f_stmt(ctx, subquery),
            }
        }
    }
}

/// Call `f` on each direct child expression of `expr`.
///
/// The borrowing counterpart to [`map_children`], for collectors that only read.
/// Same scope rules apply: subquery statements are not visited. Use
/// [`visit_children_crossing`] when you need to reach them.
pub fn visit_children<'a>(expr: &'a SqlExpression, mut f: impl FnMut(&'a SqlExpression)) {
    // The closure is its own context; the no-op statement handler is what makes
    // subqueries opaque.
    visit_children_crossing(expr, &mut f, |f, e| f(e), |_, _| {});
}

/// Call `f` on each direct child expression of `expr` **and** `f_stmt` on each
/// directly nested subquery statement.
///
/// The borrowing counterpart to [`map_children_crossing`], and the primitive
/// [`visit_children`] is built on. As there, the subquery-bearing variants are
/// listed only here, so a new one is a compile error rather than a silent skip
/// at every call site, and `ctx` carries the state the two closures share.
pub fn visit_children_crossing<'a, C>(
    expr: &'a SqlExpression,
    ctx: &mut C,
    mut f: impl FnMut(&mut C, &'a SqlExpression),
    mut f_stmt: impl FnMut(&mut C, &'a SelectStatement),
) {
    match expr {
        // ---- Leaves: nothing to walk ----
        SqlExpression::Column(_)
        | SqlExpression::StringLiteral(_)
        | SqlExpression::NumberLiteral(_)
        | SqlExpression::BooleanLiteral(_)
        | SqlExpression::Null
        | SqlExpression::DateTimeConstructor { .. }
        | SqlExpression::DateTimeToday { .. } => {}

        // ---- Scope boundary: only `f_stmt` may see the inner statement ----
        SqlExpression::ScalarSubquery { query } => f_stmt(ctx, query),

        // ---- Same-scope children ----
        SqlExpression::MethodCall { args, .. } | SqlExpression::FunctionCall { args, .. } => {
            args.iter().for_each(|e| f(&mut *ctx, e));
        }

        SqlExpression::ChainedMethodCall { base, args, .. } => {
            f(&mut *ctx, base);
            args.iter().for_each(|e| f(&mut *ctx, e));
        }

        SqlExpression::WindowFunction {
            args, window_spec, ..
        } => {
            args.iter().for_each(|e| f(&mut *ctx, e));
            // partition_by is Vec<String>; only order_by carries expressions.
            window_spec
                .order_by
                .iter()
                .for_each(|item| f(&mut *ctx, &item.expr));
        }

        SqlExpression::BinaryOp { left, right, .. } => {
            f(&mut *ctx, left);
            f(&mut *ctx, right);
        }

        SqlExpression::InList { expr, values } | SqlExpression::NotInList { expr, values } => {
            f(&mut *ctx, expr);
            values.iter().for_each(|e| f(&mut *ctx, e));
        }

        SqlExpression::Between { expr, lower, upper } => {
            f(&mut *ctx, expr);
            f(&mut *ctx, lower);
            f(&mut *ctx, upper);
        }

        SqlExpression::Not { expr } | SqlExpression::Unnest { column: expr, .. } => {
            f(&mut *ctx, expr)
        }

        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            for branch in when_branches {
                f(&mut *ctx, &branch.condition);
                f(&mut *ctx, &branch.result);
            }
            if let Some(e) = else_branch {
                f(&mut *ctx, e);
            }
        }

        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => {
            f(&mut *ctx, expr);
            for branch in when_branches {
                f(&mut *ctx, &branch.value);
                f(&mut *ctx, &branch.result);
            }
            if let Some(e) = else_branch {
                f(&mut *ctx, e);
            }
        }

        // ---- Subquery variants: same-scope operand via `f`, statement via `f_stmt` ----
        SqlExpression::InSubquery { expr, subquery }
        | SqlExpression::NotInSubquery { expr, subquery } => {
            f(&mut *ctx, expr);
            f_stmt(ctx, subquery);
        }

        SqlExpression::InSubqueryTuple { exprs, subquery }
        | SqlExpression::NotInSubqueryTuple { exprs, subquery } => {
            exprs.iter().for_each(|e| f(&mut *ctx, e));
            f_stmt(ctx, subquery);
        }
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

    /// The mirror of the test above, and the property the whole `crossing`
    /// split exists to provide: the same variants that `map_children` treats as
    /// opaque *must* be reachable through the crossing forms. If a new
    /// subquery-bearing variant is ever added and wired only into the leaves,
    /// this is what notices.
    #[test]
    fn crossing_walkers_do_reach_into_subqueries() {
        // Visit side: the nested statement is handed to `f_stmt`.
        let expr = expr_of("(SELECT MAX(inner_col) FROM other)");
        let mut statements_seen = 0;
        visit_children_crossing(&expr, &mut statements_seen, |_, _| {}, |n, _| *n += 1);
        assert_eq!(
            statements_seen, 1,
            "a scalar subquery's statement must be reachable when crossing"
        );

        // Map side: and it can be rewritten in place.
        let rewritten = map_children_crossing(
            expr,
            &mut (),
            |_, e| e,
            |_, mut stmt| {
                stmt.limit = Some(1);
                stmt
            },
        );
        match rewritten {
            SqlExpression::ScalarSubquery { query } => assert_eq!(query.limit, Some(1)),
            other => panic!("expected a scalar subquery, got {other:?}"),
        }
    }

    /// The tuple forms carry *both* same-scope operands and a nested statement;
    /// crossing must reach both, not one or the other.
    #[test]
    fn crossing_reaches_tuple_subquery_operands_and_statement() {
        let stmt = Parser::new("SELECT a FROM t WHERE (a, b) IN (SELECT x, y FROM u)")
            .parse()
            .expect("should parse");
        let cond = &stmt.where_clause.expect("where clause").conditions[0].expr;

        let mut ctx = (Vec::<String>::new(), 0);
        visit_children_crossing(
            cond,
            &mut ctx,
            |ctx, e| {
                if let SqlExpression::Column(c) = e {
                    ctx.0.push(c.name.clone());
                }
            },
            |ctx, _| ctx.1 += 1,
        );

        assert_eq!(ctx.0, vec!["a", "b"], "same-scope operands must be visited");
        assert_eq!(ctx.1, 1, "the subquery statement must be visited too");
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
