#!/usr/bin/env python3
"""
Comprehensive tests for CASE WHEN expression evaluation in non-interactive mode
"""

import subprocess
import pytest
import pandas as pd
from pathlib import Path
from io import StringIO
import math


class TestCaseWhenEvaluation:
    """Test suite for CASE WHEN expression evaluation functionality"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
    
    def run_query(self, csv_file: str, query: str):
        """Helper to run a SQL query and return DataFrame and error"""
        cmd = [
            self.sql_cli, 
            str(self.project_root / "data" / csv_file), 
            "-q", query, 
            "-o", "csv"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        
        if result.returncode != 0:
            return None, result.stderr.strip()
            
        if result.stdout.strip():
            return pd.read_csv(StringIO(result.stdout.strip())), None
        return pd.DataFrame(), None

    # BASIC CASE WHEN TESTS
    
    def test_basic_case_when_numeric(self):
        """Test basic CASE WHEN with numeric comparisons"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a > 5 THEN 'High' 
                WHEN a > 2 THEN 'Medium' 
                ELSE 'Low' 
            END as level 
        FROM test_simple_math 
        WHERE id <= 8
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 8
        assert "level" in df.columns
        
        # Verify logic for specific values
        for _, row in df.iterrows():
            if row['a'] > 5:
                assert row['level'] == 'High'
            elif row['a'] > 2:
                assert row['level'] == 'Medium'
            else:
                assert row['level'] == 'Low'
    
    def test_case_when_with_arithmetic(self):
        """Test CASE WHEN with arithmetic expressions in conditions"""
        query = """
        SELECT 
            id, 
            a, 
            b, 
            CASE 
                WHEN a * 2 > b THEN 'A Doubled Wins' 
                WHEN a + 5 > b THEN 'A Plus 5 Wins' 
                ELSE 'B Wins' 
            END as comparison 
        FROM test_simple_math 
        WHERE id <= 5
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 5
        
        # Verify arithmetic logic
        for _, row in df.iterrows():
            if row['a'] * 2 > row['b']:
                assert row['comparison'] == 'A Doubled Wins'
            elif row['a'] + 5 > row['b']:
                assert row['comparison'] == 'A Plus 5 Wins'
            else:
                assert row['comparison'] == 'B Wins'
    
    def test_nested_case_expressions(self):
        """Test nested CASE expressions"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a > 10 THEN 'Big' 
                WHEN a > 5 THEN 
                    CASE 
                        WHEN MOD(a, 2) = 0 THEN 'Medium Even' 
                        ELSE 'Medium Odd' 
                    END 
                ELSE 'Small' 
            END as category 
        FROM test_simple_math 
        WHERE id <= 12
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 12
        
        # Verify nested logic
        for _, row in df.iterrows():
            if row['a'] > 10:
                assert row['category'] == 'Big'
            elif row['a'] > 5:
                if row['a'] % 2 == 0:
                    assert row['category'] == 'Medium Even'
                else:
                    assert row['category'] == 'Medium Odd'
            else:
                assert row['category'] == 'Small'
    
    def test_multiple_case_expressions_same_query(self):
        """Test multiple CASE expressions in the same SELECT"""
        query = """
        SELECT 
            id, 
            a, 
            b, 
            CASE WHEN a > 5 THEN 'High A' ELSE 'Low A' END as a_level,
            CASE WHEN b > 50 THEN 'High B' ELSE 'Low B' END as b_level 
        FROM test_simple_math 
        WHERE id <= 6
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 6
        assert "a_level" in df.columns
        assert "b_level" in df.columns
        
        # Verify independent logic
        for _, row in df.iterrows():
            expected_a = 'High A' if row['a'] > 5 else 'Low A'
            expected_b = 'High B' if row['b'] > 50 else 'Low B'
            assert row['a_level'] == expected_a
            assert row['b_level'] == expected_b
    
    # COMPARISON OPERATORS TESTS
    
    def test_all_comparison_operators(self):
        """Test CASE WHEN with all comparison operators"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a >= 10 THEN 'GTE10'
                WHEN a <= 3 THEN 'LTE3'
                WHEN a = 5 THEN 'EQ5'
                WHEN a != 7 THEN 'NE7'
                ELSE 'Other'
            END as operator_test 
        FROM test_simple_math 
        WHERE id <= 15
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 15
        
        # Verify comparison logic
        for _, row in df.iterrows():
            a_val = row['a']
            if a_val >= 10:
                assert row['operator_test'] == 'GTE10'
            elif a_val <= 3:
                assert row['operator_test'] == 'LTE3'
            elif a_val == 5:
                assert row['operator_test'] == 'EQ5'
            elif a_val != 7:
                assert row['operator_test'] == 'NE7'
            else:
                assert row['operator_test'] == 'Other'
    
    def test_case_with_not_equal_operator(self):
        """Test CASE WHEN with != and <> operators"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a != 5 THEN 'Not Five'
                ELSE 'Is Five'
            END as ne_test,
            CASE 
                WHEN a <> 10 THEN 'Not Ten'
                ELSE 'Is Ten'
            END as ne_alt_test
        FROM test_simple_math 
        WHERE id <= 12
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify not-equal logic
        for _, row in df.iterrows():
            expected_ne = 'Not Five' if row['a'] != 5 else 'Is Five'
            expected_ne_alt = 'Not Ten' if row['a'] != 10 else 'Is Ten'
            assert row['ne_test'] == expected_ne
            assert row['ne_alt_test'] == expected_ne_alt
    
    # INTEGRATION WITH MATH FUNCTIONS
    
    def test_case_with_math_functions(self):
        """Test CASE WHEN with mathematical functions"""
        query = """
        SELECT 
            id, 
            c, 
            CASE 
                WHEN ROUND(c, 0) > 5 THEN 'Rounded High' 
                WHEN FLOOR(c) < 2 THEN 'Floor Low' 
                ELSE 'Middle' 
            END as math_case 
        FROM test_simple_math 
        WHERE id <= 8
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify math function integration
        for _, row in df.iterrows():
            c_val = row['c']
            if round(c_val, 0) > 5:
                assert row['math_case'] == 'Rounded High'
            elif math.floor(c_val) < 2:
                assert row['math_case'] == 'Floor Low'
            else:
                assert row['math_case'] == 'Middle'
    
    def test_case_with_power_and_mod_functions(self):
        """Test CASE WHEN with POWER and MOD functions"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN POWER(a, 2) > 100 THEN 'High Power'
                WHEN MOD(a, 3) = 0 THEN 'Divisible by 3'
                WHEN MOD(a, 2) = 1 THEN 'Odd Number'
                ELSE 'Even Small'
            END as power_mod_test 
        FROM test_simple_math 
        WHERE id <= 15
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify complex math function logic
        for _, row in df.iterrows():
            a_val = row['a']
            if a_val ** 2 > 100:
                assert row['power_mod_test'] == 'High Power'
            elif a_val % 3 == 0:
                assert row['power_mod_test'] == 'Divisible by 3'
            elif a_val % 2 == 1:
                assert row['power_mod_test'] == 'Odd Number'
            else:
                assert row['power_mod_test'] == 'Even Small'
    
    # EDGE CASES AND NULL HANDLING
    
    def test_case_without_else_clause(self):
        """Test CASE expression without ELSE clause (should return NULL)"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a = 5 THEN 'Five' 
                WHEN a = 10 THEN 'Ten' 
            END as special_numbers 
        FROM test_simple_math 
        WHERE id <= 15
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify NULL handling (pandas represents as NaN or empty string)
        for _, row in df.iterrows():
            a_val = row['a']
            if a_val == 5:
                assert row['special_numbers'] == 'Five'
            elif a_val == 10:
                assert row['special_numbers'] == 'Ten'
            else:
                # Should be NULL/NaN/empty
                assert pd.isna(row['special_numbers']) or row['special_numbers'] == ''
    
    def test_case_with_complex_conditions(self):
        """Test CASE with complex boolean conditions - LIMITATION: AND/OR not supported in CASE conditions"""
        # This test documents a known limitation - complex boolean expressions 
        # with AND/OR are not yet supported in CASE WHEN conditions
        
        # Test simple conditions that work
        query = """
        SELECT 
            id, 
            a, 
            b, 
            CASE 
                WHEN a > 15 THEN 'Very High A'
                WHEN b > 150 THEN 'Very High B'  
                WHEN a = b THEN 'Equal Values'
                ELSE 'Mixed'
            END as simple_condition 
        FROM test_simple_math 
        WHERE id <= 12
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify simple condition logic works
        for _, row in df.iterrows():
            a_val, b_val = row['a'], row['b']
            if a_val > 15:
                assert row['simple_condition'] == 'Very High A'
            elif b_val > 150:
                assert row['simple_condition'] == 'Very High B'
            elif a_val == b_val:
                assert row['simple_condition'] == 'Equal Values'
            else:
                assert row['simple_condition'] == 'Mixed'
        
        # Document the limitation - this should fail
        complex_query = """
        SELECT id, a, b,
            CASE WHEN a > 10 AND b > 100 THEN 'Both High' ELSE 'Other' END
        FROM test_simple_math WHERE id = 1
        """
        df_complex, err_complex = self.run_query("test_simple_math.csv", complex_query)
        
        # Assert that complex AND/OR conditions are not yet supported
        assert df_complex is None, "Complex AND/OR conditions should not be supported yet"
        assert "Parse error" in err_complex and "And" in err_complex
    
    # COMPLEX NESTED SCENARIOS
    
    def test_complex_nested_case_with_arithmetic(self):
        """Test complex nested CASE with arithmetic operations"""
        query = """
        SELECT 
            id, 
            a, 
            b, 
            CASE 
                WHEN a > 10 THEN 
                    CASE 
                        WHEN b > 100 THEN 'Big Both' 
                        ELSE 'Big A Only' 
                    END 
                WHEN a < 5 THEN 
                    CASE 
                        WHEN b < 30 THEN 'Small Both' 
                        ELSE 'Small A Big B' 
                    END 
                ELSE 'Medium A' 
            END as complex_case 
        FROM test_simple_math 
        WHERE id <= 12
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify complex nested logic
        for _, row in df.iterrows():
            a_val, b_val = row['a'], row['b']
            if a_val > 10:
                expected = 'Big Both' if b_val > 100 else 'Big A Only'
            elif a_val < 5:
                expected = 'Small Both' if b_val < 30 else 'Small A Big B'
            else:
                expected = 'Medium A'
            assert row['complex_case'] == expected
    
    def test_case_with_string_comparisons(self):
        """Test CASE WHEN with string data types"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a = 5 THEN 'Is Five' 
                WHEN a = 10 THEN 'Is Ten' 
                ELSE 'Other Number' 
            END as number_test 
        FROM test_simple_math 
        WHERE id <= 12
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify string output handling
        for _, row in df.iterrows():
            a_val = row['a']
            if a_val == 5:
                assert row['number_test'] == 'Is Five'
            elif a_val == 10:
                assert row['number_test'] == 'Is Ten'
            else:
                assert row['number_test'] == 'Other Number'
    
    # PERFORMANCE AND SCALE TESTS
    
    def test_case_performance_many_rows(self):
        """Test CASE WHEN performance with all available rows"""
        query = """
        SELECT 
            id, 
            a, 
            CASE 
                WHEN a >= 15 THEN 'Very High'
                WHEN a >= 10 THEN 'High' 
                WHEN a >= 5 THEN 'Medium' 
                ELSE 'Low' 
            END as performance_test 
        FROM test_simple_math
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        # Just verify it works with larger dataset
        assert len(df) > 0
        assert "performance_test" in df.columns
    
    def test_multiple_case_expressions_performance(self):
        """Test performance with multiple CASE expressions"""
        query = """
        SELECT 
            id, 
            a, 
            b, 
            c, 
            CASE WHEN a > 10 THEN 'H' ELSE 'L' END as a_cat,
            CASE WHEN b > 50 THEN 'H' ELSE 'L' END as b_cat,
            CASE WHEN c > 5 THEN 'H' ELSE 'L' END as c_cat,
            CASE WHEN a + b > 100 THEN 'H' ELSE 'L' END as sum_cat,
            CASE WHEN a * b > 500 THEN 'H' ELSE 'L' END as prod_cat
        FROM test_simple_math 
        WHERE id <= 20
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) <= 20
        
        # Verify all CASE columns exist
        case_cols = ['a_cat', 'b_cat', 'c_cat', 'sum_cat', 'prod_cat']
        for col in case_cols:
            assert col in df.columns
    
    # INTEGRATION TESTS
    
    def test_case_in_complex_query_with_where(self):
        """Test CASE WHEN in complex query with WHERE clause"""
        query = """
        SELECT 
            id, 
            a, 
            b,
            CASE 
                WHEN a * b > 200 THEN 'High Product'
                WHEN a + b > 50 THEN 'High Sum'
                ELSE 'Normal'
            END as classification
        FROM test_simple_math 
        WHERE 
            a > 2 AND 
            b < 200 AND
            id <= 15
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        # Query should execute successfully
        assert err is None or df is not None
        
        if df is not None and len(df) > 0:
            # Verify WHERE clause was applied
            for _, row in df.iterrows():
                assert row['a'] > 2
                assert row['b'] < 200
                assert row['id'] <= 15
                
                # Verify CASE logic
                if row['a'] * row['b'] > 200:
                    assert row['classification'] == 'High Product'
                elif row['a'] + row['b'] > 50:
                    assert row['classification'] == 'High Sum'
                else:
                    assert row['classification'] == 'Normal'
    
    # ERROR HANDLING TESTS
    
    def test_case_with_division_by_zero_protection(self):
        """Test CASE WHEN protecting against division by zero"""
        query = """
        SELECT 
            id, 
            a, 
            b,
            CASE 
                WHEN b = 0 THEN 'Division by Zero'
                WHEN a / b > 5 THEN 'High Ratio'
                ELSE 'Normal Ratio'
            END as ratio_check
        FROM test_simple_math 
        WHERE id <= 10
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify division protection logic
        for _, row in df.iterrows():
            if row['b'] == 0:
                assert row['ratio_check'] == 'Division by Zero'
            elif row['a'] / row['b'] > 5:
                assert row['ratio_check'] == 'High Ratio'
            else:
                assert row['ratio_check'] == 'Normal Ratio'


if __name__ == "__main__":
    pytest.main([__file__, "-v"])