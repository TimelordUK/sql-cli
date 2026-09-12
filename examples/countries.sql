
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

SELECT
    region,
    COUNT('*') AS n
FROM #countries
GROUP BY region
ORDER BY n desc, region ASC;
GO

