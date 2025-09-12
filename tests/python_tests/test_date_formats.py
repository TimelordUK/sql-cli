#!/usr/bin/env python3
"""
Test suite for date format handling with US and European notation preferences.
Tests DATEDIFF, DATEADD, and WHERE clause date comparisons with both formats.
"""

import subprocess
import json
import os
import tempfile
import pytest
import shutil
from pathlib import Path

# Get the path to the sql-cli binary
SQL_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"

def run_query(csv_file, query, date_notation="us"):
    """Run a SQL query and return the results."""
    cmd = [str(SQL_CLI), csv_file, "-q", query, "-o", "json"]
    
    # Set environment variable to override date notation
    env = os.environ.copy()
    env["SQL_CLI_DATE_NOTATION"] = date_notation
    
    result = subprocess.run(cmd, capture_output=True, text=True, env=env)
    
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse JSON output - the output is a JSON array
    lines = result.stdout.strip().split('\n')
    # Filter out comment lines (those starting with #)
    json_lines = [line for line in lines if line and not line.startswith('#')]
    
    # Join the lines to form the complete JSON array
    json_text = '\n'.join(json_lines)
    
    if not json_text or json_text == '[]':
        return []
    
    # Parse the JSON array
    return json.loads(json_text)

def create_test_csv(filename, content):
    """Create a test CSV file with the given content."""
    with open(filename, 'w') as f:
        f.write(content)

class TestDateFormats:
    """Test date parsing with different notation preferences."""
    
    @classmethod
    def setup_class(cls):
        """Create test data files."""
        # Test data with ambiguous dates
        cls.test_csv = "test_dates.csv"
        create_test_csv(cls.test_csv, """id,start_date,end_date,description
1,01/02/2024 09:00:00,03/02/2024 17:00:00,February in US / January-February in EU
2,02/01/2024 10:00:00,04/01/2024 18:00:00,January in US / February in EU
3,12/06/2024 14:00:00,15/06/2024 16:00:00,June in US / December in EU
4,06/12/2024 15:00:00,08/12/2024 17:00:00,December in US / June in EU
5,15/03/2024 11:00:00,20/03/2024 12:00:00,March unambiguous
6,25/12/2024 08:00:00,31/12/2024 23:59:59,December unambiguous""")
        
        # No longer need config files - using environment variable instead
    
    @classmethod
    def teardown_class(cls):
        """Clean up test files."""
        if os.path.exists(cls.test_csv):
            os.remove(cls.test_csv)
    
    def test_datediff_us_format(self):
        """Test DATEDIFF with US date format."""
        query = """
        SELECT id, start_date, end_date, 
               DATEDIFF('day', start_date, end_date) as days_diff
        FROM test_dates
        """
        results = run_query(self.test_csv, query, "us")
        
        # With US format:
        # Row 1: 01/02 = Jan 2, 03/02 = Mar 2 -> 60 days
        # Row 2: 02/01 = Feb 1, 04/01 = Apr 1 -> 60 days
        # Row 3: 12/06 = Dec 6, 15/06 = Jun 15 (next year) -> 191 days (assuming 2025)
        # Row 4: 06/12 = Jun 12, 08/12 = Aug 12 -> 61 days
        # Row 5: 15/03 = Mar 15, 20/03 = Mar 20 -> 5 days
        # Row 6: 25/12 = Dec 25, 31/12 = Dec 31 -> 6 days
        
        assert len(results) == 6
        assert results[0]['days_diff'] == 60  # Jan 2 to Mar 2
        assert results[1]['days_diff'] == 60  # Feb 1 to Apr 1
        # Row 3: With US format, 12/06 = Dec 6, but 15/06 is invalid (month 15 doesn't exist)
        # So it falls back to European format: 15/06 = Jun 15
        # Dec 6 to Jun 15 = -174 days (Jun 15 is before Dec 6 in same year)
        assert results[2]['days_diff'] == -173  # Dec 6 to Jun 15 (same year)
        assert results[3]['days_diff'] == 61  # Jun 12 to Aug 12
        assert results[4]['days_diff'] == 5   # Mar 15 to Mar 20
        assert results[5]['days_diff'] == 6   # Dec 25 to Dec 31
    
    def test_datediff_eu_format(self):
        """Test DATEDIFF with European date format."""
        query = """
        SELECT id, start_date, end_date, 
               DATEDIFF('day', start_date, end_date) as days_diff
        FROM test_dates
        """
        results = run_query(self.test_csv, query, "european")
        
        # With EU format:
        # Row 1: 01/02 = Feb 1, 03/02 = Feb 3 -> 2 days
        # Row 2: 02/01 = Jan 2, 04/01 = Jan 4 -> 2 days  
        # Row 3: 12/06 = Jun 12, 15/06 = Jun 15 -> 3 days
        # Row 4: 06/12 = Dec 6, 08/12 = Dec 8 -> 2 days
        # Row 5: 15/03 = Mar 15, 20/03 = Mar 20 -> 5 days
        # Row 6: 25/12 = Dec 25, 31/12 = Dec 31 -> 6 days
        
        assert len(results) == 6
        assert results[0]['days_diff'] == 2   # Feb 1 to Feb 3
        assert results[1]['days_diff'] == 2   # Jan 2 to Jan 4
        assert results[2]['days_diff'] == 3   # Jun 12 to Jun 15
        assert results[3]['days_diff'] == 2   # Dec 6 to Dec 8
        assert results[4]['days_diff'] == 5   # Mar 15 to Mar 20
        assert results[5]['days_diff'] == 6   # Dec 25 to Dec 31
    
    def test_where_clause_us_format(self):
        """Test WHERE clause date comparisons with US format."""
        # Query for dates after June 1, 2024
        query = """
        SELECT id, start_date FROM test_dates 
        WHERE start_date > '06/01/2024 00:00:00'
        ORDER BY id
        """
        results = run_query(self.test_csv, query, "us")
        
        # With US format, 06/01 = June 1
        # Rows that should match:
        # Row 3: 12/06/2024 = Dec 6, 2024 (after June 1)
        # Row 4: 06/12/2024 = June 12, 2024 (after June 1)
        # Row 6: 25/12/2024 = Dec 25, 2024 (after June 1)
        
        assert len(results) == 3
        assert results[0]['id'] == 3
        assert results[1]['id'] == 4
        assert results[2]['id'] == 6
    
    def test_where_clause_eu_format(self):
        """Test WHERE clause date comparisons with European format."""
        # Query for dates after June 1, 2024  
        query = """
        SELECT id, start_date FROM test_dates 
        WHERE start_date > '01/06/2024 00:00:00'
        ORDER BY id
        """
        results = run_query(self.test_csv, query, "european")
        
        # With EU format, 01/06 = June 1
        # Rows that should match:
        # Row 3: 12/06/2024 = June 12, 2024 (after June 1)
        # Row 4: 06/12/2024 = Dec 6, 2024 (after June 1)
        # Row 6: 25/12/2024 = Dec 25, 2024 (after June 1)
        
        assert len(results) == 3
        assert results[0]['id'] == 3
        assert results[1]['id'] == 4
        assert results[2]['id'] == 6
    
    def test_dateadd_us_format(self):
        """Test DATEADD with US date format."""
        query = """
        SELECT id, start_date,
               DATEADD('day', 10, start_date) as plus_10_days
        FROM test_dates
        WHERE id = 1
        """
        results = run_query(self.test_csv, query, "us")
        
        # With US format: 01/02/2024 = Jan 2, 2024
        # Plus 10 days = Jan 12, 2024
        assert len(results) == 1
        assert '2024-01-12' in results[0]['plus_10_days']
    
    def test_dateadd_eu_format(self):
        """Test DATEADD with European date format."""
        query = """
        SELECT id, start_date,
               DATEADD('day', 10, start_date) as plus_10_days
        FROM test_dates
        WHERE id = 1
        """
        results = run_query(self.test_csv, query, "european")
        
        # With EU format: 01/02/2024 = Feb 1, 2024
        # Plus 10 days = Feb 11, 2024
        assert len(results) == 1
        assert '2024-02-11' in results[0]['plus_10_days']
    
    def test_mixed_operations_us(self):
        """Test combined date operations with US format."""
        query = """
        SELECT id, 
               DATEDIFF('hour', start_date, end_date) as hours_diff,
               DATEADD('month', 1, start_date) as next_month
        FROM test_dates
        WHERE start_date >= '01/01/2024 00:00:00' 
          AND start_date <= '03/31/2024 23:59:59'
        ORDER BY id
        """
        results = run_query(self.test_csv, query, "us")
        
        # With US format, date range is Jan 1 - Mar 31
        # Should include rows 1, 2, 5
        assert len(results) == 3
        assert results[0]['id'] == 1
        assert results[1]['id'] == 2
        assert results[2]['id'] == 5
    
    def test_mixed_operations_eu(self):
        """Test combined date operations with European format."""
        query = """
        SELECT id, 
               DATEDIFF('hour', start_date, end_date) as hours_diff,
               DATEADD('month', 1, start_date) as next_month
        FROM test_dates
        WHERE start_date >= '01/01/2024 00:00:00' 
          AND start_date <= '31/03/2024 23:59:59'
        ORDER BY id
        """
        results = run_query(self.test_csv, query, "european")
        
        # With EU format, date range is Jan 1 - Mar 31  
        # Should include rows 1, 2, 5
        assert len(results) == 3
        assert results[0]['id'] == 1
        assert results[1]['id'] == 2
        assert results[2]['id'] == 5
    
    def test_iso_dates_consistent(self):
        """Test that ISO format dates work the same regardless of config."""
        query = """
        SELECT DATEDIFF('day', '2024-03-15 10:00:00', '2024-03-20 10:00:00') as diff
        """
        
        us_results = run_query(self.test_csv, query, "us")
        eu_results = run_query(self.test_csv, query, "european")
        
        assert us_results[0]['diff'] == 5
        assert eu_results[0]['diff'] == 5
        assert us_results[0]['diff'] == eu_results[0]['diff']

if __name__ == "__main__":
    # Run the tests
    pytest.main([__file__, "-v"])