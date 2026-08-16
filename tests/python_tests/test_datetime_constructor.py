#!/usr/bin/env python3
"""Test the DATETIME(year, month, day, ...) constructor.

DATETIME is lexed as a keyword (CAST(x AS DATETIME) needs the spelling
reserved), so it never reaches the parser's generic function-call arm. It used
to be assembled from NumberLiteral tokens into a parse-time-constant AST node,
which meant DATETIME(Year, Month, Day) failed to parse at all. It now lowers to
an ordinary call on the registry's DATETIME function, so its arguments are
expressions. These tests pin both halves: the literal forms must not have
changed, and the expression forms must work.
"""

import csv
import subprocess
import sys
from io import StringIO
from pathlib import Path


def run_query(query, data_file=None):
    """Execute a query and return the results as a list of dicts."""
    base_dir = Path(__file__).parent.parent.parent
    suffix = ".exe" if sys.platform == "win32" else ""
    sql_cli = base_dir / "target" / "release" / f"sql-cli{suffix}"

    if not sql_cli.exists():
        raise FileNotFoundError(f"sql-cli not found at {sql_cli}")

    cmd = [str(sql_cli)]

    if data_file:
        data_path = base_dir / "data" / data_file
        if not data_path.exists():
            raise FileNotFoundError(f"Data file not found: {data_path}")
        cmd.append(str(data_path))

    cmd.extend(["-q", query, "-o", "csv"])

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error running query: {result.stderr}")
        return None

    reader = csv.DictReader(StringIO(result.stdout))
    return list(reader)


# ---- literal forms: behaviour must be unchanged ----

def test_datetime_from_literals():
    """DATETIME with number literals still builds the same date."""
    results = run_query("SELECT DATETIME(1732, 2, 22) AS d")

    assert results is not None
    assert results[0]['d'] == '1732-02-22 00:00:00.000'
    print("✓ test_datetime_from_literals passed")


def test_datetime_with_time_components():
    """The optional hour/minute/second arguments still apply."""
    results = run_query("SELECT DATETIME(2024, 1, 15, 14, 30, 0) AS d")

    assert results is not None
    assert results[0]['d'] == '2024-01-15 14:30:00.000'
    print("✓ test_datetime_with_time_components passed")


def test_datetime_no_args_is_today():
    """DATETIME() keeps its own AST node - the registry needs 3+ args."""
    results = run_query("SELECT DATETIME() AS d")

    assert results is not None
    # Today at midnight; assert the shape rather than a moving value.
    assert results[0]['d'].endswith(' 00:00:00.000')
    assert len(results[0]['d']) == len('2026-08-16 00:00:00.000')
    print("✓ test_datetime_no_args_is_today passed")


# ---- expression forms: what the change unlocks ----

def test_datetime_from_columns():
    """The reason for the change: build a date out of three columns."""
    query = ("SELECT Name, DATETIME(Year, Month, Day) AS BirthDate "
             "FROM president_birthdays WHERE Year = 1732")
    results = run_query(query, "president_birthdays.csv")

    assert results is not None
    assert len(results) == 1
    assert results[0]['Name'] == 'George Washington'
    assert results[0]['BirthDate'] == '1732-02-22 00:00:00.000'
    print("✓ test_datetime_from_columns passed")


def test_datetime_from_arithmetic():
    """Arguments are full expressions, not just column references."""
    query = ("SELECT DATETIME(Year + 100, Month, Day) AS d "
             "FROM president_birthdays WHERE Year = 1732")
    results = run_query(query, "president_birthdays.csv")

    assert results is not None
    assert results[0]['d'] == '1832-02-22 00:00:00.000'
    print("✓ test_datetime_from_arithmetic passed")


def test_datetime_with_cast_argument():
    """A CAST inside the argument list parses (it used to fail outright)."""
    query = ("SELECT DATETIME(CAST(Year AS INTEGER), Month, Day) AS d "
             "FROM president_birthdays WHERE Year = 1732")
    results = run_query(query, "president_birthdays.csv")

    assert results is not None
    assert results[0]['d'] == '1732-02-22 00:00:00.000'
    print("✓ test_datetime_with_cast_argument passed")


def test_datetime_in_where_clause():
    """Comparing a constructed date against a literal one."""
    query = ("SELECT Name FROM president_birthdays "
             "WHERE DATETIME(Year, Month, Day) > DATETIME(1950, 1, 1)")
    results = run_query(query, "president_birthdays.csv")

    assert results is not None
    assert [row['Name'] for row in results] == ['Barack Hussein Obama']
    print("✓ test_datetime_in_where_clause passed")


def test_datetime_feeds_date_functions():
    """A constructed date is a real date, so date functions accept it."""
    query = ("SELECT DAYNAME(DATETIME(Year, Month, Day)) AS BornOn, "
             "QUARTER(DATETIME(Year, Month, Day)) AS Qtr "
             "FROM president_birthdays WHERE Year = 1732")
    results = run_query(query, "president_birthdays.csv")

    assert results is not None
    assert results[0]['BornOn'] == 'Friday'
    assert results[0]['Qtr'] == '1'
    print("✓ test_datetime_feeds_date_functions passed")


def test_datetime_propagates_null():
    """A NULL component yields NULL rather than an error.

    The extra tag column keeps the CSV row from being entirely blank, which
    csv.DictReader would otherwise skip.
    """
    query = "SELECT 'x' AS tag, DATETIME(NULL, 1, 15) AS d"
    results = run_query(query)

    assert results is not None
    assert len(results) == 1
    assert results[0]['tag'] == 'x'
    assert results[0]['d'] == ''
    print("✓ test_datetime_propagates_null passed")


def test_cast_as_datetime_still_parses():
    """The keyword is still usable as a CAST target type."""
    results = run_query("SELECT CAST('2024-01-15' AS DATETIME) AS d")

    assert results is not None
    assert results[0]['d'].startswith('2024-01-15')
    print("✓ test_cast_as_datetime_still_parses passed")


if __name__ == "__main__":
    test_datetime_from_literals()
    test_datetime_with_time_components()
    test_datetime_no_args_is_today()
    test_datetime_from_columns()
    test_datetime_from_arithmetic()
    test_datetime_with_cast_argument()
    test_datetime_in_where_clause()
    test_datetime_feeds_date_functions()
    test_datetime_propagates_null()
    test_cast_as_datetime_still_parses()
    print("\nAll DATETIME constructor tests passed!")
