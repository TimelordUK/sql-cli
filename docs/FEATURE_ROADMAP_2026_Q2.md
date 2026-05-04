# Feature Roadmap — 2026 Q2

_Living document. Written 2026-04-20 after the READ_TEXT/GREP/READ_WORDS landing._

Organised by **leverage per hour of work**, not urgency. The guiding principle
is the one-shop philosophy (see `docs/` and project memory): prefer features
that make a *new class of data queryable* over yet another scalar function.

---

## Tier 1 — Readers (highest leverage)

Each reader below turns a new class of data into a queryable table. They all
follow the established `TableGenerator` pattern in
`src/sql/generators/` — look at `file_readers.rs` (READ_TEXT, GREP, READ_WORDS)
as the template.

### READ_JSONL(path [, match_regex])  ✅ landed (2026-05-04)

- Newline-delimited JSON. One row per JSON object; schema is the *union* of
  keys across the first 100 records so heterogeneous logs (auth events with
  user_id, http events with status, db events with query_ms) coexist as
  columns. Optional regex pre-filters lines before JSON parsing.
- The same auto-detect (array vs JSONL) was wired into the file-level JSON
  loader, so `sql-cli logs.json` Just Works either way. `.jsonl` and `.ndjson`
  extensions are recognised alongside `.json`.
- See `examples/jsonl_logs.sql` and `data/app_logs.jsonl` for the demo.

### READ_FIELDS(path, delim_regex [, match_regex])

- The awk-style reader we designed but didn't build on 2026-04-19.
- Yields `(line_num, field_num, field)` — user pivots with `MAX(CASE WHEN
  field_num = 1 THEN field END) AS col1` etc.
- Default `delim_regex` to `\s+` when NULL/omitted (classical awk behavior
  already agreed).
- Real log files (access.log, syslog, colon-separated configs) need this.

### READ_STDIN([match_regex])

- Enables `cat file | sql-cli -q "SELECT ... FROM READ_STDIN()"`.
- Probably 30 lines. Huge usability win for shell pipelines.
- Yields same `(line_num, line)` as READ_TEXT.

### Glob support in existing readers

- `READ_TEXT('logs/*.log')` yields an extra `source_file` column.
- One new column; query shape unchanged.
- Multi-file ingestion without UNION ALL is a big ergonomic win.

### READ_HTTP(url [, headers_json])

- We have web CTE but a first-class reader is more discoverable.
- Natural pair with existing HTTP fetcher (see `src/web/http_fetcher.rs`).
- Consider returning raw body + a second reader like READ_JSON_HTTP that parses.

### READ_PARQUET(path)

- Columnar format is common in data work. Would need the `parquet` crate.
- Heavier lift (schema handling, type mapping) — only tackle if there's a
  clear use case, since it adds a non-trivial dependency.

---

## Tier 2 — Ship what we have

### examples/everything.sql

- Already planned in project memory. Curated tour of the whole engine
  (~30-40 deterministic queries). Ideal for a lull — pure composition,
  no new code, doubles as formal regression test via captured expectations.
- Keep deterministic: no timestamps, no filesystem paths, use RANGE/UNION ALL
  for data generation where possible.

### More captured expectations

- `examples/text_processing_demo.sql` is a great candidate to promote from
  smoke test to formal. `uv run python tests/integration/test_examples.py
  --capture text_processing_demo` after confirming output is stable.
- Each captured expectation = one more regression guard.

### README refresh

- The one-shop story deserves a headline section. Current README undersells
  what makes this project distinctive.
- Concrete "wow" query example: full-book scan at sub-100ms, heterogeneous
  sources in one query (join CSV with READ_TEXT output), etc.

### Benchmark integration

- `benchmark_baseline.json` and `benchmark_results_20260411_223032.json` sit
  uncommitted in the repo root. Either wire them into CI (perf regression
  guard) or delete them.

### Session-level config for safety caps

- `MAX_LINES_PER_FILE = 1_000_000` is hardcoded in `file_readers.rs`.
- Expose via `SET max_lines_per_file = N` so power users can raise it
  without recompiling. Same mechanism can cover future reader caps.

---

## Tier 3 — Engine gaps (narrower, lower leverage)

Per `project_leetcode_gap_next.md`, end-to-end coverage sits at 87%. Remaining
gaps are small and contained.

### MySQL DATE_FORMAT specifier translation

- We shipped `DATE_FORMAT` as a chrono-strftime alias, which covers the
  common case (`%Y`, `%m`, `%d`). Full MySQL compat needs `%i` (minute),
  `%W` (full weekday name), `%M` (full month name), `%r` (12-hour time).
- ~40 lines: translate MySQL specifiers to strftime before delegating.

### BETWEEN-in-CASE → now landed (2026-04-19)

- ✅ Fixed. LC 3705 runs verbatim.

### GROUP BY output projection bug

- `SELECT user_id + 10 AS shifted FROM t GROUP BY user_id` outputs
  `user_id, shifted` instead of just `shifted`. Real defect.
- **Risk:** may be load-bearing — other code might depend on group keys
  being present in the output. Audit carefully before changing; probe path
  is `apply_group_by` in the query engine.

### HAVING expressions parity — probe first

- HAVING arithmetic + aliases work. Systematic sweep: does HAVING+CASE,
  HAVING+function-calls, HAVING+scalar-subqueries work? File gaps as they
  surface.

### 608 InSubquery in CASE — probe first

- Older memory flagged this, but an even-earlier gap doc showed 608 passing.
- Run the query before assuming it's broken. If broken, same recursion
  template used for HAVING/GROUP BY/ORDER BY transformers.

### Functional dependency relaxation (LC 3521, 3601) — needs scoping

- MySQL allows `SELECT p1.category FROM ... GROUP BY product_id` when
  category is FD on the grouped key. Don't implement blind.
- Three options: (a) full FD analysis, (b) "pick any value from group" with
  config flag, (c) detect JOIN-primary-key case specifically.
- Have the conversation first.

### Permanent out of scope

- LC 177 CREATE FUNCTION (procedural SQL).

---

## Tier 3b — Quirks surfaced while writing showcase SQL

Found while building `examples/us_states.sql` (2026-05-04). Each is small,
contained, and has a clean workaround — so they don't block the showcase, but
they're worth documenting before they're forgotten. Reproducers all use
`data/us_states.csv`.

### `__hidden_orderby_1` error when ORDER BY references the inner alias of a GROUP BY CTE

```sql
WITH lengths AS (SELECT name, LENGTH(name) AS n FROM us_states)
SELECT n AS name_length, COUNT(*) AS states
FROM lengths GROUP BY n
ORDER BY n;       -- → "Column '__hidden_orderby_1' not found"
```

- Workaround: `ORDER BY name_length` (the projected alias).
- Hidden-orderby promotion (the same machinery that fixed LC 1341) doesn't
  account for GROUP BY rewriting `n` out of the post-aggregation projection.
- ~probably a 10-line fix in whatever resolves the hidden-orderby column
  against the group-by output schema.

### Scalar subqueries inside CASE not supported in arithmetic eval

```sql
SELECT CASE WHEN latitude = (SELECT MAX(latitude) FROM us_states)
            THEN 'Northernmost' END FROM us_states;
-- → "Unsupported expression type for arithmetic evaluation: ScalarSubquery {...}"
```

- Scalar subqueries work in WHERE; the gap is only in `arithmetic_evaluator.rs`
  CASE branch evaluation.
- Workaround in showcase: four separate `ORDER BY ... LIMIT 1` queries for the
  four compass extremes, instead of one CASE-labelled extremes query.

### Cross-CTE column equality silently filters to 0 rows

```sql
WITH conus AS (...),
     agg AS (SELECT MAX(latitude) AS lat_max FROM conus)
SELECT c.name FROM conus c CROSS JOIN agg a
WHERE c.latitude = a.lat_max;       -- → 0 rows
```

- The CROSS JOIN expands rows correctly (verified with `SELECT * LIMIT 3`),
  but post-join equality between a base column and an aggregate-CTE column
  yields no matches. Same shape in `JOIN ... ON OR ...` returns just the
  first matching row.
- Suspicion: type or precision mismatch when comparing the float column
  against the aggregated value. Worth a test case before fixing.
- Workaround: use a scalar subquery in WHERE — `WHERE latitude = (SELECT MAX(latitude) FROM conus)`
  works.

### UNION ALL legs ignore inner ORDER BY/LIMIT and lose per-leg literals

```sql
SELECT 'A' AS dir, name FROM us_states ORDER BY latitude DESC LIMIT 1
UNION ALL
SELECT 'B' AS dir, name FROM us_states ORDER BY latitude ASC LIMIT 1;
-- → all rows from both sides, all labelled 'A'
```

- Each leg's `ORDER BY ... LIMIT 1` is dropped, AND the literal alias from
  the first leg propagates to all rows.
- Wrapping each leg in `SELECT * FROM (... LIMIT 1) sub` doesn't help.
- Probable cause: the UNION ALL planner pulls SELECT lists from the first
  leg only and doesn't preserve per-leg limit/order. Two distinct bugs in
  one place.
- Workaround in showcase: four separate `GO` batches.

---

## Suggested working cadence

- **One reader per session** when there's time for a full slice.
- **Tier 2 items** (everything.sql, README, expectations) fit into shorter sessions.
- **Tier 3 engine gaps** are good "while waiting for something else" work.
- Update this document as items land — tick them off, add new gaps as they
  surface.

## Related reading

- `docs/DEVELOPMENT_ROADMAP.md` — older roadmap (bitwise ops, mostly shipped).
- `src/sql/generators/file_readers.rs` — template for new readers.
- Project memory: `project_leetcode_gap_next.md`, `project_design_philosophy.md`,
  `project_everything_sql.md`.
