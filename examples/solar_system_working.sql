-- #! data/solar_system.csv
-- Solar System Calculations - Working Version
-- This version avoids parser limitations with functions in WHERE/ORDER BY clauses
-- Run with: ./target/release/sql-cli data/solar_system.csv -f examples/solar_system_working.sql

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
    position,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name), 1) AS orbital_period_days,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS orbital_period_years
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY position;
GO

-- Density comparison
SELECT
    name,
    type,
    ROUND(DENSITY_SOLAR_BODY(name), 0) AS density_kgm3,
    ROUND(DENSITY_SOLAR_BODY(name) / 1000, 2) AS density_gcm3
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
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name) / 24, 2) AS rotation_days
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY rotation_hours;
GO

-- Distance from Sun in various units
SELECT
    name,
    position,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS distance_au,
    ROUND(DISTANCE_SOLAR_BODY(name) / 1e9, 1) AS distance_million_km
FROM test
WHERE type != 'Star' AND name != 'Moon'
ORDER BY position;
GO

-- Calculate distances between consecutive planets using LAG
SELECT
    name,
    position,
    LAG(name, 1) OVER (ORDER BY position) AS previous_body,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS distance_au
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
ORDER BY position;
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

-- Volume and density calculations
SELECT
    name,
    type,
    ROUND(4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY(name), 3) / 1e18, 2) AS volume_billion_km3,
    ROUND(MASS_SOLAR_BODY(name) / (4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY(name), 3)), 0) AS calc_density_kgm3,
    ROUND(DENSITY_SOLAR_BODY(name), 0) AS stored_density_kgm3
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

-- Kepler's Third Law verification for inner planets
SELECT
    name,
    position,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS distance_au,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS period_years,
    ROUND(POW(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2), 2) AS period_squared,
    ROUND(POW(DISTANCE_SOLAR_BODY(name) / AU(), 3), 2) AS distance_cubed
FROM test
WHERE type = 'Terrestrial'
ORDER BY position;
GO

-- Giant planets comparison
SELECT
    name,
    type,
    ROUND(MASS_SOLAR_BODY(name) / MASS_SOLAR_BODY('Jupiter'), 3) AS jupiter_mass_ratio,
    ROUND(RADIUS_SOLAR_BODY(name) / RADIUS_SOLAR_BODY('Jupiter'), 3) AS jupiter_radius_ratio,
    MOONS_SOLAR_BODY(name) AS num_moons
FROM test
WHERE type IN ('Gas Giant', 'Ice Giant')
ORDER BY jupiter_mass_ratio DESC;
GO

-- Dwarf planets comparison
SELECT
    name,
    ROUND(MASS_SOLAR_BODY(name) / 1e22, 2) AS mass_10e22_kg,
    ROUND(RADIUS_SOLAR_BODY(name) / 1e6, 2) AS radius_million_m,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
    MOONS_SOLAR_BODY(name) AS num_moons
FROM test
WHERE type = 'Dwarf Planet'
ORDER BY mass_10e22_kg DESC;
GO