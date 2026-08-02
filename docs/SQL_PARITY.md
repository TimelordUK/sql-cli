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

### Where the standard leaves a choice open, follow the reference engine

Established 2026-08-02 while deciding P17 and P20, both of which are cases the
SQL standard leaves implementation-defined and where the major engines genuinely
disagree. Rather than judge each one on its merits, the default answer is
**match DuckDB** — that is what having a reference engine is *for*, and it keeps
"broad-brush parity" a single rule instead of a growing pile of one-off
rationales. Diverging remains available, but it has to be argued for on design
grounds and recorded in *Deferred / won't fix* below.

**DuckDB is a reference point, not a specification.** The goal is to be brought
*in line* — to stop being accidentally different — not to reproduce DuckDB
exactly. Where a difference is a genuine DuckDB idiosyncrasy rather than
standard or widely-shared behaviour, we are under no obligation to follow it;
mark it ⚪ WON'T FIX with the reasoning and move on. The rule above is a default
that saves us re-litigating the ambiguous cases, not a commitment to chase
quirks.

One consequence worth naming: this makes the reference engine's *version* part
of our contract, not just its behaviour. Implementation-defined cases pin
whatever DuckDB currently chooses, so the DuckDB version is pinned in
`pyproject.toml` (`[dependency-groups].test`) and used by CI — bump it
deliberately and review the drift, rather than letting it float.

## Where this effort is up to

**Phase: fixing.** 2026-08-01/02 was a deliberate discovery push — the corpus
went from 83 to 150 cases and the open findings from one (P3) to fifteen. That
is enough surfaced work to be going on with, and several of these will take a
session apiece to fix properly, so **discovery is paused and the effort moves to
picking them off**. Widen the corpus again when the open list is short, or
opportunistically when a fix needs a case that doesn't exist yet.

Corpus coverage today: tiers 01–10. **Tier 10 (aggregate & NULL edges) is
deliberately partial** — it holds the P14 and P18–P20 cases and their baselines,
but was never built out the way tiers 08 and 09 were. Finish it during a lull;
the aggregate-function surface (`STDDEV`, `DISTINCT` aggregates, `FILTER`,
empty-vs-all-NULL distinctions) is largely unexamined.

Suggested fix order, by silent blast radius:

| | Finding | Why first |
|---|---|---|
| ~~1~~ | ~~[P21](#p21) windows evaluated before `WHERE`~~ | ✅ **Fixed 2026-08-02** |
| 2 | [P13](#p13) trailing tokens discarded | Silent, unbounded scope — any typo becomes a different working query |
| 3 | [P18](#p18)/[P19](#p19) three-valued logic | Silent, and P18 produces *extra* rows |
| 4 | [P24](#p24) `RANGE` treated as `ROWS` | Silent, hits the common `SUM(x) OVER (ORDER BY y)` running-total form |
| 5 | [P14](#p14), [P16](#p16), [P17](#p17), [P20](#p20), [P23](#p23) | Smaller, self-contained, decisions already taken |
| 6 | [P22](#p22), [P25](#p25), [P26](#p26), [P15](#p15) | Hard errors — visible, so less urgent than any of the above |

P3 (correlated subqueries) stays gated on the R7/R6 structural work in
[`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md) and is not part of this queue.

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
- **Status:** 🟢 FIXED (2026-07-25)
- **Corpus:** `06_ctes_setops.toml :: intersect`, `except` (both now AGREE;
  `expect` dropped)
- **Observed (was):** "INTERSECT is not yet implemented" / "EXCEPT is not yet
  implemented". `UNION` and `UNION ALL` already AGREE.
- **Fix:** Filled in the two `return Err(...)` stubs in the set-op loop of
  `query_engine.rs`. Both operate on the already-materialized `combined_table`
  (left) and `next_table` (right):
  - **INTERSECT [DISTINCT]** keeps left rows whose key is present in the right,
    deduplicated.
  - **EXCEPT [DISTINCT]** keeps left rows whose key is *absent* from the right,
    deduplicated.
  Both are DISTINCT by default (SQL standard), so each filters and dedups inline
  in one pass rather than setting `needs_deduplication` (that flag stays
  UNION-only). The row key is `format!("{:?}", row.values)` — the **same
  equality basis** `apply_distinct` uses for UNION, so set membership is
  consistent across all four set ops. Left-to-right evaluation of chained set
  ops is unchanged (INTERSECT-binds-tighter precedence remains a separate,
  pre-existing limitation, not exercised by the corpus).
- **Corpus note:** the `except` case threshold was moved from `amount > 2000` to
  `> 3000`. Every region has a sale > 2000, so the original form was trivially
  empty and would have AGREEd for the wrong reason (cf. the tier-7 `having_in_list`
  lesson); `> 3000` leaves `{Oceania}`, a genuine left-minus-right difference.

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
- **Status:** 🟢 FIXED (2026-07-25)
- **Corpus:** `07_grouping.toml :: having_not` (now AGREE; `expect` dropped)
- **Observed (was):** `HAVING NOT (COUNT(*) > 2)` →
  `Unsupported expression type for arithmetic evaluation: Not { ... }`.
- **Distinct from P9:** here the aggregate *is* rewritten correctly (the
  transformer does handle `Not`), and the failure was downstream — the arithmetic
  evaluator had no `Not` arm for a post-aggregation predicate. Fixing P9 did not
  fix this, exactly as predicted.
- **Fix:** Added a `Not { expr }` arm to `ArithmeticEvaluator::evaluate`
  (`src/data/arithmetic_evaluator.rs`), immediately after the `Between` arm. It
  evaluates the inner expression and negates it through the existing `to_bool`
  helper (the same truthiness the `AND`/`OR` arms use), with SQL three-valued
  logic on the NULL edge: `NOT NULL` → NULL rather than a coerced `true`. The
  corpus case returns just `Oceania,1` (the one group with `COUNT(*) <= 2`),
  matching DuckDB.

### P11 — A `SELECT` alias is not expanded on the LHS of an `IN` subquery
- **Status:** 🟢 FIXED (2026-07-25)
- **Corpus:** `02_where.toml :: select_alias_in_in_subquery` (now AGREE; `expect`
  dropped)
- **Observed (was):** `SELECT symbol, price * 2 AS dbl FROM trades WHERE dbl IN
  (SELECT ...)` → `Column 'dbl' not found`. The same alias resolves fine as the
  LHS of a plain comparison or an `IN`-list, so this looked specific to the
  subquery form.
- **Root cause (documented):** [R3](ENGINE_REFACTORING.md). `WhereAliasExpander`
  never matched `InSubquery` / `NotInSubquery` / the tuple forms, so the
  **same-scope** LHS operand was skipped along with the subquery.
- **Second, undocumented layer (found while fixing):** the alias fix alone was
  *necessary but not sufficient*. Once `dbl` expands to `price * 2`, the LHS is a
  compound expression — and `price * 2 IN (SELECT ...)` **fails even with no
  alias at all**. The `InOperatorLifter` transformer normally lifts an
  expression-LHS `IN`-list into a computed CTE column, but it only matches
  `InList` / `NotInList`; an `IN`-**subquery**'s `InList` is synthesized *later*
  by `SubqueryExecutor`, after the lifter has run, so `evaluate_in_list` received
  a raw expression LHS and its `extract_column_name` errored. This was a
  pre-existing bug the alias case merely exposed.
- **Fix (two parts):**
  1. **R2 migration of `WhereAliasExpander`.** Rewrote `expand_expression` to
     intercept only the two nodes that carry a real rule (a bare column naming an
     alias; a method-call *string* receiver the walker can't reach) and delegate
     all structural recursion to `walk::map_children`. That retired ~230 lines of
     hand-rolled arms **and** the `_ => (clone, false)` catch-all, and picked up
     the four subquery-LHS variants for free — the walker visits their same-scope
     operands while treating the nested `SelectStatement` as an opaque scope
     boundary (so a `dbl` inside the subquery body is correctly left alone).
     Regression: `where_alias_expander.rs ::
     test_expands_alias_on_in_subquery_lhs_not_body`.
  2. **Expression LHS in `evaluate_in_list` / `evaluate_between`.** Added
     `RecursiveWhereEvaluator::evaluate_operand_value`, which looks up a plain
     column but evaluates any other expression through the `ArithmeticEvaluator`
     — the same delegation `evaluate_binary_op` already does for its LHS. Both
     `IN` and `BETWEEN` now accept a compound LHS.
- **Note:** the ideal long-term home for part 2 is the [R7](ENGINE_REFACTORING.md)
  per-row subquery evaluation; this is the correct localized fix until then, and
  it stands on its own (it fixes alias-free `expr IN (subquery)` too).

### P12 — `WITH` is rejected in expression position
- **Status:** 🟢 FIXED (2026-07-25)
- **Corpus:** `06_ctes_setops.toml :: cte_in_expression_position` (now AGREE;
  `expect` dropped)
- **Observed (was):** `WHERE price > (WITH avg_cte AS (...) SELECT a FROM avg_cte)`
  → `Parse error: Unexpected token in primary expression: With`. Rejected in every
  expression position tried — scalar subquery, `BETWEEN` operand, `IN`-list
  element, and tuple `IN` (which reported "Tuple IN requires a subquery on the
  right"). DuckDB accepts a CTE inside a scalar subquery.
- **Found:** 2026-07-18, while trying to write a regression test for the
  `cte_hoister` walker migration — the test could not be expressed.
- **Fix:** Pure parser change. `parse_subquery()` *already* dispatched a leading
  `WITH` to the CTE parser (`parse_with_clause_inner`); the only blockers were the
  subquery-detection guards that gated on `Token::Select` alone. Widened them to
  `Token::Select | Token::With` in both spots a subquery is recognised:
  - `expressions/primary.rs` — the scalar-subquery branch after `(`, plus the two
    tuple-`IN` guards (`(a, b) IN (…)` / `NOT IN`).
  - `expressions/comparison.rs` — the `x IN (…)` and `x NOT IN (…)` subquery
    branches.
  So a CTE is now accepted wherever a subquery already was, matching DuckDB.
- **Side effect resolved:** the `ScalarSubquery` / `InSubquery` arms of
  `CTEHoister::hoist_from_expression` were previously **unreachable dead code**
  (expression-position CTE hoisting never had an input); they now receive real
  input.

### P13 — Unparsed trailing tokens are silently discarded, taking later clauses with them
- **Status:** 🔴 OPEN
- **Corpus:** `08_ordering.toml :: trailing_garbage_token` (OURS_ONLY — the root
  cause, pinned directly), `order_by_nulls_last_limit` and
  `order_by_nulls_first_limit` (DIFFER — the instance users actually hit).
  Controls: `order_by_limit`, `order_by_nulls_last_no_limit` (both AGREE).
- **Observed:** `ORDER BY amount DESC NULLS LAST LIMIT 3` returns **all 20 rows**
  instead of 3. No error. The `LIMIT` is simply gone.
- **Root cause — broader than it first looks.** `NULLS` is not the issue; there
  is **no `NULLS` handling anywhere in `src/sql/`**. The parser stops at the
  first token it cannot place and **silently ignores the entire remainder of the
  statement**, including every clause after it. Verified with a nonsense token:

  | Query | Result |
  |---|---|
  | `... ORDER BY amount DESC FROBNICATE LIMIT 3` | 20 rows, no error (LIMIT dropped) |
  | `... GROUP BY country FROBNICATE LIMIT 2` | 13 rows, no error (LIMIT dropped) |
  | `SELECT country FROM international_sales FROBNICATE` | 20 rows, no error |

  So *any* typo, or any clause we don't support, degrades into a **different
  query that runs successfully**. This is the same silent-wrong-answer class as
  P9, but at the parser level and therefore unbounded in scope — it is not
  confined to one clause or one transformer.
- **Decision:** **Fix**, in two stages, and keep them separate:
  1. **Reject trailing input.** After parsing a statement, require EOF (or a
     statement separator) and error otherwise. This converts an unbounded class
     of silent wrong answers into loud parse errors.
  2. **Implement `NULLS FIRST` / `NULLS LAST`** as a real `OrderByItem` option.
     DuckDB defaults to NULLS LAST for ASC and NULLS FIRST for DESC; our current
     NULL ordering is untested (see note below).
- **Expected corpus churn — plan for it.** Stage 1 alone moves the two NULLS
  cases **DIFFER → GAP**, not to AGREE, and moves `trailing_garbage_token` to
  BOTH_ERR. That is the correct intermediate state: a hard error is strictly
  better than a silently different answer. Only stage 2 flips the NULLS cases to
  AGREE.
- **Note on fixtures:** every corpus data file is NULL-free, so these cases pin
  the *lost LIMIT* only — the actual NULL ordering semantics remain untested.
  Tier 08 needs a fixture containing NULLs before that can be asserted either
  way.

### P14 — An ungrouped aggregate over an empty set returns no row
- **Status:** 🔴 OPEN
- **Corpus:** `10_aggregate_nulls.toml :: count_star_empty`, `sum_empty`,
  `min_max_empty` (all DIFFER). Controls: `count_nonempty`,
  `grouped_aggregate_empty` (both AGREE).
- **Observed:** When the WHERE clause matches nothing, an aggregate with no
  `GROUP BY` returns **zero rows**; standard SQL (and DuckDB) returns **exactly
  one row**:

  | Query | DuckDB | sql-cli |
  |---|---|---|
  | `SELECT COUNT(*) FROM t WHERE <no match>` | 1 row: `0` | **0 rows** |
  | `SELECT SUM(amount) FROM t WHERE <no match>` | 1 row: `NULL` | **0 rows** |
  | `SELECT MIN(a), MAX(a) FROM t WHERE <no match>` | 1 row: `NULL, NULL` | **0 rows** |

- **Scope — deliberately narrow.** The *grouped* form is already correct:
  `... WHERE <no match> GROUP BY region` returns zero rows in both engines,
  which is right — no rows means no groups. The defect is confined to the
  ungrouped case, where the aggregate is over the whole (empty) input and must
  still produce its one row.
- **Decision:** **Fix.** An ungrouped aggregate query has exactly one output row
  by definition, independent of input cardinality. The fix must fill *every*
  output column (hence the multi-aggregate corpus case): `COUNT` → `0`, every
  other aggregate → `NULL`.
- **Why it matters:** this is the shape a dashboard or summary query takes.
  Returning no row where the caller expects one number is a wrong answer that
  reads as "no data" rather than "zero".

### P15 — `QUALIFY` rejects an inline window function
- **Status:** 🔴 OPEN
- **Corpus:** `09_window.toml :: qualify_row_number` (GAP). Controls:
  `window_row_number`, `qualify_select_list_alias` (both AGREE).
- **Observed:** `QUALIFY ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount
  DESC) = 1` → `Expected column name, got: WindowFunction { ... }`
  (`recursive_where_evaluator.rs:987`).
- **Precisely located by the controls.** QUALIFY is **not** broken in general:
  `QUALIFY rn = 1`, referencing an alias defined in the SELECT list, AGREEs. And
  the same window expression evaluates correctly in the SELECT list. It is only
  the **inline** form that fails.
- **Root cause:** the design is lifter-first — `ExpressionLifter` hoists window
  functions into a CTE column, then `QualifyToWhereTransformer` rewrites QUALIFY
  into a WHERE against that column. But the lifter only walks the **SELECT
  list**, so a window function written inline in QUALIFY is never hoisted and
  reaches the WHERE evaluator as a raw `WindowFunction`. The fix site is
  `expression_lifter`, not `qualify_to_where_transformer`.
- **Decision:** **Fix** via the [R2](ENGINE_REFACTORING.md) migration of
  `expression_lifter` (35 patterns, currently hand-rolled with a catch-all) —
  extending it to lift from the QUALIFY clause as well as the SELECT list. This
  is the R3 pattern again: the clause the transformer doesn't visit fails
  silently or loudly depending only on luck.

### P16 — `ORDER BY <ordinal>` is silently ignored
- **Status:** 🔴 OPEN
- **Corpus:** `08_ordering.toml :: order_by_ordinal`, `order_by_ordinal_desc`
  (both DIFFER).
- **Observed:** `ORDER BY 2` and `ORDER BY 2 DESC` return rows in **natural
  insertion order** — no sorting is applied at all, and no error is raised. The
  integer is evaluated as a constant expression, so every row compares equal.
- **Distinct from P13.** Nothing is dropped here; `ORDER BY 2` parses fine. The
  defect is that a positional reference is treated as a literal instead of being
  resolved to the 2nd select-list item.
- **Not implementation-defined.** Unlike P17 below, ordinals are standard SQL and
  every major engine resolves them. This is unambiguously a bug.
- **Decision:** **Fix.** Resolve an integer literal in `ORDER BY` to the
  corresponding select-list item (1-based), and error on out-of-range. Both
  corpus cases are needed: a fix that resolves the ordinal but drops `ASC`/`DESC`
  would still pass the first one.
- **Note:** this also silently corrupted an earlier probe of mine —
  `ORDER BY 2 DESC LIMIT 3` returned three rows, so it *looked* fine, but they
  were the first three in file order rather than the top three. Row count is not
  evidence of correct ordering.

### P17 — Default NULL placement differs on `ASC`
- **Status:** 🔴 OPEN — **decision made 2026-08-02: follow the reference engine
  (NULLS LAST in both directions), plus explicit `NULLS FIRST`/`LAST` from P13
  stage 2.** Implementation pending.
- **Corpus:** `08_ordering.toml :: order_by_null_default_asc_numeric`,
  `order_by_null_default_asc_string` (DIFFER); `order_by_null_default_desc`
  (AGREE). Second site: `09_window.toml :: win_first_value_unfiltered` (DIFFER).
- **Two sites, and they disagree with each other.** Added 2026-08-02 while
  fixing P21. Besides the main `ORDER BY` path, a window's *internal* `ORDER BY`
  sorts in `window_context.rs::sort_rows` — and it places NULLs **first on
  DESC**, the opposite of the outer `ORDER BY`, which places them last
  (`order_by_null_default_desc` AGREEs). So `FIRST_VALUE(score) OVER (ORDER BY
  score DESC)` over a partition of `(50, 50, NULL)` returns NULL where DuckDB
  returns 50. The fix has to reach both comparators, and the internal
  inconsistency is worth closing regardless of which rule wins.
- **Observed:** the two engines follow different rules, which happen to coincide
  on `DESC` and diverge on `ASC`:

  | | sql-cli | DuckDB |
  |---|---|---|
  | rule | NULL sorts as the **minimum value** | **NULLS LAST**, always |
  | `ORDER BY score` (ASC) | NULLs **first** | NULLs **last** |
  | `ORDER BY score DESC` | NULLs last | NULLs last |

- **Standard SQL leaves this implementation-defined**, and the major engines
  genuinely disagree: SQLite and MySQL treat NULL as smallest (our behaviour),
  PostgreSQL treats it as largest, DuckDB pins NULLS LAST in both directions.
  So this is a **choice to record**, not a defect to correct.
- **Options:**
  1. **Match DuckDB** (NULLS LAST always) — consistent with our reference engine
     and the least surprising to explain, but changes existing behaviour.
  2. **Keep NULL-as-minimum** and record as ⚪ WON'T FIX with this rationale.
  3. Either of the above **plus** implementing `NULLS FIRST` / `NULLS LAST`
     (see P13 stage 2), after which the default matters much less because users
     can be explicit.
- **Decision (2026-08-02): option 3, with option 1 as the default.** Where the
  standard leaves a choice open, we follow the reference engine — that is the
  whole point of having one, and it keeps "broad-brush parity" a single rule
  rather than a series of case-by-case judgements. Concretely:
  1. Change the default comparator so NULLs sort **last in both directions**.
  2. Implement explicit `NULLS FIRST` / `NULLS LAST` (P13 stage 2), after which
     the default matters much less because users can override it.
  This is a user-visible behaviour change on `ORDER BY <col>` over NULL-bearing
  data; call it out in the changelog when it lands.
- **Note:** `order_by_null_default_desc` AGREEs *for the wrong reason* — the two
  different rules coincide there. It is kept as a case precisely to document
  that. Under the decision above it will keep AGREEing, now for the right reason;
  the two ASC cases flip DIFFER → AGREE and their `expect` should be dropped.

### P18 — `= NULL` matches NULL rows instead of yielding UNKNOWN
- **Status:** 🔴 OPEN
- **Corpus:** `10_aggregate_nulls.toml :: where_equals_null` (DIFFER).
  Baseline: `where_is_null` (AGREE).
- **Observed:** `WHERE score = NULL` returns the four NULL-score rows. It is
  being treated as `IS NULL`. Under SQL three-valued logic `x = NULL` evaluates
  to UNKNOWN for **every** row — including rows where `x` is itself NULL — so
  the correct result is **zero rows**.
- **Decision:** **Fix.** `IS NULL` already works and is the only correct way to
  match a NULL, so no capability is lost by making `= NULL` never match.
- **Why it matters:** this is the direction that produces *extra* rows. A
  `WHERE col = <parameter>` that receives a NULL parameter silently returns the
  NULL rows instead of nothing — a wrong answer in the more dangerous direction.

### P19 — `NOT IN` does not exclude NULLs
- **Status:** 🔴 OPEN — same family as P18
- **Corpus:** `10_aggregate_nulls.toml :: where_not_in_excludes_null` (DIFFER).
  Baselines: `where_not_equal_excludes_null`, `where_in_with_null_col` (AGREE).
- **Observed:** `WHERE score NOT IN (50, 70)` returns 8 rows including the
  NULL-score rows; DuckDB returns 4. `NULL NOT IN (50, 70)` is UNKNOWN, not
  TRUE, so those rows must not pass.
- **Internally inconsistent, which is what makes it a bug.** The equivalent
  `WHERE score <> 50` already excludes NULLs correctly (pinned as a baseline).
  So we are not applying a considered "NULLs are comparable" rule — one operator
  propagates NULL and another does not.
- **Decision:** **Fix**, alongside P18 — both are the same missing
  three-valued-logic propagation, reached through different operators. Worth
  auditing `IN`, `NOT IN`, `BETWEEN`, `NOT BETWEEN` and `LIKE` together rather
  than patching the one operator the corpus happened to catch.

### P20 — `||` treats NULL as an empty string
- **Status:** 🔴 OPEN — **decision made 2026-08-02: propagate NULL through `||`,
  matching the reference engine.** Implementation pending.
- **Corpus:** `10_aggregate_nulls.toml :: null_concat` (DIFFER).
  Baseline: `null_arithmetic` (AGREE).
- **Observed:** `team || '-' || label` on a row where `label` is NULL gives
  `'alpha-'`; DuckDB gives `NULL`. Standard SQL propagates NULL through
  concatenation.
- **Arguably deliberate.** Oracle takes our view (NULL concatenates as empty),
  and "coercion-first" is an explicit design stance
  ([FEATURE_ROADMAP_2026_Q2.md](FEATURE_ROADMAP_2026_Q2.md)), so treating a
  missing string as empty is defensible for a data-exploration tool.
- **But note the inconsistency:** `score + 1` correctly yields NULL
  (`null_arithmetic` AGREEs). So arithmetic propagates NULL and concatenation
  does not. Whichever way this is decided, the two should agree on a principle.
- **Decision (2026-08-02): propagate NULL through `||`.** Follows the rule above
  — the standard is clear here and the reference engine agrees with it — and it
  removes the internal inconsistency, which was the harder thing to defend: a
  user cannot reasonably be told that `+` propagates NULL but `||` does not.
- **Watch for the coercion-first tension.** Empty-string coercion is presumably
  *convenient* when eyeballing concatenated columns over messy data, which is
  our core use case. If that turns out to matter in practice, the right answer
  is a `CONCAT()` function with the coercing behaviour — an explicit opt-in —
  rather than overloading `||`. Not needed until someone asks.
- **User-visible change:** any query concatenating a nullable column starts
  returning NULL rather than a partial string. Changelog it when it lands.

### P21 — Window functions are evaluated *before* the `WHERE` clause
- **Status:** 🟢 FIXED (2026-08-02)
- **Corpus:** `09_window.toml :: win_count_over_filtered`,
  `win_row_number_filtered`, `win_in_derived_table_filtered` (all now AGREE;
  `expect` dropped). Control: `win_partition_null_key` (AGREE throughout).
- **Observed:** with a `WHERE` clause present, window functions see the
  **unfiltered** row set. `COUNT(*) OVER (PARTITION BY team)` under
  `WHERE score IS NOT NULL` reports partition sizes alpha=3, gamma=2; the
  filtered sizes are alpha=2, gamma=1. `ROW_NUMBER` shows the same thing as
  rank slots consumed by rows that were filtered out.
- **Proved by the control:** the *identical* query without the `WHERE` AGREEs
  exactly with DuckDB, including the NULL partition. So partitioning, ordering
  and the functions themselves are correct — the defect is purely the position
  of window evaluation in the pipeline.
- **Correct semantics:** SQL evaluates window functions **after** `FROM`/`WHERE`/
  `GROUP BY`/`HAVING` and before `SELECT`-list projection, `ORDER BY` and
  `LIMIT`. Filtering must therefore happen first.
- **Scope:** affects *every* window query that also filters — which is most real
  ones. Silent in all cases.
- **A near miss worth recording:** `win_sum_partition_ordered` AGREEs under the
  same `WHERE`, purely because the filtered-out rows carry NULL scores that `SUM`
  ignores anyway. `COUNT(*)` is what makes this visible. A tier built only from
  `SUM` windows would have concluded windows were fine.
- **Root cause — the filter was plumbed but discarded.** Two halves, both needed:
  1. `arithmetic_evaluator.rs::get_or_create_window_context` matched on
     `self.visible_rows` and then **threw the result away** — both arms of the
     `if let` built the same unfiltered `DataView`, with a comment conceding
     "in production we'd need proper filtering". The binding was even named
     `_visible_rows`, so nothing warned.
  2. `query_engine.rs::apply_select_items` — the path where windows are actually
     evaluated — never called `.with_visible_rows(...)` at all, so
     `self.visible_rows` was `None` and even the dead branch was unreachable.

  The neighbouring aggregate paths in the same file *do* honour `visible_rows`
  (four call sites), which is why filtered aggregates were correct all along and
  only windows were wrong.
- **Fix:** pass the view's visible rows into the evaluator on the select-items
  path, and build the window `DataView` with `DataView::with_rows(...)`. The
  indices are source-table indices at every step — what `with_rows` expects and
  what `WindowContext::get_visible_rows()` reads back — so the whole path stays
  in one index space and no translation was needed.
- **One fix covered both code paths.** The batch and non-batch window evaluators
  share `get_or_create_window_context`, which is also why the batch-vs-fallback
  check had found them consistent: they were equally wrong.
- **Regression test:** `tests/window_after_where_tests.rs` — env-free, runs in
  plain `cargo test` (the corpus needs DuckDB and only runs in the Parity job).
  Verified to fail without the fix: the three bug-catching tests fail, while the
  two controls — the unfiltered case and the `SUM` near-miss — pass either way.

### P22 — Unimplemented window functions return NULL instead of erroring
- **Status:** 🔴 OPEN — **scope corrected 2026-08-02, now four functions not five**
- **Corpus:** `09_window.toml :: win_nth_value`, `win_ntile`, `win_percent_rank`,
  `win_cume_dist` (all DIFFER).
- **Observed:** `NTH_VALUE`, `NTILE`, `PERCENT_RANK` and `CUME_DIST` return
  **NULL for every row**, including over partitions containing no NULLs at all.
  No error, no warning.
- **Correction — `FIRST_VALUE` was never part of this.** It was originally filed
  here on the evidence that it returned NULL for every row. That was a
  misdiagnosis: FIRST_VALUE is fully implemented, and the NULLs were **P21**.
  The unfiltered partition still contained the NULL-score row, and the window's
  internal ORDER BY sorted that NULL to the front, so FIRST_VALUE faithfully
  returned it. Fixing P21 fixed the case with no work on FIRST_VALUE at all, and
  it now returns `90` over a NULL-free ordering. `win_first_value` is retained
  as an AGREE baseline. The residue is a genuinely separate defect, now filed
  under P17 as a second site — see `win_first_value_unfiltered`.
- **Lesson:** "returns NULL for everything" is not by itself evidence that a
  function is unimplemented. The distinguishing probe is whether it also returns
  NULL over data containing no NULLs, which is what separated the four real ones
  from the false positive.
- **Not a whole missing family:** `FIRST_VALUE`, `LAST_VALUE`, `LAG`, `LEAD`,
  `ROW_NUMBER`, `RANK`, `DENSE_RANK` and the aggregate-OVER forms all work and
  are pinned as baselines.
- **Decision:** **Fix in two steps, and do the second first if the first is
  slow.** (1) Implement the five functions. (2) Independently, make an
  unrecognised window function a **hard error** rather than a NULL column — the
  silence is worse than the absence, because a NULL column reads as "no data"
  rather than "unsupported".
- **Related:** same class as P13 — unsupported input degrading into a plausible
  wrong answer instead of a refusal.

### P23 — `LAG`/`LEAD` ignore the third (default) argument
- **Status:** 🔴 OPEN
- **Corpus:** `09_window.toml :: win_lag_offset_default` (DIFFER).
  Baselines: `win_lag`, `win_lead` (AGREE).
- **Observed:** `LAG(score, 2, -1)` honours the offset — the 1-arg form is
  already correct — but drops the default, so rows past the partition edge come
  back NULL instead of `-1`.
- **Decision:** **Fix.** Small and self-contained: thread the third argument
  through as the out-of-range fallback.

### P24 — A `RANGE` frame is treated as `ROWS`
- **Status:** 🔴 OPEN
- **Corpus:** `09_window.toml :: win_range_frame_with_ties`,
  `win_default_frame_ordered` (both DIFFER). Baselines: the three explicit
  `ROWS` frame cases (all AGREE).
- **Observed:** with ties in the ORDER BY key, `RANGE BETWEEN UNBOUNDED
  PRECEDING AND CURRENT ROW` must include **all peer rows** at the current
  value. At `score = 50` (two peers) DuckDB returns 160; we return 110 — one
  peer only, i.e. ROWS behaviour.
- **The damaging half is the default frame.** With an `ORDER BY` in the window
  and no explicit frame, the SQL default is `RANGE UNBOUNDED PRECEDING AND
  CURRENT ROW`. `SUM(x) OVER (ORDER BY y)` is a far more common way to write a
  running total than any explicit frame, and it is silently wrong wherever `y`
  has duplicates. Explicit `ROWS` frames are unaffected and already correct.
- **Only detectable because the fixture has ties** — on distinct keys ROWS and
  RANGE coincide, which is why this survived until `null_edges.csv` existed.
- **Decision:** **Fix.** Implement peer-group semantics for `RANGE`, and make
  the no-frame-with-ORDER-BY default resolve to `RANGE` rather than `ROWS`.

### P25 — A window's `ORDER BY` accepts only a plain column
- **Status:** 🔴 OPEN
- **Corpus:** `09_window.toml :: win_order_by_expression` (GAP).
- **Observed:** `RANK() OVER (ORDER BY score * -1)` → "Window function ORDER BY
  ...". An expression inside the window's `ORDER BY` is rejected, though the
  *outer* `ORDER BY` handles expressions fine (`08_ordering.toml ::
  order_by_expression` AGREEs).
- **Decision:** **Fix** — a hard error, so no silent-wrong-answer urgency, but
  it is an arbitrary restriction that the outer clause does not share.
- **Note:** [R2](ENGINE_REFACTORING.md) records that `WindowSpec::order_by` is
  now descended into by the walk helpers, so the AST side is already reachable;
  this looks like an evaluator restriction rather than a traversal gap.

### P26 — A window function over an aggregate is rejected under `GROUP BY`
- **Status:** 🔴 OPEN
- **Corpus:** `09_window.toml :: win_over_aggregate_with_group_by` (GAP).
- **Observed:** `SELECT team, SUM(score) AS s, RANK() OVER (ORDER BY SUM(score)
  DESC) FROM ... GROUP BY team` → "Expression 'v' must appear in GROUP BY
  clause". The window alias is being subjected to the GROUP BY validity check,
  although window functions are evaluated *after* grouping and are not
  themselves grouped expressions.
- **Why it matters:** ranking groups by an aggregate is the standard "top N per
  group" shape. `CLAUDE.md` already documents a CTE workaround ("Window
  functions can't handle expressions directly. Use CTEs to pre-calculate"), so
  this restriction is known in practice but was never written down as a gap.
- **Decision:** **Fix.** Exclude window-function outputs from the GROUP BY
  validity check; they belong to the post-aggregation stage. Note this is the
  same pipeline-position confusion as P21, approached from the other end — both
  come down to *when* windows are evaluated relative to the rest of the query.

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
