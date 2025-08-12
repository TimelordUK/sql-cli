-- MICROSTRUCTURE ISSUE DETECTION QUERIES
-- ========================================
-- Queries to identify and analyze fade, partial fills, and retry patterns

-- 1. DETECT FADE EVENTS
-- Shows when liquidity was taken by competitors
SELECT 
    snapshot_time,
    order_id,
    venue,
    quantity as lost_quantity,
    fade_reason,
    instruction
FROM realistic_flow
WHERE event_type = 'VENUE_FADE'
ORDER BY snapshot_time;

-- 2. ANALYZE PARTIAL FILLS
-- Identify orders that didn't fill completely
SELECT 
    snapshot_time,
    order_id,
    venue,
    filled_quantity || '/' || quantity as fill_ratio,
    ROUND(filled_quantity * 100.0 / quantity, 1) || '%' as fill_pct,
    instruction
FROM realistic_flow
WHERE event_type = 'VENUE_PARTIAL'
ORDER BY snapshot_time;

-- 3. FIND RETRY ATTEMPTS
-- Shows SOR retry logic in action
SELECT 
    snapshot_time,
    order_id,
    parent_order_id,
    venue,
    quantity,
    instruction,
    attempt
FROM realistic_flow
WHERE event_type LIKE '%RETRY%'
ORDER BY snapshot_time;

-- 4. VENUE PERFORMANCE ANALYSIS
-- Which venues are performing poorly?
WITH venue_attempts AS (
    SELECT 
        venue,
        COUNT(*) as total_orders,
        SUM(CASE WHEN event_type = 'VENUE_FILLED' THEN 1 ELSE 0 END) as filled_orders,
        SUM(CASE WHEN event_type = 'VENUE_FADE' THEN 1 ELSE 0 END) as fade_count,
        SUM(CASE WHEN event_type = 'VENUE_PARTIAL' THEN 1 ELSE 0 END) as partial_count,
        SUM(CASE WHEN event_type = 'VENUE_REJECTED' THEN 1 ELSE 0 END) as reject_count
    FROM realistic_flow
    WHERE order_level = 3  -- SOR routes only
      AND event_type LIKE 'VENUE_%'
    GROUP BY venue
)
SELECT 
    venue,
    total_orders,
    filled_orders,
    fade_count,
    partial_count,
    ROUND(filled_orders * 100.0 / total_orders, 1) || '%' as fill_rate,
    CASE 
        WHEN fade_count > 0 THEN '⚠️ HIGH FADE'
        WHEN partial_count > 1 THEN '⚠️ PARTIALS'
        ELSE '✅ OK'
    END as status
FROM venue_attempts
ORDER BY fade_count DESC, partial_count DESC;

-- 5. SLICE EXECUTION ANALYSIS
-- How are algo slices performing?
SELECT 
    order_id,
    MAX(CASE WHEN event_type = 'NEW' THEN snapshot_time END) as start_time,
    MAX(CASE WHEN event_type = 'SLICE_COMPLETE' THEN snapshot_time END) as end_time,
    MAX(quantity) as target_qty,
    MAX(filled_quantity) as filled_qty,
    ROUND(MAX(filled_quantity) * 100.0 / MAX(quantity), 1) || '%' as fill_rate,
    MAX(state) as final_state
FROM realistic_flow
WHERE order_level = 2  -- Algo slices
GROUP BY order_id
ORDER BY order_id;

-- 6. CLIENT IMPACT ANALYSIS
-- What's the client actually seeing?
SELECT 
    snapshot_time,
    filled_quantity || '/' || quantity as progress,
    ROUND(filled_quantity * 100.0 / quantity, 1) || '%' as fill_pct,
    average_price,
    state,
    CASE 
        WHEN LAG(filled_quantity) OVER (ORDER BY snapshot_time) = filled_quantity 
        THEN '❌ NO PROGRESS'
        ELSE '✅ FILLING'
    END as status
FROM realistic_flow
WHERE order_level = 0  -- Client level
  AND event_type IN ('CLIENT_UPDATE', 'ACCEPTED', 'NEW')
ORDER BY snapshot_time;

-- 7. IDENTIFY PROBLEM PERIODS
-- When did we have issues?
WITH fill_gaps AS (
    SELECT 
        snapshot_time,
        filled_quantity,
        LAG(snapshot_time) OVER (ORDER BY snapshot_time) as prev_time,
        LAG(filled_quantity) OVER (ORDER BY snapshot_time) as prev_filled,
        (JULIANDAY(snapshot_time) - JULIANDAY(LAG(snapshot_time) OVER (ORDER BY snapshot_time))) * 24 * 60 as minutes_elapsed
    FROM realistic_flow
    WHERE order_level = 0 
      AND event_type = 'CLIENT_UPDATE'
)
SELECT 
    prev_time as from_time,
    snapshot_time as to_time,
    ROUND(minutes_elapsed, 1) as gap_minutes,
    filled_quantity - prev_filled as fills_in_period,
    CASE 
        WHEN filled_quantity = prev_filled THEN '❌ STUCK - No fills'
        WHEN minutes_elapsed > 5 THEN '⚠️ SLOW - Long gap'
        ELSE '✅ Normal'
    END as diagnosis
FROM fill_gaps
WHERE prev_time IS NOT NULL
ORDER BY minutes_elapsed DESC;

-- 8. FADE IMPACT ON CLIENT
-- How much did fades cost the client?
WITH fade_impact AS (
    SELECT 
        SUM(quantity) as total_fade_qty
    FROM realistic_flow
    WHERE event_type = 'VENUE_FADE'
)
SELECT 
    'Fade Impact Analysis' as report,
    (SELECT total_fade_qty FROM fade_impact) as shares_lost_to_fade,
    (SELECT MAX(quantity) FROM realistic_flow WHERE order_level = 0) as total_order_size,
    ROUND((SELECT total_fade_qty FROM fade_impact) * 100.0 / 
          (SELECT MAX(quantity) FROM realistic_flow WHERE order_level = 0), 1) || '%' as fade_impact_pct;

-- 9. RETRY EFFECTIVENESS
-- Are retries working?
WITH retry_success AS (
    SELECT 
        r.order_id as retry_order,
        r.quantity as retry_qty,
        f.filled_quantity as filled_qty,
        CASE 
            WHEN f.state = 'FILLED' THEN 'SUCCESS'
            WHEN f.state = 'PARTIAL' THEN 'PARTIAL'
            ELSE 'FAILED'
        END as outcome
    FROM realistic_flow r
    LEFT JOIN realistic_flow f 
        ON r.order_id = f.order_id 
        AND f.event_type = 'VENUE_FILLED'
    WHERE r.event_type = 'NEW_RETRY'
)
SELECT 
    retry_order,
    retry_qty,
    filled_qty,
    outcome,
    CASE outcome
        WHEN 'SUCCESS' THEN '✅ Retry worked'
        WHEN 'PARTIAL' THEN '⚠️ Partially successful'
        ELSE '❌ Retry failed'
    END as assessment
FROM retry_success;

-- 10. EXECUTION QUALITY METRICS
-- Overall execution quality for the client
SELECT 
    'Execution Quality Report' as metric,
    MAX(quantity) as order_size,
    MAX(filled_quantity) as filled,
    MAX(remaining_quantity) as unfilled,
    ROUND(MAX(filled_quantity) * 100.0 / MAX(quantity), 1) || '%' as fill_rate,
    ROUND(MAX(average_price), 2) as vwap,
    (SELECT COUNT(*) FROM realistic_flow WHERE event_type = 'VENUE_FADE') as fade_events,
    (SELECT COUNT(*) FROM realistic_flow WHERE event_type = 'VENUE_PARTIAL') as partial_events,
    (SELECT COUNT(*) FROM realistic_flow WHERE event_type LIKE '%RETRY%') as retry_events,
    CASE 
        WHEN MAX(filled_quantity) * 100.0 / MAX(quantity) < 50 THEN '❌ POOR - Under 50% filled'
        WHEN MAX(filled_quantity) * 100.0 / MAX(quantity) < 90 THEN '⚠️ FAIR - Partially filled'
        ELSE '✅ GOOD - Well executed'
    END as quality_assessment
FROM realistic_flow
WHERE order_level = 0;