-- #! ../data/sales_data.csv

-- ================================================================
-- ORDER BY Expression Support Examples
-- ================================================================
-- Demonstrates the new ORDER BY expression support with automatic
-- aggregate rewriting via the OrderByAliasTransformer.
--
-- The transformer automatically rewrites ORDER BY aggregate expressions
-- to use aliases from the SELECT clause, enabling standard SQL syntax.
-- ================================================================

-- Example 1: Basic ORDER BY with aggregate (most common pattern)
-- The SUM() in ORDER BY is automatically rewritten to use the 'total' alias
SELECT
    region,
    SUM(sales_amount) AS total
FROM sales
GROUP BY region
ORDER BY SUM(sales_amount) DESC;
GO

-- Example 2: COUNT(*) without explicit alias
-- Transformer auto-generates an alias (expr_N) and uses it
SELECT
    region,
    COUNT(*) AS count
FROM sales
GROUP BY region
ORDER BY COUNT(*) DESC;
GO

-- Example 3: Multiple aggregates in ORDER BY
-- Both aggregates are rewritten to use their respective aliases
SELECT
    region,
    SUM(sales_amount) AS total_sales,
    AVG(sales_amount) AS avg_sales
FROM sales
GROUP BY region
ORDER BY SUM(sales_amount) DESC, AVG(sales_amount) ASC;
GO

-- Example 4: Mix of aggregate and simple column in ORDER BY
SELECT
    region,
    product,
    SUM(sales_amount) AS total
FROM sales
GROUP BY region, product
ORDER BY region ASC, SUM(sales_amount) DESC;
GO

-- Example 5: Using MIN and MAX aggregates
SELECT
    region,
    MIN(sales_amount) AS min_sale,
    MAX(sales_amount) AS max_sale
FROM sales
GROUP BY region
ORDER BY MAX(sales_amount) DESC, MIN(sales_amount) ASC;
GO

-- Example 6: COUNT without explicit alias in SELECT
-- Parser auto-generates expr_N, transformer uses that
SELECT
    region,
    COUNT(*)
FROM sales
GROUP BY region
ORDER BY COUNT(*) DESC;
GO

-- Example 7: AVG with ORDER BY
SELECT
    product,
    AVG(sales_amount) AS average_price
FROM sales
GROUP BY product
ORDER BY AVG(sales_amount) DESC;
GO

-- ================================================================
-- How It Works
-- ================================================================
--
-- Before transformation:
--   ORDER BY SUM(sales_amount) DESC
--
-- After transformation:
--   ORDER BY total DESC
--
-- The OrderByAliasTransformer:
-- 1. Finds all aggregates in SELECT clause
-- 2. Maps them to their aliases (or generates aliases)
-- 3. Rewrites matching aggregates in ORDER BY to use the aliases
-- 4. Executor only sees simple column references
--
-- Use --show-transformations flag to see the rewriting in action!
-- ================================================================

-- Example 8: Testing with --show-transformations
-- Run this with: ./target/release/sql-cli -f examples/order_by_expressions.sql --show-transformations
SELECT
    region,
    COUNT(*) AS num_sales,
    SUM(sales_amount) AS total,
    AVG(sales_amount) AS average
FROM sales
GROUP BY region
ORDER BY SUM(sales_amount) DESC, COUNT(*) DESC;
GO
