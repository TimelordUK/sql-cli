-- Test Redis cache with Web CTE
-- This simulates fetching trade data from a file (as if it were an API)

WITH WEB trades AS (
    URL 'file:///home/me/dev/sql-cli/data/large_trades.csv'
    FORMAT CSV
    CACHE 300  -- Cache for 5 minutes
)
SELECT
    source,
    COUNT(*) as trade_count,
    SUM(amount) as total_amount,
    AVG(price) as avg_price
FROM trades
WHERE source IN ('Bloomberg', 'Barclays')
GROUP BY source
ORDER BY total_amount DESC;