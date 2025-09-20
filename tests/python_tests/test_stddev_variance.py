"""Test STDDEV and VARIANCE aggregate functions."""
import subprocess
import json
import csv
import tempfile
import os
import pytest
import math


def run_sql_query(csv_file, query):
    """Execute a SQL query and return the results."""
    cmd = [
        "./target/release/sql-cli",
        csv_file,
        "-q", query,
        "-o", "csv"
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse CSV output
    lines = result.stdout.strip().split('\n')
    # Filter out comments and timing info
    csv_lines = [line for line in lines if not line.startswith('#') and line]
    
    if not csv_lines:
        return []
    
    # Parse as CSV
    reader = csv.DictReader(csv_lines)
    return list(reader)


class TestBasicVariance:
    """Test basic VARIANCE and STDDEV functionality."""
    
    @pytest.fixture
    def numeric_data(self):
        """Create test data with numeric values."""
        data = """value
10
20
30
40
50"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    def test_variance_simple(self, numeric_data):
        """Test VARIANCE on simple numeric data."""
        query = "SELECT VARIANCE(value) as variance FROM test"
        results = run_sql_query(numeric_data, query)
        
        assert len(results) == 1
        variance = float(results[0]['variance'])
        # Values: 10, 20, 30, 40, 50
        # Mean: 30
        # Sample Variance: ((10-30)² + (20-30)² + (30-30)² + (40-30)² + (50-30)²) / 4 = 250
        assert abs(variance - 250.0) < 0.001
    
    def test_stddev_simple(self, numeric_data):
        """Test STDDEV on simple numeric data."""
        query = "SELECT STDDEV(value) as stddev FROM test"
        results = run_sql_query(numeric_data, query)
        
        assert len(results) == 1
        stddev = float(results[0]['stddev'])
        # Sample StdDev = sqrt(250) ≈ 15.811
        assert abs(stddev - math.sqrt(250)) < 0.001
    
    def test_variance_and_stddev_together(self, numeric_data):
        """Test VARIANCE and STDDEV in same query."""
        query = """SELECT 
                      AVG(value) as mean,
                      VARIANCE(value) as variance, 
                      STDDEV(value) as stddev 
                   FROM test"""
        results = run_sql_query(numeric_data, query)
        
        assert len(results) == 1
        mean = float(results[0]['mean'])
        variance = float(results[0]['variance'])
        stddev = float(results[0]['stddev'])
        
        assert abs(mean - 30.0) < 0.001
        assert abs(variance - 250.0) < 0.001
        assert abs(stddev - math.sqrt(250)) < 0.001
        # Verify that stddev = sqrt(variance)
        assert abs(stddev - math.sqrt(variance)) < 0.001


class TestVarianceWithNulls:
    """Test VARIANCE and STDDEV with NULL values."""
    
    @pytest.fixture
    def data_with_nulls(self):
        """Create test data with NULL values."""
        data = """value
10

30

50"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    def test_variance_ignores_nulls(self, data_with_nulls):
        """Test that VARIANCE ignores NULL values."""
        query = "SELECT COUNT(value) as count, VARIANCE(value) as variance FROM test"
        results = run_sql_query(data_with_nulls, query)
        
        assert len(results) == 1
        count = int(results[0]['count'])
        variance = float(results[0]['variance'])
        
        # Should only count 3 non-null values
        assert count == 3
        # Values: 10, 30, 50
        # Mean: 30
        # Sample Variance: ((10-30)² + (30-30)² + (50-30)²) / 2 = (400 + 0 + 400) / 2 = 400
        assert abs(variance - 400.0) < 0.001
    
    def test_stddev_ignores_nulls(self, data_with_nulls):
        """Test that STDDEV ignores NULL values."""
        query = "SELECT STDDEV(value) as stddev FROM test"
        results = run_sql_query(data_with_nulls, query)
        
        assert len(results) == 1
        stddev = float(results[0]['stddev'])
        # Sample StdDev = sqrt(400) = 20.0
        assert abs(stddev - 20.0) < 0.001


class TestVarianceWithGroupBy:
    """Test VARIANCE and STDDEV with GROUP BY."""
    
    @pytest.fixture
    def grouped_data(self):
        """Create test data for grouping."""
        data = """category,value
A,10
A,20
A,30
B,5
B,15
B,25
C,100"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    def test_variance_with_group_by(self, grouped_data):
        """Test VARIANCE with GROUP BY."""
        query = """SELECT category, 
                          COUNT(*) as count,
                          AVG(value) as mean,
                          VARIANCE(value) as variance 
                   FROM test 
                   GROUP BY category 
                   ORDER BY category"""
        results = run_sql_query(grouped_data, query)
        
        assert len(results) == 3
        
        # Category A: [10, 20, 30], mean=20, sample variance=100
        assert results[0]['category'] == 'A'
        assert int(results[0]['count']) == 3
        assert abs(float(results[0]['mean']) - 20.0) < 0.001
        assert abs(float(results[0]['variance']) - 100.0) < 0.001
        
        # Category B: [5, 15, 25], mean=15, sample variance=100
        assert results[1]['category'] == 'B'
        assert int(results[1]['count']) == 3
        assert abs(float(results[1]['mean']) - 15.0) < 0.001
        assert abs(float(results[1]['variance']) - 100.0) < 0.001
        
        # Category C: [100], mean=100, variance=NULL (single value, n-1=0)
        assert results[2]['category'] == 'C'
        assert int(results[2]['count']) == 1
        assert abs(float(results[2]['mean']) - 100.0) < 0.001
        # Sample variance is NULL for single value
        assert results[2]['variance'] == '' or results[2]['variance'].lower() == 'null'
    
    def test_stddev_with_group_by(self, grouped_data):
        """Test STDDEV with GROUP BY."""
        query = """SELECT category, STDDEV(value) as stddev 
                   FROM test 
                   GROUP BY category 
                   ORDER BY category"""
        results = run_sql_query(grouped_data, query)
        
        assert len(results) == 3
        
        # Sample StdDev for A and B should be sqrt(100) = 10.0
        assert abs(float(results[0]['stddev']) - 10.0) < 0.001
        assert abs(float(results[1]['stddev']) - 10.0) < 0.001
        # StdDev for C should be NULL (single value)
        assert results[2]['stddev'] == '' or results[2]['stddev'].lower() == 'null'
    
    def test_variance_with_having(self, grouped_data):
        """Test VARIANCE with GROUP BY and HAVING."""
        query = """SELECT category, 
                          COUNT(*) as count,
                          VARIANCE(value) as variance 
                   FROM test 
                   GROUP BY category 
                   HAVING variance > 0
                   ORDER BY category"""
        results = run_sql_query(grouped_data, query)
        
        # Should only return A and B (C has NULL variance for sample)
        assert len(results) == 2
        assert results[0]['category'] == 'A'
        assert results[1]['category'] == 'B'


class TestVarianceEdgeCases:
    """Test edge cases for VARIANCE and STDDEV."""
    
    @pytest.fixture
    def single_value(self):
        """Create test data with single value."""
        data = """value
42"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    @pytest.fixture
    def same_values(self):
        """Create test data with all same values."""
        data = """value
10
10
10
10"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    def test_variance_single_value(self, single_value):
        """Test VARIANCE with single value (should be NULL for sample variance)."""
        query = "SELECT VARIANCE(value) as variance, COUNT(value) as count FROM test"
        results = run_sql_query(single_value, query)

        assert len(results) == 1
        # Count should be 1
        assert int(results[0]['count']) == 1
        # Sample variance is NULL for single value (n-1=0)
        assert results[0]['variance'] == '' or results[0]['variance'].lower() == 'null'
    
    def test_stddev_single_value(self, single_value):
        """Test STDDEV with single value (should be NULL for sample stddev)."""
        query = "SELECT STDDEV(value) as stddev, COUNT(value) as count FROM test"
        results = run_sql_query(single_value, query)

        assert len(results) == 1
        # Count should be 1
        assert int(results[0]['count']) == 1
        # Sample stddev is NULL for single value (n-1=0)
        assert results[0]['stddev'] == '' or results[0]['stddev'].lower() == 'null'
    
    def test_variance_same_values(self, same_values):
        """Test VARIANCE when all values are the same (should be 0)."""
        query = "SELECT VARIANCE(value) as variance FROM test"
        results = run_sql_query(same_values, query)
        
        assert len(results) == 1
        variance = float(results[0]['variance'])
        assert abs(variance - 0.0) < 0.001
    
    def test_stddev_same_values(self, same_values):
        """Test STDDEV when all values are the same (should be 0)."""
        query = "SELECT STDDEV(value) as stddev FROM test"
        results = run_sql_query(same_values, query)
        
        assert len(results) == 1
        stddev = float(results[0]['stddev'])
        assert abs(stddev - 0.0) < 0.001


class TestVarianceWithMixedTypes:
    """Test VARIANCE and STDDEV with mixed numeric types."""
    
    @pytest.fixture
    def mixed_numeric_data(self):
        """Create test data with integers and floats."""
        data = """value
10
20.5
30
40.5
50"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(data)
            return f.name
    
    def test_variance_mixed_types(self, mixed_numeric_data):
        """Test VARIANCE with mixed integer and float values."""
        query = "SELECT VARIANCE(value) as variance FROM test"
        results = run_sql_query(mixed_numeric_data, query)
        
        assert len(results) == 1
        variance = float(results[0]['variance'])
        # Mean = 30.2
        # Sample Variance calculation with mixed types
        mean = 30.2
        values = [10, 20.5, 30, 40.5, 50]
        expected_variance = sum((v - mean) ** 2 for v in values) / (len(values) - 1)  # Sample variance
        assert abs(variance - expected_variance) < 0.001
    
    def test_stddev_mixed_types(self, mixed_numeric_data):
        """Test STDDEV with mixed integer and float values."""
        query = "SELECT STDDEV(value) as stddev FROM test"
        results = run_sql_query(mixed_numeric_data, query)
        
        assert len(results) == 1
        stddev = float(results[0]['stddev'])
        # Sample StdDev should be sqrt of sample variance
        mean = 30.2
        values = [10, 20.5, 30, 40.5, 50]
        expected_variance = sum((v - mean) ** 2 for v in values) / (len(values) - 1)  # Sample variance
        expected_stddev = math.sqrt(expected_variance)
        assert abs(stddev - expected_stddev) < 0.001


if __name__ == "__main__":
    pytest.main([__file__, "-v"])