-- Demo: Parameter System with Web CTE
-- This demonstrates using templates with runtime parameters

-- Define some common parameter sets for quick selection
-- @PARAMS: bloomberg_today = { SOURCE: "Bloomberg", DATE: "2024-03-27", STATUS: "FILLED" }
-- @PARAMS: reuters_pending = { SOURCE: "Reuters", DATE: "2024-03-27", STATUS: "PENDING" }
-- @PARAMS: tradeweb_all = { SOURCE: "TradeWeb", DATE: "2024-03-26", STATUS: "ALL" }

-- Step 1: First expand the template below by placing cursor on @WEB_QUERY and pressing \sre
-- You'll be prompted for:
--   TABLE_NAME: trades_data
--   URL: https://api.example.com/trades
--   AUTH_TOKEN: demo-token-123

-- Usage: Place cursor on @WEB_QUERY and press \ste
--        Then press \sx to execute with parameter prompts

WITH WEB trades_data AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer ',
        'Content-Type': 'application/json'
    )
    BODY '{
        "Select": "Source,PlatformOrderId,BloomberTicker,SignedQuantity,BuySell,Price",
        "Where": "Source = \"Bloomberg\" AND TradeDate = DateTime(2025-09-27)"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT *
FROM trades_data;


-- Step 2: After expansion, you'll have a query with {{SOURCE}}, {{DATE}}, {{STATUS}} parameters
-- Run with \sx and you'll be prompted for each parameter value

-- Step 3: Try these workflows:
-- a) Select "bloomberg_today" from parameter sets
-- b) Or pick SOURCE from the list: Bloomberg, Reuters, TradeWeb, etc.
-- c) Run again and see your previous values in history

-- After the CTE expansion, add your query:
SELECT
    source,
    COUNT(*) as total_trades,
    SUM(quantity) as total_quantity
FROM trades_data
GROUP BY source;

-- Quick Demo without expansion (ready to run):
-- Try this simpler example that's ready to go:

WITH demo_data AS (
    SELECT
        '{{SOURCE}}' as source,
        '{{DATE}}' as trade_date,
        '{{STATUS}}' as status,
        100 as quantity,
        50.25 as price
    UNION ALL
    SELECT
        '{{SOURCE}}' as source,
        '{{DATE}}' as trade_date,
        'FILLED' as status,
        200 as quantity,
        51.00 as price
    UNION ALL
    SELECT
        'Other Source' as source,
        '2024-03-26' as trade_date,
        'CANCELLED' as status,
        150 as quantity,
        49.75 as price
)
SELECT
    source,
    status,
    COUNT(*) as trades,
    SUM(quantity) as total_qty,
    AVG(price) as avg_price
FROM demo_data
GROUP BY source, status
ORDER BY source, status;
