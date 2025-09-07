#!/usr/bin/env python3
"""Test Common Table Expressions (CTEs) and subqueries"""

import pytest
import subprocess
import csv
from io import StringIO
import os

# Path to the SQL CLI executable
SQL_CLI = os.path.join(os.path.dirname(__file__), "../../target/release/sql-cli")
TEST_DATA = os.path.join(os.path.dirname(__file__), "../../data/test_simple_math.csv")


def run_query(query, data_file=TEST_DATA):
    """Execute a query and return results as list of dicts"""
    result = subprocess.run(
        [SQL_CLI, data_file, "-q", query, "-o", "csv"],
        capture_output=True,
        text=True,
    )
    
    if result.returncode != 0:
        # If error, raise it for debugging
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse CSV output
    reader = csv.DictReader(StringIO(result.stdout))
    return list(reader)


class TestBasicCTE:
    """Test basic CTE functionality"""
    
    def test_simple_cte(self):
        """Test a simple CTE that selects columns"""
        query = "WITH t AS (SELECT a, b FROM test) SELECT * FROM t"
        results = run_query(query)
        
        assert len(results) == 20
        assert "a" in results[0]
        assert "b" in results[0]
        assert results[0]["a"] == "1"
        assert results[0]["b"] == "10"
    
    def test_cte_with_where(self):
        """Test CTE with WHERE clause in main query"""
        query = "WITH nums AS (SELECT a, b FROM test) SELECT * FROM nums WHERE a > 10"
        results = run_query(query)
        
        assert len(results) == 10
        assert all(int(row["a"]) > 10 for row in results)
    
    def test_cte_with_computed_column(self):
        """Test CTE with computed expressions"""
        query = """
        WITH calc AS (
            SELECT a, b, a * b as product 
            FROM test
        )
        SELECT * FROM calc WHERE product > 100
        """
        results = run_query(query)
        
        # Verify we can filter on the computed column
        for row in results:
            assert int(row["product"]) > 100
            assert int(row["product"]) == int(row["a"]) * int(row["b"])
    
    def test_cte_filtering_computed_expression(self):
        """Test the main use case: filtering on expressions not in WHERE"""
        # This was the problem we're solving - can't do WHERE IS_PRIME(n) directly
        query = """
        WITH prime_check AS (
            SELECT a, MOD(a, 2) = 0 as is_even
            FROM test
        )
        SELECT * FROM prime_check WHERE is_even = true
        """
        results = run_query(query)
        
        # Should only get even numbers
        for row in results:
            assert int(row["a"]) % 2 == 0
            assert row["is_even"] == "true"
    
    def test_multiple_ctes(self):
        """Test multiple CTEs in one query"""
        query = """
        WITH 
            first AS (SELECT a, b FROM test WHERE a <= 5),
            second AS (SELECT a, b FROM test WHERE a > 5 AND a <= 10)
        SELECT * FROM first
        """
        results = run_query(query)
        
        assert len(results) == 5
        assert all(int(row["a"]) <= 5 for row in results)
    
    def test_cte_with_aggregates(self):
        """Test CTE with aggregate functions"""
        query = """
        WITH summary AS (
            SELECT COUNT(*) as total, MAX(a) as max_a, MIN(a) as min_a
            FROM test
        )
        SELECT * FROM summary
        """
        results = run_query(query)
        
        assert len(results) == 1
        assert results[0]["total"] == "20"
        assert results[0]["max_a"] == "20"
        assert results[0]["min_a"] == "1"
    
    def test_cte_column_alias(self):
        """Test CTE with column aliases"""
        query = """
        WITH renamed AS (
            SELECT a as id, b as value
            FROM test
        )
        SELECT * FROM renamed WHERE id > 5
        """
        results = run_query(query)
        
        assert "id" in results[0]
        assert "value" in results[0]
        assert "a" not in results[0]
        assert "b" not in results[0]
        assert all(int(row["id"]) > 5 for row in results)


class TestCTEEdgeCases:
    """Test edge cases and error conditions"""
    
    def test_cte_reference_in_where(self):
        """Test that CTE columns are accessible in WHERE"""
        query = """
        WITH calc AS (
            SELECT a, a + b as sum_val
            FROM test
        )
        SELECT * FROM calc WHERE sum_val > 50
        """
        results = run_query(query)
        
        for row in results:
            # The sum should be greater than 50
            assert int(row["sum_val"]) > 50
    
    def test_cte_with_case_expression(self):
        """Test CTE with CASE expressions"""
        query = """
        WITH categorized AS (
            SELECT 
                a,
                CASE 
                    WHEN a <= 5 THEN 'small'
                    WHEN a <= 15 THEN 'medium'
                    ELSE 'large'
                END as category
            FROM test
        )
        SELECT * FROM categorized WHERE category = 'small'
        """
        results = run_query(query)
        
        assert len(results) == 5
        assert all(row["category"] == "small" for row in results)
        assert all(int(row["a"]) <= 5 for row in results)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])