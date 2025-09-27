-- Macro: TRADE_API_URL
-- http://localhost:5001/trades
-- END MACRO

-- Macro: JWT_TOKEN_DEFAULT
-- test-token-123
-- END MACRO

-- Template: WEB_QUERY
-- Description: Web CTE with runtime parameters
-- Usage: Place cursor on @WEB_QUERY and press \ste
--        Then press \sx to execute with parameter prompts

WITH WEB @{INPUT:Table name:trades_data} AS (
    URL '@{MACRO:TRADE_API_URL}'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer @{MACRO:JWT_TOKEN_DEFAULT}',
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
FROM @{INPUT:Table name:trades_data};
GO

