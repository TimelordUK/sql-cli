#!/usr/bin/env -S uv run python
"""Differential SQL test harness: sql-cli vs a reference engine.

Runs the same data + SQL through sql-cli and a reference engine (DuckDB by
default), normalizes both result sets, and buckets each case:

  AGREE      both run, results match        -> certify as regression test
  DIFFER     both run, results disagree      -> semantics bug to investigate
  GAP        reference runs, sql-cli errors  -> the backlog to pick off
  OURS_ONLY  sql-cli runs, reference errors  -> our extension / library funcs
  BOTH_ERR   neither supports                -> frontier parity

Usage:
    uv run python tests/comparison/runner.py                 # all tiers
    uv run python tests/comparison/runner.py 01 02           # only those tiers
    uv run python tests/comparison/runner.py --verbose       # show diff detail
    uv run python tests/comparison/runner.py --ref duckdb    # pick reference
    uv run python tests/comparison/runner.py --check         # CI gate (exit 1 on drift)

Regression contract (enforced by --check, used in CI):
  - a case with `expect = "GAP"` (etc.) must still be in that bucket;
  - a case with NO `expect` must be AGREE.
Any violation fails the check. So a fixed gap, a regressed AGREE, or a new
un-annotated non-AGREE case all surface immediately. Closing a gap is therefore
a deliberate edit (drop the `expect`); regressions are caught for free.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

from engines import REFERENCE_ENGINES, SqlCliEngine
from normalize import compare, has_order_by

CORPUS_DIR = Path(__file__).resolve().parent / "corpus"
REPORT_DIR = Path(__file__).resolve().parent / "reports"


class C:
    RED = "\033[0;31m"
    GREEN = "\033[0;32m"
    YELLOW = "\033[1;33m"
    BLUE = "\033[0;34m"
    CYAN = "\033[0;36m"
    DIM = "\033[2m"
    NC = "\033[0m"


BUCKET_STYLE = {
    "AGREE": (C.GREEN, "✓"),
    "DIFFER": (C.YELLOW, "≠"),
    "GAP": (C.RED, "🚫"),
    "OURS_ONLY": (C.BLUE, "+"),
    "BOTH_ERR": (C.DIM, "·"),
}


def load_cases(tiers: list[str]) -> list[dict]:
    cases = []
    files = sorted(CORPUS_DIR.glob("*.toml"))
    for f in files:
        tier_prefix = f.stem.split("_", 1)[0]
        if tiers and tier_prefix not in tiers:
            continue
        doc = tomllib.loads(f.read_text())
        for case in doc.get("case", []):
            case.setdefault("tier", tier_prefix)
            case["_file"] = f.name
            cases.append(case)
    return cases


def bucket(cli_res, ref_res, sql) -> tuple[str, str | None]:
    if cli_res.ok and ref_res.ok:
        matches, diff = compare(cli_res.rows, ref_res.rows, has_order_by(sql))
        return ("AGREE", None) if matches else ("DIFFER", diff)
    if not cli_res.ok and ref_res.ok:
        return "GAP", cli_res.error
    if cli_res.ok and not ref_res.ok:
        return "OURS_ONLY", ref_res.error
    return "BOTH_ERR", f"sql-cli: {cli_res.error} | ref: {ref_res.error}"


def main() -> int:
    argv = sys.argv[1:]
    verbose = "--verbose" in argv
    check = "--check" in argv
    ref_name = "duckdb"
    if "--ref" in argv:
        ref_name = argv[argv.index("--ref") + 1]
    tiers = [a for a in argv if not a.startswith("--") and a != ref_name]

    if ref_name not in REFERENCE_ENGINES:
        print(f"Unknown reference engine '{ref_name}'. Available: {list(REFERENCE_ENGINES)}")
        return 2

    cli = SqlCliEngine()
    if not cli.binary.exists():
        print(f"{C.RED}ERROR: {cli.binary} not found. Run 'cargo build --release' first.{C.NC}")
        return 2
    ref = REFERENCE_ENGINES[ref_name]()

    cases = load_cases(tiers)
    print(f"=== sql-cli vs {ref.name} :: {len(cases)} cases ===\n")

    counts: dict[str, int] = {}
    violations: list[str] = []
    report_rows = []

    for case in cases:
        cid = case["id"]
        data = case["data"]
        sql = case["sql"]
        table = Path(data).stem
        cli_res = cli.run(data, table, sql)
        ref_res = ref.run(data, table, sql)
        b, detail = bucket(cli_res, ref_res, sql)
        counts[b] = counts.get(b, 0) + 1

        color, glyph = BUCKET_STYLE[b]
        print(f"{color}{glyph} {b:9}{C.NC} [{case['tier']}] {cid}")
        if detail and (verbose or b in ("DIFFER",)):
            for line in str(detail).splitlines():
                print(f"    {C.DIM}{line}{C.NC}")

        expect = case.get("expect")
        expected_bucket = (expect or "AGREE").upper()
        if b != expected_bucket:
            violations.append(f"{cid}: expected '{expected_bucket}' but is '{b}'")

        report_rows.append(
            {"id": cid, "tier": case["tier"], "file": case["_file"], "sql": sql,
             "bucket": b, "detail": detail, "expect": expect}
        )

    print("\n=== Summary ===")
    for b in ("AGREE", "DIFFER", "GAP", "OURS_ONLY", "BOTH_ERR"):
        if b in counts:
            color, glyph = BUCKET_STYLE[b]
            print(f"  {color}{glyph} {b:9}{C.NC} {counts[b]}")

    write_reports(ref.name, report_rows, counts)
    print(f"\nReports written to {REPORT_DIR}/")

    if violations:
        color = C.RED if check else C.YELLOW
        label = "Contract violations" if check else "Drift from expectations"
        print(f"\n{color}{label} ({len(violations)}):{C.NC}")
        for v in violations:
            print(f"  - {v}")
        print(
            f"\n{C.DIM}A case with `expect` must match its bucket; a case with no "
            f"`expect` must be AGREE.\n  Fixed a gap? drop its `expect`. New gap? "
            f"add `expect = \"GAP\"` and log it in docs/SQL_PARITY.md.{C.NC}"
        )
        if check:
            return 1
    elif check:
        print(f"\n{C.GREEN}Parity contract holds ({len(report_rows)} cases).{C.NC}")

    return 0


def write_reports(ref_name: str, rows: list[dict], counts: dict[str, int]) -> None:
    import json

    REPORT_DIR.mkdir(exist_ok=True)
    (REPORT_DIR / f"compare_{ref_name}.json").write_text(
        json.dumps({"reference": ref_name, "counts": counts, "cases": rows}, indent=2)
    )

    lines = [f"# sql-cli vs {ref_name}", "", "| bucket | count |", "|---|---|"]
    for b in ("AGREE", "DIFFER", "GAP", "OURS_ONLY", "BOTH_ERR"):
        if b in counts:
            lines.append(f"| {b} | {counts[b]} |")
    gaps = [r for r in rows if r["bucket"] == "GAP"]
    differs = [r for r in rows if r["bucket"] == "DIFFER"]
    if gaps:
        lines += ["", "## Gaps (reference runs, sql-cli errors)", ""]
        for r in gaps:
            lines.append(f"- **{r['id']}** ({r['tier']}): `{r['sql']}` — {r['detail']}")
    if differs:
        lines += ["", "## Differs (both run, results disagree)", ""]
        for r in differs:
            first = str(r["detail"]).splitlines()[0] if r["detail"] else ""
            lines.append(f"- **{r['id']}** ({r['tier']}): `{r['sql']}` — {first}")
    (REPORT_DIR / f"compare_{ref_name}.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
