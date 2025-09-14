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
-- Example 5: Calculate moving averages with window frames
-- Now supports ROWS n PRECEDING for rolling calculations!
-- ============================================
SELECT
    date,
    close,
    RENDER_NUMBER(AVG(close) OVER (ORDER BY date ROWS 4 PRECEDING), 'standard', 2) as ma_5day,
    RENDER_NUMBER(AVG(close) OVER (ORDER BY date ROWS 9 PRECEDING), 'standard', 2) as ma_10day,
    RENDER_NUMBER(AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING), 'standard', 2) as ma_20day
FROM AAPL_data
ORDER BY date
LIMIT 30;
GO

-- ============================================
-- Example 6: Calculate rolling min/max (support and resistance levels)
-- Using window frames to find local highs and lows
-- ============================================
SELECT
    date,
    close,
    MIN(close) OVER (ORDER BY date ROWS 19 PRECEDING) as support_20day,
    MAX(close) OVER (ORDER BY date ROWS 19 PRECEDING) as resistance_20day,
    RENDER_NUMBER((close - MIN(close) OVER (ORDER BY date ROWS 19 PRECEDING)) /
                  (MAX(close) OVER (ORDER BY date ROWS 19 PRECEDING) -
                   MIN(close) OVER (ORDER BY date ROWS 19 PRECEDING)) * 100, 'standard', 1) || '%' as position_in_range
FROM AAPL_data
WHERE date >= '2013-03-01'
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 7: Calculate rolling statistics with different windows
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
    RENDER_NUMBER(returns * 100, 'standard', 2) || '%' as returns_pct,
    COUNT(returns) OVER (ORDER BY date) as days_count,
    RENDER_NUMBER(AVG(returns) OVER (ORDER BY date) * 100, 'standard', 4) || '%' as avg_return_to_date
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
-- Note: STDDEV as aggregate not yet implemented, showing other stats
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
    RENDER_NUMBER(AVG(returns) * 252 * 100, 'standard', 2) || '%' as annualized_return
FROM returns_data
WHERE returns IS NOT NULL;
GO

-- ============================================
-- Example 10: Demonstrate UNBOUNDED frame syntax
-- Note: PARTITION BY doesn't support expressions yet, so we can't group by month
-- ============================================
SELECT
    date,
    close,
    -- Running statistics from start to current row
    MIN(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as min_to_date,
    MAX(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as max_to_date,
    AVG(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as avg_to_date,
    -- Get first and last values in entire dataset
    FIRST_VALUE(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) as first_close,
    LAST_VALUE(close) OVER (ORDER BY date ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) as last_close
FROM AAPL_data
WHERE date >= '2013-03-01' AND date < '2013-04-01'
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 11: Bollinger Bands calculation (with real STDDEV!)
-- Moving average +/- 2 standard deviations
-- ============================================
WITH price_stats AS (
    SELECT
        date,
        close,
        AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING) as ma_20,
        STDDEV(close) OVER (ORDER BY date ROWS 19 PRECEDING) as stddev_20
    FROM AAPL_data
)
SELECT
    date,
    RENDER_NUMBER(close, 'standard', 2) as price,
    RENDER_NUMBER(ma_20, 'standard', 2) as middle_band,
    RENDER_NUMBER(ma_20 - 2 * stddev_20, 'standard', 2) as lower_band,
    RENDER_NUMBER(ma_20 + 2 * stddev_20, 'standard', 2) as upper_band,
    RENDER_NUMBER(stddev_20, 'standard', 4) as volatility,
    CASE
        WHEN close < ma_20 - 2 * stddev_20 THEN 'OVERSOLD'
        WHEN close > ma_20 + 2 * stddev_20 THEN 'OVERBOUGHT'
        ELSE 'NEUTRAL'
    END as signal
FROM price_stats
WHERE date >= '2013-04-01'
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 11b: Z-Score calculation
-- How many standard deviations from the mean
-- ============================================
WITH stats AS (
    SELECT
        date,
        close,
        AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING) as ma_20,
        STDDEV(close) OVER (ORDER BY date ROWS 19 PRECEDING) as stddev_20
    FROM AAPL_data
)
SELECT
    date,
    close,
    RENDER_NUMBER((close - ma_20) / stddev_20, 'standard', 2) as z_score,
    CASE
        WHEN ABS((close - ma_20) / stddev_20) > 3 THEN 'EXTREME'
        WHEN ABS((close - ma_20) / stddev_20) > 2 THEN 'SIGNIFICANT'
        WHEN ABS((close - ma_20) / stddev_20) > 1 THEN 'NOTABLE'
        ELSE 'NORMAL'
    END as deviation_level
FROM stats
WHERE date >= '2013-04-01' AND stddev_20 > 0
ORDER BY date
LIMIT 20;
GO

-- ============================================
-- Example 12: Momentum indicators with window frames
-- Rate of change and price oscillators
-- ============================================
SELECT
    date,
    close,
    LAG(close, 10) OVER (ORDER BY date) as close_10days_ago,
    RENDER_NUMBER((close - LAG(close, 10) OVER (ORDER BY date)) /
                  LAG(close, 10) OVER (ORDER BY date) * 100, 'standard', 2) || '%' as roc_10day,
    RENDER_NUMBER((AVG(close) OVER (ORDER BY date ROWS 11 PRECEDING) -
                   AVG(close) OVER (ORDER BY date ROWS 25 PRECEDING)) /
                  AVG(close) OVER (ORDER BY date ROWS 25 PRECEDING) * 100, 'standard', 2) || '%' as price_oscillator
FROM AAPL_data
WHERE date >= '2013-04-01'
ORDER BY date
LIMIT 20;
GO