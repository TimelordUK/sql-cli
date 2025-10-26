-- #!
-- Time Series Data Generator
-- Demonstrates how to generate Q-style time series data (trades, market data, etc.)
-- Similar to Q's `?` operator for generating test data with temporal ordering

-- ============================================================================
-- BASIC TIME SERIES GENERATION
-- ============================================================================

-- Generate 20 timestamps for January 2024
-- Using RANGE to create row numbers, then add random hours/minutes/seconds
WITH time_offsets AS (
    SELECT
        id,
        value AS offset_seconds
    FROM RANDOM_INT(20, 0, 86400, 42)  -- 86400 seconds in a day
)
SELECT
    id + 1 AS row_num,
    DATEADD('second', offset_seconds, '2024-01-01 00:00:00') AS timestamp,
    offset_seconds
FROM time_offsets
ORDER BY offset_seconds;
GO

-- ============================================================================
-- SIMPLE TRADES TABLE (100 rows)
-- ============================================================================

-- Generate a basic trades table with date, time, symbol, volume, and price
WITH
    -- Generate 100 random timestamps in January 2024
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-01 00:00:00') AS dt
        FROM RANDOM_INT(100, 0, 2678400, 100)  -- 31 days * 86400 seconds
    ),
    -- Generate 100 random symbols (aapl, goog, ibm)
    symbols_raw AS (
        SELECT
            id,
            value AS sym_index
        FROM RANDOM_INT(100, 0, 2, 200)  -- 0, 1, or 2
    ),
    symbols AS (
        SELECT
            id,
            CASE sym_index
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
            END AS sym
        FROM symbols_raw
    ),
    -- Generate 100 random volumes (lots of 10, from 10 to 10000)
    volumes AS (
        SELECT
            id,
            10 * (1 + value) AS vol
        FROM RANDOM_INT(100, 0, 999, 300)
    ),
    -- Generate 100 random prices (90-110 for AAPL baseline)
    prices_raw AS (
        SELECT
            id,
            90.0 + (value / 100.0) AS px_base
        FROM RANDOM_INT(100, 0, 2000, 400)
    )
-- Combine all columns and adjust prices by symbol
SELECT
    t.dt,
    s.sym,
    v.vol,
    CASE s.sym
        WHEN 'aapl' THEN ROUND(p.px_base, 2)
        WHEN 'goog' THEN ROUND(p.px_base * 6, 2)
        WHEN 'ibm' THEN ROUND(p.px_base * 2, 2)
    END AS px
FROM timestamps t
JOIN symbols s ON t.id = s.id
JOIN volumes v ON t.id = v.id
JOIN prices_raw p ON t.id = p.id
ORDER BY t.dt
LIMIT 20;  -- Show first 20 for readability
GO

-- ============================================================================
-- LARGE TRADES TABLE (1000 rows with microsecond precision)
-- ============================================================================

-- Generate a more realistic trades table with intraday timestamps
WITH
    -- Generate base dates (spread across 5 days)
    dates AS (
        SELECT
            id,
            DATEADD('day', value % 5, '2024-01-15') AS base_date
        FROM RANDOM_INT(1000, 0, 10000, 500)
    ),
    -- Generate intraday times (9:30 AM to 4:00 PM = 23400 seconds)
    times AS (
        SELECT
            id,
            34200 + value AS seconds_offset  -- 34200 = 9:30 AM
        FROM RANDOM_INT(1000, 0, 23400, 600)
    ),
    -- Combine date and time
    timestamps AS (
        SELECT
            d.id,
            DATEADD('second', t.seconds_offset, d.base_date) AS dt
        FROM dates d
        JOIN times t ON d.id = t.id
    ),
    -- Generate symbols with weighted distribution
    symbols AS (
        SELECT
            id,
            CASE
                WHEN value < 400 THEN 'aapl'  -- 40% AAPL
                WHEN value < 700 THEN 'goog'  -- 30% GOOG
                WHEN value < 900 THEN 'ibm'   -- 20% IBM
                ELSE 'msft'                   -- 10% MSFT
            END AS sym
        FROM RANDOM_INT(1000, 0, 999, 700)
    ),
    -- Generate volumes with log-normal-ish distribution
    volumes AS (
        SELECT
            id,
            100 * (1 + value) AS vol
        FROM RANDOM_INT(1000, 0, 100, 800)
    ),
    -- Generate prices
    prices AS (
        SELECT
            id,
            sym,
            CASE sym
                WHEN 'aapl' THEN 95.0 + (value / 100.0)
                WHEN 'goog' THEN 550.0 + (value / 10.0)
                WHEN 'ibm' THEN 180.0 + (value / 50.0)
                WHEN 'msft' THEN 380.0 + (value / 20.0)
            END AS px
        FROM symbols s
        JOIN (SELECT id, value FROM RANDOM_INT(1000, 0, 1000, 900)) r
        ON s.id = r.id
    )
-- Final trades table
SELECT
    t.dt,
    p.sym,
    v.vol,
    ROUND(p.px, 2) AS px
FROM timestamps t
JOIN prices p ON t.id = p.id
JOIN volumes v ON t.id = v.id
ORDER BY t.dt;
GO

-- ============================================================================
-- CUSTOMIZABLE TRADES GENERATOR (use as template)
-- ============================================================================

-- Template: Modify parameters directly in the RANDOM_INT calls
-- To customize:
--   - Row count: Change first argument in all RANDOM_INT calls (e.g., 500, 5000)
--   - Date range: Change DATEADD base date and seconds range
--   - Symbols: Modify CASE statement in symbols CTE
--   - Price ranges: Adjust multipliers in prices CTE
--   - Seed: Change seed values (last argument) for different random sequences

WITH
    -- Timestamps (CUSTOMIZE: 500 rows, 31 days = 2678400 seconds, seed=50)
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-02-01 00:00:00') AS dt
        FROM RANDOM_INT(500, 0, 2678400, 50)
    ),
    -- Symbols (CUSTOMIZE: add/remove symbols in CASE statement)
    symbols AS (
        SELECT
            id,
            CASE value % 5
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
                WHEN 3 THEN 'msft'
                WHEN 4 THEN 'tsla'
            END AS sym
        FROM RANDOM_INT(500, 0, 999, 150)
    ),
    -- Volumes (CUSTOMIZE: multiplier and range)
    volumes AS (
        SELECT
            id,
            100 * (1 + value) AS vol
        FROM RANDOM_INT(500, 1, 100, 250)
    ),
    -- Prices (CUSTOMIZE: base prices and ranges per symbol)
    prices AS (
        SELECT
            s.id,
            s.sym,
            CASE s.sym
                WHEN 'aapl' THEN 95.0 + (r.value / 50.0)
                WHEN 'goog' THEN 550.0 + (r.value / 5.0)
                WHEN 'ibm' THEN 180.0 + (r.value / 25.0)
                WHEN 'msft' THEN 380.0 + (r.value / 10.0)
                WHEN 'tsla' THEN 200.0 + (r.value / 5.0)
            END AS px
        FROM symbols s
        JOIN (SELECT id, value FROM RANDOM_INT(500, 0, 1000, 350)) r
        ON s.id = r.id
    )
-- Generated trades table
SELECT
    t.dt,
    p.sym,
    v.vol,
    ROUND(p.px, 2) AS px
FROM timestamps t
JOIN prices p ON t.id = p.id
JOIN volumes v ON t.id = v.id
ORDER BY t.dt
LIMIT 30;  -- Remove LIMIT to see all rows
GO

-- ============================================================================
-- QUOTES TABLE (bid/ask spreads)
-- ============================================================================

-- Generate a quotes table with bid/ask prices
WITH
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(100, 0, 23400, 1000)  -- Market hours
    ),
    symbols AS (
        SELECT
            id,
            CASE value % 3
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
            END AS sym
        FROM RANDOM_INT(100, 0, 999, 1100)
    ),
    mid_prices AS (
        SELECT
            s.id,
            s.sym,
            CASE s.sym
                WHEN 'aapl' THEN 100.0
                WHEN 'goog' THEN 600.0
                WHEN 'ibm' THEN 200.0
            END + (r.value / 100.0) AS mid_px
        FROM symbols s
        JOIN (SELECT id, value FROM RANDOM_INT(100, -500, 500, 1200)) r
        ON s.id = r.id
    ),
    spreads AS (
        SELECT
            id,
            0.01 + (value / 10000.0) AS spread
        FROM RANDOM_INT(100, 1, 50, 1300)
    )
SELECT
    t.dt,
    m.sym,
    ROUND(m.mid_px - (s.spread / 2), 2) AS bid,
    ROUND(m.mid_px + (s.spread / 2), 2) AS ask,
    ROUND(s.spread, 4) AS spread
FROM timestamps t
JOIN mid_prices m ON t.id = m.id
JOIN spreads s ON t.id = s.id
ORDER BY t.dt
LIMIT 20;
GO

-- ============================================================================
-- SENSOR DATA (IoT time series)
-- ============================================================================

-- Generate temperature sensor readings
WITH
    timestamps AS (
        SELECT
            id,
            DATEADD('minute', value, '2024-01-15 00:00:00') AS dt
        FROM RANDOM_INT(100, 0, 1440, 1400)  -- 24 hours of minutes
    ),
    sensors AS (
        SELECT
            id,
            'sensor_' || (value % 5) AS sensor_id
        FROM RANDOM_INT(100, 0, 999, 1500)
    ),
    -- Generate temperature with realistic variations (68-76°F)
    temperatures AS (
        SELECT
            id,
            68.0 + (value / 100.0) AS temp_f
        FROM RANDOM_INT(100, 0, 800, 1600)
    )
SELECT
    t.dt,
    s.sensor_id,
    ROUND(temp.temp_f, 1) AS temperature_f,
    ROUND((temp.temp_f - 32) * 5 / 9, 1) AS temperature_c
FROM timestamps t
JOIN sensors s ON t.id = s.id
JOIN temperatures temp ON t.id = temp.id
ORDER BY t.dt, s.sensor_id
LIMIT 30;
GO

-- ============================================================================
-- AGGREGATED TIME SERIES (OHLC-style bars)
-- ============================================================================

-- Generate tick data, then aggregate into time-based bars
-- This demonstrates post-processing of time series data

WITH
    -- Generate 500 ticks
    ticks AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-15 09:30:00') AS dt,
            'aapl' AS sym,
            100.0 + (value / 1000.0) AS px
        FROM RANDOM_INT(500, 0, 3600, 1700)  -- 1 hour of data
    ),
    -- Create 5-minute bars (300 seconds) - use integer division
    bars AS (
        SELECT
            dt,
            QUOTIENT(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP('2024-01-15 09:30:00'), 300) AS bar_num,
            px
        FROM ticks
    ),
    -- Aggregate by bar
    aggregated AS (
        SELECT
            bar_num,
            COUNT(*) AS tick_count,
            ROUND(MIN(px), 2) AS low,
            ROUND(MAX(px), 2) AS high,
            ROUND(AVG(px), 2) AS avg_px
        FROM bars
        GROUP BY bar_num
    )
-- Add timestamps to bars
SELECT
    bar_num,
    DATEADD('second', bar_num * 300, '2024-01-15 09:30:00') AS bar_time,
    tick_count,
    low,
    high,
    avg_px
FROM aggregated
ORDER BY bar_num
LIMIT 12;  -- First hour in 5-min bars
GO

-- ============================================================================
-- USAGE PATTERNS AND BEST PRACTICES
-- ============================================================================

-- Key techniques demonstrated:
-- 1. Use RANDOM_INT/RANDOM_FLOAT with seeds for reproducible data
-- 2. Use RANGE(0, N) as base, then JOIN with random generators
-- 3. Use DATEADD to create timestamps from base date + offsets
-- 4. Use CTEs to build complex datasets step-by-step
-- 5. Use CASE statements for categorical data (symbols, status, etc.)
-- 6. Adjust random ranges to match realistic data distributions
-- 7. Use JOINs on id to align multiple random columns
-- 8. Always ORDER BY timestamp for time series data

-- Tips for large datasets:
-- - For > 10K rows, consider breaking into multiple queries
-- - Use appropriate seeds to ensure reproducibility
-- - Adjust random ranges to match your data characteristics
-- - Use window functions for derived time series metrics

-- Example queries you can run on generated data:
-- - Volume-weighted average price (VWAP)
-- - Moving averages
-- - Bollinger bands
-- - Trade imbalance analysis
-- - Volatility calculations
-- - Time-based aggregations

-- End of time series generator examples
