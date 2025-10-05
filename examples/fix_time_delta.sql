-- #! ../data/fix_timestamps.csv

-- FIX Protocol Transaction Time Delta Analysis
-- ============================================
-- FIX tag 60 (TransactTime) format: YYYYMMDD-HH:MM:SS.sss
-- Calculate millisecond deltas between consecutive transactions
--
-- NOTE: The FIX timestamp format is natively supported!
-- YYYYMMDD-HH:MM:SS.sss is automatically parsed as UTC datetime

-- Example 1: Basic LAG to show previous transaction time
SELECT
    order_id,
    transaction_time,
    LAG(transaction_time, 1) OVER (ORDER BY transaction_time) AS prev_transaction_time
FROM fix_timestamps;
GO

-- Example 2: Calculate millisecond delta using DATEDIFF
-- This is the recommended approach - clean and efficient!
SELECT
    order_id,
    transaction_time,
    LAG(transaction_time, 1) OVER (ORDER BY transaction_time) AS prev_time,
    DATEDIFF('millisecond',
             LAG(transaction_time, 1) OVER (ORDER BY transaction_time),
             transaction_time) AS ms_delta
FROM fix_timestamps;
GO

-- Example 3: Find transactions with >200ms latency
-- Identify slow transactions for performance analysis
SELECT
    order_id,
    transaction_time,
    DATEDIFF('millisecond',
             LAG(transaction_time, 1) OVER (ORDER BY transaction_time),
             transaction_time) AS ms_delta
FROM fix_timestamps
WHERE DATEDIFF('millisecond',
               LAG(transaction_time, 1) OVER (ORDER BY transaction_time),
               transaction_time) > 200
ORDER BY ms_delta DESC;
GO

-- Example 4: Latency statistics
-- Calculate min, max, and average millisecond deltas
WITH deltas AS (
    SELECT
        DATEDIFF('millisecond',
                 LAG(transaction_time, 1) OVER (ORDER BY transaction_time),
                 transaction_time) AS ms_delta
    FROM fix_timestamps
)
SELECT
    MIN(ms_delta) AS min_ms,
    MAX(ms_delta) AS max_ms,
    AVG(ms_delta) AS avg_ms,
    COUNT(*) AS total_transactions
FROM deltas
WHERE ms_delta IS NOT NULL;
GO


-- Example 5: Group by latency buckets
-- Categorize transaction latencies into buckets
WITH
    deltas AS (
        SELECT
            DATEDIFF('millisecond', LAG(transaction_time, 1) OVER (ORDER BY transaction_time ASC), transaction_time) AS ms_delta,
            *
        FROM fix_timestamps
    ),
    labels AS (
        SELECT
            order_id,
            transaction_time,
            CASE
        WHEN ms_delta < 100 THEN '<100ms'
        WHEN ms_delta < 200 THEN '100-200ms'
        WHEN ms_delta < 300 THEN '200-300ms'
        WHEN ms_delta < 500 THEN '300-500ms'
        ELSE '>500ms'
    END AS latency_bucket
        FROM deltas
    )
SELECT *
FROM labels;

-- Example 5b: Group by latency buckets
-- Categorize transaction latencies into buckets
WITH
    deltas AS (
        SELECT
            DATEDIFF('millisecond', LAG(transaction_time, 1) OVER (ORDER BY transaction_time ASC), transaction_time) AS ms_delta,
            *
        FROM fix_timestamps
    ),
    labels AS (
        SELECT
            order_id,
            transaction_time,
            CASE
        WHEN ms_delta < 100 THEN '<100ms'
        WHEN ms_delta < 200 THEN '100-200ms'
        WHEN ms_delta < 300 THEN '200-300ms'
        WHEN ms_delta < 500 THEN '300-500ms'
        ELSE '>500ms'
    END AS latency_bucket
        FROM deltas
    )
SELECT latency_bucket, count(*) as bucket_count
FROM labels
group by latency_bucket
order by latency_bucket;
GO
