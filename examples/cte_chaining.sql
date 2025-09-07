-- ============================================================================
-- CTE Chaining - Building Complex Analytical Pipelines
-- Demonstrates how CTEs can reference previous CTEs in the same query
-- ============================================================================
-- Run: ./target/release/sql-cli data/solar_system.csv -f examples/cte_chaining.sql -o csv
-- ============================================================================

-- Example 1: Simple CTE chain - each references the previous
WITH 
step1 AS (
    -- Calculate gravity for all bodies
    SELECT 
        name,
        type,
        GRAVITY_SOLAR_BODY(name) AS gravity
    FROM test
    WHERE type != 'Star'
),
step2 AS (
    -- Filter to high-gravity bodies (references step1)
    SELECT *
    FROM step1
    WHERE gravity > 5
),
step3 AS (
    -- Add Earth ratio calculation (references step2)
    SELECT 
        name,
        type,
        gravity,
        gravity / 9.807 AS earth_ratio
    FROM step2
)
-- Final output with formatting
SELECT 
    name,
    type,
    ROUND(gravity, 2) AS gravity_ms2,
    ROUND(earth_ratio, 2) AS times_earth_gravity
FROM step3
ORDER BY gravity DESC;
GO

-- Example 2: Window functions on computed values (the killer feature!)
WITH 
computed_values AS (
    -- Step 1: Calculate complex expressions once
    SELECT 
        name,
        type,
        MASS_SOLAR_BODY(name) AS mass,
        RADIUS_SOLAR_BODY(name) AS radius,
        GRAVITY_SOLAR_BODY(name) AS gravity
    FROM test
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
),
with_rankings AS (
    -- Step 2: Apply window functions to computed values
    SELECT 
        name,
        type,
        mass,
        radius,
        gravity,
        ROW_NUMBER() OVER (PARTITION BY type ORDER BY gravity DESC) AS gravity_rank,
        RANK() OVER (PARTITION BY type ORDER BY mass DESC) AS mass_rank
    FROM computed_values
),
top_performers AS (
    -- Step 3: Filter to top 2 by gravity in each category
    SELECT *
    FROM with_rankings
    WHERE gravity_rank <= 2
)
-- Final query with formatting
SELECT 
    name,
    type,
    ROUND(gravity, 2) AS gravity_ms2,
    gravity_rank,
    mass_rank
FROM top_performers
ORDER BY type, gravity_rank;
GO

-- Example 3: Aggregations on filtered data
WITH 
raw_data AS (
    -- Get all planetary data
    SELECT 
        name,
        type,
        MOONS_SOLAR_BODY(name) AS moon_count
    FROM test
    WHERE type != 'Star' AND type != 'Moon'
),
categorized AS (
    -- Add categories based on moon count
    SELECT 
        name,
        type,
        moon_count,
        CASE 
            WHEN moon_count = 0 THEN 'No moons'
            WHEN moon_count <= 5 THEN 'Few moons'
            ELSE 'Many moons'
        END AS moon_category
    FROM raw_data
),
summary_stats AS (
    -- Calculate statistics per category
    SELECT 
        moon_category,
        COUNT(*) AS body_count,
        SUM(moon_count) AS total_moons,
        AVG(moon_count) AS avg_moons,
        MAX(moon_count) AS max_moons
    FROM categorized
    GROUP BY moon_category
)
-- Show the summary
SELECT 
    moon_category,
    body_count,
    total_moons,
    ROUND(avg_moons, 1) AS avg_moons_per_body,
    max_moons
FROM summary_stats
ORDER BY total_moons DESC;
GO

-- Example 4: Complex analytical pipeline
WITH 
-- Stage 1: Calculate all metrics
metrics AS (
    SELECT 
        name,
        type,
        GRAVITY_SOLAR_BODY(name) AS gravity,
        MASS_SOLAR_BODY(name) / MASS_EARTH() AS mass_earths,
        RADIUS_SOLAR_BODY(name) / RADIUS_EARTH() AS radius_earths,
        DENSITY_SOLAR_BODY(name) AS density
    FROM test
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
),
-- Stage 2: Add percentile rankings
with_percentiles AS (
    SELECT 
        *,
        PERCENT_RANK() OVER (ORDER BY gravity) AS gravity_percentile,
        PERCENT_RANK() OVER (ORDER BY mass_earths) AS mass_percentile,
        PERCENT_RANK() OVER (ORDER BY density) AS density_percentile
    FROM metrics
),
-- Stage 3: Categorize based on percentiles
categorized AS (
    SELECT 
        name,
        type,
        gravity,
        mass_earths,
        density,
        CASE 
            WHEN gravity_percentile >= 0.75 THEN 'High gravity'
            WHEN gravity_percentile >= 0.25 THEN 'Medium gravity'
            ELSE 'Low gravity'
        END AS gravity_category,
        CASE
            WHEN mass_percentile >= 0.75 THEN 'Massive'
            WHEN mass_percentile >= 0.25 THEN 'Medium mass'
            ELSE 'Light'
        END AS mass_category
    FROM with_percentiles
),
-- Stage 4: Filter to interesting bodies
interesting AS (
    SELECT *
    FROM categorized
    WHERE gravity_category = 'High gravity' OR mass_category = 'Massive'
)
-- Final output
SELECT 
    name,
    type,
    ROUND(gravity, 2) AS gravity_ms2,
    ROUND(mass_earths, 1) AS mass_earth_ratio,
    gravity_category,
    mass_category
FROM interesting
ORDER BY gravity DESC;
GO

-- ============================================================================
-- Key Benefits of CTE Chaining:
-- 1. Break complex logic into readable steps
-- 2. Each CTE can reference ALL previous CTEs
-- 3. Compute expensive functions only once
-- 4. Apply window functions to computed values
-- 5. Filter on window function results
-- 6. Build sophisticated analytical pipelines
-- 7. Debug by selecting from intermediate CTEs
--
-- This solves major SQL limitations:
-- - Can't use window functions in WHERE
-- - Can't reference column aliases in same SELECT
-- - Can't filter on computed expressions directly
-- ============================================================================