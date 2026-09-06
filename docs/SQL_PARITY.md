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

Corpus coverage today: tiers 01–10, **181 cases** (156 AGREE / 10 DIFFER /
12 GAP / 1 OURS_ONLY / 2 BOTH_ERR as of 2026-09-06, after [P41](#p41) added four
MODE cases). The largest single movement so far remains the 2026-09-05
NULL-ordering slice, which closed [P13](#p13) stage 2 and [P17](#p17) together —
eleven cases in one change. **Tier 10 (aggregate & NULL edges) is still
deliberately partial** — it holds the P14, P18–P20 and P41 cases and their
baselines, but was never built out the way tiers 08 and 09 were. Finish it during
a lull; the aggregate-function surface (`STDDEV`, `DISTINCT` aggregates,
`FILTER`, empty-vs-all-NULL distinctions) is largely unexamined, and P41 showed
that surface is thinner than it looks.

Suggested fix order, by silent blast radius:

| | Finding | Why first |
|---|---|---|
| ~~1~~ | ~~[P21](#p21) windows evaluated before `WHERE`~~ | ✅ **Fixed 2026-08-02** |
| ~~2~~ | ~~[P13](#p13) trailing tokens discarded~~ | ✅ **Stage 1 done 2026-08-02**; stage 2 (`NULLS FIRST`/`LAST`) is in row 9b below |
| ~~3~~ | ~~[P30](#p30) `cond AND col IN (list)` returns 0 rows~~ | ✅ **Fixed 2026-08-08** |
| ~~5~~ | ~~[P29](#p29) boolean operator after `IN (...)`~~ | ✅ **Fixed 2026-08-08** — same bug as P30, one change closed both |
| ~~4~~ | ~~[P28](#p28) `INTO #tmp` stages unfiltered rows~~ | ✅ **Fixed 2026-08-16** — turned out to stage the *whole source table*, and the sweep it prescribed found [P31](#p31) |
| ~~7~~ | ~~[P18](#p18)/[P19](#p19) three-valued logic~~ | ✅ **Fixed 2026-08-22** — 125 → 129 AGREE. Delivered as three slices ([R10](ENGINE_REFACTORING.md#r10)); the two no-op ones landed first, so the semantics change reviewed on its own |
| ~~8~~ | ~~[P24](#p24) `RANGE` treated as `ROWS`~~ | ✅ **Fixed 2026-08-30** — 129 → 133 AGREE (+2 fixed, +2 new coverage). One defect, not two: the parser already emitted the right default frame, so fixing peer groups closed both cases. Spun off [P33](#p33) |
| ~~9a~~ | ~~[P16](#p16) `ORDER BY <ordinal>` ignored~~ | ✅ **Fixed 2026-08-31** — 134 → 139 AGREE. The literal was being promoted into a hidden *constant* column, so the sort ran on a column where every row tied |
| ~~9c~~ | ~~[P17](#p17) + [P13](#p13) stage 2 — NULL ordering~~ | ✅ **Fixed 2026-09-05** — 141 → **152 AGREE**, eleven cases in one change. Both halves were the same comparator's NULL rule, so they were taken as one slice. The two sorts that disagreed with each other now *share* one function (`compare_for_order_by`), which is the part that stops the divergence recurring; the window site turned out to be sorting NULL as the **maximum** via a derived `PartialOrd`, not merely following a different rule |
| ~~9d~~ | ~~[P37](#p37) window in `WHERE` returns 0 rows~~ | ✅ **Fixed 2026-09-05** — corpus count unchanged, and that is the finding: the case is `OURS_ONLY` before *and* after, so the harness cannot see this fix or a future regression of it (first entry of that kind — the regression test is a Rust module). The filed root cause was wrong: `ExpressionLifter` *does* lift from `WHERE`. The real defect was one arm in the WHERE evaluator answering FALSE for any bare value used as a predicate — `WHERE true` returned zero rows too. It did **not** close [P15](#p15), which needs the opposite change |
| ~~9e~~ | ~~[P41](#p41) `MODE` tie-break is random per run~~ | ✅ **Fixed 2026-09-06** — 152 → **156 AGREE** (four new cases). Small, as predicted, but not where it was filed: the named `ModeState` was a *shadowed* implementation and fixing it moved nothing. Reference does specify a rule and it is **first-occurrence**, not the "smallest value wins" this row proposed. Unblocked both example files, now FORMAL. Spun off [P42](#p42), [P43](#p43), [R12](ENGINE_REFACTORING.md#r12) |
| **NEXT** | [P14](#p14), [P20](#p20), [P23](#p23) | Smaller, self-contained, decisions already taken. Was row 9b |
| 9f | [P42](#p42) `MODE` is numeric-only | Companion to [R12](ENGINE_REFACTORING.md#r12), and cheap if taken with it: the shadowed implementation already handles non-numerics and preserves type, so the fix is largely to stop the live path throwing away what it knows. Also buys the corpus its clearest tie-break case |
| 10 | [P22](#p22), [P25](#p25), [P26](#p26), [P15](#p15), [P32](#p32), [P38](#p38) | Hard errors — visible, so less urgent than any of the above |
| 10b | [P39](#p39) `x/0` errors, voiding the whole statement | Hard error like row 10, but the only one whose blast radius is the *query* rather than the cell. Settle the four inconsistent call sites as one decision; it currently has no live probe (see the entry) |
| 11 | [P35](#p35), [P36](#p36), [P43](#p43) | Not parity obligations — a DuckDB extension, a naming difference, and a `RANGE` endpoint convention. Decide *whether*, not just when. P43 is the one with a migration cost attached, so it wants deciding before it accumulates more callers |
| 12 | [P40](#p40) a generator's args can't reference columns | Hard error and loudly signposted, so last by blast radius. **Piece 1 (the message) shipped 2026-09-06** after it misdirected a second investigation; pieces 2 (resolve args against the real table) and 3 (row-wise explode) remain, and the explode feature still waits on the `UNNEST` decision |
| — | [P27](#p27) `OR` in `JOIN ... ON` | **Reclassified 2026-08-08, re-scoped 2026-09-04 — possibly smaller than it was filed as.** The AST is still the blocker (`JoinCondition` is a `Vec` of AND-ed conditions with nowhere to put an `OR`), but the executor already evaluates expressions per row pair and already has a merged-row `cross_join`, so `INNER JOIN ON <expr>` may lower to cross-join + the R10 WHERE evaluator. Do the timeboxed scoping pass in the entry before sequencing this |

P27–P30 jump the queue because all four were found *by* fixing something else,
and all four are the dangerous shape: wrong data that looks plausible. Two of
them hide themselves especially well — P28's "too many rows" invites adding
another filter downstream until the output looks right, and P30's zero rows
reads as "no matching data" rather than as a defect.

That four serious silent bugs fell out of fixing one parser check is the
strongest argument yet for the corpus approach: none of them had a failing unit
test, and two had *passing* ones asserting the broken behaviour.

**A third lesson, from closing P28 (2026-08-16): a finding is written up from
one probe, and inherits that probe's blind spot.** P28 was filed as "the WHERE is
ignored" because it was diagnosed with `SELECT COUNT(*)`, which can only report a
row count. The staged table was in fact the *entire source table* — every column,
plus the ordering lost. Re-probe a finding from a different angle before scoping
the fix; the entry describes the symptom that was looked for, not necessarily the
defect. The same session's sweep for other consumers of the same route turned up
[P31](#p31), a live zero-rows bug in the `--limit` flag that nobody had reported.

**A fourth lesson, from closing P41 (2026-09-06): the entry named a fix site, and
the fix site was dead code.** `ModeState` in `src/sql/aggregates/mod.rs` is
exactly what you find by grepping for MODE, and it is shadowed — a second, newer
aggregate registry is consulted first, so the live implementation is a different
tally in a different file ([R12](ENGINE_REFACTORING.md#r12)). The fix was applied,
the tests passed, and the repro still flipped between runs. Generalised: **an
entry's "Where" line is a lead, not a location. Confirm a fix site by changing it
and watching the symptom move.** Where two implementations of the same operator
exist, expect the *older-looking* one to be the dead one, since migrations here
add to the new registry without removing from the old.

**A fifth, cheaper one from the same session: check the fixture, not just the
query.** P41's repro leaned on `RANGE(1,50)` being a 25/25 even/odd split. It is
— for us. DuckDB's `range` stop bound is exclusive, so the same query is 25/24
there and the tie the repro depends on does not exist. That is now [P43](#p43),
and it was found only because a corpus case forces both engines to run the same
text. Any repro written against one engine carries assumptions about that
engine's builtins.

**Two lessons from closing P29/P30 (2026-08-08), both worth generalising:**

1. **Two findings filed as different classes were one bug.** P29 was "a parse
   gap", P30 "an evaluation bug", and they were the same misplaced precedence
   seen from two operand orders. Before fixing findings that sit in the same
   area, check `--query-plan` on each — a mis-parse can yield a well-formed AST
   that simply means something else, which is indistinguishable from an
   evaluation fault by result alone.
2. **Closing a parse error can expose a second, unrelated divergence beneath
   it.** `in_subquery_then_and` went GAP → DIFFER, not GAP → AGREE, because the
   parse error had been masking P18. That is progress and the case stays pinned,
   now against P18. Expect this whenever a fix makes previously-unrunnable
   queries run: budget for the finding underneath rather than assuming the
   bucket goes straight to AGREE.

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
- **Status:** 🟢 FIXED — stage 1 (reject trailing input) 2026-08-02, stage 2
  (`NULLS FIRST` / `NULLS LAST`) 2026-09-05 in one slice with [P17](#p17).
- **Corpus:** `08_ordering.toml :: trailing_garbage_token` (BOTH_ERR — the root
  cause, pinned directly), `order_by_nulls_last_limit` and
  `order_by_nulls_first_limit_nullfree` (the instance users actually hit; both
  AGREE since stage 2). Controls: `order_by_limit`,
  `order_by_nulls_last_no_limit`. The stage-2 acceptance criteria are the five
  `null_edges.csv` cases listed below, not these.
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
- **Acceptance test for stage 2:** `examples/jsonl_logs.sql` carries the
  `ORDER BY latency_ms DESC NULLS LAST LIMIT 5` statement that motivated this,
  which was marked `-- [SKIP]` (2026-09-02) so the examples smoke run stayed
  green, next to the NULLS-free version that did run. **The `[SKIP]` was dropped
  2026-09-05** and both statements now return the identical top 5 — the
  filtered set contains a row with a NULL `latency_ms`, so `NULLS LAST` is doing
  real work there and the default agreeing with it is the P17 change visible
  end-to-end. Note the JSONL fixture *does* contain NULLs inside the filtered set
  (16 rows pass `status IS NOT NULL`, one of which has a NULL `latency_ms`),
  so unlike the corpus it can pin real NULL-ordering semantics, not just the
  lost `LIMIT`.
- **Note on fixtures — superseded 2026-09-04, and the correction matters.** This
  entry used to say every corpus data file was NULL-free. That was true when it
  was written; `null_edges.csv` (12 rows, NULLs in five columns, both engines
  verified to see the same ones) was purpose-built for tier 08 shortly after, and
  the P17 cases already use it. But **the three `order_by_nulls_*` cases above
  were never moved onto it** — they still run on `international_sales.csv`, which
  is 100% NULL-free (re-verified 2026-09-04: 0 empty fields in 20 rows).
  Consequence: those three pin *"the clause parses and the `LIMIT` survives"* and
  nothing more. An implementation that accepts `NULLS LAST` into the AST and then
  ignores it in the comparator flips all three to AGREE and passes `--check`.
  **They are not the acceptance criteria for stage 2.**
- **Acceptance criteria for stage 2 (added 2026-09-04):** five cases on
  `null_edges.csv` — `order_by_nulls_first_asc_numeric`,
  `order_by_nulls_last_asc_numeric`, `order_by_nulls_first_desc_numeric`,
  `order_by_nulls_first_string`, `order_by_nulls_first_limit` (all `GAP`; the
  parse error is the current state). The load-bearing ones are the **`NULLS
  FIRST`** cases: once [P17](#p17) makes NULLS LAST the default in both
  directions, an explicit `NULLS LAST` agrees with the default whether or not the
  clause is honoured, so only `NULLS FIRST` can fail on a parsed-and-dropped
  clause. `order_by_nulls_first_limit` carries both halves of P13 at once and is
  the sharpest single check: the answer is exactly ids 3, 10, 11 (all NULL-scored),
  so a dropped clause returns a different *set*, not a subtly different order.
  All five closed 2026-09-05, `order_by_nulls_first_limit` included.
- **Stage 1, as built (2026-08-02).** `Parser::parse` now ends with
  `expect_end_of_statement()`, which accepts an optional trailing `;` and
  trailing comments and otherwise errors with the offending token and its
  position. Three supporting fixes were needed, each a real bug in its own right:
  1. **`;` became a real token.** It had been falling through the lexer's
     catch-all as `Identifier(";")`. Adding `Token::Semicolon` required exactly
     **one** exhaustive match arm across the whole tree — a neat measurement of
     [R3](ENGINE_REFACTORING.md): everywhere else would have silently ignored it.
  2. **`''` escapes in string literals were never implemented.** `read_string`
     stopped at the first inner quote, so `'O''Brien'` lexed as two literals and
     the parser discarded the second — the query meant `WHERE name = 'O'`. There
     was a passing test asserting that query "should parse"; it did, into the
     wrong thing. Now matches DuckDB on `'O''Brien'`, `'a''b''c'` and `''`.
  3. **`;`-separated statements were being dropped.** Script batches split on
     `GO` only, so `a; b;` inside one batch parsed `a` and silently discarded
     `b`. `prime_numbers.sql` had a whole `SELECT` that had never run. Batches
     are now sub-split on top-level `;` (quote- and comment-aware, so a `;`
     inside a literal does not split), and `-q`/`-f` input holding more than one
     statement routes to the script executor instead of the single-query path.
     `GO` semantics are untouched, and statement *scope* is unaffected — the
     executor builds one `ExecutionContext` per script, so `INTO #tmp` stays
     visible across both separators (verified explicitly).
- **Stage 2, as built (2026-09-05).** `OrderByItem` gained a `nulls:
  NullsOrder` field (`Unspecified` / `First` / `Last`), and
  `OrderByItem::nulls_first()` is the single place the default lives — see
  [P17](#p17) for the comparator work it feeds.
  - **`NULLS`, `FIRST` and `LAST` were deliberately *not* made keywords.** They
    are matched contextually, as identifiers, in the one position they can
    appear. All three are plausible column names — `first` and `last`
    especially — and reserving them would have broken queries that have nothing
    to do with NULL ordering, in a codebase that reads user CSVs with arbitrary
    headers. `SELECT nulls FROM t ORDER BY nulls` still parses, and there is a
    test that says so.
  - **A malformed clause errors rather than being ignored**, which is stage 1's
    rule applied to the feature stage 1 was blocking: `NULLS SIDEWAYS` names the
    offending token.
  - **The formatters round-trip what was typed, not what was resolved.** That is
    why `Unspecified` is a distinct variant from `Last` even though the two mean
    the same thing to the comparator — printing `NULLS LAST` onto a query the
    user never wrote it in would be a silent edit.
- **What stage 1 exposed.** Beyond the above: `OR` in a `JOIN ... ON` clause is
  not parsed ([P27](#p27)), and a stray `end` token had been sitting unnoticed in
  `examples/case_when.sql`. Two examples (`prime_numbers`,
  `physics_astronomy_showcase`) gained a result-set each once their dropped
  statements started running; both were checked before re-capturing.

### P27 — `OR` in a `JOIN ... ON` clause is not parsed
- **Status:** 🔴 OPEN — **high priority: was a silent wrong answer until P13**
- **Corpus:** `04_joins.toml :: join_on_or_condition` (GAP).
- **Observed:** `... INNER JOIN b ON a.x = b.x OR a.y = b.y` parses only the
  first condition. Until P13 stage 1 the `OR ...` remainder was **silently
  discarded**, so the join ran on a *truncated predicate* and returned wrong
  rows with no error. Since stage 1 it is a parse error, which is why this is
  filed as a GAP rather than a DIFFER.
- **Found:** 2026-08-02, by P13 stage 1 rejecting what it had previously
  swallowed — in `examples/chemistry.sql`, which had been shipping wrong results
  from `ON Year = latest_year OR Year = earliest_year`.
- **Decision:** **Fix.** Join conditions should accept the same boolean
  expressions `WHERE` does; `AND` already works (multi-condition joins are
  well covered — see P7/P8), so this is `OR` specifically.
- **Acceptance test:** the `examples/chemistry.sql` statement that found this
  is still in the file, marked `-- [SKIP]` (2026-09-02) so the examples smoke
  run stays green. Drop the directive when P27 is fixed — it is the
  end-to-end check, alongside the corpus case.
- **Re-scoped 2026-09-04 — the 2026-08-08 "larger than P3" assessment looks
  pessimistic, and it predates a look at the executor.** Two things narrow it:
  1. **Per-row-pair expression evaluation already exists.**
     `nested_loop_join_inner_multi` / `_left_multi` / `_right_multi`
     (`data/hash_join.rs`) evaluate `SqlExpression` operands for each row pair,
     routing each operand to its owning table by alias rather than syntactic
     position (that is the P7 fix). Arbitrary join predicates are not new ground
     in the executor; only the *shape of the condition* is.
  2. **There is already a merged-row substrate.** `hash_join.rs:789 cross_join`
     materializes a table carrying both sides' columns — which is exactly what
     the Trilean WHERE evaluator from [R10](ENGINE_REFACTORING.md#r10) needs.
     `INNER JOIN ON <expr>` lowers to cross join + WHERE filter, correct by
     construction, and NULL-in-`ON` then falls out of R10 instead of needing its
     own rule.

  So the blocking work is the AST (`JoinCondition` stops being a `Vec` of
  AND-ed conditions) and the parser's AND-only loop at
  `recursive_parser.rs:1987-1992`, plus a decision on which executor path an
  `OR` predicate takes.
- **Caveats, deliberately not discharged — this is a scoping note, not a plan:**
  - **Outer joins cannot use the lowering.** They need "did any right row match"
    tracking, so cross-join-then-filter is INNER-only.
    `nested_loop_join_left_multi` already tracks that, so the shape exists, but
    the shortcut does not extend to it.
  - **`cross_join` is not drop-in.** It caps at 1M result rows for safety, and
    it clones both sides' columns *without* the `_right` collision renaming the
    nested-loop paths do — so overlapping column names would resolve differently
    than they do today.
  - **It is O(n×m).** So is the nested-loop path it would replace, so no
    regression for the shapes already routed there — but the pure
    equality-AND case must keep taking the hash path, or this becomes a
    performance regression dressed as a feature.
- **Next action: a timeboxed scoping pass, not a fix.** Prove the lowering
  end-to-end against `join_on_or_condition` first. That is the cheap experiment,
  and it is what decides whether P27 is a session or a project — which is the
  question the 2026-08-08 note left open.

### P28 — `SELECT ... INTO #tmp` stages the *unfiltered* rows
- **Status:** 🟢 FIXED (2026-08-16)
- **Corpus:** none possible — `INTO #tmp` is our own extension and the reference
  engine has no equivalent, so the oracle here is **our own behaviour
  disagreeing with itself**. Pinned instead by
  `tests/python_tests/test_temp_table_staging.py` (end-to-end, script path) and
  `tests/view_materialization_tests.rs` (env-free, engine level).
- **Observed:** the `WHERE` clause is ignored when the result is staged:

  ```sql
  SELECT COUNT(*) FROM null_edges WHERE score IS NOT NULL;      -- 8  correct
  SELECT id, score INTO #a FROM null_edges WHERE score IS NOT NULL;
  SELECT COUNT(*) FROM #a;                                      -- 12 WRONG
  ```

  Both `INTO` placements (before `FROM`, and after all clauses) do it.
- **Confirmed pre-existing**, not introduced by the P13 work: reproduced from a
  clean build of `main` with all local changes stashed.
- **Wider than filed.** The staged table was the **whole source table**, not a
  filtered copy of the result: `SELECT id, score INTO #a` staged all six columns
  of `null_edges`, and `ORDER BY` was lost too. The finding was written up from a
  `COUNT(*)`, which could only ever show the row count.
- **Why it matters more than the row count suggests.** This is the
  stage-then-combine workflow — pull several sources, stage each into a temp
  table, join them at the end. Every staged table silently contains rows the
  user filtered out, and the error only shows up as "too many rows" much later,
  by which point the natural response is to add *more* filters downstream until
  the output looks right — which masks it permanently.
- **Same family as [P21](#p21):** a filter that is applied on the direct path but
  not on a secondary one.
- **Root cause — one line, and a helper that was almost right.** A `DataView`
  carries three things the source table does not: the `WHERE` filter, the
  `SELECT` projection, and the `LIMIT`/`OFFSET` window. The script path
  (`non_interactive.rs`) stored `final_view.source_arc()` — the *backing table*,
  reached past the view entirely. `QueryEngine::materialize_view` is the shared
  helper that does this correctly and **two other call sites already used it**;
  the script path, which is the one that actually runs `INTO`, did not.
- **A second defect underneath, in the shared helper.** `materialize_view`
  iterated `visible_row_indices()`, documented as *"before limit/offset"*, so
  `SELECT id INTO #a FROM t LIMIT 3` staged all 12 rows — through the two call
  sites that were otherwise correct. Fixed by adding
  `DataView::windowed_row_indices()` (the post-limit set, matching what
  `row_count()` counts) and materializing from that. The pre-limit accessor is
  still there for callers that mean it; its doc comment now names the trap.
- **The sweep this entry asked for found a third site:** [P31](#p31), the
  `--limit` flag. Recorded separately because the symptom is different.

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
- **Not closed by the [P37](#p37) fix, and no longer its sibling.** The two were
  filed as one piece of work on the belief that the lifter skipped `WHERE` and
  `QUALIFY` alike. It does not skip `WHERE` — see P37 — so that fix went into
  the WHERE evaluator and left this untouched; inline `QUALIFY` still errors
  exactly as recorded above. What P15 actually needs is the *opposite* of what
  P37 needed: `QualifyToWhereTransformer` currently runs **after**
  `ExpressionLifter` (`src/query_plan/mod.rs`), on the reasoning that windows
  should already be lifted by the time QUALIFY is rewritten. For the alias form
  (`QUALIFY rn <= 3`) that holds. For the inline form it is backwards — rewrite
  QUALIFY to WHERE *first* and the WHERE lifting path, which works, picks it up
  for free. **Check the alias form against the swap before taking it**: the
  ordering was chosen for that case, and `WhereAliasExpander` runs after both.

### P16 — `ORDER BY <ordinal>` is silently ignored
- **Status:** 🟢 FIXED 2026-08-31 — 134 → 139 AGREE (+2 fixed, +3 new coverage,
  +1 new BOTH_ERR)
- **Corpus:** `08_ordering.toml :: order_by_ordinal`, `order_by_ordinal_desc`
  (were DIFFER, now AGREE). Added with the fix: `order_by_ordinal_star`,
  `order_by_ordinal_group_by`,
  `order_by_ordinal_expression_is_not_positional` (AGREE) and
  `order_by_ordinal_out_of_range` (BOTH_ERR).
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

#### As built (2026-08-31)

The root cause was not in the sort at all, and not where the entry above guessed
("the integer is evaluated as a constant expression"). It is close, but the
mechanism matters for where the fix goes:
`OrderByAliasTransformer::promote_hidden_order_by_columns` exists to keep an
ORDER BY key alive through projection, and it promotes **anything that is not a
visible column** into a hidden SELECT item. `NumberLiteral("2")` is not a
column, so it was promoted as a hidden column *whose value is the constant 2* —
after which the engine sorted, correctly, on a column where every row compares
equal. Nothing was ignored; the wrong thing was sorted on.

**The ordinal is resolved during execution, not in the transformer.** The
transformer knows the select list but not the *output* columns, and the two
differ in exactly the cases that matter: `SELECT *` is still unexpanded there,
and GROUP BY has not run. `apply_multi_order_by_with_context` in
`query_engine.rs` sees the projected view and resolves all three shapes with one
rule, so the transformer's only job is to stop promoting numeric literals.

Rules pinned against DuckDB before implementing, rather than assumed:

| Query | Behaviour |
|---|---|
| `ORDER BY 2` | 2nd output column — under an explicit select list, `SELECT *`, or after GROUP BY |
| `ORDER BY 1+1` | **not** an ordinal: an ordinary constant expression, sorts nothing |
| `ORDER BY 0`, `ORDER BY -1`, `ORDER BY 3` (of 2) | error, `should be between 1 and N` |
| `ORDER BY 1.5` | error — DuckDB: *"ORDER BY non-integer literal has no effect"* |

The last row is why the transformer skips **every** numeric literal and not just
integer-valued ones: leaving `1.5` to be promoted would have kept it a silent
no-op, and only the engine knows the valid range to report. That is the P13
principle — a refusal beats a different query that succeeds.

**Hidden columns are excluded from the ordinal range.** Columns promoted for
ORDER BY visibility (and HAVING's `__hidden_agg_` columns) are appended *after*
the real output, so `SELECT a, b FROM t ORDER BY c, 3` must error rather than
resolve to `__hidden_orderby_1`. Pinned by
`test_order_by_ordinal_ignores_promoted_hidden_columns`.

**A note on picking the corpus cases.** Two cases would have been enough to turn
the entry green, and would have left the two most valuable shapes untested: the
defect is about *output* columns, so every construct that changes what the
output columns are is a separate risk. `SELECT *` and GROUP BY were added for
that reason, and `order_by_ordinal_group_by` is the one that matters most in
practice — "top N by total" is where silently returning group order is most
likely to be believed.

### P17 — Default NULL placement differs on `ASC`
- **Status:** 🟢 FIXED 2026-09-05, in one slice with [P13](#p13) stage 2 — same
  comparators, and the decision on both was taken in the same sitting
  (2026-08-02). Parity 141 → **152 AGREE**; eleven cases closed at once.
- **Corpus:** `08_ordering.toml :: order_by_null_default_asc_numeric`,
  `order_by_null_default_asc_string` (were DIFFER, now AGREE);
  `order_by_null_default_desc` (AGREE throughout). Second site:
  `09_window.toml :: win_first_value_unfiltered` (was DIFFER, now AGREE).
- **What proved the default actually moved:** the two ASC `DIFFER` cases flipped
  to AGREE. Note `order_by_null_default_desc` could not help — it AGREEd before
  for the wrong reason and still AGREEs, now for the right one. The
  explicit-clause case `order_by_nulls_last_asc_numeric` (added 2026-09-04, see
  P13) pins the new default and the explicit form converging on one answer.
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
  data; called out in `CHANGELOG.md` under `[Unreleased]`. Note that file had
  gone stale (newest entry 1.69.1 against a 1.83.5 `Cargo.toml`) and no earlier
  parity fix was logged in it — an `[Unreleased]` section was opened rather than
  inventing a version number.
- **Note:** `order_by_null_default_desc` AGREEs *for the wrong reason* — the two
  different rules coincide there. It is kept as a case precisely to document
  that.
- **Fix, as built (2026-09-05).** One comparator now serves every `ORDER BY` in
  the engine: `datavalue_compare::compare_for_order_by(a, b, ascending,
  nulls_first)`. Both call sites — `DataView::apply_multi_sort` and
  `window_context::compare_by_sort_cols` — delegate to it, which is what
  actually stops the two from drifting apart again; the corpus cases only prove
  it for one shape each. Three things worth recording:
  1. **NULL placement is applied *before* direction and is never reversed by
     it.** `NULLS LAST` means last in the output whichever way the values sort.
     A comparator that reversed the NULL arm along with the values would pass
     every ASC case and fail the DESC ones, so both directions are asserted.
  2. **The window site was worse than "a different rule".** It compared
     `DataValue`s through their *derived* `PartialOrd`, which orders by variant
     index — and `Null` is the last variant, so NULL sorted as the **maximum**,
     then got reversed by DESC into first place. The same derived ordering also
     compared cross-type values by variant rather than by value, so
     `Integer(100)` sorted below `Float(1.0)` in a window's internal sort.
     Routing this site through the shared comparator fixed that too; it has its
     own regression test.
  3. **`compare_datavalues` was deliberately left alone.** It still sorts NULL
     as the minimum, because it is shared with aggregates, `MIN`/`MAX` and TUI
     column sorting, where that is not the same question. The `ORDER BY` rule
     lives in one wrapper rather than in the general-purpose comparator.
- **A third comparator exists and was *not* changed:**
  `csv_datasource.rs::sort_results` sorts `serde_json::Value`s and places NULLs
  first. It is unreachable — it hangs off `CsvApiClient`, which `buffer.rs`
  keeps only "for API compatibility" and never calls, and the `DataSourceAdapter`
  that would reach it has no callers either. Left as-is rather than fixed
  blind; noted here so that whoever revives that path knows it needs the same
  rule. Reviving it without this is a silent divergence, not a compile error.

### P18 — `= NULL` matches NULL rows instead of yielding UNKNOWN
- **Status:** 🟢 FIXED 2026-08-22, with P19 — branch
  `fix/p18-p19-three-valued-logic`. Parity 125 → **129 AGREE**; four cases
  closed at once (`where_equals_null`, `where_not_in_excludes_null`,
  `in_list_with_null_literal`, `in_subquery_then_and`).
- **Delivered in three slices** ([R10](ENGINE_REFACTORING.md#r10)), only the
  last of which changed any result: the `Trilean` type and its truth tables
  (1a), the evaluator converted to `Result<Trilean>` with one collapse point
  (1b), then UNKNOWN produced at the leaves (1c). The first two were provable
  no-ops and shipped separately, so the review of the semantics change was a
  ~40-line diff rather than one buried in 200 lines of signature churn.
- **Corpus:** `10_aggregate_nulls.toml :: where_equals_null` (DIFFER).
  Baseline: `where_is_null` (AGREE).
  Also `02_where.toml :: in_list_with_null_literal`, `in_subquery_then_and` (DIFFER).
- **Observed:** `WHERE score = NULL` returns the four NULL-score rows. It is
  being treated as `IS NULL`. Under SQL three-valued logic `x = NULL` evaluates
  to UNKNOWN for **every** row — including rows where `x` is itself NULL — so
  the correct result is **zero rows**.
- **Decision:** **Fix.** `IS NULL` already works and is the only correct way to
  match a NULL, so no capability is lost by making `= NULL` never match.
- **Why it matters:** this is the direction that produces *extra* rows. A
  `WHERE col = <parameter>` that receives a NULL parameter silently returns the
  NULL rows instead of nothing — a wrong answer in the more dangerous direction.
- **It also reaches `IN`, found 2026-08-08 while fixing P29/P30.** `IN` is built
  on the same equality, so a NULL *in the list* matches every NULL row:
  `score IN (50, NULL)` returns ids 1,2,3,10,11,12 where DuckDB returns 1,2.
  This bites hardest in the subquery form, where the NULL is not visible in the
  query text at all — `in_subquery_then_and`'s subquery yields a NULL among its
  scores and the case DIFFERs by exactly that one row. The parse error fixed in
  P29 had been hiding it.
  The discriminating pair is `where_in_with_null_col` (AGREE): a NULL *column*
  value against a list with no NULL in it is excluded correctly. The variable
  that matters is a NULL in the list, not a NULL in the column.
- **Scope note for the fix:** P19 already says to audit `IN`/`NOT IN`/`BETWEEN`/
  `LIKE` together rather than patching one operator. This confirms it — the same
  root equality is reached through at least three surfaces.

### P19 — `NOT IN` does not exclude NULLs
- **Status:** 🟢 FIXED 2026-08-22, with P18 — one change closed both, as
  predicted: they are the same missing propagation reached through different
  operators.
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
- **The bug was live in four shipped examples, and their expectations had
  captured it.** Fixing P21 broke `boe_rate_history`, `generators`,
  `window_functions` and `window_functions_formal` — every difference in a
  window column under a `WHERE`. Each was checked against DuckDB before
  re-capturing rather than re-captured on faith. The clearest proof:
  `window_functions` had `ROW_NUMBER` values of **3 and 4 in a two-row
  partition** (`WHERE month = '2024-03'` leaves exactly two rows per region), an
  arithmetically impossible result that had been sitting in an expectation file.
  `boe_rate_history` had `LAG(rate)` returning the rate from *before* the
  filtered era on that era's first row, where the correct answer is NULL.
- **Worth remembering about the examples suite:** FORMAL expectations are
  captured from our own output, so they lock in whatever the engine did that
  day — bugs included. They detect *change*, they do not establish
  *correctness*; the parity corpus is what does that. When capturing an
  expectation for anything involving window functions, NULLs or ordering, spot-
  check it against the reference engine first.

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
- **Status:** 🟢 FIXED 2026-08-30 — 129 → 133 AGREE (+2 fixed, +2 new coverage)
- **Corpus:** `09_window.toml :: win_range_frame_with_ties`,
  `win_default_frame_ordered` (both now AGREE; `expect` dropped). New cases:
  `win_range_frame_peer_start`, `win_range_frame_partitioned_ties` (AGREE),
  `win_range_numeric_offset` (GAP — see [P33](#p33)). Baselines: the three
  explicit `ROWS` frame cases, all still AGREE.
- **Observed:** with ties in the ORDER BY key, `RANGE BETWEEN UNBOUNDED
  PRECEDING AND CURRENT ROW` must include **all peer rows** at the current
  value. At `score = 50` (two peers) DuckDB returns 160; we returned 110 — one
  peer only, i.e. ROWS behaviour.
- **The damaging half is the default frame.** With an `ORDER BY` in the window
  and no explicit frame, the SQL default is `RANGE UNBOUNDED PRECEDING AND
  CURRENT ROW`. `SUM(x) OVER (ORDER BY y)` is a far more common way to write a
  running total than any explicit frame, and it was silently wrong wherever `y`
  had duplicates. Explicit `ROWS` frames were unaffected and already correct.
- **Only detectable because the fixture has ties** — on distinct keys ROWS and
  RANGE coincide, which is why this survived until `null_edges.csv` existed.
- **Decision:** **Fix.** Implement peer-group semantics for `RANGE`, and make
  the no-frame-with-ORDER-BY default resolve to `RANGE` rather than `ROWS`.

**How it was fixed.** Both halves turned out to be one defect. The parser
*already* synthesised the correct default frame — `recursive_parser.rs` has
emitted `RANGE UNBOUNDED PRECEDING .. CURRENT ROW` for an ORDER BY without an
explicit frame since before this entry was filed. The whole divergence lived in
`window_context.rs::get_frame_rows`, whose `FrameUnit::Range` arm carried the
comment *"not yet fully implemented — for now, treat like ROWS"* and duplicated
the ROWS arm verbatim. So the second half of the decision above needed no work,
and fixing the first half closed both corpus cases at once.

`OrderedPartition` now carries `peer_bounds`, computed in one linear pass over
the already-sorted rows, so a partition of all-equal keys costs no more than one
of all-distinct keys. `CURRENT ROW` resolves to the **first** row of the peer
group as a start bound and the **last** as an end bound — the asymmetry is the
entire mechanism, and `win_range_frame_peer_start` exists because the two
original cases only exercised the end bound.

Sorting and peer detection must agree exactly or frames land mid-group, so both
now route through one `compare_by_sort_cols`; peers are precisely the rows it
calls `Equal`. A pleasant consequence: with no `ORDER BY` every row is a peer of
every other, so a bare `RANGE` frame spans the partition, which is what the
standard specifies, without a special case.

**A captured expectation had frozen the bug — the third instance of this
pattern.** `examples/expectations/window_functions.json` stored `LAST_VALUE(x)
OVER (PARTITION BY region ORDER BY month)` returning each row's *own* amount,
which is the ROWS answer; the fix broke that "passing" test. DuckDB agrees with
the new output on all 24 rows, so the capture was wrong, not the fix, and it was
re-captured. The suite's other 22 failures are pre-existing smoke-test noise —
missing fixtures and unreachable URLs — and only FORMAL mismatches fail the job.

This is the same shape as the P29/P30 note above ("two had *passing* [unit tests]
asserting the broken behaviour"), and worth stating as a rule: **when a fix
breaks a golden/captured test, establish which side is right against the
reference engine before touching either.** Re-capturing is the correct move only
once the reference has confirmed the new output; done reflexively it would have
silently re-frozen the defect. Both `examples/window_functions.sql`'s comment and
a new corpus case (`win_last_value_default_frame`) now record the real semantics,
so the next person meets the rule rather than the artefact.

**A lesson worth generalising: the entry's own prescription was half stale.**
"Make the default resolve to RANGE" described a defect that had already been
fixed elsewhere, and following it literally would have meant editing a parser
that was already right. Re-read the *code* each entry points at before scoping
the work — a finding records what was true when it was filed, and the codebase
moves underneath it. This is the P28 lesson ("a finding inherits its probe's
blind spot") in a different key: there the write-up was too narrow, here it was
out of date.

### P33 — A `RANGE` frame with a numeric offset is rejected
- **Status:** 🔴 OPEN — hard error, deliberate, low urgency
- **Corpus:** `09_window.toml :: win_range_numeric_offset` (GAP).
- **Observed:** `RANGE BETWEEN 1 PRECEDING AND CURRENT ROW` → "RANGE frames with
  a numeric offset are not supported." DuckDB evaluates it.
- **Created by the [P24](#p24) fix, deliberately.** A numeric offset under RANGE
  is defined on ORDER BY *values* — "every row whose key is within 1 of mine" —
  not on positions, and needs single-key, numeric/temporal arithmetic that peer
  groups do not provide. Before P24 this form silently returned the ROWS answer.
  Rejecting it converts a silent wrong answer into a visible error, which is a
  strict improvement and the same trade the P13 stage-1 work made.
- **Decision:** **Fix eventually.** Self-contained and well-specified; wants the
  ORDER BY key restricted to one numeric or temporal column, then a value-window
  scan. Ranks below any silent finding, being a hard error.

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

### P29 — A boolean operator after `IN (...)` is not parsed
- **Status:** 🟢 FIXED 2026-08-08 — with [P30](#p30); they were one bug
- **Corpus:** `02_where.toml :: in_list_then_and` (AGREE), `in_subquery_then_and`
  (now DIFFER on [P18](#p18), see there).
- **Observed:** `WHERE score IN (50, 70) AND team = 'alpha'` does not parse the
  `AND ...`. Until P13 stage 1 the remainder was **silently discarded**, so the
  query returned 4 rows (the `IN` alone) instead of 2, with no error. Same for
  `OR`, and for the `IN (subquery)` form.
- **Specific to `IN`.** `AND` is fine everywhere else — plain comparisons,
  three-way chains, after `LIKE`, after `BETWEEN` — all verified. `NOT IN`
  followed by `AND` also parses. It is the `IN` predicate that fails to hand
  control back to the boolean-expression parser.
- **Found:** 2026-08-02, by P13 stage 1 rejecting what it had been swallowing.
  Surfaced through `tests/python_tests/test_subqueries.py`, whose two affected
  tests had been passing while the filter they were testing was ignored — one
  asserted only `count >= 0`, which is true whether or not the `AND` applies.
- **Decision:** **Fix**, with [P30](#p30) — the two are the same area and a fix
  should address both operand orders together.
- **Fixed 2026-08-08**, branch `fix/p29-p30-in-precedence`. See P30 below for the
  root cause; the "specific to `IN`" observation above was the clue that led to
  it, since `NOT IN` was already handled at the correct precedence level.

### P30 — `<cond> AND <col> IN (list)` returns zero rows
- **Status:** 🟢 FIXED 2026-08-08 — same root cause as [P29](#p29)
- **Corpus:** `02_where.toml :: and_then_in_list` (AGREE). Added with the fix:
  `in_list_then_or`, `or_then_in_list`, `in_list_between_two_conditions` (AGREE).
- **Regression test:** `tests/in_predicate_precedence_tests.rs` — six cases,
  four of which fail against the unfixed parser (verified by stashing the fix);
  the other two are controls that must pass either way.
- **Observed:** with the operands the other way round from P29 the query
  *parses* — and returns nothing:

  ```sql
  SELECT id, team, score FROM null_edges WHERE team = 'alpha' AND score IN (50, 70)
  --  ours: 0 rows        DuckDB: (1, alpha, 50), (2, alpha, 50)
  ```

- ~~**Distinct from P29.** That one is a parse gap; this is an *evaluation* bug.~~
  **Wrong — they are the same bug.** Recorded because the misdiagnosis is
  instructive: "it parses and returns wrong rows" was read as an evaluation
  fault, but a mis-parse can produce a perfectly well-formed AST that *means*
  something else. `--query-plan` settled it in one look and should have been the
  first move.
- **Root cause.** `IN` was applied at the top of `parse_expression`, *after* the
  whole OR/AND hierarchy had been parsed:

  ```rust
  let mut left = self.parse_logical_or()?;
  left = parse_in_operator(self, left)?;   // outside the hierarchy
  ```

  So `WHERE team = 'alpha' AND score IN (50, 70)` parsed as
  `InList { expr: (team = 'alpha' AND score), values: [50, 70] }` — "is this
  boolean one of 50 or 70?" — false for every row, hence zero rows. Reverse the
  operands and the same misplacement instead leaves `AND team = 'alpha'`
  unconsumed, which is P29. The operand order only decided whether the mis-parse
  surfaced as a wrong answer or as leftover tokens.
- **Fix.** Move the `IN` branch into `parse_comparison`, next to the `NOT IN`
  branch that was already there and already correct, and drop the top-level call.
  That `NOT IN` composed properly while `IN` did not was the diagnostic tell, and
  it meant the correct implementation was sitting forty lines above the defect.
- **Confirmed pre-existing**, reproduced from a clean build of `main`.
- **Decision:** **Fix**, with P29. Done.

### P31 — `--limit` returns zero rows for any query with a `SELECT` list
- **Status:** 🟢 FIXED (2026-08-16) — found by the [P28](#p28) sweep
- **Corpus:** none possible — `--limit` is a CLI flag, not SQL. Pinned by
  `tests/python_tests/test_temp_table_staging.py`.
- **Observed:**

  ```
  sql-cli data/null_edges.csv -q "SELECT id, score FROM null_edges" --limit 3 -o csv
  id,team,score,label,bonus,partner_id        <-- the source's columns
                                              <-- and no rows at all
  ```

  `SELECT *` was fine, which is exactly why this survived: the flag works on the
  shape people reach for when trying it out, and fails on every query that names
  its columns.
- **Root cause.** `non_interactive::limit_results` built the limited table from
  `dataview.source().columns` — **all** source columns — but filled it with
  `dataview.get_row(i)`, which returns only the **projected** values. Every row
  then failed the column-count check inside `add_row`, whose `Result` was
  discarded with `let _ =`. Hence zero rows under the wrong headers, reported as
  a successful query.
- **Fix.** Delete the hand-rolled copy loop and go through
  `materialize_view` like the other three consumers, over the view narrowed by a
  new `DataView::with_max_rows` — which takes the *tighter* of the SQL `LIMIT`
  and the CLI flag and leaves any `OFFSET` alone, preserving the old
  `row_count().min(limit)` intent.
- **Same root cause as P28, different symptom.** Both are "a `DataView` was
  turned back into a `DataTable` by reaching past the view". P28 kept the
  source's rows; this kept the source's columns. That is the argument for one
  materialization helper rather than a copy loop per call site — and it is what
  the P28 entry's "check whether any other consumer takes the same route" note
  was for. **The note worked; make that sweep routine.**

### P32 — `NOT LIKE` does not parse
- **Status:** 🔴 OPEN — hard error, low urgency
- **Corpus:** `02_where.toml :: not_like_pattern` (GAP)
- **Observed:** `WHERE label NOT LIKE 'a%'` fails to parse with
  `Expected IN after NOT` — `NOT` accepts only `IN` after it. `NOT (x LIKE ..)`
  is the working spelling.
- **Found 2026-08-22 while verifying the P18/P19 fix against DuckDB**, and
  **pre-existing** — confirmed against `main`, unrelated to the three-valued
  logic work. Pinned rather than fixed in that change, per the standing rule
  that a divergence found while fixing something else gets its own entry.
- **Visible, so it ranks below any silent finding.** It is a parse error, not a
  wrong answer: nobody gets bad data from it.
- **The evaluator half is already done.** DuckDB returns ids 1,3,4,7,8,9 for the
  corpus query — note it *excludes* the NULL-label rows, which is the same
  UNKNOWN-under-`NOT` rule as P19. Our `LIKE` already returns UNKNOWN for a NULL
  operand as of the P18/P19 fix, so closing this is a parser change alone, and
  the corpus case doubles as proof that `NOT LIKE` inherits the NULL semantics
  rather than growing its own.

### P34 — `ORDER BY "col.with.dot"` fails to resolve the column
- **Status:** 🟢 FIXED 2026-08-30
- **Observed:** with a CSV whose header carries dotted names (`data/countries.csv`
  has `name.common`, `name.official`, `translations.ara.common`, ...),

  ```
  SELECT "name.common", region FROM countries ORDER BY region, "name.common"
  ```

  failed with `Column 'name.common' not found. Did you mean 'name.common'?` —
  the error naming the column it had just refused to find. The same query
  without the quoted column in the ORDER BY worked, and the SELECT list
  resolved `"name.common"` correctly, which is what made it look like an
  ORDER BY *parsing* problem.
- **Not parsing.** The AST is right: the parser produces
  `ColumnRef { name: "name.common", quote_style: DoubleQuotes, table_prefix: None }`,
  and a genuinely qualified `countries.region` parses to
  `name: "region", table_prefix: Some("countries")`. The bug was in resolution.
  `apply_multi_order_by_with_context` treated *any* dot in the name as a
  qualifier and looked up only the part after the last dot (`common`), which
  does not exist. The "did you mean" suggestion was computed from the *full*
  name, hence the self-contradicting message.
- **Fix:** try the literal column name first, and fall back to
  qualifier-stripping only for an **unquoted** reference. A double-quoted
  identifier is one name, dots included — that is what the quotes are *for*.
  The fallback is kept for unquoted dotted names, which older parse paths can
  still produce.
- **Regression tests:** `tests/test_multi_column_order_by.rs` —
  `test_order_by_quoted_column_containing_dot` (the bug) and
  `test_order_by_table_qualified_column_still_resolves` (the fallback the fix
  must not break).
- **Note the shape.** `resolve_column_index` (`query_engine.rs:106`) is
  documented as *the* canonical resolver "used by all SQL clauses ... to ensure
  consistent alias resolution", and it already gets this case right — literal
  name first, dotted name as a qualified lookup second. ORDER BY never called
  it and hand-rolled a worse copy. The narrow fix above is deliberate:
  converging the ORDER BY path onto the shared resolver changes behaviour for
  unquoted dotted names (qualified-name match vs. suffix strip), so it belongs
  in [`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md), not in a bug fix.

### P35 — `#n` positional column references are not supported
- **Status:** 🔴 OPEN — **decision deliberately left open: this is a DuckDB
  extension, not standard SQL.** Argue it on usefulness or mark ⚪ WON'T FIX;
  the "follow the reference engine" default does *not* apply.
- **Corpus:** none yet — file the cases with the decision.
- **Observed:** `SELECT #1, #2, #5 FROM wrapped` returns `id, team, bonus` in
  DuckDB. We error with `Column '#1' not found. Did you mean 'id'?`
- **Raised 2026-08-31**, immediately after [P16](#p16), from the question "does
  DuckDB support `WITH wrapped AS (SELECT ...) SELECT 1, 2, 5 FROM wrapped`".
  The question has two readings and only one is a gap:

  | Written | DuckDB | sql-cli |
  |---|---|---|
  | `SELECT 1, 2, 5` | three **constants**, columns named `1`, `2`, `5` | three constants, columns named `expr_1..3` — see [P36](#p36) |
  | `SELECT #1, #2, #5` | 1st, 2nd, 5th **columns** | error |

  Worth stating plainly because it is a live trap: **`SELECT 1, 2, 5` is not
  positional in either engine**, and we already agree with DuckDB on its values.
- **`#n` is not uniform across clauses — verified, do not assume it is.** In
  `ORDER BY` it resolves against the **output** list, not the source:
  `SELECT #1, #3 FROM t ORDER BY #3` errors with *"ORDER term out of range —
  should be between 1 and 2"*. That is the same rule P16 just implemented for
  bare ordinals, which is the natural place to hang the implementation.
  Range errors: `#0` is a parser error ("needs to be >= 1"); `#7` of 6 is a
  binder error ("Positional reference 7 out of range (total 6 columns)").
- **Cheaper than it looks, and the `#tmp` collision is narrow.** `#1` already
  lexes as `Identifier("#1")` through the lexer's catch-all — the same route
  `;` took before P13 stage 1 — so **no lexer change is needed**; it arrives at
  column resolution as a name that happens to start with `#`. Temp tables are
  `#` + letters and positional refs are `#` + digits, so the two are separable,
  but `Token::Identifier(id) if id.starts_with('#')`
  (`recursive_parser.rs:1575`) and the six other `starts_with('#')` sites are
  the list to check before committing.
- **Why it might be worth doing anyway:** wrapping an awkward query in a CTE and
  taking columns by position is the workflow that hurts most on wide files with
  unwieldy headers — the same pain that produced [P34](#p34) on
  `data/countries.csv`. It is a *feature* argument, not a parity one, and should
  be recorded as such either way.

### P36 — Unaliased expression columns are named `expr_N`, not by their text
- **Status:** 🔴 OPEN — cosmetic but user-visible; **size the blast radius
  before starting**
- **Corpus:** none yet — needs a deliberately *unaliased* case; see below.
- **Observed:**

  | Query | DuckDB | sql-cli |
  |---|---|---|
  | `SELECT score*2, UPPER(team) FROM t` | `(score * 2)`, `upper(team)` | `expr_1`, `expr_2` |
  | `SELECT 1, 2, 5 FROM t` | `1`, `2`, `5` | `expr_1`, `expr_2`, `expr_3` |

- **Currently invisible to the corpus, by construction.** Tier convention is to
  alias computed columns so the comparison aligns — the harness even says so in
  its own diff output (`column mismatch ... (alias computed columns to align)`).
  So this cannot be pinned by adding a case to an existing tier; it needs a case
  written specifically *not* to alias, and a note saying why it breaks the
  convention.
- **The reason to check before fixing:** column names feed the TUI, the export
  paths and the **FORMAL example expectations**, which are captured from our own
  output. This changes no values anywhere and could still churn a lot of
  captured JSON — exactly the P21 pattern, but with none of the payoff of
  finding a bug. Measure the churn first, then decide whether the naming is
  worth it.

### P37 — A window function inline in `WHERE` silently returns zero rows
- **Status:** 🟢 FIXED 2026-09-05 — branch `fix/p37-bare-value-predicate`
- **Corpus:** `09_window.toml :: window_in_where_inline` (OURS_ONLY, before and
  after — see *Why the corpus cannot gate this* below).
- **Regression test:** `bare_value_predicate_tests` in
  `src/data/recursive_where_evaluator.rs` — six cases, on the general shape
  rather than the window that exposed it.
- **Observed:** `SELECT region, amount FROM international_sales WHERE
  ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) <= 2` returned
  **0 rows** — header printed, success exit code, no error. The table has 20
  rows. DuckDB refuses the query outright: *"Binder Error: WHERE clause cannot
  contain window functions"*.
- **Why it is `OURS_ONLY` and not `GAP`:** we "succeed" where the reference
  errors, so the harness scores it as an extension. It is not one — it is an
  empty result standing in for an unimplemented feature, which is strictly worse
  than the reference's hard error. The bucket is right; the behaviour is not.
- **Found:** 2026-09-02, while triaging examples smoke-test failures.
  `examples/expander_rewriters.sql` advertises "expression lifter (window
  function in WHERE)" as a working transformation. Half of that query fails
  loudly ([P26](#p26)); this half failed silently, which is why it had never
  been noticed.

- **The root cause first recorded here was wrong, and the correction is the
  useful part of this entry.** The original diagnosis — carried over from
  [P15](#p15) — was that `ExpressionLifter` only walks the SELECT list, so an
  inline window in `WHERE` is never hoisted. It does walk `WHERE`, and it does
  hoist; `RUST_LOG=info` says so on the failing query itself:

  ```
  INFO sql_cli::query_plan::transformer_adapters: ExpressionLifter generated 1 CTE(s)
  ```

  The lifter rewrites `WHERE <window> <op> <val>` into a CTE that computes the
  whole comparison as a boolean column, leaving the outer query as
  `WHERE lifted_value`. Both halves of that were already correct. The rows
  vanished one step later, in `evaluate_expression`
  (`src/data/recursive_where_evaluator.rs`), whose catch-all arm read:

  ```rust
  _ => Ok(Trilean::False)  // Default to false for unsupported expressions
  ```

  A **bare value used as a predicate** — which is exactly what
  `WHERE lifted_value` is — fell into that arm and was FALSE for every row.

- **The defect was never window-specific.** Two probes taken before touching
  anything, both against the unmodified binary:

  | Query | Rows returned |
  |---|---|
  | `SELECT region FROM international_sales WHERE true` | **0** |
  | `WITH t AS (SELECT *, amount > 100 AS f FROM …) SELECT … FROM t WHERE f` | **0** |
  | `… WHERE f = true` (control) | correct |

  `WHERE true` returning nothing, on any table, is the whole bug in one line.
  The window function was a way of reaching it, not the thing that was broken —
  which is why the fix is in the WHERE evaluator and not in `expression_lifter`
  at all.

- **Fix:** the catch-all now evaluates the expression for its **value** and
  coerces that to a predicate, via a helper the file already had
  (`evaluate_expression_as_bool`, used for CASE branch results) and which was
  simply never wired to the top level. The two copies of the coercion table are
  now one function, `evaluate_value_as_predicate`. A raw `WindowFunction`
  reaching it **errors** rather than coercing — an unlifted window is a defect,
  not a value, and the P37 rule is that a loud failure beats a silent empty
  result. That satisfies both acceptable end states in the original decision:
  the lifted path gives end state 1, the unlifted path gives end state 2.
- **One deliberate behaviour change came with the unification:** a NULL-valued
  predicate now yields UNKNOWN rather than FALSE. The CASE path had answered
  FALSE. Under `WHERE` the two are indistinguishable — both drop the row — and
  they diverge only under `NOT`, which is precisely the [P18](#p18)/[P19](#p19)
  trap, so the two paths now agree on the answer P18/P19 settled.

- **Why the corpus cannot gate this, and what does instead.** The corpus case
  is `OURS_ONLY` *before and after*: DuckDB rejects a window function in `WHERE`
  either way, so `runner.py --check` stays green whether we return the right
  rows or none at all — and would stay green through a regression too. This is
  the first entry whose fix is structurally invisible to the harness. The
  regression protection is therefore a Rust module
  (`bare_value_predicate_tests`), asserting the general shape: `WHERE true`,
  `WHERE <bool column>`, NULL → UNKNOWN under `NOT`, composition with `AND`/`OR`,
  and a control that an unlifted window errors rather than filtering silently.
  Worth remembering when picking the next silent bug: *the corpus finds these,
  but it cannot always hold them down.*

- **Verification** (both binaries built and run side by side, since the fix
  touches an arm every `WHERE` in the engine passes through):

  | Check | Result |
  |---|---|
  | `cargo test --release` | 755 + 469 passed, 0 failed |
  | All 177 corpus cases, old vs new binary | byte-identical except `window_in_where_inline` itself (0 rows → the correct 7, matching the CTE-with-`rn` ground truth) |
  | All 153 `examples/*.sql`, old vs new | one meaningful change: `expander_rewriters.sql` goes `[]` → correct top-3-per-region, which is what that file has always claimed to demonstrate |
  | 33 formal expectation JSONs | zero churn |

  The examples sweep also turned up [P41](#p41) — unrelated to this fix, but it
  is what the remaining old-vs-new differences turned out to be.

- **[P15](#p15) is *not* closed by this.** Inline `QUALIFY` still errors
  identically; see that entry for why it needs the opposite change (transformer
  ordering) rather than the same one.
- **Related:** [P26](#p26) is the other half of the same example statement, and
  [P21](#p21) is the same "when are windows evaluated" confusion from a third
  angle.

### P38 — A scalar subquery in a CTE's `SELECT` list is never evaluated
- **Status:** 🔴 OPEN
- **Corpus:** `06_ctes_setops.toml :: scalar_subquery_in_cte_select_list` (GAP).
  Controls: `scalar_subquery_in_select_arithmetic`,
  `plain_arithmetic_in_cte_select_list` (both AGREE).
- **Observed:** `WITH z AS (SELECT region, amount - (SELECT AVG(amount) FROM
  international_sales) AS d FROM international_sales) SELECT * FROM z` →
  *"Unsupported expression type for arithmetic evaluation: ScalarSubquery { … }"*.
- **Precisely located by the controls — narrower than it first looks.** The
  first diagnosis was "scalar subqueries don't work in arithmetic". That is
  wrong. Three probes:

  | Query shape | Result |
  |---|---|
  | Top-level `SELECT amount - (SELECT AVG(…)) AS d` | ✅ works |
  | Top-level `(a - (SELECT …)) / (SELECT …)` | ✅ works |
  | Same expression inside a **CTE body** | 🚫 error |
  | `SELECT (SELECT AVG(…)) AS a` inside a CTE body (no arithmetic at all) | 🚫 error |
  | `SELECT amount * 2` inside a CTE body | ✅ works |

  So neither scalar subqueries nor CTE arithmetic is broken in general. The CTE
  SELECT-list path routes items through an evaluator whose expression match has
  **no `ScalarSubquery` arm**, while the top-level path has one. The error text
  says "arithmetic evaluation" only because that is the evaluator it lands in;
  arithmetic is not the trigger.
- **Found:** 2026-09-02, in `examples/statistical_analysis.sql` — a z-score
  block computing `(x - (SELECT AVG(x))) / (SELECT STDDEV(x))` inside a CTE.
  The statement is restored in that file marked `-- [SKIP]`; drop the directive
  when this is fixed and it becomes the end-to-end check.
- **Decision:** **Fix.** Two SELECT-list evaluation paths that disagree on which
  expression types they support is a shape problem, not just a missing arm —
  when picking this up, check against [`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md)
  whether the honest fix is to converge the two paths rather than add the arm
  twice. Adding the arm is the tactical fix if convergence is too large.

---

### P39 — Division by zero is a hard error, failing the whole statement
- **Status:** 🔴 OPEN — hard error, but with an amplifier: one degenerate row
  kills an otherwise valid result set
- **Corpus:** none yet — see *Pinning it* below.
- **Observed:** `divide_values` (`src/data/arithmetic_evaluator.rs:481-490`)
  tests the divisor for exact zero and returns `Err("Division by zero")`. The
  error propagates out of the row loop, so the **query returns no rows at all**
  rather than one bad cell.

  Found 2026-08-31 from a real example, not from the corpus:
  `examples/stock_analysis.sql` statement #7 (Example 6, rolling min/max) is

  ```sql
  (close - MIN(close) OVER (ORDER BY date ROWS 19 PRECEDING)) /
  (MAX(close) OVER (...) - MIN(close) OVER (...)) * 100
  ```

  On the first row of the filtered set the 20-row frame holds exactly one row,
  so `MAX == MIN` and the denominator is 0. The other 19 rows are fine; the
  statement fails anyway. That is the shape that makes this worth an entry —
  the *blast radius* is a whole query, and it scales with the row count, so a
  100k-row query is hostage to its single worst row.

- **DuckDB 1.5.5 (the pinned reference) does NOT return NULL for `/`.** Checked
  directly, because the assumption that it did is what prompted this entry:

  | Expression | DuckDB 1.5.5 | sql-cli |
  |---|---|---|
  | `1/0` | `inf` (typed `DOUBLE`) | error |
  | `-1/0` | `-inf` | error |
  | `0/0` | `nan` | error |
  | `1//0` (integer division) | `NULL` | n/a |
  | `10 % 0` | `NULL` | error (`Division by zero in MOD`) |

  So DuckDB splits the answer: **IEEE semantics for `/`** (it is float division,
  returning `DOUBLE`), **NULL for the integer operators `//` and `%`**. Note
  `1/0 IS NULL` is *false* there. Following the reference engine means
  reproducing that split, not a blanket NULL.

- **We already answer this question three different ways internally**, which is
  the real finding and should be settled as one decision rather than patched at
  the call site that hurts:

  | Site | Behaviour on divide-by-zero |
  |---|---|
  | `data/arithmetic_evaluator.rs:489` (`/`) | `Err("Division by zero")` |
  | `sql/functions/math.rs:223,277` (`MOD`, `QUOTIENT`) | `Err(...)` |
  | `sql/window_functions/mod.rs:734` | returns `DataValue::Null` |
  | `sql/functions/analytics.rs:237` (`PERCENT_CHANGE`) | pushes the string `"inf"` |

- **Open question for whoever picks this up: can we represent the result?**
  This is not just a matter of deleting the `is_zero` guard. `DataValue::Float`
  is an `f64` and holds `inf`/`nan` fine, but nothing downstream has been
  checked — comparison and sort ordering (`nan` is unordered), `RENDER_NUMBER`,
  the CSV/JSON writers, and the TUI. The harness's own `normalize.py` collapses
  numerics to rounded floats and treats `""` as NULL; `inf` through that path is
  unexamined. The neighbouring hard errors suggest the engine's standing habit
  is "IEEE special values are errors" — `SELECT SQRT(-1)` fails the same way —
  so this is a small semantic decision with a wide surface, and the surface is
  what to measure first.

- **Three defensible outcomes**, listed so the next session starts from the
  decision and not from scratch:
  1. **Follow DuckDB** — `/` yields `inf`/`-inf`/`nan`, `MOD`/`QUOTIENT` yield
     NULL. Maximum parity, largest surface (needs the representability audit).
  2. **NULL everywhere** — one rule, easy to explain, matches our coercion-first
     leaning and the window-function site that already does it. A deliberate
     divergence from the reference on `/`, so it belongs in *Deferred / won't
     fix* with a rationale if chosen.
  3. **Keep erroring, but per-row** — the value stays an error, yet one bad row
     no longer voids the statement. Addresses the amplifier without taking a
     position on semantics; likely the biggest engine change of the three.

  Whichever is chosen, apply it to all four sites above — the inconsistency is
  worse than any of the three answers.

- **Pinning it.** No corpus case yet: this was filed from an example, and the
  cases want writing against tier 03 (scalar `MOD`/`QUOTIENT`/`/` by a literal
  zero) *and* tier 09 (the degenerate-window shape above, which is the one that
  actually bit). They are deliberately not added blind — the expected bucket
  differs per case (`GAP` for the scalar `/`, since DuckDB returns a value and
  we error) and it should be recorded from a real harness run. The corpus
  environment did not build in the msys64 clone during this session (`pandas`
  has no wheel for this Python and fails to compile), so the run belongs in the
  session that takes the fix.

- **The example is parked, not fixed.** `examples/stock_analysis.sql` Example 6
  was marked `-- [SKIP]` in `2061168` so the smoke suite stays green. That is
  the right call and matches the convention `dc84409` set for P27/P13/P38 —
  keep the query in the tree as evidence rather than deleting it — but it means
  the divergence now has *no* live probe. **Un-skip it when this is closed.**
  Note the example also has a genuine bug of its own, independent of the engine
  decision: the 20-row frame is legitimately zero-width on the first row, so it
  wants a `NULLIF` on the denominator or a wider `WHERE`. Fixing the engine
  must not be mistaken for fixing the query.

---

### P40 — A table generator's arguments cannot reference columns, so there is no way to explode one row into many
- **Status:** 🔴 OPEN — hard error. **Piece 1 of 3 (the misleading message) is
  done, 2026-09-06**; pieces 2 and 3 remain
- **Corpus:** none yet — see *Pinning it*.
- **Observed:** `SPLIT` is a **table generator** (`sql/generators/string_generators.rs:12`),
  in the same family as `READ_JSON` / `RANGE`, so it can only appear in `FROM`.
  Every attempt to feed it a column fails:

  | Query | Result |
  |---|---|
  | `SELECT * FROM split('a/b/c','/')` | ✅ 3 rows, columns `value,index` |
  | `sql-cli p.csv -q "SELECT * FROM split(path,'/')"` | 🚫 *Column 'path' not found* — and `path` is a real column of `p` |
  | `WITH a AS (…) SELECT * FROM split(path,'/')` | 🚫 *Column 'path' not found* |
  | `WITH a AS (…) SELECT * FROM split(a.path,'/')` | 🚫 *Column 'a.path' not found. Table 'a' may not support qualified column names* |
  | `SELECT * FROM split((SELECT p FROM …),'/')` | 🚫 *Unsupported expression type for arithmetic evaluation: ScalarSubquery* |

- **The error message was actively misleading, and it misdirected the report
  that opened this entry — 🟢 FIXED 2026-09-06.** "Table 'a' may not support
  qualified column names" was the generic fallback, written out at *three* call
  sites. Read literally it points at CTE scoping and qualified-name resolution —
  and that is exactly the conclusion it produced ("maybe table doesn't support
  qualified columns"). Neither is involved.

  **It then cost a second afternoon, 2026-09-06**, on a query that had nothing to
  do with generators: `WITH a AS (…) SELECT SPLIT_PART(a.path,'_',1)` with no
  `FROM a`. That is simply an out-of-scope reference — DuckDB says *Referenced
  table "a" not found!* — but the message sent the reader looking at qualified-name
  support again. Two people-hours to one sentence of wrong explanation is the
  argument for taking messages seriously as a class.

  The worst of the three sites was the SELECT-list one: it branched on whether
  *any* column carried a `qualified_name` and, if none did, blamed qualification.
  For a single table or a CTE no column is qualified, so the heuristic fired on
  the common case and was wrong every time.

  Now one implementation, `data::column_resolution_error`, which separates the
  two failures that were sharing a message:

  | Situation | Message |
  |---|---|
  | prefix names nothing in scope | *Unknown table or alias 'a' in 'a.path'. The query selects from 'DUAL'. A CTE has to be named in a FROM clause before its columns can be referenced.* |
  | prefix in scope, no such column | *Column 'nope' not found in 'a'. Available columns: path, id* |

  The in-scope test is deliberately generous (table name, resolved alias, or any
  column's qualified-name prefix): a false "in scope" costs only a less pointed
  message, whereas a false "unknown table" would reintroduce the confident wrong
  explanation this replaced. Five unit tests, one of which asserts the old
  wording cannot return.

- **Root cause — two defects stacked, and only the second is a feature request:**
  1. **The args are evaluated against DUAL.** `statement_executor.rs:124-148`
     picks the source table from `from_source` / `from_table` only. There is no
     `from_function` case, so a generator query falls through to
     `Arc::new(DataTable::dual())`. `query_engine.rs:1300-1314` then builds an
     `ArithmeticEvaluator` on *that* table. DUAL has no columns, so **no** column
     reference can ever resolve there — the CTE is not consulted, and neither is
     the loaded CSV. That is a plain wiring bug, not a design limit.
  2. **Even with args resolved, `generate()` is called once**, at
     `dummy_row = 0`. Exploding a column into rows needs one invocation *per
     input row* with the results concatenated — a lateral/correlated table
     function. The engine has no such concept, and no `UNNEST`.

- **DuckDB 1.5.5 (the pinned reference), verified directly** — including the
  shape that was actually reported, which turns out to be invalid there too:

  | Query | DuckDB 1.5.5 |
  |---|---|
  | `SELECT path, unnest(str_split(path,'/')) FROM a` | ✅ 5 rows from 2 — the idiomatic form |
  | `SELECT * FROM a, unnest(str_split(a.path,'/')) u(part)` | ✅ lateral join |
  | `WITH a AS (…) SELECT * FROM unnest(str_split(a.path,'/'))` | 🚫 *Binder Error: Referenced table "a" not found!* |
  | `SELECT * FROM a, generate_series(1, a.n)` | ✅ correlated table function |
  | `SELECT * FROM split('a/b','/')` | 🚫 *Catalog Error: Table Function with name split does not exist* |

  Two things follow. **The reported query is not the one to make work** — `a` is
  not in its `FROM`, and DuckDB rejects it for that reason; the target shapes are
  rows 1 and 2. And **our `SPLIT` generator is an extension**, not a parity
  obligation: DuckDB's `str_split` is scalar and returns a list. Ours already
  emits `value,index`, i.e. `WITH ORDINALITY` for free, which is worth keeping.

- **What works today:** `SPLIT_PART(path,'/',n)` — scalar, resolves columns
  normally, fine for fixed positions. There is no way to handle
  arbitrary-depth paths.

- **Found:** 2026-09-04, from real use rather than the corpus — a TeamCity API
  response piped into `READ_JSON('-', …)`, wrapped in a CTE to extract an
  artifact path, then wanting that path split into its segments. The CTE and the
  qualified `a.path` reference both work in `SELECT`; only the generator
  argument fails.

- **Decision: fix, in three independently shippable pieces**, smallest first:
  1. ~~**The message.**~~ ✅ **Done 2026-09-06.** No semantics, no risk, and it
     was the part that wasted other people's time — twice, as it turned out.
     Note what it does *not* do: a generator argument that is not a constant
     still reports "unknown table or alias", which is accurate for the reported
     shape but will read oddly once piece 2 lands. Revisit the wording then.
  2. **Resolve generator arguments against the real source table and CTE
     context** instead of DUAL. Makes single-row and constant-expression
     arguments honest, and makes the remaining failure an accurate "this needs a
     lateral join" rather than a bogus "column not found".
  3. **Row-wise explode.** Prefer `UNNEST` in the `SELECT` list (reference row
     1) over lateral table functions in `FROM` (row 2): it is the idiomatic
     form, it is what people will reach for, and it avoids touching
     `TableSource` — which the `FROM` form cannot, see below.

- **Related — [R1](ENGINE_REFACTORING.md#r1) is the structural reason this is
  the way it is.** R1 already records that `TableSource` has *no table-function
  variant*, so `from_source` is set to `None` whenever `from_function` is
  populated and generators are stranded on the legacy path — which is precisely
  the path that has no source-table resolution. Piece 3-via-`FROM` needs the
  `TableSource` change R1 assessed as its own project (it drags in joins);
  piece 3-via-`UNNEST` does not. Piece 1 and piece 2 are independent of R1.
  The `ScalarSubquery` row in the first table is
  [R7](ENGINE_REFACTORING.md#r7) showing through, not a separate finding.

- **Pinning it.** No corpus case yet, deliberately — the useful cases are for
  the *target* shapes (`UNNEST` in a `SELECT` list; a lateral table function),
  both of which we do not parse, and the bucket should be recorded from a real
  harness run rather than guessed. They want a home in tier 03 (`03_functions.toml`)
  plus one in tier 06 (`06_ctes_setops.toml`) for the shape that was actually hit. Expect `GAP`.

### P41 — `MODE` picks a tie-break winner at random, run to run
- **Status:** 🟢 FIXED 2026-09-06 — branch `fix/p41-mode-tiebreak`. 177 → **181
  cases, 152 → 156 AGREE** (all four new, no bucket changes elsewhere)
- **Corpus:** `10_aggregate_nulls.toml :: mode_two_way_tie`,
  `mode_tie_first_seen_not_smallest`, `mode_all_null`, `mode_grouped` — all AGREE.
- **Regression tests:** `mode_tie_break_tests` in
  `src/sql/aggregate_functions/mod.rs` (the live path, driven through the
  registry) and in `src/sql/aggregates/mod.rs` (the shadowed one) — six cases each.
- **Observed:** `MODE` tallied into a `HashMap` and took the highest count with
  **no tie-break rule**. When two or more values tied, the winner was whichever
  the hash iteration surfaced last. Six runs of the *same binary*, same data:

  ```
  $ for i in 1 2 3 4 5 6; do sql-cli -q "WITH r AS (SELECT value % 2 AS pn
      FROM RANGE(1,50)) SELECT MODE(pn) FROM r" -o csv; done
  0  1  1  0  0  1
  ```

- **Found:** 2026-09-05, while verifying the [P37](#p37) fix. It surfaced as
  three `examples/*.sql` files differing between the pre-fix and post-fix
  binaries; the queries involved have no `WHERE` clause at all, which is what
  prompted checking the same binary twice instead of blaming the change.

- **The reference does specify a rule, and it is not the obvious one.** This
  entry originally proposed *smallest value wins* — total, cheap, and wrong.
  DuckDB breaks ties by **first occurrence in the input**
  (`extension/core_functions/aggregate/holistic/mode.cpp`, which tracks a
  `first_row` per distinct value and compares
  `count > best.count || (count == best.count && first_row < best.first_row)`).
  Probed directly to confirm: `(0,0,1,1)` → `0` but `(1,1,0,0)` → `1`;
  `('b','b','a','a')` → `'b'`; `(5,3,9,1)` → `5`, not `1`. Stable across runs
  including 1M-row parallel aggregation. NULLs ignored; all-NULL or empty → NULL,
  which we already matched.

  So the rule is **earliest-seen wins**, per
  [*follow the reference engine*](#where-the-standard-leaves-a-choice-open-follow-the-reference-engine).
  Worth stating the trade-off plainly: earliest-seen is deterministic *given a
  row order*, not a total order over values. That is weaker than "smallest wins"
  would have been, and it is what the reference does.

- **The filed location was the wrong one — a shadowed implementation.** This
  entry named `ModeState` in `src/sql/aggregates/mod.rs`. Fixing it there changed
  nothing: the repro was still `1 1 0 0 0 1 0 0` afterwards. There are **two**
  aggregate registries, and `ArithmeticEvaluator` checks the newer one *first*
  (`arithmetic_evaluator.rs:637`, `:769`), so the live `MODE` is
  `CollectorState`/`CollectorFunction::Mode` in
  `src/sql/aggregate_functions/mod.rs` — a completely separate tally, over
  `f64::to_bits` keys, with the same missing tie-break. Both are fixed here; the
  duplication itself is filed as [R12](ENGINE_REFACTORING.md#r12).

  This is the [P28](#p28) lesson recurring in a new form. There the write-up
  inherited its probe's blind spot; here it inherited a *grep's* — the struct
  named `ModeState` is not the code that runs `MODE`. **Confirm a fix site by
  changing it and watching the symptom move**, not by name.

- **Payoff, as predicted by the entry:** both blocked example files are now
  deterministic and have been promoted to FORMAL —
  `examples/expectations/stats_examples.json` and
  `statistical_analysis.json`, each verified stable over five consecutive runs.

- **Spun off:** [P42](#p42) (`MODE` is numeric-only, which is why the corpus
  cases here are all numeric — the string form is where the divergence is easiest
  to see and is exactly what we cannot yet express) and [P43](#p43) (`RANGE`
  endpoint semantics, found because the repro query above is a 25/25 tie for us
  and would not be for DuckDB).

---

### P42 — `MODE` rejects non-numeric values, and returns a float for integers
- **Status:** 🔴 OPEN — hard error, loudly signposted
- **Corpus:** none yet. Deliberately: adding one means pinning a `GAP`, and the
  decision below should be taken first.
- **Observed:** `SELECT MODE(region) FROM international_sales` →
  *"MODE currently only supports numeric values"*
  (`aggregate_functions/mod.rs`, `CollectorState::accumulate`). DuckDB returns
  `'Europe'`. The live implementation collects into a `Vec<f64>`, so any
  non-numeric input is an error by construction, and the result is always
  `DataValue::Float` — `MODE` over an integer column returns `50.0`, not `50`.
- **Worse in the grouped form.** `SELECT region, MODE(product) FROM
  international_sales GROUP BY region` does not error — it returns one row per
  region with an **empty** `MODE` column. So the same defect is loud in one shape
  and silent in the other, which is the [R3](ENGINE_REFACTORING.md#r3) pattern.
- **Decision:** **Fix**, and note that the shadowed `ModeState` in
  `src/sql/aggregates/mod.rs` already does the right thing — it keys on a string
  rendering and returns the *original* `DataValue`, so it handles strings, dates
  and booleans and preserves type. The fix is most likely to make the live path
  do what the dead one already does, which makes this a natural companion to
  [R12](ENGINE_REFACTORING.md#r12) rather than an independent piece of work.
- **Why it matters beyond the error message:** MODE of a *category* is the common
  use ("most frequent product"), far more so than MODE of a measure. And it
  costs the corpus its best test: the discriminating tie-break case is much
  clearer on strings (`'delta'` vs `'alpha'`) than on the integer column
  `mode_tie_first_seen_not_smallest` had to fall back to.

---

### P43 — `RANGE(a, b)` is inclusive of `b`; DuckDB's is half-open
- **Status:** 🔴 OPEN — silent, and it changes row counts
- **Corpus:** none yet — `RANGE` appears nowhere in the corpus, which is how
  this went unnoticed.
- **Observed:** `SELECT COUNT(*), MIN(value), MAX(value) FROM RANGE(1,50)` gives
  us `50, 1, 50`. DuckDB's `range(1,50)` yields **49** rows, `1`–`49`: the stop
  bound is exclusive, matching Python's `range` and DuckDB's own documentation.
  `generate_series` is the inclusive spelling there.
- **Found:** 2026-09-06, while building the [P41](#p41) corpus cases. The P41
  repro relies on `RANGE(1,50)` being a perfect 25/25 even/odd split — true for
  us, false for the reference (25 odd, 24 even), so the query could not be
  lifted into the corpus as written.
- **Decision:** **Not yet taken.** Unlike most entries here this is not obviously
  a fix: changing it is a breaking change for every existing query and example
  that uses `RANGE`, and the off-by-one lands silently in each. Sequence it as a
  decision — match the reference and sweep the examples, or diverge deliberately
  and record it under *Deferred* — not as a quiet correction. Adding
  `GENERATE_SERIES` as the inclusive spelling is the move that makes matching
  the reference survivable.
- **Related:** the same class as [P36](#p36) and [P39](#p39) — a defensible local
  choice that only becomes a problem because it is undocumented and unpinned.

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
