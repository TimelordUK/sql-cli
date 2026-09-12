
-- stage all countries in a tmp table

WITH
    countries AS (
        SELECT *
        FROM READ_CSV('data/countries.csv')
    )
SELECT
 *
INTO #countries
FROM countries;
go

-- select single country - note the borders

SELECT 
    "name.common" AS name,
    region,
    cca3,
    currencies,
    capital,
    latlng,
    area,
    callingCodes,
    borders,
    landlocked
FROM #countries
WHERE "name.common" = 'Russia';
go

-- for Russia inflate borders join back to countries

WITH
    all AS (
        SELECT *
        FROM #countries
    ),
    codes AS (
        SELECT value AS code
        FROM SPLIT('AZE,BLR,CHN,EST,FIN,GEO,KAZ,PRK,LVA,LTU,MNG,NOR,POL,UKR', ',')
    )
SELECT
    "name.common" AS name,
    region,
    cca3,
    currencies,
    capital,
    latlng,
    area,
    callingCodes,
    borders,
    landlocked,
    code
FROM all
INNER JOIN codes ON all.cca3 = codes.code
ORDER BY name ASC;
GO

-- counts within each region

SELECT
    region,
    landlocked,
    COUNT('*') AS n
FROM #countries
GROUP BY region,landlocked
ORDER BY region, landlocked, n desc;
GO


-- ---------------------------------------------------------------------------
-- borders is a comma-packed list, so unpack it for EVERY country at once:
-- CROSS JOIN a slot number onto each row and pull the nth item out with
-- SPLIT_PART. 20 slots is enough for China, the record holder with 16.
-- This is the generic version of the hand-typed SPLIT list above.
-- ---------------------------------------------------------------------------

WITH
    slots AS (
        SELECT value AS n
        FROM RANGE(1, 20)
    ),
    c AS (
        SELECT *
        FROM #countries
    )
SELECT
    cca3 AS iso,
    "name.common" AS name,
    region,
    landlocked,
    CAST(SPLIT_PART(latlng, ',', 1) AS FLOAT) AS lat,
    CAST(SPLIT_PART(latlng, ',', 2) AS FLOAT) AS lng,
    SPLIT_PART(borders, ',', n) AS neighbour_iso
INTO #border_pairs
FROM c
CROSS JOIN slots
WHERE borders <> ''
  AND SPLIT_PART(borders, ',', n) <> '';
GO

-- who has the most neighbours?

SELECT
    name,
    iso,
    region,
    COUNT('*') AS neighbours
FROM #border_pairs
GROUP BY name, iso, region
ORDER BY neighbours DESC, name ASC
LIMIT 15;
GO

-- ---------------------------------------------------------------------------
-- join the pairs back to #countries to pick up the neighbour's attributes.
-- NOTE: the engine cannot compare two columns in a WHERE clause yet (P46 in
-- docs/SQL_PARITY.md - it returns zero rows), so the region comparison is done
-- here in the SELECT list, where it works, and filtered in the next statement.
-- ---------------------------------------------------------------------------

WITH
    bp AS (
        SELECT *
        FROM #border_pairs
    ),
    nb AS (
        SELECT
            cca3,
            "name.common" AS nb_name,
            region AS nb_region,
            CAST(landlocked AS INT) AS nb_landlocked,
            CAST(SPLIT_PART(latlng, ',', 1) AS FLOAT) AS nb_lat,
            CAST(SPLIT_PART(latlng, ',', 2) AS FLOAT) AS nb_lng
        FROM #countries
    )
SELECT
    bp.name,
    bp.region,
    bp.lat,
    bp.lng,
    CAST(bp.landlocked AS INT) AS landlocked,
    nb.nb_name,
    nb.nb_region,
    nb.nb_landlocked,
    nb.nb_lat,
    nb.nb_lng,
    CASE WHEN bp.region = nb.nb_region THEN 'internal' ELSE 'cross-region' END AS frontier,
    CASE WHEN bp.name < nb.nb_name THEN 'keep' ELSE 'mirror' END AS pair_dedupe
INTO #edges
FROM bp
INNER JOIN nb ON bp.neighbour_iso = nb.cca3;
GO

-- the land borders that leave their own continent

SELECT
    name,
    region,
    nb_name,
    nb_region
FROM #edges
WHERE frontier = 'cross-region'
ORDER BY region ASC, name ASC;
GO

-- ---------------------------------------------------------------------------
-- doubly landlocked: landlocked, and every neighbour is landlocked too.
-- there are exactly two in the world.
-- ---------------------------------------------------------------------------

SELECT
    name,
    COUNT('*') AS neighbours,
    SUM(nb_landlocked) AS landlocked_neighbours,
    COUNT('*') - SUM(nb_landlocked) AS coastal_neighbours
INTO #landlocked_summary
FROM #edges
WHERE landlocked = 1
GROUP BY name;
GO

SELECT
    name,
    neighbours,
    landlocked_neighbours
FROM #landlocked_summary
WHERE coastal_neighbours = 0
ORDER BY name ASC;
GO

-- ---------------------------------------------------------------------------
-- data quality: borders ought to be symmetric. Build a forward key and a
-- reverse key for every pair, then LEFT JOIN the reverse key back onto the
-- forward keys - a miss is a one-sided border in the source data.
-- ---------------------------------------------------------------------------

WITH
    a AS (
        SELECT
            iso,
            neighbour_iso,
            iso || '>' || neighbour_iso AS fwd,
            neighbour_iso || '>' || iso AS rev
        FROM #border_pairs
    ),
    b AS (
        SELECT iso || '>' || neighbour_iso AS fwd
        FROM #border_pairs
    )
SELECT
    a.iso AS claims,
    a.neighbour_iso AS but_not_listed_by,
    a.fwd AS one_sided_border
FROM a
LEFT JOIN b ON a.rev = b.fwd
WHERE b.fwd IS NULL
ORDER BY a.iso ASC;
GO

-- ---------------------------------------------------------------------------
-- no geo functions required - haversine straight off latlng with
-- RADIANS/SIN/COS/ACOS, then CONVERT for the imperial column.
-- ---------------------------------------------------------------------------

WITH
    c AS (
        SELECT
            "name.common" AS name,
            region,
            CAST(SPLIT_PART(latlng, ',', 1) AS FLOAT) AS lat,
            CAST(SPLIT_PART(latlng, ',', 2) AS FLOAT) AS lng
        FROM #countries
        WHERE latlng <> ''
    )
SELECT
    name,
    region,
    lat,
    lng,
    ROUND(6371 * ACOS(
          SIN(RADIANS(51.5)) * SIN(RADIANS(lat))
        + COS(RADIANS(51.5)) * COS(RADIANS(lat)) * COS(RADIANS(lng - (-0.13)))
    ), 0) AS km_from_london,
    ROUND(CONVERT(6371 * ACOS(
          SIN(RADIANS(51.5)) * SIN(RADIANS(lat))
        + COS(RADIANS(51.5)) * COS(RADIANS(lat)) * COS(RADIANS(lng - (-0.13)))
    ), 'km', 'miles'), 0) AS miles_from_london
FROM c
ORDER BY km_from_london DESC
LIMIT 10;
GO

-- the widest land borders: neighbours whose centroids are furthest apart.
-- every border appears twice in #edges (A>B and B>A), so pair_dedupe keeps
-- one direction only.

SELECT
    name,
    nb_name,
    region,
    nb_region,
    ROUND(6371 * ACOS(
          SIN(RADIANS(lat)) * SIN(RADIANS(nb_lat))
        + COS(RADIANS(lat)) * COS(RADIANS(nb_lat)) * COS(RADIANS(nb_lng - lng))
    ), 0) AS centroid_km
FROM #edges
WHERE pair_dedupe = 'keep'
ORDER BY centroid_km DESC
LIMIT 10;
GO

-- and the tightest ones - microstates and enclaves

SELECT
    name,
    nb_name,
    ROUND(6371 * ACOS(
          SIN(RADIANS(lat)) * SIN(RADIANS(nb_lat))
        + COS(RADIANS(lat)) * COS(RADIANS(nb_lat)) * COS(RADIANS(nb_lng - lng))
    ), 0) AS centroid_km
FROM #edges
WHERE pair_dedupe = 'keep'
ORDER BY centroid_km ASC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- currency unions - one currency code, many countries
-- ---------------------------------------------------------------------------

SELECT
    currencies,
    COUNT('*') AS members,
    STRING_AGG("name.common", ',') AS countries
FROM #countries
WHERE currencies <> ''
GROUP BY currencies
ORDER BY members DESC, currencies ASC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- languages is another packed list - same slot trick, then count the reach
-- ---------------------------------------------------------------------------

WITH
    slots AS (
        SELECT value AS n
        FROM RANGE(1, 6)
    ),
    c AS (
        SELECT *
        FROM #countries
    )
SELECT
    SPLIT_PART(languages, ',', n) AS language,
    "name.common" AS name,
    region
INTO #languages
FROM c
CROSS JOIN slots
WHERE languages <> ''
  AND SPLIT_PART(languages, ',', n) <> '';
GO

SELECT
    language,
    COUNT('*') AS countries,
    COUNT(DISTINCT region) AS regions
FROM #languages
GROUP BY language
ORDER BY countries DESC, language ASC
LIMIT 15;
GO

-- ---------------------------------------------------------------------------
-- island nations: no land borders, and not landlocked either
-- ---------------------------------------------------------------------------

SELECT
    flag,
    "name.common" AS name,
    region,
    subregion,
    capital,
    RENDER_NUMBER(area, 'compact') AS area_km2
FROM #countries
WHERE borders = ''
  AND landlocked = 0
  AND area > 10000
ORDER BY area DESC
LIMIT 20;
GO

-- ---------------------------------------------------------------------------
-- window functions plus a text bar chart. CAST(area AS FLOAT) is deliberate:
-- area mixes integers and floats, and raw MIN/MAX rank by type before value
-- (P47 in docs/SQL_PARITY.md - MAX(area) answers 34.2, not Russia).
-- ---------------------------------------------------------------------------

WITH
    c AS (
        SELECT
            "name.common" AS name,
            region,
            CAST(area AS FLOAT) AS area
        FROM #countries
        WHERE area > 0
    )
SELECT
    ROW_NUMBER() OVER (ORDER BY area DESC) AS rank_world,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY area DESC) AS rank_in_region,
    name,
    region,
    RENDER_NUMBER(area, 'compact') AS area_km2,
    ROUND(100.0 * area / SUM(area) OVER (), 2) AS pct_of_world,
    ROUND(100.0 * area / SUM(area) OVER (PARTITION BY region), 2) AS pct_of_region,
    REPEAT('#', CAST(area / 500000 AS INT)) AS bar
FROM c
ORDER BY area DESC
LIMIT 20;
GO
