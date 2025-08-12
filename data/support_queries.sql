-- SUPPORT ENGINEER QUERIES FOR TICK DATABASE
-- ============================================
-- This is exactly what you'd use when investigating a client trade issue

-- 1. CLIENT VIEW ONLY - What the client actually sees
-- Shows ONLY the client order updates, filters out all child orders
SELECT 
    snapshot_time,
    order_id,
    client_order_id,
    quantity,  -- This stays constant
    filled_quantity,  -- This goes UP
    remaining_quantity,  -- This goes DOWN
    average_price,  -- VWAP updates
    state,
    event_type
FROM tick_database 
WHERE order_level = 0  -- Client level only
ORDER BY snapshot_time;

-- 2. CHECK FILL PROGRESSION AT CLIENT LEVEL
-- See how the client order evolved over time
SELECT 
    snapshot_time,
    event_type,
    filled_quantity || '/' || quantity as progress,
    ROUND((filled_quantity * 100.0 / quantity), 1) || '%' as fill_pct,
    average_price,
    state
FROM tick_database 
WHERE order_id = 'CLIENT_C20241215_123456'
ORDER BY snapshot_time;

-- 3. IDENTIFY GAPS - Why no fills for 10 minutes?
-- Look for time gaps in client fills
WITH fill_times AS (
    SELECT 
        snapshot_time,
        filled_quantity,
        LAG(snapshot_time) OVER (ORDER BY snapshot_time) as prev_time,
        LAG(filled_quantity) OVER (ORDER BY snapshot_time) as prev_filled
    FROM tick_database 
    WHERE order_level = 0 AND event_type = 'FILL'
)
SELECT 
    prev_time as from_time,
    snapshot_time as to_time,
    ROUND((JULIANDAY(snapshot_time) - JULIANDAY(prev_time)) * 24 * 60, 1) as gap_minutes,
    filled_quantity - prev_filled as fills_in_period
FROM fill_times
WHERE prev_time IS NOT NULL
ORDER BY gap_minutes DESC;

-- 4. DRILL DOWN - What child orders were created during gap?
-- After finding a gap, investigate what the algo was doing
SELECT 
    snapshot_time,
    order_id,
    parent_order_id,
    order_level,
    CASE order_level
        WHEN 1 THEN 'ALGO_PARENT'
        WHEN 2 THEN 'ALGO_CHILD'
        WHEN 3 THEN 'SOR_ROUTE'
    END as order_type,
    quantity,
    filled_quantity,
    state,
    venue
FROM tick_database
WHERE client_order_id = 'C20241215_123456'
  AND snapshot_time BETWEEN '2025-08-12T10:00:00' AND '2025-08-12T11:00:00'
  AND event_type = 'NEW'
ORDER BY snapshot_time, order_level;

-- 5. VENUE ANALYSIS - Where did fills come from?
SELECT 
    venue,
    COUNT(*) as fill_count,
    SUM(filled_quantity) as total_filled,
    AVG(average_price) as avg_price
FROM tick_database
WHERE client_order_id = 'C20241215_123456'
  AND order_level = 3  -- SOR level
  AND event_type = 'FILL'
GROUP BY venue
ORDER BY total_filled DESC;

-- 6. CASCADING ANALYSIS - See how fills cascade up
-- This shows a single fill cascading from SOR → Algo Child → Algo Parent → Client
SELECT 
    snapshot_time,
    order_level,
    order_id,
    filled_quantity,
    event_type
FROM tick_database
WHERE client_order_id = 'C20241215_123456'
  AND event_type = 'FILL'
  AND snapshot_time BETWEEN '2025-08-12T09:30:00.200' AND '2025-08-12T09:30:00.400'
ORDER BY snapshot_time;

-- 7. PERFORMANCE METRICS - For the sales trader
SELECT 
    'C20241215_123456' as client_order_id,
    MAX(CASE WHEN order_level = 0 THEN quantity END) as ordered_qty,
    MAX(CASE WHEN order_level = 0 THEN filled_quantity END) as filled_qty,
    MAX(CASE WHEN order_level = 0 THEN average_price END) as vwap,
    COUNT(DISTINCT CASE WHEN order_level = 3 THEN venue END) as venues_used,
    COUNT(CASE WHEN order_level = 2 AND event_type = 'NEW' END) as algo_slices,
    COUNT(CASE WHEN order_level = 3 AND event_type = 'FILL' END) as sor_fills
FROM tick_database
WHERE client_order_id = 'C20241215_123456';

-- 8. REAL-TIME MONITORING - What sales trader would watch
-- Latest status of client order
SELECT * FROM tick_database 
WHERE order_level = 0 
ORDER BY snapshot_time DESC 
LIMIT 1;

-- 9. AUDIT TRAIL - Complete history for compliance
SELECT 
    record_id,
    snapshot_time,
    order_id,
    order_level,
    event_type,
    filled_quantity || '/' || quantity as progress,
    state
FROM tick_database
WHERE client_order_id = 'C20241215_123456'
ORDER BY record_id;

-- 10. ISSUE DETECTION - Find stuck orders
-- Orders that haven't filled in last 30 minutes
WITH last_fills AS (
    SELECT 
        order_id,
        MAX(snapshot_time) as last_fill_time,
        MAX(filled_quantity) as last_filled,
        MAX(quantity) as total_qty,
        MAX(state) as current_state
    FROM tick_database
    WHERE event_type = 'FILL'
    GROUP BY order_id
)
SELECT 
    order_id,
    last_fill_time,
    last_filled || '/' || total_qty as progress,
    current_state,
    ROUND((JULIANDAY('now') - JULIANDAY(last_fill_time)) * 24 * 60, 1) as minutes_since_fill
FROM last_fills
WHERE current_state != 'FILLED'
  AND minutes_since_fill > 30;