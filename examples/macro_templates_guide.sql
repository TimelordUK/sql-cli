-- ============================================
-- SQL-CLI Macro Templates Guide
-- ============================================
-- The definitive guide for creating WHERE templates
-- with custom pickers - no Lua modifications needed!

-- MACRO: CONFIG
-- BASE_URL = http://tradeapi
-- API_TOKEN = ${JWT_TOKEN}
-- END MACRO

-- ============================================
-- Define Your Choice Lists
-- ============================================

-- MACRO: SOURCES
-- Bloomberg_FIX_FX
-- Bloomberg_FIX_Equity
-- Reuters_FX
-- Internal_Trade_System
-- END MACRO

-- MACRO: DEAL_TYPES
-- Swap
-- NDF
-- NDS
-- Option
-- Spot
-- Forward
-- Future
-- CFD
-- END MACRO

-- MACRO: CURRENCIES
-- USD
-- EUR
-- GBP
-- JPY
-- CHF
-- AUD
-- CAD
-- CNY
-- END MACRO

-- MACRO: STATUSES
-- New
-- Pending
-- Confirmed
-- Settled
-- Cancelled
-- Failed
-- END MACRO

-- MACRO: ACCOUNTS
-- Account_001
-- Account_002
-- Account_003
-- Trading_Desk_A
-- Trading_Desk_B
-- Market_Maker_1
-- END MACRO

-- ============================================
-- WHERE Templates Using Generic Picker
-- ============================================

-- Standard WHERE templates for regular SQL contexts
-- MACRO: WHERE_DEAL_TYPE
-- DealType = "Picker(DEAL_TYPES)" and TradeDate = DateTimePicker()
-- END MACRO

-- MACRO: WHERE_CURRENCY_PAIR
-- BaseCurrency = "Picker(CURRENCIES, Base Currency)" and QuoteCurrency = "Picker(CURRENCIES, Quote Currency)"
-- END MACRO

-- MACRO: WHERE_STATUS_CHECK
-- Status = "Picker(STATUSES)" and Account = "Picker(ACCOUNTS)"
-- END MACRO

-- MACRO: WHERE_COMPLEX_TRADE
-- CTGSource = "Picker(SOURCES)" and DealType = "Picker(DEAL_TYPES)" and Status != "Picker(STATUSES, Exclude Status)"
-- END MACRO

-- MACRO: WHERE_CUSTOM_FILTER
-- DealType IN ("Picker(DEAL_TYPES, First Type)", "Picker(DEAL_TYPES, Second Type)") and TradeDate = DateTimePicker()
-- END MACRO

-- ============================================
-- JSON WHERE Templates (with escaped quotes)
-- ============================================
-- Use these when the WHERE clause is inside a JSON BODY

-- MACRO: WHERE_DEAL_TYPE_JSON
-- DealType = \"Picker(DEAL_TYPES)\" and TradeDate = DateTimePicker()
-- END MACRO

-- MACRO: WHERE_CURRENCY_PAIR_JSON
-- BaseCurrency = \"Picker(CURRENCIES, Base Currency)\" and QuoteCurrency = \"Picker(CURRENCIES, Quote Currency)\"
-- END MACRO

-- MACRO: WHERE_STATUS_CHECK_JSON
-- Status = \"Picker(STATUSES)\" and Account = \"Picker(ACCOUNTS)\"
-- END MACRO

-- MACRO: WHERE_COMPLEX_TRADE_JSON
-- CTGSource = \"Picker(SOURCES)\" and DealType = \"Picker(DEAL_TYPES)\" and Status != \"Picker(STATUSES, Exclude Status)\"
-- END MACRO

-- ============================================
-- Input-based WHERE Templates (NEW SYNTAX)
-- ============================================
-- Using the cleaner @{INPUT:prompt} syntax

-- MACRO: WHERE_ORDER_ID_NEW
-- PlatformOrderId = "@{INPUT:Enter Order ID}"
-- END MACRO

-- MACRO: WHERE_ORDER_ID_JSON_NEW
-- PlatformOrderId = \"@{INPUT:Enter Order ID}\"
-- END MACRO

-- MACRO: WHERE_MIXED_NEW
-- DealType = \"@{PICKER:DEAL_TYPES:Select Deal Type}\" and PlatformOrderId.StartsWith(\"@{INPUT:Order prefix}\") and TradeDate = @{DATE:Trade Date}
-- END MACRO

-- ============================================
-- Input-based WHERE Templates (OLD SYNTAX)
-- ============================================
-- Use Input() function for free-form text entry

-- MACRO: WHERE_ORDER_ID
-- PlatformOrderId = "Input(Enter Order ID)"
-- END MACRO

-- MACRO: WHERE_ORDER_ID_JSON
-- PlatformOrderId = \"Input(Enter Order ID)\"
-- END MACRO

-- MACRO: WHERE_ORDER_STARTS_WITH
-- PlatformOrderId.StartsWith("Input(Order ID prefix)")
-- END MACRO

-- MACRO: WHERE_ORDER_STARTS_WITH_JSON
-- PlatformOrderId.StartsWith(\"Input(Order ID prefix)\")
-- END MACRO

-- MACRO: WHERE_AMOUNT_RANGE
-- Amount >= Input(Min Amount, 0) and Amount <= Input(Max Amount, 1000000)
-- END MACRO

-- MACRO: WHERE_AMOUNT_RANGE_JSON
-- Amount >= Input(Min Amount, 0) and Amount <= Input(Max Amount, 1000000)
-- END MACRO

-- MACRO: WHERE_CUSTOM_SEARCH_JSON
-- DealType = \"Picker(DEAL_TYPES)\" and PlatformOrderId.Contains(\"Input(Search term)\")
-- END MACRO

-- MACRO: WHERE_MIXED_FILTERS_JSON
-- CTGSource = \"Picker(SOURCES)\" and PlatformOrderId = \"Input(Enter Order ID)\" and TradeDate = DateTimePicker()
-- END MACRO

-- ============================================
-- Usage Examples
-- ============================================

-- When you expand @WHERE_DEAL_TYPE:
-- 1. Picker(DEAL_TYPES) shows: Swap, NDF, NDS, Option, Spot, etc.
-- 2. DateTimePicker() shows date options

-- When you expand @WHERE_CURRENCY_PAIR:
-- 1. First Picker shows currencies with prompt "Base Currency"
-- 2. Second Picker shows currencies with prompt "Quote Currency"

-- ============================================
-- Function Syntax Options
-- ============================================

-- NEW SIMPLER SYNTAX (Recommended):
-- @{INPUT:prompt}              - Prompts for text input (raw value)
-- @{JINPUT:prompt}             - Prompts for text input (JSON escaped with \")
-- @{PICKER:macro:prompt}       - Shows list from macro with custom prompt
-- @{JPICKER:macro:prompt}      - Shows list from macro (JSON escaped with \")
-- @{DATE:prompt}               - Shows date picker
-- @{VAR:variable_name}         - Substitutes variable from CONFIG or environment
-- @{JVAR:variable_name}        - Substitutes variable (JSON escaped with \")
-- @{MACRO:macro_name}          - Expands another macro (recursive expansion supported)

-- Examples for JSON contexts (inside BODY):
-- "where": "OrderId = @{JINPUT:Enter Order ID}"          -- Returns: \"value\"
-- "where": "Status = @{JPICKER:STATUSES:Select Status}"  -- Returns: \"value\"
-- "where": "TradeDate = @{DATE:Select Trade Date}"       -- Returns: DateTime(...)
-- URL '@{VAR:BASE_URL}/api'                              -- Returns: http://tradeapi/api
-- 'Authorization': 'Bearer @{VAR:API_TOKEN}'             -- Returns: Bearer <token>

-- Examples for regular SQL:
-- WHERE OrderId = '@{INPUT:Enter Order ID}'              -- Returns: value
-- WHERE Status = '@{PICKER:STATUSES:Select Status}'      -- Returns: value
-- SELECT * FROM @{VAR:TABLE_NAME}                        -- Returns: table_name

-- ORIGINAL FUNCTION SYNTAX (Still supported):
-- Picker(MACRO_NAME)           - Shows list from MACRO_NAME, uses macro name as prompt
-- Picker(MACRO_NAME, "Label")  - Shows list from MACRO_NAME, uses custom label
-- Input("Prompt")              - Prompts for free-form text input
-- Input("Prompt", "default")   - Prompts with a default value
-- DateTimePicker()             - Shows date picker, returns DateTime(YYYY, M, D)
-- DateTimePicker("Label")      - Shows date picker with custom label

-- You can define ANY list as a macro and use it with Picker():
-- - Product types
-- - Regions
-- - Traders
-- - Desks
-- - Instruments
-- - Anything you need!

-- ============================================
-- Complete Trade Query Examples
-- ============================================

-- MACRO: TRADE_QUERY_FLEXIBLE
-- WITH WEB trades AS (
--     URL '${BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "${COLUMNS}",
--         "where": "${WHERE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Example of using a JSON WHERE macro directly in a query
-- MACRO: TRADE_QUERY_WITH_PICKER
-- WITH WEB trades AS (
--     URL '${BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "DealId,PlatformOrderId,DealType,SignedQuantity,BuySell",
--         "where": "@WHERE_DEAL_TYPE_JSON"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Example using Input() for order ID search
-- Method 1: Using @{JINPUT:name} for JSON contexts (BEST FOR JSON)
-- MACRO: TRADE_QUERY_BY_ORDER_SIMPLE
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "PlatformOrderId.StartsWith(@{JINPUT:Order ID prefix})"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer @{VAR:API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Method 2: Previous attempt with @{INPUT:name} in quotes
-- MACRO: TRADE_QUERY_BY_ORDER_V2
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "PlatformOrderId.StartsWith(\"@{INPUT:Order ID prefix}\")"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Method 2: Using template variables with prompts
-- MACRO: TRADE_QUERY_BY_ORDER_V3
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "PlatformOrderId.StartsWith(\"${ORDER_PREFIX?Enter Order ID prefix}\")"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Original version (for compatibility)
-- MACRO: TRADE_QUERY_BY_ORDER
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "PlatformOrderId.StartsWith(\"Input(Order ID prefix)\")"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- ============================================
-- Complete Example with New Syntax
-- ============================================
-- This shows all the new @{} syntax features

-- MACRO: TRADE_SEARCH_ADVANCED
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "DealType = @{JPICKER:DEAL_TYPES:Select Deal Type} and PlatformOrderId.StartsWith(@{JINPUT:Order ID prefix}) and TradeDate >= @{DATE:Start Date} and TradeDate <= @{DATE:End Date}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer @{VAR:API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- WHERE Amount > @{INPUT:Minimum Amount:0}
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- When you expand @TRADE_SEARCH_ADVANCED, it will:
-- 1. Show picker for Deal Type (from DEAL_TYPES list)
-- 2. Prompt for Order ID prefix
-- 3. Show date picker for Start Date
-- 4. Show date picker for End Date
-- 5. Prompt for Minimum Amount (with default of 0)

-- To add a new picker:
-- 1. Define your list:
--    -- MACRO: YOUR_LIST
--    -- Option1
--    -- Option2
--    -- Option3
--    -- END MACRO
--
-- 2. Use it in a WHERE template with new syntax:
--    -- MACRO: WHERE_YOUR_FILTER
--    -- YourColumn = \"@{PICKER:YOUR_LIST:Select Value}\"
--    -- END MACRO
--
-- That's it! No Lua changes needed!

-- ============================================
-- Nested Macro Expansion Example
-- ============================================

-- Define reusable WHERE clause components:
-- MACRO: WHERE_DATE_TODAY
-- TradeDate = DateTime(2025, 9, 24)
-- END MACRO

-- MACRO: WHERE_SOURCE_BLOOMBERG
-- Source = "Bloomberg"
-- END MACRO

-- Combine them into a complex WHERE clause:
-- MACRO: WHERE_BLOOMBERG_TODAY
-- @{MACRO:WHERE_SOURCE_BLOOMBERG} and @{MACRO:WHERE_DATE_TODAY}
-- END MACRO

-- Use in a query with variable substitution:
-- MACRO: QUERY_BLOOMBERG_TODAY
-- WITH WEB trades AS (
--     URL '@{VAR:BASE_URL}/query/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "@{MACRO:WHERE_BLOOMBERG_TODAY}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer @{VAR:API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- SELECT * FROM trades
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- When you expand @QUERY_BLOOMBERG_TODAY:
-- 1. @{VAR:BASE_URL} and @{VAR:API_TOKEN} are replaced from CONFIG
-- 2. @{MACRO:WHERE_BLOOMBERG_TODAY} is expanded
-- 3. Which recursively expands @{MACRO:WHERE_SOURCE_BLOOMBERG} and @{MACRO:WHERE_DATE_TODAY}
-- 4. Final result has all macros and variables fully expanded
