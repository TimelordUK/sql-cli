-- VWAP Analysis Queries for SQL-CLI TUI
-- =====================================

-- 1. GET PARENT ORDER ONLY (Client View)
-- This is what the client sees - just their order
SELECT * FROM vwap_example_orders 
WHERE parent_order_id IS NULL;

-- 2. GET PARENT ORDER WITH CALCULATED PROGRESS
-- Shows filled quantity from the parent order record
SELECT 
    order_id,
    client_name,
    ticker,
    side,
    quantity as ordered_qty,
    filled_quantity as filled_qty,
    (filled_quantity * 100.0 / quantity) as fill_pct,
    state,
    timestamp as order_time,
    update_timestamp as last_update
FROM vwap_example_orders 
WHERE parent_order_id IS NULL;

-- 3. GET ACCUMULATED FILLS FOR PARENT ORDER
-- Sum all fills that belong to this parent (via root_order_id)
SELECT 
    root_order_id,
    COUNT(*) as num_fills,
    SUM(quantity) as total_filled,
    AVG(price) as avg_price,
    MIN(timestamp) as first_fill,
    MAX(timestamp) as last_fill
FROM vwap_example_fills
WHERE root_order_id = 'ORD_1754985600_7487'
GROUP BY root_order_id;

-- 4. GET FILL PROGRESSION OVER TIME (for charting)
-- Shows cumulative volume at each timestamp
WITH fill_times AS (
    SELECT 
        timestamp,
        quantity,
        price,
        SUM(quantity) OVER (ORDER BY timestamp) as cumulative_qty
    FROM vwap_example_fills
    WHERE root_order_id = 'ORD_1754985600_7487'
)
SELECT * FROM fill_times
ORDER BY timestamp;

-- 5. GET VENUE BREAKDOWN FOR CLIENT
-- Shows where the order was executed
SELECT 
    venue,
    COUNT(*) as fills,
    SUM(quantity) as venue_qty,
    AVG(price) as avg_price
FROM vwap_example_fills
WHERE root_order_id = 'ORD_1754985600_7487'
GROUP BY venue
ORDER BY venue_qty DESC;

-- 6. GET HOURLY EXECUTION PATTERN
-- Shows VWAP profile - how much was executed each hour
SELECT 
    SUBSTR(timestamp, 1, 13) as hour,
    COUNT(*) as fills,
    SUM(quantity) as hour_qty,
    AVG(price) as avg_price
FROM vwap_example_fills
WHERE root_order_id = 'ORD_1754985600_7487'
GROUP BY SUBSTR(timestamp, 1, 13)
ORDER BY hour;

-- 7. FILTER TO SHOW ONLY DIRECT CHILD ORDERS
-- Algo slices sent by VWAP engine (not SOR children)
SELECT 
    order_id,
    quantity,
    filled_quantity,
    state,
    timestamp
FROM vwap_example_orders
WHERE parent_order_id = 'ORD_1754985600_7487'
  AND order_id LIKE 'ALGO_%'
ORDER BY timestamp;

-- 8. GET SIMPLIFIED CLIENT VIEW
-- Just show key metrics that matter to client
SELECT 
    'ORD_1754985600_7487' as order_id,
    'Blackrock Asset Management' as client,
    'ASML.AS' as ticker,
    100000 as ordered,
    (SELECT SUM(quantity) FROM vwap_example_fills 
     WHERE root_order_id = 'ORD_1754985600_7487') as filled,
    (SELECT AVG(price) FROM vwap_example_fills 
     WHERE root_order_id = 'ORD_1754985600_7487') as avg_price,
    (SELECT COUNT(DISTINCT venue) FROM vwap_example_fills 
     WHERE root_order_id = 'ORD_1754985600_7487') as venues_used;

-- 9. CREATE TIME SERIES FOR PLOTTING
-- Get fills in 5-minute buckets for charting
SELECT 
    SUBSTR(timestamp, 1, 15) || '0:00' as time_bucket,
    SUM(quantity) as bucket_qty,
    AVG(price) as bucket_vwap,
    SUM(SUM(quantity)) OVER (ORDER BY SUBSTR(timestamp, 1, 15)) as cumulative
FROM vwap_example_fills
WHERE root_order_id = 'ORD_1754985600_7487'
GROUP BY SUBSTR(timestamp, 1, 15)
ORDER BY time_bucket;