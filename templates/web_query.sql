-- Template: WEB_QUERY
-- Description: Web CTE with runtime parameters

WITH WEB @{INPUT:Table name:trades_data} AS (
    URL '@{INPUT:API URL:http://localhost:5001/trades}'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
    BODY '{
        "Select": "Source,PlatformOrderId,BloomberTicker,SignedQuantity,BuySell,Price",
        "Where": "Source = \"{{SOURCE}}\" AND TradeDate = DateTime({{DATE}})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT *
FROM @{INPUT:Table name:trades_data}

