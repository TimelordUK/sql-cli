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
-- Usage Examples
-- ============================================

-- When you expand @WHERE_DEAL_TYPE:
-- 1. Picker(DEAL_TYPES) shows: Swap, NDF, NDS, Option, Spot, etc.
-- 2. DateTimePicker() shows date options

-- When you expand @WHERE_CURRENCY_PAIR:
-- 1. First Picker shows currencies with prompt "Base Currency"
-- 2. Second Picker shows currencies with prompt "Quote Currency"

-- The Picker function syntax:
-- Picker(MACRO_NAME)           - Uses macro name as prompt
-- Picker(MACRO_NAME, "Label")  - Uses custom label as prompt

-- You can define ANY list as a macro and use it with Picker():
-- - Product types
-- - Regions
-- - Traders
-- - Desks
-- - Instruments
-- - Anything you need!

-- ============================================
-- Complete Trade Query Example
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

-- To add a new picker:
-- 1. Define your list:
--    -- MACRO: YOUR_LIST
--    -- Option1
--    -- Option2
--    -- Option3
--    -- END MACRO
--
-- 2. Use it in a WHERE template:
--    -- MACRO: WHERE_YOUR_FILTER
--    -- YourColumn = "Picker(YOUR_LIST)"
--    -- END MACRO
--
-- That's it! No Lua changes needed!