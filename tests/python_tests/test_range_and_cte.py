#!/usr/bin/env python3
"""
Test suite for RANGE function with CTEs and prime calculations
Tests various combinations to ensure proper functionality
"""

import subprocess
import csv
from io import StringIO
import sys
import os

# Path to the SQL CLI executable
SQL_CLI = os.path.join(os.path.dirname(__file__), "../../target/release/sql-cli")

def run_query(query):
    """Execute a query and return results as list of dicts"""
    result = subprocess.run(
        [SQL_CLI, "-q", query, "-o", "csv"],
        capture_output=True,
        text=True,
    )
    
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse CSV output
    reader = csv.DictReader(StringIO(result.stdout))
    return list(reader)

def test_basic_range():
    """Test basic RANGE function"""
    query = "SELECT value FROM RANGE(1, 5)"
    result = run_query(query)
    
    expected = [1, 2, 3, 4, 5]
    actual = [int(row['value']) for row in result]
    
    assert actual == expected, f"Expected {expected}, got {actual}"
    print("✓ Basic RANGE test passed")

def test_range_with_step():
    """Test RANGE with step parameter"""
    query = "SELECT value FROM RANGE(0, 20, 5)"
    result = run_query(query)
    
    expected = [0, 5, 10, 15, 20]
    actual = [int(row['value']) for row in result]
    
    assert actual == expected, f"Expected {expected}, got {actual}"
    print("✓ RANGE with step test passed")

def test_range_in_cte():
    """Test RANGE function within CTE"""
    query = """
    WITH numbers AS (
        SELECT value FROM RANGE(1, 5)
    )
    SELECT value, value * 2 AS doubled FROM numbers
    """
    result = run_query(query)
    
    expected_values = [1, 2, 3, 4, 5]
    expected_doubled = [2, 4, 6, 8, 10]
    
    actual_values = [int(row['value']) for row in result]
    actual_doubled = [int(row['doubled']) for row in result]
    
    assert actual_values == expected_values, f"Values mismatch"
    assert actual_doubled == expected_doubled, f"Doubled values mismatch"
    print("✓ RANGE in CTE test passed")

def test_nested_cte_with_range():
    """Test nested CTEs with RANGE"""
    query = """
    WITH first_range AS (
        SELECT value AS n FROM RANGE(1, 3)
    ),
    squared AS (
        SELECT n, n * n AS sq FROM first_range
    )
    SELECT n, sq, sq * 2 AS doubled_square FROM squared
    """
    result = run_query(query)
    
    expected = [
        {'n': 1, 'sq': 1, 'doubled_square': 2},
        {'n': 2, 'sq': 4, 'doubled_square': 8},
        {'n': 3, 'sq': 9, 'doubled_square': 18}
    ]
    
    for i, row in enumerate(result):
        assert int(row['n']) == expected[i]['n']
        assert int(row['sq']) == expected[i]['sq']
        assert int(row['doubled_square']) == expected[i]['doubled_square']
    
    print("✓ Nested CTE with RANGE test passed")

def test_range_with_where_clause():
    """Test RANGE with WHERE clause filtering"""
    query = "SELECT value FROM RANGE(1, 20) WHERE value % 3 = 0"
    result = run_query(query)
    
    expected = [3, 6, 9, 12, 15, 18]
    actual = [int(row['value']) for row in result]
    
    assert actual == expected, f"Expected {expected}, got {actual}"
    print("✓ RANGE with WHERE clause test passed")

def test_range_with_is_prime():
    """Test RANGE with IS_PRIME function"""
    query = "SELECT value FROM RANGE(2, 20) WHERE IS_PRIME(value) = true"
    result = run_query(query)
    
    expected = [2, 3, 5, 7, 11, 13, 17, 19]
    actual = [int(row['value']) for row in result]
    
    assert actual == expected, f"Expected primes {expected}, got {actual}"
    print("✓ RANGE with IS_PRIME test passed")

def test_prime_count_in_range():
    """Test counting primes in a range"""
    query = "SELECT COUNT(*) AS prime_count FROM RANGE(2, 100) WHERE IS_PRIME(value) = true"
    result = run_query(query)
    
    assert int(result[0]['prime_count']) == 25, f"Expected 25 primes below 100"
    print("✓ Prime count in range test passed")

def test_prime_pi_with_range():
    """Test PRIME_PI function with RANGE"""
    query = """
    SELECT 
        value AS n,
        PRIME_PI(value) AS primes_up_to_n
    FROM RANGE(10, 50, 10)
    """
    result = run_query(query)
    
    expected_counts = {
        10: 4,   # primes up to 10: 2,3,5,7
        20: 8,   # adds: 11,13,17,19
        30: 10,  # adds: 23,29
        40: 12,  # adds: 31,37
        50: 15   # adds: 41,43,47
    }
    
    for row in result:
        n = int(row['n'])
        expected = expected_counts[n]
        actual = int(row['primes_up_to_n'])
        assert actual == expected, f"PRIME_PI({n}) should be {expected}, got {actual}"
    
    print("✓ PRIME_PI with RANGE test passed")

def test_prime_density_blocks():
    """Test prime density calculation in blocks"""
    query = """
    WITH blocks AS (
        SELECT value AS block_num FROM RANGE(1, 5)
    )
    SELECT 
        block_num,
        PRIME_PI(block_num * 100) - PRIME_PI((block_num - 1) * 100) AS primes_in_block
    FROM blocks
    """
    result = run_query(query)
    
    # Known prime counts in each 100-number block
    expected_counts = {
        1: 25,  # 1-100
        2: 21,  # 101-200
        3: 16,  # 201-300
        4: 16,  # 301-400
        5: 17   # 401-500
    }
    
    for row in result:
        block = int(row['block_num'])
        expected = expected_counts[block]
        actual = int(row['primes_in_block'])
        assert actual == expected, f"Block {block} should have {expected} primes, got {actual}"
    
    print("✓ Prime density blocks test passed")

def test_complex_cte_with_aggregates():
    """Test complex CTE with RANGE and aggregates"""
    query = """
    WITH prime_analysis AS (
        SELECT 
            value,
            IS_PRIME(value) AS is_prime
        FROM RANGE(1, 100)
    )
    SELECT 
        COUNT(*) AS total_numbers,
        SUM(CASE WHEN is_prime = true THEN 1 ELSE 0 END) AS prime_count,
        SUM(CASE WHEN is_prime = false THEN 1 ELSE 0 END) AS non_prime_count
    FROM prime_analysis
    """
    result = run_query(query)
    
    row = result[0]
    assert int(row['total_numbers']) == 100, "Should have 100 total numbers"
    assert int(row['prime_count']) == 25, "Should have 25 primes"
    assert int(row['non_prime_count']) == 75, "Should have 75 non-primes"
    
    print("✓ Complex CTE with aggregates test passed")

def test_multiple_ranges_in_cte():
    """Test multiple RANGE calls in different CTEs"""
    query = """
    WITH small_range AS (
        SELECT value AS small FROM RANGE(1, 3)
    ),
    large_range AS (
        SELECT value AS large FROM RANGE(10, 12)
    )
    SELECT 
        small,
        large,
        small * large AS product
    FROM small_range, large_range
    ORDER BY small, large
    """
    result = run_query(query)
    
    expected_products = [
        10, 11, 12,  # 1 * 10, 1 * 11, 1 * 12
        20, 22, 24,  # 2 * 10, 2 * 11, 2 * 12
        30, 33, 36   # 3 * 10, 3 * 11, 3 * 12
    ]
    
    actual_products = [int(row['product']) for row in result]
    assert actual_products == expected_products, f"Cross product mismatch"
    
    print("✓ Multiple RANGE in CTE test passed")

def test_range_with_calculations():
    """Test RANGE with various calculations"""
    query = """
    SELECT 
        value,
        value * value AS squared,
        value * value * value AS cubed,
        SQRT(value) AS square_root
    FROM RANGE(1, 5)
    """
    result = run_query(query)
    
    for row in result:
        n = int(row['value'])
        assert int(row['squared']) == n * n
        assert int(row['cubed']) == n * n * n
        assert abs(float(row['square_root']) - (n ** 0.5)) < 0.001
    
    print("✓ RANGE with calculations test passed")

def test_range_edge_cases():
    """Test RANGE edge cases"""
    
    # Single value range
    query = "SELECT value FROM RANGE(5, 5)"
    result = run_query(query)
    assert len(result) == 1 and int(result[0]['value']) == 5
    
    # Negative range
    query = "SELECT value FROM RANGE(-5, -1)"
    result = run_query(query)
    expected = [-5, -4, -3, -2, -1]
    actual = [int(row['value']) for row in result]
    assert actual == expected
    
    # Large step
    query = "SELECT value FROM RANGE(0, 100, 25)"
    result = run_query(query)
    expected = [0, 25, 50, 75, 100]
    actual = [int(row['value']) for row in result]
    assert actual == expected
    
    print("✓ RANGE edge cases test passed")

def test_range_with_group_by():
    """Test RANGE with GROUP BY"""
    query = """
    WITH numbers AS (
        SELECT value FROM RANGE(1, 20)
    )
    SELECT 
        value % 5 AS remainder,
        COUNT(*) AS count,
        SUM(value) AS sum_values
    FROM numbers
    GROUP BY value % 5
    ORDER BY remainder
    """
    result = run_query(query)
    
    # Each remainder group should have 4 numbers
    for row in result:
        assert int(row['count']) == 4, f"Each remainder group should have 4 numbers"
    
    # Check sum for remainder 0: 5+10+15+20 = 50
    remainder_0 = [r for r in result if int(r['remainder']) == 0][0]
    assert int(remainder_0['sum_values']) == 50
    
    print("✓ RANGE with GROUP BY test passed")

def test_range_with_order_by():
    """Test RANGE with ORDER BY"""
    query = """
    SELECT 
        value,
        value * value AS squared
    FROM RANGE(1, 5)
    ORDER BY squared DESC
    """
    result = run_query(query)
    
    expected_order = [5, 4, 3, 2, 1]
    actual_order = [int(row['value']) for row in result]
    assert actual_order == expected_order
    
    print("✓ RANGE with ORDER BY test passed")

def test_range_with_window_functions():
    """Test RANGE with window functions (PARTITION BY)"""
    query = """
    WITH data AS (
        SELECT 
            value,
            value % 3 AS grp
        FROM RANGE(1, 12)
    )
    SELECT 
        value,
        grp,
        ROW_NUMBER() OVER (PARTITION BY grp ORDER BY value) AS row_num,
        SUM(value) OVER (PARTITION BY grp) AS group_sum
    FROM data
    ORDER BY value
    """
    result = run_query(query)
    
    # Verify first few rows
    assert int(result[0]['value']) == 1
    assert int(result[0]['grp']) == 1
    assert int(result[0]['row_num']) == 1
    
    # Group 0: 3,6,9,12 sum = 30
    # Group 1: 1,4,7,10 sum = 22
    # Group 2: 2,5,8,11 sum = 26
    
    # Check group sums for some values
    for row in result:
        if int(row['grp']) == 0:
            assert int(row['group_sum']) == 30
        elif int(row['grp']) == 1:
            assert int(row['group_sum']) == 22
        elif int(row['grp']) == 2:
            assert int(row['group_sum']) == 26
    
    print("✓ RANGE with window functions test passed")

def test_cte_range_with_partition():
    """Test CTE with RANGE in one CTE and PARTITION BY in another"""
    query = """
    WITH base_numbers AS (
        SELECT value AS num FROM RANGE(1, 20)
    ),
    categorized AS (
        SELECT 
            num,
            CASE 
                WHEN IS_PRIME(num) = true THEN 'prime'
                WHEN num % 2 = 0 THEN 'even'
                ELSE 'odd'
            END AS category
        FROM base_numbers
    )
    SELECT 
        num,
        category,
        ROW_NUMBER() OVER (PARTITION BY category ORDER BY num) AS position_in_category,
        COUNT(*) OVER (PARTITION BY category) AS category_count
    FROM categorized
    ORDER BY num
    """
    result = run_query(query)
    
    # Count categories
    primes = [r for r in result if r['category'] == 'prime']
    evens = [r for r in result if r['category'] == 'even']
    odds = [r for r in result if r['category'] == 'odd']
    
    # There should be 8 primes (2,3,5,7,11,13,17,19)
    assert len(primes) == 8
    
    # Verify position_in_category for first prime (2)
    prime_2 = [r for r in result if int(r['num']) == 2][0]
    assert prime_2['category'] == 'prime'
    assert int(prime_2['position_in_category']) == 1
    
    print("✓ CTE with RANGE and PARTITION BY test passed")

def main():
    """Run all tests"""
    tests = [
        test_basic_range,
        test_range_with_step,
        test_range_in_cte,
        test_nested_cte_with_range,
        test_range_with_where_clause,
        test_range_with_is_prime,
        test_prime_count_in_range,
        test_prime_pi_with_range,
        test_prime_density_blocks,
        test_complex_cte_with_aggregates,
        test_multiple_ranges_in_cte,
        test_range_with_calculations,
        test_range_edge_cases,
        test_range_with_group_by,
        test_range_with_order_by,
        test_range_with_window_functions,
        test_cte_range_with_partition
    ]
    
    passed = 0
    failed = 0
    
    print("Running RANGE and CTE tests...")
    print("=" * 60)
    
    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"✗ {test.__name__}: {str(e)}")
            failed += 1
    
    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    
    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()