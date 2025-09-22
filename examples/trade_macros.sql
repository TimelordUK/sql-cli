-- SQL-CLI Macro System Example
-- Define macros in comment blocks, then use them with @MACRO or ${MACRO}

-- MACRO: TRADE_BASE
-- WITH WEB trades AS (
--     URL 'http://localhost:5001/trades'
--     METHOD POST
--     BODY '{
--         "select": "*",
--         "where": "${WHERE_CLAUSE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
-- )
-- END MACRO

-- MACRO: TODAY_TRADES
-- WITH WEB trades AS (
--     URL 'http://localhost:5001/trades'
--     METHOD POST
--     BODY '{
--         "select": "TradeId, Source, Symbol, Quantity, Price, TradeDate, Status",
--         "where": "Source = \\"${SOURCE}\\" and TradeDate = TradeDate(2025, 9, 22)"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
-- )
-- SELECT * FROM trades
-- END MACRO

-- MACRO: DATE_RANGE
-- TradeDate >= TradeDate(${START_YEAR}, ${START_MONTH}, ${START_DAY}) and TradeDate <= TradeDate(${END_YEAR}, ${END_MONTH}, ${END_DAY})
-- END MACRO

-- MACRO: SOURCE_FILTER
-- Source = \\"${SOURCE:Bloomberg_FIX_FX}\\"
-- END MACRO

-- ============================================
-- EXAMPLE USAGE
-- ============================================

-- Example 1: Type @TODAY_TRADES and press \sTe to expand
-- It will prompt for SOURCE and then expand the full CTE

-- Example 2: Build a query using multiple macros
-- Type: @TRADE_BASE
-- Press \sTe - it will prompt for WHERE_CLAUSE
-- You can enter: @SOURCE_FILTER (which will expand to Source = \"Bloomberg_FIX_FX\")

-- Example 3: Inline variable substitution
-- Write your query with ${VARIABLES} and press \sTq to substitute all at once:

WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "select": "${COLUMNS:*}",
        "where": "Source = \\"${SOURCE}\\" and TradeDate = TradeDate(${YEAR}, ${MONTH}, ${DAY})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    *
FROM trades
WHERE Symbol LIKE '${SYMBOL_PATTERN:EUR%}';
GO

-- Example 4: Use \sTm to see all macros defined in this file and insert one

-- ============================================
-- KEYBINDINGS
-- ============================================
-- \sT     - Select from pre-defined templates
-- \sTq    - Quick substitute all ${VARIABLES} in query
-- \sTe    - Expand macro at cursor (@MACRO or ${MACRO})
-- \sTm    - List and insert macros from this file
-- \sTs    - Save current query as a template
-- \sTd    - Insert quick date
-- \sTS    - Insert trade source

-- ============================================
-- TIPS
-- ============================================
-- 1. Define frequently used CTEs as macros at the top of your file
-- 2. Use ${VAR:default} syntax to provide defaults
-- 3. Macros can reference other macros
-- 4. Special variables like ${SOURCE} and ${DATE} get smart pickers
-- 5. The system auto-escapes quotes in JSON bodies