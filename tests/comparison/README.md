# SQL engine comparison harness

Differential testing of the sql-cli engine against a reference engine
(**DuckDB** today; SQLite / SQL Server are drop-in via the adapter interface).
The goal is *broad-brush* parity, not byte-for-byte equality: run the same data
and the same SQL through both engines, normalize away formatting noise, and see
where we agree, disagree, or fall short.

## Run

```bash
uv run python tests/comparison/runner.py            # all tiers
uv run python tests/comparison/runner.py 01 02      # only tiers 01, 02
uv run python tests/comparison/runner.py --verbose  # show diff detail for every case
uv run python tests/comparison/runner.py --ref duckdb
uv run python tests/comparison/runner.py --check    # CI gate: exit 1 on any drift
```

## Regression gate (`--check`)

`--check` enforces the parity contract and exits non-zero on any violation:

- a case with `expect = "GAP"` (etc.) **must** still be in that bucket;
- a case with **no** `expect` **must** be `AGREE`.

So a regressed `AGREE`, a silently-closed gap, or a new un-annotated non-`AGREE`
case all fail the check. Closing a gap is a deliberate edit (drop the `expect`
and log it in `docs/SQL_PARITY.md`); regressions are caught for free. This is the
form run in CI (`.github/workflows/test-complete.yml`, job *SQL Parity*).

Requires `cargo build --release` (the harness shells out to `target/release/sql-cli`)
and the `duckdb` test dependency (`uv add --group test duckdb`, already in `pyproject.toml`).

## Buckets

Every case lands in exactly one bucket:

| Bucket | Meaning | What to do |
|---|---|---|
| `AGREE` | both run, results match | candidate to promote into the formal regression suite |
| `DIFFER` | both run, results disagree | semantics bug — investigate |
| `GAP` | reference runs, sql-cli errors | the backlog to pick off |
| `OURS_ONLY` | sql-cli runs, reference errors | our extension / library functions |
| `BOTH_ERR` | neither supports | frontier parity |

Reports are written to `reports/compare_<ref>.{json,md}` — the markdown lists the
GAP and DIFFER cases as a ready-made backlog.

## Corpus

Queries live in `corpus/NN_<name>.toml`, tiered from vanilla to complex. Each case:

```toml
[[case]]
id   = "where_in_subquery"     # unique, stable name
data = "trades.csv"            # file in ../../data; table name is the file stem
sql  = "SELECT ... FROM trades WHERE ..."
expect = "GAP"                 # optional: assert the current bucket; run flags changes
```

The table name a query references **must** be the data file's stem (matches how
sql-cli maps a loaded file to a table). Alias every computed column so its name
lines up with the reference engine — comparison is by column *name*, since
sql-cli's JSON output sorts keys.

`expect` is optional. When set, the run flags any case whose bucket no longer
matches — so closing a gap or fixing a DIFFER shows up immediately without
editing the harness.

## Normalization

Comparison canonicalizes harmless cross-engine differences (see `normalize.py`
for the authoritative list): NULL == empty string, int/float/numeric-string
collapse, boolean spelling, date/datetime objects → ISO with trailing
fractional-second zeros stripped. Row order is ignored unless the query has
`ORDER BY`. A `DIFFER` therefore means a *real* disagreement, not formatting.

## Adding an engine

Implement `Engine.run(data_file, table, sql) -> EngineResult` in `engines.py`
and register it in `REFERENCE_ENGINES`. Everything else (corpus, normalization,
report) is engine-agnostic.

## Roadmap

- More tiers: joins, subqueries, CTEs, recursive CTEs, window functions,
  correlated subqueries and column scoping (where the interesting gaps live).
- Promote stable `AGREE` cases into the formal regression suite.
- Import [sqllogictest](https://www.sqlite.org/sqllogictest/) corpora behind the
  same adapter interface once function/type coverage is broad enough.
