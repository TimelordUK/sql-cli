-- SQL-CLI Template File
-- This file uses SQL comments for compatibility with syntax highlighting

-- Template: FETCH_TRADES
-- Description: Fetch trades from API with flexible WHERE clause
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "@{MACRO:WHERE_CLAUSE}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
ORDER BY ExecutionTime DESC;

-- Template: TRADE_BY_DATE_RANGE
-- Description: Get trades for specific date range
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "TradeDate >= @{DATE:Start Date} and TradeDate <= @{DATE:End Date}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
ORDER BY TradeDate DESC;

-- Template: TRADE_BY_SOURCE
-- Description: Filter trades by source with amount threshold
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "Source = @{JPICKER:SOURCES:Select Source} and Amount > @{INPUT:Minimum Amount:1000}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
ORDER BY Amount DESC;

-- Template: TRADE_COMPLEX
-- Description: Complex trade query with multiple filters
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{INPUT:Columns (* for all):*}",
        "where": "Source = @{JPICKER:SOURCES:Source} and DealType = @{JPICKER:DEAL_TYPES:Deal Type} and Status = @{JPICKER:STATUSES:Status}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
WHERE Amount > @{INPUT:Local Filter Amount:0}
ORDER BY @{PICKER:ORDER_COLUMNS:Order By} DESC;

-- ============================================
-- Macro Definitions (Lists for PICKER)
-- ============================================

-- Macro: SOURCES
-- Bloomberg_FIX_FX
-- Bloomberg_FIX_Equity
-- Reuters_FX
-- Internal_Trade_System
-- END MACRO

-- Macro: DEAL_TYPES
-- Swap
-- NDF
-- NDS
-- Option
-- Spot
-- Forward
-- Future
-- CFD
-- END MACRO

-- Macro: STATUSES
-- New
-- Pending
-- Confirmed
-- Settled
-- Cancelled
-- Failed
-- END MACRO

-- Macro: ORDER_COLUMNS
-- TradeDate
-- ExecutionTime
-- Amount
-- DealId
-- PlatformOrderId
-- SignedQuantity
-- END MACRO

-- Macro: CONFIG
-- BASE_URL = http://localhost:8080
-- JWT_TOKEN = @{ENV:JWT_TOKEN}
-- END MACRO

-- ============================================
-- Reusable WHERE Clauses
-- ============================================

-- Macro: WHERE_BLOOMBERG_TODAY
-- Source = "Bloomberg" and TradeDate = DateTime(2025, 9, 25)
-- END MACRO

-- Macro: WHERE_ACTIVE
-- Status IN ("New", "Pending", "Confirmed") and Amount > 0
-- END MACRO

-- Macro: WHERE_CLAUSE
-- @{MACRO:WHERE_ACTIVE} and @{MACRO:WHERE_SOURCE}
-- END MACRO

-- Macro: WHERE_SOURCE
-- Source = @{JPICKER:SOURCES:Select Source}
-- END MACRO