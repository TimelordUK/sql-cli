-- Boolean Aggregates & Conditional Expressions
-- Demonstrates boolean expressions inside aggregate functions,
-- unary minus, and qualified PARTITION BY columns.

-- Basic boolean aggregates: what percentage of values meet a condition?
WITH nums AS (SELECT value as x FROM RANGE(1, 100))
SELECT
    COUNT(*) as total,
    SUM(x > 50) as above_50,
    ROUND(AVG(x > 50) * 100, 1) as pct_above_50,
    SUM(x % 2 = 0) as even_count,
    ROUND(AVG(x % 2 = 0) * 100, 1) as pct_even
FROM nums;
GO

-- Compound boolean conditions in aggregates
WITH nums AS (SELECT value as x FROM RANGE(1, 100))
SELECT
    SUM(x > 25 AND x <= 75) as in_middle_half,
    ROUND(AVG(x > 25 AND x <= 75) * 100, 1) as pct_middle,
    SUM(x <= 10 OR x > 90) as in_tails,
    ROUND(AVG(x <= 10 OR x > 90) * 100, 1) as pct_tails
FROM nums;
GO

-- Composability with library functions: what % of numbers are prime?
WITH nums AS (SELECT value as x FROM RANGE(1, 100))
SELECT
    COUNT(*) as total,
    SUM(IS_PRIME(x)) as prime_count,
    ROUND(AVG(IS_PRIME(x)) * 100, 1) as prime_pct
FROM nums;
GO

-- Unary minus in CASE expressions — the stock trading pattern
WITH trades AS (
    SELECT 'AAPL' as stock, 'Buy' as op, 150 as price
    UNION ALL SELECT 'AAPL', 'Buy', 145
    UNION ALL SELECT 'AAPL', 'Sell', 175
    UNION ALL SELECT 'AAPL', 'Sell', 160
    UNION ALL SELECT 'GOOG', 'Buy', 2800
    UNION ALL SELECT 'GOOG', 'Sell', 2750
)
SELECT
    stock,
    SUM(CASE WHEN op = 'Buy' THEN -price ELSE price END) as capital_gain_loss,
    COUNT(*) as trades
FROM trades
GROUP BY stock
ORDER BY capital_gain_loss DESC;
GO

-- Unary minus in arithmetic expressions
WITH measurements AS (SELECT value as temp_c FROM RANGE(-10, 10))
SELECT
    temp_c,
    -temp_c as negated,
    ABS(-temp_c) as absolute,
    CASE WHEN temp_c < 0 THEN -temp_c ELSE temp_c END as manual_abs
FROM measurements
ORDER BY temp_c;
GO

-- PARTITION BY with qualified column names
WITH sales AS (
    SELECT 'Alice' as rep, 'North' as region, 500 as amount
    UNION ALL SELECT 'Bob', 'North', 700
    UNION ALL SELECT 'Alice', 'South', 300
    UNION ALL SELECT 'Carol', 'South', 800
    UNION ALL SELECT 'Bob', 'South', 600
    UNION ALL SELECT 'Carol', 'North', 450
)
SELECT
    rep, region, amount,
    DENSE_RANK() OVER (PARTITION BY sales.region ORDER BY amount DESC) as region_rank,
    SUM(amount) OVER (PARTITION BY sales.region) as region_total
FROM sales
ORDER BY region, region_rank;
GO

-- Conditional aggregates grouped by category
WITH products AS (
    SELECT value as id,
           CASE WHEN value % 3 = 0 THEN 'A' WHEN value % 3 = 1 THEN 'B' ELSE 'C' END as category,
           value * 10 + 5 as price
    FROM RANGE(1, 30)
)
SELECT
    category,
    COUNT(*) as total,
    ROUND(AVG(price > 200) * 100, 1) as pct_expensive,
    SUM(price > 100 AND price <= 200) as mid_range_count,
    SUM(IS_PRIME(id)) as prime_ids
FROM products
GROUP BY category
ORDER BY category;
GO
