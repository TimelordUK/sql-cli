-- #! ../data/sales_data.csv
-- ====================================================================
-- SQL CLI Preprocessor & Expander Showcase
-- ====================================================================
-- This file demonstrates all the query rewriting transformations that
-- enable advanced SQL features through AST preprocessing.
--
-- All these transformations are ENABLED BY DEFAULT (opt-out via flags):
--   --no-expression-lifter    Disable expression lifting
--   --no-where-expansion      Disable WHERE alias expansion
--   --no-group-by-expansion   Disable GROUP BY alias expansion
--   --no-having-expansion     Disable HAVING auto-aliasing
--   --no-cte-hoister          Disable CTE hoisting
--   --no-in-lifter            Disable IN operator optimization
--
-- Why these matter: They allow us to support SQL features that would
-- otherwise be impossible or very complex to implement directly in the
-- query executor. By rewriting the AST before execution, we can:
--   1. Support standard SQL syntax (WHERE/GROUP BY with SELECT aliases)
--   2. Enable complex expressions with window functions
--   3. Automatically optimize queries (CTE hoisting, IN lists)
--   4. Fill feature gaps without major executor rewrites
-- ====================================================================

-- ====================================================================
-- 1. WHERE ALIAS EXPANSION (WhereAliasExpander)
-- ====================================================================
-- Problem: Query executors typically can't reference SELECT aliases in WHERE
-- Solution: Expand the alias back to its original expression
--
-- Input SQL:
--   SELECT amount * 1.1 AS adjusted, region FROM sales WHERE adjusted > 1000
--
-- Rewritten to:
--   SELECT amount * 1.1 AS adjusted, region FROM sales WHERE (amount * 1.1) > 1000
--
-- This allows natural SQL that would fail in most engines!

SELECT
    sales_amount * 1.1 AS adjusted_amount,
    region
FROM sales
WHERE adjusted_amount > 1000;
GO

-- ====================================================================
-- 2. GROUP BY ALIAS EXPANSION (GroupByAliasExpander)
-- ====================================================================
-- Problem: GROUP BY with SELECT aliases isn't supported natively
-- Solution: Expand aliases in GROUP BY to their full expressions
--
-- Input SQL:
--   SELECT UPPER(region) AS region_upper, SUM(amount) FROM sales GROUP BY region_upper
--
-- Rewritten to:
--   SELECT UPPER(region) AS region_upper, SUM(amount) FROM sales GROUP BY UPPER(region)
--
-- Standard SQL that just works!

SELECT
    UPPER(region) AS region_code,
    SUM(sales_amount) AS total_sales
FROM sales
GROUP BY region_code;
GO

-- ====================================================================
-- 3. HAVING AUTO-ALIASING (HavingAliasTransformer)
-- ====================================================================
-- Problem: HAVING with aggregate expressions requires special handling
-- Solution: Automatically create aliases for aggregates in SELECT, use in HAVING
--
-- Input SQL:
--   SELECT region, SUM(amount) FROM sales GROUP BY region HAVING SUM(amount) > 5000
--
-- Rewritten to:
--   SELECT region, SUM(amount) AS _agg_0 FROM sales GROUP BY region HAVING _agg_0 > 5000
--
-- This allows HAVING to work with aggregate results efficiently!

SELECT
    region,
    COUNT(*) AS sale_count,
    SUM(sales_amount) AS total
FROM sales
GROUP BY region
HAVING SUM(sales_amount) > 50000;
GO

-- ====================================================================
-- 4. EXPRESSION LIFTER (ExpressionLifterTransformer)
-- ====================================================================
-- Problem: Window functions and complex expressions in WHERE/GROUP BY/HAVING
-- Solution: Lift expressions to CTE, then reference the computed column
--
-- Input SQL:
--   SELECT region, amount,
--          ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) AS rn
--   FROM sales
--   WHERE ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) = 1
--
-- Rewritten to:
--   WITH _lifted_0 AS (
--       SELECT region, amount,
--              ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) AS rn
--       FROM sales
--   )
--   SELECT region, amount, rn FROM _lifted_0 WHERE rn = 1
--
-- This is the BIG ONE - it enables window functions in filters!

SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rank
FROM sales
WHERE ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) <= 3;
GO

-- Another expression lifter example: Complex calculations in GROUP BY
SELECT
    CASE
        WHEN sales_amount > 20000 THEN 'High'
        WHEN sales_amount > 15000 THEN 'Medium'
        ELSE 'Low'
    END AS amount_tier,
    COUNT(*) AS tier_count
FROM sales
GROUP BY CASE
    WHEN sales_amount > 20000 THEN 'High'
    WHEN sales_amount > 15000 THEN 'Medium'
    ELSE 'Low'
END;
GO

-- ====================================================================
-- 5. CTE HOISTER (CTEHoisterTransformer)
-- ====================================================================
-- Problem: Nested CTEs or CTEs in subqueries aren't easily optimized
-- Solution: Hoist all CTEs to the top level for better organization
--
-- Input SQL:
--   SELECT * FROM (
--       WITH regional AS (SELECT region, SUM(amount) AS total FROM sales GROUP BY region)
--       SELECT * FROM regional WHERE total > 1000
--   ) AS filtered
--
-- Rewritten to:
--   WITH regional AS (SELECT region, SUM(amount) AS total FROM sales GROUP BY region)
--   SELECT * FROM (SELECT * FROM regional WHERE total > 1000) AS filtered
--
-- Cleaner AST structure for execution!

-- Note: This example would require subquery support, which we may not have yet
-- But the transformer is ready for when we add that feature!

-- ====================================================================
-- 6. IN OPERATOR LIFTER (InOperatorLifterTransformer)
-- ====================================================================
-- Problem: Large IN lists (100s of values) are inefficient to process
-- Solution: Convert to temporary table joins (when temp table support exists)
--
-- Input SQL:
--   SELECT * FROM sales WHERE region IN ('East', 'West', 'North', 'South', ...)
--
-- Rewritten to (conceptually):
--   WITH _in_list_0 AS (SELECT 'East' AS value UNION SELECT 'West' ...)
--   SELECT * FROM sales WHERE region IN (SELECT value FROM _in_list_0)
--
-- Future optimization for large IN lists!

SELECT
    region,
    SUM(sales_amount) AS total
FROM sales
WHERE region IN ('East', 'West', 'North')
GROUP BY region;
GO

-- ====================================================================
-- COMBINED EXAMPLE: Multiple Transformers Working Together
-- ====================================================================
-- This query uses:
--   1. Expression lifter (window function in WHERE)
--   2. WHERE alias expansion (revenue_rank)
--   3. GROUP BY alias expansion (region_code)
--   4. HAVING auto-aliasing (COUNT(*))

SELECT
    UPPER(region) AS region_code,
    COUNT(*) AS deal_count,
    SUM(sales_amount) AS total_revenue,
    ROW_NUMBER() OVER (ORDER BY SUM(sales_amount) DESC) AS revenue_rank
FROM sales
WHERE ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) <= 5
GROUP BY UPPER(region)
HAVING COUNT(*) > 2
ORDER BY revenue_rank;
GO

-- ====================================================================
-- WHY THIS MATTERS
-- ====================================================================
-- Without these transformers, we would need to:
--   1. Implement complex WHERE/GROUP BY/HAVING logic in the executor
--   2. Handle window functions in filters (very complex)
--   3. Manage nested CTEs and subqueries
--   4. Optimize large IN lists manually
--
-- With these transformers, we:
--   - Write SQL that looks natural and standard
--   - Get automatic optimization and rewriting
--   - Can add new features by adding new transformers
--   - Keep the query executor simple and focused
--
-- This is the "secret sauce" that makes SQL CLI powerful!
-- ====================================================================

-- ====================================================================
-- TESTING TRANSFORMERS
-- ====================================================================
-- To see what's happening, use --show-preprocessing:
--   ./target/release/sql-cli -f examples/expander_rewriters.sql --show-preprocessing
--
-- To disable specific transformers:
--   ./target/release/sql-cli -f examples/expander_rewriters.sql --no-expression-lifter
--   ./target/release/sql-cli -f examples/expander_rewriters.sql --no-where-expansion
--
-- To test in single query mode (-q):
--   ./target/release/sql-cli -q "SELECT amount * 1.1 AS adj FROM sales WHERE adj > 1000" -d data/sales_data.csv
-- ====================================================================
