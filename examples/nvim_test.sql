-- #! ../data/house_prices_sample.csv

SELECT
    *,
    CASE
        WHEN value <= 20 THEN 'Level 1'
        WHEN value > 20 AND value <= 40 THEN 'Level 2'
        WHEN value > 40 AND value <= 60 THEN 'Level 3'
        WHEN value > 60 AND value <= 80 THEN 'Level 4'
        WHEN value > 80 THEN 'Level 5'
    END AS value_band
FROM RANGE(1, 100);
go

-- FIX Protocol Message Analysis using File CTEs
-- This example demonstrates how to join FIX protocol tables to create enriched message definitions

WITH
    WEB fields AS (URL 'file://data/FIX44_fields.csv')
select * from fields 
limit 20;
GO

WITH
    WEB messages AS (URL 'file://data/FIX44_messages.csv')
select * from messages
limit 20;
GO

-- put cursor under ocean_proximity and press \rC and generate a case based on values in that column

WITH
    WEB prices AS (
        URL 'file://data/house_prices_sample.csv'
    )
SELECT
    median_income,
    ocean_proximity,
    CASE
        WHEN median_house_value <= 118000 THEN '22500-118000'
        WHEN median_house_value > 118000 AND median_house_value <= 213500 THEN '118000-213500'
        WHEN median_house_value > 213500 AND median_house_value <= 309001 THEN '213500-309001'
        WHEN median_house_value > 309001 AND median_house_value <= 404501 THEN '309001-404501'
        WHEN median_house_value > 404501 THEN '404501+'
    END AS median_house_value_band,
    CASE
        WHEN ocean_proximity = '<1H OCEAN' THEN 'r1'
        WHEN ocean_proximity = 'INLAND' THEN 'r2'
        WHEN ocean_proximity = 'ISLAND' THEN 'r3'
        WHEN ocean_proximity = 'NEAR BAY' THEN 'r4'
        WHEN ocean_proximity = 'NEAR OCEAN' THEN 'r5'
        ELSE 'Other'
    END AS ocean_proximity_category,
    median_house_value
FROM prices
GO


