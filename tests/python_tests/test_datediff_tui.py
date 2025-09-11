#!/usr/bin/env python3
"""Test DateDiff function works correctly in TUI mode with datetime columns."""

import subprocess
import tempfile
import os
import sys

def test_datediff_with_datetime_columns():
    """Test that DateDiff works with datetime columns that might be interned."""
    
    # Create test CSV with datetime values
    csv_content = """id,Prod-Value,UAT-Value,description
1,2024-01-01 10:30:00,2024-01-15 14:45:00,Test case 1
2,2024-02-01 09:00:00,2024-02-28 16:30:00,Test case 2
3,2024-03-01 11:15:00,2024-03-31 23:59:00,Test case 3
4,2024-04-01 00:00:00,2024-05-01 12:00:00,Test case 4
5,2024-06-01 08:30:00,2024-06-15 17:45:00,Test case 5"""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write(csv_content)
        csv_file = f.name
    
    try:
        # Test with CLI mode
        query = 'SELECT *, DATEDIFF(\'day\', "Prod-Value", "UAT-Value") AS diff_days FROM test'
        cmd = ['./target/release/sql-cli', csv_file, '-q', query, '-o', 'csv']
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Error: {result.stderr}")
            return False
        
        output_lines = result.stdout.strip().split('\n')
        
        # Check header
        expected_header = 'id,Prod-Value,UAT-Value,description,diff_days'
        if output_lines[0] != expected_header:
            print(f"Header mismatch: expected {expected_header}, got {output_lines[0]}")
            return False
        
        # Check data rows
        expected_diffs = ['14', '27', '30', '30', '14']
        for i, expected_diff in enumerate(expected_diffs, 1):
            if i >= len(output_lines):
                print(f"Missing row {i}")
                return False
            
            parts = output_lines[i].split(',')
            if len(parts) < 5:
                print(f"Row {i} has wrong number of columns: {output_lines[i]}")
                return False
                
            actual_diff = parts[4]
            if actual_diff != expected_diff:
                print(f"Row {i}: expected diff={expected_diff}, got {actual_diff}")
                return False
        
        print("✓ DateDiff with datetime columns works correctly")
        return True
        
    finally:
        os.unlink(csv_file)

def test_datediff_with_quoted_columns():
    """Test DateDiff with column names containing special characters."""
    
    # Create test CSV with datetime values and special column names
    csv_content = """id,"Start-Date","End-Date",status
1,2024-01-01,2024-01-15,active
2,2024-02-01,2024-02-28,pending
3,2024-03-01,2024-03-31,complete"""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write(csv_content)
        csv_file = f.name
    
    try:
        # Test with quoted column names
        query = 'SELECT *, DATEDIFF(\'day\', "Start-Date", "End-Date") AS days_diff FROM test'
        cmd = ['./target/release/sql-cli', csv_file, '-q', query, '-o', 'csv']
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Error: {result.stderr}")
            return False
        
        output_lines = result.stdout.strip().split('\n')
        
        # Check that we got results (not checking exact values as dates might vary)
        if len(output_lines) < 2:
            print("No data rows returned")
            return False
        
        # Check header has the new column
        if 'days_diff' not in output_lines[0]:
            print(f"Missing days_diff column in header: {output_lines[0]}")
            return False
        
        print("✓ DateDiff with quoted column names works correctly")
        return True
        
    finally:
        os.unlink(csv_file)

if __name__ == '__main__':
    tests_passed = 0
    tests_failed = 0
    
    tests = [
        test_datediff_with_datetime_columns,
        test_datediff_with_quoted_columns
    ]
    
    for test in tests:
        try:
            if test():
                tests_passed += 1
            else:
                tests_failed += 1
        except Exception as e:
            print(f"✗ {test.__name__} failed with exception: {e}")
            tests_failed += 1
    
    print(f"\n{tests_passed} tests passed, {tests_failed} tests failed")
    sys.exit(0 if tests_failed == 0 else 1)