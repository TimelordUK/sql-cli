-- ============================================================================
-- Window Functions with CTEs - Top N per Group Patterns
-- Shows how to use window functions in CTEs to find top records per category
-- 
-- POWERFUL FEATURE: CTEs can reference previous CTEs in the same query!
-- This allows building complex analytical pipelines step by step.
-- ============================================================================
-- Run: ./target/release/sql-cli data/solar_system.csv -f examples/cte_window_functions.sql -o csv
-- ============================================================================

-- Pattern 1: Top 1 per category (highest gravity per type)
WITH gravity_data AS (
    -- Step 1: Compute the gravity values
    SELECT 
        name,
        type,
        GRAVITY_SOLAR_BODY(name) AS gravity
    FROM test
    WHERE type != 'Star'
),
ranked_by_gravity AS (
    -- Step 2: Rank within each type by gravity
    SELECT 
        name,
        type,
        gravity,
        ROW_NUMBER() OVER (PARTITION BY type ORDER BY gravity DESC) AS rank_in_type
    FROM gravity_data
)
-- Step 3: Filter to get only the top gravity body per type
SELECT 
    name,
    type,
    ROUND(gravity, 2) AS max_gravity_ms2,
    rank_in_type
FROM ranked_by_gravity
WHERE rank_in_type = 1
ORDER BY max_gravity_ms2 DESC;
GO

-- Pattern 2: Top 2 largest bodies per type (by mass) - CTE chaining!
WITH mass_data AS (
    -- First CTE: Calculate mass ratios
    SELECT 
        name,
        type,
        MASS_SOLAR_BODY(name) / MASS_EARTH() AS mass_earths
    FROM test
    WHERE type != 'Star' AND type != 'Moon'
),
ranked_by_mass AS (
    -- Second CTE: References first CTE! Adds ranking
    SELECT 
        name,
        type,
        mass_earths,
        ROW_NUMBER() OVER (PARTITION BY type ORDER BY mass_earths DESC) AS mass_rank
    FROM mass_data  -- References the previous CTE!
)
-- Main query: Filter on the rank from second CTE
SELECT 
    name,
    type,
    ROUND(mass_earths, 2) AS mass_earth_ratio,
    mass_rank
FROM ranked_by_mass
WHERE mass_rank <= 2  -- Get top 2 per type
ORDER BY type, mass_rank;
GO

-- Pattern 3: Closest and farthest body of each type
WITH distance_data AS (
    SELECT 
        name,
        type,
        mean_distance_au
    FROM test
    WHERE type != 'Star' AND type != 'Moon'
),
distance_extremes AS (
    SELECT 
        name,
        type,
        mean_distance_au,
        ROW_NUMBER() OVER (PARTITION BY type ORDER BY mean_distance_au ASC) AS closest_rank,
        ROW_NUMBER() OVER (PARTITION BY type ORDER BY mean_distance_au DESC) AS farthest_rank
    FROM distance_data
)
SELECT 
    name,
    type,
    mean_distance_au AS distance_au,
    CASE 
        WHEN closest_rank = 1 THEN 'Closest'
        WHEN farthest_rank = 1 THEN 'Farthest'
    END AS position_type
FROM distance_extremes
WHERE closest_rank = 1 OR farthest_rank = 1
ORDER BY type, distance_au;
GO

-- Pattern 4: Percentile ranking within type
WITH density_data AS (
    SELECT 
        name,
        type,
        DENSITY_SOLAR_BODY(name) AS density
    FROM test
    WHERE name != 'Sun'
),
density_percentiles AS (
    SELECT 
        name,
        type,
        density,
        PERCENT_RANK() OVER (PARTITION BY type ORDER BY density) AS density_percentile,
        RANK() OVER (PARTITION BY type ORDER BY density DESC) AS density_rank
    FROM density_data
)
SELECT 
    name,
    type,
    ROUND(density, 0) AS density_kgm3,
    ROUND(density_percentile * 100, 1) AS percentile,
    density_rank
FROM density_percentiles
WHERE density_percentile >= 0.5  -- Top 50% densest in each category
ORDER BY type, density_rank;
GO

-- Pattern 5: Running totals and cumulative percentages
WITH moon_counts AS (
    SELECT 
        name,
        type,
        MOONS_SOLAR_BODY(name) AS num_moons
    FROM test
    WHERE type IN ('Gas Giant', 'Ice Giant')
),
cumulative_moons AS (
    SELECT 
        name,
        type,
        num_moons,
        SUM(num_moons) OVER (ORDER BY num_moons DESC) AS running_total,
        SUM(num_moons) OVER () AS grand_total,
        ROW_NUMBER() OVER (ORDER BY num_moons DESC) AS rank
    FROM moon_counts
)
SELECT 
    name,
    type,
    num_moons,
    running_total,
    ROUND(running_total * 100.0 / grand_total, 1) AS cumulative_pct
FROM cumulative_moons
ORDER BY rank;
GO

-- Pattern 6: Gap analysis - difference from leader
WITH orbital_periods AS (
    SELECT 
        name,
        type,
        ORBITAL_PERIOD_SOLAR_BODY(name) AS period_days
    FROM test
    WHERE type = 'Terrestrial'
),
period_analysis AS (
    SELECT 
        name,
        period_days,
        FIRST_VALUE(period_days) OVER (ORDER BY period_days) AS shortest_period,
        LAST_VALUE(period_days) OVER (ORDER BY period_days ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS longest_period,
        LAG(period_days, 1) OVER (ORDER BY period_days) AS prev_period,
        LEAD(period_days, 1) OVER (ORDER BY period_days) AS next_period
    FROM orbital_periods
)
SELECT 
    name,
    ROUND(period_days, 1) AS period,
    ROUND(period_days - shortest_period, 1) AS days_from_shortest,
    ROUND(longest_period - period_days, 1) AS days_to_longest,
    ROUND(period_days - prev_period, 1) AS gap_from_prev,
    ROUND(next_period - period_days, 1) AS gap_to_next
FROM period_analysis
ORDER BY period_days;
GO

-- ============================================================================
-- Key Patterns Demonstrated:
-- 1. Compute values in first CTE, apply window functions in second CTE
-- 2. Filter on window function results (rank = 1, percentile > 0.5, etc.)
-- 3. Multiple window functions in same query (ROW_NUMBER, RANK, PERCENT_RANK)
-- 4. Different PARTITION BY and ORDER BY combinations
-- 5. Running totals and cumulative calculations
-- 6. LAG/LEAD for gap analysis between consecutive rows
--
-- This solves the core problem: Can't filter directly on window functions!
-- WITH cte AS (SELECT ..., ROW_NUMBER() OVER (...) AS rn FROM ...)
-- SELECT * FROM cte WHERE rn = 1  -- Now we can filter!
-- ============================================================================