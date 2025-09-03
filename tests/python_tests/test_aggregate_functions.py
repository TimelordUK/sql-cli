#!/usr/bin/env python3
"""
Test suite for SQL aggregate functions
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


class TestBasicAggregates:
    """Test basic aggregate functions."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        # Create numeric test data
        cls.numeric_file = os.path.join(cls.temp_dir, "numeric.csv")
        with open(cls.numeric_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'value', 'category'])
            writer.writerow([1, 10, 'A'])
            writer.writerow([2, 20, 'B'])
            writer.writerow([3, 30, 'A'])
            writer.writerow([4, 40, 'B'])
            writer.writerow([5, 50, 'A'])
        
        # Create data with nulls
        cls.nulls_file = os.path.join(cls.temp_dir, "nulls.csv")
        with open(cls.nulls_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'value', 'category'])
            writer.writerow([1, 10, 'A'])
            writer.writerow([2, '', 'B'])  # NULL value
            writer.writerow([3, 30, ''])   # NULL category
            writer.writerow([4, '', 'B'])  # NULL value
            writer.writerow([5, 50, 'A'])
    
    def test_count_star(self):
        """Test COUNT(*) function."""
        result = run_query(self.numeric_file, "SELECT COUNT(*) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 5
    
    def test_count_column(self):
        """Test COUNT(column) function."""
        result = run_query(self.nulls_file, "SELECT COUNT(value), COUNT(category), COUNT(*) FROM nulls")
        assert len(result) == 1
        # COUNT(value) should skip nulls
        assert result[0]['expr_1'] == 3  # Only 3 non-null values
        assert result[0]['expr_2'] == 4  # 4 non-null categories
        assert result[0]['expr_3'] == 5  # COUNT(*) counts all rows
    
    def test_sum(self):
        """Test SUM() function."""
        result = run_query(self.numeric_file, "SELECT SUM(value) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 150  # 10+20+30+40+50
    
    def test_sum_with_nulls(self):
        """Test SUM() with null values."""
        result = run_query(self.nulls_file, "SELECT SUM(value) FROM nulls")
        assert len(result) == 1
        assert result[0]['expr_1'] == 90  # 10+30+50 (nulls ignored)
    
    def test_avg(self):
        """Test AVG() function."""
        result = run_query(self.numeric_file, "SELECT AVG(value) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 30.0  # 150/5
    
    def test_avg_with_nulls(self):
        """Test AVG() with null values."""
        result = run_query(self.nulls_file, "SELECT AVG(value) FROM nulls")
        assert len(result) == 1
        assert result[0]['expr_1'] == 30.0  # 90/3 (only non-null values)
    
    def test_min_max(self):
        """Test MIN() and MAX() functions."""
        result = run_query(self.numeric_file, "SELECT MIN(value), MAX(value) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 10
        assert result[0]['expr_2'] == 50
    
    def test_min_max_strings(self):
        """Test MIN() and MAX() on string columns."""
        result = run_query(self.numeric_file, "SELECT MIN(category), MAX(category) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 'A'
        assert result[0]['expr_2'] == 'B'
    
    def test_multiple_aggregates(self):
        """Test multiple aggregates in one query."""
        result = run_query(self.numeric_file, 
                          "SELECT COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM numeric")
        assert len(result) == 1
        assert result[0]['expr_1'] == 5
        assert result[0]['expr_2'] == 150
        assert result[0]['expr_3'] == 30.0
        assert result[0]['expr_4'] == 10
        assert result[0]['expr_5'] == 50


class TestAggregatesWithExpressions:
    """Test aggregate functions with expressions."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        cls.sales_file = os.path.join(cls.temp_dir, "sales.csv")
        with open(cls.sales_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['product', 'quantity', 'price'])
            writer.writerow(['Apple', 10, 2.5])
            writer.writerow(['Banana', 5, 1.0])
            writer.writerow(['Orange', 8, 3.0])
            writer.writerow(['Apple', 15, 2.5])
            writer.writerow(['Banana', 12, 1.0])
    
    def test_sum_expression(self):
        """Test SUM() with expression."""
        result = run_query(self.sales_file, "SELECT SUM(quantity * price) FROM sales")
        assert len(result) == 1
        # (10*2.5) + (5*1.0) + (8*3.0) + (15*2.5) + (12*1.0) = 25+5+24+37.5+12 = 103.5
        assert result[0]['expr_1'] == pytest.approx(103.5)
    
    def test_avg_expression(self):
        """Test AVG() with expression."""
        result = run_query(self.sales_file, "SELECT AVG(quantity * price) FROM sales")
        assert len(result) == 1
        # 103.5 / 5 = 20.7
        assert result[0]['expr_1'] == pytest.approx(20.7)
    
    def test_complex_aggregates(self):
        """Test complex aggregate expressions."""
        result = run_query(self.sales_file, 
                          "SELECT SUM(quantity), AVG(price), SUM(quantity) * AVG(price) FROM sales")
        assert len(result) == 1
        assert result[0]['expr_1'] == 50  # Total quantity
        assert result[0]['expr_2'] == 2.0  # Avg price (2.5+1+3+2.5+1)/5
        assert result[0]['expr_3'] == 100.0  # 50 * 2.0


class TestAggregatesWithWhere:
    """Test aggregate functions with WHERE clause."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()
        
        cls.data_file = os.path.join(cls.temp_dir, "data.csv")
        with open(cls.data_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'value', 'category', 'active'])
            writer.writerow([1, 100, 'A', 'true'])
            writer.writerow([2, 200, 'B', 'false'])
            writer.writerow([3, 300, 'A', 'true'])
            writer.writerow([4, 400, 'B', 'true'])
            writer.writerow([5, 500, 'A', 'false'])
    
    def test_aggregates_with_where(self):
        """Test aggregates with WHERE filtering."""
        result = run_query(self.data_file, 
                          "SELECT COUNT(*), SUM(value), AVG(value) FROM data WHERE category = 'A'")
        assert len(result) == 1
        assert result[0]['expr_1'] == 3  # 3 rows with category A
        assert result[0]['expr_2'] == 900  # 100+300+500
        assert result[0]['expr_3'] == 300.0  # 900/3
    
    def test_aggregates_with_complex_where(self):
        """Test aggregates with complex WHERE clause."""
        # Note: Using numeric comparison since string comparison has separate issues
        result = run_query(self.data_file, 
                          "SELECT COUNT(*), SUM(value) FROM data WHERE value > 150 AND value < 450")
        assert len(result) == 1
        assert result[0]['expr_1'] == 3  # Rows 2, 3, 4 (200, 300, 400)
        assert result[0]['expr_2'] == 900  # 200+300+400


if __name__ == "__main__":
    pytest.main([__file__, "-v"])