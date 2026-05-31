-- #! ../data/us_states.csv
-- ============================================================
--  Subqueries inside CASE expressions
--  -------------------------------------------------------------
--  Data: data/us_states.csv  (state, name, latitude, longitude)
--
--  Until 2026-05-31 a subquery inside a CASE branch errored with
--    "Unsupported expression type for arithmetic evaluation:
--     ScalarSubquery {...}"  /  "... InList {...}"
--  because SubqueryExecutor never recursed into CASE branches and the
--  arithmetic evaluator had no IN-list dispatch. Both are fixed now, so
--  scalar subqueries AND IN/NOT IN subqueries work anywhere inside CASE.
-- ============================================================


-- ---- 1. Scalar subqueries in CASE -----------------------------
-- Each WHEN compares a row value against a single-row sub-select
-- (MAX / AVG over the whole table). Previously this needed one
-- separate query per band; now it is a single labelled projection.
SELECT
    name,
    ROUND(latitude, 2) AS latitude,
    CASE
        WHEN latitude = (SELECT MAX(latitude) FROM us_states) THEN 'northern extreme'
        WHEN latitude > (SELECT AVG(latitude) FROM us_states) THEN 'above average'
        ELSE 'below average'
    END AS latitude_band
FROM us_states
WHERE state IN ('AK', 'WA', 'ME', 'CO', 'TX', 'FL', 'HI')
ORDER BY latitude DESC;
GO


-- ---- 2. IN subquery in CASE -----------------------------------
-- The CASE condition is a membership test against a sub-select.
-- The IN-subquery is pre-executed into an in-list, then evaluated
-- per row inside the CASE.
SELECT
    name,
    CASE
        WHEN state IN (SELECT state FROM us_states WHERE latitude > 45)
            THEN 'far north'
        ELSE 'rest'
    END AS region
FROM us_states
WHERE state IN ('AK', 'WA', 'ME', 'TX', 'HI')
ORDER BY name;
GO


-- ---- 3. NOT IN subquery in CASE -------------------------------
-- The complementary NOT IN form resolves through the same path.
SELECT
    name,
    CASE
        WHEN state NOT IN (SELECT state FROM us_states WHERE latitude > 30)
            THEN 'tropical / far south'
        ELSE 'temperate'
    END AS climate_hint
FROM us_states
WHERE state IN ('AK', 'HI', 'FL', 'ME')
ORDER BY name;
GO
