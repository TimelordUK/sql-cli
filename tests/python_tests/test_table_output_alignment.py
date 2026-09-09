#!/usr/bin/env python3
"""Alignment guarantees for `-o table` output.

The renderer sizes every column, caps that size at --max-col-width (default
50), then draws a border from the capped widths. It used to print the cell
values untruncated, so any value longer than the cap overflowed its cell and
that row ran past the border — a table of long "/"-delimited names (TeamCity
project paths, say) came out ragged and unusable for pasting elsewhere.

Every line of a rendered table must therefore have the same display width.
"""

import os
import subprocess
import sys
import unicodedata
from pathlib import Path

import pytest


def _sql_cli_binary():
    """Path to the release binary, with the .exe suffix Windows needs."""
    base_dir = Path(__file__).parent.parent.parent
    suffix = ".exe" if sys.platform == "win32" else ""
    return base_dir / "target" / "release" / f"sql-cli{suffix}"


def _display_width(text):
    """Column count of a string, matching the Rust side's unicode-width use."""
    return sum(2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1 for ch in text)


def render_table(csv_text, query, extra_args=(), tmp_path=None):
    """Run a query with -o table and return the table's lines."""
    sql_cli = _sql_cli_binary()
    if not sql_cli.exists():
        pytest.skip(f"sql-cli not built at {sql_cli}")

    data_file = tmp_path / "projects.csv"
    data_file.write_text(csv_text, encoding="utf-8")

    cmd = [str(sql_cli), str(data_file), "-q", query, "-o", "table"]
    cmd.extend(str(a) for a in extra_args)

    env = dict(os.environ, PYTHONIOENCODING="utf-8")
    result = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", env=env)
    assert result.returncode == 0, f"query failed: {result.stderr}"

    # Drop the trailing "# Query completed" note and any blank lines
    return [
        line
        for line in result.stdout.splitlines()
        if line.strip() and not line.startswith("#")
    ]


def assert_lines_aligned(lines):
    widths = {_display_width(line) for line in lines}
    assert len(widths) == 1, (
        "table rows have differing widths "
        f"{sorted(widths)}:\n" + "\n".join(lines)
    )


# A TeamCity-style project path: long, "/"-delimited, well past the 50-char default cap
LONG_PATH = "xTrader2 / Trading Services / Pricing / Analytics / master"

PROJECTS_CSV = (
    "Project,DurationSecs\n"
    f'"{LONG_PATH}",120\n'
    f'"{LONG_PATH}",240\n'
    '"xTrader2 / Risk / Limits / nightly",900\n'
    "Core,50\n"
    "Core,70\n"
)


def test_long_values_do_not_overflow_the_default_cap(tmp_path):
    lines = render_table(
        PROJECTS_CSV,
        "SELECT Project, sum(DurationSecs) as total_secs, count(*) as n "
        "FROM projects GROUP BY Project ORDER BY total_secs DESC",
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)
    # The over-long value is truncated with an ellipsis rather than spilling out
    assert any("..." in line for line in lines)


def test_unlimited_width_keeps_values_intact(tmp_path):
    lines = render_table(
        PROJECTS_CSV,
        "SELECT Project, sum(DurationSecs) as total_secs FROM projects GROUP BY Project",
        extra_args=["--max-col-width", "0"],
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)
    assert any(LONG_PATH in line for line in lines), "full path should survive uncapped"


@pytest.mark.parametrize("max_width", [1, 2, 3, 4, 5, 12, 30, 57, 58, 59])
def test_alignment_holds_at_every_cap(tmp_path, max_width):
    lines = render_table(
        PROJECTS_CSV,
        "SELECT Project, sum(DurationSecs) as total_secs FROM projects GROUP BY Project",
        extra_args=["--max-col-width", max_width],
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)


def test_long_header_is_truncated_too(tmp_path):
    lines = render_table(
        PROJECTS_CSV,
        "SELECT Project, sum(DurationSecs) as total_duration_seconds_for_this_project "
        "FROM projects GROUP BY Project",
        extra_args=["--max-col-width", "20"],
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)


def test_alignment_survives_wide_and_accented_characters(tmp_path):
    csv_text = (
        "Project,DurationSecs\n"
        '"café ⚡ / Trading Services / Pricing / Analytics / master",120\n'
        "Core,50\n"
    )
    lines = render_table(
        csv_text,
        "SELECT Project, sum(DurationSecs) as total_secs FROM projects GROUP BY Project",
        extra_args=["--max-col-width", "20"],
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)


def test_markdown_style_is_also_aligned(tmp_path):
    lines = render_table(
        PROJECTS_CSV,
        "SELECT Project, sum(DurationSecs) as total_secs, count(*) as n "
        "FROM projects GROUP BY Project ORDER BY total_secs DESC",
        extra_args=["--table-style", "markdown"],
        tmp_path=tmp_path,
    )
    assert_lines_aligned(lines)
