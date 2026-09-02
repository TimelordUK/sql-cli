-- [TEST:SKIP] illustrative -- posts to the placeholder host api.example.com
-- Example: Advanced WEB CTE features including POST requests, JSON path extraction, and authentication
-- This demonstrates the new capabilities added for REST API integration

-- Example 1: Using JSON_PATH to extract nested data
-- When an API returns { Result: [...] }, we can extract the array directly
WITH WEB nested_data AS (
    URL 'https://api.example.com/data'
    FORMAT JSON
    JSON_PATH 'Result'  -- Extract the Result array from { Result: [...] }
)
SELECT
    id,
    name,
    value
FROM nested_data
WHERE value > 100
ORDER BY value DESC;
GO

-- Example 2: POST request with JSON body
-- Send a request body to filter data on the server side
WITH WEB filtered_trades AS (
    URL 'https://api.trading.com/trades'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer ${JWT_TOKEN}',
        'Content-Type': 'application/json'
    )
    BODY '{
        "Select": "TradeId, Symbol, Price, Quantity, ExecutionTime",
        "Where": "Symbol = \"AAPL\" AND ExecutionTime >= \"2024-01-01\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    TradeId,
    Symbol,
    Price * Quantity as TotalValue,
    ExecutionTime
FROM filtered_trades
ORDER BY ExecutionTime DESC
LIMIT 100;
GO

-- Example 3: Multiple headers with environment variables
WITH WEB secured_api AS (
    URL 'https://api.company.com/v2/reports'
    METHOD GET
    HEADERS (
        'Authorization': 'Bearer ${API_TOKEN}',
        'X-API-Key': '${API_KEY}',
        'X-Request-ID': '${REQUEST_ID}',
        'Accept': 'application/json',
        'User-Agent': 'SQL-CLI/1.52'
    )
    FORMAT JSON
)
SELECT
    report_id,
    created_date,
    status,
    total_records
FROM secured_api
WHERE status = 'completed'
ORDER BY created_date DESC;
GO

-- Example 4: GraphQL query via POST
WITH WEB graphql_data AS (
    URL 'https://api.github.com/graphql'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer ${GITHUB_TOKEN}',
        'Content-Type': 'application/json'
    )
    BODY '{
        "query": "query { viewer { login repositories(first: 10, orderBy: {field: STARGAZERS, direction: DESC}) { nodes { name stargazerCount updatedAt } } } }"
    }'
    FORMAT JSON
    JSON_PATH 'data.viewer.repositories.nodes'
)
SELECT
    name as repo_name,
    stargazerCount as stars,
    updatedAt as last_updated
FROM graphql_data
ORDER BY stars DESC;
GO

-- Example 5: Combining JSON_PATH with complex filtering
-- Extract nested array and apply SQL filtering
WITH WEB market_data AS (
    URL 'https://api.marketdata.com/quotes'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer ${MARKET_API_TOKEN}'
    )
    BODY '{
        "symbols": ["AAPL", "GOOGL", "MSFT", "AMZN"],
        "fields": ["symbol", "price", "volume", "change", "changePercent"]
    }'
    FORMAT JSON
    JSON_PATH 'data.quotes'  -- Extract from { data: { quotes: [...] } }
)
SELECT
    symbol,
    price,
    volume,
    changePercent as pct_change,
    CASE
        WHEN changePercent > 2 THEN 'Strong Buy'
        WHEN changePercent > 0 THEN 'Buy'
        WHEN changePercent > -2 THEN 'Hold'
        ELSE 'Sell'
    END as recommendation
FROM market_data
WHERE volume > 1000000
ORDER BY changePercent DESC;
GO

-- Example 6: PUT request to update data
WITH WEB update_result AS (
    URL 'https://api.company.com/resources/123'
    METHOD PUT
    HEADERS (
        'Authorization': 'Bearer ${API_TOKEN}',
        'Content-Type': 'application/json'
    )
    BODY '{
        "status": "active",
        "lastModified": "2024-01-15T10:00:00Z",
        "metadata": {
            "updatedBy": "sql-cli",
            "version": 2
        }
    }'
    FORMAT JSON
)
SELECT
    id,
    status,
    message,
    timestamp
FROM update_result;
GO

-- Example 7: DELETE request
WITH WEB delete_result AS (
    URL 'https://api.company.com/temp/session/abc123'
    METHOD DELETE
    HEADERS (
        'Authorization': 'Bearer ${API_TOKEN}'
    )
    FORMAT JSON
)
SELECT
    success,
    message,
    deletedCount
FROM delete_result;
GO

-- Example 8: Chaining multiple WEB CTEs
-- First get authentication token, then use it for data request
WITH WEB auth_token AS (
    URL 'https://auth.company.com/oauth/token'
    METHOD POST
    HEADERS (
        'Content-Type': 'application/x-www-form-urlencoded'
    )
    BODY 'client_id=${CLIENT_ID}&client_secret=${CLIENT_SECRET}&grant_type=client_credentials'
    FORMAT JSON
),
WEB protected_data AS (
    URL 'https://api.company.com/protected/data'
    METHOD GET
    HEADERS (
        'Authorization': CONCAT('Bearer ', (SELECT access_token FROM auth_token))
    )
    FORMAT JSON
    JSON_PATH 'data.items'
)
SELECT
    id,
    name,
    value,
    created_at
FROM protected_data
WHERE value > 1000
ORDER BY created_at DESC;
GO

-- Example 9: Working example with JSONPlaceholder (no auth needed)
-- This demonstrates POST with JSON_PATH on a public API
WITH WEB todos AS (
    URL 'https://jsonplaceholder.typicode.com/todos'
    METHOD GET
    FORMAT JSON
)
SELECT
    userId,
    COUNT(*) as total_todos,
    SUM(CASE WHEN completed THEN 1 ELSE 0 END) as completed_todos,
    SUM(CASE WHEN NOT completed THEN 1 ELSE 0 END) as pending_todos,
    ROUND(100.0 * SUM(CASE WHEN completed THEN 1 ELSE 0 END) / COUNT(*), 2) as completion_rate
FROM todos
GROUP BY userId
ORDER BY completion_rate DESC, userId
LIMIT 10;
GO