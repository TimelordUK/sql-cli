-- Template: WEB_QUERY
-- Description: Web CTE with runtime parameters

WITH WEB @{INPUT:Table name:trades_data} AS (
    URL 'https://api.trading.com/trades'
    METHOD POST
    HEADERS (
        "Authorization": "Bearer @{VAR:JWT_TOKEN}",
        "Content-Type": "application/json"
    )
    BODY '{
        "Select": "Source,PlatformOrderId,BloomberTicker,SignedQuantity,BuySell,Price",
        "Where": "Source = \"{{SOURCE}}\" AND TradeDate = DateTime({{DATE}})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
FROM trades_data
GO

