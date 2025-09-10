#!/usr/bin/env python3
"""Test format and random SQL functions"""

import subprocess
import json
import csv
import io

def run_query(query):
    """Run a SQL query and return the result"""
    cmd = ["./target/release/sql-cli", "-q", query, "-o", "csv"]
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    
    # Parse CSV output
    reader = csv.DictReader(io.StringIO(result.stdout))
    rows = list(reader)
    return rows

def test_random_functions():
    """Test random number generation functions"""
    print("Testing RANDOM()...")
    result = run_query("SELECT RANDOM()")
    assert len(result) == 1
    val = float(result[0]['expr_1'])
    assert 0.0 <= val < 1.0, f"RANDOM() returned {val}, expected 0.0 <= value < 1.0"
    print(f"  ✓ RANDOM() = {val}")
    
    print("Testing RAND_INT(1, 10)...")
    result = run_query("SELECT RAND_INT(1, 10)")
    assert len(result) == 1
    val = int(result[0]['expr_1'])
    assert 1 <= val <= 10, f"RAND_INT(1, 10) returned {val}, expected 1 <= value <= 10"
    print(f"  ✓ RAND_INT(1, 10) = {val}")
    
    print("Testing RAND_RANGE(5, 1, 100)...")
    result = run_query("SELECT RAND_RANGE(5, 1, 100)")
    assert len(result) == 1
    # Currently returns single value, will be enhanced later
    val = int(result[0]['expr_1'])
    assert 1 <= val <= 100, f"RAND_RANGE returned {val}, expected 1 <= value <= 100"
    print(f"  ✓ RAND_RANGE(5, 1, 100) = {val}")

def test_format_number():
    """Test FORMAT_NUMBER function"""
    print("\nTesting FORMAT_NUMBER...")
    
    # Test with separator
    result = run_query("SELECT FORMAT_NUMBER(1234567.89, 2)")
    assert result[0]['expr_1'] == '1,234,567.89'
    print("  ✓ FORMAT_NUMBER(1234567.89, 2) = '1,234,567.89'")
    
    # Test without separator
    result = run_query("SELECT FORMAT_NUMBER(1234567.89, 2, false)")
    assert result[0]['expr_1'] == '1234567.89'
    print("  ✓ FORMAT_NUMBER(1234567.89, 2, false) = '1234567.89'")
    
    # Test integer formatting
    result = run_query("SELECT FORMAT_NUMBER(1234567)")
    assert result[0]['expr_1'] == '1,234,567'
    print("  ✓ FORMAT_NUMBER(1234567) = '1,234,567'")

def test_padding_functions():
    """Test LPAD, RPAD, and CENTER functions"""
    print("\nTesting padding functions...")
    
    # Test LPAD
    result = run_query("SELECT LPAD('123', 7, '0')")
    assert result[0]['expr_1'] == '0000123'
    print("  ✓ LPAD('123', 7, '0') = '0000123'")
    
    result = run_query("SELECT LPAD('hello', 10, '.')")
    assert result[0]['expr_1'] == '.....hello'
    print("  ✓ LPAD('hello', 10, '.') = '.....hello'")
    
    # Test RPAD
    result = run_query("SELECT RPAD('123', 7, '0')")
    assert result[0]['expr_1'] == '1230000'
    print("  ✓ RPAD('123', 7, '0') = '1230000'")
    
    result = run_query("SELECT RPAD('hello', 10, '.')")
    assert result[0]['expr_1'] == 'hello.....'
    print("  ✓ RPAD('hello', 10, '.') = 'hello.....'")
    
    # Test CENTER
    result = run_query("SELECT CENTER('SQL', 9, '-')")
    assert result[0]['expr_1'] == '---SQL---'
    print("  ✓ CENTER('SQL', 9, '-') = '---SQL---'")
    
    result = run_query("SELECT CENTER('hello', 11)")
    assert result[0]['expr_1'] == '   hello   '
    print("  ✓ CENTER('hello', 11) = '   hello   '")

def test_format_date():
    """Test FORMAT_DATE function"""
    print("\nTesting FORMAT_DATE...")
    
    # Test date formatting
    result = run_query("SELECT FORMAT_DATE('2024-03-15', '%B %d, %Y')")
    assert result[0]['expr_1'] == 'March 15, 2024'
    print("  ✓ FORMAT_DATE('2024-03-15', '%B %d, %Y') = 'March 15, 2024'")
    
    result = run_query("SELECT FORMAT_DATE('2024-03-15', '%Y%m%d')")
    assert result[0]['expr_1'] == '20240315'
    print("  ✓ FORMAT_DATE('2024-03-15', '%Y%m%d') = '20240315'")
    
    # Test datetime formatting
    result = run_query("SELECT FORMAT_DATE('2024-03-15 14:30:00', '%Y-%m-%d %H:%M')")
    assert result[0]['expr_1'] == '2024-03-15 14:30'
    print("  ✓ FORMAT_DATE('2024-03-15 14:30:00', '%Y-%m-%d %H:%M') = '2024-03-15 14:30'")

def main():
    print("=" * 60)
    print("Testing Format and Random Functions")
    print("=" * 60)
    
    test_random_functions()
    test_format_number()
    test_padding_functions()
    test_format_date()
    
    print("\n" + "=" * 60)
    print("All tests passed! ✓")
    print("=" * 60)

if __name__ == "__main__":
    main()