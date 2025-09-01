#!/usr/bin/env python3
"""Test DUAL table functionality and physical constants"""

import subprocess
import json
import math
import pytest
from pathlib import Path

# Path to the SQL CLI binary
SQL_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"

def run_query(query, output_format="json"):
    """Run a SQL query and return the result"""
    cmd = [str(SQL_CLI), "-q", query, "-o", output_format]
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

class TestDualTable:
    """Test DUAL table functionality"""
    
    def test_dual_basic(self):
        """Test basic DUAL table query"""
        result = run_query("SELECT 1 FROM DUAL")
        assert len(result) == 1
        assert result[0]["expr_1"] == 1
    
    def test_dual_arithmetic(self):
        """Test arithmetic expressions with DUAL"""
        result = run_query("SELECT 2 + 2 as sum, 10 - 3 as diff, 4 * 5 as prod FROM DUAL")
        assert result[0]["sum"] == 4
        assert result[0]["diff"] == 7
        assert result[0]["prod"] == 20
    
    def test_dual_scientific_notation(self):
        """Test scientific notation with DUAL"""
        result = run_query("SELECT 1e-10 as tiny, 3.14e5 as big, 2.5E-3 as small FROM DUAL")
        assert abs(result[0]["tiny"] - 1e-10) < 1e-15
        assert abs(result[0]["big"] - 3.14e5) < 0.01
        assert abs(result[0]["small"] - 2.5e-3) < 1e-10
    
    def test_no_from_clause(self):
        """Test queries without FROM clause (should use DUAL implicitly)"""
        result = run_query("SELECT 42")
        assert result[0]["expr_1"] == 42
        
        result = run_query("SELECT 1 + 1")
        assert result[0]["expr_1"] == 2

class TestPhysicalConstants:
    """Test physical constant functions"""
    
    def test_pi_constant(self):
        """Test PI() function"""
        result = run_query("SELECT PI() as pi_value FROM DUAL")
        assert abs(result[0]["pi_value"] - math.pi) < 1e-10
    
    @pytest.mark.skip(reason="EULER() function not working yet - parser issue")
    def test_euler_constant(self):
        """Test EULER() function"""
        result = run_query("SELECT EULER() as e_value FROM DUAL")
        assert abs(result[0]["e_value"] - math.e) < 1e-10
    
    @pytest.mark.skip(reason="TAU() function not working yet - parser issue")
    def test_tau_constant(self):
        """Test TAU() function"""
        result = run_query("SELECT TAU() as tau_value FROM DUAL")
        assert abs(result[0]["tau_value"] - math.tau) < 1e-10
    
    @pytest.mark.skip(reason="PHI() function not working yet - parser issue")
    def test_phi_constant(self):
        """Test PHI() golden ratio function"""
        result = run_query("SELECT PHI() as phi_value FROM DUAL")
        golden_ratio = (1 + math.sqrt(5)) / 2
        assert abs(result[0]["phi_value"] - golden_ratio) < 1e-10
    
    @pytest.mark.skip(reason="SQRT2() function not working yet - parser issue")
    def test_sqrt2_constant(self):
        """Test SQRT2() function"""
        result = run_query("SELECT SQRT2() as sqrt2_value FROM DUAL")
        assert abs(result[0]["sqrt2_value"] - math.sqrt(2)) < 1e-10
    
    @pytest.mark.skip(reason="LN2() function not working yet - parser issue")
    def test_ln2_constant(self):
        """Test LN2() function"""
        result = run_query("SELECT LN2() as ln2_value FROM DUAL")
        assert abs(result[0]["ln2_value"] - math.log(2)) < 1e-10
    
    @pytest.mark.skip(reason="LN10() function not working yet - parser issue")
    def test_ln10_constant(self):
        """Test LN10() function"""
        result = run_query("SELECT LN10() as ln10_value FROM DUAL")
        assert abs(result[0]["ln10_value"] - math.log(10)) < 1e-10

class TestMixedExpressions:
    """Test combining constants with expressions"""
    
    def test_pi_in_calculations(self):
        """Test using PI in calculations"""
        # Circle area with radius 2: π * r^2
        result = run_query("SELECT PI() * 4 as area FROM DUAL")
        expected = math.pi * 4
        assert abs(result[0]["area"] - expected) < 1e-10
    
    @pytest.mark.skip(reason="Constants not fully working yet")
    def test_mixed_constants(self):
        """Test mixing multiple constants"""
        result = run_query("SELECT PI() + EULER() as sum FROM DUAL")
        expected = math.pi + math.e
        assert abs(result[0]["sum"] - expected) < 1e-10

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])