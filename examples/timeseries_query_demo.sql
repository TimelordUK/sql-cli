-- #!
-- Time Series Query Demo
-- Shows how to generate data and run analytical queries on it

-- Generate 500 trades and calculate analytics
WITH
    -- Generate trades data
    timestamps AS (
        SELECT
            id,
            DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(500, 0, 23400, 2000)  -- Market hours
    ),
    symbols AS (
        SELECT
            id,
            CASE value % 3
                WHEN 0 THEN 'aapl'
                WHEN 1 THEN 'goog'
                WHEN 2 THEN 'ibm'
            END AS sym
        FROM RANDOM_INT(500, 0, 999, 2100)
    ),
    volumes AS (
        SELECT id, 100 * (1 + value) AS vol
        FROM RANDOM_INT(500, 1, 100, 2200)
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
        JOIN (SELECT id, value FROM RANDOM_INT(500, 0, 1000, 2300)) r
        ON s.id = r.id
    ),
    trades AS (
        SELECT t.dt, p.sym, v.vol, ROUND(p.px, 2) AS px
        FROM timestamps t
        JOIN prices p ON t.id = p.id
        JOIN volumes v ON t.id = v.id
    )
-- Calculate VWAP (Volume-Weighted Average Price) by symbol
SELECT
    sym,
    COUNT(*) AS trade_count,
    FORMAT_NUMBER(SUM(vol), 0) AS total_volume,
    ROUND(SUM(px * vol) / SUM(vol), 2) AS vwap,
    ROUND(MIN(px), 2) AS min_price,
    ROUND(MAX(px), 2) AS max_price,
    ROUND(MAX(px) - MIN(px), 2) AS price_range
FROM trades
GROUP BY sym
ORDER BY total_volume DESC;
GO

-- Generate trades with moving averages
WITH
    timestamps AS (
        SELECT id, DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(200, 0, 10800, 3000)  -- 3 hours
    ),
    symbols AS (
        SELECT id, 'aapl' AS sym
        FROM RANDOM_INT(200, 0, 999, 3100)
    ),
    prices AS (
        SELECT id, 100.0 + (value / 100.0) AS px
        FROM RANDOM_INT(200, 0, 1000, 3200)
    ),
    trades AS (
        SELECT t.dt, s.sym, ROUND(p.px, 2) AS px
        FROM timestamps t
        JOIN symbols s ON t.id = s.id
        JOIN prices p ON t.id = p.id
    )
-- Calculate moving averages
SELECT
    dt,
    sym,
    px,
    ROUND(AVG(px) OVER (ORDER BY dt ROWS BETWEEN 4 PRECEDING AND CURRENT ROW), 2) AS ma_5,
    ROUND(AVG(px) OVER (ORDER BY dt ROWS BETWEEN 19 PRECEDING AND CURRENT ROW), 2) AS ma_20
FROM trades
ORDER BY dt
LIMIT 30;
GO

-- Volume analysis by hour
WITH
    timestamps AS (
        SELECT id, DATEADD('second', value, '2024-01-15 09:30:00') AS dt
        FROM RANDOM_INT(300, 0, 23400, 4000)
    ),
    symbols AS (
        SELECT id, CASE value % 3 WHEN 0 THEN 'aapl' WHEN 1 THEN 'goog' ELSE 'ibm' END AS sym
        FROM RANDOM_INT(300, 0, 999, 4100)
    ),
    volumes AS (
        SELECT id, 100 * (1 + value) AS vol
        FROM RANDOM_INT(300, 1, 100, 4200)
    ),
    trades AS (
        SELECT
            t.dt,
            s.sym,
            v.vol,
            QUOTIENT(UNIX_TIMESTAMP(t.dt) - UNIX_TIMESTAMP('2024-01-15 09:30:00'), 3600) AS hour_num
        FROM timestamps t
        JOIN symbols s ON t.id = s.id
        JOIN volumes v ON t.id = v.id
    ),
    aggregated AS (
        SELECT
            hour_num,
            sym,
            COUNT(*) AS trade_count,
            SUM(vol) AS total_volume
        FROM trades
        GROUP BY hour_num, sym
    )
SELECT
    hour_num,
    DATEADD('hour', hour_num, '2024-01-15 09:30:00') AS hour_start,
    sym,
    trade_count,
    FORMAT_NUMBER(total_volume, 0) AS total_volume
FROM aggregated
ORDER BY hour_num, sym;
GO
