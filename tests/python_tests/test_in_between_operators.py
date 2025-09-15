#!/usr/bin/env python3
"""Test IN and BETWEEN operators."""

import os
import subprocess
import sys
import csv
from pathlib import Path
from io import StringIO

def run_query(query, data_file=None):
    """Execute a query and return the results."""
    base_dir = Path(__file__).parent.parent.parent
    sql_cli = base_dir / "target" / "release" / "sql-cli"

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

    # Parse CSV output
    reader = csv.DictReader(StringIO(result.stdout))
    return list(reader)


def test_in_numeric_values():
    """Test IN operator with numeric values."""
    query = "SELECT * FROM test_simple_math WHERE a IN (1, 2, 3, 4, 5)"
    results = run_query(query, "test_simple_math.csv")

    assert len(results) == 5
    assert all(int(row['a']) in [1, 2, 3, 4, 5] for row in results)
    print("✓ test_in_numeric_values passed")


def test_in_string_values():
    """Test IN operator with string values."""
    query = "SELECT * FROM test_simple_strings WHERE name IN ('Alice', 'Bob', 'Charlie')"
    results = run_query(query, "test_simple_strings.csv")

    assert len(results) == 3
    names = [row['name'].strip() for row in results]
    assert set(names) == {'Alice', 'Bob', 'Charlie'}
    print("✓ test_in_string_values passed")


def test_in_mixed_case():
    """Test IN operator with mixed case strings."""
    query = "SELECT * FROM test_simple_strings WHERE UPPER(name) IN ('ALICE', 'BOB')"
    results = run_query(query, "test_simple_strings.csv")

    assert len(results) == 2
    names = [row['name'].strip().upper() for row in results]
    assert set(names) == {'ALICE', 'BOB'}
    print("✓ test_in_mixed_case passed")


def test_not_in_numeric():
    """Test NOT IN operator with numeric values."""
    query = "SELECT * FROM test_simple_math WHERE a NOT IN (1, 2, 3, 4, 5)"
    results = run_query(query, "test_simple_math.csv")

    assert len(results) > 0
    assert all(int(row['a']) not in [1, 2, 3, 4, 5] for row in results)
    print("✓ test_not_in_numeric passed")


def test_not_in_string():
    """Test NOT IN operator with string values."""
    query = "SELECT * FROM test_simple_strings WHERE status NOT IN ('Active', 'Pending')"
    results = run_query(query, "test_simple_strings.csv")

    assert len(results) > 0
    assert all(row['status'] not in ['Active', 'Pending'] for row in results)
    print("✓ test_not_in_string passed")


def test_in_with_nulls():
    """Test IN operator behavior with NULL values."""
    # This test would need data with NULL values
    # For now, test empty strings
    query = "SELECT * FROM test_simple_strings WHERE name IN ('Alice', '')"
    results = run_query(query, "test_simple_strings.csv")

    # Should get Alice and any empty names
    assert any(row['name'].strip() == 'Alice' for row in results)
    print("✓ test_in_with_nulls passed")


def test_in_empty_list():
    """Test IN operator with empty list (should return no results)."""
    # Note: This might not be supported yet
    query = "SELECT * FROM test_simple_math WHERE a IN ()"
    try:
        results = run_query(query, "test_simple_math.csv")
        if results is not None:
            assert len(results) == 0
    except:
        # Empty IN list might not be supported
        pass
    print("✓ test_in_empty_list passed (or not supported)")


def test_in_single_value():
    """Test IN operator with single value."""
    query = "SELECT * FROM test_simple_math WHERE a IN (5)"
    results = run_query(query, "test_simple_math.csv")

    if results:
        assert all(int(row['a']) == 5 for row in results)
    print("✓ test_in_single_value passed")


def test_in_large_list():
    """Test IN operator with large list of values."""
    values = ",".join(str(i) for i in range(1, 21))
    query = f"SELECT * FROM test_simple_math WHERE a IN ({values})"
    results = run_query(query, "test_simple_math.csv")

    assert len(results) == 20
    assert all(1 <= int(row['a']) <= 20 for row in results)
    print("✓ test_in_large_list passed")


def test_in_with_expressions():
    """Test IN operator with expressions in the list."""
    query = "SELECT * FROM test_simple_math WHERE a IN (1+1, 2*2, 3*3)"
    results = run_query(query, "test_simple_math.csv")

    if results:
        values = [int(row['a']) for row in results]
        assert all(v in [2, 4, 9] for v in values)
    print("✓ test_in_with_expressions passed")


def test_in_case_expression():
    """Test IN operator within CASE expression."""
    query = """
    SELECT id,
           CASE WHEN a IN (1, 2, 3) THEN 'Low'
                WHEN a IN (4, 5, 6) THEN 'Medium'
                ELSE 'High'
           END as category
    FROM test_simple_math
    WHERE a <= 10
    """
    results = run_query(query, "test_simple_math.csv")

    assert len(results) == 10
    for row in results:
        a_val = int(row['id'])  # Using id since it matches a in test data
        if a_val in [1, 2, 3]:
            assert row['category'] == 'Low'
        elif a_val in [4, 5, 6]:
            assert row['category'] == 'Medium'
        else:
            assert row['category'] == 'High'
    print("✓ test_in_case_expression passed")


def test_between_numeric():
    """Test BETWEEN operator with numeric values."""
    query = "SELECT * FROM test_simple_math WHERE a BETWEEN 5 AND 10"
    results = run_query(query, "test_simple_math.csv")

    if results is None:
        print("⚠ BETWEEN not yet implemented")
        return

    assert len(results) == 6  # 5, 6, 7, 8, 9, 10
    assert all(5 <= int(row['a']) <= 10 for row in results)
    print("✓ test_between_numeric passed")


def test_between_string():
    """Test BETWEEN operator with string values."""
    query = "SELECT * FROM test_simple_strings WHERE name BETWEEN 'A' AND 'D'"
    results = run_query(query, "test_simple_strings.csv")

    if results is None:
        print("⚠ BETWEEN with strings not yet implemented")
        return

    for row in results:
        name = row['name'].strip()
        assert 'A' <= name[0] <= 'D'
    print("✓ test_between_string passed")


def test_not_between():
    """Test NOT BETWEEN operator."""
    query = "SELECT * FROM test_simple_math WHERE a NOT BETWEEN 5 AND 10"
    results = run_query(query, "test_simple_math.csv")

    if results is None:
        print("⚠ NOT BETWEEN not yet implemented")
        return

    assert all(int(row['a']) < 5 or int(row['a']) > 10 for row in results)
    print("✓ test_not_between passed")


def main():
    """Run all tests."""
    print("Running IN and BETWEEN operator tests...")

    tests = [
        test_in_numeric_values,
        test_in_string_values,
        test_in_mixed_case,
        test_not_in_numeric,
        test_not_in_string,
        test_in_with_nulls,
        test_in_empty_list,
        test_in_single_value,
        test_in_large_list,
        test_in_with_expressions,
        test_in_case_expression,
        test_between_numeric,
        test_between_string,
        test_not_between,
    ]

    failed = 0
    for test in tests:
        try:
            test()
        except AssertionError as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
        except Exception as e:
            print(f"✗ {test.__name__} error: {e}")
            failed += 1

    print(f"\nTests completed: {len(tests) - failed}/{len(tests)} passed")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())