-- Comprehensive Window Functions Test Suite
-- This file tests all window function types with various configurations
-- to ensure batch evaluation produces identical results to per-row evaluation

-- =============================================================================
-- Test Dataset: Sales data with partitions and ordering
-- =============================================================================
-- Create test data: id, region, product, amount, sale_date
-- Regions: East, West, North (3 partitions)
-- Products: A, B, C
-- Amounts: varying values including NULLs
-- 15 rows total

-- Note: This test assumes data will be loaded from a CSV file
-- For now, we'll use inline data generation via UNION ALL

-- =============================================================================
-- LAG Function Tests
-- =============================================================================

-- Test 1: LAG with default offset (1) and no PARTITION BY
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (ORDER BY id) as prev_amount
FROM sales_test
ORDER BY id;
GO

-- Test 2: LAG with offset 2
SELECT
    id,
    region,
    amount,
    LAG(amount, 2) OVER (ORDER BY id) as prev_amount_2
FROM sales_test
ORDER BY id;
GO

-- Test 3: LAG with PARTITION BY
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount_in_region
FROM sales_test
ORDER BY region, id;
GO

-- Test 4: LAG with NULL values
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount
FROM sales_test
ORDER BY region, id;
GO

-- Test 5: LAG with offset beyond partition start (should return NULL)
SELECT
    id,
    region,
    amount,
    LAG(amount, 10) OVER (PARTITION BY region ORDER BY id) as prev_amount_10
FROM sales_test
ORDER BY region, id;
GO

-- =============================================================================
-- LEAD Function Tests
-- =============================================================================

-- Test 6: LEAD with default offset (1)
SELECT
    id,
    region,
    amount,
    LEAD(amount) OVER (ORDER BY id) as next_amount
FROM sales_test
ORDER BY id;
GO

-- Test 7: LEAD with offset 2 and PARTITION BY
SELECT
    id,
    region,
    amount,
    LEAD(amount, 2) OVER (PARTITION BY region ORDER BY id) as next_amount_2
FROM sales_test
ORDER BY region, id;
GO

-- Test 8: LEAD with offset beyond partition end (should return NULL)
SELECT
    id,
    region,
    amount,
    LEAD(amount, 10) OVER (PARTITION BY region ORDER BY id) as next_amount_10
FROM sales_test
ORDER BY region, id;
GO

-- =============================================================================
-- ROW_NUMBER Function Tests
-- =============================================================================

-- Test 9: ROW_NUMBER without PARTITION BY
SELECT
    id,
    region,
    amount,
    ROW_NUMBER() OVER (ORDER BY id) as row_num
FROM sales_test
ORDER BY id;
GO

-- Test 10: ROW_NUMBER with PARTITION BY
SELECT
    id,
    region,
    amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY id) as row_num_in_region
FROM sales_test
ORDER BY region, id;
GO

-- Test 11: ROW_NUMBER with different ORDER BY
SELECT
    id,
    region,
    amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) as row_num_by_amount
FROM sales_test
ORDER BY region, amount DESC;
GO

-- =============================================================================
-- RANK Function Tests
-- =============================================================================

-- Test 12: RANK without PARTITION BY
SELECT
    id,
    region,
    amount,
    RANK() OVER (ORDER BY amount) as rank_by_amount
FROM sales_test
ORDER BY amount, id;
GO

-- Test 13: RANK with PARTITION BY
SELECT
    id,
    region,
    amount,
    RANK() OVER (PARTITION BY region ORDER BY amount) as rank_in_region
FROM sales_test
ORDER BY region, amount, id;
GO

-- Test 14: RANK with ties (multiple rows with same amount)
SELECT
    id,
    region,
    amount,
    RANK() OVER (PARTITION BY region ORDER BY amount DESC) as rank_desc
FROM sales_test
ORDER BY region, amount DESC, id;
GO

-- =============================================================================
-- DENSE_RANK Function Tests
-- =============================================================================

-- Test 15: DENSE_RANK without PARTITION BY
SELECT
    id,
    region,
    amount,
    DENSE_RANK() OVER (ORDER BY amount) as dense_rank_by_amount
FROM sales_test
ORDER BY amount, id;
GO

-- Test 16: DENSE_RANK with PARTITION BY
SELECT
    id,
    region,
    amount,
    DENSE_RANK() OVER (PARTITION BY region ORDER BY amount) as dense_rank_in_region
FROM sales_test
ORDER BY region, amount, id;
GO

-- Test 17: DENSE_RANK vs RANK comparison (show the difference)
SELECT
    id,
    region,
    amount,
    RANK() OVER (ORDER BY amount) as rank_val,
    DENSE_RANK() OVER (ORDER BY amount) as dense_rank_val
FROM sales_test
ORDER BY amount, id;
GO

-- =============================================================================
-- Multiple Window Functions in One Query
-- =============================================================================

-- Test 18: Multiple window functions with same window spec
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount,
    LEAD(amount) OVER (PARTITION BY region ORDER BY id) as next_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY id) as row_num
FROM sales_test
ORDER BY region, id;
GO

-- Test 19: Multiple window functions with different window specs
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (ORDER BY id) as prev_amount_global,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount_region,
    ROW_NUMBER() OVER (ORDER BY amount DESC) as rank_by_amount
FROM sales_test
ORDER BY id;
GO

-- Test 20: All window functions together
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as lag_amt,
    LEAD(amount) OVER (PARTITION BY region ORDER BY id) as lead_amt,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY id) as row_num,
    RANK() OVER (PARTITION BY region ORDER BY amount) as rank_amt,
    DENSE_RANK() OVER (PARTITION BY region ORDER BY amount) as dense_rank_amt
FROM sales_test
ORDER BY region, id;
GO

-- =============================================================================
-- Edge Cases
-- =============================================================================

-- Test 21: Single row partition
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY product ORDER BY id) as prev_amount,
    LEAD(amount) OVER (PARTITION BY product ORDER BY id) as next_amount,
    ROW_NUMBER() OVER (PARTITION BY product ORDER BY id) as row_num
FROM sales_test
WHERE product = 'A'
ORDER BY id;
GO

-- Test 22: Empty result set handling
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (ORDER BY id) as prev_amount
FROM sales_test
WHERE id > 1000
ORDER BY id;
GO

-- Test 23: All NULL values in window
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY id) as row_num
FROM sales_test
WHERE amount IS NULL
ORDER BY id;
GO

-- Test 24: Mixed NULL and non-NULL values
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (ORDER BY id) as prev_amount,
    LEAD(amount) OVER (ORDER BY id) as next_amount
FROM sales_test
ORDER BY id;
GO

-- Test 25: Window function on different data types
SELECT
    id,
    region,
    product,
    sale_date,
    LAG(product) OVER (ORDER BY id) as prev_product,
    LAG(sale_date) OVER (ORDER BY id) as prev_date
FROM sales_test
ORDER BY id;
GO

-- =============================================================================
-- Performance Test Cases (for benchmarking)
-- =============================================================================

-- Test 26: Large dataset simulation (if sales_test has enough rows)
-- This tests performance with many rows
SELECT
    id,
    LAG(amount) OVER (ORDER BY id) as prev_amount
FROM sales_test
ORDER BY id;
GO

-- Test 27: Multiple partitions performance
SELECT
    id,
    region,
    amount,
    LAG(amount) OVER (PARTITION BY region ORDER BY id) as prev_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY id) as row_num
FROM sales_test
ORDER BY region, id;
GO

-- Test 28: Complex window with multiple order columns
SELECT
    id,
    region,
    product,
    amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY product, amount DESC, id) as complex_row_num
FROM sales_test
ORDER BY region, product, amount DESC, id;
GO
