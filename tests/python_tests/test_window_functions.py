#!/usr/bin/env python3
"""
Test suite for SQL window functions in sql-cli.

Tests LAG, LEAD, ROW_NUMBER, FIRST_VALUE, LAST_VALUE with various
PARTITION BY and ORDER BY combinations.
"""

import subprocess
import csv
import io
import json
from pathlib import Path
import tempfile
import pandas as pd

# Path to the sql-cli executable
SQL_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"

def run_query(csv_file, query):
    """Run a SQL query and return the results as a list of dictionaries."""
    result = subprocess.run(
        [str(SQL_CLI), str(csv_file), "-q", query, "-o", "csv"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse CSV output
    output_lines = result.stdout.strip().split('\n')
    # Remove comment lines
    csv_lines = [line for line in output_lines if not line.startswith('#')]
    
    if not csv_lines:
        return []
    
    reader = csv.DictReader(io.StringIO('\n'.join(csv_lines)))
    return list(reader)

def test_lag_basic():
    """Test basic LAG function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        f.write("1,100\n")
        f.write("2,200\n")
        f.write("3,300\n")
        f.write("4,400\n")
        temp_file = f.name
    
    try:
        query = "SELECT id, value, LAG(value, 1) OVER (ORDER BY id) as prev_value FROM test"
        results = run_query(temp_file, query)
        
        assert len(results) == 4
        assert results[0]['prev_value'] == ''  # NULL shown as empty string in CSV
        assert results[1]['prev_value'] == '100'
        assert results[2]['prev_value'] == '200'
        assert results[3]['prev_value'] == '300'
        print("✓ test_lag_basic passed")
    finally:
        Path(temp_file).unlink()

def test_lead_basic():
    """Test basic LEAD function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        f.write("1,100\n")
        f.write("2,200\n")
        f.write("3,300\n")
        f.write("4,400\n")
        temp_file = f.name
    
    try:
        query = "SELECT id, value, LEAD(value, 1) OVER (ORDER BY id) as next_value FROM test"
        results = run_query(temp_file, query)
        
        assert len(results) == 4
        assert results[0]['next_value'] == '200'
        assert results[1]['next_value'] == '300'
        assert results[2]['next_value'] == '400'
        assert results[3]['next_value'] == ''  # NULL
        print("✓ test_lead_basic passed")
    finally:
        Path(temp_file).unlink()

def test_row_number_basic():
    """Test basic ROW_NUMBER function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("name,score\n")
        f.write("Alice,95\n")
        f.write("Bob,87\n")
        f.write("Charlie,92\n")
        f.write("Diana,88\n")
        temp_file = f.name
    
    try:
        query = "SELECT name, score, ROW_NUMBER() OVER (ORDER BY score DESC) as rank FROM test"
        results = run_query(temp_file, query)
        
        assert len(results) == 4
        # Window functions don't change result order, just add computed columns
        # Alice has highest score (95) so rank 1
        assert results[0]['name'] == 'Alice' and results[0]['rank'] == '1'
        # Bob has lowest score (87) so rank 4
        assert results[1]['name'] == 'Bob' and results[1]['rank'] == '4'
        # Charlie has second highest (92) so rank 2
        assert results[2]['name'] == 'Charlie' and results[2]['rank'] == '2'
        # Diana has third highest (88) so rank 3
        assert results[3]['name'] == 'Diana' and results[3]['rank'] == '3'
        print("✓ test_row_number_basic passed")
    finally:
        Path(temp_file).unlink()

def test_partition_by():
    """Test window functions with PARTITION BY."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("department,employee,salary\n")
        f.write("Sales,Alice,50000\n")
        f.write("Sales,Bob,45000\n")
        f.write("Sales,Charlie,55000\n")
        f.write("IT,David,60000\n")
        f.write("IT,Eve,65000\n")
        f.write("IT,Frank,58000\n")
        temp_file = f.name
    
    try:
        # Test ROW_NUMBER with PARTITION BY
        query = """
        SELECT department, employee, salary,
               ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as dept_rank
        FROM test
        """
        results = run_query(temp_file, query)
        
        # Check Sales department ranking
        sales = [r for r in results if r['department'] == 'Sales']
        # Window functions don't change row order, so Alice is still first
        assert sales[0]['employee'] == 'Alice' and sales[0]['dept_rank'] == '2'
        assert sales[1]['employee'] == 'Bob' and sales[1]['dept_rank'] == '3'
        assert sales[2]['employee'] == 'Charlie' and sales[2]['dept_rank'] == '1'
        
        # Check IT department ranking
        it = [r for r in results if r['department'] == 'IT']
        # Window functions don't change row order, so David is still first
        assert it[0]['employee'] == 'David' and it[0]['dept_rank'] == '2'
        assert it[1]['employee'] == 'Eve' and it[1]['dept_rank'] == '1'
        assert it[2]['employee'] == 'Frank' and it[2]['dept_rank'] == '3'
        
        print("✓ test_partition_by passed")
    finally:
        Path(temp_file).unlink()

def test_lag_with_partition():
    """Test LAG with PARTITION BY."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("category,month,sales\n")
        f.write("A,1,100\n")
        f.write("A,2,150\n")
        f.write("A,3,200\n")
        f.write("B,1,300\n")
        f.write("B,2,350\n")
        f.write("B,3,400\n")
        temp_file = f.name
    
    try:
        query = """
        SELECT category, month, sales,
               LAG(sales, 1) OVER (PARTITION BY category ORDER BY month) as prev_sales
        FROM test
        """
        results = run_query(temp_file, query)
        
        # Category A should have NULL, 100, 150 as prev_sales
        cat_a = [r for r in results if r['category'] == 'A']
        assert cat_a[0]['prev_sales'] == ''  # NULL
        assert cat_a[1]['prev_sales'] == '100'
        assert cat_a[2]['prev_sales'] == '150'
        
        # Category B should have NULL, 300, 350 as prev_sales
        cat_b = [r for r in results if r['category'] == 'B']
        assert cat_b[0]['prev_sales'] == ''  # NULL
        assert cat_b[1]['prev_sales'] == '300'
        assert cat_b[2]['prev_sales'] == '350'
        
        print("✓ test_lag_with_partition passed")
    finally:
        Path(temp_file).unlink()

def test_multiple_window_functions():
    """Test multiple window functions in same query."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        f.write("1,10\n")
        f.write("2,20\n")
        f.write("3,30\n")
        f.write("4,40\n")
        temp_file = f.name
    
    try:
        query = """
        SELECT id, value,
               LAG(value, 1) OVER (ORDER BY id) as prev_val,
               LEAD(value, 1) OVER (ORDER BY id) as next_val,
               ROW_NUMBER() OVER (ORDER BY value DESC) as rank
        FROM test
        """
        results = run_query(temp_file, query)
        
        # Check first row
        assert results[0]['id'] == '1'
        assert results[0]['prev_val'] == ''  # NULL
        assert results[0]['next_val'] == '20'
        assert results[0]['rank'] == '4'  # Smallest value gets rank 4 in DESC order
        
        # Check last row
        assert results[3]['id'] == '4'
        assert results[3]['prev_val'] == '30'
        assert results[3]['next_val'] == ''  # NULL
        assert results[3]['rank'] == '1'  # Largest value gets rank 1 in DESC order
        
        print("✓ test_multiple_window_functions passed")
    finally:
        Path(temp_file).unlink()

def test_first_last_value():
    """Test FIRST_VALUE and LAST_VALUE functions."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("group_id,sequence,value\n")
        f.write("A,1,100\n")
        f.write("A,2,200\n")
        f.write("A,3,300\n")
        f.write("B,1,400\n")
        f.write("B,2,500\n")
        temp_file = f.name
    
    try:
        query = """
        SELECT group_id, sequence, value,
               FIRST_VALUE(value) OVER (PARTITION BY group_id ORDER BY sequence) as first_val,
               LAST_VALUE(value) OVER (PARTITION BY group_id ORDER BY sequence) as last_val
        FROM test
        """
        results = run_query(temp_file, query)
        
        # Group A - all rows should have first_val=100
        group_a = [r for r in results if r['group_id'] == 'A']
        for row in group_a:
            assert row['first_val'] == '100'
        
        # With SQL standard implicit frame (RANGE UNBOUNDED PRECEDING to CURRENT ROW),
        # LAST_VALUE shows the last value in the frame up to the current row
        assert group_a[0]['last_val'] == '100'  # seq=1, last value up to current row
        assert group_a[1]['last_val'] == '200'  # seq=2, last value up to current row
        assert group_a[2]['last_val'] == '300'  # seq=3, last value up to current row
        
        # Group B - first_val is always 400, last_val changes based on frame
        group_b = [r for r in results if r['group_id'] == 'B']
        assert group_b[0]['first_val'] == '400'
        assert group_b[0]['last_val'] == '400'  # seq=1, last value up to current row
        assert group_b[1]['first_val'] == '400'
        assert group_b[1]['last_val'] == '500'  # seq=2, last value up to current row
        
        print("✓ test_first_last_value passed")
    finally:
        Path(temp_file).unlink()

def test_lag_with_offset():
    """Test LAG with different offsets."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        for i in range(1, 11):
            f.write(f"{i},{i*10}\n")
        temp_file = f.name
    
    try:
        # Test LAG with offset 2
        query = """
        SELECT id, value,
               LAG(value, 2) OVER (ORDER BY id) as lag_2
        FROM test
        WHERE id <= 5
        """
        results = run_query(temp_file, query)
        
        assert results[0]['lag_2'] == ''  # NULL for id=1
        assert results[1]['lag_2'] == ''  # NULL for id=2
        assert results[2]['lag_2'] == '10'  # id=3 looks back 2 to id=1
        assert results[3]['lag_2'] == '20'  # id=4 looks back 2 to id=2
        assert results[4]['lag_2'] == '30'  # id=5 looks back 2 to id=3
        
        print("✓ test_lag_with_offset passed")
    finally:
        Path(temp_file).unlink()

def test_complex_partitioning():
    """Test complex partitioning with multiple columns."""
    sales_data = Path(__file__).parent.parent.parent / "data" / "sales_data.csv"
    
    # Test getting top salesperson per region
    query = """
    SELECT region, salesperson, month, sales_amount,
           ROW_NUMBER() OVER (PARTITION BY region, month ORDER BY sales_amount DESC) as rank
    FROM test
    """
    results = run_query(sales_data, query)
    
    # Check that each region-month combination has proper ranking
    for region in ['North', 'South', 'East', 'West']:
        for month in ['2024-01', '2024-02', '2024-03']:
            region_month = [r for r in results 
                          if r['region'] == region and r['month'] == month]
            # Should have exactly 2 salespersons per region
            assert len(region_month) == 2
            # Rankings should be 1 and 2
            ranks = sorted([int(r['rank']) for r in region_month])
            assert ranks == [1, 2]
    
    print("✓ test_complex_partitioning passed")

def test_where_with_window_function():
    """Test using window functions in WHERE clause (via subquery)."""
    sales_data = Path(__file__).parent.parent.parent / "data" / "sales_data.csv"
    
    # Note: Direct window functions in WHERE are not standard SQL
    # But we can test if the computed column can be used in further filtering
    # This tests getting the top performer from each region
    query = """
    SELECT region, salesperson, month, sales_amount,
           ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
    FROM test
    """
    results = run_query(sales_data, query)
    
    # Filter results in Python to simulate WHERE rank = 1
    top_performers = [r for r in results if r['rank'] == '1']
    
    # Should have one top performer per region (with highest sales in that region)
    regions = set(r['region'] for r in top_performers)
    assert len(regions) == 4  # North, South, East, West
    
    # Verify these are actually the top sales amounts
    for region in regions:
        region_top = [r for r in top_performers if r['region'] == region][0]
        region_all = [r for r in results if r['region'] == region]
        max_sales = max(int(r['sales_amount']) for r in region_all)
        assert int(region_top['sales_amount']) == max_sales
    
    print("✓ test_where_with_window_function passed")

def main():
    """Run all tests."""
    print("Running window function tests...")
    
    # Check if sql-cli is built
    if not SQL_CLI.exists():
        print(f"Error: sql-cli not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1
    
    tests = [
        test_lag_basic,
        test_lead_basic,
        test_row_number_basic,
        test_partition_by,
        test_lag_with_partition,
        test_multiple_window_functions,
        test_first_last_value,
        test_lag_with_offset,
        test_complex_partitioning,
        test_where_with_window_function,
    ]
    
    failed = 0
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
    
    if failed == 0:
        print(f"\n✅ All {len(tests)} window function tests passed!")
    else:
        print(f"\n❌ {failed}/{len(tests)} tests failed")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())