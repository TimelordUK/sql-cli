#!/usr/bin/env python3
"""Test type checking functions: IS_DATE, IS_BOOL, IS_NUMERIC, IS_INTEGER, IS_FLOAT, IS_NULL, IS_NOT_NULL"""

import subprocess
import pytest
from pathlib import Path
import tempfile
import os

class TestTypeCheckingFunctions:
    """Test suite for type checking SQL functions"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
    
    def run_query(self, query: str, csv_file: str = None):
        """Helper to run a SQL query"""
        if csv_file:
            cmd = [self.sql_cli, csv_file, "-q", query, "-o", "csv"]
        else:
            cmd = [self.sql_cli, "-q", query, "-o", "csv"]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        assert result.returncode == 0, f"Query failed: {result.stderr}"
        
        lines = result.stdout.strip().split('\n')
        # Filter out comment lines
        data_lines = [line for line in lines if not line.startswith('#')]
        
        if len(data_lines) > 1:
            # Has header and data
            return data_lines[1:]  # Return data rows
        else:
            # Only header, no data
            return []
    
    def test_is_date_function(self):
        """Test IS_DATE function with various inputs"""
        # Test valid dates
        query = "SELECT IS_DATE('2024-01-01'), IS_DATE('01/15/2024'), IS_DATE('2024-03-10T14:30:00')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,true,true"
        
        # Test invalid dates
        query = "SELECT IS_DATE('not a date'), IS_DATE('2024-13-45'), IS_DATE('abc123')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,false,false"
        
        # Test with NULL
        query = "SELECT IS_DATE(NULL)"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false"
    
    def test_is_bool_function(self):
        """Test IS_BOOL function with various inputs"""
        # Test valid booleans
        query = "SELECT IS_BOOL('true'), IS_BOOL('false'), IS_BOOL('yes'), IS_BOOL('no'), IS_BOOL('1'), IS_BOOL('0')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,true,true,true,true,true"
        
        # Test invalid booleans
        query = "SELECT IS_BOOL('maybe'), IS_BOOL('123'), IS_BOOL('text')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,false,false"
    
    def test_is_numeric_function(self):
        """Test IS_NUMERIC function with various inputs"""
        # Test valid numbers
        query = "SELECT IS_NUMERIC('123'), IS_NUMERIC('123.45'), IS_NUMERIC('-99.99'), IS_NUMERIC('0')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,true,true,true"
        
        # Test invalid numbers
        query = "SELECT IS_NUMERIC('not a number'), IS_NUMERIC('12.34.56'), IS_NUMERIC('abc')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,false,false"
    
    def test_is_integer_function(self):
        """Test IS_INTEGER function with various inputs"""
        # Test valid integers
        query = "SELECT IS_INTEGER('123'), IS_INTEGER('-456'), IS_INTEGER('0')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,true,true"
        
        # Test non-integers
        query = "SELECT IS_INTEGER('123.45'), IS_INTEGER('text'), IS_INTEGER('1.0e10')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,false,false"
    
    def test_is_float_function(self):
        """Test IS_FLOAT function with various inputs"""
        # Test valid floats (including integers which can be represented as floats)
        query = "SELECT IS_FLOAT('123.45'), IS_FLOAT('123'), IS_FLOAT('-99.99'), IS_FLOAT('1.0e10')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,true,true,true"
        
        # Test non-floats
        query = "SELECT IS_FLOAT('not a float'), IS_FLOAT('12.34.56')"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,false"
    
    def test_is_null_and_not_null_functions(self):
        """Test IS_NULL and IS_NOT_NULL functions"""
        # Test IS_NULL
        query = "SELECT IS_NULL(NULL), IS_NULL('value'), IS_NULL(123)"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "true,false,false"
        
        # Test IS_NOT_NULL
        query = "SELECT IS_NOT_NULL(NULL), IS_NOT_NULL('value'), IS_NOT_NULL(123)"
        result = self.run_query(query)
        assert len(result) == 1
        assert result[0] == "false,true,true"
    
    def test_type_checking_with_csv_data(self):
        """Test type checking functions with CSV data"""
        # Create a temporary CSV file with mixed data types
        csv_content = """id,value,date_col,bool_col,numeric_col
1,string_value,2024-01-15,true,123
2,another_string,not_a_date,false,456.78
3,123,2024-02-20,yes,not_numeric
4,456.78,invalid_date,maybe,999
5,,2024-03-10,1,
"""
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(csv_content)
            csv_file = f.name
        
        try:
            # Test filtering by date validity
            query = f"SELECT COUNT(*) FROM {Path(csv_file).stem} WHERE IS_DATE(date_col) = true"
            result = self.run_query(query, csv_file)
            assert len(result) == 1
            assert result[0] == "3"
            
            # Test filtering by boolean validity
            query = f"SELECT COUNT(*) FROM {Path(csv_file).stem} WHERE IS_BOOL(bool_col) = true"
            result = self.run_query(query, csv_file)
            assert len(result) == 1
            assert result[0] == "4"
            
            # Test filtering by numeric validity
            query = f"SELECT COUNT(*) FROM {Path(csv_file).stem} WHERE IS_NUMERIC(numeric_col) = true"
            result = self.run_query(query, csv_file)
            assert len(result) == 1
            assert result[0] == "3"
            
            # Test combined conditions
            query = f"SELECT id FROM {Path(csv_file).stem} WHERE IS_DATE(date_col) = true AND IS_NUMERIC(numeric_col) = true"
            result = self.run_query(query, csv_file)
            assert len(result) == 1  # Only row 1 has both valid date and numeric
            
        finally:
            # Clean up temporary file
            os.unlink(csv_file)
    
    def test_type_checking_in_case_expressions(self):
        """Test using type checking functions in CASE expressions"""
        # Create a temporary CSV file
        csv_content = """id,mixed_col
1,2024-01-15
2,123
3,true
4,not_valid
5,456.78
"""
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(csv_content)
            csv_file = f.name
        
        try:
            # Test CASE with type checking
            query = f"""
            SELECT 
                id,
                mixed_col,
                CASE 
                    WHEN IS_DATE(mixed_col) THEN 'Date'
                    WHEN IS_INTEGER(mixed_col) THEN 'Integer'
                    WHEN IS_FLOAT(mixed_col) THEN 'Float'
                    WHEN IS_BOOL(mixed_col) THEN 'Boolean'
                    ELSE 'Other'
                END
            FROM {Path(csv_file).stem}
            """
            result = self.run_query(query, csv_file)
            assert len(result) == 5
            
            # Check the type classifications
            expected_types = ['Date', 'Integer', 'Boolean', 'Other', 'Float']
            for i, row in enumerate(result):
                cols = row.split(',')
                assert cols[2] == expected_types[i], f"Row {i+1}: Expected {expected_types[i]}, got {cols[2]}"
            
        finally:
            # Clean up temporary file
            os.unlink(csv_file)
    
    def test_type_checking_for_conditional_date_operations(self):
        """Test the specific use case: using DATEDIFF only on valid dates"""
        # Create a CSV with mixed date and non-date values
        csv_content = """order_id,amount,order_date
101,150.00,2024-01-15
102,200.50,2024-02-20
103,75.25,not_a_date
104,300.00,2024-03-10
105,125.75,invalid
106,450.00,2024-04-01
"""
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write(csv_content)
            csv_file = f.name
        
        try:
            # Filter and calculate DATEDIFF only for valid dates
            query = f"""
            SELECT 
                order_id, 
                amount, 
                order_date,
                DATEDIFF('day', order_date, '2024-12-31')
            FROM {Path(csv_file).stem}
            WHERE IS_DATE(order_date) = true
            """
            result = self.run_query(query, csv_file)
            
            # Should only get 4 rows with valid dates
            assert len(result) == 4
            
            # Verify the date differences are correct
            expected_days = ['351', '315', '296', '274']  # Days from each date to 2024-12-31
            for i, row in enumerate(result):
                cols = row.split(',')
                assert cols[3] == expected_days[i], f"Row {i}: Expected {expected_days[i]} days, got {cols[3]}"
            
        finally:
            # Clean up temporary file
            os.unlink(csv_file)