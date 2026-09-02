-- [TEST:SKIP] nvim plugin macro template (expand with \sTe); needs the test server on localhost:5001
-- ============================================
-- SQL-CLI Trade Queries Master Template
-- ============================================
-- This is the definitive template for trade queries with macros
-- Server: http://localhost:5001 (test server)
-- Setup: export JWT_TOKEN="your-token-here"
--
-- USAGE:
-- 1. Place cursor on @MACRO_NAME
-- 2. Press \sTe to expand
-- 3. Fill in prompts (SOURCE picker, DATE picker, etc.)
-- 4. Run with \sq

-- ============================================
-- CONFIGURATION
-- ============================================

-- MACRO: CONFIG
-- BASE_URL = http://localhost:5001
-- API_TOKEN = ${JWT_TOKEN}
-- DEFAULT_COLUMNS = TradeId, Source, Symbol, Quantity, Price, TradeDate, Status
-- END MACRO

-- MACRO: SOURCES
-- Bloomberg_FIX_FX
-- Bloomberg_FIX_Equity
-- Bloomberg_FIX_Commodities
-- Reuters_FX
-- Reuters_Equity
-- ICE_Energy
-- CME_Futures
-- Internal_Trade_System
-- Manual_Entry
-- FIX_Bridge_Primary
-- END MACRO

-- ============================================
-- CORE QUERY MACROS
-- ============================================

WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "Source = \"Bloomberg_FIX_FX\" and TradeDate = TradeDate(2025, 09, 21)"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer test-token-123',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
ORDER BY ExecutionTime DESC;

-- MACRO: TRADE_BASE
-- WITH WEB trades AS (
--     URL '${BASE_URL}/trades'
--     METHOD POST
--     BODY '{
--         "select": "${COLUMNS:*}",
--         "where": "${WHERE_CLAUSE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- END MACRO

-- MACRO: TODAY_TRADES
-- @TRADE_BASE
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- MACRO: YESTERDAY_TRADES
-- @TRADE_BASE
-- SELECT * FROM trades
-- WHERE Date(TradeDate) = Date('now', '-1 day')
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- MACRO: DATE_RANGE_TRADES
-- @TRADE_BASE
-- SELECT * FROM trades
-- WHERE TradeDate >= '${START_DATE}'
--   AND TradeDate <= '${END_DATE}'
-- ORDER BY TradeDate DESC, ExecutionTime DESC
-- END MACRO

-- ============================================
-- ANALYSIS MACROS
-- ============================================

-- MACRO: SOURCE_SUMMARY
-- @TRADE_BASE
-- SELECT
--     Source,
--     COUNT(*) as trade_count,
--     COUNT(DISTINCT Symbol) as unique_symbols,
--     SUM(Quantity * Price) as total_value,
--     AVG(Price) as avg_price,
--     MIN(ExecutionTime) as first_trade,
--     MAX(ExecutionTime) as last_trade
-- FROM trades
-- GROUP BY Source
-- ORDER BY total_value DESC
-- END MACRO

-- MACRO: SYMBOL_ANALYSIS
-- @TRADE_BASE
-- SELECT
--     Symbol,
--     Source,
--     COUNT(*) as trades,
--     SUM(Quantity) as total_quantity,
--     AVG(Price) as avg_price,
--     MIN(Price) as min_price,
--     MAX(Price) as max_price,
--     MAX(Price) - MIN(Price) as price_range
-- FROM trades
-- GROUP BY Symbol, Source
-- ORDER BY trades DESC
-- END MACRO

-- MACRO: TOP_TRADES
-- @TRADE_BASE
-- SELECT
--     TradeId,
--     Source,
--     Symbol,
--     Quantity,
--     Price,
--     Quantity * Price as total_value,
--     TradeDate,
--     Status
-- FROM trades
-- ORDER BY total_value DESC
-- LIMIT ${LIMIT:20}
-- END MACRO

-- ============================================
-- RECONCILIATION MACROS
-- ============================================

-- MACRO: RECONCILE_SOURCES
-- WITH WEB our_trades AS (
--     URL '${BASE_URL}/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "Source = \"${OUR_SOURCE}\" and TradeDate = ${TRADE_DATE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS ('Authorization': 'Bearer ${API_TOKEN}')
-- ),
-- WEB their_trades AS (
--     URL '${BASE_URL}/counterparty_trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "Source = \"${THEIR_SOURCE}\" and TradeDate = ${TRADE_DATE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS ('Authorization': 'Bearer ${API_TOKEN}')
-- )
-- SELECT
--     COALESCE(o.Symbol, t.Symbol) as Symbol,
--     o.TradeId as our_trade,
--     t.TradeId as their_trade,
--     o.Quantity as our_qty,
--     t.Quantity as their_qty,
--     o.Price as our_price,
--     t.Price as their_price,
--     CASE
--         WHEN o.TradeId IS NULL THEN '❌ MISSING (ours)'
--         WHEN t.TradeId IS NULL THEN '❌ MISSING (theirs)'
--         WHEN o.Quantity != t.Quantity THEN '⚠️ QTY MISMATCH'
--         WHEN ABS(o.Price - t.Price) > 0.01 THEN '⚠️ PRICE MISMATCH'
--         ELSE '✅ MATCHED'
--     END as status
-- FROM our_trades o
-- FULL OUTER JOIN their_trades t
--     ON o.Symbol = t.Symbol
--     AND o.TradeDate = t.TradeDate
-- WHERE o.TradeId IS NULL
--    OR t.TradeId IS NULL
--    OR o.Quantity != t.Quantity
--    OR ABS(o.Price - t.Price) > 0.01
-- ORDER BY Symbol
-- END MACRO

-- ============================================
-- QUICK SNIPPETS (for common filters)
-- ============================================

-- MACRO: WHERE_TODAY
-- TradeDate = Date('now')
-- END MACRO

-- MACRO: WHERE_YESTERDAY
-- TradeDate = Date('now', '-1 day')
-- END MACRO

-- MACRO: WHERE_THIS_WEEK
-- TradeDate >= Date('now', 'weekday 0', '-7 days')
-- END MACRO

-- MACRO: WHERE_FX
-- (Symbol LIKE '%/%' OR Symbol IN ('EURUSD', 'GBPUSD', 'USDJPY'))
-- END MACRO

-- MACRO: WHERE_EQUITY
-- Symbol IN ('AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA')
-- END MACRO

-- MACRO: WHERE_BLOOMBERG
-- Source LIKE 'Bloomberg%'
-- END MACRO

-- ============================================
-- EXAMPLE USAGE
-- ============================================

-- Example 1: Get today's trades for a specific source
-- Type: @TODAY_TRADES
-- Press: \sTe
-- Fill: WHERE_CLAUSE = Source = "Bloomberg_FIX_FX" and @WHERE_TODAY
-- Result: Expands to full query with today's trades

-- Example 2: Quick yesterday analysis
-- Type: @YESTERDAY_TRADES
-- Press: \sTe
-- Fill: WHERE_CLAUSE = Source = "${SOURCE}" and @WHERE_YESTERDAY

-- Example 3: Source reconciliation
-- Type: @RECONCILE_SOURCES
-- Press: \sTe
-- Fill: OUR_SOURCE, THEIR_SOURCE, TRADE_DATE (use date picker)

-- Example 4: Top trades by value
-- Type: @TOP_TRADES
-- Press: \sTe
-- Fill: WHERE_CLAUSE = @WHERE_TODAY, LIMIT = 10

-- ============================================
-- PRE-BUILT QUERIES (Ready to run with \sq)
-- ============================================

-- Get Bloomberg FX trades for today
WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "Source = \"Bloomberg_FIX_FX\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer ${JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT
    TradeId,
    Symbol,
    Quantity,
    Price,
    Quantity * Price as total_value,
    Status
FROM trades
ORDER BY total_value DESC
LIMIT 10;
GO

-- ============================================
-- KEYBINDING REFERENCE
-- ============================================
-- \sTe    - Expand macro at cursor
-- \sTq    - Quick substitute all ${VARIABLES}
-- \sTm    - List macros in this file
-- \sTd    - Quick date picker
-- \sTS    - Quick source picker
-- \sq     - Execute entire query
-- \ss     - Execute selection
-- \sn     - Table navigation mode
