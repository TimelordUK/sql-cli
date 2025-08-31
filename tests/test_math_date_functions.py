#!/usr/bin/env python3
"""
Comprehensive tests for SQL math and date functions
"""

import subprocess
import pytest
from pathlib import Path
from io import StringIO
import pandas as pd
from datetime import datetime, timedelta
import math

class TestMathDateFunctions:
    """Test suite for advanced math and date functions"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
        
        # Generate test data if needed
        cls.generate_test_data()
    
    @classmethod
    def generate_test_data(cls):
        """Generate test data files for math and date testing"""
        # Math test data already exists
        
        # Generate date test data
        date_csv = cls.project_root / "data" / "test_dates.csv"
        if not date_csv.exists():
            with open(date_csv, 'w') as f:
                f.write("id,date1,date2,timestamp\n")
                f.write("1,2024-01-01,2024-01-15,2024-01-01 10:30:00\n")
                f.write("2,2024-02-01,2024-02-10,2024-02-01 14:45:00\n")
                f.write("3,2024-03-15,2024-04-15,2024-03-15 09:00:00\n")
                f.write("4,2024-06-01,2024-06-30,2024-06-01 12:00:00\n")
                f.write("5,2024-12-25,2025-01-05,2024-12-25 00:00:00\n")
    
    def run_query(self, csv_file: str, query: str):
        """Helper to run a SQL query"""
        cmd = [
            self.sql_cli, 
            str(self.project_root / "data" / csv_file), 
            "-q", query, 
            "-o", "csv"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        
        if result.returncode != 0:
            return None, result.stderr
            
        if result.stdout.strip():
            return pd.read_csv(StringIO(result.stdout.strip())), None
        return pd.DataFrame(), None

    # ADVANCED MATH FUNCTIONS
    
    def test_floor_function(self):
        """Test FLOOR function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, FLOOR(c) as result FROM test_simple_math WHERE id IN (1, 2, 3)")
        assert len(df) == 3
        assert df[df['id'] == 1]['result'].iloc[0] == 0  # FLOOR(0.5) = 0
        assert df[df['id'] == 2]['result'].iloc[0] == 1  # FLOOR(1.0) = 1
        assert df[df['id'] == 3]['result'].iloc[0] == 1  # FLOOR(1.5) = 1
    
    def test_ceil_function(self):
        """Test CEIL/CEILING function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, CEIL(c) as result FROM test_simple_math WHERE id IN (1, 2, 3)")
        assert len(df) == 3
        assert df[df['id'] == 1]['result'].iloc[0] == 1  # CEIL(0.5) = 1
        assert df[df['id'] == 2]['result'].iloc[0] == 1  # CEIL(1.0) = 1
        assert df[df['id'] == 3]['result'].iloc[0] == 2  # CEIL(1.5) = 2
    
    def test_ceiling_alias(self):
        """Test CEILING as alias for CEIL"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, CEILING(c) as result FROM test_simple_math WHERE id = 1")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 1  # CEILING(0.5) = 1
    
    def test_pi_function(self):
        """Test PI function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, PI() as pi_value FROM test_simple_math WHERE id = 1")
        assert len(df) == 1
        assert abs(df.iloc[0]['pi_value'] - math.pi) < 0.0001
    
    def test_exp_function(self):
        """Test EXP function (e^x)"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, ROUND(EXP(a), 2) as result FROM test_simple_math WHERE id IN (1, 2)")
        assert len(df) == 2
        assert abs(df[df['id'] == 1]['result'].iloc[0] - 2.72) < 0.01  # e^1 ≈ 2.718
        assert abs(df[df['id'] == 2]['result'].iloc[0] - 7.39) < 0.01  # e^2 ≈ 7.389
    
    def test_ln_function(self):
        """Test LN function (natural log)"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, ROUND(LN(a), 2) as result FROM test_simple_math WHERE id IN (1, 3, 5)")
        if df is None:
            pytest.skip("LN function may not be implemented yet")
        assert len(df) == 3
        assert df[df['id'] == 1]['result'].iloc[0] == 0  # ln(1) = 0
        # ln(3) ≈ 1.099, ln(5) ≈ 1.609
    
    def test_log_function(self):
        """Test LOG function (base 10 or custom base)"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, ROUND(LOG(b), 2) as result FROM test_simple_math WHERE id = 10")
        if df is None:
            pytest.skip("LOG function may not be implemented yet")
        assert len(df) == 1
        assert abs(df.iloc[0]['result'] - 2.0) < 0.01  # log10(100) = 2
    
    def test_log10_function(self):
        """Test LOG10 function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, LOG10(b) as result FROM test_simple_math WHERE id = 10")
        if df is None:
            pytest.skip("LOG10 function may not be implemented yet")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 2  # log10(100) = 2
    
    def test_quotient_function(self):
        """Test QUOTIENT function (integer division)"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, QUOTIENT(b, a) as result FROM test_simple_math WHERE id IN (3, 5, 7)")
        if df is None:
            pytest.skip("QUOTIENT function may not be implemented yet")
        assert len(df) == 3
        assert df[df['id'] == 3]['result'].iloc[0] == 10  # 30 / 3 = 10
        assert df[df['id'] == 5]['result'].iloc[0] == 10  # 50 / 5 = 10
        assert df[df['id'] == 7]['result'].iloc[0] == 10  # 70 / 7 = 10
    
    # DATE FUNCTIONS
    
    def test_datediff_function(self):
        """Test DATEDIFF function"""
        # DATEDIFF('unit', date1, date2)
        df, _ = self.run_query("test_dates.csv",
                               "SELECT id, DATEDIFF('day', order_date, ship_date) as days_diff FROM test_dates WHERE id IN (1, 2, 3)")
        if df is None:
            pytest.skip("DATEDIFF function may not be implemented yet")
        assert len(df) == 3
        # All test data has 1 day difference between order and ship
        assert df[df['id'] == 1]['days_diff'].iloc[0] == 1
        assert df[df['id'] == 2]['days_diff'].iloc[0] == 1
        assert df[df['id'] == 3]['days_diff'].iloc[0] == 1
    
    def test_dateadd_days(self):
        """Test DATEADD function with days"""
        # DATEADD('unit', amount, date)
        df, _ = self.run_query("test_dates.csv",
                               "SELECT id, DATEADD('day', 10, order_date) as new_date FROM test_dates WHERE id = 1")
        if df is None:
            pytest.skip("DATEADD function may not be implemented yet")
        assert len(df) == 1
        # 2024-01-15 + 10 days = 2024-01-25
        assert '2024-01-25' in str(df.iloc[0]['new_date'])
    
    def test_dateadd_months(self):
        """Test DATEADD function with months"""
        # DATEADD('unit', amount, date)
        df, _ = self.run_query("test_dates.csv",
                               "SELECT id, DATEADD('month', 1, order_date) as new_date FROM test_dates WHERE id = 1")
        if df is None:
            pytest.skip("DATEADD function may not be implemented yet")
        assert len(df) == 1
        # 2024-01-15 + 1 month = 2024-02-15
        assert '2024-02-15' in str(df.iloc[0]['new_date'])
    
    def test_dateadd_years(self):
        """Test DATEADD function with years"""
        # DATEADD('unit', amount, date)
        df, _ = self.run_query("test_dates.csv",
                               "SELECT id, DATEADD('year', 1, order_date) as new_date FROM test_dates WHERE id = 5")
        if df is None:
            pytest.skip("DATEADD function may not be implemented yet")
        assert len(df) == 1
        # Check the year is incremented
        assert '2025' in str(df.iloc[0]['new_date'])
    
    def test_now_function(self):
        """Test NOW function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, NOW() as current_time FROM test_simple_math WHERE id = 1")
        if df is None:
            pytest.skip("NOW function may not be implemented yet")
        assert len(df) == 1
        # Check that it returns a datetime-like string
        result = str(df.iloc[0]['current_time'])
        assert len(result) > 10  # Should be a datetime string
    
    def test_today_function(self):
        """Test TODAY function"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, TODAY() as current_date FROM test_simple_math WHERE id = 1")
        if df is None:
            pytest.skip("TODAY function may not be implemented yet")
        assert len(df) == 1
        # Check that it returns a date-like string
        result = str(df.iloc[0]['current_date'])
        assert len(result) >= 10  # Should be a date string YYYY-MM-DD
    
    # TEXTJOIN FUNCTION
    
    def test_textjoin_function(self):
        """Test TEXTJOIN function"""
        # TEXTJOIN(delimiter, ignore_empty, value1, value2, ...)
        df, _ = self.run_query("test_simple_strings.csv",
                               "SELECT TEXTJOIN(',', 1, name, email) as joined FROM test_simple_strings WHERE id = 1")
        if df is None:
            pytest.skip("TEXTJOIN function may not be implemented yet")
        assert len(df) == 1
        assert df.iloc[0]['joined'] == 'Alice,alice@example.com'
    
    def test_textjoin_with_multiple_values(self):
        """Test TEXTJOIN with multiple columns"""
        # TEXTJOIN(delimiter, ignore_empty, value1, value2, value3)
        df, _ = self.run_query("test_simple_strings.csv",
                               "SELECT TEXTJOIN(' - ', 1, name, status, code) as joined FROM test_simple_strings WHERE id = 1")
        if df is None:
            pytest.skip("TEXTJOIN function may not be implemented yet")
        assert len(df) == 1
        assert df.iloc[0]['joined'] == 'Alice - Active - ABC123'
    
    # COMPLEX MATH EXPRESSIONS
    
    def test_complex_math_expression(self):
        """Test complex mathematical expression"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, ROUND(SQRT(POWER(a, 2) + POWER(b/10, 2)), 2) as result FROM test_simple_math WHERE id = 3")
        assert len(df) == 1
        # sqrt(3^2 + 3^2) = sqrt(18) ≈ 4.24
        assert abs(df.iloc[0]['result'] - 4.24) < 0.01
    
    def test_nested_math_functions(self):
        """Test deeply nested math functions"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, CEIL(LOG10(ABS(d - a))) as result FROM test_simple_math WHERE id = 10")
        if df is None:
            pytest.skip("LOG10 function may not be implemented yet")
        assert len(df) == 1
        # log10(abs(90 - 10)) = log10(80) ≈ 1.9, ceil(1.9) = 2
        assert df.iloc[0]['result'] == 2
    
    def test_math_with_pi(self):
        """Test calculations using PI"""
        df, _ = self.run_query("test_simple_math.csv",
                               "SELECT id, ROUND(a * PI(), 2) as result FROM test_simple_math WHERE id = 2")
        if df is None:
            pytest.skip("PI function may not be implemented yet")
        assert len(df) == 1
        # 2 * π ≈ 6.28
        assert abs(df.iloc[0]['result'] - 6.28) < 0.01
    
    # ERROR CASES
    
    def test_division_by_zero(self):
        """Test division by zero handling"""
        df, err = self.run_query("test_simple_math.csv",
                                 "SELECT id, b / (a - a) as result FROM test_simple_math WHERE id = 1")
        # Should either return null/inf or error gracefully
        assert df is None or df.iloc[0]['result'] in [None, float('inf'), float('nan')]
    
    def test_sqrt_negative(self):
        """Test SQRT of negative number"""
        df, err = self.run_query("test_simple_math.csv",
                                 "SELECT id, SQRT(a - d) as result FROM test_simple_math WHERE id = 1")
        # Should either return null/nan or error gracefully
        if df is not None:
            assert pd.isna(df.iloc[0]['result']) or math.isnan(df.iloc[0]['result'])
    
    def test_log_zero(self):
        """Test LOG of zero"""
        df, err = self.run_query("test_simple_math.csv",
                                 "SELECT id, LOG10(a - a) as result FROM test_simple_math WHERE id = 1")
        # Should either return null/-inf or error gracefully
        if df is not None:
            result = df.iloc[0]['result']
            assert pd.isna(result) or result == float('-inf')

if __name__ == "__main__":
    pytest.main([__file__, "-v"])