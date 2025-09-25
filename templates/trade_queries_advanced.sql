-- Advanced Trade Query Templates with WHERE clause builders
-- This file demonstrates escape patterns and dynamic WHERE clauses

-- =====================================
-- CONFIGURATION VARIABLES
-- =====================================

-- Macro: SOURCES
-- Bloomberg
-- Reuters
-- Internal
-- END MACRO

-- Macro: WHERE_BUILDER
-- Source = @{JPICKER:SOURCES:Select data source} AND TradeDate = @{DATE:Select trade date}
-- END MACRO

-- Macro: WHERE_BLOOMBERG_TODAY
-- Source = \"Bloomberg\" AND TradeDate = DateTime(2025, 9, 25)
-- END MACRO

-- =====================================
-- TEMPLATES
-- =====================================

-- Template: FETCH_TRADES_DYNAMIC
-- Description: Fetch trades with dynamic WHERE clause
WITH web_trades AS (
    URL 'http://localhost:8080/query/trades'
    METHOD POST
    BODY '{
        "select": "DealId, PlatformOrderId, DealType, SignedQuantity, BuySell, ExecutionTime",
        "where": "@{MACRO:WHERE_BUILDER}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer $$@{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM web_trades
ORDER BY ExecutionTime DESC;
-- END TEMPLATE

-- Template: FETCH_TRADES_LITERAL
-- Description: Fetch trades with literal token placeholder (not expanded)
WITH web_trades AS (
    URL 'http://localhost:8080/query/trades'
    METHOD POST
    BODY '{
        "select": "DealId, PlatformOrderId, SignedQuantity",
        "where": "@{MACRO:WHERE_BLOOMBERG_TODAY}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer $$@{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM web_trades;
-- END TEMPLATE

-- Template: BUILD_WHERE_CLAUSE
-- Description: Interactive WHERE clause builder
-- @{PICKER:SOURCES:Select Source} = '@{INPUT:Enter source value}'
-- AND TradeDate = @{DATE:Select trade date}
-- AND Amount > @{INPUT:Enter minimum amount:1000}
-- END TEMPLATE

-- =====================================
-- USAGE EXAMPLES
-- =====================================

-- Example 1: Dynamic WHERE with source/date pickers
-- Type: @FETCH_TRADES_DYNAMIC
-- Press: \ste
-- You'll be prompted for:
--   1. Source selection (Bloomberg/Reuters/Internal)
--   2. Date selection
-- The WHERE clause will be built as:
--   Source = "Bloomberg" AND TradeDate = DateTime(2025, 9, 25)

-- Example 2: Keep JWT token literal (not expanded)
-- Type: @FETCH_TRADES_LITERAL
-- Press: \ste
-- The $$@{VAR:JWT_TOKEN} will remain as @{VAR:JWT_TOKEN} in the output

-- Example 3: Build a custom WHERE clause
-- Type: @BUILD_WHERE_CLAUSE
-- Press: \ste
-- You'll be prompted for each component of the WHERE clause

-- =====================================
-- ESCAPE PATTERNS
-- =====================================

-- Use $$ to escape variables that should NOT be expanded:
-- $$@{VAR:NAME}     -> @{VAR:NAME} (literal)
-- @{VAR:NAME}       -> expanded value

-- Use @{LITERAL:content} to preserve content as-is:
-- @{LITERAL:@{VAR:SECRET}} -> @{VAR:SECRET}