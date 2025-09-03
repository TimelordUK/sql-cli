#!/usr/bin/env python3
"""
Test suite for boolean literal support in SQL
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


class TestBooleanLiterals:
    """Test boolean literal support in SQL."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        # Create data with boolean columns
        cls.bool_file = os.path.join(cls.temp_dir, "booleans.csv")
        with open(cls.bool_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'name', 'active', 'verified'])
            writer.writerow([1, 'Alice', 'true', 'false'])
            writer.writerow([2, 'Bob', 'false', 'true'])
            writer.writerow([3, 'Charlie', 'true', 'true'])
            writer.writerow([4, 'David', 'false', 'false'])
    
    def test_boolean_equals_true(self):
        """Test WHERE column = true."""
        result = run_query(self.bool_file, "SELECT * FROM booleans WHERE active = true")
        assert len(result) == 2
        names = [r['name'] for r in result]
        assert 'Alice' in names
        assert 'Charlie' in names
    
    def test_boolean_equals_false(self):
        """Test WHERE column = false."""
        result = run_query(self.bool_file, "SELECT * FROM booleans WHERE active = false")
        assert len(result) == 2
        names = [r['name'] for r in result]
        assert 'Bob' in names
        assert 'David' in names
    
    def test_boolean_not_equals(self):
        """Test WHERE column != boolean."""
        result = run_query(self.bool_file, "SELECT * FROM booleans WHERE active != true")
        assert len(result) == 2
        names = [r['name'] for r in result]
        assert 'Bob' in names
        assert 'David' in names
    
    def test_multiple_boolean_conditions(self):
        """Test multiple boolean conditions with AND/OR."""
        # Test AND
        result = run_query(self.bool_file, 
                          "SELECT * FROM booleans WHERE active = true AND verified = true")
        assert len(result) == 1
        assert result[0]['name'] == 'Charlie'
        
        # Test OR
        result = run_query(self.bool_file,
                          "SELECT * FROM booleans WHERE active = true OR verified = true")
        assert len(result) == 3
        names = [r['name'] for r in result]
        assert 'Alice' in names
        assert 'Bob' in names
        assert 'Charlie' in names
    
    def test_boolean_with_aggregates(self):
        """Test aggregates with boolean WHERE clauses."""
        result = run_query(self.bool_file,
                          "SELECT COUNT(*) as count FROM booleans WHERE active = true")
        assert result[0]['count'] == 2
        
        result = run_query(self.bool_file,
                          "SELECT COUNT(*) as total, SUM(id) as id_sum FROM booleans WHERE verified = false")
        assert result[0]['total'] == 2
        assert result[0]['id_sum'] == 5  # 1 + 4
    
    def test_case_insensitive_boolean_literals(self):
        """Test that TRUE/True/true all work."""
        # Test various capitalizations
        queries = [
            "SELECT COUNT(*) as count FROM booleans WHERE active = TRUE",
            "SELECT COUNT(*) as count FROM booleans WHERE active = True", 
            "SELECT COUNT(*) as count FROM booleans WHERE active = true"
        ]
        
        for query in queries:
            result = run_query(self.bool_file, query)
            assert result[0]['count'] == 2, f"Failed for query: {query}"
    
    def test_boolean_string_compatibility(self):
        """Test that boolean columns can still be compared with strings."""
        # This should work for backward compatibility
        result = run_query(self.bool_file, "SELECT * FROM booleans WHERE active = 'true'")
        assert len(result) == 2
        names = [r['name'] for r in result]
        assert 'Alice' in names
        assert 'Charlie' in names


class TestMixedBooleanOperations:
    """Test boolean operations with other data types."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        cls.mixed_file = os.path.join(cls.temp_dir, "mixed.csv")
        with open(cls.mixed_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'value', 'flag', 'category'])
            writer.writerow([1, 100, 'true', 'A'])
            writer.writerow([2, 200, 'false', 'B'])
            writer.writerow([3, 150, 'true', 'A'])
            writer.writerow([4, 300, 'false', 'B'])
            writer.writerow([5, 250, 'true', 'C'])
    
    def test_boolean_with_numeric_conditions(self):
        """Test boolean combined with numeric comparisons."""
        result = run_query(self.mixed_file,
                          "SELECT * FROM mixed WHERE flag = true AND value > 150")
        assert len(result) == 1  # Only row 5 has flag=true AND value > 150
        assert result[0]['id'] == 5
        assert result[0]['value'] == 250
    
    def test_boolean_with_string_conditions(self):
        """Test boolean combined with string comparisons."""
        result = run_query(self.mixed_file,
                          "SELECT * FROM mixed WHERE flag = false AND category = 'B'")
        assert len(result) == 2
        ids = [r['id'] for r in result]
        assert 2 in ids
        assert 4 in ids
    
    def test_complex_mixed_conditions(self):
        """Test complex conditions without parentheses."""
        # Note: Parentheses in WHERE clauses are not yet supported
        # Testing equivalent logic without parentheses
        result = run_query(self.mixed_file,
                          "SELECT * FROM mixed WHERE flag = true AND value <= 150")
        assert len(result) == 2
        ids = [r['id'] for r in result]
        assert 1 in ids  # flag=true, value=100
        assert 3 in ids  # flag=true, value=150


if __name__ == "__main__":
    pytest.main([__file__, "-v"])