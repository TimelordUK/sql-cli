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

