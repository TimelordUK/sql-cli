-- [TEST:SKIP] Needs live TeamCity + Elasticsearch credentials
--
-- Would more build agents actually help?  (live endpoints)
--
-- Runnable twin with sample data: teamcity_agent_capacity_demo.sql — start there
-- to see the output shape, then swap the two source CTEs below for these.
--
-- Environment expected:
--   TC_HOST    https://teamcity.example.com
--   TC_TOKEN   TeamCity bearer token
--   ES_HOST    https://elastic.example.com:9200
--   ES_KEY     Elasticsearch API key
--
-- ---------------------------------------------------------------------------
-- THINGS TO FIX BEFORE THIS RUNS  (all marked <<FIX>> below)
--   1. Server names.  Assumed `tcbuildNN` for NN in 21..26, and agent names
--      shaped `<host>-agentN` so the host is the part before the first '-'.
--      If agents are named differently, change the LEFT(...INDEXOF...) split
--      and the host list in the ES query.  This is the #1 thing to get right:
--      every join in here keys on host.
--   2. AGENTS_PER_HOST is hardcoded to 5 in the CASE below.
--   3. The 0.85 "CPU is pegged" line, and the 60s "this build actually waited"
--      floor, are both judgement calls — tune once you see the distribution.
--   4. The time window is inlined in BOTH queries and must match, or the asof
--      join silently has nothing to match against.
-- ---------------------------------------------------------------------------

WITH
    -- ==== SOURCE 1: TeamCity builds ========================================
    -- Two handles on the same data: the occupancy self-join needs the table on
    -- both sides, and one CTE is one source.  Yes, this fetches twice.
    WEB builds_raw AS (
        URL '${TC_HOST}/app/rest/builds?locator=sinceDate:20260909T000000%2B0000,count:5000&fields=build(id,queuedDate,startDate,finishDate,agent(name),buildType(projectName))'
        METHOD GET
        HEADERS (
            'Authorization': 'Bearer ${TC_TOKEN}',
            'Accept': 'application/json'
        )
        FORMAT JSON
        JSON_PATH 'build'
    ),
    WEB builds_raw2 AS (
        URL '${TC_HOST}/app/rest/builds?locator=sinceDate:20260909T000000%2B0000,count:5000&fields=build(id,queuedDate,startDate,finishDate,agent(name),buildType(projectName))'
        METHOD GET
        HEADERS (
            'Authorization': 'Bearer ${TC_TOKEN}',
            'Accept': 'application/json'
        )
        FORMAT JSON
        JSON_PATH 'build'
    ),

    -- ==== SOURCE 2: Metricbeat host CPU ====================================
    -- A `composite` aggregation, not a plain search: it returns ONE FLAT LIST of
    -- {host, minute} buckets, which is much easier to read than nested
    -- per_host -> per_minute buckets, or than raw `_source` hits.
    --
    -- Use system.cpu.total.norm.pct (0..1 regardless of core count), NOT
    -- system.cpu.total.pct (which goes to N for an N-core box).  "Is the box
    -- flat out" is a normalised question.
    --
    -- <<FIX>> the host list, and mind the composite `size` — beyond 1000
    -- host/minute buckets you need to page it with `after_key`.
    WEB cpu_raw AS (
        URL '${ES_HOST}/metricbeat-*/_search'
        METHOD POST
        HEADERS (
            'Authorization': 'ApiKey ${ES_KEY}',
            'Content-Type': 'application/json'
        )
        BODY '{
          "size": 0,
          "query": { "bool": { "filter": [
            { "range": { "@timestamp": { "gte": "2026-09-09T00:00:00Z", "lte": "2026-09-09T23:59:59Z" } } },
            { "terms": { "host.name": ["tcbuild21","tcbuild22","tcbuild23","tcbuild24","tcbuild25","tcbuild26"] } }
          ] } },
          "aggs": {
            "flat": {
              "composite": {
                "size": 1000,
                "sources": [
                  { "host": { "terms": { "field": "host.name" } } },
                  { "ts":   { "date_histogram": { "field": "@timestamp", "fixed_interval": "1m" } } }
                ]
              },
              "aggs": { "cpu": { "avg": { "field": "system.cpu.total.norm.pct" } } }
            }
          }
        }'
        FORMAT JSON
        JSON_PATH 'aggregations.flat.buckets'
    ),

    -- ==== NORMALISE ========================================================
    -- TeamCity stamps builds `20260909T143000+0100` — compact, WITH a real
    -- offset.  Two traps, both live:
    --   * the one-arg auto-detect parser does not recognise the compact form
    --   * the two-arg explicit-format parser DOES parse '%z' and then THROWS THE
    --     OFFSET AWAY (docs/SQL_PARITY.md P44) — so through BST every TeamCity
    --     timestamp lands an hour off UTC while Metricbeat stays correct,
    --     silently scrambling the correlation for half the year.
    -- So: rebuild the stamp as ISO-8601 and use the one-arg parser, which does
    -- honour the offset.  Delete this dance once P44 is fixed.
    builds_iso AS (
        SELECT
            id as build_id,
            buildType_projectName as project,
            agent_name as agent,
            SUBSTRING(queuedDate,1,4) || '-' || SUBSTRING(queuedDate,5,2) || '-' ||
            SUBSTRING(queuedDate,7,2) || 'T' || SUBSTRING(queuedDate,10,2) || ':' ||
            SUBSTRING(queuedDate,12,2) || ':' || SUBSTRING(queuedDate,14,2) ||
            SUBSTRING(queuedDate,16,3) || ':' || SUBSTRING(queuedDate,19,2) as queued_iso,
            SUBSTRING(startDate,1,4) || '-' || SUBSTRING(startDate,5,2) || '-' ||
            SUBSTRING(startDate,7,2) || 'T' || SUBSTRING(startDate,10,2) || ':' ||
            SUBSTRING(startDate,12,2) || ':' || SUBSTRING(startDate,14,2) ||
            SUBSTRING(startDate,16,3) || ':' || SUBSTRING(startDate,19,2) as started_iso
        FROM builds_raw
    ),
    waiting AS (
        SELECT
            build_id,
            project,
            LEFT(agent, INDEXOF(agent, '-')) as host,          -- <<FIX>> host split
            UNIX_TIMESTAMP(PARSE_DATETIME_UTC(queued_iso))  as queued_epoch,
            UNIX_TIMESTAMP(PARSE_DATETIME_UTC(started_iso)) as started_epoch
        FROM builds_iso
    ),
    waited AS (
        SELECT build_id, project, host, queued_epoch,
               started_epoch - queued_epoch as queue_secs
        FROM waiting
        WHERE started_epoch - queued_epoch > 60                -- <<FIX>> jitter floor
    ),

    -- Same normalisation on the second handle, as occupancy intervals.
    running AS (
        SELECT
            LEFT(agent_name, INDEXOF(agent_name, '-')) as r_host,
            UNIX_TIMESTAMP(PARSE_DATETIME_UTC(
                SUBSTRING(startDate,1,4) || '-' || SUBSTRING(startDate,5,2) || '-' ||
                SUBSTRING(startDate,7,2) || 'T' || SUBSTRING(startDate,10,2) || ':' ||
                SUBSTRING(startDate,12,2) || ':' || SUBSTRING(startDate,14,2) ||
                SUBSTRING(startDate,16,3) || ':' || SUBSTRING(startDate,19,2))) as r_start,
            UNIX_TIMESTAMP(PARSE_DATETIME_UTC(
                SUBSTRING(finishDate,1,4) || '-' || SUBSTRING(finishDate,5,2) || '-' ||
                SUBSTRING(finishDate,7,2) || 'T' || SUBSTRING(finishDate,10,2) || ':' ||
                SUBSTRING(finishDate,12,2) || ':' || SUBSTRING(finishDate,14,2) ||
                SUBSTRING(finishDate,16,3) || ':' || SUBSTRING(finishDate,19,2))) as r_end
        FROM builds_raw2
    ),

    -- Metricbeat composite buckets -> flat columns.
    -- <<FIX>> Bucket keys arrive nested as key.host / key.ts, and the metric as
    -- cpu.value.  Reading nested columns is a known rough edge here; if these
    -- names do not resolve, dump the ES response to a file and READ_JSON it, or
    -- flatten with a one-line jq first.  This is the one part I could not test
    -- without a live cluster.
    cpu_norm AS (
        SELECT
            key_host      as host,
            key_ts / 1000 as ts_epoch,     -- composite date key is epoch MILLIS
            cpu_value     as cpu_pct
        FROM cpu_raw
    ),

    -- ==== HOW BUSY WAS THE HOST WHEN THIS BUILD QUEUED? ====================
    occupancy AS (
        SELECT
            waited.build_id, waited.project, waited.host,
            waited.queued_epoch, waited.queue_secs,
            COUNT(*) as busy_agents
        FROM waited
        JOIN running
          ON waited.host = running.r_host
         AND waited.queued_epoch >= running.r_start
         AND waited.queued_epoch <  running.r_end
        GROUP BY waited.build_id, waited.project, waited.host,
                 waited.queued_epoch, waited.queue_secs
    ),

    -- ==== ASOF JOIN, BY HAND ===============================================
    -- There is no ASOF JOIN / kdb `aj` here.  Pair each build with every CPU
    -- sample at or before it, then keep the latest — same semantics as `aj`.
    -- The join condition needs the LEFT table's column as the LEFT operand
    -- (`a.t >= b.t`, never `b.t <= a.t`) or the planner rejects it outright.
    cpu_candidates AS (
        SELECT occupancy.build_id, cpu_norm.ts_epoch, cpu_norm.cpu_pct
        FROM occupancy
        JOIN cpu_norm
          ON occupancy.host = cpu_norm.host
         AND occupancy.queued_epoch >= cpu_norm.ts_epoch
    ),
    cpu_ranked AS (
        SELECT build_id, ts_epoch, cpu_pct,
               ROW_NUMBER() OVER (PARTITION BY build_id ORDER BY ts_epoch DESC) as rn
        FROM cpu_candidates
    ),
    cpu_at_queue AS (
        SELECT build_id as c_build_id, ts_epoch as c_ts, cpu_pct as c_cpu
        FROM cpu_ranked WHERE rn = 1
    ),

    -- ==== VERDICT ==========================================================
    classified AS (
        SELECT
            occupancy.build_id, occupancy.project, occupancy.host,
            occupancy.queue_secs, occupancy.busy_agents,
            cpu_at_queue.c_cpu as cpu_at_queue,
            -- Guard: if the nearest CPU sample is minutes stale, the verdict on
            -- that row is worth little.  Watch this column.
            occupancy.queued_epoch - cpu_at_queue.c_ts as cpu_staleness_secs,
            CASE
                WHEN occupancy.busy_agents < 5  THEN 'pool_mismatch'   -- <<FIX>> agents/host
                WHEN cpu_at_queue.c_cpu >= 0.85 THEN 'hardware_bound'  -- <<FIX>> pegged line
                ELSE                                 'add_agents'
            END as verdict
        FROM occupancy
        JOIN cpu_at_queue ON occupancy.build_id = cpu_at_queue.c_build_id
    )

SELECT
    verdict,
    COUNT(*)                    as builds_waiting,
    SUM(queue_secs)             as total_queue_secs,
    ROUND(AVG(queue_secs), 0)   as avg_queue_secs,
    ROUND(AVG(cpu_at_queue), 2) as avg_cpu,
    ROUND(AVG(busy_agents), 1)  as avg_busy_agents,
    MAX(cpu_staleness_secs)     as worst_cpu_staleness
FROM classified
GROUP BY verdict
ORDER BY total_queue_secs DESC;
GO
