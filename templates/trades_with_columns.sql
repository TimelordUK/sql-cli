-- SQL-CLI Template File with Column Macros
-- Define reusable column sets and use them in templates

-- ============================================
-- Column Set Definitions
-- ============================================

-- Macro: COLUMNS_BASIC
-- DealId, PlatformOrderId, DealType, SignedQuantity, BuySell, ExecutionTime
-- END MACRO

-- Macro: COLUMNS_PRICING
-- DealId, Symbol, Price, SignedQuantity, Amount, Currency, SpotRate, AllInRate
-- END MACRO

-- Macro: COLUMNS_FULL
-- DealId, PlatformOrderId, DealType, Source, Symbol, SignedQuantity, BuySell, Price, Amount, Currency, ExecutionTime, TradeDate, SettlementDate, Status, Account, Broker, Commission, SpotRate, AllInRate, CounterpartyId
-- END MACRO

-- Macro: COLUMNS_RECONCILIATION
-- DealId, PlatformOrderId, Source, Status, SignedQuantity, Price, Amount, ExecutionTime, MatchStatus, BreakReason
-- END MACRO

-- Macro: COLUMNS_RISK
-- DealId, Symbol, DealType, SignedQuantity, Price, Amount, Currency, Delta, Gamma, Vega, Theta, SpotRate
-- END MACRO

-- ============================================
-- Templates Using Column Macros
-- ============================================

-- Template: FETCH_TRADES_BASIC
-- Description: Fetch trades with basic columns
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{MACRO:COLUMNS_BASIC}",
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

-- Template: FETCH_TRADES_CUSTOM
-- Description: Fetch trades with selectable column set
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{PICKER:COLUMN_SETS:Select Column Set}",
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

-- Template: FETCH_TRADES_MIXED
-- Description: Fetch with standard columns plus custom additions
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{MACRO:COLUMNS_BASIC}, @{INPUT:Additional Columns (comma-separated):}",
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

-- Template: RECONCILIATION_QUERY
-- Description: Reconciliation query with specific columns
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{MACRO:COLUMNS_RECONCILIATION}",
        "where": "Source = @{JPICKER:SOURCES:Source} and Status = @{JPICKER:STATUSES:Status}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
WHERE MatchStatus IS NULL OR MatchStatus = 'BREAK'
ORDER BY ExecutionTime DESC;

-- Template: RISK_REPORT
-- Description: Risk metrics for positions
WITH WEB positions AS (
    URL '@{VAR:BASE_URL}/query/positions'
    METHOD POST
    BODY '{
        "select": "@{MACRO:COLUMNS_RISK}",
        "where": "Symbol = @{INPUT:Symbol:} and DealType IN (@{JPICKER:DEAL_TYPES:Deal Type 1}, @{JPICKER:DEAL_TYPES:Deal Type 2})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT
    Symbol,
    DealType,
    SUM(SignedQuantity * Delta) as TotalDelta,
    SUM(SignedQuantity * Gamma) as TotalGamma,
    SUM(SignedQuantity * Vega) as TotalVega
FROM positions
GROUP BY Symbol, DealType;

-- ============================================
-- Column Set Picker List
-- ============================================

-- Macro: COLUMN_SETS
-- @{MACRO:COLUMNS_BASIC}
-- @{MACRO:COLUMNS_PRICING}
-- @{MACRO:COLUMNS_FULL}
-- @{MACRO:COLUMNS_RECONCILIATION}
-- @{MACRO:COLUMNS_RISK}
-- *
-- END MACRO

-- ============================================
-- Supporting Macros
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
-- Option
-- Spot
-- Forward
-- END MACRO

-- Macro: STATUSES
-- New
-- Pending
-- Confirmed
-- Settled
-- Cancelled
-- END MACRO

-- Macro: CONFIG
-- BASE_URL = http://localhost:8080
-- JWT_TOKEN = @{ENV:JWT_TOKEN}
-- END MACRO

-- Macro: WHERE_CLAUSE
-- Source = @{JPICKER:SOURCES:Source} and DealType = @{JPICKER:DEAL_TYPES:Deal Type}
-- END MACRO