-- Solar System Functions Demo
-- Demonstrates the solar system lookup functions with the CSV data
-- Run: ./target/release/sql-cli examples/solar_system.csv < examples/solar_system_demo.sql

-- Show all bodies with their key properties
SELECT 
    name,
    type,
    ROUND(MASS_SOLAR_BODY(name) / 1e24, 3) AS mass_10e24_kg,
    ROUND(RADIUS_SOLAR_BODY(name) / 1e6, 3) AS radius_Mm,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2
FROM test
ORDER BY position;
GO

-- Find high-gravity worlds (greater than Earth's)
SELECT 
    name,
    type,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
    ROUND(GRAVITY_SOLAR_BODY(name) / 9.807, 2) AS earth_g
FROM test
WHERE GRAVITY_SOLAR_BODY(name) > 9.807
ORDER BY gravity_ms2 DESC;
GO

-- Low-gravity bodies where you could jump very high
SELECT
    name,
    type,
    ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
    ROUND(ESCAPE_VELOCITY_SOLAR_BODY(name), 0) AS escape_vel_ms
FROM test
WHERE GRAVITY_SOLAR_BODY(name) < 2.0
ORDER BY gravity_ms2;
GO

-- Dense rocky worlds vs gas giants
SELECT
    name,
    type,
    ROUND(DENSITY_SOLAR_BODY(name), 0) AS density_kgm3,
    CASE 
        WHEN DENSITY_SOLAR_BODY(name) > 5000 THEN 'Very Dense (Metal/Rock)'
        WHEN DENSITY_SOLAR_BODY(name) > 3000 THEN 'Dense (Rocky)'
        WHEN DENSITY_SOLAR_BODY(name) > 1000 THEN 'Light (Icy/Rocky)'
        ELSE 'Very Light (Gas)'
    END AS composition
FROM test
WHERE DENSITY_SOLAR_BODY(name) > 500  -- Filter out the lightest
ORDER BY density_kgm3 DESC;
GO

-- Bodies with many moons
SELECT
    name,
    type,
    MOONS_SOLAR_BODY(name) AS num_moons
FROM test
WHERE MOONS_SOLAR_BODY(name) > 10
ORDER BY num_moons DESC;
GO

-- Fast rotators (short days)
SELECT
    name,
    type,
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name), 2) AS rotation_hours,
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name) / 24, 2) AS earth_days
FROM test
WHERE ROTATION_PERIOD_SOLAR_BODY(name) < 24  -- Faster than Earth
ORDER BY rotation_hours;
GO

-- Slow rotators (very long days)
SELECT
    name,
    type,
    ROUND(ROTATION_PERIOD_SOLAR_BODY(name) / 24, 1) AS rotation_days
FROM test
WHERE ROTATION_PERIOD_SOLAR_BODY(name) > 100  -- More than 4 Earth days
ORDER BY rotation_days DESC;
GO

-- Orbital periods - quick orbits
SELECT
    name,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name), 1) AS orbit_days,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS orbit_years
FROM test
WHERE ORBITAL_PERIOD_SOLAR_BODY(name) > 0 
  AND ORBITAL_PERIOD_SOLAR_BODY(name) < 1000  -- Less than 3 Earth years
ORDER BY orbit_days;
GO

-- Far distant worlds
SELECT
    name,
    type,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 1) AS distance_au,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 0) AS orbit_years
FROM test
WHERE DISTANCE_SOLAR_BODY(name) / AU() > 20  -- Beyond Uranus
ORDER BY distance_au DESC;
GO

-- Weight comparison for a 100kg person
SELECT 
    name,
    type,
    ROUND(100 * GRAVITY_SOLAR_BODY(name) / 9.807, 1) AS weight_kg,
    CASE 
        WHEN GRAVITY_SOLAR_BODY(name) / 9.807 < 0.1 THEN 'Almost weightless!'
        WHEN GRAVITY_SOLAR_BODY(name) / 9.807 < 0.5 THEN 'Very light'
        WHEN GRAVITY_SOLAR_BODY(name) / 9.807 < 0.9 THEN 'Lighter'
        WHEN GRAVITY_SOLAR_BODY(name) / 9.807 < 1.1 THEN 'Earth-like'
        ELSE 'Heavier'
    END AS feeling
FROM test
WHERE type IN ('Terrestrial', 'Dwarf Planet', 'Moon')
ORDER BY weight_kg DESC;
GO

-- Escape velocity comparison - where rockets struggle
SELECT
    name,
    type,
    ROUND(ESCAPE_VELOCITY_SOLAR_BODY(name) / 1000, 1) AS escape_km_s,
    ROUND(ESCAPE_VELOCITY_SOLAR_BODY(name) / 11186, 2) AS earth_escape_ratio
FROM test
WHERE ESCAPE_VELOCITY_SOLAR_BODY(name) > 10000  -- Harder than Earth
ORDER BY escape_km_s DESC;
GO

-- Calculate planet volumes and compare to Earth
SELECT
    name,
    ROUND(4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY(name), 3) / (4.0/3.0 * PI() * POW(RADIUS_SOLAR_BODY('Earth'), 3)), 1) AS volume_earths,
    ROUND(MASS_SOLAR_BODY(name) / MASS_SOLAR_BODY('Earth'), 1) AS mass_earths
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
ORDER BY volume_earths DESC;
GO

-- Using LAG to show planet spacing
SELECT
    name,
    position,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS dist_au,
    LAG(name, 1) OVER (ORDER BY position) AS prev_planet
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
ORDER BY position;
GO

-- Verify Kepler's 3rd Law (P² = a³ in appropriate units)
SELECT
    name,
    ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS a_au,
    ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS P_years,
    ROUND(POW(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) / POW(DISTANCE_SOLAR_BODY(name) / AU(), 3), 3) AS keplers_const
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
  AND ORBITAL_PERIOD_SOLAR_BODY(name) > 0
ORDER BY a_au;
GO

-- Summary statistics
SELECT
    'Planets' AS category,
    COUNT(*) AS count,
    ROUND(AVG(GRAVITY_SOLAR_BODY(name)), 1) AS avg_gravity,
    ROUND(AVG(MOONS_SOLAR_BODY(name)), 0) AS avg_moons
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant');
GO