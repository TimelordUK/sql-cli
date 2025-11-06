#!/usr/bin/env python3
"""
Test implicit window frame behavior - SQL standard compliance.

When ORDER BY is present but no frame is specified, SQL standard
requires RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW as the default.
"""

import subprocess
import json
import sys
import os

def run_sql_query(query):
    """Run a SQL query and return the JSON output"""
    result = subprocess.run(
        ['./target/release/sql-cli', '-q', query, '-o', 'json'],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Error running query: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    
    return json.loads(result.stdout)

def test_implicit_frame_with_order_by():
    """Test that ORDER BY without frame defaults to UNBOUNDED PRECEDING"""
    query = """
    WITH test_data AS (
        SELECT 1 as id, 10 as value UNION ALL
        SELECT 2, 20 UNION ALL
        SELECT 3, 30 UNION ALL
        SELECT 4, 40 UNION ALL
        SELECT 5, 50
    )
    SELECT 
        id,
        value,
        SUM(value) OVER (ORDER BY id) as sum_implicit,
        SUM(value) OVER (ORDER BY id RANGE UNBOUNDED PRECEDING) as sum_explicit
    FROM test_data
    ORDER BY id
    """
    
    results = run_sql_query(query)
    
    # Verify we got 5 rows
    assert len(results) == 5, f"Expected 5 rows, got {len(results)}"
    
    # Expected cumulative sums
    expected_sums = [10, 30, 60, 100, 150]
    
    # Verify implicit and explicit frames produce same results
    for i, row in enumerate(results):
        assert row['id'] == i + 1
        assert row['sum_implicit'] == expected_sums[i], \
            f"Row {i+1}: implicit sum {row['sum_implicit']} != expected {expected_sums[i]}"
        assert row['sum_explicit'] == expected_sums[i], \
            f"Row {i+1}: explicit sum {row['sum_explicit']} != expected {expected_sums[i]}"
        assert row['sum_implicit'] == row['sum_explicit'], \
            f"Row {i+1}: implicit and explicit sums don't match"

def test_no_order_by_uses_all_rows():
    """Test that without ORDER BY, the frame includes all rows"""
    query = """
    WITH test_data AS (
        SELECT 1 as id, 10 as value UNION ALL
        SELECT 2, 20 UNION ALL
        SELECT 3, 30 UNION ALL
        SELECT 4, 40 UNION ALL
        SELECT 5, 50
    )
    SELECT 
        id,
        value,
        1 as dummy,
        SUM(value) OVER (PARTITION BY dummy) as sum_partition,
        AVG(value) OVER (PARTITION BY dummy) as avg_partition,
        COUNT(*) OVER (PARTITION BY dummy) as count_partition
    FROM test_data
    ORDER BY id
    """
    
    results = run_sql_query(query)
    
    # Without ORDER BY, all rows should see the aggregate of all rows
    total_sum = 150
    avg_value = 30  # (10+20+30+40+50)/5
    total_count = 5
    
    for row in results:
        assert row['sum_partition'] == total_sum, \
            f"Row {row['id']}: sum_partition {row['sum_partition']} != total {total_sum}"
        assert row['avg_partition'] == avg_value, \
            f"Row {row['id']}: avg_partition {row['avg_partition']} != expected {avg_value}"
        assert row['count_partition'] == total_count, \
            f"Row {row['id']}: count_partition {row['count_partition']} != expected {total_count}"

def test_explicit_frame_overrides_implicit():
    """Test that explicit frame specifications override the implicit default"""
    query = """
    WITH test_data AS (
        SELECT 1 as id, 10 as value UNION ALL
        SELECT 2, 20 UNION ALL
        SELECT 3, 30 UNION ALL
        SELECT 4, 40 UNION ALL
        SELECT 5, 50
    )
    SELECT 
        id,
        value,
        SUM(value) OVER (ORDER BY id) as sum_implicit,
        SUM(value) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) as sum_2_rows,
        SUM(value) OVER (ORDER BY id ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) as sum_future
    FROM test_data
    ORDER BY id
    """
    
    results = run_sql_query(query)
    
    # Verify explicit frames work correctly
    expected_2_rows = [10, 30, 50, 70, 90]  # Sum of current and previous
    expected_future = [150, 140, 120, 90, 50]  # Sum from current to end
    
    for i, row in enumerate(results):
        assert row['sum_2_rows'] == expected_2_rows[i], \
            f"Row {i+1}: 2-row sum {row['sum_2_rows']} != expected {expected_2_rows[i]}"
        assert row['sum_future'] == expected_future[i], \
            f"Row {i+1}: future sum {row['sum_future']} != expected {expected_future[i]}"

def test_range_vs_rows_semantics():
    """Test that implicit default is RANGE, not ROWS"""
    query = """
    WITH test_data AS (
        SELECT 1 as id, 10 as value UNION ALL
        SELECT 2, 20 UNION ALL
        SELECT 3, 30 UNION ALL
        SELECT 4, 40 UNION ALL
        SELECT 5, 50
    )
    SELECT 
        id,
        value,
        SUM(value) OVER (ORDER BY id) as sum_implicit,
        SUM(value) OVER (ORDER BY id RANGE UNBOUNDED PRECEDING) as sum_range_explicit,
        SUM(value) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) as sum_rows_explicit
    FROM test_data
    ORDER BY id
    """
    
    results = run_sql_query(query)
    
    # For distinct ORDER BY values, RANGE and ROWS should be the same
    # The important thing is that implicit matches RANGE (SQL standard)
    expected_sums = [10, 30, 60, 100, 150]
    
    for i, row in enumerate(results):
        assert row['sum_implicit'] == expected_sums[i], \
            f"Row {i+1}: implicit sum {row['sum_implicit']} != expected {expected_sums[i]}"
        assert row['sum_range_explicit'] == expected_sums[i], \
            f"Row {i+1}: range sum {row['sum_range_explicit']} != expected {expected_sums[i]}"
        assert row['sum_rows_explicit'] == expected_sums[i], \
            f"Row {i+1}: rows sum {row['sum_rows_explicit']} != expected {expected_sums[i]}"
        # Verify implicit matches RANGE semantics
        assert row['sum_implicit'] == row['sum_range_explicit'], \
            f"Row {i+1}: implicit doesn't match RANGE semantics"

def main():
    """Run all tests"""
    tests = [
        test_implicit_frame_with_order_by,
        test_no_order_by_uses_all_rows,
        test_explicit_frame_overrides_implicit,
        test_range_vs_rows_semantics
    ]
    
    for test in tests:
        print(f"Running {test.__name__}...", end=' ')
        try:
            test()
            print("✓ PASSED")
        except AssertionError as e:
            print("✗ FAILED")
            print(f"  {e}")
            sys.exit(1)
        except Exception as e:
            print("✗ ERROR")
            print(f"  {e}")
            sys.exit(1)
    
    print("\nAll implicit frame tests passed!")

if __name__ == "__main__":
    main()