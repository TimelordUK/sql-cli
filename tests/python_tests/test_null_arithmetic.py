#!/usr/bin/env python3
"""
Test suite for NULL handling in arithmetic operations.

Tests that arithmetic operations with NULL values return NULL
as per SQL standard.
"""

import subprocess
import csv
import io
from pathlib import Path
import tempfile

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
    csv_lines = [line for line in output_lines if not line.startswith('#')]
    
    if not csv_lines:
        return []
    
    reader = csv.DictReader(io.StringIO('\n'.join(csv_lines)))
    return list(reader)

def test_null_arithmetic_with_lag():
    """Test NULL arithmetic with LAG function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        f.write("1,100\n")
        f.write("2,200\n")
        f.write("3,300\n")
        temp_file = f.name
    
    try:
        # Test subtraction with NULL from LAG
        query = """
        SELECT 
            id, 
            value,
            LAG(value, 1) OVER (ORDER BY id) as prev,
            value - LAG(value, 1) OVER (ORDER BY id) as diff
        FROM test
        """
        results = run_query(temp_file, query)
        
        # First row should have NULL prev and NULL diff
        assert results[0]['prev'] == ''  # NULL shown as empty
        assert results[0]['diff'] == ''  # NULL - should be empty
        
        # Second row should have valid diff
        assert results[1]['prev'] == '100'
        assert results[1]['diff'] == '100'  # 200 - 100
        
        print("✓ test_null_arithmetic_with_lag passed")
    finally:
        Path(temp_file).unlink()

def test_null_arithmetic_operations():
    """Test all arithmetic operations with NULL."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("a,b\n")
        f.write("10,5\n")
        f.write("20,\n")  # b is NULL
        f.write(",30\n")  # a is NULL
        temp_file = f.name
    
    try:
        # Test addition with NULL
        query = "SELECT a, b, a + b as sum FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['sum'] == '15'  # 10 + 5
        assert results[1]['sum'] == ''     # 20 + NULL = NULL
        assert results[2]['sum'] == ''     # NULL + 30 = NULL
        
        # Test subtraction with NULL
        query = "SELECT a, b, a - b as diff FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['diff'] == '5'   # 10 - 5
        assert results[1]['diff'] == ''     # 20 - NULL = NULL
        assert results[2]['diff'] == ''     # NULL - 30 = NULL
        
        # Test multiplication with NULL
        query = "SELECT a, b, a * b as product FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['product'] == '50'  # 10 * 5
        assert results[1]['product'] == ''     # 20 * NULL = NULL
        assert results[2]['product'] == ''     # NULL * 30 = NULL
        
        # Test division with NULL
        query = "SELECT a, b, a / b as quotient FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['quotient'] == '2'  # 10 / 5
        assert results[1]['quotient'] == ''    # 20 / NULL = NULL
        assert results[2]['quotient'] == ''    # NULL / 30 = NULL
        
        print("✓ test_null_arithmetic_operations passed")
    finally:
        Path(temp_file).unlink()

def test_complex_null_expressions():
    """Test complex expressions with NULL."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("x,y,z\n")
        f.write("10,20,30\n")
        f.write("5,,15\n")  # y is NULL
        temp_file = f.name
    
    try:
        # Complex expression with NULL
        query = "SELECT x, y, z, (x + y) * z as result FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['result'] == '900'  # (10 + 20) * 30 = 900
        assert results[1]['result'] == ''      # (5 + NULL) * 15 = NULL
        
        print("✓ test_complex_null_expressions passed")
    finally:
        Path(temp_file).unlink()

def main():
    """Run all tests."""
    print("Running NULL arithmetic tests...")
    
    # Check if sql-cli is built
    if not SQL_CLI.exists():
        print(f"Error: sql-cli not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1
    
    tests = [
        test_null_arithmetic_with_lag,
        test_null_arithmetic_operations,
        test_complex_null_expressions,
    ]
    
    failed = 0
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
    
    if failed == 0:
        print(f"\n✅ All {len(tests)} NULL arithmetic tests passed!")
    else:
        print(f"\n❌ {failed}/{len(tests)} tests failed")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())