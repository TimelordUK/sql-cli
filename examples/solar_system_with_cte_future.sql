-- [TEST:SKIP] aspirational design sketch, not bound to a data file
-- Solar System Queries with CTEs (Future Capability)
-- This file shows what will be possible once CTEs are implemented
-- It demonstrates how CTEs solve current limitations with window functions and ORDER BY

-- ============================================================================
-- Example 1: Complete planet analysis with all computed values
-- ============================================================================
WITH planet_properties AS (
    -- First, compute all the function values once
    SELECT 
        name,
        type,
        position,
        mean_distance_au,
        MASS_SOLAR_BODY(name) AS mass_kg,
        RADIUS_SOLAR_BODY(name) AS radius_m,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2,
        DISTANCE_SOLAR_BODY(name) AS distance_m,
        ORBITAL_PERIOD_SOLAR_BODY(name) AS period_days,
        DENSITY_SOLAR_BODY(name) AS density_kgm3,
        ESCAPE_VELOCITY_SOLAR_BODY(name) AS escape_vel_ms,
        ROTATION_PERIOD_SOLAR_BODY(name) AS rotation_hrs,
        MOONS_SOLAR_BODY(name) AS num_moons
    FROM test
    WHERE type != 'Star'
)
SELECT 
    name,
    type,
    ROUND(gravity_ms2, 2) AS gravity,
    -- Now we can use LAG on computed values!
    LAG(gravity_ms2, 1) OVER (ORDER BY gravity_ms2) AS prev_gravity,
    ROUND(gravity_ms2 - LAG(gravity_ms2, 1) OVER (ORDER BY gravity_ms2), 2) AS gravity_increase,
    -- Ranking by computed values
    RANK() OVER (ORDER BY gravity_ms2 DESC) AS gravity_rank,
    DENSE_RANK() OVER (ORDER BY num_moons DESC) AS moons_rank
FROM planet_properties
ORDER BY gravity_ms2 DESC;  -- Can ORDER BY computed column directly!
GO

-- ============================================================================
-- Example 2: Distance gap analysis with window functions
-- ============================================================================
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
    ROUND(period_years, 2) AS period_yr,
    -- Calculate gaps between consecutive planets
    LAG(name, 1) OVER (ORDER BY distance_au) AS inner_neighbor,
    LEAD(name, 1) OVER (ORDER BY distance_au) AS outer_neighbor,
    ROUND(distance_au - LAG(distance_au, 1) OVER (ORDER BY distance_au), 2) AS gap_from_inner,
    ROUND(LEAD(distance_au, 1) OVER (ORDER BY distance_au) - distance_au, 2) AS gap_to_outer,
    -- Running calculations
    ROUND(SUM(distance_au) OVER (ORDER BY distance_au ROWS UNBOUNDED PRECEDING), 2) AS cumulative_au
FROM orbital_data
ORDER BY distance_au;
GO

-- ============================================================================
-- Example 3: Comparative analysis with ratios
-- ============================================================================
WITH earth_baseline AS (
    -- Get Earth's values as a baseline
    SELECT 
        MASS_SOLAR_BODY('Earth') AS earth_mass,
        RADIUS_SOLAR_BODY('Earth') AS earth_radius,
        GRAVITY_SOLAR_BODY('Earth') AS earth_gravity,
        DENSITY_SOLAR_BODY('Earth') AS earth_density
),
planet_comparisons AS (
    SELECT 
        name,
        type,
        MASS_SOLAR_BODY(name) / earth_mass AS mass_earths,
        RADIUS_SOLAR_BODY(name) / earth_radius AS radius_earths,
        GRAVITY_SOLAR_BODY(name) / earth_gravity AS gravity_earths,
        DENSITY_SOLAR_BODY(name) / earth_density AS density_earths
    FROM test, earth_baseline
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant', 'Dwarf Planet')
)
SELECT 
    name,
    type,
    ROUND(mass_earths, 2) AS mass_ratio,
    ROUND(radius_earths, 2) AS radius_ratio,
    ROUND(gravity_earths, 2) AS gravity_ratio,
    ROUND(density_earths, 2) AS density_ratio,
    -- Percentile rankings
    PERCENT_RANK() OVER (ORDER BY mass_earths) AS mass_percentile,
    PERCENT_RANK() OVER (ORDER BY gravity_earths) AS gravity_percentile
FROM planet_comparisons
ORDER BY mass_earths DESC;
GO

-- ============================================================================
-- Example 4: Nested CTEs for complex analysis
-- ============================================================================
WITH raw_data AS (
    -- First level: get raw computed values
    SELECT 
        name,
        type,
        position,
        GRAVITY_SOLAR_BODY(name) AS gravity,
        ESCAPE_VELOCITY_SOLAR_BODY(name) AS escape_vel,
        MOONS_SOLAR_BODY(name) AS moons
    FROM test
    WHERE name != 'Sun'
),
categorized AS (
    -- Second level: categorize based on computed values
    SELECT 
        name,
        type,
        gravity,
        escape_vel,
        moons,
        CASE 
            WHEN gravity < 2 THEN 'Micro-gravity'
            WHEN gravity < 5 THEN 'Low-gravity'
            WHEN gravity < 12 THEN 'Earth-like'
            ELSE 'High-gravity'
        END AS gravity_category,
        CASE
            WHEN escape_vel < 5000 THEN 'Easy escape'
            WHEN escape_vel < 15000 THEN 'Moderate escape'
            ELSE 'Hard escape'
        END AS escape_category
    FROM raw_data
),
aggregated AS (
    -- Third level: aggregate by categories
    SELECT 
        gravity_category,
        COUNT(*) AS body_count,
        AVG(moons) AS avg_moons,
        MIN(gravity) AS min_gravity,
        MAX(gravity) AS max_gravity
    FROM categorized
    GROUP BY gravity_category
)
-- Final query uses all the CTEs
SELECT 
    c.name,
    c.type,
    ROUND(c.gravity, 2) AS gravity_ms2,
    c.gravity_category,
    a.body_count AS bodies_in_category,
    ROUND(a.avg_moons, 1) AS category_avg_moons,
    -- Window functions on the categorized data
    RANK() OVER (PARTITION BY c.gravity_category ORDER BY c.moons DESC) AS moons_rank_in_category
FROM categorized c
JOIN aggregated a ON c.gravity_category = a.gravity_category
ORDER BY c.gravity DESC;
GO

-- ============================================================================
-- Example 5: Recursive CTE for orbital resonances (if supported)
-- ============================================================================
WITH RECURSIVE orbital_chain AS (
    -- Base case: start with Mercury
    SELECT 
        name,
        position,
        ORBITAL_PERIOD_SOLAR_BODY(name) AS period,
        1 AS level
    FROM test
    WHERE name = 'Mercury'
    
    UNION ALL
    
    -- Recursive case: find next planet
    SELECT 
        t.name,
        t.position,
        ORBITAL_PERIOD_SOLAR_BODY(t.name) AS period,
        oc.level + 1
    FROM test t
    JOIN orbital_chain oc ON t.position = oc.position + 1
    WHERE t.type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
)
SELECT 
    name,
    ROUND(period, 1) AS period_days,
    level,
    ROUND(period / LAG(period, 1) OVER (ORDER BY level), 2) AS period_ratio
FROM orbital_chain
ORDER BY level;
GO

-- ============================================================================
-- Example 6: Multiple CTEs for Kepler's Law verification
-- ============================================================================
WITH planetary_data AS (
    SELECT 
        name,
        DISTANCE_SOLAR_BODY(name) / AU() AS semi_major_axis_au,
        ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256 AS period_years
    FROM test
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
),
kepler_calc AS (
    SELECT 
        name,
        semi_major_axis_au AS a,
        period_years AS P,
        POW(period_years, 2) AS P_squared,
        POW(semi_major_axis_au, 3) AS a_cubed,
        POW(period_years, 2) / POW(semi_major_axis_au, 3) AS kepler_constant
    FROM planetary_data
),
kepler_stats AS (
    SELECT 
        AVG(kepler_constant) AS avg_constant,
        STDDEV(kepler_constant) AS stddev_constant,
        MIN(kepler_constant) AS min_constant,
        MAX(kepler_constant) AS max_constant
    FROM kepler_calc
)
SELECT 
    k.name,
    ROUND(k.a, 2) AS distance_au,
    ROUND(k.P, 2) AS period_yr,
    ROUND(k.kepler_constant, 4) AS k_value,
    ROUND(ABS(k.kepler_constant - s.avg_constant) / s.stddev_constant, 2) AS z_score,
    CASE 
        WHEN ABS(k.kepler_constant - s.avg_constant) < s.stddev_constant THEN 'Normal'
        ELSE 'Outlier'
    END AS kepler_status
FROM kepler_calc k, kepler_stats s
ORDER BY k.a;
GO

-- ============================================================================
-- Notes on what CTEs enable:
-- ============================================================================
-- 1. Compute function values once, reuse everywhere
-- 2. Use computed values in window functions (LAG, LEAD, RANK, etc.)
-- 3. ORDER BY computed values directly
-- 4. JOIN on computed values
-- 5. GROUP BY computed values
-- 6. Create complex multi-level transformations
-- 7. Recursive calculations (if RECURSIVE CTEs are supported)
-- 8. Much cleaner, more readable queries
-- 9. Better performance (functions called once, not repeatedly)
-- 10. Enable filtering on window function results

-- Current workarounds without CTEs:
-- - Must use column aliases for ORDER BY
-- - Cannot use functions in LAG/LEAD
-- - Cannot filter on window function results
-- - Must repeat complex expressions
-- - Limited ability to build complex analytical queries