# SQL Parity — Book of Work

The durable decision log for SQL-engine parity. We differentially test sql-cli
against a reference engine (**DuckDB** today) and systematically find, document,
and either **fix** divergences or record **why we won't**.

- **Harness / raw output:** `tests/comparison/` — run
  `uv run python tests/comparison/runner.py`; it regenerates
  `tests/comparison/reports/compare_<ref>.md` (the machine's current GAP/DIFFER list).
- **This file:** the curated, human decisions behind those buckets. The report
  says *what* diverges right now; this file says *what we're doing about it and why*.

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
- **Status:** 🔴 OPEN
- **Corpus:** `03_functions.toml :: fn_substring`
- **Observed:** `SUBSTRING('AAPL', 1, 2)` → sql-cli `'AP'`, DuckDB/standard `'AA'`.
- **Decision:** **Fix** — make `SUBSTRING` 1-indexed per SQL standard.
- **Notes:** String-position semantics may be inconsistent elsewhere (e.g. any
  `INDEXOF`/`CHARINDEX`/`LEFT`/`RIGHT` style helpers). Audit all position-based
  string functions as part of this fix, not just `SUBSTRING`.

### P2 — `CAST(expr AS type)` not supported
- **Status:** 🔴 OPEN
- **Corpus:** `03_functions.toml :: fn_cast_int`
- **Observed:** Parse error `Expected RightParen, found As`. There is no `CAST`
  in the parser; the engine relies on evaluation-time **coercion**, and `CONVERT`
  is unit conversion (3 args), not type casting.
- **Decision:** **Fix (best-effort within constraints).** Add `CAST(expr AS type)`
  to the parser and map it onto the existing coercion layer
  (`src/data/arithmetic_evaluator.rs` / `DataValue`). Support the common target
  types we can represent: INTEGER/BIGINT, DOUBLE/FLOAT/REAL, VARCHAR/TEXT,
  BOOLEAN, DATE/TIMESTAMP. Types we cannot faithfully represent are documented
  as unsupported rather than silently mis-cast.
- **Notes:** Coercion-first is our design; CAST should be explicit sugar over the
  same rules so results stay consistent with implicit coercion.

---

## Won't fix (intentional divergences)

_None yet. When we consciously diverge from the reference engine, record it here
with the rationale so the DIFFER is understood, not mistaken for a bug._

---

## Workflow

1. Add/extend a tier in `tests/comparison/corpus/NN_*.toml` and run the harness.
2. For each `GAP`/`DIFFER`, add an entry here with a decision.
3. Annotate the corpus case with `expect = "GAP"` / `"DIFFER"` so the run flags
   the day the bucket changes.
4. On fix: implement, confirm the case flips to `AGREE`, drop the `expect`, set
   the entry to 🟢 FIXED (or move to Won't Fix with rationale).
