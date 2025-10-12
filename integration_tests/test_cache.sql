-- Test Redis caching with Web CTE
-- First execution will fetch from HTTP server (slow)
-- Subsequent executions will use Redis cache (instant)

WITH WEB trades AS (
    URL 'http://localhost:5002/trades.csv'
    FORMAT CSV
    CACHE 300  -- Cache for 5 minutes
)
SELECT
    source,
    COUNT(*) as trade_count,
    SUM(amount) as total_amount,
    AVG(price) as avg_price
FROM trades
WHERE source IN ('Bloomberg', 'Barclays', 'Reuters')
GROUP BY source
ORDER BY total_amount DESC
LIMIT 10;