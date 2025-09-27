-- Template: WEB_QUERY
-- Description: Web CTE with runtime parameters
-- Usage: Place cursor on @WEB_QUERY and press \ste
--        Then press \sx to execute with parameter prompts

-- Macro: TRADE_API_URL
-- Default: http://localhost:5001/trades

WITH WEB @{INPUT:Table name:trades_data} AS (
    URL '@{MACRO:TRADE_API_URL}'
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

