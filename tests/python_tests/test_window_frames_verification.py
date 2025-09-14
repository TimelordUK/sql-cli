#!/usr/bin/env python3
"""
Verify window frame calculations against pandas/numpy
"""

import pandas as pd
import numpy as np
import subprocess
import json
import csv
from io import StringIO
import os

# Path to AAPL data
DATA_FILE = "data/AAPL_data.csv"
SQL_CLI = "./target/release/sql-cli"

def run_sql_query(query):
    """Run a SQL query through sql-cli and return results as DataFrame"""
    cmd = [SQL_CLI, DATA_FILE, "-q", query, "-o", "csv"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")

    # Parse CSV output (skip the timing comment at the end)
    lines = result.stdout.strip().split('\n')
    csv_lines = [l for l in lines if not l.startswith('#')]

    df = pd.read_csv(StringIO('\n'.join(csv_lines)))
    return df

def test_moving_average():
    """Test moving average calculations"""
    print("Testing Moving Average (5-day)...")

    # Load data in pandas
    df = pd.read_csv(DATA_FILE)
    df = df.sort_values('date')

    # Calculate 5-day moving average in pandas (current + 4 preceding = 5 total)
    df['ma_5day_pandas'] = df['close'].rolling(window=5, min_periods=1).mean()

    # Get SQL-CLI results
    sql_query = """
    SELECT
        date,
        close,
        AVG(close) OVER (ORDER BY date ROWS 4 PRECEDING) as ma_5day
    FROM AAPL_data
    ORDER BY date
    LIMIT 20
    """
    sql_df = run_sql_query(sql_query)

    # Merge and compare
    comparison = pd.merge(
        df[['date', 'close', 'ma_5day_pandas']].head(20),
        sql_df,
        on=['date', 'close']
    )

    # Check if values match (within floating point tolerance)
    comparison['diff'] = abs(comparison['ma_5day'] - comparison['ma_5day_pandas'])
    max_diff = comparison['diff'].max()

    print(f"  Max difference: {max_diff:.10f}")
    if max_diff < 0.0001:
        print("  ✓ Moving average matches pandas!")
    else:
        print("  ✗ Moving average has discrepancies:")
        print(comparison[comparison['diff'] > 0.0001])

    return comparison

def test_min_max_window():
    """Test MIN/MAX over window frames"""
    print("\nTesting MIN/MAX over 3-day window...")

    # Load data in pandas
    df = pd.read_csv(DATA_FILE)
    df = df.sort_values('date')

    # Calculate rolling min/max in pandas (current + 2 preceding = 3 total)
    df['min_3day_pandas'] = df['close'].rolling(window=3, min_periods=1).min()
    df['max_3day_pandas'] = df['close'].rolling(window=3, min_periods=1).max()

    # Get SQL-CLI results
    sql_query = """
    SELECT
        date,
        close,
        MIN(close) OVER (ORDER BY date ROWS 2 PRECEDING) as min_3day,
        MAX(close) OVER (ORDER BY date ROWS 2 PRECEDING) as max_3day
    FROM AAPL_data
    ORDER BY date
    LIMIT 20
    """
    sql_df = run_sql_query(sql_query)

    # Merge and compare
    comparison = pd.merge(
        df[['date', 'close', 'min_3day_pandas', 'max_3day_pandas']].head(20),
        sql_df,
        on=['date', 'close']
    )

    # Check MIN
    comparison['min_diff'] = abs(comparison['min_3day'] - comparison['min_3day_pandas'])
    max_min_diff = comparison['min_diff'].max()

    # Check MAX
    comparison['max_diff'] = abs(comparison['max_3day'] - comparison['max_3day_pandas'])
    max_max_diff = comparison['max_diff'].max()

    print(f"  MIN max difference: {max_min_diff:.10f}")
    print(f"  MAX max difference: {max_max_diff:.10f}")

    if max_min_diff < 0.0001 and max_max_diff < 0.0001:
        print("  ✓ MIN/MAX window functions match pandas!")
    else:
        print("  ✗ MIN/MAX has discrepancies:")
        if max_min_diff >= 0.0001:
            print("  MIN issues:")
            print(comparison[comparison['min_diff'] > 0.0001][['date', 'close', 'min_3day', 'min_3day_pandas']])
        if max_max_diff >= 0.0001:
            print("  MAX issues:")
            print(comparison[comparison['max_diff'] > 0.0001][['date', 'close', 'max_3day', 'max_3day_pandas']])

    return comparison

def test_unbounded_preceding():
    """Test UNBOUNDED PRECEDING (cumulative calculations)"""
    print("\nTesting UNBOUNDED PRECEDING (cumulative min/max)...")

    # Load data in pandas
    df = pd.read_csv(DATA_FILE)
    df = df.sort_values('date')

    # Calculate cumulative min/max in pandas
    df['min_cumulative_pandas'] = df['close'].expanding().min()
    df['max_cumulative_pandas'] = df['close'].expanding().max()
    df['avg_cumulative_pandas'] = df['close'].expanding().mean()

    # Get SQL-CLI results
    sql_query = """
    SELECT
        date,
        close,
        MIN(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as min_cumulative,
        MAX(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as max_cumulative,
        AVG(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as avg_cumulative
    FROM AAPL_data
    ORDER BY date
    LIMIT 20
    """
    sql_df = run_sql_query(sql_query)

    # Merge and compare
    comparison = pd.merge(
        df[['date', 'close', 'min_cumulative_pandas', 'max_cumulative_pandas', 'avg_cumulative_pandas']].head(20),
        sql_df,
        on=['date', 'close']
    )

    # Check differences
    comparison['min_diff'] = abs(comparison['min_cumulative'] - comparison['min_cumulative_pandas'])
    comparison['max_diff'] = abs(comparison['max_cumulative'] - comparison['max_cumulative_pandas'])
    comparison['avg_diff'] = abs(comparison['avg_cumulative'] - comparison['avg_cumulative_pandas'])

    max_min_diff = comparison['min_diff'].max()
    max_max_diff = comparison['max_diff'].max()
    max_avg_diff = comparison['avg_diff'].max()

    print(f"  MIN max difference: {max_min_diff:.10f}")
    print(f"  MAX max difference: {max_max_diff:.10f}")
    print(f"  AVG max difference: {max_avg_diff:.10f}")

    if max_min_diff < 0.0001 and max_max_diff < 0.0001 and max_avg_diff < 0.0001:
        print("  ✓ UNBOUNDED PRECEDING calculations match pandas!")
    else:
        print("  ✗ UNBOUNDED PRECEDING has discrepancies:")
        if max_min_diff >= 0.0001:
            print("  MIN issues:")
            print(comparison[comparison['min_diff'] > 0.0001][['date', 'min_cumulative', 'min_cumulative_pandas']])
        if max_max_diff >= 0.0001:
            print("  MAX issues:")
            print(comparison[comparison['max_diff'] > 0.0001][['date', 'max_cumulative', 'max_cumulative_pandas']])
        if max_avg_diff >= 0.0001:
            print("  AVG issues:")
            print(comparison[comparison['avg_diff'] > 0.0001][['date', 'avg_cumulative', 'avg_cumulative_pandas']])

    return comparison

def test_sum_window():
    """Test SUM over window frames"""
    print("\nTesting SUM over 5-day window...")

    # Load data in pandas
    df = pd.read_csv(DATA_FILE)
    df = df.sort_values('date')

    # Calculate rolling sum in pandas
    df['sum_5day_pandas'] = df['close'].rolling(window=5, min_periods=1).sum()

    # Get SQL-CLI results
    sql_query = """
    SELECT
        date,
        close,
        SUM(close) OVER (ORDER BY date ROWS 4 PRECEDING) as sum_5day
    FROM AAPL_data
    ORDER BY date
    LIMIT 20
    """
    sql_df = run_sql_query(sql_query)

    # Merge and compare
    comparison = pd.merge(
        df[['date', 'close', 'sum_5day_pandas']].head(20),
        sql_df,
        on=['date', 'close']
    )

    # Check differences
    comparison['diff'] = abs(comparison['sum_5day'] - comparison['sum_5day_pandas'])
    max_diff = comparison['diff'].max()

    print(f"  Max difference: {max_diff:.10f}")
    if max_diff < 0.0001:
        print("  ✓ SUM window function matches pandas!")
    else:
        print("  ✗ SUM has discrepancies:")
        print(comparison[comparison['diff'] > 0.0001])

    return comparison

def test_lag_lead():
    """Test LAG/LEAD functions"""
    print("\nTesting LAG/LEAD functions...")

    # Load data in pandas
    df = pd.read_csv(DATA_FILE)
    df = df.sort_values('date')

    # Calculate lag/lead in pandas
    df['lag_1_pandas'] = df['close'].shift(1)
    df['lag_5_pandas'] = df['close'].shift(5)
    df['lead_1_pandas'] = df['close'].shift(-1)

    # Get SQL-CLI results
    sql_query = """
    SELECT
        date,
        close,
        LAG(close, 1) OVER (ORDER BY date) as lag_1,
        LAG(close, 5) OVER (ORDER BY date) as lag_5,
        LEAD(close, 1) OVER (ORDER BY date) as lead_1
    FROM AAPL_data
    ORDER BY date
    LIMIT 20
    """
    sql_df = run_sql_query(sql_query)

    # Merge and compare
    comparison = pd.merge(
        df[['date', 'close', 'lag_1_pandas', 'lag_5_pandas', 'lead_1_pandas']].head(20),
        sql_df,
        on=['date', 'close']
    )

    # Handle NaN comparison (pandas uses NaN, SQL uses NULL which becomes NaN in pandas)
    comparison['lag_1_match'] = (
        (comparison['lag_1'].isna() & comparison['lag_1_pandas'].isna()) |
        (abs(comparison['lag_1'] - comparison['lag_1_pandas']) < 0.0001)
    )
    comparison['lag_5_match'] = (
        (comparison['lag_5'].isna() & comparison['lag_5_pandas'].isna()) |
        (abs(comparison['lag_5'] - comparison['lag_5_pandas']) < 0.0001)
    )
    comparison['lead_1_match'] = (
        (comparison['lead_1'].isna() & comparison['lead_1_pandas'].isna()) |
        (abs(comparison['lead_1'] - comparison['lead_1_pandas']) < 0.0001)
    )

    all_match = comparison['lag_1_match'].all() and comparison['lag_5_match'].all() and comparison['lead_1_match'].all()

    if all_match:
        print("  ✓ LAG/LEAD functions match pandas!")
    else:
        print("  ✗ LAG/LEAD has discrepancies:")
        if not comparison['lag_1_match'].all():
            print("  LAG(1) issues:")
            print(comparison[~comparison['lag_1_match']][['date', 'lag_1', 'lag_1_pandas']])
        if not comparison['lag_5_match'].all():
            print("  LAG(5) issues:")
            print(comparison[~comparison['lag_5_match']][['date', 'lag_5', 'lag_5_pandas']])
        if not comparison['lead_1_match'].all():
            print("  LEAD(1) issues:")
            print(comparison[~comparison['lead_1_match']][['date', 'lead_1', 'lead_1_pandas']])

    return comparison

def main():
    """Run all verification tests"""
    print("=" * 60)
    print("Window Frame Verification Tests")
    print("=" * 60)

    # Check if SQL-CLI exists
    if not os.path.exists(SQL_CLI):
        print(f"Error: SQL-CLI not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1

    # Check if data file exists
    if not os.path.exists(DATA_FILE):
        print(f"Error: Data file not found at {DATA_FILE}")
        return 1

    try:
        test_moving_average()
        test_min_max_window()
        test_unbounded_preceding()
        test_sum_window()
        test_lag_lead()

        print("\n" + "=" * 60)
        print("All verification tests completed!")
        print("=" * 60)
        return 0

    except Exception as e:
        print(f"\nError during testing: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())