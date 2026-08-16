#!/usr/bin/env python3
"""Regression tests for SELECT ... INTO #tmp staging (parity P28) and for the
--limit flag (parity P31).

Both bugs had the same shape: a DataView carries the WHERE filter, the SELECT
projection and the LIMIT window, and the code that turned that view back into a
table reached past it to the underlying source table. Staging then quietly
carried rows and columns the query had excluded.

These go through the real binary because the defect lived in the script
execution path (src/non_interactive.rs), not in the engine.
"""

import csv
import os
import subprocess
import sys
import tempfile
from io import StringIO

_BIN = os.path.join(os.path.dirname(__file__), "../../target/release/sql-cli")
SQL_CLI = _BIN + ".exe" if sys.platform == "win32" else _BIN

# 12 rows; 8 have a non-NULL score.
CSV_CONTENT = """id,team,score,label
1,alpha,50,delta
2,alpha,50,
3,alpha,,charlie
4,beta,70,bravo
5,beta,30,
6,beta,70,alpha
7,,90,echo
8,,10,foxtrot
9,gamma,20,golf
10,gamma,,
11,delta,,
12,delta,,
"""


def _data_file():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".csv", delete=False) as f:
        f.write(CSV_CONTENT)
        return f.name


def run_script(statements, data_file):
    """Run SQL statements as a GO-separated script; return the last result."""
    script = "".join(f"{s};\nGO\n" for s in statements)
    with tempfile.NamedTemporaryFile(mode="w", suffix=".sql", delete=False) as f:
        f.write(script)
        script_file = f.name
    try:
        result = subprocess.run(
            [SQL_CLI, data_file, "-f", script_file, "-o", "csv"],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            raise AssertionError(f"Script failed: {result.stderr}")
        return result.stdout
    finally:
        os.unlink(script_file)


def run_query(query, data_file, extra_args=()):
    result = subprocess.run(
        [SQL_CLI, data_file, "-q", query, "-o", "csv", *extra_args],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise AssertionError(f"Query failed: {result.stderr}")
    return result.stdout


def parse_last_result(stdout):
    """Parse the final CSV block emitted by a script run."""
    blocks = [b for b in stdout.strip().split("\n\n") if b.strip()]
    rows = list(csv.DictReader(StringIO(blocks[-1])))
    return rows


def test_into_honours_where():
    """P28: the staged table must not carry rows the WHERE excluded."""
    data = _data_file()
    try:
        out = run_script(
            [
                "SELECT id, score INTO #a FROM null_edges WHERE score IS NOT NULL",
                "SELECT COUNT(*) AS n FROM #a",
            ],
            data,
        )
        rows = parse_last_result(out)
        assert rows[0]["n"] == "8", f"expected 8 staged rows, got {rows[0]['n']}"
    finally:
        os.unlink(data)


def test_into_honours_select_list():
    """P28: the staged table must have only the projected columns."""
    data = _data_file()
    try:
        out = run_script(
            [
                "SELECT id, score INTO #a FROM null_edges WHERE score IS NOT NULL",
                "SELECT * FROM #a",
            ],
            data,
        )
        rows = parse_last_result(out)
        assert len(rows) == 8, f"expected 8 rows, got {len(rows)}"
        assert list(rows[0].keys()) == ["id", "score"], (
            f"staged columns should be the projection, got {list(rows[0].keys())}"
        )
    finally:
        os.unlink(data)


def test_into_honours_limit():
    """P28: LIMIT is part of the result, so it must be part of what is staged."""
    data = _data_file()
    try:
        out = run_script(
            [
                "SELECT id INTO #a FROM null_edges LIMIT 3",
                "SELECT COUNT(*) AS n FROM #a",
            ],
            data,
        )
        rows = parse_last_result(out)
        assert rows[0]["n"] == "3", f"expected 3 staged rows, got {rows[0]['n']}"
    finally:
        os.unlink(data)


def test_into_after_clauses_honours_where():
    """P28: the trailing INTO placement takes the same path."""
    data = _data_file()
    try:
        out = run_script(
            [
                "SELECT id, score FROM null_edges WHERE score IS NOT NULL INTO #b",
                "SELECT COUNT(*) AS n FROM #b",
            ],
            data,
        )
        rows = parse_last_result(out)
        assert rows[0]["n"] == "8", f"expected 8 staged rows, got {rows[0]['n']}"
    finally:
        os.unlink(data)


def test_into_preserves_order_by():
    """Control: staging must keep the ordering the query asked for."""
    data = _data_file()
    try:
        out = run_script(
            [
                "SELECT id, score INTO #c FROM null_edges "
                "WHERE score > 40 ORDER BY score DESC",
                "SELECT * FROM #c",
            ],
            data,
        )
        rows = parse_last_result(out)
        scores = [r["score"] for r in rows]
        assert scores == sorted(scores, key=int, reverse=True), (
            f"staged rows lost their ordering: {scores}"
        )
    finally:
        os.unlink(data)


def test_into_star_stages_everything():
    """Control: SELECT * INTO still stages all columns and all matching rows."""
    data = _data_file()
    try:
        out = run_script(
            ["SELECT * INTO #d FROM null_edges", "SELECT * FROM #d"], data
        )
        rows = parse_last_result(out)
        assert len(rows) == 12
        assert list(rows[0].keys()) == ["id", "team", "score", "label"]
    finally:
        os.unlink(data)


def test_cli_limit_with_projection():
    """P31: --limit with a SELECT list used to return 0 rows under the source's
    headers, because the limited table was built from the source's columns but
    the projection's row values."""
    data = _data_file()
    try:
        out = run_query(
            "SELECT id, score FROM null_edges WHERE score > 40", data, ("--limit", "2")
        )
        rows = list(csv.DictReader(StringIO(out)))
        assert len(rows) == 2, f"expected 2 rows, got {len(rows)}"
        assert list(rows[0].keys()) == ["id", "score"], (
            f"expected the projected columns, got {list(rows[0].keys())}"
        )
    finally:
        os.unlink(data)


def test_cli_limit_does_not_widen_sql_limit():
    """Control: the tighter of SQL LIMIT and --limit wins."""
    data = _data_file()
    try:
        out = run_query("SELECT id FROM null_edges LIMIT 2", data, ("--limit", "5"))
        rows = list(csv.DictReader(StringIO(out)))
        assert len(rows) == 2, f"SQL LIMIT 2 should win over --limit 5, got {len(rows)}"
    finally:
        os.unlink(data)


def test_cli_limit_star():
    """Control: SELECT * with --limit was always correct and must stay so."""
    data = _data_file()
    try:
        out = run_query("SELECT * FROM null_edges", data, ("--limit", "3"))
        rows = list(csv.DictReader(StringIO(out)))
        assert len(rows) == 3
        assert list(rows[0].keys()) == ["id", "team", "score", "label"]
    finally:
        os.unlink(data)
