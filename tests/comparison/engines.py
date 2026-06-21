"""Engine adapters for the differential SQL comparison harness.

Each adapter exposes the same `run(data_file, table, sql) -> EngineResult`
interface so a reference engine (DuckDB today, SQLite / SQL Server later) is a
drop-in. The corpus SQL always refers to its source by the file *stem* as the
table name (this matches how sql-cli maps a loaded file to a table), so every
adapter loads `data_file` into a table called `table` before running `sql`.
"""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

PROJECT_ROOT = Path(__file__).resolve().parents[2]
SQL_CLI = PROJECT_ROOT / "target" / "release" / "sql-cli"
DATA_DIR = PROJECT_ROOT / "data"


@dataclass
class EngineResult:
    """Outcome of running one query on one engine."""

    ok: bool
    rows: Optional[list[dict]] = None  # list of {column_name: value}
    error: Optional[str] = None


class Engine:
    name = "engine"

    def run(self, data_file: str, table: str, sql: str) -> EngineResult:  # pragma: no cover
        raise NotImplementedError


class SqlCliEngine(Engine):
    """Runs queries through the release sql-cli binary in JSON output mode."""

    name = "sql-cli"

    def __init__(self, binary: Path = SQL_CLI):
        self.binary = binary

    def run(self, data_file: str, table: str, sql: str) -> EngineResult:
        path = DATA_DIR / data_file
        cmd = [str(self.binary), str(path), "-q", sql, "-o", "json"]
        try:
            proc = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        except subprocess.TimeoutExpired:
            return EngineResult(False, error="timeout (30s)")

        if proc.returncode != 0:
            err = (proc.stderr or proc.stdout).strip()
            last = err.splitlines()[-1] if err else f"exit code {proc.returncode}"
            return EngineResult(False, error=last)

        out = proc.stdout.strip()
        if not out:
            return EngineResult(True, rows=[])
        try:
            data = json.loads(out)
        except json.JSONDecodeError as exc:
            return EngineResult(False, error=f"invalid JSON output: {exc}")
        if isinstance(data, dict):
            data = [data]
        if not isinstance(data, list):
            return EngineResult(False, error=f"unexpected JSON shape: {type(data).__name__}")
        return EngineResult(True, rows=data)


class DuckDBEngine(Engine):
    """Reference engine: loads the data file in-process and runs the query."""

    name = "duckdb"

    def run(self, data_file: str, table: str, sql: str) -> EngineResult:
        import duckdb

        path = DATA_DIR / data_file
        loc = str(path).replace("'", "''")
        con = duckdb.connect()
        try:
            if path.suffix == ".json":
                con.execute(f'CREATE TABLE "{table}" AS SELECT * FROM read_json_auto(\'{loc}\')')
            else:
                con.execute(f'CREATE TABLE "{table}" AS SELECT * FROM read_csv_auto(\'{loc}\', header=true)')
            cur = con.execute(sql)
            cols = [d[0] for d in cur.description]
            rows = [dict(zip(cols, r)) for r in cur.fetchall()]
            return EngineResult(True, rows=rows)
        except Exception as exc:  # noqa: BLE001 - any engine error is a result, not a crash
            msg = str(exc).strip().splitlines()[0]
            return EngineResult(False, error=msg)
        finally:
            con.close()


REFERENCE_ENGINES = {
    "duckdb": DuckDBEngine,
    # "sqlite": SqliteEngine,   # future
    # "sqlserver": SqlServerEngine,
}
