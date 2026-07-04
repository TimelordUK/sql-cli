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

### P4 — Self-join of the base table fails to resolve
- **Status:** 🔴 OPEN
- **Corpus:** `04_joins.toml :: self_join_base`
- **Observed:** `FROM trades a JOIN trades b ...` → "Cannot resolve table 'trades'
  for JOIN". Joins to **derived tables / CTEs** built from the same source already
  work (those cases AGREE); only re-referencing the base table by name fails.
- **Decision:** **Fix** — register the loaded source so it can be referenced more
  than once (with aliases) in a join.

### P5 — `CROSS JOIN` to a FROM-less subquery has wrong cardinality
- **Status:** 🔴 OPEN
- **Corpus:** `04_joins.toml :: cross_join_constant`
- **Observed:** `trades t CROSS JOIN (SELECT 1 AS k) c` returns 92×92 = 8464 rows
  instead of 92. A FROM-less subquery (`SELECT 1 AS k`) yields one row per outer
  row instead of a single constant row.
- **Decision:** **Fix** — a FROM-less SELECT must produce exactly one row.

### P6 — `INTERSECT` / `EXCEPT` not implemented
- **Status:** 🔴 OPEN
- **Corpus:** `06_ctes_setops.toml :: intersect`, `except`
- **Observed:** "INTERSECT is not yet implemented" / "EXCEPT is not yet implemented".
  `UNION` and `UNION ALL` already AGREE.
- **Decision:** **Fix** — implement alongside the existing `UNION` set-op path.

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
