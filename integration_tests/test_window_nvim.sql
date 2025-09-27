-- Test file for nvim window function integration
-- Try pressing K on any of these function names in nvim
--
-- The window functions should now appear in:
-- 1. The floating window when you type a function name
-- 2. The K help when cursor is on a function name
-- 3. The --list-functions output
--
-- Window functions require OVER clause but simplify complex patterns

SELECT
    date,
    close,
    -- Syntactic sugar window functions (press K on function name)
    MOVING_AVG(close, 20) OVER (ORDER BY date) as ma_20,
    ROLLING_STDDEV(close, 20) OVER (ORDER BY date) as volatility,
    CUMULATIVE_SUM(volume) OVER (ORDER BY date) as cumul_volume,
    CUMULATIVE_AVG(close) OVER (ORDER BY date) as avg_to_date,
    Z_SCORE(close, 20) OVER (ORDER BY date) as z_score,

    -- Regular functions that should also work with K
    ROUND(close, 2) as rounded_close,
    CONVERT(close, 'USD', 'EUR') as price_eur
FROM stock_data
ORDER BY date;