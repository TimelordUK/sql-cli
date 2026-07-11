-- #! ../data/numbers_1_100.csv
--
-- Self-joins and FROM-less subqueries
-- ===================================
-- These queries demonstrate join features that were brought to DuckDB parity
-- in v1.80.0 (see docs/SQL_PARITY.md, issues P4 and P5). Every result here
-- matches DuckDB 1.5.4 bit-for-bit via tests/comparison/runner.py.
--
--   * P4 — a base table can now be joined to itself with aliases
--          (`FROM numbers a JOIN numbers b ...`).
--   * P5 — a FROM-less subquery (`SELECT 1 AS k`) yields exactly one row,
--          so `CROSS JOIN (SELECT ...)` has the correct cardinality.
--
-- The source is numbers 1..100 (column `n`); most queries constrain to 1..10
-- to keep the output easy to read.

-- --------------------------------------------------------------------------
-- 1. Triangular running total: for each n, the sum of all numbers up to n.
--    Classic self-join on `a.n >= b.n`. STRING_AGG lists the actual members
--    that make up each total, so you can see 1+2+...+n = running_total.
--    Expect 1, 3, 6, 10, ..., 55 with members "1", "1,2", "1,2,3", ...
-- --------------------------------------------------------------------------
SELECT
    a.n                     AS n,
    SUM(b.n)                AS running_total,
    STRING_AGG(b.n, ',')    AS members
FROM numbers_1_100 a
INNER JOIN numbers_1_100 b ON a.n >= b.n
WHERE a.n <= 10
GROUP BY a.n
ORDER BY a.n;
GO

-- --------------------------------------------------------------------------
-- 2. Reverse of the above: for each n, the sum of the numbers strictly
--    greater than n (`a.n < b.n`). Written left-table-column first so the
--    join condition is unambiguous. Expect 54, 52, 49, ..., 10.
-- --------------------------------------------------------------------------
SELECT
    a.n            AS n,
    SUM(b.n)       AS sum_above
FROM numbers_1_100 a
INNER JOIN numbers_1_100 b ON a.n < b.n
WHERE a.n <= 10 AND b.n <= 10
GROUP BY a.n
ORDER BY a.n;
GO

-- --------------------------------------------------------------------------
-- 3. Companion to #1 over the reverse join (`a.n < b.n`): for each n, how many
--    numbers lie above it and what they are. COUNT + STRING_AGG on the same
--    self-join. (n=10 has nothing above it in 1..10, so it drops from the
--    INNER join.)
-- --------------------------------------------------------------------------
SELECT
    a.n                     AS n,
    COUNT(*)                AS count_above,
    STRING_AGG(b.n, ',')    AS numbers_above
FROM numbers_1_100 a
INNER JOIN numbers_1_100 b ON a.n < b.n
WHERE a.n <= 10 AND b.n <= 10
GROUP BY a.n
ORDER BY a.n;
GO

-- --------------------------------------------------------------------------
-- 4. Adjacency self-join: pair each number with its successor.
--    Demonstrates an equi-join on a computed expression over the same table.
--    Note the ON is written with the `a` (left) side first (`a.n + 1 = b.n`):
--    the engine currently binds a join condition's operands to the left/right
--    table by position, so keep the driving table's column on the left.
-- --------------------------------------------------------------------------
SELECT
    a.n            AS n,
    b.n            AS next_n
FROM numbers_1_100 a
INNER JOIN numbers_1_100 b ON a.n + 1 = b.n
WHERE a.n <= 10
ORDER BY a.n;
GO

-- --------------------------------------------------------------------------
-- 5. CROSS JOIN to a FROM-less constant subquery (P5). The subquery
--    `SELECT 1 AS tag` is a single constant row, so this tags each of the
--    first 5 numbers exactly once — 5 rows out, not 5xN.
-- --------------------------------------------------------------------------
SELECT
    a.n            AS n,
    c.tag          AS tag
FROM numbers_1_100 a
CROSS JOIN (SELECT 1 AS tag) c
WHERE a.n <= 5
ORDER BY a.n;
GO
