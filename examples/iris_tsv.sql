-- Iris dataset (TSV) — demonstrating non-comma delimiters end-to-end.
--
-- The classic Fisher iris dataset, shipped as tab-separated values. Three
-- different surfaces all parse it correctly:
--
--   # Auto-detect from extension (.tsv → tab):
--   ./target/release/sql-cli data/iris.tsv -q "SELECT * FROM iris LIMIT 5"
--
--   # Same auto-detect inside SQL:
--   SELECT * FROM READ_CSV('data/iris.tsv') LIMIT 5;
--
--   # Explicit override (forces tab regardless of extension):
--   SELECT * FROM READ_CSV('data/iris.tsv', '\t') LIMIT 5;
--
--   # CLI flag override (wins over extension):
--   ./target/release/sql-cli --delimiter '\t' data/iris.tsv -q "..."
--
-- Run the whole script from the repo root:
--   ./target/release/sql-cli -f examples/iris_tsv.sql


-- ================================================================
-- SECTION 1 — extension auto-detect via READ_CSV
-- ================================================================
-- No 2nd arg needed; .tsv → tab is detected from the path.

-- Headline counts — how many rows per species?
SELECT Species, COUNT(*) AS samples
FROM READ_CSV('data/iris.tsv')
GROUP BY Species
ORDER BY Species;
GO

-- A peek at the first few rows so the columns are visible.
SELECT *
FROM READ_CSV('data/iris.tsv')
LIMIT 5;
GO


-- ================================================================
-- SECTION 2 — explicit delimiter override
-- ================================================================
-- The 2nd argument forces the delimiter. Useful when the file has an
-- unrecognised extension, or when you want to be explicit for clarity.
-- '\t' is the canonical way to spell tab in SQL string literals.

SELECT Species, ROUND(AVG(SepalLength), 2) AS avg_sepal_len
FROM READ_CSV('data/iris.tsv', '\t')
GROUP BY Species
ORDER BY avg_sepal_len DESC;
GO


-- ================================================================
-- SECTION 3 — per-species summary stats
-- ================================================================
-- Now that the TSV parses cleanly, the rest is just SQL. Classic
-- "min / avg / max per group" shape.

SELECT
    Species,
    COUNT(*)                    AS samples,
    ROUND(MIN(PetalLength), 2)  AS petal_len_min,
    ROUND(AVG(PetalLength), 2)  AS petal_len_avg,
    ROUND(MAX(PetalLength), 2)  AS petal_len_max,
    ROUND(AVG(PetalWidth), 2)   AS petal_width_avg
FROM READ_CSV('data/iris.tsv')
GROUP BY Species
ORDER BY petal_len_avg;
GO


-- ================================================================
-- SECTION 4 — derived columns and filtering
-- ================================================================
-- Compute petal area, flag "large" flowers, count by species.

WITH measured AS (
    SELECT
        Species,
        PetalLength,
        PetalWidth,
        PetalLength * PetalWidth AS petal_area
    FROM READ_CSV('data/iris.tsv')
)
SELECT
    Species,
    COUNT(*)                          AS large_flowers,
    ROUND(AVG(petal_area), 2)         AS avg_petal_area
FROM measured
WHERE petal_area > 5
GROUP BY Species
ORDER BY large_flowers DESC;
GO


-- ================================================================
-- SECTION 5 — sepal vs petal ratios via CTE
-- ================================================================
-- A common iris exploration: which species has the most elongated sepals
-- relative to its petals? The CTE pre-computes the ratio so the outer
-- query can aggregate it.

WITH ratios AS (
    SELECT
        Species,
        SepalLength / PetalLength AS sepal_to_petal_ratio
    FROM READ_CSV('data/iris.tsv')
)
SELECT
    Species,
    ROUND(MIN(sepal_to_petal_ratio), 2) AS ratio_min,
    ROUND(AVG(sepal_to_petal_ratio), 2) AS ratio_avg,
    ROUND(MAX(sepal_to_petal_ratio), 2) AS ratio_max
FROM ratios
GROUP BY Species
ORDER BY ratio_avg DESC;
GO


-- ================================================================
-- SECTION 6 — WEB CTE with file:// URL + DELIMITER
-- ================================================================
-- The same TSV file can be loaded as a WEB CTE. URLs don't have reliable
-- extensions (query strings, redirects), so DELIMITER must be explicit
-- when the source isn't comma-delimited.

WITH WEB iris_web AS (
    URL 'file://data/iris.tsv'
    FORMAT CSV
    DELIMITER '\t'
)
SELECT Species, COUNT(*) AS samples
FROM iris_web
GROUP BY Species
ORDER BY Species;
GO
