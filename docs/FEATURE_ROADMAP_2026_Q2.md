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

### READ_STDIN([match_regex])  ✅ landed (2026-05-17)

- `cat file | sql-cli -q "SELECT ... FROM READ_STDIN()"` works.
- Yields `(line_num, line)` like READ_TEXT; optional regex pre-filter.
- Cached once per process via `OnceLock` so multiple `READ_STDIN()` calls in
  the same query (CTE self-joins, etc.) see the same rows — stdin is
  consumable, this preserves composability.
- TTY-check on `is_terminal()` short-circuits with a helpful error instead of
  blocking forever on keyboard input.
- See `examples/stdin_pipeline.sql` for usage.

### Stdin sentinel `-` for all line-based readers  ✅ landed (2026-05-17)

- `READ_TEXT('-')`, `READ_JSONL('-')`, `GREP('-', pat)`, `READ_WORDS('-')` all
  read stdin (cached, same buffer shared with `READ_STDIN()`).
- Critically this unlocks **field-level JSONL pipelines** without needing a
  scalar `JSON_VALUE` function:
  ```
  cat events.jsonl | sql-cli -q "SELECT level, COUNT(*) FROM READ_JSONL('-')
                                  GROUP BY level"
  ```
- Universal Unix `-` convention; no surprise for shell users.

### READ_CSV(path)  ✅ landed (2026-05-17)

- `READ_CSV('data/trades.csv')` and `cat foo.csv | sql-cli -q "SELECT ... FROM
  READ_CSV('-')"` both work.
- Reuses `AdvancedCsvLoader::load_csv_from_reader` so type inference, string
  interning, and column-categorisation heuristics are inherited from the main
  CSV loader — behaviour matches `sql-cli file.csv -q "..."`.
- v1 is minimal (no `has_header`/`delimiter` args). Add those later if a real
  use case asks for them.

With READ_CSV the reader family now covers the four core formats — CSV, JSON
array files, JSONL, plain text — and all of them accept `-` for stdin. This
is the "complete loop" the user flagged on 2026-05-17.

#### Arbitrary delimiter argument  ✅ landed (2026-05-23)

```sql
READ_CSV(path [, delimiter])    -- ',' default; '|' for PSV, '\t' for TSV
```

Delivered across four commits, all on the shared `CsvReadOptions` plumbing
(`da3e739` infra → `c32986a` READ_CSV arg → `a488fac` CLI flag → `1515112`
WEB CTE clause):

- **READ_CSV 2nd arg** resolves the delimiter as: explicit arg wins → else
  extension auto-detect (`.tsv` → tab, `.psv` → pipe) → else comma (and comma
  for stdin `-`). `parse_delimiter_arg` accepts `\t`/`\n`/`\r` escapes since
  literal tabs in SQL strings are awkward; multi-char / non-ASCII error clearly.
- **Extension auto-detect for every caller** — the convenience loaders call
  `detect_delimiter_from_path` internally, so `sql-cli sales.psv` Just Works.
- **CLI `--delimiter` flag** for explicit override at the file level.
- **WEB CTE `DELIMITER` clause** for non-comma HTTP sources.
- Resolved delimiter is recorded in `table.metadata["delimiter"]` for F5
  visibility; `is_null_field` is parameterised on the delimiter so NULL
  detection works for non-comma sources.
- Demo: `examples/iris_tsv.sql` over `data/iris.tsv`.

This closes the four-format reader loop with full delimiter flexibility. The
only deferred follow-up is optional `has_header` (Bool, default true) for
headerless CSVs where we'd synthesise `col_1`, `col_2`, etc. — left until a
real use case asks for it.

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

### MySQL DATE_FORMAT specifier translation  ✅ landed (2026-05-31)

- `DateFormatFunction` now runs a single-pass `translate_mysql_date_format`
  before delegating to `FormatDateFunction`. MySQL-only specifiers are remapped:
  `%i`→`%M` (minute), `%M`→`%B` (full month name), `%W`→`%A` (full weekday),
  `%s`→`%S` (seconds, dodging strftime's epoch `%s`), `%f`→`%6f` (microseconds).
  Shared specifiers (`%Y %m %d %H %h %p %a %b %r %T %%` …) pass through verbatim.
- Single-pass is deliberate: a sequential string-replace would double-translate
  because `%i`→`%M` and `%M`→`%B` collide on the intermediate `%M`.
- `DATE_FORMAT('...', '%W, %M %d, %Y %H:%i:%s')` → `Sunday, May 31, 2026 14:05:09`.
- 4 unit tests in `format.rs::date_format_tests`. Demo + formal regression
  guard: `examples/date_format_mysql.sql` →
  `examples/expectations/date_format_mysql.json`.

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

### 608 InSubquery in CASE  ✅ landed (2026-05-31)

- Was broken (verified): `CASE WHEN x IN (SELECT ...) THEN ...` errored with
  `Unsupported expression type for arithmetic evaluation`. Two-part root cause,
  both fixed:
  1. `SubqueryExecutor::process_expression` didn't recurse into `CaseExpression`
     / `SimpleCaseExpression` / `Not`, so the subquery was never pre-executed
     and rewritten to an `InList`. Added those arms (line ~345 used to say
     `// CaseWhen doesn't exist in current AST, skip for now`).
  2. Once rewritten to `InList`/`NotInList`, the *arithmetic* evaluator had no
     dispatch for those nodes (only the WHERE evaluator did). Added `InList`/
     `NotInList` arms to `arithmetic_evaluator.rs` returning Boolean via the
     shared `compare_with_op`.
- Same fix also closes Tier 3b's "scalar subquery inside CASE" (below).
- Demo + formal regression guard: `examples/subquery_in_case.sql` (scalar,
  IN, and NOT IN subqueries inside CASE), captured to
  `examples/expectations/subquery_in_case.json`.

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

### Scalar subqueries inside CASE not supported in arithmetic eval  ✅ landed (2026-05-31)

```sql
SELECT CASE WHEN latitude = (SELECT MAX(latitude) FROM us_states)
            THEN 'Northernmost' END FROM us_states;   -- now works
```

- Fixed alongside Tier 3's "608 InSubquery in CASE" (same `SubqueryExecutor`
  recursion gap — see above). `SubqueryExecutor::process_expression` now
  descends into CASE branches, so the scalar subquery is pre-executed and
  materialised to a literal before arithmetic evaluation.
- The four-compass-extremes showcase query can now be a single CASE-labelled
  query instead of four `ORDER BY ... LIMIT 1` batches.

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
