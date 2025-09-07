-- ============================================================================
-- Solar System Queries with CTEs (Working Examples)
-- Demonstrates using CTEs to compute and filter on solar system functions
-- ============================================================================
-- Run: ./target/release/sql-cli data/solar_system.csv -f examples/solar_system_with_cte.sql -o csv
-- ============================================================================

-- Example 1: Compute planet properties once and filter
WITH planet_properties AS (
    SELECT 
        name,
        type,
        position,
        mean_distance_au,
        MASS_SOLAR_BODY(name) AS mass_kg,
        RADIUS_SOLAR_BODY(name) AS radius_m,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2,
        ESCAPE_VELOCITY_SOLAR_BODY(name) AS escape_vel_ms
    FROM test
    WHERE type != 'Star'
)
SELECT 
    name,
    type,
    ROUND(gravity_ms2, 2) AS gravity,
    ROUND(escape_vel_ms / 1000, 1) AS escape_km_s
FROM planet_properties
WHERE gravity_ms2 > 10;  -- Filter on computed value!
GO

-- Example 2: Orbital period analysis
WITH orbital_data AS (
    SELECT 
        name,
        type,
        DISTANCE_SOLAR_BODY(name) / AU() AS distance_au,
        ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256 AS period_years
    FROM test
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
)
SELECT 
    name,
    ROUND(distance_au, 2) AS dist_au,
    ROUND(period_years, 2) AS period_yr
FROM orbital_data
WHERE period_years < 30;  -- Inner solar system
GO

-- Example 3: Density analysis with categorization
WITH density_calc AS (
    SELECT 
        name,
        type,
        DENSITY_SOLAR_BODY(name) AS density_kgm3,
        CASE 
            WHEN DENSITY_SOLAR_BODY(name) < 1000 THEN 'Low density (gas)'
            WHEN DENSITY_SOLAR_BODY(name) < 3000 THEN 'Medium density (ice/gas)'
            ELSE 'High density (rock/metal)'
        END AS density_category
    FROM test
    WHERE name != 'Sun'
)
SELECT 
    name,
    type,
    ROUND(density_kgm3, 0) AS density,
    density_category
FROM density_calc
WHERE density_category = 'High density (rock/metal)';
GO

-- Example 4: Moons analysis
WITH moon_data AS (
    SELECT 
        name,
        type,
        MOONS_SOLAR_BODY(name) AS num_moons,
        CASE 
            WHEN MOONS_SOLAR_BODY(name) = 0 THEN 'No moons'
            WHEN MOONS_SOLAR_BODY(name) <= 2 THEN 'Few moons'
            WHEN MOONS_SOLAR_BODY(name) <= 20 THEN 'Several moons'
            ELSE 'Many moons'
        END AS moon_category
    FROM test
    WHERE type != 'Star'
)
SELECT * FROM moon_data
WHERE num_moons > 0;
GO

-- Example 5: Temperature and habitability zone
WITH temperature_zones AS (
    SELECT 
        name,
        type,
        mean_distance_au,
        CASE 
            WHEN mean_distance_au < 0.7 THEN 'Too hot'
            WHEN mean_distance_au <= 1.5 THEN 'Habitable zone'
            WHEN mean_distance_au <= 5 THEN 'Cold zone'
            ELSE 'Frozen zone'
        END AS temp_zone,
        ROUND(MASS_SOLAR_BODY(name) / MASS_EARTH(), 2) AS mass_earths
    FROM test
    WHERE type IN ('Terrestrial', 'Dwarf Planet')
)
SELECT 
    name,
    temp_zone,
    mass_earths
FROM temperature_zones
WHERE temp_zone = 'Habitable zone';
GO

-- Example 6: Giant planets comparison with ORDER BY
WITH giants AS (
    SELECT 
        name,
        type,
        RADIUS_SOLAR_BODY(name) / RADIUS_EARTH() AS radius_earths,
        MASS_SOLAR_BODY(name) / MASS_EARTH() AS mass_earths,
        GRAVITY_SOLAR_BODY(name) / 9.807 AS gravity_earths
    FROM test
    WHERE type IN ('Gas Giant', 'Ice Giant')
)
SELECT 
    name,
    type,
    ROUND(radius_earths, 1) AS radius_ratio,
    ROUND(mass_earths, 0) AS mass_ratio,
    ROUND(gravity_earths, 2) AS gravity_ratio
FROM giants
WHERE mass_earths > 50  -- Filter for the largest planets
ORDER BY mass_ratio DESC;  -- ORDER BY uses the SELECT alias!
GO

-- ============================================================================
-- Key Benefits Demonstrated:
-- 1. Compute complex expressions ONCE in the CTE
-- 2. Filter on computed values in WHERE clause (main problem solved!)
-- 3. Use CASE expressions with computed values
-- 4. Clean, readable queries without repeated function calls
-- 5. Better performance - functions evaluated once per row, not multiple times
--
-- Current Limitations:
-- - ORDER BY must use the SELECT alias, not the CTE column name
--   Wrong: SELECT ROUND(gravity_ms2, 2) AS gravity ... ORDER BY gravity_ms2
--   Right: SELECT ROUND(gravity_ms2, 2) AS gravity ... ORDER BY gravity
--   Or:    SELECT gravity_ms2 ... ORDER BY gravity_ms2
-- ============================================================================