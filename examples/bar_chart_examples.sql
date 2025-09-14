-- #! ../data/AAPL_data.csv

WITH
    price_buckets AS (
        SELECT
            CASE
        WHEN close < 70 THEN 'Under \$70'
        WHEN close < 80 THEN '\$70-80'
        WHEN close < 90 THEN '\$80-90'
        WHEN close < 100 THEN '\$90-100'
        WHEN close < 110 THEN '\$100-110'
        WHEN close < 120 THEN '\$110-120'
        WHEN close < 130 THEN '\$120-130'
        WHEN close < 140 THEN '\$130-140'
        WHEN close < 150 THEN '\$140-150'
        ELSE 'Over \$150'
    END AS price_range_label,
            CASE
        WHEN close < 70 THEN 1
        WHEN close < 80 THEN 2
        WHEN close < 90 THEN 3
        WHEN close < 100 THEN 4
        WHEN close < 110 THEN 5
        WHEN close < 120 THEN 6
        WHEN close < 130 THEN 7
        WHEN close < 140 THEN 8
        WHEN close < 150 THEN 9
        ELSE 10
    END AS price_range,
            close
        FROM AAPL_data
    )
SELECT
    price_range,
    COUNT('*') AS frequency
FROM price_buckets
GROUP BY price_range
ORDER BY price_range ASC;
GO

WITH
    daily_volume AS (
        SELECT
            DAYOFWEEK(date) AS dow_num,
            DAYNAME(date) AS day_name,
            volume
        FROM AAPL_data
    )
SELECT
    day_name AS trading_day,
    AVG(volume) AS avg_volume
FROM daily_volume
WHERE dow_num >= 1 AND dow_num <= 5
GROUP BY day_name, dow_num
ORDER BY dow_num ASC;
GO

WITH
    monthly_data AS (
        SELECT
            MONTHNAME(date) AS month_name,
            MONTH(date) AS month,
            close
        FROM AAPL_data
    ),
    ordered AS (
        SELECT
            month,
            month_name,
            AVG(close) AS avg_close
        FROM monthly_data
        GROUP BY month, month_name
        ORDER BY month ASC
    )
SELECT month_name, avg_close
FROM ordered;
GO

-- ==================================================
-- 4. VOLATILITY DISTRIBUTION - Daily Price Changes
-- ==================================================
-- Categorize days by volatility (price change percentage)

-- WITH daily_changes AS (
    -- SELECT
        -- ABS((close - open) / open * 100) as pct_change
    -- FROM AAPL_data
-- )
-- SELECT
    -- CASE
        -- WHEN pct_change < 0.5 THEN 'Very Low (<0.5%)'
        -- WHEN pct_change < 1.0 THEN 'Low (0.5-1%)'
        -- WHEN pct_change < 2.0 THEN 'Medium (1-2%)'
        -- WHEN pct_change < 3.0 THEN 'High (2-3%)'
        -- WHEN pct_change < 5.0 THEN 'Very High (3-5%)'
        -- ELSE 'Extreme (>5%)'
    -- END as volatility_level,    -- Label
    -- COUNT(*) as days            -- Value
-- FROM daily_changes
-- GROUP BY volatility_level
-- ORDER BY
    -- CASE volatility_level
        -- WHEN 'Very Low (<0.5%)' THEN 1
        -- WHEN 'Low (0.5-1%)' THEN 2
        -- WHEN 'Medium (1-2%)' THEN 3
        -- WHEN 'High (2-3%)' THEN 4
        -- WHEN 'Very High (3-5%)' THEN 5
        -- ELSE 6
    -- END;
GO

-- ==================================================
-- 5. TOP 10 HIGHEST VOLUME DAYS
-- ==================================================
-- Simple bar chart of specific dates

SELECT
    date as trading_date,       -- Label
    volume                      -- Value
FROM AAPL_data
ORDER BY volume DESC
LIMIT 10;
GO

-- ==================================================
-- 6. PRICE RANGE ANALYSIS - Intraday Range
-- ==================================================
-- Average daily range (high-low) grouped by price levels

WITH range_analysis AS (
    SELECT
        (high - low) as daily_range,
        CASE
            WHEN close < 80 THEN 'Low Price Period'
            WHEN close < 120 THEN 'Medium Price Period'
            ELSE 'High Price Period'
        END as price_period
    FROM AAPL_data
)
SELECT
    price_period,                      -- Label
    AVG(daily_range) as avg_range     -- Value
FROM range_analysis
GROUP BY price_period
ORDER BY avg_range;
GO

-- ==================================================
-- HOW TO USE IN NVIM
-- ==================================================
-- 1. Open this file in nvim
-- 2. Place cursor on a query
-- 3. Run: :SqlCliExecuteAtCursor to see the data
-- 4. Then run: :SqlBarChart <paste the query>
--
-- Or for a quick test with simple data:
-- :SqlBarChart SELECT 'Category A' as name, 100 as value UNION SELECT 'Category B', 150 UNION SELECT 'Category C', 75

-- ==================================================
-- UNDERSTANDING THE OUTPUT
-- ==================================================
-- The bar chart will look like this:
--
-- Category A │████████████░░░░░░░░░░░░ 100
-- Category B │██████████████████░░░░░░ 150
-- Category C │█████████░░░░░░░░░░░░░░░  75
--
-- Where:
-- - Left side: Labels from first column
-- - Middle: Visual bars (█ = filled, ░ = empty)
-- - Right side: Actual values from second column

-- ==================================================
-- CUSTOMIZATION TIPS
-- ==================================================
-- 1. Use ORDER BY to control the order of bars
-- 2. Use LIMIT to show only top N items
-- 3. Round numbers for cleaner display: ROUND(AVG(value), 2)
-- 4. Use meaningful labels that fit well on screen
-- 5. Consider using abbreviations for long labels
