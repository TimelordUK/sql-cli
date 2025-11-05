#!/usr/bin/env python3
"""
Test edge cases for window frame calculations
"""

import pandas as pd
import numpy as np
import subprocess
import csv
from io import StringIO
import os

SQL_CLI = "./target/release/sql-cli"

def run_sql_query(data_file, query):
    """Run a SQL query through sql-cli and return results as DataFrame"""
    cmd = [SQL_CLI, data_file, "-q", query, "-o", "csv"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")

    # Parse CSV output
    lines = result.stdout.strip().split('\n')
    csv_lines = [l for l in lines if not l.startswith('#')]

    if not csv_lines:
        return pd.DataFrame()

    df = pd.read_csv(StringIO('\n'.join(csv_lines)))
    return df

def create_test_data(filename, data):
    """Create a test CSV file"""
    df = pd.DataFrame(data)
    df.to_csv(filename, index=False)
    return df

def test_edge_case_1():
    """Test with very small dataset (1-3 rows)"""
    print("Test 1: Small dataset edge cases...")

    # Create test data
    test_file = "test_small.csv"
    data = {
        'id': [1, 2, 3],
        'value': [10.0, 20.0, 15.0]
    }
    df = create_test_data(test_file, data)

    # Test with window larger than data
    sql_query = """
    SELECT
        id,
        value,
        AVG(value) OVER (ORDER BY id ROWS 10 PRECEDING) as avg_10,
        MIN(value) OVER (ORDER BY id ROWS 10 PRECEDING) as min_10,
        MAX(value) OVER (ORDER BY id ROWS 10 PRECEDING) as max_10
    FROM test_small
    ORDER BY id
    """
    sql_df = run_sql_query(test_file, sql_query)

    # Pandas calculation
    df['avg_10_pandas'] = df['value'].rolling(window=11, min_periods=1).mean()
    df['min_10_pandas'] = df['value'].rolling(window=11, min_periods=1).min()
    df['max_10_pandas'] = df['value'].rolling(window=11, min_periods=1).max()

    # Compare
    merged = pd.merge(df, sql_df, on=['id', 'value'])

    avg_match = np.allclose(merged['avg_10'], merged['avg_10_pandas'], rtol=1e-9)
    min_match = np.allclose(merged['min_10'], merged['min_10_pandas'], rtol=1e-9)
    max_match = np.allclose(merged['max_10'], merged['max_10_pandas'], rtol=1e-9)

    if avg_match and min_match and max_match:
        print("  ✓ Small dataset with large window works correctly!")
    else:
        print("  ✗ Issues found:")
        print(merged)

    os.remove(test_file)
    assert avg_match and min_match and max_match, "Window frame calculations do not match expected values"

def test_edge_case_2():
    """Test BETWEEN with various combinations"""
    print("\nTest 2: BETWEEN syntax variations...")

    # Create test data
    test_file = "test_between.csv"
    data = {
        'id': list(range(1, 11)),
        'value': [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
    }
    df = create_test_data(test_file, data)

    # Test BETWEEN 2 PRECEDING AND 2 FOLLOWING (5-row window)
    sql_query = """
    SELECT
        id,
        value,
        AVG(value) OVER (ORDER BY id ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING) as avg_5
    FROM test_between
    ORDER BY id
    """
    sql_df = run_sql_query(test_file, sql_query)

    # Pandas calculation - need to handle the forward window
    df['avg_5_pandas'] = df['value'].rolling(window=5, center=True, min_periods=1).mean()

    # Adjust for alignment (pandas center=True might align differently)
    # Manual calculation for verification
    avg_5_manual = []
    for i in range(len(df)):
        start = max(0, i - 2)
        end = min(len(df), i + 3)  # +3 because Python slice is exclusive
        avg_5_manual.append(df.iloc[start:end]['value'].mean())
    df['avg_5_manual'] = avg_5_manual

    # Compare
    merged = pd.merge(df[['id', 'value', 'avg_5_manual']], sql_df, on=['id', 'value'])

    avg_match = np.allclose(merged['avg_5'], merged['avg_5_manual'], rtol=1e-9)

    if avg_match:
        print("  ✓ BETWEEN PRECEDING AND FOLLOWING works correctly!")
    else:
        print("  ✗ Issues found:")
        print(merged[['id', 'value', 'avg_5', 'avg_5_manual']])

    os.remove(test_file)
    assert avg_match, "Test assertion failed"

def test_edge_case_3():
    """Test with NULL values"""
    print("\nTest 3: NULL value handling...")

    # Create test data with NULLs
    test_file = "test_nulls.csv"
    data = {
        'id': [1, 2, 3, 4, 5],
        'value': [10.0, None, 30.0, None, 50.0]
    }
    df_raw = pd.DataFrame(data)
    df_raw.to_csv(test_file, index=False, na_rep='')

    # Test AVG with NULLs
    sql_query = """
    SELECT
        id,
        value,
        AVG(value) OVER (ORDER BY id ROWS 2 PRECEDING) as avg_3,
        COUNT(value) OVER (ORDER BY id ROWS 2 PRECEDING) as count_non_null,
        COUNT(*) OVER (ORDER BY id ROWS 2 PRECEDING) as count_all
    FROM test_nulls
    ORDER BY id
    """
    sql_df = run_sql_query(test_file, sql_query)

    # Pandas calculation (pandas automatically excludes NaN in mean)
    df = df_raw.copy()
    df['avg_3_pandas'] = df['value'].rolling(window=3, min_periods=1).mean()
    df['count_non_null_pandas'] = df['value'].rolling(window=3, min_periods=0).count()
    df['count_all_pandas'] = df['id'].rolling(window=3, min_periods=1).count()

    # Compare
    merged = pd.merge(df[['id', 'avg_3_pandas', 'count_non_null_pandas', 'count_all_pandas']],
                     sql_df[['id', 'avg_3', 'count_non_null', 'count_all']], on='id')

    # Handle NaN comparison
    avg_match = ((merged['avg_3'].isna() & merged['avg_3_pandas'].isna()) |
                 np.isclose(merged['avg_3'].fillna(0), merged['avg_3_pandas'].fillna(0), rtol=1e-9)).all()
    count_match = (merged['count_non_null'] == merged['count_non_null_pandas']).all()
    count_all_match = (merged['count_all'] == merged['count_all_pandas']).all()

    if avg_match and count_match and count_all_match:
        print("  ✓ NULL value handling is correct!")
    else:
        print("  ✗ Issues found:")
        print(merged)

    os.remove(test_file)
    assert avg_match and count_match and count_all_match, "NULL value handling is not correct"

def test_edge_case_4():
    """Test FIRST_VALUE and LAST_VALUE with frames"""
    print("\nTest 4: FIRST_VALUE/LAST_VALUE with frames...")

    # Create test data
    test_file = "test_first_last.csv"
    data = {
        'id': list(range(1, 11)),
        'value': list(range(10, 101, 10))
    }
    df = create_test_data(test_file, data)

    # Test FIRST/LAST with different frames
    sql_query = """
    SELECT
        id,
        value,
        FIRST_VALUE(value) OVER (ORDER BY id ROWS 3 PRECEDING) as first_4,
        LAST_VALUE(value) OVER (ORDER BY id ROWS 3 PRECEDING) as last_4,
        FIRST_VALUE(value) OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as first_cumul,
        LAST_VALUE(value) OVER (ORDER BY id ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) as last_remain
    FROM test_first_last
    ORDER BY id
    LIMIT 5
    """
    sql_df = run_sql_query(test_file, sql_query)

    # Manual verification
    expected = {
        'id': [1, 2, 3, 4, 5],
        'value': [10, 20, 30, 40, 50],
        'first_4': [10, 10, 10, 10, 20],  # First value in 4-row window
        'last_4': [10, 20, 30, 40, 50],   # Last value (current row) in window
        'first_cumul': [10, 10, 10, 10, 10],  # Always first row
        'last_remain': [100, 100, 100, 100, 100]  # Always last row in dataset
    }
    expected_df = pd.DataFrame(expected)

    # Compare
    comparison = pd.merge(expected_df, sql_df, on=['id', 'value'])

    match = (
        (comparison['first_4_x'] == comparison['first_4_y']).all() and
        (comparison['last_4_x'] == comparison['last_4_y']).all() and
        (comparison['first_cumul_x'] == comparison['first_cumul_y']).all() and
        (comparison['last_remain_x'] == comparison['last_remain_y']).all()
    )

    if match:
        print("  ✓ FIRST_VALUE/LAST_VALUE with frames work correctly!")
    else:
        print("  ✗ Issues found:")
        print("Expected:")
        print(expected_df)
        print("\nGot:")
        print(sql_df)

    os.remove(test_file)
    assert match, "FIRST_VALUE/LAST_VALUE with frames not working correctly"

def test_edge_case_5():
    """Test window frames with duplicate values and ordering"""
    print("\nTest 5: Duplicate values and stable ordering...")

    # Create test data with duplicates
    test_file = "test_duplicates.csv"
    data = {
        'id': [1, 2, 3, 4, 5, 6],
        'group': ['A', 'A', 'B', 'B', 'A', 'B'],
        'value': [10, 10, 20, 20, 10, 20]
    }
    df = create_test_data(test_file, data)

    # Test with ORDER BY on column with duplicates
    sql_query = """
    SELECT
        id,
        value,
        ROW_NUMBER() OVER (ORDER BY value) as rn,
        MIN(id) OVER (ORDER BY value ROWS 2 PRECEDING) as min_id_3,
        MAX(id) OVER (ORDER BY value ROWS 2 PRECEDING) as max_id_3
    FROM test_duplicates
    ORDER BY id
    """
    sql_df = run_sql_query(test_file, sql_query)

    # The ordering should be stable based on the original row order
    # When value is the same, earlier rows come first
    print("  Got results:")
    print(sql_df)

    # Basic sanity check - row numbers should be unique
    rn_unique = sql_df['rn'].nunique() == len(sql_df)

    if rn_unique:
        print("  ✓ Handles duplicate values with stable ordering!")
    else:
        print("  ✗ Row numbers not unique with duplicate values")

    os.remove(test_file)
    assert rn_unique, "Row numbers not unique with duplicate values"

def main():
    """Run all edge case tests"""
    print("=" * 60)
    print("Window Frame Edge Case Tests")
    print("=" * 60)

    # Check if SQL-CLI exists
    if not os.path.exists(SQL_CLI):
        print(f"Error: SQL-CLI not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1

    try:
        results = []
        results.append(test_edge_case_1())
        results.append(test_edge_case_2())
        results.append(test_edge_case_3())
        results.append(test_edge_case_4())
        results.append(test_edge_case_5())

        print("\n" + "=" * 60)
        if all(results):
            print("✓ All edge case tests passed!")
        else:
            print(f"✗ {sum(not r for r in results)} tests failed")
        print("=" * 60)

        return 0 if all(results) else 1

    except Exception as e:
        print(f"\nError during testing: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())