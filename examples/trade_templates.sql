-- SQL-CLI Template Examples for Trade Fetching
-- #! ../data/trades.json

-- Example 1: Basic template with variables
-- Use \sT to apply a template
-- Variables: ${SOURCE}, ${TRADE_DATE}, ${LIMIT:100}

WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "Where": "Source = \"${SOURCE}\"",
        "Limit": ${LIMIT:100}
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades;
GO

-- Example 2: Template with date range
-- Variables can have defaults using ${VAR:default}

WITH WEB trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    METHOD POST
    BODY '{
        "Where": "TradeDate >= TradeDate(${START_YEAR}, ${START_MONTH}, ${START_DAY}) and TradeDate <= TradeDate(${END_YEAR}, ${END_MONTH}, ${END_DAY})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    Source,
    COUNT(*) as trade_count,
    SUM(Quantity * Price) as total_value
FROM trades
GROUP BY Source;
GO

-- Example 3: Using environment variables
-- ${JWT_TOKEN} will be replaced with env var

WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "Where": "Source = \"Bloomberg_FIX_FX\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer ${JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT * FROM trades
WHERE Symbol LIKE '${SYMBOL_PATTERN:EUR%}';
GO

-- Template Usage Instructions:
-- 1. \sT     - Select and apply a template
-- 2. \sTq    - Quick substitute variables in current query
-- 3. \sTs    - Save current query as a template
-- 4. \sTd    - Insert a quick date
-- 5. \sTS    - Insert a trade source

-- The template system will:
-- - Prompt for each variable found in the query
-- - Remember common values like sources
-- - Automatically escape quotes in JSON bodies
-- - Support date pickers and source selectors
-- - Allow saving frequently used queries as templates