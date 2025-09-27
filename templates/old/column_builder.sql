-- SQL-CLI Column Builder Templates
-- Dynamic column selection with composition

-- ============================================
-- Core Column Groups (Building Blocks)
-- ============================================

-- Macro: COLS_ID
-- DealId, PlatformOrderId
-- END MACRO

-- Macro: COLS_TRADE_BASIC
-- DealType, Source, Symbol
-- END MACRO

-- Macro: COLS_QUANTITY
-- SignedQuantity, BuySell
-- END MACRO

-- Macro: COLS_PRICING
-- Price, Amount, Currency, SpotRate, AllInRate
-- END MACRO

-- Macro: COLS_DATES
-- ExecutionTime, TradeDate, SettlementDate
-- END MACRO

-- Macro: COLS_STATUS
-- Status, MatchStatus, BreakReason
-- END MACRO

-- Macro: COLS_PARTIES
-- Account, Broker, CounterpartyId
-- END MACRO

-- Macro: COLS_COSTS
-- Commission, Fees, Tax
-- END MACRO

-- Macro: COLS_RISK
-- Delta, Gamma, Vega, Theta, Rho
-- END MACRO

-- ============================================
-- Composite Column Sets (Combinations)
-- ============================================

-- Macro: COLUMNS_STANDARD
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}, @{MACRO:COLS_QUANTITY}, @{MACRO:COLS_PRICING}, @{MACRO:COLS_DATES}
-- END MACRO

-- Macro: COLUMNS_RECONCILIATION
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}, @{MACRO:COLS_STATUS}, @{MACRO:COLS_QUANTITY}, @{MACRO:COLS_PRICING}
-- END MACRO

-- Macro: COLUMNS_ACCOUNTING
-- @{MACRO:COLS_ID}, @{MACRO:COLS_PARTIES}, @{MACRO:COLS_PRICING}, @{MACRO:COLS_COSTS}, @{MACRO:COLS_DATES}
-- END MACRO

-- ============================================
-- Templates with Flexible Column Selection
-- ============================================

-- Template: BUILD_QUERY
-- Description: Build query with interactive column selection
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{PICKER:COLUMN_PRESETS:Choose Preset or Custom}",
        "where": "@{INPUT:WHERE clause:1=1}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades;

-- Template: MULTI_COLUMN_SELECT
-- Description: Select multiple column groups
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{PICKER:PRIMARY_COLS:Primary Columns}, @{PICKER:SECONDARY_COLS:Additional Columns}",
        "where": "@{INPUT:WHERE clause:1=1}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades;

-- Template: CONDITIONAL_COLUMNS
-- Description: Different columns based on query type
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "@{PICKER:QUERY_TYPE_COLUMNS:Query Type}",
        "where": "@{MACRO:WHERE_CLAUSE}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades;

-- ============================================
-- Column Selection Lists
-- ============================================

-- Macro: COLUMN_PRESETS
-- @{MACRO:COLUMNS_STANDARD}
-- @{MACRO:COLUMNS_RECONCILIATION}
-- @{MACRO:COLUMNS_ACCOUNTING}
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}
-- @{MACRO:COLS_ID}, @{MACRO:COLS_PRICING}
-- @{INPUT:Custom column list:*}
-- *
-- END MACRO

-- Macro: PRIMARY_COLS
-- @{MACRO:COLS_ID}
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}, @{MACRO:COLS_QUANTITY}
-- END MACRO

-- Macro: SECONDARY_COLS
-- @{MACRO:COLS_PRICING}
-- @{MACRO:COLS_DATES}
-- @{MACRO:COLS_STATUS}
-- @{MACRO:COLS_PARTIES}
-- @{MACRO:COLS_COSTS}
-- @{MACRO:COLS_RISK}
-- END MACRO

-- Macro: QUERY_TYPE_COLUMNS
-- @{MACRO:COLUMNS_STANDARD}::Standard Query
-- @{MACRO:COLUMNS_RECONCILIATION}::Reconciliation
-- @{MACRO:COLUMNS_ACCOUNTING}::Accounting Report
-- @{MACRO:COLS_ID}, @{MACRO:COLS_RISK}::Risk Analysis
-- @{MACRO:COLS_ID}, @{MACRO:COLS_PRICING}, @{MACRO:COLS_DATES}::Pricing Report
-- END MACRO

-- ============================================
-- Helper Macros
-- ============================================

-- Macro: CONFIG
-- BASE_URL = http://localhost:8080
-- JWT_TOKEN = @{ENV:JWT_TOKEN}
-- END MACRO

-- Macro: WHERE_CLAUSE
-- TradeDate = @{DATE:Trade Date}
-- END MACRO

-- ============================================
-- Usage Examples (Comments)
-- ============================================

-- Example 1: When you expand COLUMNS_STANDARD, it becomes:
-- DealId, PlatformOrderId, DealType, Source, Symbol, SignedQuantity, BuySell, Price, Amount, Currency, SpotRate, AllInRate, ExecutionTime, TradeDate, SettlementDate

-- Example 2: You can nest macros arbitrarily deep:
-- @{MACRO:COLUMNS_STANDARD} expands to:
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}, ... which then expands to:
-- DealId, PlatformOrderId, DealType, Source, Symbol, ...

-- Example 3: Mix macros with literal columns:
-- "select": "@{MACRO:COLS_ID}, CustomField1, @{MACRO:COLS_PRICING}, CustomField2"