-- #! ../data/AAPL_data.csv
-- Stock Analysis Examples using AAPL data
-- This file demonstrates financial calculations and analytics

-- ============================================
-- Example 1: Basic LAG function to get previous values
-- ============================================
SELECT
    date,
    close,
    LAG(close) OVER (ORDER BY date) as prev_close,
    LAG(close, 2) OVER (ORDER BY date) as prev_close_2days,
    LAG(close, 5) OVER (ORDER BY date) as prev_close_week
FROM AAPL_data
ORDER BY date
LIMIT 10;
GO

-- ============================================
-- Example 2: Calculate simple returns manually
-- Returns = (price[t] - price[t-1]) / price[t-1]
-- ============================================
SELECT
    date,
    close,
    LAG(close) OVER (ORDER BY date) as prev_close,
    (close - LAG(close) OVER (ORDER BY date)) / LAG(close) OVER (ORDER BY date) as manual_returns
FROM AAPL_data
ORDER BY date
LIMIT 10;
GO

-- ============================================
-- Example 3: Using RETURNS function
-- ============================================
SELECT
    date,
    close,
    RETURNS(close, LAG(close) OVER (ORDER BY date)) as returns
FROM AAPL_data
ORDER BY date
LIMIT 10;
GO

-- ============================================
-- Example 4: Calculate logarithmic returns
-- Log returns are better for multi-period analysis
-- ============================================
SELECT
    date,
    close,
    LOG_RETURNS(close, LAG(close) OVER (ORDER BY date)) as log_returns
FROM AAPL_data
ORDER BY date
LIMIT 10;
GO

-- ============================================
-- Example 5: Calculate 5-day and 20-day moving averages
-- ============================================
SELECT
    date,
    close,
    AVG(close) OVER (ORDER BY date ROWS 4 PRECEDING) as ma_5day,
    AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING) as ma_20day
FROM AAPL_data
ORDER BY date
LIMIT 30;
GO

-- ============================================
-- Example 6: Calculate rolling volatility (20-day)
-- This requires calculating returns first, then standard deviation
-- ============================================
WITH returns_data AS (
    SELECT
        date,
        close,
        RETURNS(close, LAG(close) OVER (ORDER BY date)) as returns
    FROM AAPL_data
)
SELECT
    date,
    close,
    returns,
    STDDEV(returns) OVER (ORDER BY date ROWS 19 PRECEDING) as volatility_20day
FROM returns_data
WHERE returns IS NOT NULL
ORDER BY date
LIMIT 30;
GO

-- ============================================
-- Example 7: Identify significant moves (> 2% change)
-- ============================================
WITH price_changes AS (
    SELECT
        date,
        close,
        RETURNS(close, LAG(close) OVER (ORDER BY date)) as returns
    FROM AAPL_data
)
SELECT
    date,
    close,
    RENDER_NUMBER(returns * 100, 'standard', 2) || '%' as returns_pct,
    CASE
        WHEN returns > 0.02 THEN 'UP >2%'
        WHEN returns < -0.02 THEN 'DOWN >2%'
        ELSE 'Normal'
    END as movement
FROM price_changes
WHERE ABS(returns) > 0.02
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 8: Calculate cumulative returns
-- Shows total return from the start of the period
-- ============================================
WITH daily_returns AS (
    SELECT
        date,
        close,
        RETURNS(close, LAG(close) OVER (ORDER BY date)) as daily_return
    FROM AAPL_data
),
indexed_prices AS (
    SELECT
        date,
        close,
        daily_return,
        EXP(SUM(LN(1 + COALESCE(daily_return, 0))) OVER (ORDER BY date)) - 1 as cumulative_return
    FROM daily_returns
)
SELECT
    date,
    close,
    RENDER_NUMBER(daily_return * 100, 'standard', 2) || '%' as daily_ret_pct,
    RENDER_NUMBER(cumulative_return * 100, 'standard', 2) || '%' as cumul_ret_pct
FROM indexed_prices
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 9: Basic statistics for the full period
-- ============================================
WITH returns_data AS (
    SELECT
        RETURNS(close, LAG(close) OVER (ORDER BY date)) as returns
    FROM AAPL_data
)
SELECT
    COUNT(*) as trading_days,
    RENDER_NUMBER(AVG(returns) * 100, 'standard', 4) || '%' as avg_daily_return,
    RENDER_NUMBER(MIN(returns) * 100, 'standard', 2) || '%' as worst_day,
    RENDER_NUMBER(MAX(returns) * 100, 'standard', 2) || '%' as best_day,
    RENDER_NUMBER(STDDEV(returns) * 100, 'standard', 4) || '%' as daily_volatility,
    RENDER_NUMBER(STDDEV(returns) * SQRT(252) * 100, 'standard', 2) || '%' as annual_volatility
FROM returns_data
WHERE returns IS NOT NULL;
GO

-- ============================================
-- Example 10: Monthly aggregation
-- ============================================
WITH monthly_data AS (
    SELECT
        SUBSTRING(date, 1, 7) as month,
        FIRST_VALUE(close) OVER (PARTITION BY SUBSTRING(date, 1, 7) ORDER BY date) as month_open,
        LAST_VALUE(close) OVER (PARTITION BY SUBSTRING(date, 1, 7) ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) as month_close,
        MIN(close) OVER (PARTITION BY SUBSTRING(date, 1, 7)) as month_low,
        MAX(close) OVER (PARTITION BY SUBSTRING(date, 1, 7)) as month_high
    FROM AAPL_data
)
SELECT DISTINCT
    month,
    RENDER_NUMBER(month_open, 'standard', 2) as open,
    RENDER_NUMBER(month_high, 'standard', 2) as high,
    RENDER_NUMBER(month_low, 'standard', 2) as low,
    RENDER_NUMBER(month_close, 'standard', 2) as close,
    RENDER_NUMBER((month_close - month_open) / month_open * 100, 'standard', 2) || '%' as month_return
FROM monthly_data
ORDER BY month
LIMIT 12;
GO