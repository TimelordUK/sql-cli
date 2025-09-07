-- ============================================================================
-- Unit Conversions Using CONVERT() Function
-- Demonstrates comprehensive unit conversion capabilities in SQL CLI
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/unit_conversions_with_convert.sql -o csv
-- ============================================================================

-- Temperature Conversions
SELECT 
    '=== TEMPERATURE ===' as category,
    CONVERT(0, 'celsius', 'fahrenheit') as freezing_f,
    CONVERT(100, 'celsius', 'fahrenheit') as boiling_f,
    CONVERT(25, 'celsius', 'kelvin') as room_temp_k,
    CONVERT(98.6, 'fahrenheit', 'celsius') as body_temp_c,
    CONVERT(0, 'kelvin', 'celsius') as absolute_zero_c;
GO

-- Distance Conversions
SELECT 
    '=== DISTANCE ===' as category,
    CONVERT(42.195, 'km', 'miles') as marathon_miles,
    CONVERT(100, 'meters', 'feet') as sprint_feet,
    CONVERT(1, 'mile', 'km') as mile_to_km,
    CONVERT(1, 'yard', 'meters') as yard_to_m,
    CONVERT(1, 'inch', 'cm') as inch_to_cm;
GO

-- Weight/Mass Conversions
SELECT 
    '=== WEIGHT ===' as category,
    CONVERT(70, 'kg', 'pounds') as weight_lbs,
    CONVERT(150, 'pounds', 'kg') as weight_kg,
    CONVERT(1, 'ounce', 'grams') as oz_to_g,
    CONVERT(1, 'ton', 'kg') as ton_to_kg,
    CONVERT(1, 'stone', 'kg') as stone_to_kg;
GO

-- Volume Conversions
SELECT 
    '=== VOLUME ===' as category,
    CONVERT(1, 'gallon', 'liters') as gal_to_l,
    CONVERT(1, 'liter', 'gallon') as l_to_gal,
    CONVERT(1, 'cup', 'ml') as cup_to_ml,
    CONVERT(1, 'tablespoon', 'ml') as tbsp_to_ml,
    CONVERT(1, 'teaspoon', 'ml') as tsp_to_ml;
GO

-- Area Conversions
SELECT 
    '=== AREA ===' as category,
    CONVERT(100, 'sqm', 'sq_ft') as apt_sqft,
    CONVERT(1, 'hectare', 'acres') as ha_to_acre,
    CONVERT(1, 'sqkm', 'sqmiles') as sqkm_to_sqmi,
    CONVERT(1, 'acre', 'sqm') as acre_to_sqm;
GO

-- Speed Conversions
SELECT 
    '=== SPEED ===' as category,
    CONVERT(100, 'kph', 'mph') as speed_mph,
    CONVERT(60, 'mph', 'kph') as highway_kph,
    CONVERT(1, 'mps', 'kph') as meters_per_sec_to_kph,
    CONVERT(1, 'knot', 'mph') as knot_to_mph;
GO

-- Pressure Conversions
SELECT 
    '=== PRESSURE ===' as category,
    CONVERT(1, 'bar', 'psi') as bar_to_psi,
    CONVERT(14.696, 'psi', 'bar') as atm_to_bar,
    CONVERT(1, 'atm', 'pascal') as atm_to_pa,
    CONVERT(760, 'torr', 'mbar') as torr_to_mbar;
GO

-- Time Conversions
SELECT 
    '=== TIME ===' as category,
    CONVERT(1, 'hour', 'minutes') as hr_to_min,
    CONVERT(1, 'day', 'hours') as day_to_hr,
    CONVERT(1, 'week', 'days') as week_to_days,
    CONVERT(1, 'year', 'days') as year_to_days,
    CONVERT(1000, 'milliseconds', 'seconds') as ms_to_s;
GO

-- Energy Conversions
SELECT 
    '=== ENERGY ===' as category,
    CONVERT(1, 'joule', 'calorie') as j_to_cal,
    CONVERT(1, 'kwh', 'joule') as kwh_to_j,
    CONVERT(1, 'btu', 'joule') as btu_to_j,
    CONVERT(1, 'erg', 'joule') as erg_to_j;
GO

-- Astronomical Distance Conversions
SELECT 
    '=== ASTRONOMICAL ===' as category,
    CONVERT(1, 'au', 'km') as au_in_km,
    CONVERT(1, 'lightyear', 'km') as lightyear_in_km,
    CONVERT(1, 'parsec', 'lightyear') as parsec_in_ly,
    CONVERT(384400, 'km', 'miles') as moon_distance_miles;
GO

-- Practical Examples: International Travel
SELECT 
    '=== TRAVEL EXAMPLE ===' as category,
    CONVERT(500, 'miles', 'km') as drive_distance_km,
    CONVERT(70, 'mph', 'kph') as speed_limit_kph,
    CONVERT(50, 'pounds', 'kg') as luggage_kg,
    CONVERT(72, 'fahrenheit', 'celsius') as weather_c;
GO

-- Practical Examples: Cooking
SELECT 
    '=== COOKING EXAMPLE ===' as category,
    CONVERT(350, 'fahrenheit', 'celsius') as oven_temp_c,
    CONVERT(2, 'cup', 'ml') as flour_ml,
    CONVERT(3, 'tablespoon', 'ml') as sugar_ml,
    CONVERT(8, 'ounce', 'grams') as butter_g;
GO

-- Practical Examples: Science Lab
SELECT 
    '=== LAB EXAMPLE ===' as category,
    CONVERT(1, 'atm', 'pascal') as standard_pressure_pa,
    CONVERT(300, 'kelvin', 'celsius') as lab_temp_c,
    CONVERT(1500, 'ml', 'liter') as solution_l,
    CONVERT(2.5, 'grams', 'mg') as sample_mg;
GO

-- Complex Calculations with Mixed Units
SELECT 
    '=== PHYSICS CALCULATIONS ===' as category,
    -- Calculate g using Earth parameters with unit conversion
    G() * MASS_EARTH() / POWER(CONVERT(RADIUS_EARTH(), 'meters', 'km'), 2) as g_calc,
    -- Convert escape velocity to km/h
    CONVERT(SQRT(2 * G() * MASS_EARTH() / RADIUS_EARTH()), 'mps', 'kph') as escape_velocity_kph,
    -- Energy conversions
    CONVERT(3600, 'joule', 'kwh') as joules_to_kwh;
GO

-- Data Storage Conversions
--SELECT 
--    '=== DATA STORAGE ===' as category,
--    CONVERT(1, 'gb', 'mb') as gb_to_mb,
--    CONVERT(1, 'tb', 'gb') as tb_to_gb,
--    CONVERT(1024, 'mb', 'gb') as mb_to_gb,
--    CONVERT(1, 'gib', 'gb') as gib_to_gb;
--GO

-- Summary of Available Conversion Categories
SELECT 
    'CONVERT Function' as feature,
    'Supports temperature, distance, weight, volume, area, speed, pressure, time, energy, data' as categories,
    'CONVERT(value, from_unit, to_unit)' as syntax,
    'Case-insensitive units, handles abbreviations and full names' as notes;
GO
