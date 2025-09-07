-- ============================================================================
-- CTE ORDER BY Patterns and Best Practices
-- Shows different ways to use ORDER BY with CTEs
-- ============================================================================
-- Run: ./target/release/sql-cli data/solar_system.csv -f examples/cte_order_by_patterns.sql -o csv
-- ============================================================================

-- Pattern 1: ORDER BY using SELECT alias (recommended)
WITH planet_data AS (
    SELECT 
        name,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2
    FROM test
    WHERE type != 'Star'
)
SELECT 
    name,
    ROUND(gravity_ms2, 2) AS gravity  -- Transform and alias
FROM planet_data 
ORDER BY gravity DESC  -- ORDER BY the SELECT alias
LIMIT 5;
GO

-- Pattern 2: Re-alias CTE column for ORDER BY
WITH planet_data AS (
    SELECT 
        name,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2
    FROM test
    WHERE type != 'Star'
)
SELECT 
    name,
    gravity_ms2 AS raw_gravity,  -- Re-alias without transformation
    ROUND(gravity_ms2, 2) AS rounded_gravity  -- Transform with different alias
FROM planet_data 
ORDER BY raw_gravity DESC  -- Can ORDER BY either alias!
LIMIT 5;
GO

-- Pattern 3: Use CTE column directly (when no transformation needed)
WITH planet_data AS (
    SELECT 
        name,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2
    FROM test
    WHERE type != 'Star'
)
SELECT 
    name,
    gravity_ms2  -- Use CTE column as-is
FROM planet_data 
ORDER BY gravity_ms2 DESC  -- ORDER BY the actual column
LIMIT 5;
GO

-- Pattern 4: Multiple computed columns with ORDER BY
WITH calculations AS (
    SELECT 
        name,
        MASS_SOLAR_BODY(name) AS mass_kg,
        RADIUS_SOLAR_BODY(name) AS radius_m,
        GRAVITY_SOLAR_BODY(name) AS gravity_ms2
    FROM test
    WHERE type IN ('Terrestrial', 'Gas Giant', 'Ice Giant')
)
SELECT 
    name,
    ROUND(mass_kg / MASS_EARTH(), 2) AS mass_earths,
    ROUND(radius_m / RADIUS_EARTH(), 2) AS radius_earths,
    ROUND(gravity_ms2, 2) AS gravity
FROM calculations
ORDER BY mass_earths DESC;  -- ORDER BY computed alias
GO

-- Pattern 5: Multi-column ORDER BY with aliases
WITH categorized AS (
    SELECT 
        name,
        type,
        GRAVITY_SOLAR_BODY(name) AS gravity_raw,
        CASE 
            WHEN GRAVITY_SOLAR_BODY(name) < 5 THEN 1  -- Low gravity
            WHEN GRAVITY_SOLAR_BODY(name) < 15 THEN 2  -- Medium gravity
            ELSE 3  -- High gravity
        END AS gravity_priority
    FROM test
    WHERE name != 'Sun'
)
SELECT 
    name,
    type,
    ROUND(gravity_raw, 2) AS gravity,
    gravity_priority AS priority,
    CASE 
        WHEN gravity_priority = 1 THEN 'Low'
        WHEN gravity_priority = 2 THEN 'Medium'
        ELSE 'High'
    END AS category
FROM categorized
ORDER BY priority DESC, gravity DESC;  -- Multi-column ORDER BY works!
GO

-- ============================================================================
-- Key Takeaways:
-- 1. Always ORDER BY the SELECT alias when you transform columns
-- 2. You can re-alias CTE columns without transformation for ORDER BY
-- 3. If you need the original CTE column for ORDER BY, include it in SELECT
-- 4. Multiple ORDER BY columns work perfectly with aliases
--
-- Note: Currently only supports CASE WHEN syntax, not CASE column WHEN value
--   Supported:     CASE WHEN col = 1 THEN 'A' ELSE 'B' END
--   Not supported: CASE col WHEN 1 THEN 'A' ELSE 'B' END
-- ============================================================================