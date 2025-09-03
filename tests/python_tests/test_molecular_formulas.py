#!/usr/bin/env python3
"""
Test suite for molecular formula support in ATOMIC_MASS function
"""

import subprocess
import json
import csv
import os
import tempfile
import pytest
from pathlib import Path

# Find the sql-cli binary
PROJECT_ROOT = Path(__file__).parent.parent.parent
SQL_CLI = PROJECT_ROOT / "target" / "release" / "sql-cli"

if not SQL_CLI.exists():
    SQL_CLI = PROJECT_ROOT / "target" / "debug" / "sql-cli"


def run_query(csv_file, query):
    """Run a SQL query and return results as a list of dictionaries."""
    cmd = [str(SQL_CLI), csv_file, "-q", query, "-o", "json"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse JSON output
    lines = result.stdout.strip().split('\n')
    # Filter out comment lines
    json_lines = [l for l in lines if not l.startswith('#')]
    if not json_lines:
        return []
    
    return json.loads(''.join(json_lines))


class TestMolecularFormulas:
    """Test molecular formula parsing in ATOMIC_MASS function."""
    
    @classmethod
    def setup_class(cls):
        """Create test data file."""
        cls.temp_dir = tempfile.mkdtemp()
        cls.test_file = os.path.join(cls.temp_dir, "test.csv")
        with open(cls.test_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id'])
            writer.writerow([1])
    
    def test_simple_molecules(self):
        """Test simple molecular formulas."""
        result = run_query(self.test_file, 
                          "SELECT ATOMIC_MASS('H2O') as water, "
                          "ATOMIC_MASS('CO2') as carbon_dioxide, "
                          "ATOMIC_MASS('NH3') as ammonia FROM test")
        
        assert len(result) == 1
        # Water: 2*1.008 + 16.00 = 18.016
        assert abs(result[0]['water'] - 18.016) < 0.01
        # CO2: 12.01 + 2*16.00 = 44.01
        assert abs(result[0]['carbon_dioxide'] - 44.01) < 0.01
        # NH3: 14.01 + 3*1.008 = 17.034
        assert abs(result[0]['ammonia'] - 17.034) < 0.01
    
    def test_complex_molecules(self):
        """Test complex molecular formulas with parentheses."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('Ca(OH)2') as calcium_hydroxide, "
                          "ATOMIC_MASS('Mg(NO3)2') as magnesium_nitrate FROM test")
        
        assert len(result) == 1
        # Ca(OH)2: 40.08 + 2*(16.00 + 1.008) = 74.096
        assert abs(result[0]['calcium_hydroxide'] - 74.096) < 0.01
        # Mg(NO3)2: 24.31 + 2*(14.01 + 3*16.00) = 148.33
        assert abs(result[0]['magnesium_nitrate'] - 148.33) < 0.01
    
    def test_organic_molecules(self):
        """Test organic molecular formulas."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('C6H12O6') as glucose, "
                          "ATOMIC_MASS('C2H5OH') as ethanol, "
                          "ATOMIC_MASS('CH3COOH') as acetic_acid FROM test")
        
        assert len(result) == 1
        # C6H12O6: 6*12.01 + 12*1.008 + 6*16.00 = 180.156
        assert abs(result[0]['glucose'] - 180.156) < 0.01
        # C2H5OH: 2*12.01 + 6*1.008 + 16.00 = 46.068
        assert abs(result[0]['ethanol'] - 46.068) < 0.01
        # CH3COOH: 2*12.01 + 4*1.008 + 2*16.00 = 60.052
        assert abs(result[0]['acetic_acid'] - 60.052) < 0.01
    
    def test_compound_aliases(self):
        """Test common compound name aliases."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('water') as water, "
                          "ATOMIC_MASS('WATER') as water_upper, "
                          "ATOMIC_MASS('glucose') as glucose, "
                          "ATOMIC_MASS('salt') as salt, "
                          "ATOMIC_MASS('ammonia') as ammonia FROM test")
        
        assert len(result) == 1
        # All water variants should give H2O mass
        assert abs(result[0]['water'] - 18.016) < 0.01
        assert abs(result[0]['water_upper'] - 18.016) < 0.01
        # Glucose alias
        assert abs(result[0]['glucose'] - 180.156) < 0.01
        # Salt (NaCl)
        assert abs(result[0]['salt'] - 58.44) < 0.01
        # Ammonia
        assert abs(result[0]['ammonia'] - 17.034) < 0.01
    
    def test_acid_aliases(self):
        """Test acid name aliases."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('sulfuric acid') as h2so4, "
                          "ATOMIC_MASS('hydrochloric acid') as hcl, "
                          "ATOMIC_MASS('nitric acid') as hno3 FROM test")
        
        assert len(result) == 1
        # H2SO4: 2*1.008 + 32.07 + 4*16.00 = 98.086
        assert abs(result[0]['h2so4'] - 98.086) < 0.01
        # HCl: 1.008 + 35.45 = 36.458
        assert abs(result[0]['hcl'] - 36.458) < 0.01
        # HNO3: 1.008 + 14.01 + 3*16.00 = 63.018
        assert abs(result[0]['hno3'] - 63.018) < 0.01
    
    def test_single_elements_still_work(self):
        """Test that single element symbols still work."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('H') as hydrogen, "
                          "ATOMIC_MASS('C') as carbon, "
                          "ATOMIC_MASS('O') as oxygen, "
                          "ATOMIC_MASS('Gold') as gold FROM test")
        
        assert len(result) == 1
        assert abs(result[0]['hydrogen'] - 1.008) < 0.01
        assert abs(result[0]['carbon'] - 12.01) < 0.01
        assert abs(result[0]['oxygen'] - 16.00) < 0.01
        assert abs(result[0]['gold'] - 196.97) < 0.01
    
    def test_hydrates(self):
        """Test hydrated compounds (future enhancement)."""
        # This test documents desired future behavior for hydrates
        # Currently this would fail as we don't support dot notation yet
        pass
        # result = run_query(self.test_file,
        #                   "SELECT ATOMIC_MASS('CuSO4.5H2O') as copper_sulfate_pentahydrate")
        # CuSO4.5H2O: 63.55 + 32.07 + 4*16.00 + 5*(2*1.008 + 16.00) = 249.69


class TestMolecularFormulaEdgeCases:
    """Test edge cases and error handling."""
    
    @classmethod
    def setup_class(cls):
        """Create test data file."""
        cls.temp_dir = tempfile.mkdtemp()
        cls.test_file = os.path.join(cls.temp_dir, "test.csv")
        with open(cls.test_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id'])
            writer.writerow([1])
    
    def test_no_number_means_one(self):
        """Test that missing numbers default to 1."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('HCl') as hcl, "
                          "ATOMIC_MASS('NaCl') as nacl FROM test")
        
        assert len(result) == 1
        # HCl (not H1Cl1)
        assert abs(result[0]['hcl'] - 36.458) < 0.01
        # NaCl
        assert abs(result[0]['nacl'] - 58.44) < 0.01
    
    def test_multi_digit_numbers(self):
        """Test formulas with numbers > 9."""
        result = run_query(self.test_file,
                          "SELECT ATOMIC_MASS('C12H22O11') as sucrose FROM test")
        
        assert len(result) == 1
        # Sucrose: 12*12.01 + 22*1.008 + 11*16.00 = 342.296
        assert abs(result[0]['sucrose'] - 342.296) < 0.1
    
    def test_nested_parentheses(self):
        """Test formulas with nested parentheses (if supported)."""
        # This documents potential future enhancement
        pass
        # result = run_query(self.test_file,
        #                   "SELECT ATOMIC_MASS('Ca3(Fe(CN)6)2') as complex")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])