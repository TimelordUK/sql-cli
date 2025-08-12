-- Client View Queries - See order from client perspective
-- =========================================================

-- 1. FILTER TO CLIENT LEVEL ONLY (what client sees)
-- Shows just the root parent order with its current state
SELECT 
    order_id,
    client_order_id,
    ticker,
    side,
    quantity as ordered_qty,
    filled_quantity,
    remaining_quantity,
    state,
    timestamp as order_time,
    update_timestamp as last_update
FROM vwap_proper_orders 
WHERE order_level = 0;  -- Client level only

-- 2. SEE ALL ORDERS WITH SAME CLIENT ID
-- Shows entire hierarchy but filtered by client order ID
SELECT 
    order_level,
    order_id,
    parent_order_id,
    client_order_id,
    quantity,
    filled_quantity,
    remaining_quantity,
    state
FROM vwap_proper_orders
WHERE client_order_id = 'C987654'
ORDER BY order_level, timestamp;

-- 3. VIEW CASCADING UPDATES (from cascading view file)
-- This shows how the client order updates with each fill
SELECT * FROM vwap_cascading_view
ORDER BY update_time;

-- 4. SEE FILL PROGRESSION
-- Shows how fills accumulate over time
SELECT 
    update_time,
    filled_quantity,
    ordered_quantity,
    fill_percentage,
    average_price,
    state
FROM vwap_cascading_view
WHERE update_type = 'Fill';

-- 5. TRACK FILL RATE OVER TIME
-- See fills binned by hour
SELECT 
    SUBSTR(update_time, 1, 13) as hour,
    COUNT(*) as fills_in_hour,
    MAX(filled_quantity) as cumulative_filled,
    MAX(fill_percentage) as cumulative_pct
FROM vwap_cascading_view
WHERE update_type = 'Fill'
GROUP BY SUBSTR(update_time, 1, 13)
ORDER BY hour;

-- 6. GET LATEST STATE OF CLIENT ORDER
-- Just the current/final state
SELECT * FROM vwap_cascading_view
ORDER BY update_time DESC
LIMIT 1;

-- 7. COMPARE ORDER LEVELS
-- See how many orders at each level
SELECT 
    order_level,
    CASE order_level
        WHEN 0 THEN 'Client Order'
        WHEN 1 THEN 'Algo Parent'
        WHEN 2 THEN 'Algo Child'
        WHEN 3 THEN 'SOR Route'
    END as level_name,
    COUNT(*) as order_count,
    SUM(filled_quantity) as total_filled
FROM vwap_proper_orders
WHERE client_order_id = 'C987654'
GROUP BY order_level
ORDER BY order_level;

-- 8. SIMULATE FIX MESSAGE UPDATES
-- This is what would be sent back to client over FIX
SELECT 
    update_time as fix_timestamp,
    'ExecutionReport' as fix_msg_type,
    client_order_id as ClOrdID,
    order_id as OrderID,
    filled_quantity as CumQty,
    remaining_quantity as LeavesQty,
    average_price as AvgPx,
    last_fill_quantity as LastQty,
    last_fill_price as LastPx,
    state as OrdStatus
FROM vwap_cascading_view
WHERE update_type = 'Fill'
ORDER BY update_time;

-- 9. PLOT-READY DATA
-- For charting cumulative volume
SELECT 
    update_time as x,
    filled_quantity as y,
    fill_percentage as y2
FROM vwap_cascading_view
WHERE update_type = 'Fill';