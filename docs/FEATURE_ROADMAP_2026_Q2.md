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

### READ_JSONL(path [, match_regex])  ⭐ suggested next slice

- Newline-delimited JSON — the dominant format for app logs, streaming events,
  LLM output, dataset dumps. Unlocks querying those with zero pre-processing.
- One row per JSON object. Column schema inferred from the first N records
  (or flattened into `(line_num, json_text)` columns + scalar extractors).
- JSON parsing infrastructure already exists (see `src/data/json_datasource.rs`).
- Probably 60-100 lines + tests + example.

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
