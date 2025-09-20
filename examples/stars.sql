-- #!  ../data/star_systems_20_25_ly.csv
SELECT *
FROM star_systems_20_25_ly;
GO

SELECT
    constellation,
    COUNT('*') AS num
FROM star_systems_20_25_ly
GROUP BY constellation
ORDER BY num DESC;
GO

SELECT *
FROM star_systems_20_25_ly
WHERE notes.Contains('dwarf')
ORDER BY distance_ly DESC;
GO

SELECT *
FROM star_systems_20_25_ly
WHERE distance_ly = (
    SELECT
        MIN(distance_ly) AS expr_1
    FROM star_systems_20_25_ly
);
GO

-- Query 5: Distance differences between consecutive stars using LAG
-- Shows each star with its previous neighbor and the gap between them
SELECT
    designation,
    distance_ly,
    constellation,
    LAG(designation) OVER (ORDER BY distance_ly ASC) AS prev_star,
    LAG(distance_ly) OVER (ORDER BY distance_ly ASC) AS prev_distance,
    LAG(constellation) OVER (ORDER BY distance_ly ASC) AS prev_constellation,
    ROUND(distance_ly - LAG(distance_ly) OVER (ORDER BY distance_ly ASC), 2) AS distance_delta
FROM star_systems_20_25_ly
ORDER BY distance_ly ASC;
GO

-- Query 6: Partition stars into 5 light-year bands and rank within each band
-- Shows concentration of stars in distance ranges
-- Note: Using a pre-calculated band column since PARTITION BY doesn't support CASE expressions yet
WITH
    distance_bands AS (
        SELECT
            designation,
            distance_ly,
            constellation,
            stellar_class,
            CASE
        WHEN distance_ly < 15 THEN '10-15 ly'
        WHEN distance_ly < 20 THEN '15-20 ly'
        WHEN distance_ly < 25 THEN '20-25 ly'
        WHEN distance_ly < 30 THEN '25-30 ly'
        ELSE '> 30 ly'
    END AS distance_band
        FROM star_systems_20_25_ly
    ),
    ranked_bands AS (
        SELECT
            designation,
            distance_ly,
            constellation,
            stellar_class,
            distance_band,
            ROW_NUMBER() OVER (PARTITION BY distance_band ORDER BY distance_ly ASC) AS rank_in_band
        FROM distance_bands
    )
SELECT
    distance_band,
    rank_in_band,
    designation,
    distance_ly,
    constellation
FROM ranked_bands
ORDER BY distance_band ASC, rank_in_band ASC;
GO

-- Query 7: Summary of stars per 5 ly band with statistics
WITH
    distance_bands AS (
        SELECT
            designation,
            distance_ly,
            constellation,
            stellar_class,
            CASE
        WHEN distance_ly < 15 THEN '10-15 ly'
        WHEN distance_ly < 20 THEN '15-20 ly'
        WHEN distance_ly < 25 THEN '20-25 ly'
        WHEN distance_ly < 30 THEN '25-30 ly'
        ELSE '> 30 ly'
    END AS distance_band
        FROM star_systems_20_25_ly
    )
SELECT
    distance_band,
    COUNT('*') AS star_count,
    ROUND(MIN(distance_ly), 1) AS min_distance,
    ROUND(MAX(distance_ly), 1) AS max_distance,
    ROUND(AVG(distance_ly), 1) AS avg_distance
FROM distance_bands
GROUP BY distance_band 
ORDER BY star_count desc;
GO

-- Query 8: Bar chart visualization of star distribution by distance band
WITH
    distance_bands AS (
        SELECT
            designation,
            distance_ly,
            constellation,
            stellar_class,
            CASE
                WHEN distance_ly < 15 THEN '10-15 ly'
                WHEN distance_ly < 20 THEN '15-20 ly'
                WHEN distance_ly < 25 THEN '20-25 ly'
                WHEN distance_ly < 30 THEN '25-30 ly'
                ELSE '> 30 ly'
            END AS distance_band
        FROM star_systems_20_25_ly
    ),
    counts AS (
        SELECT
            distance_band,
            COUNT('*') AS star_count,
            ROUND(MIN(distance_ly), 1) AS min_distance,
            ROUND(MAX(distance_ly), 1) AS max_distance,
            ROUND(AVG(distance_ly), 1) AS avg_distance
        FROM distance_bands
        GROUP BY distance_band
    )
SELECT
    distance_band,
    star_count,
    REPEAT('*', star_count) AS bar_chart,
    ROUND((star_count * 100.0 / (SELECT SUM(star_count) FROM counts)), 1) AS percent,
    min_distance
FROM counts
ORDER BY min_distance;
GO

