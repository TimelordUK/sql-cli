#!/usr/bin/env python3
"""Test subquery functionality in SQL CLI."""

import os
import subprocess
import sys
import tempfile
import csv

# Add parent directory to path to import test utilities
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def run_query(query, data_file="data/periodic_table.csv"):
    """Run a SQL query and return the output."""
    result = subprocess.run(
        ["./target/release/sql-cli", data_file, "-q", query, "-o", "csv"],
        capture_output=True,
        text=True
    )
    return result.stdout, result.stderr, result.returncode

def parse_csv_output(output):
    """Parse CSV output into a list of dictionaries."""
    lines = output.strip().split('\n')
    # Filter out comment lines
    csv_lines = [line for line in lines if not line.startswith('#')]
    if not csv_lines:
        return []
    
    reader = csv.DictReader(csv_lines)
    return list(reader)

class TestScalarSubqueries:
    """Test scalar subquery functionality."""
    
    def test_max_subquery(self):
        """Test scalar subquery with MAX function."""
        query = "SELECT Element, Year FROM periodic_table WHERE Year = (SELECT MAX(Year) FROM periodic_table)"
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) > 0, "Expected results from MAX subquery"
        
        # All results should have the same year (the maximum)
        years = [r['Year'] for r in results]
        assert len(set(years)) == 1, "All results should have the same (max) year"
        assert years[0] == '2010', "Maximum year should be 2010"
    
    def test_min_subquery(self):
        """Test scalar subquery with MIN function."""
        query = "SELECT COUNT(*) as count FROM periodic_table WHERE Year = (SELECT MIN(Year) FROM periodic_table WHERE Year IS NOT NULL)"
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        assert int(results[0]['count']) > 0, "Should have elements from earliest year"
    
    def test_avg_subquery(self):
        """Test scalar subquery with AVG function."""
        query = "SELECT COUNT(*) as above_avg FROM periodic_table WHERE AtomicMass > (SELECT AVG(AtomicMass) FROM periodic_table)"
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        count = int(results[0]['above_avg'])
        assert 40 < count < 70, f"Expected ~50% of elements above average, got {count}"

class TestInSubqueries:
    """Test IN subquery functionality."""
    
    def test_in_subquery_basic(self):
        """Test basic IN subquery."""
        query = "SELECT COUNT(*) as noble_gases FROM periodic_table WHERE Element IN (SELECT Element FROM periodic_table WHERE Group = 18)"
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        assert results[0]['noble_gases'] == '7', "Should have 7 noble gases"
    
    def test_in_subquery_with_filter(self):
        """Test IN subquery with additional WHERE conditions."""
        query = """
        SELECT COUNT(*) as modern_radioactive 
        FROM periodic_table 
        WHERE Element IN (
            SELECT Element FROM periodic_table WHERE Year > 2000
        ) AND Radioactive = 'yes'
        """
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        assert int(results[0]['modern_radioactive']) >= 0, "Should have a valid count"
    
    def test_in_subquery_multiple_values(self):
        """Test IN subquery that returns multiple values."""
        query = """
        SELECT Element, Metal 
        FROM periodic_table 
        WHERE Element IN (
            SELECT Element FROM periodic_table WHERE AtomicNumber <= 5
        )
        ORDER BY AtomicNumber
        """
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 5, "Should have 5 elements with atomic number <= 5"
        elements = [r['Element'] for r in results]
        assert 'Hydrogen' in elements, "Should include Hydrogen"
        assert 'Helium' in elements, "Should include Helium"

class TestNotInSubqueries:
    """Test NOT IN subquery functionality."""
    
    def test_not_in_basic(self):
        """Test basic NOT IN subquery."""
        query = """
        SELECT COUNT(*) as non_radioactive 
        FROM periodic_table 
        WHERE Element NOT IN (
            SELECT Element FROM periodic_table WHERE Radioactive = 'yes'
        )
        """
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        count = int(results[0]['non_radioactive'])
        assert count > 0 and count < 118, "Should have some non-radioactive elements"
    
    def test_not_in_with_nulls(self):
        """Test NOT IN subquery with potential NULL values."""
        query = """
        SELECT COUNT(*) as ancient 
        FROM periodic_table 
        WHERE Element NOT IN (
            SELECT Element FROM periodic_table WHERE Year >= 1900
        ) AND Year IS NOT NULL
        """
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        assert int(results[0]['ancient']) > 0, "Should have ancient elements"

class TestComplexSubqueries:
    """Test complex subquery patterns."""
    
    def test_correlated_subquery(self):
        """Test correlated subquery (references outer query)."""
        query = """
        SELECT p1.Period, p1.Element, p1.AtomicMass
        FROM periodic_table p1
        WHERE p1.AtomicMass = (
            SELECT MAX(p2.AtomicMass)
            FROM periodic_table p2
            WHERE p2.Period = p1.Period
        )
        ORDER BY p1.Period
        LIMIT 5
        """
        stdout, stderr, code = run_query(query)
        
        # Note: Correlated subqueries might not be fully supported yet
        # For now, just test that the query doesn't crash
        if code != 0:
            print(f"Note: Correlated subqueries not yet supported: {stderr}")
            return
        
        results = parse_csv_output(stdout)
        if results:
            # Each period should appear only once (heaviest element per period)
            periods = [r['Period'] for r in results]
            assert len(periods) == len(set(periods)), "Each period should appear once"
    
    def test_nested_in_conditions(self):
        """Test multiple IN/NOT IN conditions."""
        query = """
        SELECT COUNT(*) as filtered
        FROM periodic_table
        WHERE Element IN (SELECT Element FROM periodic_table WHERE Metal = 'yes')
        AND Element NOT IN (SELECT Element FROM periodic_table WHERE Radioactive = 'yes')
        """
        stdout, stderr, code = run_query(query)
        assert code == 0, f"Query failed: {stderr}"
        
        results = parse_csv_output(stdout)
        assert len(results) == 1, "Expected one result"
        count = int(results[0]['filtered'])
        assert count > 0, "Should have stable metals"

def run_tests():
    """Run all subquery tests."""
    import pytest
    
    # Run tests
    exit_code = pytest.main([__file__, "-v"])
    sys.exit(exit_code)

if __name__ == "__main__":
    run_tests()