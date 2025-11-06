-- Window Functions Formal Test Suite
-- This example creates its own test data and validates ALL window functions
-- Run with: sql-cli -f examples/window_functions_formal.sql -o json

-- Step 1: Create test data using a temp table with SQL CLI syntax
WITH test_data AS (
    -- Department A employees
    SELECT 'A' as dept, 'Alice' as name, 2024 as year, 1 as quarter, 100 as sales UNION ALL
    SELECT 'A', 'Alice', 2024, 2, 150 UNION ALL
    SELECT 'A', 'Alice', 2024, 3, 200 UNION ALL
    SELECT 'A', 'Alice', 2024, 4, 180 UNION ALL
    SELECT 'A', 'Bob', 2024, 1, 90 UNION ALL
    SELECT 'A', 'Bob', 2024, 2, 110 UNION ALL
    SELECT 'A', 'Bob', 2024, 3, 130 UNION ALL
    SELECT 'A', 'Bob', 2024, 4, 120 UNION ALL
    -- Department B employees
    SELECT 'B', 'Charlie', 2024, 1, 200 UNION ALL
    SELECT 'B', 'Charlie', 2024, 2, 220 UNION ALL
    SELECT 'B', 'Charlie', 2024, 3, 210 UNION ALL
    SELECT 'B', 'Charlie', 2024, 4, 230 UNION ALL
    SELECT 'B', 'Diana', 2024, 1, 160 UNION ALL
    SELECT 'B', 'Diana', 2024, 2, 170 UNION ALL
    SELECT 'B', 'Diana', 2024, 3, 165 UNION ALL
    SELECT 'B', 'Diana', 2024, 4, 175
)
SELECT * FROM test_data
INTO #window_test_data;
GO

-- Step 2: Test Positional Window Functions (LAG, LEAD, FIRST_VALUE, LAST_VALUE)
SELECT 
    'Positional Functions Test' as test_category,
    dept,
    name,
    quarter,
    sales,
    -- LAG tests
    LAG(sales, 1) OVER (PARTITION BY dept, name ORDER BY quarter) as lag1,
    LAG(sales, 2, -999) OVER (PARTITION BY dept, name ORDER BY quarter) as lag2_with_default,
    -- LEAD tests
    LEAD(sales, 1) OVER (PARTITION BY dept, name ORDER BY quarter) as lead1,
    LEAD(sales, 2, -999) OVER (PARTITION BY dept, name ORDER BY quarter) as lead2_with_default,
    -- FIRST_VALUE and LAST_VALUE
    FIRST_VALUE(sales) OVER (PARTITION BY dept ORDER BY quarter) as first_in_dept,
    LAST_VALUE(sales) OVER (PARTITION BY dept ORDER BY quarter ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) as last_in_dept
FROM #window_test_data
ORDER BY dept, name, quarter;
GO

-- Step 3: Test Ranking Functions (ROW_NUMBER, RANK, DENSE_RANK)
SELECT 
    'Ranking Functions Test' as test_category,
    dept,
    name,
    quarter,
    sales,
    -- ROW_NUMBER
    ROW_NUMBER() OVER (ORDER BY sales DESC) as global_row_num,
    ROW_NUMBER() OVER (PARTITION BY dept ORDER BY sales DESC) as dept_row_num,
    -- RANK (with ties)
    RANK() OVER (ORDER BY sales DESC) as global_rank,
    RANK() OVER (PARTITION BY dept ORDER BY sales DESC) as dept_rank,
    -- DENSE_RANK
    DENSE_RANK() OVER (ORDER BY sales DESC) as global_dense_rank,
    DENSE_RANK() OVER (PARTITION BY dept ORDER BY sales DESC) as dept_dense_rank
FROM #window_test_data
ORDER BY sales DESC;
GO

-- Step 4: Test Aggregate Window Functions with Different Frame Specifications
SELECT 
    'Aggregate Functions Test' as test_category,
    dept,
    name,
    quarter,
    sales,
    -- SUM with different frames
    SUM(sales) OVER (PARTITION BY dept) as dept_total,
    SUM(sales) OVER (PARTITION BY dept ORDER BY quarter ROWS UNBOUNDED PRECEDING) as running_total,
    SUM(sales) OVER (PARTITION BY dept ORDER BY quarter ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) as sum_3_rows,
    -- AVG with different frames
    ROUND(AVG(sales) OVER (PARTITION BY dept), 2) as dept_avg,
    ROUND(AVG(sales) OVER (PARTITION BY dept ORDER BY quarter ROWS 2 PRECEDING), 2) as moving_avg_3,
    -- MIN/MAX
    MIN(sales) OVER (PARTITION BY dept) as dept_min,
    MAX(sales) OVER (PARTITION BY dept) as dept_max,
    MIN(sales) OVER (ORDER BY quarter ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as min_last_3,
    MAX(sales) OVER (ORDER BY quarter ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as max_last_3,
    -- COUNT
    COUNT(*) OVER (PARTITION BY dept) as dept_count,
    COUNT(*) OVER (ORDER BY quarter ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_count
FROM #window_test_data
WHERE dept = 'A'  -- Focus on one department for clarity
ORDER BY name, quarter;
GO

-- Step 5: Test Complex Expressions with Window Functions
SELECT 
    'Expression Test' as test_category,
    dept,
    name,
    quarter,
    sales,
    LAG(sales, 1) OVER (PARTITION BY name ORDER BY quarter) as prev_sales,
    -- Expressions using window functions
    sales - LAG(sales, 1, sales) OVER (PARTITION BY name ORDER BY quarter) as quarter_change,
    ROUND(100.0 * (sales - LAG(sales, 1, sales) OVER (PARTITION BY name ORDER BY quarter)) / 
          NULLIF(LAG(sales, 1, sales) OVER (PARTITION BY name ORDER BY quarter), 0), 2) as pct_change,
    -- Multiple window functions in expression
    ROUND(sales * 100.0 / SUM(sales) OVER (PARTITION BY dept), 2) as dept_sales_pct,
    -- Cumulative calculations
    SUM(sales) OVER (PARTITION BY name ORDER BY quarter ROWS UNBOUNDED PRECEDING) as cumulative_sales,
    ROUND(AVG(sales) OVER (PARTITION BY name ORDER BY quarter ROWS UNBOUNDED PRECEDING), 2) as cumulative_avg
FROM #window_test_data
ORDER BY dept, name, quarter;
GO

-- Step 6: Test Window Functions with NULLs
WITH null_test AS (
    SELECT 1 as id, 10 as value UNION ALL
    SELECT 2, NULL UNION ALL
    SELECT 3, 30 UNION ALL
    SELECT 4, NULL UNION ALL
    SELECT 5, 50
)
SELECT * FROM null_test
INTO #window_null_test;
GO

SELECT 
    'NULL Handling Test' as test_category,
    id,
    value,
    -- Window functions should handle NULLs correctly
    LAG(value, 1) OVER (ORDER BY id) as lag_value,
    LEAD(value, 1) OVER (ORDER BY id) as lead_value,
    SUM(value) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) as running_sum,
    AVG(value) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) as running_avg,
    COUNT(value) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) as count_non_null,
    COUNT(*) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) as count_all
FROM #window_null_test
ORDER BY id;
GO

-- Step 7: Test Frame Boundary Variations
SELECT 
    'Frame Boundaries Test' as test_category,
    dept,
    name,
    quarter,
    sales,
    -- ROWS frame
    SUM(sales) OVER (ORDER BY quarter ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as rows_unbounded_current,
    SUM(sales) OVER (ORDER BY quarter ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING) as rows_2_2,
    SUM(sales) OVER (ORDER BY quarter ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) as rows_current_unbounded,
    -- Edge cases
    AVG(sales) OVER (ORDER BY quarter ROWS BETWEEN 10 PRECEDING AND 10 FOLLOWING) as avg_large_window,
    COUNT(*) OVER (ORDER BY quarter ROWS BETWEEN 1 FOLLOWING AND 2 FOLLOWING) as count_future_only
FROM #window_test_data
WHERE name = 'Alice'
ORDER BY quarter;
GO

-- Step 8: Performance validation - Create larger dataset
WITH series AS (
    SELECT 1 as n UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL 
    SELECT 5 UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL 
    SELECT 9 UNION ALL SELECT 10
),
large_data AS (
    SELECT 
        'Dept' || ((s1.n * 100 + s2.n * 10 + s3.n) % 5) as dept,
        'User' || ((s1.n * 100 + s2.n * 10 + s3.n) % 20) as user_name,
        s1.n * 100 + s2.n * 10 + s3.n as transaction_id,
        ((s1.n * 100 + s2.n * 10 + s3.n) * 7 % 1000) + 100 as amount
    FROM series s1
    CROSS JOIN series s2
    CROSS JOIN series s3
)
SELECT * FROM large_data
INTO #perf_test;
GO

-- Simple performance validation (not exhaustive, just ensures it runs)
SELECT 
    'Performance Test' as test_category,
    COUNT(*) as total_rows,
    MIN(running_sum) as min_running_sum,
    MAX(running_sum) as max_running_sum,
    MIN(dept_rank) as min_rank,
    MAX(dept_rank) as max_rank
FROM (
    SELECT 
        dept,
        user_name,
        transaction_id,
        amount,
        SUM(amount) OVER (PARTITION BY dept ORDER BY transaction_id ROWS UNBOUNDED PRECEDING) as running_sum,
        RANK() OVER (PARTITION BY dept ORDER BY amount DESC) as dept_rank
    FROM #perf_test
) t;
GO

-- Step 9: Verify All Window Function Categories
WITH summary AS (
    SELECT 
        'Summary' as test_category,
        'LAG/LEAD' as function_type,
        COUNT(DISTINCT LAG(sales, 1) OVER (ORDER BY quarter)) as distinct_values
    FROM #window_test_data
    UNION ALL
    SELECT 
        'Summary',
        'ROW_NUMBER',
        COUNT(DISTINCT ROW_NUMBER() OVER (ORDER BY sales))
    FROM #window_test_data
    UNION ALL
    SELECT 
        'Summary',
        'RANK/DENSE_RANK',
        COUNT(DISTINCT RANK() OVER (ORDER BY sales))
    FROM #window_test_data
    UNION ALL
    SELECT 
        'Summary',
        'SUM/AVG',
        COUNT(DISTINCT SUM(sales) OVER (PARTITION BY dept))
    FROM #window_test_data
    UNION ALL
    SELECT 
        'Summary',
        'MIN/MAX',
        COUNT(DISTINCT MAX(sales) OVER (PARTITION BY dept))
    FROM #window_test_data
    UNION ALL
    SELECT 
        'Summary',
        'FIRST/LAST_VALUE',
        COUNT(DISTINCT FIRST_VALUE(sales) OVER (PARTITION BY dept ORDER BY quarter))
    FROM #window_test_data
)
SELECT * FROM summary
ORDER BY function_type;
GO