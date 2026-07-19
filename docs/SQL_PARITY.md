# SQL Parity — Book of Work

The durable decision log for SQL-engine parity. We differentially test sql-cli
against a reference engine (**DuckDB** today) and systematically find, document,
and either **fix** divergences or record **why we won't**.

- **Harness / raw output:** `tests/comparison/` — run
  `uv run python tests/comparison/runner.py`; it regenerates
  `tests/comparison/reports/compare_<ref>.md` (the machine's current GAP/DIFFER list).
- **This file:** the curated, human decisions behind those buckets. The report
  says *what* diverges right now; this file says *what we're doing about it and why*.
- **CI gate:** `runner.py --check` runs in `.github/workflows/test-complete.yml`
  (job *SQL Parity*) and fails the build on any drift from the contract below.
- **Companion log:** [`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md) (R-numbers)
  tracks *structural* debt rather than wrong answers. Where a parity gap turns
  out to have a shape problem underneath it, the P-entry links to the R-entry.

### Regression contract

Each corpus case carries an expectation; `--check` fails if reality disagrees:

- a case with `expect = "GAP" | "DIFFER" | ...` must still be in that bucket;
- a case with **no** `expect` must be `AGREE`.

This means: fixing a gap is a deliberate edit (drop the `expect`, flip the entry
below to 🟢), and any *regression* of a passing case is caught automatically. So
as we work through the backlog, the formal comparison — not just the older test
suites — is what locks the gains in.

Parity is **broad-brush**, not byte-for-byte. We follow SQL-standard / DuckDB
semantics where reasonable, and consciously diverge where our design
([heterogeneous one-shop querying, coercion-first](FEATURE_ROADMAP_2026_Q2.md))
makes a different choice better.

## Status legend

| Status | Meaning |
|---|---|
| 🔴 OPEN | confirmed divergence, not yet addressed |
| 🟡 IN PROGRESS | being worked |
| 🟢 FIXED | resolved; corpus case now AGREEs |
| ⚪ WON'T FIX | intentional divergence — rationale recorded |

Each entry maps to a corpus case (so `expect=` in the TOML and this log stay in
sync). When an issue is FIXED, its case should flip to `AGREE` and the `expect`
annotation be removed.

---

## Open issues

### P1 — `SUBSTRING` is 0-indexed; SQL standard is 1-indexed
- **Status:** 🟢 FIXED (2026-06-27)
- **Corpus:** `03_functions.toml :: fn_substring` (now AGREE; `expect` dropped)
- **Observed:** `SUBSTRING('AAPL', 1, 2)` → sql-cli `'AP'`, DuckDB/standard `'AA'`.
- **Decision:** **Fixed by splitting the two call syntaxes**, not by flipping one
  global index. The same `SubstringMethod` struct backs both forms:
  - **SQL function** `SUBSTRING(s, start, len)` → now **1-based** (SQL standard /
    DuckDB / SQL Server). A `start < 1` still anchors at the head and consumes
    part of `len`, matching DuckDB (`SUBSTRING('hello',0,2)` → `'h'`).
  - **C# method** `s.Substring(start, len)` → stays **0-based** (.NET semantics),
    preserving the deliberate C#-style affordance.
  Mechanically: `SqlFunction::evaluate` is 1-based; `MethodFunction::evaluate_method`
  is overridden to be 0-based; both delegate to a shared `SubstringMethod::extract`.
  Method-call dispatch (`arithmetic_evaluator.rs::evaluate_method_on_value`) now
  routes through the method registry first (`get_method().evaluate_method()`), so
  the two forms can diverge — behavior-preserving for every other method, whose
  default `evaluate_method` just prepends the receiver and calls `evaluate`.
- **Notes:** This generalizes the position-function audit: `INDEXOF` (method,
  0-based) vs `INSTR` (SQL, 1-based) already followed the same function-vs-method
  split, and now `SUBSTRING` is consistent with it. `LEFT`/`RIGHT` take counts,
  not positions, so are unaffected. Examples using the SQL form with 0-based
  args were corrected (`join_left_expression_demo.sql`); the
  `showcase_deterministic` expectation was re-captured (`'ello '` → `'Hello'`).

### P2 — `CAST(expr AS type)` not supported
- **Status:** 🟢 FIXED (2026-07-04)
- **Corpus:** `03_functions.toml :: fn_cast_int` (now AGREE; `expect` dropped),
  plus `fn_cast_int_to_double`, `fn_cast_num_to_varchar`,
  `fn_cast_precision_ignored`, `fn_try_cast_null_on_failure`.
- **Observed (before):** Parse error `Expected RightParen, found As`. There was no
  `CAST` in the parser; the engine relied on evaluation-time **coercion**, and
  `CONVERT` is unit conversion (3 args), not type casting.
- **Decision:** **Fixed as explicit sugar over the coercion layer.** `CAST` /
  `TRY_CAST` are **lowered in the parser** into a two-arg function call
  `CAST(expr, 'TYPE')` (the `AS type` clause is intercepted in
  `src/sql/parser/expressions/primary.rs`), so they flow through the existing
  evaluator, WHERE path, and every AST walker without a new `SqlExpression`
  variant. The cast itself is a registry function
  (`src/sql/functions/cast.rs`) — matching the "everything goes through the
  registry" principle.
- **Type confines (deliberate):** we collapse SQL's char/numeric "zoo" onto the
  five types `DataValue` stores — INTEGER, DOUBLE (FLOAT/REAL/DECIMAL/NUMERIC),
  VARCHAR (CHAR/TEXT/STRING/…), BOOLEAN, DATE/TIMESTAMP. A precision/scale spec
  such as `DECIMAL(10,2)` or `VARCHAR(50)` **parses and is ignored** — no
  fixed-width CHAR, no decimal scale. Target types we cannot represent (e.g.
  `BLOB`) are a **query error**, even under `TRY_CAST`.
- **Semantics matched to DuckDB:** NULL casts to NULL; float→int **rounds**
  (DuckDB rounds, does not truncate) using **round-half-to-even** so `.5` ties
  agree (`CAST(2.5 AS INT)=2`); `CAST` errors on an invalid value while
  `TRY_CAST` yields NULL.
- **Notes:** Coercion-first is our design; CAST is explicit sugar over the same
  rules so results stay consistent with implicit coercion. The DuckDB-idiomatic
  `expr::type` postfix operator is a deliberate follow-up (needs a lexer token);
  the portable `CAST(... AS ...)` form works in both engines for the corpus.

### P3 — Correlated subqueries do not apply the outer-row correlation
- **Status:** 🔴 OPEN — **theme / root cause**, covers several corpus cases
- **Corpus:** `05_subqueries.toml :: in_subquery_correlated` (DIFFER, returns empty),
  `scalar_subquery_correlated` (GAP, "returned 0 rows"),
  `scalar_subquery_in_select_correlated` (GAP),
  `exists_correlated` (GAP), `not_exists_correlated` (GAP).
- **Observed:** A subquery referencing an outer column (`WHERE x.region = s.region`)
  does not see the outer row — it evaluates as if the outer reference is empty,
  so correlated scalar subqueries error ("0 rows"), correlated `IN` returns an
  empty set, and `EXISTS` / `NOT EXISTS` don't parse at all.
- **Decision:** **Fix** — central to the column-scoping work. Two parts:
  1. **Parser:** accept `[NOT] EXISTS (<subquery>)` as a predicate.
  2. **Executor:** evaluate correlated subqueries per outer row, resolving outer
     column references through an enclosing scope. This is the same scoping spine
     that nested-SQL column resolution needs generally.
- **Notes:** Uncorrelated scalar / `IN` / derived-table subqueries already AGREE;
  the gap is specifically the outer-row binding. Highest-leverage fix here — one
  root cause unlocks five cases and the broader nested-scoping goal.
- **Structural root cause:** [R7](ENGINE_REFACTORING.md) — subqueries are
  *substituted* by an up-front AST rewrite pass, never *evaluated* per row, so
  there is no outer row to correlate against. Groundwork tracked as
  [R2](ENGINE_REFACTORING.md) (traversal helpers, needed before the `EXISTS`
  variant can be added safely) and [R6](ENGINE_REFACTORING.md) (the existing
  correlation analyzer is unwired and untested).

### P4 — Self-join of the base table fails to resolve
- **Status:** 🟢 FIXED (2026-07-11)
- **Corpus:** `04_joins.toml :: self_join_base`, `self_join_aggregate`,
  `self_left_join_base` (all AGREE)
- **Observed (was):** `FROM trades a JOIN trades b ...` → "Cannot resolve table
  'trades' for JOIN". Joins to derived tables / CTEs built from the same source
  already worked; only re-referencing the base table by name failed.
- **Fix:** In `query_engine.rs`, when a JOIN target names the main FROM table it
  now re-references the already-loaded source (`base_table_name` check) and applies
  the join alias to its qualified columns, mirroring the CTE-in-join path. The
  right side's columns collide by name with the left, so `HashJoinExecutor` renames
  them to `<alias>.<col>`, which lets `b.col` resolve in projection.

### P5 — `CROSS JOIN` to a FROM-less subquery has wrong cardinality
- **Status:** 🟢 FIXED (2026-07-11)
- **Corpus:** `04_joins.toml :: cross_join_constant` (now AGREEs)
- **Observed (was):** `trades t CROSS JOIN (SELECT 1 AS k) c` returned 92×92 = 8464
  rows instead of 92. A FROM-less subquery (`SELECT 1 AS k`) yielded one row per
  outer row instead of a single constant row.
- **Fix:** A FROM-less SELECT now sources from `DataTable::dual()` (a single-row
  DUAL table) instead of reusing the caller's outer table, in
  `query_engine.rs`. It produces exactly one row.

### P6 — `INTERSECT` / `EXCEPT` not implemented
- **Status:** 🔴 OPEN
- **Corpus:** `06_ctes_setops.toml :: intersect`, `except`
- **Observed:** "INTERSECT is not yet implemented" / "EXCEPT is not yet implemented".
  `UNION` and `UNION ALL` already AGREE.
- **Decision:** **Fix** — implement alongside the existing `UNION` set-op path.

### P7 — Multi-condition join evaluates extra-condition operands by position
- **Status:** 🟢 FIXED (2026-07-12)
- **Corpus:** `04_joins.toml :: join_condition_operand_order` (now AGREEs)
- **Observed (was):** `... JOIN trades b ON a.symbol = b.symbol AND b.price < a.price`
  returned rows where `bp > ap`, violating the predicate. The multi-condition
  nested-loop paths (`nested_loop_join_{inner,left}_multi` in `hash_join.rs`)
  evaluated each extra condition's `left_expr` against the **left** table and
  `right_expr` against the **right** table by *syntactic position*, ignoring the
  actual alias/table each operand belongs to. So `b.price < a.price` (right-table
  column written first) was silently evaluated as `a.price < b.price`. Writing the
  same predicate left-table-first (`a.price > b.price`) AGREEd. Affected INNER and
  LEFT joins.
- **Fix:** Each ON operand is now routed to its owning table by *alias qualifier*
  rather than syntactic position, in `hash_join.rs`. `operand_uses_right` decides
  the side: an operand whose prefix equals the join alias belongs to the joined
  table, any other prefix belongs to the opposite table, and unqualified operands
  fall back to the old positional default. A `join_alias_is_right` flag threaded
  into `nested_loop_join_{inner,left}_multi` keeps this correct for the swapped
  RIGHT-join path (where the join-alias columns live in the `left_table` arg).
  This is orientation-independent and needs no separate left-alias plumbing: the
  left/current table accumulates every non-join alias, so "prefix != join alias →
  left table" holds for chained joins too. The operator is then applied between the
  two operands exactly as written.
- **Regression test:** `tests/join_operand_order_tests.rs` pins the self-consistency
  property (operand order can't change the result) for INNER and LEFT in plain
  `cargo test`, independent of the DuckDB corpus.

### P8 — Multi-condition RIGHT JOIN mislabels columns and NULLs the wrong side
- **Status:** 🟢 FIXED (found 2026-07-12 while verifying P7; fixed 2026-07-17)
- **Corpus:** `04_joins.toml :: right_join_multi_condition` (now AGREE)
- **Observed:** `... a RIGHT JOIN trades b ON a.symbol = b.symbol AND a.price < b.price`
  returned the right *number* of rows but wrong content: the outer (`a`) columns'
  values surfaced under `b`'s alias and vice-versa, and NULLs were emitted for the
  wrong side (the `b` columns instead of the unmatched `a` columns). The RIGHT path
  reused `nested_loop_join_left_multi` with the tables swapped but passed the join
  alias unchanged, so both the `[joined, FROM]` result-column order and the
  outer-side NULL emission were applied to the swapped-in table. This was
  **separate from P7** — the P7 operand routing was orientation-correct here; the
  defect was purely in RIGHT-join result-column assembly.
- **Scope note:** Single-condition RIGHT joins (the hash path) always AGREEd, so
  this was confined to the multi-condition nested-loop RIGHT path.
- **Fix:** Added a dedicated `nested_loop_join_right_multi` in `hash_join.rs`
  instead of reusing the swapped LEFT builder. It emits result columns in
  `[FROM, joined]` order (matching INNER/LEFT), keeps the FROM table's qualified
  names, applies the join alias only to the joined table on a name collision, and
  iterates the joined table as the outer loop so every joined row is kept and the
  FROM columns NULL-fill on no match. Operand routing (P7) is preserved. Data and
  matching were already correct — this was a labelling/ordering change only.
- **Regression test:** `tests/join_operand_order_tests.rs ::
  right_join_multi_condition_labels_correct_side` pins that the `a.*`/`b.*` values
  land under the correct aliases and NULLs fall on the FROM side, in plain
  `cargo test` (independent of the DuckDB corpus).

### P9 — `HAVING` silently mishandles an aggregate nested in a non-comparison operator
- **Status:** 🟢 CLOSED 2026-07-19 — all three corpus cases now AGREE
- **Corpus:** `07_grouping.toml :: having_between`, `having_in_list`,
  `having_case` — `expect` dropped, they are plain AGREE cases now.
  `having_comparison` / `having_sum_comparison` were already AGREE.
- **Observed:** `HavingAliasTransformer` rewrites an aggregate in `HAVING` to
  reference its computed alias, but its traversal handles only `FunctionCall`,
  `BinaryOp` and `Not`. An aggregate reached through any other operator is never
  rewritten, and the result is **silently wrong in both directions**:

  | Query | Correct | sql-cli |
  |---|---|---|
  | `HAVING COUNT(*) BETWEEN 1 AND 2` | 1 row | **4 rows** (under-filters) |
  | `HAVING COUNT(*) IN (4, 5)` | 2 rows | **0 rows** (over-filters) |
  | `HAVING CASE WHEN COUNT(*) > 2 THEN 1 ELSE 0 END = 1` | 2 rows | **0 rows** |

  No error is raised in any of these — the predicate simply doesn't do what it
  says. `HAVING COUNT(*) > 2` works, which is why this went unnoticed.
- **Found:** 2026-07-18, while surveying transformers for the
  [R2](ENGINE_REFACTORING.md) walker migration. **Not found by testing** — the
  corpus had no `HAVING` coverage at all before this entry.
- **Root cause:** [R3](ENGINE_REFACTORING.md) — hand-rolled traversals ending in
  a `_ => {}` catch-all. `collect_aggregates_in_having` and
  `rewrite_having_expression` each miss 14 of the 24 expression variants. The
  code comment at `having_alias_transformer.rs:213` acknowledges the untransformed
  aggregate "will fail later"; in practice it does not fail, it returns wrong rows.
- **Decision:** **Fix** via the R2 migration — moving both functions onto
  `walk::visit_children` / `map_children` retires the catch-all and covers every
  variant. Constraint: the aggregate arm must *not* delegate to the walker, or it
  would start recursing into aggregate arguments and break the deliberate
  "no nested aggregates" invariant.
- **Fixed:** 2026-07-19. Both functions now read "handle the aggregate, delegate
  the rest" — `collect_aggregates_in_having` returns early on an aggregate then
  calls `visit_children`; `rewrite_having_expression` does the same with
  `map_children`. 74 lines of hand-rolled match became 24. The constraint above
  held: the early return *is* what keeps aggregate arguments untraversed.
  Two things worth recording, neither obvious before the migration:
  - **The subquery boundary works in our favour.** `map_children` does not
    descend into a nested `SelectStatement`, so an aggregate belonging to a
    subquery's own scope is correctly left alone — the outer HAVING must not
    claim it. The opaque default is load-bearing here, not incidental.
  - **`having_not` stayed a GAP**, exactly as P10 predicted. Good evidence the
    two entries were correctly split rather than being one finding.

### P10 — `HAVING NOT (...)` errors in the evaluator
- **Status:** 🔴 OPEN
- **Corpus:** `07_grouping.toml :: having_not` (GAP)
- **Observed:** `HAVING NOT (COUNT(*) > 2)` →
  `Unsupported expression type for arithmetic evaluation: Not { ... }`.
- **Distinct from P9:** here the aggregate *is* rewritten correctly (the
  transformer does handle `Not`), and the failure is downstream — the arithmetic
  evaluator has no `Not` arm for a post-aggregation predicate. Fixing P9 will not
  fix this.
- **Decision:** **Fix** — add the missing evaluator arm. Small and independent.

### P11 — A `SELECT` alias is not expanded on the LHS of an `IN` subquery
- **Status:** 🔴 OPEN
- **Corpus:** `02_where.toml :: select_alias_in_in_subquery` (GAP)
- **Observed:** `SELECT symbol, price * 2 AS dbl FROM trades WHERE dbl IN
  (SELECT ...)` → `Column 'dbl' not found`. The same alias resolves fine as the
  LHS of a plain comparison or an `IN`-list, so this is specific to the subquery
  form.
- **Root cause:** [R3](ENGINE_REFACTORING.md). `WhereAliasExpander` never matches
  `InSubquery` / `NotInSubquery` / the tuple forms, so the **same-scope** LHS
  operand is skipped along with the subquery. Note the expander is right not to
  descend into the subquery *body* (different scope) — the bug is that it drops
  the operand too.
- **Decision:** **Fix** via the R2 migration. `walk` visits `InSubquery`'s `expr`
  while still treating the nested statement as a scope boundary, which is exactly
  the required behaviour.

### P12 — `WITH` is rejected in expression position
- **Status:** 🔴 OPEN
- **Corpus:** `06_ctes_setops.toml :: cte_in_expression_position` (GAP)
- **Observed:** `WHERE price > (WITH avg_cte AS (...) SELECT a FROM avg_cte)` →
  `Parse error: Unexpected token in primary expression: With`. Rejected in every
  expression position tried — scalar subquery, `BETWEEN` operand, `IN`-list
  element, and tuple `IN` (which reports "Tuple IN requires a subquery on the
  right"). DuckDB accepts a CTE inside a scalar subquery.
- **Found:** 2026-07-18, while trying to write a regression test for the
  `cte_hoister` walker migration — the test could not be expressed.
- **Side effect worth noting:** the `ScalarSubquery` / `InSubquery` arms of
  `CTEHoister::hoist_from_expression` are therefore **unreachable dead code**
  today; expression-position CTE hoisting has never had an input.
- **Decision:** **Fix** — a parser change (accept `WITH` where a subquery is
  already accepted). The hoister machinery to handle the result already exists.

---

## Deferred / won't fix (intentional)

### D1 — Recursive CTEs (`WITH RECURSIVE`)
- **Status:** ⚪ DEFERRED (considered, not supported)
- **Corpus:** `06_ctes_setops.toml :: recursive_cte`
- **Observed:** Parser rejects the `name(col, ...)` column-list form;
  `WITH RECURSIVE` is not implemented.
- **Rationale:** Considered and consciously deferred. It belongs to a larger
  potential design direction — a script/session **scope** that can hold
  variables, staged temp tables, and iterative evaluation — which is out of scope
  for the current "vanilla SQL consistency" effort. Revisit if/when that scope
  layer is pursued. Not a bug; do not let the corpus case churn — keep `expect = "GAP"`.

_When we consciously diverge from the reference engine on results (rather than
simply not implementing a feature), record it here with the rationale so the
DIFFER is understood, not mistaken for a bug._

---

## Workflow

1. Add/extend a tier in `tests/comparison/corpus/NN_*.toml` and run the harness.
2. For each `GAP`/`DIFFER`, add an entry here with a decision.
3. Annotate the corpus case with `expect = "GAP"` / `"DIFFER"` so the run flags
   the day the bucket changes.
4. On fix: implement, confirm the case flips to `AGREE`, drop the `expect`, set
   the entry to 🟢 FIXED (or move to Won't Fix with rationale).
