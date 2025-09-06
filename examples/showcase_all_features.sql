-- SQL-CLI Feature Showcase
-- This file demonstrates all implemented features with working examples
-- Each section is separated by GO to execute as independent statements

-- ===== MATHEMATICAL FUNCTIONS =====
-- Basic math operations and advanced functions
SELECT 
    -- Arithmetic
    10 + 5 as addition,
    20 - 8 as subtraction,
    7 * 6 as multiplication,
    100 / 4 as division,
    10 % 3 as modulo,
    
    -- Power and roots
    POWER(2, 10) as two_to_tenth,
    SQRT(144) as square_root,
    
    -- Rounding
    ROUND(3.14159, 2) as rounded,
    FLOOR(9.8) as floored,
    CEIL(9.2) as ceiling,
    
    -- Advanced
    ABS(-42) as absolute,
    FACTORIAL(5) as five_factorial,
    LN(2.71828) as natural_log,
    LOG(100) as log_base10,
    EXP(1) as e_to_one;
GO

-- ===== PRIME NUMBER FUNCTIONS =====
-- Working with prime numbers
SELECT 
    PRIME(10) as tenth_prime,
    NTH_PRIME(20) as twentieth_prime,
    PRIME_COUNT(100) as primes_up_to_100,
    PRIME_PI(50) as primes_up_to_50,
    IS_PRIME(17) as seventeen_is_prime,
    IS_PRIME(18) as eighteen_is_prime;
GO

-- ===== PHYSICS CONSTANTS =====
-- Fundamental physical constants
SELECT 
    C() as speed_of_light_ms,
    H() as planck_constant,
    G() as gravitational_constant,
    K() as boltzmann_constant,
    AVOGADRO() as avogadro_number,
    ME() as electron_mass,
    MP() as proton_mass,
    MN() as neutron_mass;
GO

-- ===== ASTRONOMICAL MASSES =====
-- Celestial body masses in kg
SELECT 
    MASS_SUN() as sun_mass,
    MASS_EARTH() as earth_mass,
    MASS_MOON() as moon_mass,
    MASS_JUPITER() as jupiter_mass,
    MASS_MARS() as mars_mass,
    MASS_SUN() / MASS_EARTH() as sun_earth_ratio;
GO

-- ===== CHEMISTRY FUNCTIONS =====
-- Chemical formulas and atomic masses
SELECT 
    MOLECULE_FORMULA('water') as water_formula,
    ATOMIC_MASS('water') as water_mass,
    MOLECULE_FORMULA('methane') as methane_formula,
    ATOMIC_MASS('methane') as methane_mass,
    MOLECULE_FORMULA('glucose') as glucose_formula,
    ATOMIC_MASS('glucose') as glucose_mass;
GO

-- ===== STRING FUNCTIONS =====
-- Text manipulation capabilities
SELECT 
    -- Case conversion
    UPPER('hello world') as uppercase,
    LOWER('HELLO WORLD') as lowercase,
    
    -- String properties
    LENGTH('sql-cli') as string_length,
    
    -- String modification
    TRIM('  spaces  ') as trimmed,
    SUBSTRING('Hello World', 1, 5) as substr,
    MID('Database', 3, 4) as mid_extract,
    REPLACE('Hello World', 'World', 'SQL-CLI') as replaced;
GO

-- ===== ADVANCED STRING FUNCTIONS =====
-- More string operations
SELECT 
    -- String joining
    TEXTJOIN(' ', 1, 'SQL', 'CLI', 'Rocks') as joined_text,
    TEXTJOIN(', ', 1, 'apple', 'banana', 'orange') as fruit_list,
    
    -- String checking
    STARTSWITH('sql-cli', 'sql') as starts_with_sql,
    ENDSWITH('database.db', '.db') as ends_with_db,
    CONTAINS('hello world', 'world') as contains_world,
    
    -- Fuzzy matching
    EDIT_DISTANCE('kitten', 'sitting') as edit_dist;
GO

-- ===== DATE AND TIME FUNCTIONS =====
-- Working with dates and times
SELECT 
    -- Current date/time
    NOW() as current_timestamp,
    TODAY() as current_date;
GO

-- ===== DATE ARITHMETIC =====
-- Date calculations
SELECT 
    -- Adding intervals
    DATEADD('day', 7, '2024-01-01') as week_later,
    DATEADD('month', 3, '2024-01-01') as three_months_later,
    DATEADD('year', 1, '2024-01-01') as next_year,
    
    -- Date differences
    DATEDIFF('day', '2024-01-01', '2024-12-31') as days_in_2024,
    DATEDIFF('month', '2024-01-01', '2024-12-31') as months_diff,
    DATEDIFF('year', '2000-01-01', '2024-01-01') as years_diff;
GO

-- Note: EXTRACT, DATE_FORMAT, and DATE_PARSE are not yet implemented
-- These would be useful additions for future versions

-- ===== PRACTICAL PHYSICS CALCULATIONS =====
-- Real-world physics applications
SELECT 
    -- Escape velocity from Earth: v = sqrt(2GM/r)
    SQRT(2 * G() * MASS_EARTH() / 6371000) as earth_escape_velocity_ms,
    
    -- Energy from mass (E=mc²)
    1 * POWER(C(), 2) as energy_joules_from_1kg,
    
    -- Schwarzschild radius of Sun
    (2 * G() * MASS_SUN()) / POWER(C(), 2) as sun_schwarzschild_radius_m;
GO

-- ===== FACTORIAL SHOWCASE =====
-- Demonstrate factorial function with lookup table
SELECT 
    FACTORIAL(0) as fact_0,
    FACTORIAL(5) as fact_5,
    FACTORIAL(10) as fact_10,
    FACTORIAL(15) as fact_15,
    FACTORIAL(20) as fact_20;
GO

-- End of showcase
-- All statements above work with sql-cli -f showcase_all_features.sql