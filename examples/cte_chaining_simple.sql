-- #! ../data/solar_system.csv
WITH
    stage1 AS (
        SELECT
            name,
            type,
            GRAVITY_SOLAR_BODY(name) AS gravity
        FROM test
        WHERE type != 'Star'
    ),
    stage2 AS (
        SELECT
            name,
            type,
            gravity,
            MASS_SOLAR_BODY(name) / MASS_EARTH() AS mass_earths
        FROM stage1
        WHERE gravity > 5
    ),
    stage3 AS (
        SELECT
            name,
            type,
            gravity,
            mass_earths,
            DENSITY_SOLAR_BODY(name) AS density
        FROM stage2
    )
SELECT
    name,
    type,
    ROUND(gravity, 2) AS gravity_ms2,
    ROUND(mass_earths, 1) AS mass_ratio,
    ROUND(density, 0) AS density_kgm3
FROM stage3
ORDER BY gravity_ms2 DESC;
GO

-- Example 2: Top body per type using ROW_NUMBER
WITH
    computed AS (
        SELECT
            name,
            type,
            GRAVITY_SOLAR_BODY(name) AS gravity
        FROM test
        WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant', 'Dwarf Planet')
    ),
    ranked AS (
        SELECT
            name,
            type,
            gravity,
            ROW_NUMBER() OVER (PARTITION BY type ORDER BY gravity DESC) AS rank_in_type
        FROM computed
    )
SELECT
    name,
    type,
    ROUND(gravity, 2) AS max_gravity_ms2
FROM ranked
WHERE rank_in_type = 1
ORDER BY max_gravity_ms2 DESC;
GO

-- Example 3: Aggregations on categorized data
WITH
    raw_data AS (
        SELECT
            name,
            type,
            MOONS_SOLAR_BODY(name) AS moon_count
        FROM test
        WHERE type != 'Star' AND type != 'Moon'
    ),
    categorized AS (
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
    )
SELECT
    moon_category,
    COUNT('*') AS body_count,
    SUM(moon_count) AS total_moons,
    ROUND(AVG(moon_count), 1) AS avg_moons
FROM categorized
GROUP BY moon_category
ORDER BY total_moons DESC;
GO

-- ============================================================================
-- Key Points:
-- 1. Each CTE can reference ALL previous CTEs in the chain
-- 2. Compute expensive functions once, reuse in later stages
-- 3. ROW_NUMBER() works for ranking (RANK and PERCENT_RANK not yet supported)
-- 4. ORDER BY must use the SELECT aliases, not CTE column names
-- ============================================================================
