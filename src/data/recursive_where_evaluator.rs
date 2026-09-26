use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::query_engine::ExecutionContext;
use crate::data::trilean::Trilean;
use crate::sql::recursive_parser::{Condition, LogicalOp, SqlExpression, WhereClause};
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use tracing::debug;

/// Evaluates WHERE clauses from `recursive_parser` directly against `DataTable`
pub struct RecursiveWhereEvaluator<'a, 'exec> {
    table: &'a DataTable,
    case_insensitive: bool,
    exec_context: Option<&'exec ExecutionContext>,
    /// One `ArithmeticEvaluator` for the whole evaluation, built on first use.
    /// Constructing one builds the function and aggregate registries, and its
    /// window-context cache only pays off if it outlives a single row -- a
    /// fresh evaluator per row made an inline window predicate quadratic.
    arithmetic: RefCell<Option<ArithmeticEvaluator<'a>>>,
}

impl<'a, 'exec> RecursiveWhereEvaluator<'a, 'exec> {
    #[must_use]
    pub fn new(table: &'a DataTable) -> RecursiveWhereEvaluator<'a, 'static> {
        Self::with_case_insensitive(table, false)
    }

    #[must_use]
    pub fn with_case_insensitive(
        table: &'a DataTable,
        case_insensitive: bool,
    ) -> RecursiveWhereEvaluator<'a, 'static> {
        RecursiveWhereEvaluator {
            table,
            case_insensitive,
            exec_context: None,
            arithmetic: RefCell::new(None),
        }
    }

    /// Create evaluator with execution context for alias resolution
    pub fn with_exec_context(
        table: &'a DataTable,
        exec_context: &'exec ExecutionContext,
        case_insensitive: bool,
    ) -> Self {
        Self {
            table,
            case_insensitive,
            exec_context: Some(exec_context),
            arithmetic: RefCell::new(None),
        }
    }

    /// Evaluate a WHERE clause for a specific row
    pub fn evaluate(&mut self, where_clause: &WhereClause, row_index: usize) -> Result<Trilean> {
        // Only log for first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate() ENTRY - row {}, {} conditions, case_insensitive={}",
                row_index,
                where_clause.conditions.len(),
                self.case_insensitive
            );
        }

        if where_clause.conditions.is_empty() {
            if row_index < 3 {
                debug!("RecursiveWhereEvaluator: evaluate() EXIT - no conditions, returning true");
            }
            return Ok(Trilean::True);
        }

        // With the new expression tree structure, we should have a single condition
        // containing the entire WHERE clause expression tree
        if where_clause.conditions.len() == 1 {
            // New structure: single expression tree
            if row_index < 3 {
                debug!(
                    "RecursiveWhereEvaluator: evaluate() - evaluating expression tree for row {}",
                    row_index
                );
            }
            self.evaluate_condition(&where_clause.conditions[0], row_index)
        } else {
            // Legacy structure: multiple conditions with connectors
            // This path is kept for backward compatibility
            if row_index < 3 {
                debug!(
                    "RecursiveWhereEvaluator: evaluate() - evaluating {} conditions with connectors for row {}",
                    where_clause.conditions.len(),
                    row_index
                );
            }
            let mut result = self.evaluate_condition(&where_clause.conditions[0], row_index)?;

            // Apply connectors (AND/OR) with subsequent conditions
            for i in 1..where_clause.conditions.len() {
                let next_result =
                    self.evaluate_condition(&where_clause.conditions[i], row_index)?;

                // Use the connector from the previous condition
                if let Some(connector) = &where_clause.conditions[i - 1].connector {
                    result = match connector {
                        LogicalOp::And => result.and(next_result),
                        LogicalOp::Or => result.or(next_result),
                    };
                }
            }

            Ok(result)
        }
    }

    fn evaluate_condition(&mut self, condition: &Condition, row_index: usize) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_condition() ENTRY - row {}",
                row_index
            );
        }
        let result = self.evaluate_expression(&condition.expr, row_index);
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_condition() EXIT - row {}, result = {:?}",
                row_index, result
            );
        }
        result
    }

    fn evaluate_expression(&mut self, expr: &SqlExpression, row_index: usize) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_expression() ENTRY - row {}, expr = {:?}",
                row_index, expr
            );
        }

        let result = match expr {
            SqlExpression::BinaryOp { left, op, right } => {
                self.evaluate_binary_op(expr, left, op, right, row_index)
            }
            SqlExpression::InList {
                expr: probe,
                values,
            }
            | SqlExpression::NotInList {
                expr: probe,
                values,
            } => {
                reject_unlifted_window(probe)?;
                for item in values {
                    reject_unlifted_window(item)?;
                }
                self.evaluate_delegated(expr, row_index)
            }
            SqlExpression::Between {
                expr: value,
                lower,
                upper,
            } => {
                for operand in [value, lower, upper] {
                    reject_unlifted_window(operand)?;
                }
                self.evaluate_delegated(expr, row_index)
            }
            SqlExpression::Not { expr } => {
                let inner_result = self.evaluate_expression(expr, row_index)?;
                // `NOT UNKNOWN` is UNKNOWN, not TRUE — see P18/P19.
                Ok(inner_result.negate())
            }
            // `WHERE s.Contains('x')`: the registry's method function, through
            // the value evaluator (R13 slice 5).
            SqlExpression::MethodCall { .. } => {
                reject_unlifted_window(expr)?;
                self.evaluate_delegated(expr, row_index)
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                if row_index < 3 {
                    debug!("RecursiveWhereEvaluator: evaluate_expression() - found CaseExpression, evaluating");
                }
                self.evaluate_case_expression_as_bool(when_branches, else_branch, row_index)
            }
            // A bare value expression used as a predicate -- `WHERE flag`,
            // `WHERE true`, and the `WHERE lifted_value` that
            // `ExpressionLifter` rewrites a window comparison into. This arm
            // used to answer FALSE for every row, which is how P37 turned an
            // unsupported shape into a silently empty result set.
            _ => self.evaluate_value_as_predicate(expr, row_index),
        };

        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_expression() EXIT - row {}, result = {:?}",
                row_index, result
            );
        }
        result
    }

    /// `expr` is the `BinaryOp` node itself, and `left` / `op` / `right` its
    /// parts: the arms that hand the whole predicate to the value evaluator
    /// need the node, and rebuilding it would clone the subtree on every row.
    fn evaluate_binary_op(
        &mut self,
        expr: &SqlExpression,
        left: &SqlExpression,
        op: &str,
        right: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        // Only log first few rows to avoid performance impact
        if row_index < 3 {
            debug!(
                "RecursiveWhereEvaluator: evaluate_binary_op() ENTRY - row {}, op = '{}'",
                row_index, op
            );
        }

        // Handle logical operators (AND, OR) specially
        if op.to_uppercase() == "OR" || op.to_uppercase() == "AND" {
            let left_result = self.evaluate_expression(left, row_index)?;
            let right_result = self.evaluate_expression(right, row_index)?;

            return Ok(match op.to_uppercase().as_str() {
                // `Trilean::or` / `and` are the SQL truth tables, so FALSE still
                // dominates AND and TRUE still dominates OR even when the other
                // side is UNKNOWN — which `||`/`&&` over a collapsed bool could
                // not express.
                "OR" => left_result.or(right_result),
                "AND" => left_result.and(right_result),
                _ => unreachable!(),
            });
        }

        let op_upper = op.to_uppercase();

        // An operator that does not yield a truth value (`WHERE a + b`,
        // `WHERE x || y`) makes the whole expression a value used as a
        // predicate -- the P37 rule, rather than a comparison under an
        // operator `compare_with_op` would silently answer false for.
        if !is_predicate_operator(&op_upper) {
            return self.evaluate_value_as_predicate(expr, row_index);
        }

        // Comparisons, NULL tests and LIKE are the value evaluator's (R13
        // slices 4 and 5): its operand readers, three-valued `compare_trilean`,
        // `sql_like` and the registry's method functions are the one
        // implementation, whatever shape either operand takes.
        reject_unlifted_window(left)?;
        reject_unlifted_window(right)?;
        self.evaluate_delegated(expr, row_index)
    }

    /// Evaluate a value expression with the evaluation's shared
    /// `ArithmeticEvaluator` (see the `arithmetic` field for why it is shared).
    fn evaluate_arithmetic(&self, expr: &SqlExpression, row_index: usize) -> Result<DataValue> {
        let mut slot = self.arithmetic.borrow_mut();
        slot.get_or_insert_with(|| {
            let evaluator =
                ArithmeticEvaluator::new(self.table).with_case_insensitive(self.case_insensitive);
            // The same aliases WHERE's own column operands resolve through,
            // so `i.a` means the same column inside an expression operand.
            match self.exec_context {
                Some(exec_ctx) => evaluator.with_table_aliases(exec_ctx.get_aliases()),
                None => evaluator,
            }
        })
        .evaluate(expr, row_index)
    }

    /// Evaluate a predicate with the value evaluator and read its truth value
    /// back, NULL as UNKNOWN (R13 slice 4). An arm that delegates here has no
    /// rules of its own left in this file: the value evaluator's arm is the
    /// only implementation, and this is the one place its answer crosses back
    /// into a `Trilean`.
    fn evaluate_delegated(&self, expr: &SqlExpression, row_index: usize) -> Result<Trilean> {
        let value = self.evaluate_arithmetic(expr, row_index)?;
        Trilean::from_value(&value)
            .ok_or_else(|| anyhow!("Predicate did not evaluate to a truth value: {value:?}"))
    }

    /// Evaluate a CASE expression as a boolean (for WHERE clauses)
    fn evaluate_case_expression_as_bool(
        &mut self,
        when_branches: &[crate::sql::recursive_parser::WhenBranch],
        else_branch: &Option<Box<SqlExpression>>,
        row_index: usize,
    ) -> Result<Trilean> {
        debug!(
            "RecursiveWhereEvaluator: evaluating CASE expression as bool for row {}",
            row_index
        );

        // Evaluate each WHEN condition in order
        for branch in when_branches {
            // Evaluate the condition as a boolean
            let condition_result = self.evaluate_expression(&branch.condition, row_index)?;

            // A WHEN whose condition is UNKNOWN does not match, exactly like
            // FALSE — the same rule the row filter applies.
            if condition_result.is_true() {
                debug!("CASE: WHEN condition matched, evaluating result expression as bool");
                // Evaluate the result and convert to boolean
                return self.evaluate_expression_as_bool(&branch.result, row_index);
            }
        }

        // If no WHEN condition matched, evaluate ELSE clause (or return false)
        if let Some(else_expr) = else_branch {
            debug!("CASE: No WHEN matched, evaluating ELSE expression as bool");
            self.evaluate_expression_as_bool(else_expr, row_index)
        } else {
            debug!("CASE: No WHEN matched and no ELSE, returning false");
            Ok(Trilean::False)
        }
    }

    /// Evaluate an expression for its VALUE and coerce that value to a
    /// predicate (P37).
    ///
    /// This is the tail of both `evaluate_expression` (a bare value used
    /// directly as a `WHERE` predicate: `WHERE flag`, `WHERE true`, and the
    /// `WHERE lifted_value` that `ExpressionLifter` rewrites a window
    /// comparison into) and `evaluate_expression_as_bool` (the result of a
    /// CASE branch). Both used to have their own copy of the coercion table
    /// and disagreed on NULL; they now share this one.
    ///
    /// A raw `WindowFunction` reaching here means the lifter did not hoist it,
    /// which is a defect rather than a value to coerce -- so it errors. That
    /// is the P37 rule: a loud failure beats a silently empty result.
    fn evaluate_value_as_predicate(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        if let SqlExpression::WindowFunction { name, .. } = expr {
            return Err(anyhow::anyhow!(
                "Window function {name} cannot be used directly as a predicate (the expression was not lifted to a CTE column)"
            ));
        }

        let value = self.evaluate_arithmetic(expr, row_index)?;

        use crate::data::datatable::DataValue;
        Ok(match value {
            DataValue::Boolean(b) => Trilean::from_bool(b),
            DataValue::Integer(i) => Trilean::from_bool(i != 0),
            DataValue::Float(f) => Trilean::from_bool(f != 0.0),
            // A NULL predicate is UNKNOWN, not FALSE. Under `WHERE` alone the
            // two are indistinguishable -- both drop the row -- and they only
            // diverge under `NOT`, which is exactly the P18/P19 trap. The CASE
            // path used to answer FALSE here; this is the one deliberate
            // behaviour change in unifying the two copies.
            DataValue::Null => Trilean::Unknown,
            DataValue::String(ref s) => Trilean::from_bool(!s.is_empty()),
            DataValue::InternedString(ref s) => Trilean::from_bool(!s.is_empty()),
            _ => Trilean::True,
        })
    }

    fn evaluate_expression_as_bool(
        &mut self,
        expr: &SqlExpression,
        row_index: usize,
    ) -> Result<Trilean> {
        match expr {
            // For expressions that naturally return booleans, use the existing evaluator
            SqlExpression::BinaryOp { .. }
            | SqlExpression::InList { .. }
            | SqlExpression::NotInList { .. }
            | SqlExpression::Between { .. }
            | SqlExpression::Not { .. }
            | SqlExpression::MethodCall { .. } => self.evaluate_expression(expr, row_index),
            // For CASE expressions, recurse
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => self.evaluate_case_expression_as_bool(when_branches, else_branch, row_index),
            // For other expressions (columns, literals), evaluate the value
            // and coerce -- the same rule the WHERE predicate path uses.
            _ => self.evaluate_value_as_predicate(expr, row_index),
        }
    }
}

/// A raw window function in a WHERE operand was not lifted to a CTE column.
/// Evaluating it would compute the window over the rows the WHERE is still
/// filtering, so an inline `QUALIFY ROW_NUMBER() OVER (...) = 1` combined with
/// a WHERE would silently rank the unfiltered table (P15). The value evaluator
/// would evaluate one quietly, so this check stays on WHERE's side of every arm
/// that delegates to it.
fn reject_unlifted_window(expr: &SqlExpression) -> Result<()> {
    match expr {
        SqlExpression::WindowFunction { name, .. } => Err(anyhow!(
            "Window function {name} cannot be used directly in a comparison (the expression was not lifted to a CTE column)"
        )),
        _ => Ok(()),
    }
}

/// Operators whose result is a truth value - the comparisons, LIKE and the NULL
/// tests - which WHERE hands whole to the value evaluator. Expects the operator
/// already upper-cased.
fn is_predicate_operator(op_upper: &str) -> bool {
    matches!(
        op_upper,
        "=" | "!=" | "<>" | "<" | "<=" | ">" | ">=" | "LIKE" | "IS NULL" | "IS NOT NULL"
    )
}

#[cfg(test)]
mod three_valued_logic_tests {
    //! Regression tests for P18/P19 — SQL three-valued logic in `WHERE`.
    //!
    //! These assert the `Trilean` the evaluator produces for a single row,
    //! rather than the rows a query returns, because that is where the defect
    //! actually lived: UNKNOWN and FALSE are indistinguishable by row count
    //! under `WHERE` (both drop the row) and only diverge under `NOT`. A
    //! row-counting test would have passed against the broken evaluator for
    //! half of these cases.
    //!
    //! They also run without DuckDB, unlike the corpus cases in
    //! `tests/comparison/corpus/` that pin the same behaviour end to end.

    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};
    use crate::sql::recursive_parser::Parser;

    /// Rows: 0 = score 50, 1 = score 30, 2 = score NULL.
    fn table_with_nulls() -> DataTable {
        let mut table = DataTable::new("t");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("score"));
        table.add_column(DataColumn::new("label"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::Integer(50),
                DataValue::String("alpha".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Integer(30),
                DataValue::String("beta".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Null,
                DataValue::Null,
            ]))
            .unwrap();

        table
    }

    /// Evaluate a WHERE clause against one row and return its truth value.
    fn eval(table: &DataTable, predicate: &str, row: usize) -> Trilean {
        let sql = format!("SELECT * FROM t WHERE {predicate}");
        let mut parser = Parser::new(&sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(table);
        evaluator
            .evaluate(&where_clause, row)
            .expect("evaluation failed")
    }

    // --- P18: `= NULL` yields UNKNOWN, never a match ---

    #[test]
    fn equals_null_is_unknown_for_every_row() {
        let t = table_with_nulls();
        // Including the NULL row itself: `NULL = NULL` is UNKNOWN, not TRUE.
        // Returning TRUE here is exactly what made `WHERE score = NULL` behave
        // like `IS NULL`.
        for row in 0..3 {
            assert_eq!(eval(&t, "score = NULL", row), Trilean::Unknown, "row {row}");
        }
    }

    #[test]
    fn comparison_against_a_null_column_is_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score > 10", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score <> 50", 2), Trilean::Unknown);
    }

    #[test]
    fn is_null_stays_two_valued() {
        // The sanctioned way to match a NULL must keep working — that is what
        // makes losing `= NULL` costless.
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score IS NULL", 2), Trilean::True);
        assert_eq!(eval(&t, "score IS NULL", 0), Trilean::False);
        assert_eq!(eval(&t, "score IS NOT NULL", 0), Trilean::True);
        assert_eq!(eval(&t, "score IS NOT NULL", 2), Trilean::False);
    }

    #[test]
    fn in_list_with_a_null_matches_only_real_equals() {
        let t = table_with_nulls();
        // A match still wins outright, even with a NULL in the list.
        assert_eq!(eval(&t, "score IN (50, NULL)", 0), Trilean::True);
        // No match, but a NULL was compared: UNKNOWN, not FALSE. Returning
        // FALSE here is invisible under IN and wrong under NOT IN.
        assert_eq!(eval(&t, "score IN (50, NULL)", 1), Trilean::Unknown);
        // NULL column against a list with no NULL in it: also UNKNOWN.
        assert_eq!(eval(&t, "score IN (50, 70)", 2), Trilean::Unknown);
        // Nothing NULL anywhere: ordinary FALSE.
        assert_eq!(eval(&t, "score IN (50, 70)", 1), Trilean::False);
    }

    // --- P19: NOT must not turn UNKNOWN into TRUE ---

    #[test]
    fn not_in_excludes_nulls() {
        let t = table_with_nulls();
        // The bug: UNKNOWN collapsed to false and `!false` admitted the row.
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 0), Trilean::False);
        assert_eq!(eval(&t, "score NOT IN (50, 70)", 1), Trilean::True);
    }

    #[test]
    fn not_over_an_unknown_comparison_stays_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "NOT (score > 50)", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT (score > 50)", 1), Trilean::True);
    }

    // --- The truth tables, exercised through real predicates ---

    /// CONTROL — passes with and without the P18/P19 fix, because the old
    /// bool evaluator also produced FALSE here (for the wrong reason: it
    /// collapsed the UNKNOWN rather than letting FALSE dominate). Kept because
    /// it pins the half of AND's truth table the fix must not disturb; do not
    /// read a pass here as evidence the fix works.
    #[test]
    fn false_still_dominates_and_over_unknown() {
        let t = table_with_nulls();
        // Row 1 has score 30, so `score = 50` is FALSE. FALSE AND UNKNOWN is
        // FALSE — the row is excluded for a definite reason, not an unknown
        // one. Getting this wrong would make the NOT of it wrong too.
        assert_eq!(eval(&t, "score = 50 AND label = NULL", 1), Trilean::False);
        assert_eq!(
            eval(&t, "NOT (score = 50 AND label = NULL)", 1),
            Trilean::True
        );
    }

    #[test]
    fn true_still_dominates_or_over_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50 OR label = NULL", 0), Trilean::True);
        // Neither side definite: UNKNOWN.
        assert_eq!(eval(&t, "score = 99 OR label = NULL", 1), Trilean::Unknown);
    }

    #[test]
    fn between_follows_ands_truth_table() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 0), Trilean::True);
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 1), Trilean::False);
        // NULL operand: UNKNOWN, and so is its negation.
        assert_eq!(eval(&t, "score BETWEEN 40 AND 60", 2), Trilean::Unknown);
        assert_eq!(
            eval(&t, "NOT (score BETWEEN 40 AND 60)", 2),
            Trilean::Unknown
        );
        // Row 1 is 30, below the lower bound, so the answer is FALSE outright
        // even though the upper bound is NULL and that comparison is UNKNOWN.
        assert_eq!(eval(&t, "score BETWEEN 40 AND NULL", 1), Trilean::False);
    }

    #[test]
    fn like_against_a_null_is_unknown() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "label LIKE 'a%'", 0), Trilean::True);
        assert_eq!(eval(&t, "label LIKE 'a%'", 1), Trilean::False);
        assert_eq!(eval(&t, "label LIKE 'a%'", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT (label LIKE 'a%')", 2), Trilean::Unknown);
    }

    // --- Controls: ordinary predicates over non-NULL data are untouched ---

    #[test]
    fn control_non_null_predicates_are_unchanged() {
        let t = table_with_nulls();
        assert_eq!(eval(&t, "score = 50", 0), Trilean::True);
        assert_eq!(eval(&t, "score = 50", 1), Trilean::False);
        assert_eq!(eval(&t, "score > 40 AND label = 'alpha'", 0), Trilean::True);
        assert_eq!(eval(&t, "score > 40 OR label = 'beta'", 1), Trilean::True);
        assert_eq!(eval(&t, "NOT (score = 50)", 1), Trilean::True);
    }
}

#[cfg(test)]
mod bare_value_predicate_tests {
    //! Regression tests for P37 — a bare value expression used as a `WHERE`
    //! predicate.
    //!
    //! These live here rather than in `tests/comparison/corpus/` because the
    //! parity harness structurally cannot see this fix. The corpus case that
    //! found P37 (`09_window.toml :: window_in_where_inline`) is bucketed
    //! `OURS_ONLY`: DuckDB rejects a window function in `WHERE` outright, so we
    //! are in that bucket whether we return the right rows or, as before, zero
    //! rows with a success exit code. `runner.py --check` stays green either
    //! way, and would stay green through a regression too.
    //!
    //! So they assert the general shape rather than the window that exposed
    //! it: the defect was never window-specific. `WHERE true` returned no rows
    //! for the same reason.

    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};
    use crate::sql::recursive_parser::Parser;

    /// Rows: 0 = (true, 1, "x"), 1 = (false, 0, ""), 2 = (NULL, NULL, NULL).
    ///
    /// `flag` stands in for the column `ExpressionLifter` synthesises when it
    /// hoists a window comparison out of `WHERE` — the lifted CTE column is a
    /// plain boolean, and `WHERE lifted_value` is what the main query is left
    /// referencing.
    fn table_with_flags() -> DataTable {
        let mut table = DataTable::new("t");
        table.add_column(DataColumn::new("flag"));
        table.add_column(DataColumn::new("n"));
        table.add_column(DataColumn::new("s"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Boolean(true),
                DataValue::Integer(1),
                DataValue::String("x".to_string()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Boolean(false),
                DataValue::Integer(0),
                DataValue::String(String::new()),
            ]))
            .unwrap();
        table
            .add_row(DataRow::new(vec![
                DataValue::Null,
                DataValue::Null,
                DataValue::Null,
            ]))
            .unwrap();

        table
    }

    fn eval(table: &DataTable, predicate: &str, row: usize) -> Trilean {
        let sql = format!("SELECT * FROM t WHERE {predicate}");
        let mut parser = Parser::new(&sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(table);
        evaluator
            .evaluate(&where_clause, row)
            .expect("evaluation failed")
    }

    // --- P37: the shapes that used to be FALSE for every row ---

    #[test]
    fn a_boolean_column_is_a_predicate_in_its_own_right() {
        let t = table_with_flags();
        assert_eq!(eval(&t, "flag", 0), Trilean::True);
        assert_eq!(eval(&t, "flag", 1), Trilean::False);
    }

    #[test]
    fn a_boolean_literal_is_a_predicate() {
        let t = table_with_flags();
        // `WHERE true` returned zero rows before the fix, on any table.
        assert_eq!(eval(&t, "true", 0), Trilean::True);
        assert_eq!(eval(&t, "false", 0), Trilean::False);
    }

    #[test]
    fn a_null_valued_predicate_is_unknown_not_false() {
        let t = table_with_flags();
        // The P18/P19 distinction: UNKNOWN and FALSE both drop the row under
        // `WHERE`, and only diverge under `NOT`.
        assert_eq!(eval(&t, "flag", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT flag", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT flag", 1), Trilean::True);
    }

    #[test]
    fn a_bare_value_composes_with_ordinary_predicates() {
        // The lifter can leave `WHERE lifted_value` beside other conditions,
        // so the bare form has to survive AND/OR like any other predicate.
        let t = table_with_flags();
        assert_eq!(eval(&t, "flag AND n = 1", 0), Trilean::True);
        assert_eq!(eval(&t, "flag AND n = 99", 0), Trilean::False);
        // Row 1 is flag=false, n=0 -- so the right operand has to be a miss
        // for the OR to come out FALSE.
        assert_eq!(eval(&t, "flag OR n = 99", 1), Trilean::False);
        assert_eq!(eval(&t, "flag OR n = 99", 0), Trilean::True);
    }

    #[test]
    fn numeric_values_coerce_by_zero_ness() {
        let t = table_with_flags();
        assert_eq!(eval(&t, "n", 0), Trilean::True);
        assert_eq!(eval(&t, "n", 1), Trilean::False);
    }

    // --- Control: the unlifted window is loud, not silently empty ---

    #[test]
    fn an_unlifted_window_function_errors_rather_than_filtering_everything() {
        let t = table_with_flags();
        let sql = "SELECT * FROM t WHERE ROW_NUMBER() OVER (ORDER BY n)";
        let mut parser = Parser::new(sql);
        let statement = parser.parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");

        let mut evaluator = RecursiveWhereEvaluator::new(&t);
        let err = evaluator
            .evaluate(&where_clause, 0)
            .expect_err("a raw window function in WHERE must not evaluate quietly");
        assert!(
            err.to_string().contains("ROW_NUMBER"),
            "error should name the function, got: {err}"
        );
    }
}

#[cfg(test)]
mod operand_resolution_tests {
    //! Regression tests for P46 -- every WHERE operand resolves to its real
    //! value. The right-hand side used to go through a literal-only reader that
    //! answered NULL for a column, a CASE or an expression, so any such
    //! predicate was UNKNOWN on every row. Asserting the `Trilean` per row keeps
    //! UNKNOWN and FALSE apart, which a row count could not.

    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};
    use crate::sql::recursive_parser::Parser;

    /// Rows: 0 = (1, 3), 1 = (5, 4), 2 = (6, NULL).
    fn pairs() -> DataTable {
        let mut table = DataTable::new("t");
        table.add_column(DataColumn::new("a"));
        table.add_column(DataColumn::new("b"));
        for (a, b) in [(1, Some(3)), (5, Some(4)), (6, None)] {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(a),
                    b.map_or(DataValue::Null, DataValue::Integer),
                ]))
                .unwrap();
        }
        table
    }

    fn try_eval(table: &DataTable, predicate: &str, row: usize) -> Result<Trilean> {
        let sql = format!("SELECT * FROM t WHERE {predicate}");
        let statement = Parser::new(&sql).parse().expect("failed to parse");
        let where_clause = statement.where_clause.expect("expected a WHERE clause");
        RecursiveWhereEvaluator::new(table).evaluate(&where_clause, row)
    }

    fn eval(table: &DataTable, predicate: &str, row: usize) -> Trilean {
        try_eval(table, predicate, row).expect("evaluation failed")
    }

    #[test]
    fn column_against_column() {
        let t = pairs();
        assert_eq!(eval(&t, "a < b", 0), Trilean::True);
        assert_eq!(eval(&t, "a < b", 1), Trilean::False);
        assert_eq!(eval(&t, "a = a", 1), Trilean::True);
    }

    #[test]
    fn column_against_null_column_stays_unknown_under_not() {
        // The fix must not land as FALSE for a NULL operand: NOT would flip it.
        let t = pairs();
        assert_eq!(eval(&t, "a < b", 2), Trilean::Unknown);
        assert_eq!(eval(&t, "NOT (a < b)", 2), Trilean::Unknown);
        // The arithmetic-operand shape used to answer FALSE here (P48 leaking
        // into WHERE through the ArithmeticEvaluator's comparison).
        assert_eq!(eval(&t, "NOT (a + 0 < b)", 2), Trilean::Unknown);
    }

    #[test]
    fn case_and_expression_operands() {
        let t = pairs();
        assert_eq!(
            eval(&t, "a < CASE WHEN b > 3 THEN 10 ELSE 0 END", 0),
            Trilean::False
        );
        assert_eq!(
            eval(&t, "a < CASE WHEN b > 3 THEN 10 ELSE 0 END", 1),
            Trilean::True
        );
        assert_eq!(eval(&t, "b = a + 2", 0), Trilean::True);
    }

    #[test]
    fn between_bounds_and_in_items_resolve_columns() {
        let t = pairs();
        assert_eq!(eval(&t, "b BETWEEN a AND 10", 0), Trilean::True);
        assert_eq!(eval(&t, "b BETWEEN a AND 10", 1), Trilean::False);
        assert_eq!(eval(&t, "b IN (a + 2, 99)", 0), Trilean::True);
        assert_eq!(eval(&t, "b IN (a + 2, 99)", 2), Trilean::Unknown);
    }

    #[test]
    fn a_literal_on_the_left_resolves_too() {
        let t = pairs();
        assert_eq!(eval(&t, "3 < a", 1), Trilean::True);
        assert_eq!(eval(&t, "3 < a", 0), Trilean::False);
    }

    #[test]
    fn a_raw_window_operand_errors_rather_than_ranking_unfiltered_rows() {
        // An inline QUALIFY reaches here unlifted (P15). Evaluating it would
        // rank the rows the WHERE has not yet filtered -- a silent wrong answer.
        let t = pairs();
        let err = try_eval(&t, "ROW_NUMBER() OVER (ORDER BY a) = 1", 0)
            .expect_err("a raw window operand must not evaluate quietly");
        assert!(err.to_string().contains("ROW_NUMBER"), "got: {err}");
    }

    #[test]
    fn a_raw_window_operand_errors_in_between_and_in_too() {
        // Same P15 rule in every operand position, so it has to survive these
        // arms delegating to the value evaluator (R13 slice 4), which would
        // otherwise evaluate the window function over the unfiltered rows.
        let t = pairs();
        for predicate in [
            "ROW_NUMBER() OVER (ORDER BY a) BETWEEN 1 AND 2",
            "a BETWEEN ROW_NUMBER() OVER (ORDER BY a) AND 10",
            "ROW_NUMBER() OVER (ORDER BY a) IN (1, 2)",
            "a IN (ROW_NUMBER() OVER (ORDER BY a), 2)",
            "ROW_NUMBER() OVER (ORDER BY a) NOT IN (1, 2)",
        ] {
            let err = try_eval(&t, predicate, 0)
                .expect_err("a raw window operand must not evaluate quietly");
            assert!(
                err.to_string().contains("ROW_NUMBER"),
                "{predicate}: got {err}"
            );
        }
    }

    /// Two source tables joined, both with a column `a`, qualified by table
    /// name: row 0 = (orders.a 1, items.a 7), row 1 = (2, 9).
    fn joined() -> DataTable {
        let mut table = DataTable::new("joined");
        table.add_column(DataColumn::new("a").with_qualified_name("orders"));
        table.add_column(DataColumn::new("a").with_qualified_name("items"));
        for (o, i) in [(1, 7), (2, 9)] {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(o),
                    DataValue::Integer(i),
                ]))
                .unwrap();
        }
        table
    }

    #[test]
    fn alias_qualified_operands_resolve_through_the_execution_context() {
        // `i` is an alias for `items`. The unqualified fallback would find
        // `orders.a` first, so a wrong resolution shows as a wrong answer.
        //
        // Latent, not reachable from SQL as probed: join output already
        // disambiguates duplicate names before WHERE sees them. WHERE's column
        // fast path used to resolve the alias to an index and then look the
        // bare name up again, and the inner value evaluator was given no
        // aliases -- so both answered from `orders.a` here.
        let t = joined();
        let mut ctx = ExecutionContext::new();
        ctx.register_alias("i".to_string(), "items".to_string());
        let eval = |predicate: &str, row: usize| {
            let sql = format!("SELECT * FROM joined WHERE {predicate}");
            let statement = Parser::new(&sql).parse().expect("failed to parse");
            let where_clause = statement.where_clause.expect("expected a WHERE clause");
            RecursiveWhereEvaluator::with_exec_context(&t, &ctx, false)
                .evaluate(&where_clause, row)
                .expect("evaluation failed")
        };
        // (predicate, correct answer for row 0, answer today if it differs)
        for (predicate, correct, known) in [
            ("i.a = 7", Trilean::True, None),
            ("i.a BETWEEN 5 AND 8", Trilean::True, None),
            ("i.a IN (7, 99)", Trilean::True, None),
            ("i.a NOT IN (7, 99)", Trilean::False, None),
            ("i.a + 0 = 7", Trilean::True, None),
        ] {
            let observed = eval(predicate, 0);
            match known {
                Some(today) => assert_eq!(
                    observed, today,
                    "{predicate}: recorded divergence changed -- if fixed, drop it"
                ),
                None => assert_eq!(observed, correct, "{predicate}"),
            }
        }
    }
}
