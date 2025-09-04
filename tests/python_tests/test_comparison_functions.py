#!/usr/bin/env python3
"""
Test suite for comparison functions (GREATEST, LEAST, COALESCE, NULLIF)
"""

import subprocess
import json
import csv
import os
import tempfile
import pytest
from pathlib import Path

# Find the sql-cli binary
PROJECT_ROOT = Path(__file__).parent.parent.parent
SQL_CLI = PROJECT_ROOT / "target" / "release" / "sql-cli"

if not SQL_CLI.exists():
    SQL_CLI = PROJECT_ROOT / "target" / "debug" / "sql-cli"


def run_query(csv_file, query):
    """Run a SQL query and return results as a list of dictionaries."""
    cmd = [str(SQL_CLI), csv_file, "-q", query, "-o", "json"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse JSON output
    lines = result.stdout.strip().split('\n')
    # Filter out comment lines
    json_lines = [l for l in lines if not l.startswith('#')]
    if not json_lines:
        return []
    
    return json.loads(''.join(json_lines))


class TestGreatestLeast:
    """Test GREATEST and LEAST functions."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        # Numeric test data
        cls.numeric_file = os.path.join(cls.temp_dir, "numeric.csv")
        with open(cls.numeric_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'val1', 'val2', 'val3', 'val4'])
            writer.writerow([1, 10, 20, 5, 15])
            writer.writerow([2, -5, 0, 10, -2])
            writer.writerow([3, 100, 100, 100, 100])
            writer.writerow([4, 1.5, 2.5, 1.0, 3.0])
            
        # String test data
        cls.string_file = os.path.join(cls.temp_dir, "strings.csv")
        with open(cls.string_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'str1', 'str2', 'str3'])
            writer.writerow([1, 'apple', 'banana', 'cherry'])
            writer.writerow([2, 'zebra', 'aardvark', 'monkey'])
            writer.writerow([3, 'ABC', 'abc', 'Abc'])
    
    def test_greatest_with_numeric_columns(self):
        """Test GREATEST with numeric columns."""
        result = run_query(self.numeric_file, 
                          "SELECT id, GREATEST(val1, val2, val3, val4) as max_val FROM numeric")
        
        assert len(result) == 4
        assert result[0]['max_val'] == 20
        assert result[1]['max_val'] == 10
        assert result[2]['max_val'] == 100
        assert result[3]['max_val'] == 3.0
    
    def test_least_with_numeric_columns(self):
        """Test LEAST with numeric columns."""
        result = run_query(self.numeric_file,
                          "SELECT id, LEAST(val1, val2, val3, val4) as min_val FROM numeric")
        
        assert len(result) == 4
        assert result[0]['min_val'] == 5
        assert result[1]['min_val'] == -5
        assert result[2]['min_val'] == 100
        assert result[3]['min_val'] == 1.0
    
    def test_greatest_with_string_columns(self):
        """Test GREATEST with string columns."""
        result = run_query(self.string_file,
                          "SELECT id, GREATEST(str1, str2, str3) as max_str FROM strings")
        
        assert len(result) == 3
        assert result[0]['max_str'] == 'cherry'
        assert result[1]['max_str'] == 'zebra'
        assert result[2]['max_str'] == 'abc'  # Lowercase > uppercase in ASCII
    
    def test_least_with_string_columns(self):
        """Test LEAST with string columns."""
        result = run_query(self.string_file,
                          "SELECT id, LEAST(str1, str2, str3) as min_str FROM strings")
        
        assert len(result) == 3
        assert result[0]['min_str'] == 'apple'
        assert result[1]['min_str'] == 'aardvark'
        assert result[2]['min_str'] == 'ABC'  # Uppercase < lowercase in ASCII
    
    def test_greatest_with_literals(self):
        """Test GREATEST with literal values."""
        result = run_query(self.numeric_file,
                          "SELECT GREATEST(100, 200, 50) as max_literal FROM numeric LIMIT 1")
        
        assert len(result) == 1
        assert result[0]['max_literal'] == 200
    
    def test_least_with_literals(self):
        """Test LEAST with literal values."""
        result = run_query(self.numeric_file,
                          "SELECT LEAST(100, 200, 50) as min_literal FROM numeric LIMIT 1")
        
        assert len(result) == 1
        assert result[0]['min_literal'] == 50
    
    def test_mixed_columns_and_literals(self):
        """Test GREATEST/LEAST with mix of columns and literals."""
        result = run_query(self.numeric_file,
                          "SELECT id, "
                          "GREATEST(val1, 0) as at_least_zero, "
                          "LEAST(val2, 100) as at_most_100 "
                          "FROM numeric")
        
        assert len(result) == 4
        # Row 2 has val1 = -5, so GREATEST(val1, 0) should be 0
        assert result[1]['at_least_zero'] == 0
        # Row 3 has val2 = 100, so LEAST(val2, 100) should be 100
        assert result[2]['at_most_100'] == 100


class TestNullIfFunction:
    """Test NULLIF function."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        cls.test_file = os.path.join(cls.temp_dir, "nullif_test.csv")
        with open(cls.test_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'value', 'zero_val', 'status'])
            writer.writerow([1, 100, 0, 'active'])
            writer.writerow([2, 0, 0, 'deleted'])
            writer.writerow([3, 50, 10, 'active'])
            writer.writerow([4, -1, 0, 'deleted'])
    
    def test_nullif_with_equal_values(self):
        """Test NULLIF returns NULL when values are equal."""
        result = run_query(self.test_file,
                          "SELECT id, NULLIF(zero_val, 0) as non_zero FROM nullif_test")
        
        assert len(result) == 4
        # All zero_val entries are 0 except row 3
        assert result[0]['non_zero'] is None or result[0]['non_zero'] == ''
        assert result[1]['non_zero'] is None or result[1]['non_zero'] == ''
        assert result[2]['non_zero'] == 10
        assert result[3]['non_zero'] is None or result[3]['non_zero'] == ''
    
    def test_nullif_with_different_values(self):
        """Test NULLIF returns first value when different."""
        result = run_query(self.test_file,
                          "SELECT id, NULLIF(value, 0) as value_or_null FROM nullif_test")
        
        assert len(result) == 4
        assert result[0]['value_or_null'] == 100
        assert result[1]['value_or_null'] is None or result[1]['value_or_null'] == ''
        assert result[2]['value_or_null'] == 50
        assert result[3]['value_or_null'] == -1
    
    def test_nullif_with_strings(self):
        """Test NULLIF with string values."""
        result = run_query(self.test_file,
                          "SELECT id, NULLIF(status, 'deleted') as active_status FROM nullif_test")
        
        assert len(result) == 4
        assert result[0]['active_status'] == 'active'
        assert result[1]['active_status'] is None or result[1]['active_status'] == ''
        assert result[2]['active_status'] == 'active'
        assert result[3]['active_status'] is None or result[3]['active_status'] == ''


class TestComparisonEdgeCases:
    """Test edge cases and type mixing."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        cls.mixed_file = os.path.join(cls.temp_dir, "mixed.csv")
        with open(cls.mixed_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'int_val', 'float_val'])
            writer.writerow([1, 10, 10.5])
            writer.writerow([2, 15, 14.9])
            writer.writerow([3, 20, 20.0])
    
    def test_mixed_int_float_comparison(self):
        """Test GREATEST/LEAST with mixed integer and float types."""
        result = run_query(self.mixed_file,
                          "SELECT id, "
                          "GREATEST(int_val, float_val) as max_val, "
                          "LEAST(int_val, float_val) as min_val "
                          "FROM mixed")
        
        assert len(result) == 3
        # Row 1: GREATEST(10, 10.5) = 10.5
        assert result[0]['max_val'] == 10.5
        assert result[0]['min_val'] == 10
        # Row 2: GREATEST(15, 14.9) = 15
        assert result[1]['max_val'] == 15
        assert result[1]['min_val'] == 14.9
        # Row 3: GREATEST(20, 20.0) = 20.0 (or 20, both are equal)
        assert result[2]['max_val'] in [20, 20.0]
        assert result[2]['min_val'] in [20, 20.0]
    
    def test_single_argument(self):
        """Test GREATEST/LEAST with single argument."""
        result = run_query(self.mixed_file,
                          "SELECT GREATEST(42) as single_greatest, "
                          "LEAST(42) as single_least FROM mixed LIMIT 1")
        
        assert len(result) == 1
        assert result[0]['single_greatest'] == 42
        assert result[0]['single_least'] == 42


if __name__ == "__main__":
    pytest.main([__file__, "-v"])