-- #! ../data/sales_data.csv

-- Demonstration of temporary tables in sql-cli
-- Temporary tables persist across GO-separated batches within a script
-- They are automatically cleaned up when the script completes

-- =====================================================
-- Example: Store filtered data and query it multiple times
-- =====================================================

-- Step 1: Store high-value sales in a temporary table
SELECT
    month,
    region,
    product,
    sales_amount,
    salesperson
INTO #high_value_sales
FROM sales_data
WHERE sales_amount > 15000;
GO

-- Step 2: Query the temporary table to get regional summaries
-- Note: No need to re-filter or reload the CSV!
SELECT
    region,
    COUNT(*) as sale_count,
    SUM(sales_amount) as total_amount,
    AVG(sales_amount) as avg_amount
FROM #high_value_sales
GROUP BY region
ORDER BY total_amount DESC;
GO

-- Step 3: Query the same temp table for product analysis
SELECT
    product,
    COUNT(*) as sale_count,
    SUM(sales_amount) as total_amount
FROM #high_value_sales
GROUP BY product
ORDER BY total_amount DESC;
GO

-- Step 4: Get detailed breakdown by region and product
SELECT
    region,
    product,
    SUM(sales_amount) as total_amount
FROM #high_value_sales
GROUP BY region, product
ORDER BY region, total_amount DESC;
GO

-- ============================================================================
-- TEMPLATE INJECTION FOR WEB CTEs (Phase 2A)
-- ============================================================================
-- Temp tables can now be injected into WEB CTE requests using template syntax!
--
-- TEMPLATE SYNTAX:
--   ${#table}              → Entire table as JSON array
--   ${#table.column}       → Column values as JSON array
--   ${#table[0]}           → First row as JSON object
--   ${#table[0].column}    → Single cell value
--
-- EXAMPLE 1: Single value injection
-- WITH WEB api_call AS (
--     URL 'https://api.example.com/region/${#high_value_sales[0].region}'
--     METHOD GET
--     FORMAT JSON
-- )
-- SELECT * FROM api_call;
--
-- This would request: https://api.example.com/region/West
--
-- EXAMPLE 2: JSON body injection
-- WITH WEB forecast AS (
--     URL 'https://api.example.com/forecast'
--     METHOD POST
--     BODY '{"region": "${#high_value_sales[0].region}", "amount": ${#high_value_sales[0].sales_amount}}'
--     FORMAT JSON
-- )
-- SELECT * FROM forecast;
--
-- This sends: {"region": "West", "amount": 22000}
--
-- REAL-WORLD USE CASE: Multi-System Data Integration
-- 1. Parse FIX logs → SELECT DISTINCT instrument INTO #instruments FROM fix_logs
-- 2. Query trades DB → WEB CTE with ${#instruments.instrument} in URL
-- 3. Get security master → WEB CTE with ${#trades.isin} for details
-- 4. Submit to risk system → WEB CTE with ${#positions} in POST body
-- All in one script, dynamically building queries based on previous results!
-- ============================================================================
