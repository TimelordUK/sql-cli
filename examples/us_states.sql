-- #! ../data/us_states.csv
-- ============================================================
--  US States Showcase
--  -------------------------------------------------------------
--  Two small CSVs:
--    data/us_states.csv     state,name,latitude,longitude  (52 rows)
--    data/state-areas.csv   name,area_sq_mi                (52 rows)
--
--  Demonstrates:
--    * basic GROUP BY histograms (REPEAT for ASCII bars)
--    * string functions (LENGTH, LEFT, REPLACE, FREQUENCY, UPPER)
--    * CTE chaining and CROSS JOIN of an aggregate "constant"
--    * the haversine formula using SIN/COS/ASIN/RADIANS/RADIUS_EARTH
--    * joining two CSVs via WEB CTEs
-- ============================================================


-- ---- 1. Quick look at the data --------------------------------
-- Show a few rows so the rest of the file makes sense.
SELECT state, name, latitude, longitude
FROM us_states
WHERE state IN ('AK','CA','FL','HI','ME','TX')
ORDER BY name;
GO


-- ---- 2. How long are state names? -----------------------------
-- LENGTH() + GROUP BY gives a histogram. REPEAT() draws ASCII bars
-- so the shape is visible without a chart library.
WITH lengths AS (
    SELECT name, LENGTH(name) AS n
    FROM us_states
)
SELECT n                    AS name_length,
       COUNT(*)             AS states,
       REPEAT('*', COUNT(*)) AS bar
FROM lengths
GROUP BY n
ORDER BY name_length;
GO


-- ---- 3. Distribution by starting letter -----------------------
-- LEFT(name, 1) is the first letter; HAVING filters the long tail.
SELECT LEFT(name, 1)         AS first_letter,
       COUNT(*)              AS states,
       REPEAT('#', COUNT(*)) AS bar
FROM us_states
GROUP BY LEFT(name, 1)
HAVING COUNT(*) >= 2
ORDER BY states DESC, first_letter;
GO


-- ---- 4. Vowel-heavy and consonant-heavy names -----------------
-- FREQUENCY counts substring occurrences. We strip spaces first
-- so multi-word names like "New York" use only their letters.
WITH cleaned AS (
    SELECT name,
           UPPER(REPLACE(name, ' ', '')) AS letters,
           LENGTH(REPLACE(name, ' ', '')) AS letter_count
    FROM us_states
),
counted AS (
    SELECT name, letter_count,
           FREQUENCY(letters, 'A') + FREQUENCY(letters, 'E')
         + FREQUENCY(letters, 'I') + FREQUENCY(letters, 'O')
         + FREQUENCY(letters, 'U') AS vowels
    FROM cleaned
)
SELECT name, letter_count, vowels,
       (letter_count - vowels)                       AS consonants,
       ROUND(100.0 * vowels / letter_count, 1)       AS vowel_pct
FROM counted
ORDER BY vowel_pct DESC, name
LIMIT 8;
GO


-- ---- 5. Compass extremes (lower 48) ---------------------------
-- AK, HI and PR are excluded so the answers are intuitive for the
-- contiguous United States. Each direction is a tiny ORDER BY/LIMIT.
SELECT 'Northernmost' AS direction, name, latitude, longitude
FROM us_states
WHERE state NOT IN ('AK','HI','PR')
ORDER BY latitude DESC LIMIT 1;
GO

SELECT 'Southernmost' AS direction, name, latitude, longitude
FROM us_states
WHERE state NOT IN ('AK','HI','PR')
ORDER BY latitude ASC LIMIT 1;
GO

SELECT 'Easternmost' AS direction, name, latitude, longitude
FROM us_states
WHERE state NOT IN ('AK','HI','PR')
ORDER BY longitude DESC LIMIT 1;
GO

SELECT 'Westernmost' AS direction, name, latitude, longitude
FROM us_states
WHERE state NOT IN ('AK','HI','PR')
ORDER BY longitude ASC LIMIT 1;
GO


-- ---- 6. Bounding box of the lower 48 --------------------------
-- A single-row CTE of the corners, then haversine between the
-- two opposite corners gives a "diagonal" of the contiguous USA.
WITH bbox AS (
    SELECT MAX(latitude)  AS lat_max,
           MIN(latitude)  AS lat_min,
           MAX(longitude) AS lon_max,
           MIN(longitude) AS lon_min
    FROM us_states
    WHERE state NOT IN ('AK','HI','PR')
)
SELECT ROUND(lat_max - lat_min, 2) AS lat_span_deg,
       ROUND(lon_max - lon_min, 2) AS lon_span_deg,
       ROUND(
         2 * RADIUS_EARTH() / 1000 *
         ASIN(SQRT(
             POWER(SIN(RADIANS((lat_max - lat_min) / 2)), 2)
           + COS(RADIANS(lat_min)) * COS(RADIANS(lat_max))
             * POWER(SIN(RADIANS((lon_max - lon_min) / 2)), 2)
         )), 0
       ) AS bbox_diagonal_km
FROM bbox;
GO


-- ---- 7. Geographic centroid of the lower 48 -------------------
-- Naive AVG centroid (no area weighting). Then the closest states
-- to that centroid via the haversine formula. The "constant" row
-- is brought in with CROSS JOIN, a standard CTE pattern.
WITH conus AS (
    SELECT * FROM us_states WHERE state NOT IN ('AK','HI','PR')
),
centroid AS (
    SELECT AVG(latitude) AS clat, AVG(longitude) AS clon FROM conus
),
distances AS (
    SELECT s.name, s.latitude, s.longitude,
           2 * RADIUS_EARTH() / 1000 *
             ASIN(SQRT(
                 POWER(SIN(RADIANS((c.clat - s.latitude) / 2)), 2)
               + COS(RADIANS(s.latitude)) * COS(RADIANS(c.clat))
                 * POWER(SIN(RADIANS((c.clon - s.longitude) / 2)), 2)
             )) AS km_from_center
    FROM conus s CROSS JOIN centroid c
)
SELECT name, ROUND(latitude, 3)  AS lat,
             ROUND(longitude, 3) AS lon,
             ROUND(km_from_center, 0) AS km_from_centroid
FROM distances
ORDER BY km_from_centroid
LIMIT 10;
GO


-- ---- 8. Quadrant scatter relative to the centroid -------------
-- For each state, classify which quadrant of the centroid it falls
-- into. A balanced country would have roughly equal counts.
WITH conus AS (
    SELECT * FROM us_states WHERE state NOT IN ('AK','HI','PR')
),
centroid AS (
    SELECT AVG(latitude) AS clat, AVG(longitude) AS clon FROM conus
),
classified AS (
    SELECT s.name, s.latitude, s.longitude,
           CASE WHEN s.latitude  >= c.clat AND s.longitude >= c.clon THEN 'NE'
                WHEN s.latitude  >= c.clat AND s.longitude <  c.clon THEN 'NW'
                WHEN s.latitude  <  c.clat AND s.longitude >= c.clon THEN 'SE'
                ELSE 'SW' END AS quadrant
    FROM conus s CROSS JOIN centroid c
)
SELECT quadrant,
       COUNT(*)              AS states,
       REPEAT('*', COUNT(*)) AS bar
FROM classified
GROUP BY quadrant
ORDER BY states DESC;
GO


-- ---- 9. Distance from Washington, DC --------------------------
-- Same haversine, but the second point is a literal pair of coords
-- (38.9072 N, -77.0369 W). Useful for "as the crow flies" rankings.
SELECT name,
       ROUND(
         2 * RADIUS_EARTH() / 1000 *
         ASIN(SQRT(
             POWER(SIN(RADIANS((38.9072 - latitude) / 2)), 2)
           + COS(RADIANS(latitude)) * COS(RADIANS(38.9072))
             * POWER(SIN(RADIANS((-77.0369 - longitude) / 2)), 2)
         )), 0
       ) AS km_from_dc
FROM us_states
ORDER BY km_from_dc DESC
LIMIT 5;
GO


-- ---- 10. JOIN with state-areas: largest and smallest ---------
-- Two CSVs joined on state name. Loaded as WEB CTEs so the query
-- is self-contained (the file's shebang only loads the first CSV).
-- Top-5 largest by land area:
WITH WEB state_areas AS (URL 'file://data/state-areas.csv' FORMAT CSV)
SELECT us_states.state, us_states.name AS state_name,
       state_areas.area_sq_mi
FROM us_states
JOIN state_areas ON us_states.name = state_areas.name
ORDER BY state_areas.area_sq_mi DESC
LIMIT 5;
GO

-- Bottom-5 smallest:
WITH WEB state_areas AS (URL 'file://data/state-areas.csv' FORMAT CSV)
SELECT us_states.state, us_states.name AS state_name,
       state_areas.area_sq_mi
FROM us_states
JOIN state_areas ON us_states.name = state_areas.name
ORDER BY state_areas.area_sq_mi ASC
LIMIT 5;
GO


-- ---- 11. Area distribution buckets ---------------------------
-- Group states into rough size buckets, then aggregate.
WITH WEB state_areas AS (URL 'file://data/state-areas.csv' FORMAT CSV),
joined AS (
    SELECT us_states.name AS state_name,
           us_states.latitude,
           us_states.longitude,
           state_areas.area_sq_mi AS area
    FROM us_states
    JOIN state_areas ON us_states.name = state_areas.name
),
bucketed AS (
    SELECT state_name, area,
           CASE WHEN area < 10000  THEN '1: tiny      (<10k)'
                WHEN area < 50000  THEN '2: small     (10k-50k)'
                WHEN area < 100000 THEN '3: medium    (50k-100k)'
                WHEN area < 200000 THEN '4: large     (100k-200k)'
                ELSE                    '5: huge      (200k+)'
           END AS bucket
    FROM joined
)
SELECT bucket,
       COUNT(*)              AS states,
       MIN(area)             AS min_area,
       MAX(area)             AS max_area,
       ROUND(AVG(area), 0)   AS avg_area,
       REPEAT('*', COUNT(*)) AS bar
FROM bucketed
GROUP BY bucket
ORDER BY bucket;
GO


-- ---- 12. Total CONUS area + each state's share ---------------
-- Compute a grand total in a CTE, then JOIN it back to compute
-- each state's percentage of the contiguous USA.
WITH WEB state_areas AS (URL 'file://data/state-areas.csv' FORMAT CSV),
conus AS (
    SELECT us_states.state,
           us_states.name      AS state_name,
           state_areas.area_sq_mi AS area
    FROM us_states
    JOIN state_areas ON us_states.name = state_areas.name
    WHERE us_states.state NOT IN ('AK','HI','PR')
),
totals AS (
    SELECT SUM(area) AS total_area FROM conus
)
SELECT c.state, c.state_name, c.area,
       ROUND(100.0 * c.area / t.total_area, 2) AS pct_of_conus
FROM conus c CROSS JOIN totals t
ORDER BY pct_of_conus DESC
LIMIT 10;
GO
