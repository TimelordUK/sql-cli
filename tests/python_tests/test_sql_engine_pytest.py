#!/usr/bin/env python3
"""
SQL engine tests using pytest framework
"""

import subprocess
import pytest
from pathlib import Path
from io import StringIO
import pandas as pd
import json

class TestSqlEngine:
    """Test suite for SQL engine functionality"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
        
        # Generate test data if needed
        test_files = [
            cls.project_root / "data" / "test_simple_math.csv",
            cls.project_root / "data" / "test_simple_strings.csv"
        ]
        
        if not all(f.exists() for f in test_files):
            subprocess.run(["python3", str(cls.project_root / "scripts" / "generate_simple_test.py")],
                          cwd=cls.project_root, check=True)
    
    def run_query(self, csv_file: str, query: str):
        """Helper to run a SQL query"""
        cmd = [
            self.sql_cli, 
            str(self.project_root / "data" / csv_file), 
            "-q", query, 
            "-o", "csv"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        assert result.returncode == 0, f"Query failed: {result.stderr}"
        
        if result.stdout.strip():
            return pd.read_csv(StringIO(result.stdout.strip()))
        return pd.DataFrame()
    
    # ARITHMETIC TESTS
    
    def test_addition(self):
        """Test basic addition"""
        df = self.run_query("test_simple_math.csv", 
                           "SELECT id, a + b as result FROM test_simple_math WHERE id = 1")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 11
    
    def test_multiplication(self):
        """Test multiplication"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, a * b as result FROM test_simple_math WHERE id = 2")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 40
    
    def test_round_function(self):
        """Test ROUND function"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, ROUND(c, 0) as result FROM test_simple_math WHERE id = 3")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 2
    
    def test_abs_function(self):
        """Test ABS function"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, ABS(a - d) as result FROM test_simple_math WHERE id = 10")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 80
    
    def test_power_function(self):
        """Test POWER function"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, POWER(a, 2) as result FROM test_simple_math WHERE id = 5")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 25
    
    def test_sqrt_function(self):
        """Test SQRT function"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, SQRT(e) as result FROM test_simple_math WHERE id = 4")
        assert len(df) == 1
        assert abs(df.iloc[0]['result'] - 4) < 0.001
    
    def test_mod_function(self):
        """Test MOD function"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, MOD(b, 7) as result FROM test_simple_math WHERE id = 3")
        assert len(df) == 1
        assert df.iloc[0]['result'] == 2
    
    def test_complex_expression(self):
        """Test complex mathematical expression"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, ROUND((a + b) * c / 2, 1) as result FROM test_simple_math WHERE id = 2")
        assert len(df) == 1
        assert abs(df.iloc[0]['result'] - 11.0) < 0.1
    
    # STRING TESTS
    
    def test_contains_method(self):
        """Test string Contains method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id, name FROM test_simple_strings WHERE name.Contains('li')")
        assert len(df) == 2
        assert set(df['id'].tolist()) == {1, 3}
    
    def test_endswith_method(self):
        """Test string EndsWith method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id FROM test_simple_strings WHERE email.EndsWith('.com')")
        assert len(df) == 8
        # All emails ending with .com (including gmail.com and yahoo.com)
        assert set(df['id'].tolist()) == {1, 3, 4, 5, 6, 8, 9, 10}
    
    def test_startswith_method(self):
        """Test string StartsWith method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id FROM test_simple_strings WHERE status.StartsWith('A')")
        assert len(df) == 6
        # Active and Archived both start with 'A'
        assert set(df['id'].tolist()) == {1, 3, 5, 6, 7, 9}
    
    def test_trim_method(self):
        """Test string Trim method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id, name.Trim() as trimmed FROM test_simple_strings WHERE id = 4")
        assert len(df) == 1
        assert df.iloc[0]['trimmed'] == 'David'
    
    def test_length_method(self):
        """Test string Length method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id, name.Length() as len FROM test_simple_strings WHERE id = 1")
        assert len(df) == 1
        assert df.iloc[0]['len'] == 5
    
    def test_indexof_method(self):
        """Test string IndexOf method"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id, code.IndexOf('2') as pos FROM test_simple_strings WHERE id = 1")
        assert len(df) == 1
        assert df.iloc[0]['pos'] == 4
    
    # AGGREGATION TESTS
    
    @pytest.mark.skip(reason="Aggregate functions not yet implemented")
    def test_count_function(self):
        """Test COUNT aggregation"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT COUNT(*) as total FROM test_simple_math WHERE a <= 5")
        assert len(df) == 1
        assert df.iloc[0]['total'] == 5
    
    @pytest.mark.skip(reason="Aggregate functions not yet implemented")
    def test_sum_function(self):
        """Test SUM aggregation"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT SUM(a) as total FROM test_simple_math WHERE a <= 5")
        assert len(df) == 1
        assert df.iloc[0]['total'] == 15
    
    @pytest.mark.skip(reason="Aggregate functions not yet implemented")
    def test_avg_function(self):
        """Test AVG aggregation"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT ROUND(AVG(a), 1) as average FROM test_simple_math WHERE a <= 4")
        assert len(df) == 1
        assert abs(df.iloc[0]['average'] - 2.5) < 0.1
    
    # COMPLEX QUERIES
    
    def test_math_in_where(self):
        """Test mathematical expressions in WHERE clause"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id FROM test_simple_math WHERE a * b > 100")
        assert len(df) == 17  # IDs 4-20 have a*b > 100 (4*40=160, 5*50=250, etc.)
    
    def test_multiple_conditions(self):
        """Test multiple WHERE conditions"""
        df = self.run_query("test_simple_strings.csv",
                           "SELECT id FROM test_simple_strings WHERE status = 'Active' AND email.EndsWith('.com')")
        assert len(df) == 4
        # Active status and emails ending with .com (including gmail.com)
        assert set(df['id'].tolist()) == {1, 3, 5, 9}
    
    def test_order_by(self):
        """Test ORDER BY clause"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id, a FROM test_simple_math WHERE a <= 5 ORDER BY a DESC")
        assert len(df) == 5
        assert df['a'].tolist() == [5, 4, 3, 2, 1]
    
    def test_limit(self):
        """Test LIMIT clause"""
        df = self.run_query("test_simple_math.csv",
                           "SELECT id FROM test_simple_math ORDER BY id LIMIT 3")
        assert len(df) == 3
        assert df['id'].tolist() == [1, 2, 3]


if __name__ == "__main__":
    # Run with pytest
    pytest.main([__file__, "-v"])