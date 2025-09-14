-- #! ../data/trades.csv
-- Time-based bar aggregation examples for financial data

-- ============================================
-- Basic Time Functions
-- ============================================

-- Convert timestamps to Unix epoch
SELECT 
    trade_time,
    UNIX_TIMESTAMP(trade_time) as epoch_seconds
FROM trades
LIMIT 5;

GO
-- Round timestamps to 1-minute boundaries
SELECT 
    trade_time,
    TIME_BUCKET(60, UNIX_TIMESTAMP(trade_time)) as minute_bucket,
    FROM_UNIXTIME(TIME_BUCKET(60, UNIX_TIMESTAMP(trade_time))) as minute_start
FROM trades
LIMIT 10;
GO

-- Round timestamps to 5-minute boundaries
SELECT 
    trade_time,
    TIME_BUCKET(300, UNIX_TIMESTAMP(trade_time)) as five_min_bucket,
    FROM_UNIXTIME(TIME_BUCKET(300, UNIX_TIMESTAMP(trade_time))) as bar_start
FROM trades
LIMIT 10;
GO

-- ============================================
-- 1-Minute OHLCV Bars (Manual Bucketing)
-- ============================================

-- First minute bar (9:30:00 - 9:30:59)
SELECT 
    '09:30' as minute,
    MIN(price) as low,
    MAX(price) as high,
    SUM(volume) as volume,
    COUNT(*) as trades,
    SUM(price * volume) / SUM(volume) as vwap
FROM trades
WHERE trade_time >= '2024-01-15 09:30:00' 
  AND trade_time < '2024-01-15 09:31:00';
GO

-- Second minute bar (9:31:00 - 9:31:59)
SELECT 
    '09:31' as minute,
    MIN(price) as low,
    MAX(price) as high,
    SUM(volume) as volume,
    COUNT(*) as trades,
    SUM(price * volume) / SUM(volume) as vwap
FROM trades
WHERE trade_time >= '2024-01-15 09:31:00' 
  AND trade_time < '2024-01-15 09:32:00';
GO

-- ============================================
-- 5-Minute OHLCV Bars
-- ============================================

-- 9:30-9:35 bar
SELECT 
    '09:30-09:35' as bar,
    MIN(trade_time) as first_trade,
    MAX(trade_time) as last_trade,
    MIN(price) as low,
    MAX(price) as high,
    SUM(volume) as volume,
    COUNT(*) as trades,
    SUM(price * volume) / SUM(volume) as vwap,
    MAX(price) - MIN(price) as range
FROM trades
WHERE trade_time >= '2024-01-15 09:30:00' 
  AND trade_time < '2024-01-15 09:35:00';
GO

-- ============================================
-- Statistical Metrics
-- ============================================

-- Full session statistics
SELECT 
    symbol,
    MIN(trade_time) as session_start,
    MAX(trade_time) as session_end,
    MIN(price) as low,
    MAX(price) as high,
    MAX(price) - MIN(price) as range,
    SUM(volume) as total_volume,
    COUNT(*) as total_trades,
    SUM(price * volume) / SUM(volume) as vwap,
    AVG(volume) as avg_trade_size
FROM trades
GROUP BY symbol;
GO

SELECT 
    *,
    CASE 
        WHEN volume < 1000 THEN '< 1K'
        WHEN volume < 2000 THEN '1K-2K'
        ELSE '2K+'
    END as size_bucket
FROM trades;
GO

-- Trade size distribution
with compute as
(
SELECT 
    *,
    CASE 
        WHEN volume < 1000 THEN '< 1K'
        WHEN volume < 2000 THEN '1K-2K'
        ELSE '2K+'
    END as size_bucket
FROM trades
) select size_bucket,
    COUNT(*) as count,
    AVG(price) as avg_price
    from compute
GROUP BY size_bucket
ORDER BY size_bucket;
GO

-- ============================================
-- Price Movement Analysis
-- ============================================

-- Identify large trades (> 2000 shares)
SELECT 
    trade_time,
    price,
    volume,
    price * volume as notional
FROM trades
WHERE volume > 2000
ORDER BY trade_time;
GO

-- Find high volume minutes (requires manual bucketing for now)
-- SELECT 
    -- '09:30' as minute,
    -- SUM(volume) as volume,
    -- SUM(price * volume) as notional
-- FROM trades
-- WHERE trade_time >= '2024-01-15 09:30:00' 
  -- AND trade_time < '2024-01-15 09:31:00'
-- UNION ALL
-- SELECT 
    -- '09:31' as minute,
    -- SUM(volume) as volume,
    -- SUM(price * volume) as notional
-- FROM trades
-- WHERE trade_time >= '2024-01-15 09:31:00' 
  -- AND trade_time < '2024-01-15 09:32:00'
-- UNION ALL
-- SELECT 
    -- '09:32' as minute,
    -- SUM(volume) as volume,
    -- SUM(price * volume) as notional
-- FROM trades
-- WHERE trade_time >= '2024-01-15 09:32:00' 
  -- AND trade_time < '2024-01-15 09:33:00'
-- ORDER BY volume DESC;
