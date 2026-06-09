-- Elasticsearch: broad filter at the server, heavy lifting in SQL
--
-- Pattern: let Elasticsearch do what it's good at (time-window filtering,
-- term matching, limiting cardinality), then bring a slice of docs back and
-- do GROUP BY / percentiles / windowing / joins in the SQL engine.
--
-- Why this pattern:
--   - One ES round-trip per slice, many SQL queries per slice.
--   - SQL is more expressive than ES aggs for ad-hoc analysis.
--   - With CACHE n set on the WEB CTE, sql-cli (built with the `redis-cache`
--     feature and SQL_CLI_CACHE=true) re-uses the cached response for `n`
--     seconds, so re-running this script costs zero ES requests within the
--     cache window.
--
-- Target: a local stack with `app-events-*` indices, Filebeat -> Logstash -> ES.
-- Docs have top-level `event`, `level`, `outcome`, `path`, `method`,
-- `duration_ms`, `request_id`, `@timestamp`, `ts`, `ls_node`, plus a nested
-- `agent` object (which currently comes through as a debug-formatted string;
-- the array-projection trick below is what unlocks the flat fields).
--
-- Two key features make this work cleanly:
--   1. JSON_PATH 'hits.hits[]._source' — `[]` projects each hit's `_source`
--      object as a row, so ES fields become flat SQL columns.
--   2. CACHE 300 — re-uses the 1k-doc pull for 5 min, so all four analytic
--      queries below share one ES round-trip.
--
-- BODY syntax — two forms are accepted:
--   Single-quoted ('...') — works but has NO escape processing in the lexer.
--     Any apostrophe inside the body closes the string. Embedded double
--     quotes are fine. Query 1 below uses this form for reference.
--   $JSON$...$JSON$       — verbatim block, no escape processing either, but
--     happily contains both ' and ". Multi-line indentation is preserved as
--     written. Reads like a Kibana Dev Tools snippet. Queries 2-4 use this.
-- Both forms still expand ${ENV_VARS} (resolved by the HTTP fetcher, not the
-- lexer), so secrets/tokens can be injected the same way in either.
--
-- Quoting tip: `@timestamp` and other identifiers starting with `@` (or
-- containing `.`, `-`) must be wrapped in double quotes -> `"@timestamp"`.
--
-- To run against an auth'd cluster, add a Bearer header:
--   HEADERS ('Authorization': 'Bearer ${ES_TOKEN}')
-- Do NOT also add Content-Type when your body starts with `{` -- the fetcher
-- auto-adds it. ES 8.x rejects duplicate Content-Type headers with a 400.

-- ---------------------------------------------------------------------------
-- Query 1: traffic mix by event type
-- ---------------------------------------------------------------------------
-- The "broad filter" at ES: last 24h of http.request and search.executed
-- events, ranked by recency. 1k docs is small enough to round-trip quickly
-- and big enough for meaningful percentiles / top-N analysis.
--
-- Note the two-CTE pattern: aggregate first into `counts`, then run the
-- window function (SUM ... OVER ()) over the pre-aggregated values. The SQL
-- engine doesn't support SUM(COUNT(*)) OVER () directly -- pre-aggregating
-- in a CTE is the documented workaround.
--
-- BODY form: single-quoted '...' (kept as a reference). The lexer just reads
-- chars until the next `'`, so the body cannot contain an apostrophe. Compare
-- with Queries 2-4 which use the $JSON$...$JSON$ block form.
WITH WEB raw_events AS (
    URL 'http://localhost:9200/app-events-*/_search'
    METHOD POST
    CACHE 300
    BODY '{
        "size": 1000,
        "sort": [{"@timestamp": {"order": "desc"}}],
        "query": {
            "bool": {
                "filter": [
                    {"terms": {"event.keyword": ["http.request", "search.executed"]}},
                    {"range": {"@timestamp": {"gte": "now-24h"}}}
                ]
            }
        }
    }'
    FORMAT JSON
    JSON_PATH 'hits.hits[]._source'
),
counts AS (
    SELECT event, outcome, COUNT(*) AS docs
    FROM raw_events
    GROUP BY event, outcome
)
SELECT
    event,
    outcome,
    docs,
    ROUND(100.0 * docs / SUM(docs) OVER (), 2) AS pct_of_total
FROM counts
ORDER BY docs DESC;
GO

-- ---------------------------------------------------------------------------
-- Query 2: latency percentiles per path (http.request only)
-- ---------------------------------------------------------------------------
-- ES can compute percentiles via aggs, but only one-at-a-time and not over
-- expressions of multiple fields. In SQL we can do p50/p95/p99 alongside
-- count, error rate, and slowest single hit in one pass.
WITH WEB raw_events AS (
    URL 'http://localhost:9200/app-events-*/_search'
    METHOD POST
    CACHE 300
    BODY $JSON$
    {
        "size": 1000,
        "sort": [{"@timestamp": {"order": "desc"}}],
        "query": {
            "bool": {
                "filter": [
                    {"terms": {"event.keyword": ["http.request", "search.executed"]}},
                    {"range": {"@timestamp": {"gte": "now-24h"}}}
                ]
            }
        }
    }
    $JSON$
    FORMAT JSON
    JSON_PATH 'hits.hits[]._source'
)
SELECT
    path,
    method,
    COUNT(*) AS requests,
    ROUND(MEDIAN(duration_ms), 2) AS p50_ms,
    ROUND(PERCENTILE(duration_ms, 95), 2) AS p95_ms,
    ROUND(PERCENTILE(duration_ms, 99), 2) AS p99_ms,
    ROUND(MAX(duration_ms), 2) AS max_ms,
    SUM(CASE WHEN outcome != 'ok' THEN 1 ELSE 0 END) AS errors,
    ROUND(
        100.0 * SUM(CASE WHEN outcome != 'ok' THEN 1 ELSE 0 END) / COUNT(*),
        2
    ) AS error_pct
FROM raw_events
WHERE event = 'http.request'
GROUP BY path, method
HAVING COUNT(*) >= 5
ORDER BY p95_ms DESC;
GO

-- ---------------------------------------------------------------------------
-- Query 3: per-request_id timelines
-- ---------------------------------------------------------------------------
-- Per-request_id tracing (counting events per request, finding straddling
-- timestamps) is awkward in ES aggs but trivial in SQL.
WITH WEB raw_events AS (
    URL 'http://localhost:9200/app-events-*/_search'
    METHOD POST
    CACHE 300
    BODY $JSON$
    {
        "size": 1000,
        "sort": [{"@timestamp": {"order": "desc"}}],
        "query": {
            "bool": {
                "filter": [
                    {"terms": {"event.keyword": ["http.request", "search.executed"]}},
                    {"range": {"@timestamp": {"gte": "now-24h"}}}
                ]
            }
        }
    }
    $JSON$
    FORMAT JSON
    JSON_PATH 'hits.hits[]._source'
)
SELECT
    request_id,
    COUNT(*) AS event_count,
    COUNT(DISTINCT event) AS distinct_event_types,
    SUM(CASE WHEN outcome != 'ok' THEN 1 ELSE 0 END) AS errors,
    ROUND(SUM(duration_ms), 2) AS total_duration_ms,
    MIN("@timestamp") AS first_seen,
    MAX("@timestamp") AS last_seen
FROM raw_events
WHERE request_id IS NOT NULL
GROUP BY request_id
ORDER BY event_count DESC, errors DESC
LIMIT 20;
GO

-- ---------------------------------------------------------------------------
-- Query 4: top search queries and their hit-rate distribution
-- ---------------------------------------------------------------------------
-- search.executed docs carry `query` and `hits` fields. SQL banding is much
-- nicer than ES range aggs when you want named buckets.
WITH WEB raw_events AS (
    URL 'http://localhost:9200/app-events-*/_search'
    METHOD POST
    CACHE 300
    BODY $JSON$
    {
        "size": 1000,
        "sort": [{"@timestamp": {"order": "desc"}}],
        "query": {
            "bool": {
                "filter": [
                    {"terms": {"event.keyword": ["http.request", "search.executed"]}},
                    {"range": {"@timestamp": {"gte": "now-24h"}}}
                ]
            }
        }
    }
    $JSON$
    FORMAT JSON
    JSON_PATH 'hits.hits[]._source'
),
searches AS (
    SELECT
        query,
        hits,
        CASE
            WHEN hits = 0           THEN '0 (zero-result)'
            WHEN hits BETWEEN 1   AND 10   THEN '1-10'
            WHEN hits BETWEEN 11  AND 100  THEN '11-100'
            WHEN hits BETWEEN 101 AND 1000 THEN '101-1000'
            ELSE '1000+'
        END AS hit_band
    FROM raw_events
    WHERE event = 'search.executed'
)
SELECT
    query,
    COUNT(*) AS times_searched,
    ROUND(AVG(hits), 1) AS avg_hits,
    MIN(hits) AS min_hits,
    MAX(hits) AS max_hits,
    SUM(CASE WHEN hits = 0 THEN 1 ELSE 0 END) AS zero_result_runs
FROM searches
GROUP BY query
ORDER BY times_searched DESC, query
LIMIT 15;
GO
