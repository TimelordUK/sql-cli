#!/usr/bin/env python3
"""Test physics constants with scientific notation support"""

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

class TestFundamentalConstants:
    """Test fundamental physics constants"""
    
    def test_speed_of_light(self):
        """Test speed of light constant"""
        result = run_query("SELECT C() as c FROM DUAL")
        assert result[0]["c"] == 299792458.0
        
        # Also test long form
        result = run_query("SELECT SPEED_OF_LIGHT() as c FROM DUAL")
        assert result[0]["c"] == 299792458.0
    
    def test_gravitational_constant(self):
        """Test gravitational constant with scientific notation"""
        result = run_query("SELECT G() as g FROM DUAL")
        assert abs(result[0]["g"] - 6.67430e-11) < 1e-20
        
        # Test that we can use it in calculations
        result = run_query("SELECT G() * 1e11 as g_scaled FROM DUAL")
        assert abs(result[0]["g_scaled"] - 6.67430) < 1e-5
    
    def test_planck_constant(self):
        """Test Planck constant - very small number"""
        result = run_query("SELECT H() as h FROM DUAL")
        assert abs(result[0]["h"] - 6.62607015e-34) < 1e-40
        
        # Also test PLANCK() alias
        result = run_query("SELECT PLANCK() as h FROM DUAL")
        assert abs(result[0]["h"] - 6.62607015e-34) < 1e-40
    
    def test_reduced_planck_constant(self):
        """Test reduced Planck constant (hbar)"""
        result = run_query("SELECT HBAR() as hbar FROM DUAL")
        assert abs(result[0]["hbar"] - 1.054571817e-34) < 1e-40
        
        # Verify it's approximately h/(2*pi)
        result = run_query("SELECT PLANCK() / (2 * PI()) as calc_hbar FROM DUAL")
        assert abs(result[0]["calc_hbar"] - 1.054571817e-34) < 1e-36
    
    def test_boltzmann_constant(self):
        """Test Boltzmann constant"""
        result = run_query("SELECT K() as k FROM DUAL")
        assert abs(result[0]["k"] - 1.380649e-23) < 1e-30
        
        # Also test BOLTZMANN() alias
        result = run_query("SELECT BOLTZMANN() as k FROM DUAL")
        assert abs(result[0]["k"] - 1.380649e-23) < 1e-30
    
    def test_avogadro_number(self):
        """Test Avogadro's number - very large"""
        result = run_query("SELECT NA() as na FROM DUAL")
        assert abs(result[0]["na"] - 6.02214076e23) < 1e15
        
        # Also test AVOGADRO() alias
        result = run_query("SELECT AVOGADRO() as na FROM DUAL")
        assert abs(result[0]["na"] - 6.02214076e23) < 1e15
    
    def test_gas_constant(self):
        """Test universal gas constant"""
        result = run_query("SELECT R() as r FROM DUAL")
        assert abs(result[0]["r"] - 8.314462618) < 1e-8
        
        # Verify R = NA * k (within precision)
        result = run_query("SELECT AVOGADRO() * BOLTZMANN() as calc_r FROM DUAL")
        assert abs(result[0]["calc_r"] - 8.314462618) < 0.01

class TestElectromagneticConstants:
    """Test electromagnetic constants"""
    
    def test_permittivity(self):
        """Test electric permittivity of vacuum"""
        result = run_query("SELECT E0() as e0 FROM DUAL")
        assert abs(result[0]["e0"] - 8.8541878128e-12) < 1e-20
        
        # Test EPSILON0 alias
        result = run_query("SELECT EPSILON0() as e0 FROM DUAL")
        assert abs(result[0]["e0"] - 8.8541878128e-12) < 1e-20
    
    def test_permeability(self):
        """Test magnetic permeability of vacuum"""
        result = run_query("SELECT MU0() as mu0 FROM DUAL")
        assert abs(result[0]["mu0"] - 1.25663706212e-6) < 1e-15
    
    def test_elementary_charge(self):
        """Test elementary charge"""
        result = run_query("SELECT QE() as qe FROM DUAL")
        assert abs(result[0]["qe"] - 1.602176634e-19) < 1e-28
        
        # Test ELEMENTARY_CHARGE alias
        result = run_query("SELECT ELEMENTARY_CHARGE() as qe FROM DUAL")
        assert abs(result[0]["qe"] - 1.602176634e-19) < 1e-28
    
    def test_speed_of_light_relation(self):
        """Test c^2 = 1/(ε₀μ₀)"""
        result = run_query("SELECT C() * C() as c_squared, 1.0 / (E0() * MU0()) as calc FROM DUAL")
        # These should be equal within floating point precision
        assert abs(result[0]["c_squared"] - result[0]["calc"]) / result[0]["c_squared"] < 1e-10

class TestParticleMasses:
    """Test particle mass constants"""
    
    def test_electron_mass(self):
        """Test electron mass - extremely small"""
        result = run_query("SELECT ME() as me FROM DUAL")
        assert abs(result[0]["me"] - 9.1093837015e-31) < 1e-40
        
        # Test MASS_ELECTRON alias
        result = run_query("SELECT MASS_ELECTRON() as me FROM DUAL")
        assert abs(result[0]["me"] - 9.1093837015e-31) < 1e-40
    
    def test_proton_mass(self):
        """Test proton mass"""
        result = run_query("SELECT MP() as mp FROM DUAL")
        assert abs(result[0]["mp"] - 1.67262192369e-27) < 1e-36
        
        # Test MASS_PROTON alias
        result = run_query("SELECT MASS_PROTON() as mp FROM DUAL")
        assert abs(result[0]["mp"] - 1.67262192369e-27) < 1e-36
    
    def test_neutron_mass(self):
        """Test neutron mass"""
        result = run_query("SELECT MN() as mn FROM DUAL")
        assert abs(result[0]["mn"] - 1.67492749804e-27) < 1e-36
        
        # Test MASS_NEUTRON alias
        result = run_query("SELECT MASS_NEUTRON() as mn FROM DUAL")
        assert abs(result[0]["mn"] - 1.67492749804e-27) < 1e-36
    
    def test_atomic_mass_unit(self):
        """Test atomic mass unit"""
        result = run_query("SELECT AMU() as amu FROM DUAL")
        assert abs(result[0]["amu"] - 1.66053906660e-27) < 1e-36
        
        # Test ATOMIC_MASS_UNIT alias
        result = run_query("SELECT ATOMIC_MASS_UNIT() as amu FROM DUAL")
        assert abs(result[0]["amu"] - 1.66053906660e-27) < 1e-36
    
    def test_mass_ratios(self):
        """Test proton/electron mass ratio"""
        result = run_query("SELECT MP() / ME() as ratio FROM DUAL")
        # Proton is about 1836 times heavier than electron
        assert abs(result[0]["ratio"] - 1836.15) < 1

class TestOtherConstants:
    """Test other physics constants"""
    
    def test_fine_structure_constant(self):
        """Test fine structure constant (dimensionless)"""
        result = run_query("SELECT ALPHA() as alpha FROM DUAL")
        assert abs(result[0]["alpha"] - 7.2973525693e-3) < 1e-12
        
        # Test FINE_STRUCTURE alias
        result = run_query("SELECT FINE_STRUCTURE() as alpha FROM DUAL")
        assert abs(result[0]["alpha"] - 7.2973525693e-3) < 1e-12
        
        # Verify it's approximately 1/137
        result = run_query("SELECT 1.0 / ALPHA() as inv_alpha FROM DUAL")
        assert abs(result[0]["inv_alpha"] - 137.036) < 0.01
    
    def test_rydberg_constant(self):
        """Test Rydberg constant"""
        result = run_query("SELECT RY() as ry FROM DUAL")
        assert abs(result[0]["ry"] - 10973731.568160) < 0.01
        
        # Test RYDBERG alias
        result = run_query("SELECT RYDBERG() as ry FROM DUAL")
        assert abs(result[0]["ry"] - 10973731.568160) < 0.01
    
    def test_stefan_boltzmann_constant(self):
        """Test Stefan-Boltzmann constant"""
        result = run_query("SELECT SIGMA() as sigma FROM DUAL")
        assert abs(result[0]["sigma"] - 5.670374419e-8) < 1e-16
        
        # Test STEFAN_BOLTZMANN alias
        result = run_query("SELECT STEFAN_BOLTZMANN() as sigma FROM DUAL")
        assert abs(result[0]["sigma"] - 5.670374419e-8) < 1e-16

class TestScientificCalculations:
    """Test physics calculations using constants"""
    
    def test_photon_energy(self):
        """Test E = hf calculation for visible light"""
        # Red light at 700nm wavelength
        result = run_query("""
            SELECT PLANCK() * C() / 700e-9 as energy_joules
            FROM DUAL
        """)
        # Should be around 2.84e-19 Joules
        assert abs(result[0]["energy_joules"] - 2.84e-19) < 1e-20
    
    def test_schwarzschild_radius(self):
        """Test Schwarzschild radius calculation for Earth"""
        # r_s = 2GM/c^2, Earth mass = 5.972e24 kg
        result = run_query("""
            SELECT 2 * G() * 5.972e24 / (C() * C()) as rs_earth
            FROM DUAL
        """)
        # Earth's Schwarzschild radius is about 8.87mm
        assert abs(result[0]["rs_earth"] - 0.00887) < 0.0001
    
    def test_thermal_energy(self):
        """Test thermal energy kT at room temperature"""
        # Room temperature = 293K
        result = run_query("""
            SELECT BOLTZMANN() * 293 as kt_room
            FROM DUAL
        """)
        # Should be around 4.05e-21 Joules
        assert abs(result[0]["kt_room"] - 4.05e-21) < 1e-22
    
    def test_compton_wavelength(self):
        """Test electron Compton wavelength"""
        # λ_c = h/(m_e * c)
        result = run_query("""
            SELECT PLANCK() / (ME() * C()) as compton_wavelength
            FROM DUAL
        """)
        # Should be 2.426e-12 meters
        assert abs(result[0]["compton_wavelength"] - 2.426e-12) < 1e-14
    
    def test_bohr_radius(self):
        """Test Bohr radius calculation"""
        # a_0 = 4πε₀ℏ²/(m_e * e²)
        result = run_query("""
            SELECT 4 * PI() * E0() * HBAR() * HBAR() / (ME() * QE() * QE()) as bohr_radius
            FROM DUAL
        """)
        # Should be 5.29e-11 meters
        assert abs(result[0]["bohr_radius"] - 5.29e-11) < 1e-12
    
    def test_electron_volt_conversion(self):
        """Test eV to Joules conversion"""
        # 1 eV = elementary charge * 1 Volt
        result = run_query("""
            SELECT QE() as eV_to_joules,
                   QE() * 1e6 as MeV_to_joules,
                   QE() * 1e9 as GeV_to_joules
            FROM DUAL
        """)
        assert abs(result[0]["eV_to_joules"] - 1.602e-19) < 1e-22
        assert abs(result[0]["MeV_to_joules"] - 1.602e-13) < 1e-16
        assert abs(result[0]["GeV_to_joules"] - 1.602e-10) < 1e-13

class TestExtremePrecision:
    """Test handling of extreme values with scientific notation"""
    
    def test_very_small_numbers(self):
        """Test calculations with very small numbers"""
        result = run_query("""
            SELECT PLANCK() / 1e20 as tiny1,
                   ME() * 1e-10 as tiny2,
                   G() * PLANCK() as tiny3
            FROM DUAL
        """)
        assert result[0]["tiny1"] < 1e-50
        assert result[0]["tiny2"] < 1e-40
        assert result[0]["tiny3"] < 1e-40
    
    def test_very_large_numbers(self):
        """Test calculations with very large numbers"""
        result = run_query("""
            SELECT AVOGADRO() * 1e10 as huge1,
                   C() * C() as huge2,
                   RYDBERG() * AVOGADRO() as huge3
            FROM DUAL
        """)
        assert result[0]["huge1"] > 1e33
        assert result[0]["huge2"] > 1e16
        assert result[0]["huge3"] > 1e30
    
    def test_precision_preservation(self):
        """Test that precision is preserved in calculations"""
        # Test that dividing and multiplying preserves the original value
        result = run_query("""
            SELECT ME() as original,
                   (ME() * 1e50) / 1e50 as recovered
            FROM DUAL
        """)
        relative_error = abs(result[0]["original"] - result[0]["recovered"]) / result[0]["original"]
        assert relative_error < 1e-10

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])