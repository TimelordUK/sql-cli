-- JSONL log demo: querying app logs (newline-delimited JSON) as a SQL table.
--
-- Each line of data/app_logs.jsonl is one JSON object — the format Elasticsearch,
-- structured loggers, and most "log shippers" persist by default. Schema is the
-- union of object keys across the first 100 records, so events that introduce
-- new fields (auth events with user_id, http events with status, db events with
-- query_ms) all become columns automatically.
--
-- Run from the repo root:
--   ./target/release/sql-cli examples/jsonl_logs.sql


-- ================================================================
-- SECTION 1 — basic shape
-- ================================================================

-- How many events, and what services / levels are in play?
SELECT
    COUNT(*)                    AS total_events,
   COUNT(DISTINCT service)     AS distinct_services,
    COUNT(DISTINCT level)       AS distinct_levels
FROM READ_JSONL('data/app_logs.jsonl');
GO

-- Volume by level.
SELECT level, COUNT(*) AS events
FROM READ_JSONL('data/app_logs.jsonl')
GROUP BY level
ORDER BY events DESC;
GO

-- Volume by service x level — classic log triage view.
SELECT service, level, COUNT(*) AS events
FROM READ_JSONL('data/app_logs.jsonl')
GROUP BY service, level
ORDER BY service, events DESC;
GO


-- ================================================================
-- SECTION 2 — regex pre-filter at read time
-- ================================================================
-- READ_JSONL's optional second argument is a regex that filters source lines
-- BEFORE JSON parsing. On large log files this is the fast path: parse only
-- what matches.

-- Just the errors. (Faster than reading everything and filtering with WHERE
-- because non-matching lines are skipped before JSON parse.)
SELECT ts, service, msg
FROM READ_JSONL('data/app_logs.jsonl', '"level":"ERROR"')
ORDER BY ts;
GO

-- All auth-service activity.
SELECT ts, level, msg, user_id
FROM READ_JSONL('data/app_logs.jsonl', '"service":"auth"')
ORDER BY ts;
GO


-- ================================================================
-- SECTION 3 — heterogeneous schema in action
-- ================================================================
-- Different event shapes live side by side. http events have status &
-- latency_ms, db events have query_ms, auth events have user_id. Missing
-- fields surface as NULL.

-- HTTP request analytics: only rows with a status code.
SELECT method, path, status, latency_ms
FROM READ_JSONL('data/app_logs.jsonl')
WHERE status IS NOT NULL
ORDER BY latency_ms DESC 
LIMIT 5;
GO

-- DB query latency profile.
SELECT
    COUNT(*)            AS queries,
    MIN(query_ms)       AS p_min,
    AVG(query_ms)       AS p_avg,
    MAX(query_ms)       AS p_max
FROM READ_JSONL('data/app_logs.jsonl')
WHERE query_ms IS NOT NULL;
GO

-- Slow queries (>500ms) — typical perf-investigation query.
SELECT ts, msg, query_ms, rows
FROM READ_JSONL('data/app_logs.jsonl')
WHERE query_ms > 500
ORDER BY query_ms DESC;
GO


-- ================================================================
-- SECTION 4 — security pattern: failed logins per user
-- ================================================================

SELECT
    user_id,
    COUNT(*) AS failed_attempts,
    MIN(ts)  AS first_attempt,
    MAX(ts)  AS last_attempt
FROM READ_JSONL('data/app_logs.jsonl')
WHERE service = 'auth'
  AND msg = 'failed login'
GROUP BY user_id
ORDER BY failed_attempts DESC;
GO


-- ================================================================
-- SECTION 5 — composing readers as CTEs
-- ================================================================
-- Treat READ_JSONL like any other table: build CTEs, join, union. Here we
-- pull errors and warnings into a unified incident view.

WITH issues AS (
    SELECT ts, service, level, msg
    FROM READ_JSONL('data/app_logs.jsonl', '"level":"(ERROR|WARN)"')
)
SELECT level, service, COUNT(*) AS issue_count
FROM issues
GROUP BY level, service
ORDER BY level, issue_count DESC;
GO


-- ================================================================
-- SECTION 6 — HTTP status distribution
-- ================================================================
-- Bucket http responses into 2xx/3xx/4xx/5xx classes and count. Status only
-- exists on http events; the IS NOT NULL guard scopes us to those rows.

SELECT
    CASE
        WHEN status BETWEEN 200 AND 299 THEN '2xx'
        WHEN status BETWEEN 300 AND 399 THEN '3xx'
        WHEN status BETWEEN 400 AND 499 THEN '4xx'
        WHEN status BETWEEN 500 AND 599 THEN '5xx'
        ELSE 'other'
    END AS status_class,
    COUNT(*) AS responses
FROM READ_JSONL('data/app_logs.jsonl')
WHERE status IS NOT NULL
GROUP BY status_class
ORDER BY status_class;
GO
