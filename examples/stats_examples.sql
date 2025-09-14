-- #! ../data/AAPL_data.csv

-- Statistical Analysis Examples
-- Demonstrates all statistical functions with Apple stock data and RANGE-generated data

-- ==================================================
-- 1. BASIC DESCRIPTIVE STATISTICS - Apple Stock Closing Prices
-- ==================================================

SELECT
    'AAPL Closing Prices' as analysis,
    COUNT(close) as observations,
    MIN(close) as minimum,
    MAX(close) as maximum,
    AVG(close) as mean,
    MEDIAN(close) as median,
    PERCENTILE(close) as p50_percentile,
    VARIANCE(close) as variance,
    STDDEV(close) as std_deviation
FROM AAPL_data;
GO

-- ==================================================
-- 2. VOLUME ANALYSIS - Trading Volume Statistics
-- ==================================================

SELECT
    'AAPL Volume Analysis' as analysis,
    COUNT(volume) as trading_days,
    MIN(volume) as min_volume,
    MAX(volume) as max_volume,
    AVG(volume) as avg_volume,
    MEDIAN(volume) as median_volume,
    STDDEV(volume) as volume_volatility
FROM AAPL_data;
GO

-- ==================================================
-- 3. PRICE RANGE ANALYSIS - Daily Price Movements
-- ==================================================

WITH daily_ranges AS (
    SELECT
        date,
        (high - low) as daily_range,
        ((high - low) / open) * 100 as range_percentage
    FROM AAPL_data
)
SELECT
    'Price Range Distribution' as analysis,
    COUNT(*) as total_days,
    MIN(daily_range) as min_daily_range,
    MAX(daily_range) as max_daily_range,
    AVG(daily_range) as avg_daily_range,
    MEDIAN(daily_range) as median_range,
    STDDEV(daily_range) as range_volatility,
    AVG(range_percentage) as avg_range_percent
FROM daily_ranges;
GO

-- ==================================================
-- 4. DISTRIBUTION ANALYSIS - Bucket Frequencies (Perfect for Pie Charts!)
-- ==================================================

WITH price_buckets AS (
    SELECT
        close,
        CASE
            WHEN close < 70 THEN 'Under $70'
            WHEN close < 80 THEN '$70-80'
            WHEN close < 90 THEN '$80-90'
            WHEN close < 100 THEN '$90-100'
            WHEN close < 110 THEN '$100-110'
            ELSE 'Over $110'
        END as price_bucket
    FROM AAPL_data
)
SELECT
    price_bucket,
    COUNT(*) as frequency,
    COUNT(*) * 100.0 / (SELECT COUNT(*) FROM AAPL_data) as percentage,
    AVG(close) as avg_price_in_bucket,
    MIN(close) as min_price_in_bucket,
    MAX(close) as max_price_in_bucket
FROM price_buckets
GROUP BY price_bucket
ORDER BY frequency DESC;
GO

-- ==================================================
-- 5. SYNTHETIC DATA ANALYSIS - Using RANGE Function
-- ==================================================

WITH synthetic_data AS (
    SELECT
        value,
        POWER(value, 2) as squared,
        SQRT(value) as square_root,
        value * 1.5 + 10 as transformed
    FROM RANGE(1, 20)
)
SELECT
    'Synthetic Data (1-20)' as analysis,
    COUNT(value) as n,
    MIN(value) as min_val,
    MAX(value) as max_val,
    AVG(value) as mean,
    MEDIAN(value) as median,
    MODE(value) as mode_value,
    STDDEV(value) as std_dev,
    VARIANCE(value) as variance
FROM synthetic_data;
GO

-- ==================================================
-- 6. MODE ANALYSIS - Most Frequent Values
-- ==================================================

WITH rounded_closes AS (
    SELECT
        ROUND(close, 0) as rounded_close
    FROM AAPL_data
)
SELECT
    'Most Frequent Price Levels' as analysis,
    MODE(rounded_close) as most_frequent_price,
    COUNT(*) as total_observations,
    AVG(rounded_close) as avg_rounded_price,
    MEDIAN(rounded_close) as median_rounded_price
FROM rounded_closes;
GO

-- ==================================================
-- 7. VOLATILITY PERIODS - Statistical Classification
-- ==================================================

WITH daily_changes AS (
    SELECT
        date,
        ABS(close - open) as daily_change,
        CASE
            WHEN ABS(close - open) < 1 THEN 'Low Volatility'
            WHEN ABS(close - open) < 3 THEN 'Medium Volatility'
            ELSE 'High Volatility'
        END as volatility_class
    FROM AAPL_data
)
SELECT
    volatility_class,
    COUNT(*) as frequency,
    COUNT(*) * 100.0 / (SELECT COUNT(*) FROM AAPL_data) as percentage,
    AVG(daily_change) as avg_change,
    MIN(daily_change) as min_change,
    MAX(daily_change) as max_change,
    STDDEV(daily_change) as volatility_stddev
FROM daily_changes
GROUP BY volatility_class
ORDER BY frequency DESC;
GO

-- ==================================================
-- 8. COMPREHENSIVE RANGE ANALYSIS - Perfect Statistical Demo
-- ==================================================

WITH range_data AS (
    SELECT
        value,
        value * value as quadratic,
        CASE
            WHEN value % 2 = 0 THEN 'Even'
            ELSE 'Odd'
        END as parity
    FROM RANGE(1, 50)
)
SELECT
    'Range Analysis (1-50)' as dataset,
    COUNT(*) as n,
    MIN(value) as minimum,
    MAX(value) as maximum,
    AVG(value) as arithmetic_mean,
    MEDIAN(value) as median_value,
    MODE(parity) as modal_parity,
    STDDEV(value) as standard_deviation,
    VARIANCE(value) as variance,
    AVG(quadratic) as mean_quadratic,
    MEDIAN(quadratic) as median_quadratic
FROM range_data;
GO
