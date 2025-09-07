-- ============================================================================
-- Test Data Generation using RANGE and CTEs
-- ============================================================================
-- This example shows how to use RANGE to generate test datasets for various
-- scenarios without needing external CSV files
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/range_test_data_generation.sql -o table
-- ============================================================================

-- 1. Generate Mock User IDs with Categories
WITH users AS (
    SELECT 
        value AS user_id,
        value * 1000 AS user_code,
        CASE 
            WHEN value % 3 = 0 THEN 'Premium'
            WHEN value % 3 = 1 THEN 'Standard'
            ELSE 'Basic'
        END AS account_type
    FROM RANGE(1, 20)
)
SELECT 
    user_id,
    user_code,
    account_type,
    ROW_NUMBER() OVER (PARTITION BY account_type ORDER BY user_id) AS rank_in_tier
FROM users
ORDER BY user_id
LIMIT 10;
GO

-- 2. Generate Time Series Data (Mock Daily Sales)
WITH dates AS (
    SELECT 
        value AS day_number,
        TEXTJOIN('-', 1, '2024', '01', 
            CASE WHEN value < 10 THEN TEXTJOIN('', 1, '0', value) ELSE value END
        ) AS date_str
    FROM RANGE(1, 30)
),
sales_data AS (
    SELECT 
        day_number,
        date_str,
        -- Simulate sales with some pattern
        (value * 10) + (value % 7) * 5 AS daily_sales,
        CASE 
            WHEN value % 7 IN (0, 6) THEN 'Weekend'
            ELSE 'Weekday'
        END AS day_type
    FROM dates
)
SELECT 
    date_str AS date,
    day_type,
    daily_sales,
    SUM(daily_sales) OVER (ORDER BY day_number) AS cumulative_sales,
    LAG(daily_sales, 7, 0) OVER (ORDER BY day_number) AS same_day_last_week
FROM sales_data
WHERE day_number <= 14
ORDER BY day_number;
GO

-- 3. Generate Product Inventory Test Data
WITH products AS (
    SELECT 
        value AS product_id,
        TEXTJOIN('_', 1, 'PROD', (1000 + value)) AS sku,
        CASE 
            WHEN IS_PRIME(value) = true THEN 'Electronics'
            WHEN value % 2 = 0 THEN 'Clothing'
            ELSE 'Books'
        END AS category,
        value * 10 AS base_price
    FROM RANGE(1, 30)
),
inventory AS (
    SELECT 
        product_id,
        sku,
        category,
        base_price,
        (50 - value) AS stock_quantity,
        CASE 
            WHEN (50 - value) < 10 THEN 'Low Stock'
            WHEN (50 - value) < 25 THEN 'Normal'
            ELSE 'Well Stocked'
        END AS stock_status
    FROM products
)
SELECT 
    sku,
    category,
    base_price,
    stock_quantity,
    stock_status,
    COUNT(*) OVER (PARTITION BY category) AS products_in_category,
    SUM(base_price * stock_quantity) OVER (PARTITION BY category) AS category_inventory_value
FROM inventory
WHERE stock_quantity > 0
ORDER BY category, product_id
LIMIT 15;
GO

-- 4. Generate Test Scores Distribution
WITH students AS (
    SELECT value AS student_id FROM RANGE(1, 50)
),
scores AS (
    SELECT 
        student_id,
        -- Generate scores with some distribution
        CASE 
            WHEN IS_PRIME(student_id) = true THEN 85 + (student_id % 15)
            WHEN student_id % 2 = 0 THEN 70 + (student_id % 20)
            ELSE 60 + (student_id % 30)
        END AS test_score
    FROM students
),
graded AS (
    SELECT 
        student_id,
        test_score,
        CASE 
            WHEN test_score >= 90 THEN 'A'
            WHEN test_score >= 80 THEN 'B'
            WHEN test_score >= 70 THEN 'C'
            WHEN test_score >= 60 THEN 'D'
            ELSE 'F'
        END AS grade
    FROM scores
)
SELECT 
    grade,
    COUNT(*) AS student_count,
    MIN(test_score) AS min_score,
    MAX(test_score) AS max_score,
    AVG(test_score) AS avg_score
FROM graded
GROUP BY grade
ORDER BY grade;
GO

-- 5. Generate Network Traffic Simulation
WITH time_slots AS (
    SELECT value AS minute FROM RANGE(0, 59)
),
traffic AS (
    SELECT 
        minute,
        TEXTJOIN(':', 1, minute, '00') AS time_label,
        -- Simulate traffic with peaks
        CASE 
            WHEN minute BETWEEN 10 AND 20 THEN 100 + (minute * 5)
            WHEN minute BETWEEN 40 AND 50 THEN 150 + (minute * 3)
            ELSE 50 + (minute % 10) * 10
        END AS requests_per_minute,
        CASE 
            WHEN minute % 5 = 0 THEN 'Checkpoint'
            ELSE 'Normal'
        END AS monitoring_flag
    FROM time_slots
)
SELECT 
    time_label,
    requests_per_minute,
    monitoring_flag,
    SUM(requests_per_minute) OVER (ORDER BY minute) AS cumulative_requests,
    requests_per_minute - LAG(requests_per_minute, 1, 0) OVER (ORDER BY minute) AS change_from_prev
FROM traffic
WHERE minute <= 20
ORDER BY minute;
GO

-- 6. Generate Factorial-like Sequences
WITH numbers AS (
    SELECT value AS n FROM RANGE(1, 10)
),
factorials AS (
    SELECT 
        n,
        n AS term,
        -- Simulate factorial growth pattern
        POWER(n, 2) AS squared,
        POWER(n, 3) AS cubed
    FROM numbers
)
SELECT 
    n,
    term,
    squared,
    cubed,
    SUM(term) OVER (ORDER BY n) AS sum_to_n,
    SUM(squared) OVER (ORDER BY n) AS sum_of_squares,
    SUM(cubed) OVER (ORDER BY n) AS sum_of_cubes
FROM factorials
ORDER BY n;
GO