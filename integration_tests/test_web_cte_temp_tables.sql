-- Example: Multi-stage analysis with WEB CTE and temp tables
-- Test with: \sq to run all, or \sx to run statement at cursor

-- Stage 1: Fetch trade data from Flask API into temp table
WITH WEB trades_data AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer test-token',
        'Content-Type': 'application/json'
    )
    BODY '{
        "Select": "Source,PlatformOrderId,BloomberTicker,SignedQuantity,BuySell,Price",
        "Where": "Source = \"Bloomberg\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades_data INTO #bloomberg_trades;
GO

-- Stage 2: Basic exploration - what's in the temp table?
SELECT
    COUNT(*) as total_trades,
    COUNT(DISTINCT BloomberTicker) as unique_tickers,
    MIN(Price) as min_price,
    MAX(Price) as max_price,
    SUM(SignedQuantity) as net_quantity
FROM #bloomberg_trades;
GO

-- Stage 3: Group by ticker to see activity
SELECT
    BloomberTicker as ticker,
    BuySell,
    COUNT(*) as trade_count,
    SUM(SignedQuantity) as total_quantity,
    AVG(Price) as avg_price,
    MIN(Price) as min_price,
    MAX(Price) as max_price
FROM #bloomberg_trades
GROUP BY BloomberTicker, BuySell
ORDER BY trade_count DESC
LIMIT 10;
GO

-- Stage 4: Find largest trades by quantity
SELECT
    Source,
    BloomberTicker,
    BuySell,
    SignedQuantity,
    Price
FROM #bloomberg_trades
ORDER BY SignedQuantity DESC
LIMIT 5;
GO
