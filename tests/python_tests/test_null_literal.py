#!/usr/bin/env python3
"""
Test suite for NULL literal support and COALESCE function.

Tests that NULL can be used as a literal in expressions and
that COALESCE function works correctly with NULL values.
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
    # Don't strip the entire output - just split on newlines to preserve empty lines
    output_lines = result.stdout.split('\n')
    # Remove only trailing empty line if exists (from final newline)
    if output_lines and output_lines[-1] == '':
        output_lines = output_lines[:-1]
    
    csv_lines = [line for line in output_lines if not line.startswith('#')]
    
    if not csv_lines:
        return []
    
    # Special handling for queries that return NULL values
    # If we have a header and then an empty line, that represents a single row with all NULL values
    if len(csv_lines) == 2 and csv_lines[1] == '':
        # Get column names from header
        headers = csv_lines[0].split(',')
        # Return a single row with all empty values
        return [{col: '' for col in headers}]
    
    reader = csv.DictReader(io.StringIO('\n'.join(csv_lines)))
    return list(reader)

def run_standalone_query(query):
    """Run a standalone SQL query (without data file) and return results."""
    result = subprocess.run(
        [str(SQL_CLI), "-q", query, "-o", "csv"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        # For standalone queries, we might need a dummy file
        # Create a minimal CSV file to work with
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("dummy\n1\n")
            temp_file = f.name
        
        try:
            return run_query(temp_file, query)
        finally:
            Path(temp_file).unlink()
    
    # Parse CSV output
    # Don't strip the entire output - just split on newlines to preserve empty lines
    output_lines = result.stdout.split('\n')
    # Remove only trailing empty line if exists (from final newline)
    if output_lines and output_lines[-1] == '':
        output_lines = output_lines[:-1]
    
    csv_lines = [line for line in output_lines if not line.startswith('#')]
    
    if not csv_lines:
        return []
    
    # Special handling for queries that return NULL values
    # If we have a header and then an empty line, that represents a single row with all NULL values
    if len(csv_lines) == 2 and csv_lines[1] == '':
        # Get column names from header
        headers = csv_lines[0].split(',')
        # Return a single row with all empty values
        return [{col: '' for col in headers}]
    
    reader = csv.DictReader(io.StringIO('\n'.join(csv_lines)))
    return list(reader)

def test_null_literal_select():
    """Test selecting NULL literal."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id\n1\n")
        temp_file = f.name
    
    try:
        # Test NULL literal (without FROM, should return 1 row)
        query = "SELECT NULL as null_col"
        results = run_query(temp_file, query)
        
        assert len(results) >= 1  # May return 1 or more rows depending on implementation
        assert results[0]['null_col'] == ''  # NULL shown as empty
        
        # Test NULL with alias
        query = "SELECT NULL as test_null, 42 as test_num"
        results = run_query(temp_file, query)
        
        assert results[0]['test_null'] == ''
        assert results[0]['test_num'] == '42'
        
        print("✓ test_null_literal_select passed")
    finally:
        Path(temp_file).unlink()

def test_coalesce_with_null_literal():
    """Test COALESCE function with NULL literals."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        # Use two columns to properly represent NULL values
        # In CSV, empty fields between commas are NULL
        f.write("id,value\n")
        f.write("1,10\n")
        f.write("2,\n")    # NULL value field  
        f.write("3,30\n")
        temp_file = f.name
    
    try:
        # Test COALESCE with NULL literals
        query = "SELECT COALESCE(NULL, 2, 3) as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == '2'
        
        # Test COALESCE with multiple NULLs
        query = "SELECT COALESCE(NULL, NULL, 'test') as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == 'test'
        
        # Test COALESCE with all NULLs
        query = "SELECT COALESCE(NULL, NULL, NULL) as result"
        results = run_query(temp_file, query)
        assert len(results) >= 1
        assert results[0]['result'] == ''  # All NULL returns NULL
        
        # Test COALESCE with column and NULL
        query = "SELECT value, COALESCE(value, 999) as filled FROM test"
        results = run_query(temp_file, query)
        
        assert len(results) == 3, f"Expected 3 rows, got {len(results)}"
        assert results[0]['filled'] == '10'   # Non-null value
        assert results[1]['filled'] == '999'   # NULL replaced with 999
        assert results[2]['filled'] == '30'    # Non-null value
        
        print("✓ test_coalesce_with_null_literal passed")
    finally:
        Path(temp_file).unlink()

def test_null_in_expressions():
    """Test NULL in various expressions."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("x,y\n5,10\n,20\n15,\n")
        temp_file = f.name
    
    try:
        
        # Test NULL in arithmetic with COALESCE
        query = """
        SELECT 
            x, y,
            COALESCE(x, 0) + COALESCE(y, 0) as safe_sum
        FROM test
        """
        results = run_query(temp_file, query)
        
        assert results[0]['safe_sum'] == '15'  # 5 + 10
        assert results[1]['safe_sum'] == '20'  # 0 + 20
        assert results[2]['safe_sum'] == '15'  # 15 + 0
        
        print("✓ test_null_in_expressions passed")
    finally:
        Path(temp_file).unlink()

def test_standalone_expressions():
    """Test standalone SQL expressions with NULL."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("dummy\n1\n")
        temp_file = f.name
    
    try:
        # Test standalone COALESCE
        query = "SELECT COALESCE(1, 2, 3) as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == '1'
        
        query = "SELECT COALESCE(NULL, 2, 3) as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == '2'
        
        print("✓ test_standalone_expressions passed")
    finally:
        Path(temp_file).unlink()

def test_null_is_null_expressions():
    """Test NULL IS NULL and NULL IS NOT NULL expressions."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("dummy\n1\n")
        temp_file = f.name
    
    try:
        # Test NULL IS NULL (should be true)
        query = "SELECT NULL IS NULL as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == 'true', f"NULL IS NULL should be true, got {results[0]['result']}"
        
        # Test NULL IS NOT NULL (should be false)
        query = "SELECT NULL IS NOT NULL as result"
        results = run_query(temp_file, query)
        assert results[0]['result'] == 'false', f"NULL IS NOT NULL should be false, got {results[0]['result']}"
        
        print("✓ test_null_is_null_expressions passed")
    finally:
        Path(temp_file).unlink()

def test_case_when_with_null():
    """Test CASE WHEN with NULL handling."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,value\n")
        f.write("1,10\n")
        f.write("2,\n")    # NULL value
        f.write("3,0\n")   # Zero value (not NULL)
        f.write("4,\n")    # Another NULL
        temp_file = f.name
    
    try:
        # Test CASE WHEN with IS NULL
        query = """
        SELECT 
            id, 
            value,
            CASE 
                WHEN value IS NULL THEN 'null_value'
                WHEN value = 0 THEN 'zero_value'
                ELSE 'has_value'
            END as status
        FROM test
        """
        results = run_query(temp_file, query)
        
        assert len(results) == 4
        assert results[0]['status'] == 'has_value'  # 10
        assert results[1]['status'] == 'null_value'  # NULL
        assert results[2]['status'] == 'zero_value'  # 0
        assert results[3]['status'] == 'null_value'  # NULL
        
        # Test CASE with NULL in THEN clause
        query = """
        SELECT 
            id,
            CASE 
                WHEN id = 2 THEN NULL
                ELSE value
            END as modified_value
        FROM test
        """
        results = run_query(temp_file, query)
        
        assert results[0]['modified_value'] == '10'
        assert results[1]['modified_value'] == ''  # NULL returned as empty
        assert results[2]['modified_value'] == '0'
        assert results[3]['modified_value'] == ''  # Original NULL
        
        print("✓ test_case_when_with_null passed")
    finally:
        Path(temp_file).unlink()

def test_null_in_where_clause():
    """Test NULL handling in WHERE clauses."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,name,score\n")
        f.write("1,Alice,95\n")
        f.write("2,Bob,\n")      # NULL score
        f.write("3,Charlie,87\n")
        f.write("4,,92\n")       # NULL name
        f.write("5,Eve,\n")      # NULL score
        temp_file = f.name
    
    try:
        # Test WHERE IS NULL
        query = "SELECT id, name FROM test WHERE score IS NULL"
        results = run_query(temp_file, query)
        assert len(results) == 2
        assert results[0]['id'] == '2'
        assert results[1]['id'] == '5'
        
        # Test WHERE IS NOT NULL
        query = "SELECT id, name FROM test WHERE score IS NOT NULL"
        results = run_query(temp_file, query)
        assert len(results) == 3
        assert results[0]['id'] == '1'
        assert results[1]['id'] == '3'
        assert results[2]['id'] == '4'
        
        # Test multiple NULL conditions
        query = "SELECT id FROM test WHERE name IS NOT NULL AND score IS NOT NULL"
        results = run_query(temp_file, query)
        assert len(results) == 2
        assert results[0]['id'] == '1'
        assert results[1]['id'] == '3'
        
        print("✓ test_null_in_where_clause passed")
    finally:
        Path(temp_file).unlink()

def test_null_arithmetic():
    """Test NULL in arithmetic operations."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("a,b\n")
        f.write("10,5\n")
        f.write(",3\n")    # NULL + 3
        f.write("7,\n")    # 7 + NULL
        temp_file = f.name
    
    try:
        # Test NULL in arithmetic (should propagate NULL)
        query = "SELECT a, b, a + b as sum FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['sum'] == '15'  # 10 + 5
        assert results[1]['sum'] == ''    # NULL + 3 = NULL
        assert results[2]['sum'] == ''    # 7 + NULL = NULL
        
        # Test COALESCE to handle NULL in arithmetic
        query = "SELECT COALESCE(a, 0) + COALESCE(b, 0) as safe_sum FROM test"
        results = run_query(temp_file, query)
        
        assert results[0]['safe_sum'] == '15'  # 10 + 5
        assert results[1]['safe_sum'] == '3'   # 0 + 3
        assert results[2]['safe_sum'] == '7'   # 7 + 0
        
        print("✓ test_null_arithmetic passed")
    finally:
        Path(temp_file).unlink()

def main():
    """Run all tests."""
    print("Running NULL literal and COALESCE tests...")
    
    # Check if sql-cli is built
    if not SQL_CLI.exists():
        print(f"Error: sql-cli not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1
    
    tests = [
        test_null_literal_select,
        test_coalesce_with_null_literal,
        test_null_in_expressions,
        test_standalone_expressions,
        test_null_is_null_expressions,
        test_case_when_with_null,
        test_null_in_where_clause,
        test_null_arithmetic,
    ]
    
    failed = 0
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
            import traceback
            traceback.print_exc()
            failed += 1
    
    if failed == 0:
        print(f"\n✅ All {len(tests)} NULL tests passed!")
    else:
        print(f"\n❌ {failed}/{len(tests)} tests failed")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())