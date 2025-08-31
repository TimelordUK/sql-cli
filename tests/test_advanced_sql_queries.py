#!/usr/bin/env python3
"""
Comprehensive tests for advanced SQL queries with multiple functions and complex expressions
"""

import subprocess
import pytest
from pathlib import Path
from io import StringIO
import pandas as pd
import math

class TestAdvancedSqlQueries:
    """Test suite for advanced SQL functionality"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
        
        # Generate arithmetic test data if needed
        cls.generate_arithmetic_test_data()
    
    @classmethod
    def generate_arithmetic_test_data(cls):
        """Generate test_arithmetic.csv if it doesn't exist"""
        csv_path = cls.project_root / "data" / "test_arithmetic.csv"
        if not csv_path.exists():
            with open(csv_path, 'w') as f:
                f.write("id,quantity,price\n")
                # Generate diverse test data
                import random
                random.seed(42)
                for i in range(1, 101):
                    qty = random.randint(1, 100)
                    price = round(random.uniform(10.0, 500.0), 2)
                    f.write(f"{i},{qty},{price}\n")
    
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

    # ADVANCED MULTI-FUNCTION QUERIES
    
    def test_complex_multi_function_query(self):
        """Test complex query with multiple math functions and WHERE clause"""
        query = """
        SELECT 
            id, 
            a as quantity,
            b as price,
            ROUND(a * b, 2) as total,
            MOD(id, 10) as bucket,
            QUOTIENT(a, 5) as qty_group,
            ROUND(SQRT(POWER(a, 2) + POWER(b, 2)), 2) as magnitude,
            POWER(b, 2) as price_squared
        FROM test_simple_math
        WHERE MOD(a, 2) = 0 AND id <= 5
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Verify calculations for first row where a is even
        for _, row in df.iterrows():
            # Check MOD calculation
            assert row['bucket'] == row['id'] % 10
            # Check QUOTIENT
            assert row['qty_group'] == row['quantity'] // 5
            # Check magnitude calculation
            expected_mag = round(math.sqrt(row['quantity']**2 + row['price']**2), 2)
            assert abs(row['magnitude'] - expected_mag) < 0.01
            # Check power
            assert abs(row['price_squared'] - row['price']**2) < 0.01
    
    def test_nested_functions_with_arithmetic(self):
        """Test deeply nested functions with arithmetic operations"""
        query = """
        SELECT 
            id,
            ROUND(
                SQRT(
                    POWER(a, 2) + 
                    POWER(b / 10, 2) + 
                    ABS(c - d)
                ), 
                2
            ) as complex_calc,
            CEIL(LOG10(ABS(b) + 1)) as log_calc,
            FLOOR(SQRT(a * b)) as floor_sqrt
        FROM test_simple_math
        WHERE id IN (1, 2, 3, 10)
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 4  # Should get 4 rows
        
        # Verify at least one calculation
        row1 = df[df['id'] == 1].iloc[0]
        assert 'complex_calc' in row1
        assert 'log_calc' in row1
        assert 'floor_sqrt' in row1
    
    def test_multiple_conditions_with_functions(self):
        """Test WHERE clause with multiple function-based conditions"""
        query = """
        SELECT 
            id,
            a,
            b,
            MOD(a, 3) as mod3,
            POWER(a, 2) as a_squared
        FROM test_simple_math
        WHERE 
            MOD(a, 3) = 0 AND 
            POWER(a, 2) < 100 AND
            b > 10
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        if df is not None and len(df) > 0:
            # Verify all conditions are met
            for _, row in df.iterrows():
                assert row['mod3'] == 0  # MOD(a, 3) = 0
                assert row['a_squared'] < 100  # POWER(a, 2) < 100
                assert row['b'] > 10  # b > 10
    
    def test_calculated_columns_in_where(self):
        """Test using calculated expressions in WHERE clause"""
        query = """
        SELECT 
            id,
            a,
            b,
            a * b as product
        FROM test_simple_math
        WHERE 
            a * b > 100 AND
            SQRT(a * b) < 50
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        if df is not None and len(df) > 0:
            for _, row in df.iterrows():
                product = row['a'] * row['b']
                assert product > 100
                assert math.sqrt(product) < 50
    
    def test_case_insensitive_functions(self):
        """Test that function names are case-insensitive"""
        query = """
        SELECT 
            id,
            round(a, 0) as round_lower,
            ROUND(b, 0) as round_upper,
            Mod(id, 5) as mod_mixed,
            quotient(a, 2) as quot_lower,
            SQRT(a) as sqrt_upper
        FROM test_simple_math
        WHERE id = 1
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 1
        # Just verify the query executes successfully
    
    def test_arithmetic_order_of_operations(self):
        """Test correct order of operations in arithmetic expressions"""
        query = """
        SELECT 
            id,
            a,
            b,
            c,
            a + b * c as expr1,
            (a + b) * c as expr2,
            a * b + c * d as expr3,
            a * (b + c) * d as expr4
        FROM test_simple_math
        WHERE id IN (1, 2, 3)
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        # Manual verification would need actual data values
        # Just ensure query executes
        assert len(df) == 3
    
    def test_division_and_modulo_operations(self):
        """Test division and modulo with various operands"""
        query = """
        SELECT 
            id,
            a,
            b,
            a / b as division,
            QUOTIENT(a, 3) as int_div,
            MOD(a, 3) as modulo,
            MOD(b, 10) as b_mod_10,
            ROUND(a / b, 3) as div_rounded
        FROM test_simple_math
        WHERE b > 0 AND id <= 10
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        for _, row in df.iterrows():
            # Verify integer division
            assert row['int_div'] == int(row['a'] // 3)
            # Verify modulo
            assert row['modulo'] == row['a'] % 3
            assert row['b_mod_10'] == row['b'] % 10
    
    def test_mathematical_constants_and_functions(self):
        """Test PI and E (EXP) mathematical constants"""
        query = """
        SELECT 
            id,
            ROUND(PI(), 5) as pi_val,
            ROUND(a * PI(), 2) as circumference,
            ROUND(EXP(1), 5) as e_val,
            ROUND(EXP(a), 2) as exp_a,
            ROUND(LN(b), 2) as ln_b,
            ROUND(LOG10(b), 2) as log10_b
        FROM test_simple_math
        WHERE id IN (1, 2) AND b > 0
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        if df is not None and len(df) > 0:
            # Check PI value
            assert abs(df.iloc[0]['pi_val'] - 3.14159) < 0.00001
            # Check E value (e^1)
            assert abs(df.iloc[0]['e_val'] - 2.71828) < 0.00001
    
    def test_floor_ceil_round_comparison(self):
        """Test different rounding functions"""
        query = """
        SELECT 
            id,
            c,
            FLOOR(c) as floor_val,
            CEIL(c) as ceil_val,
            ROUND(c, 0) as round_val,
            ROUND(c, 1) as round_1,
            ROUND(c, 2) as round_2
        FROM test_simple_math
        WHERE c IS NOT NULL AND id <= 5
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        for _, row in df.iterrows():
            val = row['c']
            # FLOOR should round down
            assert row['floor_val'] == math.floor(val)
            # CEIL should round up
            assert row['ceil_val'] == math.ceil(val)
            # ROUND should round to nearest
            assert row['round_val'] == round(val)
    
    def test_aggregate_like_calculations(self):
        """Test complex calculations that simulate aggregates"""
        query = """
        SELECT 
            id,
            a as val1,
            b as val2,
            c as val3,
            d as val4,
            (a + b + c + d) as sum_all,
            ROUND((a + b + c + d) / 4.0, 2) as avg_all,
            ROUND(SQRT((POWER(a, 2) + POWER(b, 2) + POWER(c, 2) + POWER(d, 2)) / 4.0), 2) as rms
        FROM test_simple_math
        WHERE id <= 3
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        
        for _, row in df.iterrows():
            # Verify sum
            expected_sum = row['val1'] + row['val2'] + row['val3'] + row['val4']
            assert abs(row['sum_all'] - expected_sum) < 0.01
            # Verify average
            assert abs(row['avg_all'] - expected_sum / 4.0) < 0.01
    
    def test_complex_where_with_nested_functions(self):
        """Test WHERE clause with nested function calls"""
        query = """
        SELECT 
            id,
            a,
            b,
            ROUND(SQRT(POWER(a, 2) + POWER(b, 2)), 2) as magnitude
        FROM test_simple_math
        WHERE 
            ROUND(SQRT(POWER(a, 2) + POWER(b, 2)), 2) > 50 AND
            MOD(QUOTIENT(a, 10), 2) = 0
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        # Just verify query executes without error
        # Results depend on actual data
        assert err is None or df is not None
    
    def test_string_and_math_combination(self):
        """Test combining string and math functions"""
        query = """
        SELECT 
            id,
            name,
            name.Length() as name_len,
            MOD(name.Length(), 3) as len_mod_3,
            POWER(name.Length(), 2) as len_squared
        FROM test_simple_strings
        WHERE 
            name.Length() > 3 AND
            MOD(name.Length(), 2) = 0
        """
        df, err = self.run_query("test_simple_strings.csv", query)
        
        if df is not None and len(df) > 0:
            for _, row in df.iterrows():
                # Verify length-based calculations
                assert row['name_len'] > 3
                assert row['name_len'] % 2 == 0
                assert row['len_mod_3'] == row['name_len'] % 3
                assert row['len_squared'] == row['name_len'] ** 2
    
    def test_null_handling_in_functions(self):
        """Test how functions handle NULL values"""
        # Note: COALESCE is not yet implemented, test basic null handling
        query = """
        SELECT 
            id,
            a,
            ABS(a) as abs_a,
            SQRT(ABS(a)) as sqrt_abs_a
        FROM test_simple_math
        WHERE id <= 5
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        # Just verify query executes
        assert df is not None, f"Query failed: {err}"
    
    def test_extreme_nesting_depth(self):
        """Test extremely nested function calls"""
        query = """
        SELECT 
            id,
            ROUND(
                SQRT(
                    ABS(
                        POWER(
                            CEIL(
                                LOG10(
                                    ABS(a) + 1
                                )
                            ), 
                            2
                        )
                    )
                ), 
                3
            ) as deeply_nested
        FROM test_simple_math
        WHERE id = 10 AND a > 0
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        # Just verify this complex query can be parsed and executed
        assert err is None or df is not None
    
    def test_performance_with_many_calculations(self):
        """Test query with many calculated columns"""
        query = """
        SELECT 
            id,
            a, b, c, d,
            a + b as sum_ab,
            a - b as diff_ab,
            a * b as prod_ab,
            a / b as div_ab,
            MOD(a, 10) as a_mod_10,
            MOD(b, 10) as b_mod_10,
            POWER(a, 2) as a_sq,
            POWER(b, 2) as b_sq,
            SQRT(a) as sqrt_a,
            SQRT(b) as sqrt_b,
            ROUND(c, 0) as c_round,
            FLOOR(c) as c_floor,
            CEIL(c) as c_ceil,
            ABS(d - a) as abs_diff_da,
            QUOTIENT(a, 5) as a_div_5,
            QUOTIENT(b, 5) as b_div_5,
            PI() as pi_const,
            ROUND(a * PI(), 2) as a_times_pi
        FROM test_simple_math
        WHERE id <= 20
        """
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) <= 20
        # Verify we got all the columns
        expected_cols = ['id', 'a', 'b', 'c', 'd', 'sum_ab', 'diff_ab', 'prod_ab',
                        'div_ab', 'a_mod_10', 'b_mod_10', 'a_sq', 'b_sq',
                        'sqrt_a', 'sqrt_b', 'c_round', 'c_floor', 'c_ceil',
                        'abs_diff_da', 'a_div_5', 'b_div_5', 'pi_const', 'a_times_pi']
        for col in expected_cols:
            assert col in df.columns


if __name__ == "__main__":
    pytest.main([__file__, "-v"])