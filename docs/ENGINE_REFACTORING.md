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
