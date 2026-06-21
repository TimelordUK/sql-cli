"""Cross-engine result normalization and comparison.

Different SQL engines disagree on harmless surface details (int vs float, NULL
vs empty string, date objects vs ISO strings, row order). This module canon-
icalizes both result sets so the comparison surfaces *semantic* differences,
not formatting noise. The rules are intentionally documented and lenient --
a "DIFFER" verdict should mean a real disagreement, not a typing quirk.

Normalization rules
-------------------
- Column names      : compared case-insensitively, whitespace-trimmed.
- NULL == ""        : empty strings are treated as NULL (sql-cli loads empty
                      CSV fields as NULL; DuckDB does the same for typed cols).
- Numbers           : int/float/Decimal and numeric-looking strings collapse to
                      float rounded to FLOAT_TOL decimals (so 1, 1.0, "1" match).
- Booleans          : True/False and "true"/"false" collapse to "true"/"false".
- Dates / datetimes : date/datetime objects -> ISO; "T" separator normalized to
                      a space so "2025-01-01T00:00:00" == "2025-01-01 00:00:00".
- Row order         : ignored (multiset compare) unless the query has ORDER BY.
"""

from __future__ import annotations

import datetime as _dt
import re
from collections import Counter
from decimal import Decimal
from typing import Optional

FLOAT_TOL = 6

# Fractional seconds in a datetime ("00.123000" -> "00.123", "00.000" -> "00"):
# engines emit millisecond vs microsecond precision for the same instant.
_FRAC_SECONDS = re.compile(r"(:\d{2})\.(\d+)")


def _strip_frac(m: "re.Match") -> str:
    frac = m.group(2).rstrip("0")
    return m.group(1) + ("." + frac if frac else "")


def canon_scalar(v):
    """Reduce a value to a hashable canonical form for cross-engine comparison."""
    if v is None:
        return None
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float, Decimal)):
        return round(float(v), FLOAT_TOL)
    if isinstance(v, _dt.datetime):
        return _FRAC_SECONDS.sub(_strip_frac, v.isoformat(sep=" "))
    if isinstance(v, _dt.date):
        return v.isoformat()
    if isinstance(v, _dt.time):
        return _FRAC_SECONDS.sub(_strip_frac, v.isoformat())

    s = str(v).strip()
    if s == "":
        return None
    low = s.lower()
    if low in ("true", "false"):
        return low
    try:
        return round(float(s), FLOAT_TOL)
    except ValueError:
        pass
    return _FRAC_SECONDS.sub(_strip_frac, s.replace("T", " "))


def canon_row(row: dict) -> dict:
    return {str(k).strip().lower(): canon_scalar(v) for k, v in row.items()}


def _col_set(rows: list[dict]) -> set:
    cols: set = set()
    for r in rows:
        cols.update(r.keys())
    return cols


def _fmt(rows, cols, limit=3) -> str:
    shown = [dict(zip(cols, r)) for r in rows[:limit]]
    extra = "" if len(rows) <= limit else f" (+{len(rows) - limit} more)"
    return "; ".join(str(s) for s in shown) + extra


def has_order_by(sql: str) -> bool:
    return "order by" in sql.lower()


def compare(sqlcli_rows: list[dict], ref_rows: list[dict], ordered: bool) -> tuple[bool, Optional[str]]:
    """Return (matches, diff_message). diff_message is None on a match."""
    a = [canon_row(r) for r in sqlcli_rows]
    b = [canon_row(r) for r in ref_rows]

    ca, cb = _col_set(a), _col_set(b)
    if ca != cb:
        only_a = sorted(ca - cb)
        only_b = sorted(cb - ca)
        parts = []
        if only_a:
            parts.append(f"sql-cli-only cols={only_a}")
        if only_b:
            parts.append(f"ref-only cols={only_b}")
        return False, "column mismatch: " + ", ".join(parts) + " (alias computed columns to align)"

    cols = sorted(ca)
    ta = [tuple(r.get(c) for c in cols) for r in a]
    tb = [tuple(r.get(c) for c in cols) for r in b]

    if ordered:
        if ta == tb:
            return True, None
        msg = [f"rows: sql-cli={len(ta)} ref={len(tb)} (ordered)"]
        for i, (x, y) in enumerate(zip(ta, tb)):
            if x != y:
                msg.append(f"  first diff at row {i}:")
                msg.append(f"    sql-cli: {dict(zip(cols, x))}")
                msg.append(f"    ref:     {dict(zip(cols, y))}")
                break
        return False, "\n".join(msg)

    ca_count, cb_count = Counter(ta), Counter(tb)
    if ca_count == cb_count:
        return True, None
    only_a = list((ca_count - cb_count).elements())
    only_b = list((cb_count - ca_count).elements())
    msg = [f"rows: sql-cli={len(ta)} ref={len(tb)}"]
    if only_a:
        msg.append(f"  only in sql-cli ({len(only_a)}): {_fmt(only_a, cols)}")
    if only_b:
        msg.append(f"  only in ref ({len(only_b)}): {_fmt(only_b, cols)}")
    return False, "\n".join(msg)
