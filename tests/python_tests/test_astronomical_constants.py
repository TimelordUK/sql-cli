#!/usr/bin/env python3
"""Test astronomical constants and calculations"""

import subprocess
import json
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

class TestParticleRadii:
    """Test particle radius constants"""
    
    def test_electron_radius(self):
        """Test classical electron radius"""
        result = run_query("SELECT RE() as radius FROM DUAL")
        assert abs(result[0]["radius"] - 2.8179403262e-15) < 1e-24
    
    def test_proton_radius(self):
        """Test proton radius"""
        result = run_query("SELECT RP() as radius FROM DUAL")
        assert abs(result[0]["radius"] - 8.414e-16) < 1e-25
    
    def test_neutron_radius(self):
        """Test neutron radius"""
        result = run_query("SELECT RN() as radius FROM DUAL")
        assert abs(result[0]["radius"] - 8.4e-16) < 1e-25
    
    def test_particle_size_comparison(self):
        """Test relative sizes of particles"""
        query = """
        SELECT 
            RE() / RP() as electron_proton_ratio,
            RP() / RN() as proton_neutron_ratio
        FROM DUAL
        """
        result = run_query(query)
        # Electron radius is ~3.35x proton radius
        assert abs(result[0]["electron_proton_ratio"] - 3.349) < 0.01
        # Proton and neutron radii are nearly identical
        assert abs(result[0]["proton_neutron_ratio"] - 1.002) < 0.01

class TestSolarSystemMasses:
    """Test solar system mass constants"""
    
    def test_sun_mass(self):
        """Test solar mass"""
        result = run_query("SELECT MASS_SUN() as mass FROM DUAL")
        # Solar mass with tolerance for precision differences
        assert abs(result[0]["mass"] - 1.98892e30) < 1e27
    
    def test_earth_mass(self):
        """Test Earth mass"""
        result = run_query("SELECT MASS_EARTH() as mass FROM DUAL")
        assert result[0]["mass"] == 5.97237e24
    
    def test_moon_mass(self):
        """Test Moon mass"""
        result = run_query("SELECT MASS_MOON() as mass FROM DUAL")
        assert result[0]["mass"] == 7.342e22
    
    def test_planet_masses(self):
        """Test all planet masses"""
        query = """
        SELECT 
            MASS_MERCURY() as mercury,
            MASS_VENUS() as venus,
            MASS_EARTH() as earth,
            MASS_MARS() as mars,
            MASS_JUPITER() as jupiter,
            MASS_SATURN() as saturn,
            MASS_URANUS() as uranus,
            MASS_NEPTUNE() as neptune
        FROM DUAL
        """
        result = run_query(query)
        assert result[0]["mercury"] == 3.3011e23
        assert result[0]["venus"] == 4.8675e24
        assert result[0]["earth"] == 5.97237e24
        assert result[0]["mars"] == 6.4171e23
        assert result[0]["jupiter"] == 1.8982e27
        assert result[0]["saturn"] == 5.6834e26
        assert result[0]["uranus"] == 8.6810e25
        assert result[0]["neptune"] == 1.02413e26
    
    def test_mass_ratios(self):
        """Test mass ratios in solar system"""
        query = """
        SELECT 
            MASS_SUN() / MASS_EARTH() as sun_earth_ratio,
            MASS_EARTH() / MASS_MOON() as earth_moon_ratio,
            MASS_JUPITER() / MASS_EARTH() as jupiter_earth_ratio
        FROM DUAL
        """
        result = run_query(query)
        # Sun is ~333,000 times Earth's mass (updated for more precise constants)
        assert abs(result[0]["sun_earth_ratio"] - 333020) < 100
        # Earth is ~81 times Moon's mass
        assert abs(result[0]["earth_moon_ratio"] - 81.35) < 0.1
        # Jupiter is ~318 times Earth's mass
        assert abs(result[0]["jupiter_earth_ratio"] - 317.8) < 0.1

class TestOrbitalDistances:
    """Test orbital distance constants"""
    
    def test_astronomical_unit(self):
        """Test AU constant"""
        result = run_query("SELECT AU() as au_meters FROM DUAL")
        assert result[0]["au_meters"] == 1.495978707e11
    
    def test_planetary_distances(self):
        """Test distances from sun"""
        query = """
        SELECT 
            DIST_MERCURY() as mercury,
            DIST_VENUS() as venus,
            AU() as earth,
            DIST_MARS() as mars,
            DIST_JUPITER() as jupiter,
            DIST_SATURN() as saturn,
            DIST_URANUS() as uranus,
            DIST_NEPTUNE() as neptune
        FROM DUAL
        """
        result = run_query(query)
        # All distances in meters (using tolerance for precision differences)
        assert abs(result[0]["mercury"] - 5.791e10) < 1e9
        assert abs(result[0]["venus"] - 1.082e11) < 1e9
        # Earth distance (AU value is more precise)
        assert abs(result[0]["earth"] - 149597870700.0) < 1e8
        assert abs(result[0]["mars"] - 2.279e11) < 1e9
        assert abs(result[0]["jupiter"] - 7.786e11) < 1e10
        assert abs(result[0]["saturn"] - 1.4335e12) < 1e10
        assert abs(result[0]["uranus"] - 2.8725e12) < 1e10
        assert abs(result[0]["neptune"] - 4.4951e12) < 1e10
    
    def test_distances_in_au(self):
        """Test planetary distances in AU"""
        query = """
        SELECT 
            DIST_MERCURY() / AU() as mercury_au,
            DIST_VENUS() / AU() as venus_au,
            AU() / AU() as earth_au,
            DIST_MARS() / AU() as mars_au,
            DIST_JUPITER() / AU() as jupiter_au
        FROM DUAL
        """
        result = run_query(query)
        # Verify known AU distances
        assert abs(result[0]["mercury_au"] - 0.387) < 0.001
        assert abs(result[0]["venus_au"] - 0.723) < 0.001
        assert abs(result[0]["earth_au"] - 1.0) < 0.001
        assert abs(result[0]["mars_au"] - 1.523) < 0.001
        assert abs(result[0]["jupiter_au"] - 5.204) < 0.01
    
    def test_other_distance_units(self):
        """Test parsec and light-year"""
        query = """
        SELECT 
            PARSEC() as parsec_meters,
            LIGHT_YEAR() as ly_meters,
            PARSEC() / LIGHT_YEAR() as parsec_ly_ratio
        FROM DUAL
        """
        result = run_query(query)
        assert result[0]["parsec_meters"] == 3.0857e16
        assert result[0]["ly_meters"] == 9.4607e15
        # 1 parsec = ~3.26 light-years
        assert abs(result[0]["parsec_ly_ratio"] - 3.262) < 0.001

class TestAstronomicalCalculations:
    """Test complex astronomical calculations"""
    
    def test_kepler_third_law(self):
        """Test Kepler's third law: T² ∝ a³"""
        query = """
        SELECT 
            -- For Earth: orbital period should be ~1 year
            SQRT(4 * PI() * PI() * POWER(AU(), 3) / (G() * MASS_SUN())) / (365.25 * 24 * 3600) as earth_period_years,
            -- For Mars
            SQRT(4 * PI() * PI() * POWER(DIST_MARS(), 3) / (G() * MASS_SUN())) / (365.25 * 24 * 3600) as mars_period_years
        FROM DUAL
        """
        result = run_query(query)
        # Earth's orbital period should be ~1 year
        assert abs(result[0]["earth_period_years"] - 1.0) < 0.01
        # Mars' orbital period should be ~1.88 years
        assert abs(result[0]["mars_period_years"] - 1.88) < 0.01
    
    def test_escape_velocity(self):
        """Test escape velocity calculations"""
        query = """
        SELECT 
            -- Escape velocity from Earth: v = sqrt(2GM/r)
            SQRT(2 * G() * MASS_EARTH() / 6.371e6) as earth_escape_ms,
            -- Escape velocity from Moon
            SQRT(2 * G() * MASS_MOON() / 1.737e6) as moon_escape_ms,
            -- Escape velocity from Jupiter
            SQRT(2 * G() * MASS_JUPITER() / 7.1492e7) as jupiter_escape_ms
        FROM DUAL
        """
        result = run_query(query)
        # Earth escape velocity ~11.2 km/s
        assert abs(result[0]["earth_escape_ms"] - 11200) < 100
        # Moon escape velocity ~2.4 km/s
        assert abs(result[0]["moon_escape_ms"] - 2380) < 50
        # Jupiter escape velocity ~59.5 km/s
        assert abs(result[0]["jupiter_escape_ms"] - 59500) < 500
    
    def test_schwarzschild_radius(self):
        """Test Schwarzschild radius for celestial bodies"""
        query = """
        SELECT 
            -- Schwarzschild radius: r_s = 2GM/c²
            2 * G() * MASS_SUN() / (C() * C()) as sun_rs_meters,
            2 * G() * MASS_EARTH() / (C() * C()) as earth_rs_meters,
            2 * G() * MASS_JUPITER() / (C() * C()) as jupiter_rs_meters
        FROM DUAL
        """
        result = run_query(query)
        # Sun's Schwarzschild radius ~2.95 km
        assert abs(result[0]["sun_rs_meters"] - 2954) < 10
        # Earth's Schwarzschild radius ~8.87 mm
        assert abs(result[0]["earth_rs_meters"] - 0.00887) < 0.0001
        # Jupiter's Schwarzschild radius ~2.82 m
        assert abs(result[0]["jupiter_rs_meters"] - 2.82) < 0.01
    
    def test_gravitational_force(self):
        """Test gravitational force between bodies"""
        query = """
        SELECT 
            -- Force between Earth and Moon: F = GMm/r²
            -- Using average Earth-Moon distance of 384,400 km
            G() * MASS_EARTH() * MASS_MOON() / POWER(3.844e8, 2) as earth_moon_force_n,
            -- Force between Sun and Earth
            G() * MASS_SUN() * MASS_EARTH() / POWER(AU(), 2) as sun_earth_force_n
        FROM DUAL
        """
        result = run_query(query)
        # Earth-Moon force ~1.98e20 N
        assert abs(result[0]["earth_moon_force_n"] - 1.98e20) < 1e19
        # Sun-Earth force ~3.52e22 N
        assert abs(result[0]["sun_earth_force_n"] - 3.52e22) < 1e21
    
    def test_surface_gravity(self):
        """Test surface gravity calculations for planets"""
        query = """
        SELECT 
            -- Surface gravity: g = GM/r²
            -- Earth: radius 6.371e6 m
            G() * MASS_EARTH() / POWER(6.371e6, 2) as earth_g,
            -- Moon: radius 1.737e6 m
            G() * MASS_MOON() / POWER(1.737e6, 2) as moon_g,
            -- Mars: radius 3.390e6 m
            G() * MASS_MARS() / POWER(3.390e6, 2) as mars_g,
            -- Jupiter: radius 7.1492e7 m
            G() * MASS_JUPITER() / POWER(7.1492e7, 2) as jupiter_g
        FROM DUAL
        """
        result = run_query(query)
        # Earth's surface gravity ~9.81 m/s²
        assert abs(result[0]["earth_g"] - 9.82) < 0.02
        # Moon's surface gravity ~1.62 m/s²
        assert abs(result[0]["moon_g"] - 1.625) < 0.01
        # Mars' surface gravity ~3.71 m/s²
        assert abs(result[0]["mars_g"] - 3.73) < 0.02
        # Jupiter's surface gravity ~24.79 m/s²
        assert abs(result[0]["jupiter_g"] - 24.79) < 0.1
    
    def test_unit_conversions_with_astronomy(self):
        """Test unit conversions with astronomical values"""
        query = """
        SELECT 
            -- Convert Earth's orbital distance to miles
            CONVERT(AU(), 'm', 'miles') as earth_orbit_miles,
            -- Convert light-year to kilometers
            CONVERT(LIGHT_YEAR(), 'm', 'km') as ly_km,
            -- Convert parsec to miles
            CONVERT(PARSEC(), 'm', 'miles') as parsec_miles
        FROM DUAL
        """
        result = run_query(query)
        # Earth's orbit ~93 million miles
        assert abs(result[0]["earth_orbit_miles"] - 9.296e7) < 1e6
        # Light-year ~9.46 trillion km
        assert abs(result[0]["ly_km"] - 9.4607e12) < 1e11
        # Parsec ~19.17 trillion miles
        assert abs(result[0]["parsec_miles"] - 1.917e13) < 1e12

class TestComplexAstronomicalQueries:
    """Test complex queries combining multiple astronomical constants"""
    
    def test_planetary_densities(self):
        """Calculate and compare planetary densities"""
        query = """
        SELECT 
            -- Density = mass / volume, assuming spherical planets
            -- Earth: radius 6.371e6 m
            MASS_EARTH() / (4.0/3.0 * PI() * POWER(6.371e6, 3)) as earth_density,
            -- Mars: radius 3.390e6 m  
            MASS_MARS() / (4.0/3.0 * PI() * POWER(3.390e6, 3)) as mars_density,
            -- Jupiter: radius 7.1492e7 m
            MASS_JUPITER() / (4.0/3.0 * PI() * POWER(7.1492e7, 3)) as jupiter_density
        FROM DUAL
        """
        result = run_query(query)
        # Earth density ~5515 kg/m³
        assert abs(result[0]["earth_density"] - 5515) < 50
        # Mars density ~3933 kg/m³
        assert abs(result[0]["mars_density"] - 3933) < 50
        # Jupiter density ~1240 kg/m³ (updated for correct calculation)
        assert abs(result[0]["jupiter_density"] - 1240) < 50
    
    def test_solar_system_scale(self):
        """Test scale comparisons in the solar system"""
        query = """
        SELECT 
            -- How many Earths fit in Jupiter (by volume)
            POWER(7.1492e7 / 6.371e6, 3) as earths_in_jupiter_volume,
            -- How many Moons fit in Earth (by mass)
            MASS_EARTH() / MASS_MOON() as moons_in_earth_mass,
            -- Solar system span (Neptune's orbit diameter) in AU
            2 * DIST_NEPTUNE() / AU() as solar_system_diameter_au
        FROM DUAL
        """
        result = run_query(query)
        # ~1413 Earths fit in Jupiter by volume (updated for correct calculation)
        assert abs(result[0]["earths_in_jupiter_volume"] - 1413) < 20
        # ~81 Moons equal Earth's mass
        assert abs(result[0]["moons_in_earth_mass"] - 81.35) < 0.5
        # Solar system diameter ~60 AU
        assert abs(result[0]["solar_system_diameter_au"] - 60.33) < 0.5

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])