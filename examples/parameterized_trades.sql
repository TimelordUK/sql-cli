-- [TEST:SKIP] illustrative -- {{PARAM}} templates resolved by the nvim plugin, placeholder host
-- Example of parameterized trade queries
-- Use {{PARAM_NAME}} to define parameters that will be resolved at execution time

-- Parameter sets for common scenarios
-- @PARAMS: morning_bloomberg = { ORDER_ID: "ORDER_001", SOURCE: "Bloomberg", DATE: "2025,09,27" }
-- @PARAMS: afternoon_reuters = { ORDER_ID: "ORDER_002", SOURCE: "Reuters", DATE: "2025,09,27" }
-- @PARAMS: test_order = { ORDER_ID: "TEST_999", SOURCE: "TradeWeb", DATE: "2025,09,26" }

-- Example 1: Query specific order
WITH web_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "PlatformOrderId.StartsWith(\"{{ORDER_ID}}\")"
    }')
)
SELECT
    PlatformOrderId,
    TradeDate,
    Symbol,
    Quantity,
    Price,
    Status
FROM web_trades
ORDER BY TradeDate DESC;

-- Example 2: Query by source and date
WITH source_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "Source = \"{{SOURCE}}\" AND TradeDate = DateTime({{DATE}})"
    }')
)
SELECT
    Source,
    COUNT(*) as trade_count,
    SUM(Quantity * Price) as total_value,
    AVG(Price) as avg_price
FROM source_trades
GROUP BY Source;

-- Example 3: Complex query with multiple parameters
WITH filtered_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "PlatformOrderId.StartsWith(\"{{ORDER_PREFIX}}\") AND Source = \"{{SOURCE}}\" AND TradeDate >= DateTime({{START_DATE}}) AND TradeDate <= DateTime({{END_DATE}})"
    }')
)
SELECT
    DATE(TradeDate) as trade_date,
    Source,
    COUNT(*) as trades,
    SUM(CASE WHEN Status = 'FILLED' THEN Quantity ELSE 0 END) as filled_qty,
    SUM(CASE WHEN Status = 'PARTIAL' THEN Quantity ELSE 0 END) as partial_qty,
    SUM(CASE WHEN Status = 'CANCELLED' THEN Quantity ELSE 0 END) as cancelled_qty
FROM filtered_trades
GROUP BY DATE(TradeDate), Source
ORDER BY trade_date DESC;

-- Usage:
-- 1. Put cursor on a query with parameters
-- 2. Press \sx to execute
-- 3. You'll be prompted to select/enter values for each parameter
-- 4. Previous values are remembered and available for quick selection
--
-- Quick parameter cycling:
-- - Put cursor on a {{PARAMETER}}
-- - Press <leader>spn for next value from history
-- - Press <leader>spp for previous value from history
--
-- Save frequently used parameter combinations:
-- - Press <leader>sps to save current parameter values as a named set