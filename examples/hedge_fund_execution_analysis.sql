-- #! Multi-stage Execution Analysis Pipeline
-- Demonstrates sophisticated analysis workflow similar to hedge fund trading systems
--
-- This example shows:
-- 1. Fetching FIX message data (simulates parsing FIX logs)
-- 2. Computing timing metrics and rolling aggregates with window functions
-- 3. Joining with trade database
-- 4. Enriching with securities master data
-- 5. Final analytics: execution quality, slippage, VWAP analysis
--
-- Usage:
--   Run all:     ./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql
--   Run stage 5: ./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql --execute-statement 5
--   In Neovim:   \sq (all) or \sx (statement at cursor)

-- need to run the python test server for this example to work
-- uv run python tests/test_trade_server.py

exit;
go

-- ============================================================================
-- Stage 1: Parse FIX Messages (simulates FIX log parsing)
-- ============================================================================
WITH WEB fix_data AS (
    URL 'http://localhost:5001/fix_messages'
    METHOD POST
    HEADERS ('Content-Type': 'application/json')
    BODY '{"tags": ["11", "37", "55", "54", "31", "32", "52", "60"]}'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    ClOrdID,
    OrderID,
    Symbol,
    CASE WHEN Side = '1' THEN 'Buy' ELSE 'Sell' END as Side,
    LastPx as ExecutionPrice,
    LastQty as ExecutionQty,
    SendingTime,
    TransactTime,
    LatencyMs
FROM fix_data
INTO #fix_messages;
GO

-- ============================================================================
-- Stage 2: Compute Timing Metrics and Rolling Aggregates
-- ============================================================================
SELECT
    *,
    -- Time lag between consecutive messages (in seconds)
    LAG(LatencyMs) OVER (PARTITION BY Symbol ORDER BY SendingTime) as prev_latency,
    -- Rolling average execution price (last 5 trades)
    AVG(ExecutionPrice) OVER (
        PARTITION BY Symbol
        ORDER BY SendingTime
        ROWS BETWEEN 4 PRECEDING AND CURRENT ROW
    ) as rolling_vwap_5,
    -- Cumulative volume
    SUM(ExecutionQty) OVER (
        PARTITION BY Symbol
        ORDER BY SendingTime
    ) as cumulative_volume,
    -- Execution sequence number per symbol
    ROW_NUMBER() OVER (PARTITION BY Symbol ORDER BY SendingTime) as exec_sequence
FROM #fix_messages
INTO #enriched_fix;
GO

-- ============================================================================
-- Stage 3: Fetch Trade Database Records
-- ============================================================================
WITH WEB db_trades_raw AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    HEADERS ('Content-Type': 'application/json')
    BODY '{
        "Select": "Source,PlatformOrderId,BloomberTicker,SignedQuantity,BuySell,Price",
        "Where": "Source = \"Bloomberg\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    PlatformOrderId as OrderID,
    BloomberTicker as Ticker,
    BuySell as Side,
    SignedQuantity as Quantity,
    ABS(SignedQuantity) as AbsQuantity,
    Price,
    Source
FROM db_trades_raw
INTO #db_trades;
GO

-- ============================================================================
-- Stage 4: Enrich with Securities Master Data
-- ============================================================================
WITH WEB securities_raw AS (
    URL 'http://localhost:5001/securities'
    METHOD POST
    HEADERS ('Content-Type': 'application/json')
    BODY '{}'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    Ticker,
    SecurityName,
    Sector,
    Exchange,
    Currency,
    ISIN
FROM securities_raw
INTO #securities;
GO

-- ============================================================================
-- Stage 5: Join FIX with Trades and Securities for Full Picture
-- ============================================================================
SELECT
    f.Symbol,
    s.SecurityName,
    s.Sector,
    f.Side,
    f.ExecutionQty,
    f.ExecutionPrice,
    f.LatencyMs,
    f.rolling_vwap_5,
    f.cumulative_volume,
    f.exec_sequence,
    d.Source as TradeSource
FROM #enriched_fix f
LEFT JOIN #db_trades d ON f.Symbol = d.Ticker
LEFT JOIN #securities s ON f.Symbol = s.Ticker
INTO #full_dataset;
GO

-- ============================================================================
-- Stage 6: Execution Quality Analysis by Sector
-- ============================================================================
SELECT
    Sector,
    Side,
    COUNT(*) as execution_count,
    SUM(ExecutionQty) as total_volume,
    AVG(ExecutionPrice) as avg_execution_price,
    AVG(LatencyMs) as avg_latency_ms,
    MIN(LatencyMs) as best_latency_ms,
    MAX(LatencyMs) as worst_latency_ms
FROM #full_dataset
WHERE Sector IS NOT NULL
GROUP BY Sector, Side
ORDER BY total_volume DESC;
GO

-- ============================================================================
-- Stage 7: Symbol-Level VWAP Analysis
-- ============================================================================
SELECT
    Symbol,
    SecurityName,
    Sector,
    COUNT(*) as num_executions,
    SUM(ExecutionQty) as total_quantity,
    AVG(ExecutionPrice) as simple_avg_price,
    -- Note: True VWAP would be SUM(Qty*Price)/SUM(Qty) but we'll use rolling_vwap_5
    AVG(rolling_vwap_5) as avg_rolling_vwap,
    MAX(cumulative_volume) as final_cumulative_volume,
    AVG(LatencyMs) as avg_latency
FROM #full_dataset
GROUP BY Symbol, SecurityName, Sector
ORDER BY total_quantity DESC
LIMIT 10;
GO

-- ============================================================================
-- Stage 8: Latency Distribution Analysis
-- ============================================================================
SELECT
    CASE
        WHEN LatencyMs < 100 THEN '< 100ms'
        WHEN LatencyMs < 200 THEN '100-200ms'
        WHEN LatencyMs < 300 THEN '200-300ms'
        WHEN LatencyMs < 400 THEN '300-400ms'
        ELSE '> 400ms'
    END as latency_bucket,
    COUNT(*) as execution_count,
    AVG(ExecutionPrice) as avg_price,
    SUM(ExecutionQty) as total_volume
FROM #full_dataset
GROUP BY CASE
    WHEN LatencyMs < 100 THEN '< 100ms'
    WHEN LatencyMs < 200 THEN '100-200ms'
    WHEN LatencyMs < 300 THEN '200-300ms'
    WHEN LatencyMs < 400 THEN '300-400ms'
    ELSE '> 400ms'
END;
GO
