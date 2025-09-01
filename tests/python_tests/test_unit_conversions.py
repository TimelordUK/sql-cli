#!/usr/bin/env python3
"""Test unit conversion functions with CONVERT()"""

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

class TestLengthConversions:
    """Test length unit conversions"""
    
    def test_kilometers_to_miles(self):
        """Test km to miles conversion"""
        result = run_query("SELECT CONVERT(100, 'km', 'miles') as distance FROM DUAL")
        # 100 km = 62.137 miles
        assert abs(result[0]["distance"] - 62.137) < 0.01
    
    def test_miles_to_kilometers(self):
        """Test miles to km conversion"""
        result = run_query("SELECT CONVERT(100, 'miles', 'km') as distance FROM DUAL")
        # 100 miles = 160.934 km
        assert abs(result[0]["distance"] - 160.934) < 0.01
    
    def test_feet_to_meters(self):
        """Test feet to meters conversion"""
        result = run_query("SELECT CONVERT(100, 'ft', 'm') as distance FROM DUAL")
        # 100 ft = 30.48 m
        assert abs(result[0]["distance"] - 30.48) < 0.01
    
    def test_inches_to_centimeters(self):
        """Test inches to cm conversion"""
        result = run_query("SELECT CONVERT(12, 'in', 'cm') as distance FROM DUAL")
        # 12 inches = 30.48 cm (1 foot)
        assert abs(result[0]["distance"] - 30.48) < 0.01
    
    def test_nautical_miles_to_kilometers(self):
        """Test nautical miles to km"""
        result = run_query("SELECT CONVERT(10, 'nmi', 'km') as distance FROM DUAL")
        # 10 nautical miles = 18.52 km
        assert abs(result[0]["distance"] - 18.52) < 0.01
    
    def test_micrometers_to_millimeters(self):
        """Test micrometers to millimeters"""
        result = run_query("SELECT CONVERT(1000, 'um', 'mm') as distance FROM DUAL")
        # 1000 μm = 1 mm
        assert abs(result[0]["distance"] - 1.0) < 0.001

class TestMassConversions:
    """Test mass/weight unit conversions"""
    
    def test_pounds_to_kilograms(self):
        """Test pounds to kg conversion"""
        result = run_query("SELECT CONVERT(100, 'lb', 'kg') as weight FROM DUAL")
        # 100 lb = 45.359 kg
        assert abs(result[0]["weight"] - 45.359) < 0.01
    
    def test_kilograms_to_pounds(self):
        """Test kg to pounds conversion"""
        result = run_query("SELECT CONVERT(50, 'kg', 'lb') as weight FROM DUAL")
        # 50 kg = 110.231 lb
        assert abs(result[0]["weight"] - 110.231) < 0.01
    
    def test_ounces_to_grams(self):
        """Test ounces to grams conversion"""
        result = run_query("SELECT CONVERT(16, 'oz', 'g') as weight FROM DUAL")
        # 16 oz = 453.592 g (1 pound)
        assert abs(result[0]["weight"] - 453.592) < 0.01
    
    def test_metric_tons_to_pounds(self):
        """Test metric tons to pounds"""
        result = run_query("SELECT CONVERT(1, 'tonne', 'lb') as weight FROM DUAL")
        # 1 metric ton = 2204.62 lb
        assert abs(result[0]["weight"] - 2204.62) < 0.1
    
    def test_milligrams_to_grams(self):
        """Test milligrams to grams"""
        result = run_query("SELECT CONVERT(5000, 'mg', 'g') as weight FROM DUAL")
        # 5000 mg = 5 g
        assert abs(result[0]["weight"] - 5.0) < 0.001

class TestTemperatureConversions:
    """Test temperature unit conversions"""
    
    def test_fahrenheit_to_celsius(self):
        """Test Fahrenheit to Celsius conversion"""
        # Freezing point
        result = run_query("SELECT CONVERT(32, 'F', 'C') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 0.0) < 0.01
        
        # Boiling point
        result = run_query("SELECT CONVERT(212, 'F', 'C') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 100.0) < 0.01
        
        # Room temperature
        result = run_query("SELECT CONVERT(72, 'F', 'C') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 22.22) < 0.1
    
    def test_celsius_to_fahrenheit(self):
        """Test Celsius to Fahrenheit conversion"""
        # Freezing point
        result = run_query("SELECT CONVERT(0, 'C', 'F') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 32.0) < 0.01
        
        # Boiling point
        result = run_query("SELECT CONVERT(100, 'C', 'F') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 212.0) < 0.01
    
    def test_celsius_to_kelvin(self):
        """Test Celsius to Kelvin conversion"""
        result = run_query("SELECT CONVERT(0, 'C', 'K') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 273.15) < 0.01
        
        result = run_query("SELECT CONVERT(100, 'C', 'K') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 373.15) < 0.01
    
    def test_kelvin_to_celsius(self):
        """Test Kelvin to Celsius conversion"""
        result = run_query("SELECT CONVERT(273.15, 'K', 'C') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 0.0) < 0.01
        
        result = run_query("SELECT CONVERT(373.15, 'K', 'C') as temp FROM DUAL")
        assert abs(result[0]["temp"] - 100.0) < 0.01

class TestVolumeConversions:
    """Test volume unit conversions"""
    
    def test_gallons_to_liters(self):
        """Test gallons to liters conversion"""
        result = run_query("SELECT CONVERT(1, 'gal', 'L') as volume FROM DUAL")
        # 1 US gallon = 3.785 liters
        assert abs(result[0]["volume"] - 3.785) < 0.01
    
    def test_liters_to_gallons(self):
        """Test liters to gallons conversion"""
        result = run_query("SELECT CONVERT(10, 'L', 'gal') as volume FROM DUAL")
        # 10 L = 2.642 gallons
        assert abs(result[0]["volume"] - 2.642) < 0.01
    
    def test_cups_to_milliliters(self):
        """Test cups to milliliters conversion"""
        result = run_query("SELECT CONVERT(1, 'cup', 'ml') as volume FROM DUAL")
        # 1 US cup = 236.588 ml
        assert abs(result[0]["volume"] - 236.588) < 0.1
    
    def test_tablespoons_to_milliliters(self):
        """Test tablespoons to ml"""
        result = run_query("SELECT CONVERT(1, 'tbsp', 'ml') as volume FROM DUAL")
        # 1 tbsp = 14.787 ml
        assert abs(result[0]["volume"] - 14.787) < 0.1
    
    def test_cubic_meters_to_liters(self):
        """Test cubic meters to liters"""
        result = run_query("SELECT CONVERT(1, 'm3', 'L') as volume FROM DUAL")
        # 1 m³ = 1000 L
        assert abs(result[0]["volume"] - 1000.0) < 0.01

class TestTimeConversions:
    """Test time unit conversions"""
    
    def test_hours_to_minutes(self):
        """Test hours to minutes conversion"""
        result = run_query("SELECT CONVERT(2.5, 'hour', 'min') as time FROM DUAL")
        # 2.5 hours = 150 minutes
        assert abs(result[0]["time"] - 150.0) < 0.01
    
    def test_days_to_hours(self):
        """Test days to hours conversion"""
        result = run_query("SELECT CONVERT(3, 'days', 'hours') as time FROM DUAL")
        # 3 days = 72 hours
        assert abs(result[0]["time"] - 72.0) < 0.01
    
    def test_milliseconds_to_seconds(self):
        """Test milliseconds to seconds"""
        result = run_query("SELECT CONVERT(5000, 'ms', 's') as time FROM DUAL")
        # 5000 ms = 5 s
        assert abs(result[0]["time"] - 5.0) < 0.01
    
    def test_weeks_to_days(self):
        """Test weeks to days"""
        result = run_query("SELECT CONVERT(2, 'weeks', 'days') as time FROM DUAL")
        # 2 weeks = 14 days
        assert abs(result[0]["time"] - 14.0) < 0.01
    
    def test_years_to_days(self):
        """Test years to days (accounting for leap years)"""
        result = run_query("SELECT CONVERT(1, 'year', 'days') as time FROM DUAL")
        # 1 year = 365.25 days (average)
        assert abs(result[0]["time"] - 365.25) < 0.01

class TestAreaConversions:
    """Test area unit conversions"""
    
    def test_square_feet_to_square_meters(self):
        """Test sq ft to sq m conversion"""
        result = run_query("SELECT CONVERT(100, 'sq_ft', 'm2') as area FROM DUAL")
        # 100 sq ft = 9.290 sq m
        assert abs(result[0]["area"] - 9.290) < 0.01
    
    def test_acres_to_hectares(self):
        """Test acres to hectares conversion"""
        result = run_query("SELECT CONVERT(10, 'acres', 'hectares') as area FROM DUAL")
        # 10 acres = 4.047 hectares
        assert abs(result[0]["area"] - 4.047) < 0.01
    
    def test_square_miles_to_square_kilometers(self):
        """Test sq miles to sq km"""
        result = run_query("SELECT CONVERT(1, 'sq_mi', 'km2') as area FROM DUAL")
        # 1 sq mile = 2.590 sq km
        assert abs(result[0]["area"] - 2.590) < 0.01

class TestSpeedConversions:
    """Test speed unit conversions"""
    
    def test_mph_to_kph(self):
        """Test miles per hour to km per hour"""
        result = run_query("SELECT CONVERT(60, 'mph', 'kph') as speed FROM DUAL")
        # 60 mph = 96.561 km/h
        assert abs(result[0]["speed"] - 96.561) < 0.1
    
    def test_knots_to_mph(self):
        """Test knots to miles per hour"""
        result = run_query("SELECT CONVERT(100, 'knots', 'mph') as speed FROM DUAL")
        # 100 knots = 115.078 mph
        assert abs(result[0]["speed"] - 115.078) < 0.1
    
    def test_meters_per_second_to_kph(self):
        """Test m/s to km/h"""
        result = run_query("SELECT CONVERT(10, 'm/s', 'kph') as speed FROM DUAL")
        # 10 m/s = 36 km/h
        assert abs(result[0]["speed"] - 36.0) < 0.01

class TestPressureConversions:
    """Test pressure unit conversions"""
    
    def test_psi_to_bar(self):
        """Test PSI to bar conversion"""
        result = run_query("SELECT CONVERT(30, 'psi', 'bar') as pressure FROM DUAL")
        # 30 PSI = 2.068 bar
        assert abs(result[0]["pressure"] - 2.068) < 0.01
    
    def test_atmospheres_to_pascal(self):
        """Test atmospheres to Pascal"""
        result = run_query("SELECT CONVERT(1, 'atm', 'Pa') as pressure FROM DUAL")
        # 1 atm = 101325 Pa
        assert abs(result[0]["pressure"] - 101325.0) < 1.0
    
    def test_torr_to_millibar(self):
        """Test torr to millibar"""
        result = run_query("SELECT CONVERT(760, 'torr', 'mbar') as pressure FROM DUAL")
        # 760 torr = 1013.25 mbar (1 atm)
        assert abs(result[0]["pressure"] - 1013.25) < 0.1

class TestMixedCalculations:
    """Test calculations combining unit conversions"""
    
    def test_fuel_efficiency_conversion(self):
        """Convert fuel consumption from miles per gallon to liters per 100km"""
        # Note: This requires multiple conversions
        # 30 mpg = 7.84 L/100km (approximately)
        # We'll calculate: (100 km / miles) * (gallons / 1) * (liters/gallon)
        result = run_query("""
            SELECT 
                CONVERT(100, 'km', 'miles') as km_to_miles,
                CONVERT(1, 'gal', 'L') as gal_to_L,
                (CONVERT(100, 'km', 'miles') / 30.0) * CONVERT(1, 'gal', 'L') as L_per_100km
            FROM DUAL
        """)
        # 30 mpg ≈ 7.84 L/100km
        assert abs(result[0]["L_per_100km"] - 7.84) < 0.1
    
    def test_bmi_calculation_imperial_to_metric(self):
        """Calculate BMI converting from imperial to metric units"""
        # BMI = weight(kg) / height(m)²
        # Example: 180 lbs, 6 feet (72 inches)
        result = run_query("""
            SELECT 
                CONVERT(180, 'lb', 'kg') as weight_kg,
                CONVERT(72, 'in', 'm') as height_m,
                CONVERT(180, 'lb', 'kg') / (CONVERT(72, 'in', 'm') * CONVERT(72, 'in', 'm')) as bmi
            FROM DUAL
        """)
        # BMI should be around 24.4
        assert abs(result[0]["bmi"] - 24.4) < 0.2
    
    def test_physics_calculation_with_units(self):
        """Calculate kinetic energy with unit conversions"""
        # KE = 0.5 * m * v²
        # Mass: 2000 lbs, Speed: 60 mph
        # Convert to SI units (kg, m/s) for energy in Joules
        result = run_query("""
            SELECT 
                CONVERT(2000, 'lb', 'kg') as mass_kg,
                CONVERT(60, 'mph', 'm/s') as speed_ms,
                0.5 * CONVERT(2000, 'lb', 'kg') * CONVERT(60, 'mph', 'm/s') * CONVERT(60, 'mph', 'm/s') as KE_joules
            FROM DUAL
        """)
        # KE should be around 326,000 Joules
        assert abs(result[0]["KE_joules"] - 326000) < 5000

class TestEdgeCases:
    """Test edge cases and error handling"""
    
    def test_case_insensitive_units(self):
        """Test that unit names are case insensitive"""
        result1 = run_query("SELECT CONVERT(100, 'KM', 'MILES') as d1 FROM DUAL")
        result2 = run_query("SELECT CONVERT(100, 'km', 'miles') as d2 FROM DUAL")
        result3 = run_query("SELECT CONVERT(100, 'Km', 'Miles') as d3 FROM DUAL")
        
        assert abs(result1[0]["d1"] - result2[0]["d2"]) < 0.001
        assert abs(result2[0]["d2"] - result3[0]["d3"]) < 0.001
    
    def test_unit_aliases(self):
        """Test that unit aliases work correctly"""
        # Test various aliases for the same unit
        result1 = run_query("SELECT CONVERT(10, 'kilometer', 'mi') as d FROM DUAL")
        result2 = run_query("SELECT CONVERT(10, 'kilometers', 'mile') as d FROM DUAL")
        result3 = run_query("SELECT CONVERT(10, 'km', 'miles') as d FROM DUAL")
        
        assert abs(result1[0]["d"] - result2[0]["d"]) < 0.001
        assert abs(result2[0]["d"] - result3[0]["d"]) < 0.001
    
    def test_precision_preservation(self):
        """Test that precision is preserved in conversions"""
        # Convert 1 meter to millimeters and back
        result = run_query("""
            SELECT 
                CONVERT(1, 'm', 'mm') as to_mm,
                CONVERT(CONVERT(1, 'm', 'mm'), 'mm', 'm') as back_to_m
            FROM DUAL
        """)
        assert result[0]["to_mm"] == 1000.0
        assert abs(result[0]["back_to_m"] - 1.0) < 1e-10

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])