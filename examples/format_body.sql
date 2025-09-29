-- Example: Testing SQL formatter with WEB CTE using BODY with JSON

WITH WEB trades AS (
    URL 'http://tradesapi/api/trades/query' METHOD POST BODY '{"Select":"Source,Trader,Exchange,Currency","Where":"PlatformOrderId.StartsWith(\"ORD-001\")","OrderBy":"DealId","showArchivedTrades":true}' FORMAT JSON JSON_PATH 'Result' HEADERS ('Authorization': 'Bearer ${JWT_TOKEN}', 'Content-Type': 'application/json', 'Accept': 'application/json')
)
SELECT * FROM trades;