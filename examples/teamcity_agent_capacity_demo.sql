-- #! ../data/teamcity_builds_sample.csv
--
-- Would more build agents actually help?  (runnable demo, sample data)
--
-- This is the same logic as teamcity_agent_capacity.sql, but against checked-in
-- sample data so it runs with no credentials.  See that file for the versions of
-- the two source CTEs that hit the real TeamCity and Elasticsearch endpoints.
--
-- The point of the analysis: queue time on its own does NOT tell you agents are
-- short.  Each queued build is classified into one of three verdicts, and only
-- one of them is fixed by adding agents:
--
--   hardware_bound   agents all busy AND host CPU pegged -> more agents on these
--                    boxes make things WORSE (ninja already owns every core)
--   add_agents       agents all busy AND host CPU idle   -> more agents per
--                    server is a free win
--   pool_mismatch    agents were FREE while this build waited -> not a capacity
--                    problem at all; check agent pool / build requirements
--
-- The sample data is built to produce all three, two hosts each.

WITH
    -- Two handles on the same build data: the self-join that computes agent
    -- occupancy needs the table on both sides, and each CTE is one source.
    WEB builds_raw  AS (URL 'file://data/teamcity_builds_sample.csv' FORMAT CSV),
    WEB builds_raw2 AS (URL 'file://data/teamcity_builds_sample.csv' FORMAT CSV),
    WEB cpu_raw     AS (URL 'file://data/teamcity_cpu_sample.csv'    FORMAT CSV),

    -- Builds that actually had to wait.  60s filter drops normal scheduling jitter.
    waiting AS (
        SELECT
            build_id,
            project,
            LEFT(agent, INDEXOF(agent, '-')) as host,
            queued_epoch,
            started_epoch - queued_epoch as queue_secs
        FROM builds_raw
        WHERE started_epoch - queued_epoch > 60
    ),

    -- Every build, as an occupancy interval [start, end) on its host.
    running AS (
        SELECT
            LEFT(agent, INDEXOF(agent, '-')) as r_host,
            started_epoch as r_start,
            finished_epoch as r_end
        FROM builds_raw2
    ),

    -- How many agents on that host were busy at the moment this build queued?
    occupancy AS (
        SELECT
            waiting.build_id,
            waiting.project,
            waiting.host,
            waiting.queued_epoch,
            waiting.queue_secs,
            COUNT(*) as busy_agents
        FROM waiting
        JOIN running
          ON waiting.host = running.r_host
         AND waiting.queued_epoch >= running.r_start
         AND waiting.queued_epoch <  running.r_end
        GROUP BY waiting.build_id, waiting.project, waiting.host,
                 waiting.queued_epoch, waiting.queue_secs
    ),

    -- ASOF JOIN, done by hand: sql-cli has no ASOF/kdb-style `aj`, so pair every
    -- build with every CPU sample at or before it, then keep the latest one.
    -- NOTE: the join condition needs the LEFT table's column as the LEFT operand
    -- (`a.t >= b.t`, never `b.t <= a.t`) or the planner rejects it.
    cpu_candidates AS (
        SELECT
            occupancy.build_id,
            cpu_raw.ts_epoch,
            cpu_raw.cpu_pct
        FROM occupancy
        JOIN cpu_raw
          ON occupancy.host = cpu_raw.host
         AND occupancy.queued_epoch >= cpu_raw.ts_epoch
    ),
    cpu_ranked AS (
        SELECT
            build_id,
            ts_epoch,
            cpu_pct,
            ROW_NUMBER() OVER (PARTITION BY build_id ORDER BY ts_epoch DESC) as rn
        FROM cpu_candidates
    ),
    cpu_at_queue AS (
        SELECT build_id as c_build_id, ts_epoch as c_ts, cpu_pct as c_cpu
        FROM cpu_ranked
        WHERE rn = 1
    ),

    -- Join the two together and apply the verdict.
    -- AGENTS_PER_HOST = 5 and the 0.85 CPU line are the two knobs to tune.
    classified AS (
        SELECT
            occupancy.build_id,
            occupancy.project,
            occupancy.host,
            occupancy.queue_secs,
            occupancy.busy_agents,
            cpu_at_queue.c_cpu as cpu_at_queue,
            occupancy.queued_epoch - cpu_at_queue.c_ts as cpu_staleness_secs,
            CASE
                WHEN occupancy.busy_agents < 5           THEN 'pool_mismatch'
                WHEN cpu_at_queue.c_cpu >= 0.85          THEN 'hardware_bound'
                ELSE                                          'add_agents'
            END as verdict
        FROM occupancy
        JOIN cpu_at_queue ON occupancy.build_id = cpu_at_queue.c_build_id
    )

SELECT
    verdict,
    COUNT(*) as builds_waiting,
    SUM(queue_secs) as total_queue_secs,
    ROUND(AVG(queue_secs), 0) as avg_queue_secs,
    ROUND(AVG(cpu_at_queue), 2) as avg_cpu,
    ROUND(AVG(busy_agents), 1) as avg_busy_agents
FROM classified
GROUP BY verdict
ORDER BY total_queue_secs DESC;
GO
