-- #! ../data/fix_messages.json
-- Example: Analyzing FIX messages using Web CTE with file upload
-- This demonstrates how to upload a JSON file containing FIX messages to a selector service
-- and query the flattened results

-- Query 1: Upload FIX messages and analyze trades by symbol
-- The service applies message-type-specific selectors to extract relevant fields
WITH WEB fix_trades AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' '../data/fix_messages.json'
    FORMAT CSV
)
SELECT
    msg_type,
    CASE
        WHEN msg_type = '8' THEN 'Execution Report'
        WHEN msg_type = 'J' THEN 'Allocation Instruction'
        ELSE 'Unknown'
    END as message_type,
    symbol,
    CASE
        WHEN side = '1' THEN 'Buy'
        WHEN side = '2' THEN 'Sell'
        ELSE 'Unknown'
    END as side_desc,
    quantity,
    price,
    trader,
    fund_allocations,
    total_allocated
FROM fix_trades
ORDER BY symbol, msg_type;
GO

-- Query 2: Aggregate analysis - Total volume by symbol
WITH WEB fix_trades AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' '../data/fix_messages.json'
    FORMAT CSV
)
SELECT
    symbol,
    COUNT(*) as message_count,
    SUM(quantity) as total_quantity,
    AVG(price) as avg_price,
    MIN(price) as min_price,
    MAX(price) as max_price
FROM fix_trades
WHERE price IS NOT NULL
GROUP BY symbol
ORDER BY total_quantity DESC;
GO

-- Query 3: Find large trades (quantity > 1000)
WITH WEB fix_trades AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' '../data/fix_messages.json'
    FORMAT CSV
)
SELECT
    msg_type,
    symbol,
    side,
    quantity,
    price,
    ROUND(quantity * price, 2) as notional_value,
    trader,
    total_allocated
FROM fix_trades
WHERE quantity > 1000
ORDER BY notional_value DESC;
GO

-- Query 4: Analyze allocation patterns
WITH WEB fix_trades AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' '../data/fix_messages.json'
    FORMAT CSV
)
SELECT
    msg_type,
    symbol,
    fund_allocations,
    total_allocated,
    CASE
        WHEN total_allocated = quantity THEN 'Fully Allocated'
        WHEN total_allocated > 0 THEN 'Partially Allocated'
        ELSE 'Not Allocated'
    END as allocation_status
FROM fix_trades
WHERE fund_allocations > 0
ORDER BY fund_allocations DESC, symbol;
GO

-- Note: This example requires the JSON selector service to be running on port 5050
-- Start it with: dotnet run --project proxy/json-selector/JsonSelector/JsonSelector.csproj
-- The service transforms hierarchical FIX JSON messages into tabular format
-- using message-type-specific selectors for fields that vary by message type