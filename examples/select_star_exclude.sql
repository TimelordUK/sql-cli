-- #! ../data/solar_system.csv

-- =============================================================================
-- SELECT * EXCLUDE - Column Exclusion Syntax
-- =============================================================================
-- DuckDB-style syntax for selecting all columns except specific ones
-- Useful for excluding sensitive, large, or irrelevant columns

-- =============================================================================
-- 1. BASIC USAGE - Exclude Single Column
-- =============================================================================
SELECT '=== Exclude Single Column ===' as demo;
GO

-- Exclude the 'type' column
SELECT * EXCLUDE (type)
FROM solar_system
LIMIT 3;
GO

-- =============================================================================
-- 2. EXCLUDE MULTIPLE COLUMNS
-- =============================================================================
SELECT '=== Exclude Multiple Columns ===' as demo;
GO

-- Exclude multiple columns for cleaner output
SELECT * EXCLUDE (eccentricity, inclination_deg, axial_tilt_deg)
FROM solar_system
WHERE name IN ('Earth', 'Mars', 'Jupiter');
GO

-- =============================================================================
-- 3. WITH WHERE CLAUSE
-- =============================================================================
SELECT '=== With Filtering ===' as demo;
GO

-- Exclude columns and filter rows
SELECT * EXCLUDE (eccentricity, inclination_deg)
FROM solar_system
WHERE mean_distance_au > 5
ORDER BY mean_distance_au DESC;
GO

-- =============================================================================
-- 4. EXCLUDE WITH AGGREGATION
-- =============================================================================
SELECT '=== With GROUP BY ===' as demo;
GO

-- Group by type and show averages
SELECT
    type,
    COUNT(*) as count,
    ROUND(AVG(mean_distance_au), 2) as avg_distance,
    ROUND(AVG(axial_tilt_deg), 1) as avg_tilt
FROM solar_system
GROUP BY type
ORDER BY avg_distance;
GO

-- =============================================================================
-- 5. PRACTICAL USE CASES
-- =============================================================================

-- Use Case 1: Security - Exclude sensitive columns
SELECT '=== Security: Hiding Sensitive Data ===' as demo;
GO

-- In a real application, you might have user tables with passwords, SSNs, etc.
-- SELECT * EXCLUDE (password, ssn, credit_card) FROM users;
-- For demo purposes using solar_system:
SELECT * EXCLUDE (axial_tilt_deg, albedo)  -- Exclude less important columns
FROM solar_system
WHERE type = 'Planet'
LIMIT 3;
GO

-- Use Case 2: Performance - Exclude large columns
SELECT '=== Performance: Skip Large Columns ===' as demo;
GO

-- In real apps, you might exclude BLOB, TEXT, or JSON columns
-- SELECT * EXCLUDE (profile_image, document_text, metadata_json) FROM documents;
-- For demo:
SELECT * EXCLUDE (atmosphere)  -- Skip text column
FROM solar_system
WHERE name IN ('Mercury', 'Venus', 'Earth');
GO

-- Use Case 3: Readability - Exclude irrelevant columns
SELECT '=== Readability: Clean Output ===' as demo;
GO

-- When presenting data, exclude technical IDs, timestamps, etc.
-- SELECT * EXCLUDE (id, created_at, updated_at, deleted_at) FROM products;
-- For demo:
SELECT * EXCLUDE (position, type, albedo)  -- Focus on core attributes
FROM solar_system
WHERE mean_distance_au > 0.5 AND mean_distance_au < 2
ORDER BY mean_distance_au;
GO

-- =============================================================================
-- 6. COMPARISON: WITH vs WITHOUT EXCLUDE
-- =============================================================================
SELECT '=== Without EXCLUDE (verbose) ===' as demo;
GO

-- Traditional way: List every column explicitly
SELECT name, mean_distance_au, eccentricity
FROM solar_system
WHERE type = 'Planet'
LIMIT 2;
GO

SELECT '=== With EXCLUDE (cleaner) ===' as demo;
GO

-- Modern way: Exclude unwanted columns
SELECT * EXCLUDE (type, position, inclination_deg, axial_tilt_deg, albedo, atmosphere)
FROM solar_system
WHERE type = 'Planet'
LIMIT 2;
GO

-- =============================================================================
-- SUMMARY OF BENEFITS
-- =============================================================================
-- ✓ Less typing than listing all columns
-- ✓ More maintainable - new columns auto-included
-- ✓ Clearer intent - "I want everything except X"
-- ✓ Safer - helps avoid accidentally exposing sensitive data
-- ✓ Works with WHERE, ORDER BY, LIMIT, GROUP BY, etc.
-- ✓ PostgreSQL/DuckDB compatibility
