# Engine Refactoring — Book of Work

The durable decision log for **structural** debt in the query engine, the
counterpart to [`SQL_PARITY.md`](SQL_PARITY.md).

The two tracks answer different questions:

| | Question | Driven by |
|---|---|---|
| `SQL_PARITY.md` (P-numbers) | *Do we return the right answer?* | Differential testing vs DuckDB |
| **This file (R-numbers)** | *Can we keep changing the engine safely?* | Findings from doing the work |

A parity gap is a wrong result. An **R-finding** is a place where the engine's
shape makes the *next* change disproportionately expensive or risky — duplicated
branch logic, two representations of one fact, a test fixture that can't catch
the bug it's meant to guard.

## Why this file exists

The engine grew organically and on demand, which is why it does as much as it
does. The cost is that some structure was never designed, only accreted — and
that cost is now being paid at the point where we want to go further (richer
function support, deeper nesting, correlated scoping).

The purpose of this log is explicitly **not** to justify a rewrite. It is to
make the debt legible so it can be paid **incrementally, in the course of
feature work**, and so we can tell the difference between "this is awkward" and
"this is blocking".

## Principles

1. **No wholesale rewrites.** No multi-month refactor projects. Every entry here
   must be reducible to steps that ship independently.
2. **Groundwork is justified by the feature it unblocks**, and the link is
   recorded. Refactoring for its own sake doesn't earn a slot.
3. **Each step must be independently verifiable** — green tests, parity contract
   intact, and where possible a test that fails without the change.
4. **Prefer compiler-enforced invariants over conventions.** A rule the compiler
   checks survives; a rule in a comment does not.
5. **Additive first.** Introduce the better mechanism, migrate call sites in
   separate commits, delete the old path last.

## Status legend

| Status | Meaning |
|---|---|
| 🔴 OPEN | confirmed weakness, not yet addressed |
| 🟡 IN PROGRESS | mechanism landed, migration outstanding |
| 🟢 DONE | resolved |
| ⚪ ACCEPTED | known, deliberately not changing — rationale recorded |

---

## Open findings

### R1 — Two representations of the FROM clause
- **Status:** 🟡 IN PROGRESS — sync helpers landed (PR #30); full migration deferred
- **Where:** `ast.rs` `SelectStatement.from_source` vs the deprecated
  `from_table` / `from_subquery` / `from_function` / `from_alias`
- **Observed:** The parser populates **both**; the executor
  (`query_engine.rs:1217`) reads `from_source` first and falls back to the
  legacy fields. Four transformers rewrote only one of the two, leaving the
  stale copy as the one that actually executed. `from_source` is a
  *second-class projection* derived from the legacy fields
  (`recursive_parser.rs:1119-1136`), not the source of truth.
- **Impact:** ~56 live use sites across 19 files. Any code reasoning about the
  FROM clause has to know which representation is authoritative in that context
  — and the answer differs by call site.
- **Done so far:** Added `impl SelectStatement` (there was none) with
  `set_from_table` / `set_from_subquery` / `map_from_subquery`, and routed the
  four desync sites through them, so the representations can only move together.
- **Deferred, deliberately:** The actual migration off the legacy fields is
  **blocked on AST work, not mechanical churn**:
  - `TableSource` has **no table-function variant** — `recursive_parser.rs:1132`
    explicitly sets `from_source = None` when `from_function` is set, so a
    `from_source`-only reader silently loses `READ_CSV()`, `RANGE()`, etc.
  - `TableSource::Table(String)` **cannot carry an alias**; `FROM trades t`
    stores `t` only in `from_alias`.

  Fixing both breaks `TableSource`, which `JoinClause.table` also uses — so
  joins get dragged in. Assessed as **larger than P3 itself**. Revisit as its
  own project.
- **Related:** [P3](SQL_PARITY.md) needs `from_source` to be authoritative for
  the scope work.
- **Related:** [P40](SQL_PARITY.md#p40) is this entry's first field report.
  The missing table-function variant is *why* generators stay on the legacy
  path — and that path is the one with no source-table resolution, so a
  generator's arguments are evaluated against DUAL and no column reference
  can resolve. Note P40's first two fixes do **not** need the `TableSource`
  change; only lateral table functions in `FROM` would drag it in.

### R2 — Branch logic over `SqlExpression` is copy-pasted everywhere
- **Status:** 🟡 IN PROGRESS — helpers landed (#31), crossing forms made primitive
  (#35), 5 of 11 transformers migrated; 6 outstanding
- **Where:** `src/query_plan/*.rs`, `src/data/*.rs`, `src/analysis/*.rs`
- **Observed:** No traversal abstraction existed. **446
  `SqlExpression::<Variant>` patterns across 40 files**, every consumer
  hand-rolling its own match over 24 variants. `ilike_to_like_transformer` and
  `cte_hoister` have `BinaryOp` / `FunctionCall` / `CaseExpression` arms that are
  byte-for-byte identical apart from the method name — each wrapping *one* real
  rule in ~50 lines of boilerplate.
- **Impact:** This is the finding that makes everything else expensive. Adding a
  variant (e.g. `Exists` for P3) means auditing every copy by hand. It is also
  the direct blocker on richer expression features generally — each new
  expression form multiplies across ~40 files.
- **Done so far:** `src/sql/parser/walk.rs` — `map_children` (owned rewrite) and
  `visit_children` / `visit_all` (borrowing collector), exhaustive, no catch-all.
  Three design rules worth preserving:
  - **Direct children only, callers drive recursion** — this is what collapses a
    transformer to its one real rule plus a delegate.
  - **Scope boundary = walker boundary** — subquery statements are not descended
    into. Auto-descending would break the alias expanders, which get outer-alias
    scoping right today only *by omission*.
  - **The crossing forms are the primitives** — `map_children_crossing` /
    `visit_children_crossing` take a second closure for the nested statement,
    and the opaque forms are those with an identity/no-op handler. This keeps
    the set of subquery-bearing variants written down *once*. Transformers that
    must cross (CTE hoisting, INTO removal, `ILIKE`) previously hand-listed
    those five variants each, which compiles clean — and silently stops
    crossing — the day an `Exists` is added. Both crossing closures take an
    explicit `ctx` parameter: a `&mut self` transformer cannot hand out two
    closures that each capture `self` mutably.
- **Migrated so far:** `cte_hoister`, `into_clause_remover`,
  `ilike_to_like_transformer` — i.e. exactly the three that cross the scope
  boundary; all three now delegate and none names a subquery variant — plus
  `having_alias_transformer`, which closed [P9](SQL_PARITY.md).
- **The migration is paying for itself.** Every transformer moved onto `walk` so
  far has either fixed a live silent bug or been proven not to have one. P9 is
  the clearest case: 74 lines of hand-rolled match became 24, and three corpus
  cases went DIFFER → AGREE with no other change. Use it as the template —
  *handle the one real rule, return early, delegate everything else*.
- **Next, in priority order.** Counts are `SqlExpression::` patterns remaining
  in each file, as a rough size guide:

  | Transformer | Patterns | Note |
  |---|---|---|
  | ~~`having_alias_transformer`~~ | ~~30~~ | ✅ Done 2026-07-19 — closed [P9](SQL_PARITY.md). |
  | ~~`where_alias_expander`~~ | ~~60~~ | ✅ Done 2026-07-25 — closed [P11](SQL_PARITY.md); ~230 hand-rolled lines retired. |
  | `expression_lifter` | 35 | **Now has a parity case behind it: [P15](SQL_PARITY.md).** It lifts window functions from the SELECT list only, so an inline window fn in `QUALIFY` is never hoisted and dies in the WHERE evaluator. Migrate *and* extend it to the QUALIFY clause. |
  | `group_by_alias_expander` | 34 | Same scoping caveat as the two done. |
  | `in_operator_lifter` | 21 | Related to [P11](SQL_PARITY.md). |
  | `order_by_alias_transformer` | 18 | Same scoping caveat. Tier 08 now exercises it. |
  | `correlated_subquery_analyzer` | 12 | Touches P3; likely wants the crossing form. |
  | `pivot_expander` / `qualify_to_where_transformer` | 10 / 9 | Smallest; good warm-ups. Note P15 is *not* fixed here despite the name — see `expression_lifter`. |

  One commit each. **`WindowSpec::order_by` is now descended into**, so every
  migration needs a per-transformer behaviour check — a blanket "pure refactor"
  claim is not available, and two of the bugs #33 fixed were exactly this.

### R3 — Catch-all match arms silently swallow new variants
- **Status:** 🔴 OPEN
- **Observed:** Almost every hand-rolled walker ends in `_ => {}` or
  `other => other`. Verified by temporarily adding an `Exists` variant to
  `SqlExpression`: only **five** match sites in the entire codebase failed to
  compile —

  ```
  src/sql/parser/formatter.rs:139
  src/sql/parser/formatter.rs:668
  src/sql/recursive_parser.rs:569
  src/sql/parser/walk.rs:108    (added by R2 — map_children_crossing)
  src/sql/parser/walk.rs:280    (added by R2 — visit_children_crossing)
  ```

  Every other consumer compiled clean and would have silently ignored it.
- **Impact:** A new expression variant is *silently* dropped by the formatter,
  the lifters, and the aggregate detector. The failure mode is a wrong answer or
  a no-op, not a build error.
- **Confirmed live bugs from this pattern.** These are not hypothetical — each
  was reproduced against the CLI and is now pinned by a corpus case:

  | Parity entry | Symptom | Severity |
  |---|---|---|
  | [P9](SQL_PARITY.md) | `HAVING` with an aggregate inside `BETWEEN` / `IN` / `CASE` returns wrong rows, **silently, in both directions** | **wrong results, no error** |
  | [P11](SQL_PARITY.md) | A `SELECT` alias on the LHS of an `IN` subquery → `Column not found` | hard error |
  | [P15](SQL_PARITY.md) | An inline window function in `QUALIFY` is never lifted (the lifter walks the SELECT list only) → `Expected column name, got: WindowFunction` | hard error |
  | *(fixed, PR #33)* | `ILIKE` inside `OVER (ORDER BY ...)` left unrewritten, reaching the executor as an unknown operator | hard error |
  | *(fixed, PR #33)* | `INTO` inside `(a, b) IN (SELECT ...)` never removed | reaches executor |
  | *(fixed 2026-09-13)* [P46](SQL_PARITY.md#p46) | The WHERE evaluator's literal reader ended `_ => Ok(Null)`, so every column, `CASE` or expression on the right of a comparison, in an `IN` list or as a `BETWEEN` bound read as NULL — **zero rows, no error**. Not a walker, but the same catch-all shape one layer down: the arm that should have been a compile-time gap was a value | **wrong results, no error** |

  P9 is the one that matters most: `HAVING COUNT(*) BETWEEN 1 AND 2` returned
  4 rows where DuckDB returns 1, with no error. A catch-all turned a missing
  match arm into a wrong answer.

- **The corpus had no `HAVING` coverage at all** before 2026-07-18, which is why
  P9 survived. Absence of a test bucket is itself a finding: when migrating a
  transformer, check whether the clause it serves is represented in
  `tests/comparison/corpus/`.
- **Decision:** Fix by attrition through the R2 migration; each transformer that
  moves onto `walk` loses its catch-all. Write the failing corpus case **before**
  migrating the transformer, so the fix is demonstrated rather than assumed.

### R4 — Transformer test fixtures use ASTs the parser never produces
- **Status:** 🔴 OPEN
- **Observed:** Hand-built `SelectStatement` fixtures in `cte_hoister`,
  `into_clause_remover` and others set `from_source: None` while setting
  `from_table: Some(...)`. The parser always populates both.
- **Impact:** **This is why R1's desync survived.** The transformers were only
  ever exercised on inputs that couldn't exhibit the bug. A green suite meant
  nothing here.
- **Decision:** Prefer parsing real SQL in transformer tests over hand-building
  ASTs. Where a fixture is genuinely needed, derive it from a parse rather than
  writing the struct literal. The two regression tests added in PR #30 follow
  this pattern.
- **Note:** A `SelectStatement::from_sql(&str)` test helper would make the right
  thing the easy thing; the struct literals are ~25 lines each, which is most of
  why people reach for `Default`-ish shortcuts.

### R5 — Dead code accumulates undetected
- **Status:** 🔴 OPEN
- **Observed:** `column_dependency_lifter.rs` sat in the tree fully dead —
  absent from `query_plan/mod.rs`, referenced nowhere, and no longer compiling
  against the current AST (its `SelectStatement` literal was missing six
  fields). Deleted in PR #30. `cte_hoister::hoist_from_condition` is still dead
  (confirmed pre-existing).
- **Impact:** Modest in isolation, but dead code inflates every audit — the
  deleted file alone accounted for 9 deprecated-field sites and 14
  `SqlExpression` match arms in the R2/R3 survey.
- **Decision:** Not worth a dedicated project. Cheapest fix is to stop tolerating
  the warnings: the tree currently emits ~368 clippy warnings and ~67
  `#[allow(deprecated)]` annotations, which is enough noise to hide a real
  signal. Consider a `[lints]` table in `Cargo.toml` once the count is down.

### R6 — `CorrelatedSubqueryAnalyzer` is unwired and untested
- **Status:** 🔴 OPEN
- **Where:** `src/query_plan/correlated_subquery_analyzer.rs`
- **Observed:** It already has the scope stack that P3 needs (`scope_stack:
  Vec<HashSet<String>>`, push/pop, outer-frame lookup) and a stubbed
  `Exists { negated }` variant marked *"not yet parsed, but for future"*. But it
  is not wired into the execution path, and its tests are hollow —
  `test_non_correlated_scalar_subquery` builds a statement containing **no
  subquery at all** and asserts the count is zero, so correlation detection is
  effectively untested.
- **Also:** `collect_references_from_expr` records only `table_prefix`, not
  column names, and misses several expression variants (an instance of R3).
- **Decision:** Fix as part of P3 rather than standalone — it is the natural
  detection stage. Reuse, don't rewrite: the structure is right, the wiring and
  tests are missing.

### R7 — Subqueries are substituted, not evaluated
- **Status:** 🔴 OPEN — the structural root cause behind parity [P3](SQL_PARITY.md)
- **Where:** `src/data/subquery_executor.rs`
- **Observed:** Subquery execution is a whole-statement **AST rewrite pass** that
  runs once, up front, replacing each subquery node with literal values before
  any row is touched. There is no outer-row context anywhere in the evaluation
  path. Two aggravating details:
  - The result cache keys on the AST debug string alone
    (`format!("scalar:{:?}", query)`), which is actively wrong for correlation —
    identical AST across outer rows, different required results.
  - `arithmetic_evaluator.rs:250` falls back to dropping a column's qualifier
    when it doesn't resolve, so an unresolved *outer* reference silently binds to
    an inner column instead of erroring. This turns a missing feature into a
    **silent wrong answer**.
- **Impact:** Blocks all five correlated-subquery parity cases, and more
  generally blocks any nested-scope feature.
- **Decision:** Fix via P3. Keep the substitution path for uncorrelated
  subqueries (three corpus cases pass *because* of it) and fork correlated nodes
  to per-row evaluation — not a replacement.

### R8 — A second, older WHERE stack survives alongside the engine
- **Status:** 🟡 IN PROGRESS — stage 1 done 2026-08-08; stage 2 outstanding
- **Where:** `src/sql/where_parser.rs`, `src/sql/where_ast.rs`, and (deleted)
  `src/data/where_evaluator.rs`, `src/data/where_clause_converter.rs`,
  `src/data/simple_where.rs`
- **Observed:** From when `WHERE` was not yet fully recursive, a complete
  parallel stack remained in the tree: its own AST (`WhereExpr` / `WhereValue` /
  `ComparisonOp`), its own parser, and its own evaluator with its own equality
  rules (the deleted `where_evaluator.rs:417` compared floats via
  `f64::EPSILON`). None of it shares `value_comparisons::compare_with_op`, which
  the real engine centralises on — so it is a second set of SQL semantics.
- **Impact:** Not a live wrong answer — no product path reaches it — but it is a
  standing source of false confidence, and it inflates every audit of "how many
  places implement `IN`/comparison". `CsvDataSource::query_with_options` is the
  worst of it: it parses with the *real* parser, discards that for the WHERE, and
  re-extracts the clause with `sql_lower.find(" where ")` before handing the
  substring to the legacy parser.
- **Stage 1 (done, 2026-08-08).** Deleted the three components with **zero
  callers anywhere**: `where_clause_converter.rs` (194), `where_evaluator.rs`
  (443), `simple_where.rs` (193) — 830 lines. Also removed
  `DebugWidget::generate_debug`'s WHERE-AST section and
  `parse_where_clause_ast`, the only product-side reference to the legacy
  parser. **That method is itself never called** (the live F5 view is served by
  `src/ui/debug/context.rs` via `set_content`), so this was dead code rather
  than a misleading debug view — worth recording, because it was initially
  assessed the other way round. Pure deletion: parity stayed at exactly 125
  AGREE / 159 cases, all FORMAL examples passed, `cargo test` unchanged.
- **Stage 2 (outstanding).** `where_parser.rs` (717) + `where_ast.rs` (748)
  remain, reachable only from tests: 12 tests in
  `tests/parser_regression_tests.rs` ride `CsvApiClient::query_csv` into the
  legacy path, and `tests/test_numeric_columns.rs` tests the legacy AST outright.
  Repoint those onto the real engine, then delete both files and
  `CsvDataSource`'s query path.
  **Guard rail:** those 12 tests assert expected rows produced by the *legacy*
  engine. Any query that behaves differently through the real one gets pinned as
  a P-finding plus a corpus case, not fixed in the same change — the same trap as
  the FORMAL expectations that had captured [P21](SQL_PARITY.md).
- **Decision:** Delete, don't migrate. Related to [R5](#r5) but logged separately
  because it is a coherent subsystem rather than scattered dead files.

### R9 — Turning a `DataView` back into a `DataTable` was open-coded per caller
- **Status:** 🟢 DONE (2026-08-16), alongside parity P28/P31
- **Where:** `query_engine::materialize_view`, `non_interactive::limit_results`,
  the `INTO` branch in `non_interactive::execute_script`
- **Observed:** A `DataView` holds three pieces of state the source table does
  not — the row filter, the column projection, and the LIMIT/OFFSET window — and
  four places needed to collapse one back into a table. There was one correct
  helper and **two hand-rolled copies that each dropped a different part of it**:
  the `INTO` branch took `source_arc()` (dropping all three), `limit_results`
  copied source *columns* with projected *row values* (dropping the projection,
  and then silently discarding every mismatched row because `add_row`'s `Result`
  went to `let _ =`). The one correct helper still missed LIMIT, because the
  accessor it used, `visible_row_indices()`, is documented as the *pre*-limit set
  — a name that reads like the answer and isn't.
- **Impact:** Two live silent wrong answers ([P28](SQL_PARITY.md#p28),
  [P31](SQL_PARITY.md#p31)) from one shape. Neither was a hard error: both
  returned a well-formed result of the wrong size.
- **Done:** `DataView` gained `windowed_row_indices()` (the post-limit set that
  `row_count()` actually counts) and `with_max_rows()` (tighten the window,
  keeping the tighter of two limits and preserving the offset).
  `materialize_view` is now the only implementation, and all callers go through
  it. The pre-limit accessor stays for callers that mean it, with its trap named
  in the doc comment.
- **Worth generalising:** an accessor whose name states the common case and whose
  doc states the exception will be misused. `visible_row_indices()` was the
  obvious-looking call at all four sites and the wrong one at three of them —
  the same failure mode as [R3](#r3)'s catch-all arms, one layer down.

### R10 — Predicate evaluation has no way to say UNKNOWN
- **Status:** 🟢 DONE 2026-08-22 — all three slices landed the same day;
  closes [P18](SQL_PARITY.md#p18) and [P19](SQL_PARITY.md#p19), parity 125 → 129
- **Where:** `src/data/recursive_where_evaluator.rs` (9 `Result<bool>`
  signatures, 14 match arms), two construction sites in
  `src/data/query_engine.rs`, and one semantic boundary — the
  `if result { filtered_rows.push(row_idx) }` row filter in the same file.
- **Observed:** SQL predicates are evaluated in *three-valued* logic, but the
  evaluator returns `Result<bool>`, so UNKNOWN has no representation. `NOT` is
  `Ok(!inner)` and `NOT IN` is `Ok(!in_result)`, which turns UNKNOWN into TRUE
  and admits rows that must not pass. This is the mechanism behind
  [P18](SQL_PARITY.md#p18) and [P19](SQL_PARITY.md#p19) — a **type** problem
  before it is a semantics problem, which is why patching per-operator has not
  worked and should not be attempted.
- **Slicing** (agreed 2026-08-08; only the last slice changes results):
  - **1a — done 2026-08-22.** `src/data/trilean.rs`: the `Trilean` type, the
    `AND`/`OR`/`NOT` truth tables, and `is_true()` as the *only* sanctioned
    collapse back to `bool`. 13 unit tests assert the tables cell by cell rather
    than deriving them, plus the algebraic laws the evaluator's rewrites lean on
    (double negation, De Morgan, associativity) and the one that **fails** in
    3VL — excluded middle, `x OR NOT x` is UNKNOWN when `x` is. Nothing is
    wired; the engine does not reference it yet.
  - **1b — done 2026-08-22.** All 9 signatures return `Result<Trilean>`, the
    combinators are the truth tables (`&&`/`||` → `and`/`or`, `!` → `negate`),
    and `is_true()` is applied at exactly one place: the row filter. Parity
    stayed at **exactly** 125 AGREE / 159, all 710 Rust tests green, all FORMAL
    examples green, and clippy at the same 365 warnings as before.
    **The no-op is structural, not observed:** `Trilean::Unknown` is
    constructed **zero times** in the engine, so the evaluator still yields only
    TRUE/FALSE, and `boolean_subset_matches_two_valued_logic` (1a) pins
    `Trilean` to `bool` over that subset. The parity run confirms it; the type
    is what guarantees it.
    Method: change the 9 signatures first and let `rustc` enumerate the 57
    leaves, rather than grepping for them — the compiler's list is exhaustive
    by definition and the transform was applied from its own line/column output.
    Three test files (`method_evaluation_test`, `test_indexof_space`,
    `test_trim_methods`) asserted on the `bool` and moved to
    `is_true()`/`is_false()`; `is_false()` is deliberately used for the negative
    cases rather than `!is_true()`, so that when 1c lands, a result that turns
    UNKNOWN fails the assertion instead of silently satisfying it.
  - **1c — done 2026-08-22, the only slice that changed results.** UNKNOWN is
    now produced at the leaves and the already-wired propagation does the rest.
    The whole semantic change is one helper, `compare_trilean`: if either
    operand is NULL the answer is UNKNOWN. It is applied at the comparison arm,
    inside `evaluate_in_list`, in `BETWEEN`'s two bounds, in `LIKE`, and to the
    arithmetic evaluator's NULL result. **~40 lines**, which is the payoff for
    having shipped 1a/1b separately.
    **The root cause was one line in a function nobody would have suspected:**
    `compare_values` reports `(Null, Null) => Some(Ordering::Equal)`. That is
    *correct* — `ORDER BY` needs NULLs to group together — and the WHERE
    evaluator was reusing the same answer, which is why `= NULL` behaved like
    `IS NULL`. The fix tests for NULL at the predicate layer and leaves the
    shared comparator alone; pushing it down would have broken sorting.
    No FORMAL example churn materialised — none of them exercise NULL
    predicates — so the P21 re-capture warning did not bite this time.
- **Why the split was worth it, in hindsight:** 1b was 187 changed lines with
  zero behaviour change; 1c was ~40 lines with all of it. Landed together, the
  semantics would have been reviewed inside a wall of signature churn, and there
  would have been no checkpoint to bisect against. The split also made the
  acceptance criteria different in kind — 1b had to leave parity *exactly*
  unmoved, 1c had to move exactly the four pinned cases and nothing else. Both
  held.
- **Reusable method:** for a type change of this shape, change the signatures
  first and let `rustc` enumerate the leaves (57 here) instead of grepping for
  them. The compiler's list is exhaustive by definition; the transform was
  applied from its own line/column output.

### R11 — ORDER BY resolves columns with its own copy of the resolver
- **Status:** 🔴 OPEN — filed 2026-08-30 by [P34](SQL_PARITY.md#p34)
- **Where:** `query_engine::apply_multi_order_by_with_context` vs.
  `query_engine::resolve_column_index` (same file, ~3000 lines apart)
- **Observed:** `resolve_column_index` carries a doc comment declaring itself
  the canonical resolver, "used by all SQL clauses (WHERE, SELECT, ORDER BY,
  GROUP BY) to ensure consistent alias resolution behavior". ORDER BY does not
  call it. It has its own inline resolution instead, and that copy differs on
  two points: it treats any dot in the name as a table qualifier and takes the
  suffix (the canonical one tries the literal name **first**), and it ignores
  `table_prefix` entirely (the canonical one resolves the alias and tries the
  qualified name). The first divergence was a live wrong-answer bug for
  quoted dotted column names — [P34](SQL_PARITY.md#p34).
- **Impact:** one hard error, now fixed narrowly in place. The second
  divergence — ORDER BY ignoring `table_prefix` — has no known failing case
  because the fallback to the bare name happens to work after projection
  unqualifies the columns. That is luck, not design.
- **Same shape as [R9](#r9):** one helper documented as the single
  implementation, plus a hand-rolled copy at a site that never adopted it, and
  the copy is the one that is wrong. As with R9, the copy looks locally
  reasonable — nothing at the ORDER BY site hints that a shared resolver exists.
- **Deliberately deferred at fix time.** Converging on `resolve_column_index`
  changes behaviour for *unquoted* dotted names: the canonical path matches a
  column's `qualified_name`, the ORDER BY copy strips to the suffix. Those
  differ whenever a table's columns were not enriched with qualified names, so
  the swap is a behaviour change and wants its own change with parity run,
  not a rider on a bug fix. The P34 fix therefore only reordered the ORDER BY
  copy's own lookups (literal first) and left the copy in place.
- **When done:** delete the inline resolution, call `resolve_column_index`, and
  keep both P34 regression tests green — they pin exactly the two behaviours
  the shared resolver has to reproduce.

### R12 — Two aggregate registries, nine functions implemented twice
- **Status:** 🔴 OPEN — filed 2026-09-06 by [P41](SQL_PARITY.md#p41)
- **Where:** `src/sql/aggregates/` (old, `AggregateRegistry`) and
  `src/sql/aggregate_functions/` (new, `AggregateFunctionRegistry`). Dispatch is
  in `ArithmeticEvaluator`, which holds both (`arithmetic_evaluator.rs:24-25`,
  the fields commented *"old registry (being phased out)"* and *"new registry"*)
  and checks the new one **first** at both call sites (`:637`, `:769`).
- **Observed:** the migration adds to the new registry without removing from the
  old, so the two overlap and the old copy is unreachable for everything in the
  intersection. As of filing:

  | | Functions |
  |---|---|
  | **Both — old copy is dead** | `AVG`, `MIN`, `MAX`, `STDDEV`, `VARIANCE`, `MEDIAN`, `MODE`, `PERCENTILE`, `STRING_AGG` |
  | **New only** | `COUNT`, `COUNT_STAR`, `SUM` — the three done properly, commented out in the old list |
  | **Old only — live** | `STDDEV_POP`, `STDDEV_SAMP`, `VAR_POP`, `VAR_SAMP`, and the analytics set (`DELTAS`, `SUMS`, `MAVG`, `PCT_CHANGE`, `RANK`, `CUMMAX`, `CUMMIN`) |

  So the old registry is neither dead nor live — it is both, per function, and
  nothing in either file says which.
- **Impact: a fix can land in the wrong one and pass its own tests.** That is not
  hypothetical; it is how [P41](SQL_PARITY.md#p41) went. The finding named
  `ModeState` (old), the fix was written and unit-tested there, and the repro
  still flipped between runs, because the live `MODE` is `CollectorState` in the
  new registry. Worse, the two implementations *disagree on semantics*: the dead
  one keys on a string rendering and returns the original `DataValue`; the live
  one collects `Vec<f64>` and so rejects non-numeric input and returns a float
  for integers ([P42](SQL_PARITY.md#p42)). The better implementation is the
  unreachable one.
- **Same shape as [R8](#r8) and [R11](#r11)**, with the failure mode of R8 (a
  parallel stack with its own semantics) and the trap of R11 (the copy that runs
  is the wrong one). It is worse than either in one respect: R8's legacy stack is
  reachable only from tests, and R11's duplication is 3000 lines apart in *one*
  file. Here both registries are genuinely live, and which one serves a given
  function is invisible at every call site.
- **Decision:** converge on the new registry, but **not as one change**. Order:
  (1) make the overlap harmless — assert at construction that the two key sets
  are disjoint, or simply delete the nine shadowed entries from the old list,
  which is a provable no-op since they are unreachable today; (2) port the
  old-only functions (`*_POP`/`*_SAMP` and analytics) across; (3) delete the old
  registry. Step 1 is the one worth doing soon and is small — it is what stops
  the next P41.
- **Guard rail:** step 1 must not change behaviour, so the acceptance test is
  parity staying at exactly its current AGREE count, plus the FORMAL examples.
  [P42](SQL_PARITY.md#p42) is the natural companion to step 2 — porting `MODE`'s
  type handling from the dead implementation to the live one is most of that fix.

### R13 — Two expression evaluators, every boolean operator implemented twice
- **Status:** 🟡 IN PROGRESS — filed 2026-09-13 out of [P46](SQL_PARITY.md#p46);
  **slices 1 and 2 done 2026-09-13, slice 3 (with 3b) done 2026-09-14**, slice 4
  in progress (`BETWEEN`, `IN`, `NOT IN` delegated 2026-09-15).
  **The active workstream:** parity fixes that touch expression evaluation land
  as slices of this entry, not as patches.
- **Where:** `src/data/arithmetic_evaluator.rs` (`ArithmeticEvaluator`, value →
  `DataValue`, 1.8k lines) and `src/data/recursive_where_evaluator.rs`
  (`RecursiveWhereEvaluator`, predicate → `Trilean`, 1.6k lines). Callers in
  `query_engine.rs`, `group_by_expressions.rs` and `hash_join.rs`.
- **Observed:** the engine began with one evaluator per clause and never
  converged. Both walk the same `SqlExpression` tree and each has its own arms
  for the same operators:

  | | `ArithmeticEvaluator` | `RecursiveWhereEvaluator` |
  |---|---|---|
  | Serves | SELECT, GROUP BY, HAVING, JOIN conditions, generator args — and, since P46, WHERE *operands* | WHERE only (one construction site, `query_engine.rs`) |
  | Own arms for | `IN`, `NOT IN`, `BETWEEN`, `NOT`, `AND`/`OR`, `LIKE`, `IS NULL`, `CASE`, method calls | the same set again, plus `CASE`-as-predicate |
  | Logic | two-valued: comparisons/`IN`/`BETWEEN` answer `false`, `AND`/`OR` go through `to_bool` (NULL → false) | three-valued ([R10](#r10)) |
  | Method calls | map `.Contains()` etc. onto registry functions | hand-rolled `contains`/`startswith`/`endswith`/`trim*`/`length` |
  | Case-insensitive | hard-coded `false` in its `compare_with_op` calls | honoured |

  And three different "is this true?" collapses: `Trilean::is_true`
  (WHERE), `ArithmeticEvaluator::to_bool` (inside the value evaluator), and
  `group_by_expressions::is_truthy` (HAVING). JOIN compares with bare
  `compare_with_op`, which answers `NULL = NULL` as equal.
- **Impact: one predicate means different things depending on the clause it is
  written in.** Probed 2026-09-13 on `null_edges.csv`; filed by slice 1:

  | Shape | Where | Ours | SQL / DuckDB |
  |---|---|---|---|
  | `NOT (score > bonus)`, bonus all NULL | WHERE | no rows ✅ | no rows |
  | `NOT (MIN(score) > 40)`, one all-NULL group | HAVING | **group kept** 🚫 | dropped — [P50](SQL_PARITY.md#p50) |
  | `score IN (50, NULL)` for a NULL score | SELECT | **true** 🚫 | NULL — [P48](SQL_PARITY.md#p48) |
  | `a.label = b.label` self-join, 5 NULL labels | JOIN ON | **32 rows** 🚫 (NULLs pair up, 7 + 5×5) | 7 — [P52](SQL_PARITY.md#p52) |
  | `id < partner_id` for a NULL partner | SELECT | **false** 🚫 | NULL — [P48](SQL_PARITY.md#p48) |

  [P46](SQL_PARITY.md#p46) was the same split from the other side: WHERE lagging
  on operand resolution the value evaluator already had. So every operator fix
  is two fixes, and the history says they drift — R10 fixed NULL logic in one
  evaluator and none of the others. This is the finding that makes future
  semantics work ad hoc by construction.
- **Also:** evaluator construction is scattered — 16 construction sites outside
  the evaluator's own file, each `ArithmeticEvaluator::new` rebuilding the
  function registry and both aggregate registries. P46 showed the per-row form
  of this costing 9.8 s over 100k rows.
- **Target shape — one decision tree.** A single expression evaluator: every
  expression evaluates to a `DataValue`, with `Null` standing for UNKNOWN when
  the expression is a predicate. The R10 truth tables become its `AND`/`OR`/`NOT`
  arms. Every filtering site — WHERE, HAVING, JOIN ON, QUALIFY, CASE WHEN —
  evaluates the same way and collapses exactly once, at its own boundary, with
  one sanctioned function (`Trilean::from` a value, then `is_true`).
  `RecursiveWhereEvaluator` shrinks to that adapter plus the LIKE regex cache,
  then goes. This is the evaluation counterpart of [R2](#r2): R2 gave traversal
  one shape; R13 gives *meaning* one place.
- **Which evaluator is the base — decided with slice 1.** Neither, wholesale.
  `ArithmeticEvaluator` has the right *shape*: it maps any expression to a value,
  already serves every clause, and routes method calls through the function
  registry. `RecursiveWhereEvaluator` has the right *rules*: three-valued logic
  (R10) and case-insensitivity. So the value evaluator is the path everything
  threads through, and the WHERE evaluator's rules move into it (slice 2) before
  WHERE starts delegating to it (slice 4). The matrix below is what makes that
  move checkable.
- **Slice 1 result (2026-09-13).** `tests/evaluator_matrix_tests.rs` runs 36
  predicates through both evaluators over a five-row table with NULL in every
  operand position; the expected per-row answers were generated by DuckDB, not
  derived. **WHERE evaluator: 36 of 36. Value evaluator: 4 of 36** — the 32
  divergences are recorded in the file's `KNOWN` list, grouped by fault, with the
  corpus contract: a recorded divergence that changes, *including being fixed*,
  fails the test. The faults are wider than the entry assumed at filing — beyond
  "NULL gives false", the value evaluator compares **NULL to NULL as equal**, so
  `SELECT a = NULL` is TRUE for NULL rows ([P18](SQL_PARITY.md#p18) itself,
  live outside WHERE). Six clause-level corpus cases pin the user-visible forms;
  parity AGREE unchanged at 168, 197 → 203 cases. Filed:
  [P48](SQL_PARITY.md#p48) widened to the whole evaluator,
  [P50](SQL_PARITY.md#p50) HAVING, [P51](SQL_PARITY.md#p51) `AND` in a SELECT
  item does not parse (found by the matrix's own first run),
  [P52](SQL_PARITY.md#p52) NULL join keys on both join paths.
  **Slice 2's acceptance test is now concrete:** every `KNOWN` entry for the
  value evaluator disappears, and the corpus moves by exactly the P48/P50 cases.
- **Slice 2 result (2026-09-13).** Every boolean-producing arm of
  `ArithmeticEvaluator` now answers in three-valued logic, NULL for UNKNOWN:
  a predicate-layer `compare_trilean` for comparisons (the shared comparator keeps
  `NULL = NULL` equal for ORDER BY), `Trilean`'s tables for `AND`/`OR`/`NOT`
  (`to_bool` deleted), OR's table for `IN`/`NOT IN`, AND's for `BETWEEN`, NULL
  from `LIKE`, and simple `CASE x WHEN NULL` no longer matching (found and pinned
  at the start of the slice). `Trilean::from_value`/`to_value` became the one
  crossing between `DataValue` and the truth tables. **The acceptance slice 1
  set held exactly:** all 32 value-evaluator `KNOWN` entries went FIXED in one
  change with the WHERE column unmoved, and parity moved by exactly the six
  P48/P50 cases (168 → 174 AGREE over 204). No FORMAL example churn. Scoping
  first paid off here too: the only external consumers of predicate output were
  the WHERE evaluator (already NULL → UNKNOWN), HAVING's `is_truthy` and `IIF`
  (both NULL → not-true, already correct), so nothing downstream had to change.
  **Two evaluators now agree on all 36 predicates, but there are still two** —
  slices 3–6 are what make that agreement structural rather than coincidental.
- **Slice 3 result (2026-09-14).** Two no-op commits, then a pinned semantic
  one (3b), each checked against a byte-identical parity report.
  - *Registries built once.* The function, both aggregate and the window
    registries are immutable after construction, so a process-wide `OnceLock`
    builds them and every `ArithmeticEvaluator` takes `Arc` clones. GROUP BY was
    paying for it per group and per aggregate: over `trades_100000.csv`, net of
    ~3 s load, a high-cardinality GROUP BY went 3.4 s → 1.9 s and GROUP BY +
    HAVING 2.5 s → 0.8 s. `estimate_group_cardinality` also built one per row.
  - *One constructor.* `new` / `with_date_notation` /
    `with_date_notation_and_registry` differed only by a date-notation string
    stored in a field **nothing read** — dates parse through the global
    notation. So "pass date notation in" turned out to be "delete it": the
    field, the constructors, `apply_group_by_expressions`' parameter and
    `QueryEngine`'s own unread field went. Table aliases needed nothing:
    alias-qualified operands already resolve in every clause probed.
  - *3b — case-insensitivity.* The one context gap with a visible answer: under
    `--case-insensitive`, `WHERE status = 'ACTIVE'` matched `Active` but
    `SELECT status = 'ACTIVE'`, HAVING, aggregate arguments and CASE WHEN did
    not — nor WHERE itself once the predicate sat under a CASE, which WHERE
    hands to its inner value evaluator. Slice 1 had not pinned it (the parity
    harness cannot: DuckDB has no such mode), so 3b pinned first — a
    case-insensitive run of the matrix (10 predicates, expectations from DuckDB
    with `COLLATE NOCASE` / `ILIKE`) plus five clause-level cases through
    `QueryEngine::with_case_insensitive(true)` — recording 11 + 5 divergences.
    The fix, `ArithmeticEvaluator::with_case_insensitive` handed in at all 16
    construction sites (comparisons, `IN`, `BETWEEN`, `LIKE`), flipped exactly
    those and nothing else; parity report byte-identical.
  - *Still case-sensitive outside WHERE, not pinned:* simple
    `CASE x WHEN 'v'` (its own `values_equal`, with its own numeric rules — goes
    with slice 4) and method calls such as `.Contains()` routed to registry
    functions (slice 5).
- **Slice 4, first part (2026-09-15).** `BETWEEN`, `IN` and `NOT IN` in WHERE
  now evaluate through the value evaluator and cross back once, in
  `evaluate_delegated` (`Trilean::from_value`); WHERE's own copies of the
  AND/OR-over-comparisons rules are deleted. The P15 raw-window-operand check
  stays on WHERE's side of the delegation (`reject_unlifted_window`) — the value
  evaluator would evaluate one quietly. Five commits, parity report
  byte-identical after each, matrix green in both runs:
  - *Pin, then fix, alias resolution.* Delegating moves column resolution onto
    the value evaluator, so both resolvers were probed first. Both were wrong
    for an alias-qualified operand over two columns sharing a name: WHERE's fast
    path resolved `i.a` to an index through the `ExecutionContext`, turned it
    back into the bare name and looked *that* up (finding the first `a`), and
    the inner value evaluator was built with no aliases. Latent — join output
    disambiguates duplicate names before WHERE sees them, so no SQL probe
    reached it — but pinned as five recorded divergences, then fixed by using
    the index directly and handing the context's aliases in.
  - *Logging was the cost.* The first delegation (BETWEEN) looked like a 6×
    per-row regression. It was not the evaluator: non-interactive mode records
    TRACE to a ring buffer and a log file, and `ArithmeticEvaluator::evaluate`
    formatted the expression with `{:?}` at every node of every row. Removed —
    and every existing value-evaluator path got faster with it (over
    `trades_100000.csv`, ~2.4 s load: `WHERE price + 0 > quantity` 5.5 → 3.0 s,
    `SELECT price * quantity` 4.8 → 2.6 s, `GROUP BY UPPER(trader)` 4.3 →
    2.6 s). Delegated BETWEEN / IN / NOT IN each ended *faster* than the arm
    they replaced.
  - *Still ahead for the comparison arms* — each needs its own pin, since the
    operand readers are not quite the same: WHERE reads `DATETIME(...)` as a
    `DateTime` and `TODAY` in **local** time, the value evaluator as a `String`
    and in **UTC** (the two disagree near midnight); WHERE reads `5.0` as
    `Integer`, the value evaluator as `Float`; a comparison's left side may be a
    legacy method call (`Length()`, `IndexOf()`, `Trim*()`) with no registry
    twin yet (slice 5). `LIKE` has a regex cache in `EvaluationContext` that the
    value evaluator's matcher does not use. `AND` / `OR` / `NOT` / `CASE` stay
    recursive until their children can be WHERE-only method calls no longer.
  - *Noticed, not done:* `WHERE price > quantity` (WHERE's own path) still runs
    ~1 s over load where the delegated shapes do not — likely more per-row
    logging in or around the WHERE loop; and window evaluation logs `info!`
    timings per row (out of R13's scope).
- **Slices, in the R10 pattern — no-op slices kept apart from the one that
  changes answers:**
  1. ✅ **Pin the divergences, no engine change.** *(Done 2026-09-13.)* A per-operator matrix run through
     both evaluators — `=`, `<>`, `IN`, `NOT IN`, `BETWEEN`, `LIKE`, `NOT`,
     `AND`/`OR`, `CASE WHEN` — each with NULL on either side, asserting the two
     agree, with today's disagreements marked. Plus corpus cases for the
     clause-level shapes above (HAVING, SELECT, JOIN ON), filed as P-findings.
     Acceptance: parity buckets move only by the new cases.
  2. ✅ **Make `ArithmeticEvaluator` three-valued.** *(Done 2026-09-13 — see result below.)* The one semantic slice:
     comparisons, `IN`/`NOT IN`, `BETWEEN`, `LIKE` yield NULL for a NULL operand;
     `AND`/`OR`/`NOT` follow the truth tables; `to_bool` stops mapping NULL to
     false for predicates. Closes [P48](SQL_PARITY.md#p48) and the HAVING / SELECT
     rows above. Check `CASE WHEN` (UNKNOWN takes ELSE) and every `to_bool` /
     `is_truthy` caller. Acceptance: exactly the slice-1 cases flip, nothing else.
  3. ✅ **One construction path.** *(Done 2026-09-14 — see result above.)* Registries built once and shared (`Arc`), context
     — case sensitivity, date notation, table aliases — passed in rather than
     defaulted per site. No-op; acceptance is parity *exactly* unchanged, and it
     fixes the case-insensitivity gap as a side effect only if slice 1 pinned it.
  4. **Retire WHERE's duplicate arms, one operator per commit.** Each arm of
     `RecursiveWhereEvaluator` delegates to the value evaluator and collapses;
     the hand-written arm is deleted. After slice 2 the rules already agree, so
     each commit is a provable no-op — parity exactly unchanged, and the slice-1
     matrix still green.
  5. **Method calls:** WHERE's hand-rolled `Contains`/`StartsWith`/`Trim`… move
     onto the registry mapping the value evaluator already uses.
  6. **Collapse sites converge:** HAVING's `is_truthy` and JOIN's bare comparison
     go through the same collapse. JOIN NULL keys need care on the *hash* path
     too, not only the nested loop. [R8](#r8) stage 2 (the legacy WHERE stack) is
     a natural lull task alongside.
- **Explicitly not in scope:** window evaluation (`BatchWindowEvaluator`),
  aggregate registries ([R12](#r12)), correlated scoping ([R7](#r7) /
  [P3](SQL_PARITY.md#p3), [P49](SQL_PARITY.md#p49)). And [P15](SQL_PARITY.md#p15)
  stays a lifter fix — a single evaluator must still refuse a raw window
  function in a filter.
- **Guard rails:** the FORMAL examples capture our own output, so slice 2 may
  churn expectations — review each diff as a potential [P21](SQL_PARITY.md#p21)-
  style capture of a wrong answer, never re-capture blind. Watch the per-row
  cost when WHERE arms start delegating (slice 4): the P46 benchmarks
  (`price > quantity`, `price + 0 > quantity` over `trades_100000.csv`, inline
  QUALIFY) are the baseline.

---

## Sequencing

The dependency that matters: **the advanced work is gated on R2/R3.** Richer
expression and function support means new `SqlExpression` variants, and today
each one costs a 40-file hand audit with silent failure as the default outcome.

```
R2 walkers ──┬─→ R3 catch-alls retired
             │
             └─→ P3 Exists variant + scope spine ──→ correlated subqueries
                        ↑
                   R6 analyzer wiring
                        ↑
                   R7 per-row evaluation

R1 FROM migration ── independent; deferred (larger than P3)
R4 fixtures ──────── adopt opportunistically, per transformer touched
R5 dead code ─────── opportunistic
R8 legacy WHERE ──── independent; stage 2 is self-contained, do it in a lull
R10 Trilean ──────── DONE; closed P18/P19 (parity 125 → 129)
R11 ORDER BY resolver ─ independent; small, but a behaviour change — wants its own parity run
R12 aggregate registries ─ independent; step 1 is a provable no-op, do it before the next aggregate fix
R13 one evaluator ─── ACTIVE from 2026-09-13; slices 1 (pin), 2 (3VL), 3 (construction + case mode) DONE; 4 IN PROGRESS (BETWEEN/IN done) → 5–6
```

**A note on ordering, from the P18/P19 work being next.** The WHERE evaluator
returns `Result<bool>`, so there is no UNKNOWN in the type: `NOT IN` is
implemented as `Ok(!in_result)` and `NOT` as `Ok(!inner)`
(`recursive_where_evaluator.rs`). Three-valued logic cannot be *stated* until
that result type carries UNKNOWN, so P18/P19 is a type change first and a
semantics change second — not a per-operator patch. The seam is narrow: 9
signatures and 14 match arms in one file, two construction sites, and exactly
one semantic boundary (`if result` in `query_engine.rs`, the row filter). Doing
the type conversion while collapsing `Unknown → false` at that boundary is a
provable no-op — the acceptance test is that parity stays at exactly its current
AGREE count — which makes it safe to land well before the semantics change.

## Log

| Date | Change | PR |
|---|---|---|
| 2026-07-18 | R1 partial: FROM sync helpers; four desync sites fixed; dead lifter deleted | #30 |
| 2026-07-18 | R2 partial: `walk.rs` traversal helpers landed (additive) | #31 |
| 2026-07-18 | R2 group 1: the three boundary-crossing transformers migrated; two silent bugs fixed | #33 |
| 2026-07-18 | R3 evidence: P9–P12 filed after probing the engine; corpus gains tier 07 (grouping) | — |
| 2026-07-18 | R2: `*_crossing` helpers become the primitives; the three crossing transformers stop hand-listing subquery variants | #35 |
| 2026-07-19 | R2: `having_alias_transformer` migrated — closes P9, three corpus cases DIFFER → AGREE | — |
| 2026-07-25 | R2: `where_alias_expander` migrated — closes P11; the four subquery-LHS variants came for free | — |
| 2026-08-02 | Corpus tiers 08 (ordering), 09 (window/QUALIFY), 10 (aggregate/NULL) added; P13–P15 filed. 83 → 96 cases | — |
| 2026-08-08 | R8 filed and stage 1 done: 830 lines of zero-caller legacy WHERE deleted (`where_clause_converter`, `where_evaluator`, `simple_where`) plus `DebugWidget`'s dead WHERE-AST code. No behaviour change | #47 |
| 2026-08-16 | R9 filed and done: view→table materialization consolidated on `materialize_view`; `DataView` gains `windowed_row_indices`/`with_max_rows`. Closes parity P28 and P31 | — |
| 2026-08-22 | R10 filed; slice 1a landed: `data::trilean` with the 3VL truth tables and 13 unit tests. Unwired — no behaviour change, parity unmoved at 125/159 | #51 |
| 2026-08-22 | R10 slice 1b: WHERE evaluator converted to `Result<Trilean>`; `is_true()` collapse at the single row-filter boundary. `Unknown` still never constructed, so no behaviour change — parity unmoved at 125/159 | #53 |
| 2026-08-22 | R10 slice 1c: UNKNOWN produced at the leaves via `compare_trilean`. Closes parity P18/P19 — 125 → **129 AGREE**; new finding P32 (`NOT LIKE` parse gap) pinned, not fixed | — |
| 2026-08-30 | `RANGE` window frames given peer-group semantics (`OrderedPartition::peer_bounds`); sorting and peer detection unified on one comparator. Closes parity P24 — 129 → **133 AGREE**; new finding P33 (`RANGE` with a numeric offset) now a deliberate hard error rather than a silent ROWS answer | — |
| 2026-08-30 | P34 fixed: `ORDER BY "col.with.dot"` no longer strips a quoted identifier at the dot. R11 filed — ORDER BY still resolves columns with its own copy of `resolve_column_index` rather than the canonical one | — |
| 2026-09-04 | P40 filed from field use: a generator's args are evaluated against DUAL (`statement_executor.rs` has no `from_function` case), so no column reference resolves in `FROM SPLIT(col, …)`. Cross-linked here — R1's missing table-function variant is the reason generators sit on the legacy path | — |
| 2026-09-06 | R12 filed by parity P41: two aggregate registries, nine functions implemented twice with the newer one shadowing the older. P41's fix was written against the dead copy first and changed nothing — the entry records the disjointness assertion as step 1 | — |
| 2026-09-13 | P46 fixed: all WHERE operands resolve through one path, one `ArithmeticEvaluator` per evaluation. Parity 157 → **168 AGREE**; P49 filed | #80 |
| 2026-09-13 | R13 filed and made the active workstream: two evaluators with divergent NULL semantics, three truth collapses. Probing found four live clause-dependent divergences (HAVING, SELECT `IN`, JOIN NULL keys, P48) | — |
| 2026-09-13 | R13 slice 1: evaluator matrix (36 predicates × both evaluators × 5 NULL-bearing rows, DuckDB-derived expectations). WHERE 36/36, value evaluator 4/36. P48 widened; P50–P52 filed; six corpus cases, AGREE unchanged | — |
| 2026-09-13 | R13 slice 2: value evaluator three-valued; `Trilean::from_value`/`to_value`; `to_bool` removed. Matrix value column 4/36 → 36/36 in one change. Closes P48, P50 — 168 → **174 AGREE** | — |
| 2026-09-14 | R13 slice 3: registries built once per process; one `ArithmeticEvaluator` constructor, dead date-notation plumbing deleted — parity byte-identical, high-cardinality GROUP BY ~2× faster. 3b: case-insensitive mode pinned (11 matrix + 5 clause cases) then handed in at every construction site; all flipped, parity byte-identical | — |
| 2026-09-15 | R13 slice 4 (first part): WHERE's `BETWEEN`, `IN`, `NOT IN` delegate to the value evaluator. Latent alias-resolution fault pinned and fixed first; per-node TRACE logging found to be most of the value evaluator's cost and removed (SELECT/GROUP BY expressions ~2 s faster over 100k rows). Parity byte-identical at every commit | — |
