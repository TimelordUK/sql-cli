#!/usr/bin/env python3
"""Test SQL comment support (-- and /* */)"""

import subprocess
import json
import pytest
from pathlib import Path
import tempfile

# Path to the SQL CLI binary
SQL_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"
TEST_DATA = Path(__file__).parent.parent.parent / "data" / "test_simple_math.csv"

def run_query(query, output_format="json"):
    """Run a SQL query and return the result"""
    cmd = [str(SQL_CLI), str(TEST_DATA), "-q", query, "-o", output_format]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Command failed: {' '.join(cmd)}")
        print(f"stderr: {result.stderr}")
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse JSON output
    if output_format == "json":
        lines = result.stdout.strip().split('\n')
        # Filter out comment lines and empty lines
        json_lines = [line for line in lines if not line.startswith('#') and line.strip()]
        # Join all non-comment lines to form complete JSON
        json_str = '\n'.join(json_lines)
        if json_str:
            return json.loads(json_str)
    return result.stdout

def run_query_file(sql_file_content, output_format="json"):
    """Run a SQL query from a file"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.sql', delete=False) as f:
        f.write(sql_file_content)
        temp_path = f.name
    
    try:
        cmd = [str(SQL_CLI), str(TEST_DATA), "-f", temp_path, "-o", output_format]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Command failed: {' '.join(cmd)}")
            print(f"stderr: {result.stderr}")
            raise Exception(f"Query failed: {result.stderr}")
        
        # Parse JSON output
        if output_format == "json":
            lines = result.stdout.strip().split('\n')
            json_lines = [line for line in lines if not line.startswith('#') and line.strip()]
            json_str = '\n'.join(json_lines)
            if json_str:
                return json.loads(json_str)
        return result.stdout
    finally:
        Path(temp_path).unlink()

class TestSingleLineComments:
    """Test single-line comments with --"""
    
    def test_simple_comment(self):
        """Test basic single-line comment"""
        result = run_query("SELECT 1 + 1 as result -- this is a comment")
        assert result[0]["result"] == 2
    
    def test_comment_at_start(self):
        """Test comment at the beginning"""
        query = """-- This is a comment at the start
        SELECT 2 * 3 as result"""
        result = run_query(query)
        assert result[0]["result"] == 6
    
    def test_multiple_comments(self):
        """Test multiple single-line comments"""
        query = """
        -- First comment
        SELECT 
            5 + 5 as sum, -- inline comment
            10 - 3 as diff -- another comment
        -- Final comment
        """
        result = run_query(query)
        assert result[0]["sum"] == 10
        assert result[0]["diff"] == 7
    
    def test_comment_with_sql_keywords(self):
        """Test that SQL keywords in comments are ignored"""
        query = """
        SELECT 1 as num -- SELECT * FROM WHERE AND OR NOT
        """
        result = run_query(query)
        assert result[0]["num"] == 1
    
    def test_multiple_dashes(self):
        """Test multiple dashes in comments"""
        query = """
        SELECT 
            42 as answer ------ this has many dashes ------
        """
        result = run_query(query)
        assert result[0]["answer"] == 42

class TestBlockComments:
    """Test block comments with /* */"""
    
    def test_simple_block_comment(self):
        """Test basic block comment"""
        result = run_query("SELECT 3 /* this is a block comment */ * 4 as result")
        assert result[0]["result"] == 12
    
    def test_multiline_block_comment(self):
        """Test multi-line block comment"""
        query = """
        SELECT 
            /* This is a
               multi-line
               block comment */
            7 + 8 as sum
        """
        result = run_query(query)
        assert result[0]["sum"] == 15
    
    def test_nested_operators_in_comments(self):
        """Test that operators inside comments are ignored"""
        query = """
        SELECT 
            10 /* + * / - */ + 5 as result
        """
        result = run_query(query)
        assert result[0]["result"] == 15
    
    def test_multiple_block_comments(self):
        """Test multiple block comments in one query"""
        query = """
        SELECT 
            2 /* first */ + /* second */ 3 /* third */ as sum
        """
        result = run_query(query)
        assert result[0]["sum"] == 5
    
    def test_block_comment_at_end(self):
        """Test block comment at the end of query"""
        query = """
        SELECT 99 as num
        /* This is a comment at the end
           It spans multiple lines
           and should be ignored */
        """
        result = run_query(query)
        assert result[0]["num"] == 99

class TestMixedComments:
    """Test mixing -- and /* */ comments"""
    
    def test_both_comment_styles(self):
        """Test using both comment styles in one query"""
        query = """
        -- Single line comment
        SELECT 
            1 + 1 as a, /* block comment */ -- and inline
            2 * 2 as b
        /* Final block
           comment */
        """
        result = run_query(query)
        assert result[0]["a"] == 2
        assert result[0]["b"] == 4
    
    def test_comment_in_strings(self):
        """Test that comment markers in strings are preserved"""
        query = """
        SELECT 
            'test--value' as str1, -- real comment
            'test /* block */ value' as str2 /* real block */
        """
        result = run_query(query)
        assert result[0]["str1"] == "test--value"
        assert result[0]["str2"] == "test /* block */ value"

class TestCommentsWithDUAL:
    """Test comments with DUAL table calculations"""
    
    def test_commented_physics_calc(self):
        """Test physics calculations with extensive comments"""
        query = """
        -- Calculate photon energy
        SELECT 
            -- Planck's constant * speed of light / wavelength
            PLANCK() * C() / 700e-9 as photon_energy, -- Red light at 700nm
            
            /* Calculate Schwarzschild radius for Earth
               r_s = 2GM/c^2 where M = 5.972e24 kg */
            2 * G() * 5.972e24 / (C() * C()) as earth_radius
        FROM DUAL
        """
        result = run_query(query)
        assert abs(result[0]["photon_energy"] - 2.84e-19) < 1e-20
        assert abs(result[0]["earth_radius"] - 0.00887) < 0.0001
    
    def test_unit_conversion_with_comments(self):
        """Test unit conversions with comments"""
        query = """
        SELECT 
            -- Convert 100 km to miles
            CONVERT(100, 'km', 'miles') as distance,
            
            /* Temperature conversion
               Fahrenheit to Celsius */
            CONVERT(32, 'F', 'C') as freezing_point
        FROM DUAL -- Using DUAL for calculations
        """
        result = run_query(query)
        assert abs(result[0]["distance"] - 62.137) < 0.01
        assert abs(result[0]["freezing_point"] - 0.0) < 0.01

class TestCommentEdgeCases:
    """Test edge cases and potential parsing issues"""
    
    def test_comment_after_operators(self):
        """Test comments immediately after operators"""
        query = """
        SELECT 
            10 - -- comment after minus
            5 as result
        """
        result = run_query(query)
        assert result[0]["result"] == 5
    
    def test_division_not_comment(self):
        """Test that division operator is not treated as comment start"""
        query = "SELECT 10 / 2 as result"
        result = run_query(query)
        assert result[0]["result"] == 5
    
    def test_multiplication_not_comment(self):
        """Test that multiplication is not treated as comment end"""
        query = "SELECT 3 * 4 as result"
        result = run_query(query)
        assert result[0]["result"] == 12
    
    def test_negative_number_not_comment(self):
        """Test that negative numbers aren't treated as comments"""
        query = "SELECT -5 + 10 as result"
        result = run_query(query)
        assert result[0]["result"] == 5
    
    def test_complex_expression_with_comments(self):
        """Test complex expression with various comment placements"""
        query = """
        SELECT 
            ( -- comment
                5 /* block */ + /* another */ 3 -- inline
            ) * -- multiply
            2 /* by two */ as result -- final comment
        """
        result = run_query(query)
        assert result[0]["result"] == 16

class TestQueryFileComments:
    """Test running SQL files with comments"""
    
    def test_file_with_header_comments(self):
        """Test SQL file with header documentation"""
        sql_content = """
        -- ============================================
        -- SQL CLI Test Query
        -- Purpose: Test comment parsing
        -- Author: Test Suite
        -- Date: 2024
        -- ============================================
        
        SELECT 
            -- Basic calculations
            PI() as pi_value,
            
            /* Constants test */
            C() as speed_of_light
        FROM DUAL
        
        -- End of query
        """
        result = run_query_file(sql_content)
        assert abs(result[0]["pi_value"] - 3.14159) < 0.001
        assert result[0]["speed_of_light"] == 299792458.0
    
    def test_documented_calculation(self):
        """Test well-documented scientific calculation"""
        sql_content = """
        /*
         * Bohr Radius Calculation
         * =======================
         * Calculate the Bohr radius of hydrogen atom
         * Formula: a₀ = 4πε₀ℏ²/(mₑe²)
         *
         * Where:
         *   ε₀ = electric permittivity of vacuum
         *   ℏ  = reduced Planck constant
         *   mₑ = electron mass
         *   e  = elementary charge
         */
        
        SELECT 
            -- Step 1: Calculate numerator
            4 * PI() * E0() * HBAR() * HBAR() as numerator,
            
            -- Step 2: Calculate denominator  
            ME() * QE() * QE() as denominator,
            
            -- Step 3: Final result
            4 * PI() * E0() * HBAR() * HBAR() / (ME() * QE() * QE()) as bohr_radius
        FROM DUAL
        
        /* Expected result: ~5.29e-11 meters */
        """
        result = run_query_file(sql_content)
        assert abs(result[0]["bohr_radius"] - 5.29e-11) < 1e-12

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])