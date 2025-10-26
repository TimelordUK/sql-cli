-- #! ../data/test_simple_math.csv
-- Alias Expansion Transformers - Feature Reference
-- ================================================
-- This file demonstrates what the preprocessing pipeline's alias expansion
-- transformers support, and what could be supported with additional work.
--
-- USAGE:
--   sql-cli examples/expansion_transformers.sql
--
-- Or test individual queries:
--   sql-cli data/test_simple_math.csv -q "SELECT ..." -o table

-- ============================================================================
-- SUPPORTED FEATURES (Working Now!)
-- ============================================================================

-- ----------------------------------------------------------------------------
-- Quick Win #1: HAVING Auto-Aliasing
-- ----------------------------------------------------------------------------
-- Automatically adds aliases to aggregates and rewrites HAVING to use them
-- Before: SELECT region, COUNT(*) FROM sales GROUP BY region HAVING COUNT(*) > 5
-- After:  SELECT region, COUNT(*) as __agg_1 FROM sales GROUP BY region HAVING __agg_1 > 5

-- Example 1: Basic HAVING with auto-aliasing
SELECT a, COUNT(*)
FROM test_simple_math
GROUP BY a
HAVING COUNT(*) >= 1;

-- Example 2: HAVING with GROUP BY alias (two transformers working together!)
SELECT id % 3 as grp, COUNT(*), SUM(b)
FROM test_simple_math
GROUP BY grp
HAVING COUNT(*) > 2 AND SUM(b) > 100;

-- ----------------------------------------------------------------------------
-- Quick Win #2: WHERE Alias Expansion
-- ----------------------------------------------------------------------------
-- Expands SELECT aliases to their full expressions in WHERE clauses
-- Works with: =, <, >, <=, >=, !=, IS NULL, IS NOT NULL, AND, OR, NOT

SELECT a, b, a * 2 as double_a
FROM test_simple_math
WHERE double_a > 10;
GO

SELECT a, b, b / 10 as tens
FROM test_simple_math
WHERE tens > 5;
GO

SELECT a, b, a * 2 as double_a, a * 3 as triple_a
FROM test_simple_math
WHERE double_a > 10 AND triple_a < 25;
GO

-- Arithmetic operators work
SELECT a, a + 5 as plus5
FROM test_simple_math
WHERE plus5 > 15;
GO

SELECT a, a - 3 as minus3
FROM test_simple_math
WHERE minus3 < 5;
GO

SELECT a, a * 2 as times2
FROM test_simple_math
WHERE times2 = 20;
GO

SELECT a, b / 10 as divided
FROM test_simple_math
WHERE divided >= 10;
GO

SELECT a, a % 3 as modulo
FROM test_simple_math
WHERE modulo = 0;
GO

-- IS NULL / IS NOT NULL work with aliases
SELECT a, b, a * 2 as doubled
FROM test_simple_math
WHERE doubled IS NOT NULL;
GO

-- Multiple uses of same alias
SELECT a, a * 2 as double_a
FROM test_simple_math
WHERE double_a > 10 AND double_a < 30;
GO

-- ----------------------------------------------------------------------------
-- Quick Win #3: GROUP BY Alias Expansion
-- ----------------------------------------------------------------------------
-- Expands SELECT aliases to their full expressions in GROUP BY clauses

SELECT id % 3 as grp, COUNT(*)
FROM test_simple_math
GROUP BY grp;
GO

SELECT b / 10 as tens, COUNT(*), AVG(a) as avg_a
FROM test_simple_math
GROUP BY tens;
GO

-- Multiple aliases in GROUP BY
SELECT a % 2 as even_odd, a % 3 as mod3, COUNT(*)
FROM test_simple_math
GROUP BY even_odd, mod3;
GO

-- ----------------------------------------------------------------------------
-- All Three Transformers Working Together
-- ----------------------------------------------------------------------------

SELECT id % 3 as grp, COUNT(*) as cnt, SUM(b) as total
FROM test_simple_math
WHERE b > 50                  -- WHERE: can use simple comparisons
GROUP BY grp                  -- GROUP BY: alias expansion
HAVING cnt > 2 AND total > 100  -- HAVING: aggregate auto-aliasing
ORDER BY total DESC;          -- ORDER BY: works naturally (evaluated after SELECT)
GO

-- ----------------------------------------------------------------------------
-- DISTINCT with Expressions (Already Works)
-- ----------------------------------------------------------------------------

SELECT DISTINCT a % 3 FROM test_simple_math;
GO

SELECT DISTINCT a % 2, a % 3 FROM test_simple_math;
GO

-- ============================================================================
-- END OF WORKING FEATURES
-- ============================================================================
-- Everything above works perfectly when run with -q flag or in interactive mode.
--
-- Test examples:
-- sql-cli data/test_simple_math.csv -q "SELECT a, a*2 as double_a FROM test_simple_math WHERE double_a > 10"
-- sql-cli data/test_simple_math.csv -q "SELECT id % 3 as grp, COUNT(*) FROM test_simple_math GROUP BY grp"
-- sql-cli -q "SELECT value % 3 as grp, COUNT(*) as cnt FROM range(20) WHERE value > 5 GROUP BY grp HAVING cnt > 3"

-- ============================================================================
-- FUTURE ENHANCEMENTS (Not Yet Supported)
-- ============================================================================
-- The features below would require additional work on the WHERE evaluator
-- or other parts of the query engine. They are documented here as potential
-- future improvements.

-- ----------------------------------------------------------------------------
-- CASE Expressions in WHERE with Aliases
-- ----------------------------------------------------------------------------
-- Status: NOT SUPPORTED
-- Issue: WHERE evaluator doesn't handle CASE expressions
-- Workaround: Use the full CASE expression in WHERE, or use a CTE
-- Example that currently fails:

-- SELECT value,
--        CASE WHEN value > 5 THEN 'high' ELSE 'low' END as category
-- FROM range(10)
-- WHERE category = 'high';

-- Workaround using CTE:
-- WITH categorized AS (
--   SELECT value,
--          CASE WHEN value > 5 THEN 'high' ELSE 'low' END as category
--   FROM range(10)
-- )
-- SELECT * FROM categorized WHERE category = 'high';

-- ----------------------------------------------------------------------------
-- BETWEEN with Expression Aliases
-- ----------------------------------------------------------------------------
-- Status: NOT SUPPORTED
-- Issue: WHERE evaluator expects simple column references in BETWEEN
-- Example that currently fails:

-- SELECT value, value * 2 as doubled
-- FROM range(10)
-- WHERE doubled BETWEEN 10 AND 20;

-- Workaround: Use comparison operators
-- SELECT value, value * 2 as doubled
-- FROM range(10)
-- WHERE doubled >= 10 AND doubled <= 20;

-- ----------------------------------------------------------------------------
-- IN with Expression Aliases
-- ----------------------------------------------------------------------------
-- Status: NOT SUPPORTED
-- Issue: WHERE evaluator expects simple column references in IN lists
-- Example that currently fails:

-- SELECT value, value * 2 as doubled
-- FROM range(10)
-- WHERE doubled IN (4, 6, 8);

-- Workaround: Use multiple OR conditions
-- SELECT value, value * 2 as doubled
-- FROM range(10)
-- WHERE doubled = 4 OR doubled = 6 OR doubled = 8;

-- ----------------------------------------------------------------------------
-- Subquery Column References
-- ----------------------------------------------------------------------------
-- Status: COMPLEX - Not a simple enhancement
-- Would require significant changes to subquery evaluation

-- ----------------------------------------------------------------------------
-- JOIN ON with Aliases
-- ----------------------------------------------------------------------------
-- Status: UNCLEAR SEMANTICS
-- Issue: Which table's alias? Confusing for users
-- Probably not worth supporting

-- ============================================================================
-- IMPLEMENTATION NOTES
-- ============================================================================

-- The preprocessing pipeline runs transformers in this order:
-- 1. ExpressionLifter - Lifts window functions and column alias dependencies
-- 2. WhereAliasExpander - Expands SELECT aliases in WHERE (Quick Win #2)
-- 3. GroupByAliasExpander - Expands SELECT aliases in GROUP BY (Quick Win #3)
-- 4. HavingAliasTransformer - Adds aliases to aggregates, rewrites HAVING (Quick Win #1)
-- 5. CTEHoister - Hoists nested CTEs to top level
-- 6. InOperatorLifter - Optimizes large IN expressions

-- Performance Impact:
-- - Transformation overhead: ~0.06ms total for all transformers
-- - Zero runtime overhead - aliases expanded at query planning time
-- - Same execution performance as writing full expressions

-- SQL Evaluation Order (Standard):
-- FROM → WHERE → GROUP BY → HAVING → SELECT → ORDER BY → LIMIT
--
-- This is why:
-- - WHERE needs alias expansion (evaluated before SELECT)
-- - GROUP BY needs alias expansion (evaluated before SELECT)
-- - HAVING needs alias expansion (evaluated before SELECT, but after GROUP BY)
-- - ORDER BY works naturally (evaluated after SELECT, aliases already exist)

-- ============================================================================
-- TESTING
-- ============================================================================

-- Run the supported features:
-- $ sql-cli examples/expansion_transformers.sql

-- Run with preprocessing details:
-- $ sql-cli data/test_simple_math.csv -q "SELECT a, a*2 as d FROM test_simple_math WHERE d > 10" --show-preprocessing

-- Test individual features:
-- $ ./tests/integration/test_where_alias_expansion.sh
-- $ ./tests/integration/test_group_by_alias_expansion.sh
-- $ ./tests/integration/test_having_auto_alias.sh

-- Demo scripts:
-- $ ./scripts/demo_where_alias_expansion.sh
-- $ ./scripts/demo_group_by_alias_expansion.sh
-- $ ./scripts/demo_having_auto_alias.sh
