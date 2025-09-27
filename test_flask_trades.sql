-- Test Flask Server with Web CTE
-- This tests the Flask demo server at localhost:5001

WITH WEB trades_data AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer test-token',
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
FROM trades_data
LIMIT 5;