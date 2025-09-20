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
WITH
    dates_1 AS (
        SELECT value AS day_number
        FROM RANGE(1, 30)
    ),
    dates AS (
        SELECT
            day_number,
            DATEADD('day', day_number, '2024-01-01') AS new_date
        FROM dates_1
    ),
    sales_data AS (
        SELECT
            new_date,
            day_number,
            DAYOFWEEK(new_date) AS weekday_num,
            day_number * 10 + day_number % 7 * 5 AS daily_sales
        FROM dates
    )
SELECT
    day_number,
    new_date,
    daily_sales,
    SUM(daily_sales) OVER (ORDER BY day_number ASC) AS cumulative_sales,
    LAG(daily_sales, 7, 0) OVER (ORDER BY day_number ASC) AS same_day_last_week
FROM sales_data
WHERE day_number <= 14
ORDER BY day_number ASC;
GO

-- 3. Generate Product Inventory Test Data
WITH
    ranged AS (
        SELECT value AS product_id
        FROM RANGE(1, 30)
    ),
    products AS (
        SELECT
            product_id,
            TEXTJOIN('_', 1, 'PROD', 1000 + product_id) AS sku,
            CASE
        WHEN IS_PRIME(product_id) = TRUE THEN 'Electronics'
        WHEN product_id % 2 = 0 THEN 'Clothing'
        ELSE 'Books'
    END AS category,
            product_id * 10 AS base_price
        FROM ranged
    ),
    inventory AS (
        SELECT
            product_id,
            sku,
            category,
            base_price,
            50 - product_id AS stock_quantity,
            CASE
        WHEN 50 - product_id < 10 THEN 'Low Stock'
        WHEN 50 - product_id < 25 THEN 'Normal'
        ELSE 'Well Stocked'
    END AS stock_status
        FROM products
    ),
    inventory_2 AS (
        SELECT
            *,
            base_price * stock_quantity AS inv_value
        FROM inventory
    )
SELECT
    product_id,
    sku,
    category,
    base_price,
    stock_quantity,
    stock_status,
    COUNT('*') OVER (PARTITION BY category) AS products_in_category,
    SUM(inv_value) OVER (PARTITION BY category) AS category_inventory_value
FROM inventory_2
WHERE stock_quantity > 0
ORDER BY category ASC, product_id ASC
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
WITH
    time_slots AS (
        SELECT value AS minute
        FROM RANGE(0, 59)
    ),
    traffic AS (
        SELECT
            minute,
            TEXTJOIN(':', 1, minute, '00') AS time_label,
            CASE
        WHEN minute >= 10 AND minute <= 20 THEN 100 + minute * 5
        WHEN minute >= 40 AND minute <= 50 THEN 150 + minute * 3
        ELSE 50 + minute % 10 * 10
    END AS requests_per_minute,
            CASE
        WHEN minute % 5 = 0 THEN 'Checkpoint'
        ELSE 'Normal'
    END AS monitoring_flag
        FROM time_slots
    ),
    lagged AS (
        SELECT
            minute,
            time_label,
            requests_per_minute,
            monitoring_flag,
            SUM(requests_per_minute) OVER (ORDER BY minute ASC) AS cumulative_requests,
            LAG(requests_per_minute, 1, 0) OVER (ORDER BY minute ASC) AS lag_change_from_prev
        FROM traffic
    )
SELECT *
FROM lagged;
GO

-- 6. Generate Factorial-like Sequences
WITH nums AS (
   SELECT value AS n FROM RANGE(1, 10)
), 
facts AS (
    SELECT 
       n,
       POWER(n, 2) AS squared,
       POWER(n, 3) AS cubed
    FROM nums
), 
calcs as (
    select
    *,
    SUM_N(n) AS sum_to_n,
    SUM_N_SQR(n)  AS sum_of_squares,
    SUM_N_CUBE(n) AS sum_of_cubes
    from facts
)
SELECT * from calcs
ORDER BY n;
GO
