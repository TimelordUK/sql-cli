-- #! data/solar_system.csv
-- Solar System Calculations Example (Simplified)
-- This version works around parser limitations with functions in ORDER BY clauses
-- Run with: ./target/release/sql-cli data/solar_system.csv -f examples/solar_system_calculations_simple.sql

-- Basic properties lookup
SELECT 
    name,
    type,
    position,
    ROUND(MASS_SOLAR_BODY(name) / 1e24, 3) AS mass_10e24_kg,
    ROUND(RADIUS_SOLAR_BODY(name) / 1e6, 3) AS radius_million_m,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS surface_gravity_ms2
FROM test
WHERE name != 'Moon'
ORDER BY position;
GO

-- Calculate surface gravity relative to Earth
SELECT 
    name,
    type,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
    ROUND(GRAVITY_SOLAR_BODY(name) / 9.807, 3) AS earth_g
FROM test
WHERE type IN ('Terrestrial', 'Dwarf Planet')
ORDER BY gravity_ms2 DESC;
GO

-- Calculate escape velocities
SELECT
    name,
    type,
    ROUND(ESCAPE_VELOCITY_SOLAR_BODY(name) / 1000, 2) AS escape_velocity_kms
FROM test
WHERE name != 'Moon'
ORDER BY escape_velocity_kms DESC;
GO

-- Orbital periods in Earth years
SELECT
    name,
    type,
    mean_distance_au,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name), 1) AS orbital_period_days,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS orbital_period_years
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY mean_distance_au;
GO

-- Density comparison
SELECT
    name,
    type,
    ROUND(DENSITY_SOLAR_BODY(name), 0) AS density_kgm3,
    ROUND(DENSITY_SOLAR_BODY(name) / 1000, 2) AS density_gcm3,
    CASE 
        WHEN DENSITY_SOLAR_BODY(name) < 1000 THEN 'Less than water'
        WHEN DENSITY_SOLAR_BODY(name) < 3000 THEN 'Rocky'
        WHEN DENSITY_SOLAR_BODY(name) < 5000 THEN 'Rocky/Metallic'
        ELSE 'Metallic'
    END AS density_class
FROM test
WHERE name != 'Moon'
ORDER BY density_kgm3 DESC;
GO

-- Number of moons
SELECT
    name,
    type,
    MOONS_SOLAR_BODY(name) AS num_moons
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant', 'Dwarf Planet')
ORDER BY num_moons DESC;
GO

-- Rotation periods (day length)
SELECT
    name,
    type,
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name), 2) AS rotation_hours,
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name) / 24, 2) AS rotation_days,
    CASE
        WHEN ROTATION_PERIOD_SOLAR_BODY(name) < 24 THEN 'Shorter than Earth day'
        WHEN ROTATION_PERIOD_SOLAR_BODY(name) < 48 THEN 'Similar to Earth day'
        ELSE 'Much longer than Earth day'
    END AS day_comparison
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY rotation_hours;
GO

-- Distance from Sun in various units
SELECT
    name,
    mean_distance_au,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS distance_au_calc,
    ROUND(DISTANCE_SOLAR_BODY(name) / 1e9, 1) AS distance_million_km,
    ROUND(DISTANCE_SOLAR_BODY(name) / LIGHT_YEAR(), 6) AS distance_light_years
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY mean_distance_au;
GO

-- Calculate distances between consecutive planets using LAG
SELECT
    name,
    mean_distance_au,
    LAG(name, 1) OVER (ORDER BY mean_distance_au) AS previous_body,
    LAG(mean_distance_au, 1) OVER (ORDER BY mean_distance_au) AS prev_distance_au,
    ROUND(mean_distance_au - LAG(mean_distance_au, 1) OVER (ORDER BY mean_distance_au), 3) AS gap_au
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
ORDER BY mean_distance_au;
GO

-- Kepler's Third Law verification: P² ∝ a³
SELECT
    name,
    mean_distance_au,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS period_years,
    ROUND(POW(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2), 2) AS period_squared,
    ROUND(POW(mean_distance_au, 3), 2) AS distance_cubed
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant') 
ORDER BY mean_distance_au;
GO

-- Calculate weight of 100kg on different bodies
SELECT 
    name,
    type,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
    ROUND(100 * GRAVITY_SOLAR_BODY(name), 0) AS weight_100kg_newtons,
    ROUND(100 * GRAVITY_SOLAR_BODY(name) / 9.807, 1) AS apparent_weight_kg
FROM test
WHERE type IN ('Terrestrial', 'Dwarf Planet', 'Moon')
ORDER BY weight_100kg_newtons DESC;
GO

-- Volume calculations
SELECT
    name,
    type,
    ROUND(4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY(name), 3) / 1e18, 2) AS volume_billion_km3,
    ROUND(MASS_SOLAR_BODY(name) / (4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY(name), 3)), 0) AS calc_density_kgm3
FROM test
WHERE name IN ('Earth', 'Mars', 'Jupiter', 'Saturn', 'Pluto')
ORDER BY volume_billion_km3 DESC;
GO

-- Comparison of planets to Earth
SELECT 
    name,
    position,
    ROUND(MASS_SOLAR_BODY(name) / MASS_SOLAR_BODY('Earth'), 3) AS mass_earth_ratio,
    ROUND(RADIUS_SOLAR_BODY(name) / RADIUS_SOLAR_BODY('Earth'), 3) AS radius_earth_ratio,
    ROUND(GRAVITY_SOLAR_BODY(name) / GRAVITY_SOLAR_BODY('Earth'), 3) AS gravity_earth_ratio
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
ORDER BY position;
GO