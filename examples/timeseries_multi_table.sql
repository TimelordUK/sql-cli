-- #!
-- Multi-Table Time Series Analysis
-- Demonstrates generating multiple related tables (trades, quotes)
-- and running cross-table analytics - perfect for market microstructure analysis

-- ============================================================================
-- STEP 1: Generate QUOTES table (bid/ask data)
-- ============================================================================

WITH
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(500, 0, 3600, 1000)  -- 1 hour of quotes
    ),
    symbols AS (
        SELECT
            id,
            CASE value % 3
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
            END AS sym
        FROM RANDOM_INT(500, 0, 999, 1100)
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
        JOIN (SELECT id, value FROM RANDOM_INT(500, -300, 300, 1200)) r
        ON s.id = r.id
    ),
    spreads AS (
        SELECT
            id,
            0.01 + (value / 10000.0) AS spread
        FROM RANDOM_INT(500, 1, 50, 1300)
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
INTO #quotes;
GO

-- ============================================================================
-- STEP 2: Generate TRADES table
-- ============================================================================

WITH
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(200, 0, 3600, 2000)  -- 1 hour of trades (fewer than quotes)
    ),
    symbols AS (
        SELECT
            id,
            CASE value % 3
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
            END AS sym
        FROM RANDOM_INT(200, 0, 999, 2100)
    ),
    volumes AS (
        SELECT
            id,
            100 * (1 + value) AS vol
        FROM RANDOM_INT(200, 1, 100, 2200)
    ),
    prices AS (
        SELECT
            s.id,
            s.sym,
            CASE s.sym
                WHEN 'aapl' THEN 100.0 + (r.value / 100.0)
                WHEN 'goog' THEN 600.0 + (r.value / 10.0)
                WHEN 'ibm' THEN 200.0 + (r.value / 50.0)
            END AS px
        FROM symbols s
        JOIN (SELECT id, value FROM RANDOM_INT(200, -300, 300, 2300)) r
        ON s.id = r.id
    )
SELECT
    t.dt,
    p.sym,
    v.vol,
    ROUND(p.px, 2) AS px
FROM timestamps t
JOIN prices p ON t.id = p.id
JOIN volumes v ON t.id = v.id
ORDER BY t.dt
INTO #trades;
GO

-- ============================================================================
-- ANALYSIS 1: Quote Summary by Symbol
-- ============================================================================

SELECT
    sym,
    COUNT(*) AS quote_count,
    ROUND(AVG(bid), 2) AS avg_bid,
    ROUND(AVG(ask), 2) AS avg_ask,
    ROUND(AVG(spread), 4) AS avg_spread,
    ROUND(MIN(bid), 2) AS min_bid,
    ROUND(MAX(ask), 2) AS max_ask
FROM #quotes
GROUP BY sym
ORDER BY sym;
GO

-- ============================================================================
-- ANALYSIS 2: Trade Summary by Symbol
-- ============================================================================

SELECT
    sym,
    COUNT(*) AS trade_count,
    FORMAT_NUMBER(SUM(vol), 0) AS total_volume,
    ROUND(SUM(px * vol) / SUM(vol), 2) AS vwap,
    ROUND(MIN(px), 2) AS min_price,
    ROUND(MAX(px), 2) AS max_price
FROM #trades
GROUP BY sym
ORDER BY sym;
GO

-- ============================================================================
-- ANALYSIS 3: Trade and Quote Sample (simplified)
-- ============================================================================
-- Show sample of trades and quotes side by side

SELECT
    'TRADES' AS source,
    t.dt,
    t.sym,
    t.px AS price,
    t.vol AS volume
FROM #trades t
ORDER BY t.dt
LIMIT 10;
GO

SELECT
    'QUOTES' AS source,
    q.dt,
    q.sym,
    q.bid,
    q.ask,
    q.spread
FROM #quotes q
ORDER BY q.dt
LIMIT 10;
GO

-- ============================================================================
-- ANALYSIS 4: Price Volatility Analysis
-- ============================================================================
-- Calculate price ranges and volatility measures

SELECT
    sym,
    COUNT(*) AS trade_count,
    ROUND(MIN(px), 2) AS min_px,
    ROUND(MAX(px), 2) AS max_px,
    ROUND(MAX(px) - MIN(px), 2) AS price_range,
    ROUND(AVG(px), 2) AS avg_px
FROM #trades
GROUP BY sym
ORDER BY sym;
GO

-- ============================================================================
-- ANALYSIS 5: Quote Activity by Time Bar
-- ============================================================================
-- Count quotes per 5-minute interval

SELECT
    QUOTIENT(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP('2024-01-15 09:30:00'), 300) AS bar_num,
    sym,
    COUNT(*) AS quote_count
FROM #quotes
GROUP BY bar_num, sym
ORDER BY bar_num, sym
LIMIT 20;
GO

-- Trade activity by time bar
SELECT
    QUOTIENT(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP('2024-01-15 09:30:00'), 300) AS bar_num,
    sym,
    COUNT(*) AS trade_count,
    FORMAT_NUMBER(SUM(vol), 0) AS total_volume
FROM #trades
GROUP BY bar_num, sym
ORDER BY bar_num, sym
LIMIT 20;
GO

-- ============================================================================
-- ANALYSIS 6: Quote Update Frequency
-- ============================================================================
-- Measure time between quote updates for each symbol

WITH
    quote_times AS (
        SELECT
            sym,
            dt,
            LAG(dt) OVER (PARTITION BY sym ORDER BY dt) AS prev_dt
        FROM #quotes
    )
SELECT
    sym,
    COUNT(*) AS quote_updates,
    ROUND(AVG(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP(prev_dt)), 2) AS avg_seconds_between_quotes,
    ROUND(MIN(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP(prev_dt)), 2) AS min_seconds,
    ROUND(MAX(UNIX_TIMESTAMP(dt) - UNIX_TIMESTAMP(prev_dt)), 2) AS max_seconds
FROM quote_times
WHERE prev_dt IS NOT NULL
GROUP BY sym
ORDER BY sym;
GO

-- ============================================================================
-- USAGE NOTES
-- ============================================================================

-- This multi-table approach is perfect for:
-- 1. Market microstructure analysis
-- 2. Testing cross-table joins
-- 3. Building complex datasets incrementally
-- 4. Simulating real-world trading scenarios

-- In Neovim plugin, you can run these step by step:
-- 1. Generate #quotes with first query
-- 2. Generate #trades with second query
-- 3. Run any of the analytical queries
-- 4. Modify and re-run as needed

-- The temp tables persist for your session, so you can:
-- - Build up multiple tables
-- - Run exploratory queries
-- - Refine your analysis
-- - Test different scenarios by regenerating with different seeds

-- End of multi-table analysis examples
