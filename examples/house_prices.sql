-- #! ../data/house_prices_sample.csv

-- House Prices Analysis Examples
-- California housing data with median prices by district

-- Overview: Average house prices by ocean proximity
SELECT
    ocean_proximity,
    COUNT(*) as districts,
    ROUND(AVG(median_house_value), 0) as avg_price,
    ROUND(MIN(median_house_value), 0) as min_price,
    ROUND(MAX(median_house_value), 0) as max_price
FROM house_prices_sample
GROUP BY ocean_proximity
ORDER BY avg_price DESC;
GO

-- Income vs House Price Correlation
SELECT
    ocean_proximity,
    ROUND(AVG(median_income), 2) as avg_income,
    ROUND(AVG(median_house_value), 0) as avg_house_price,
    ROUND(AVG(median_house_value) / AVG(median_income), 0) as price_to_income_ratio
FROM house_prices_sample
GROUP BY ocean_proximity
ORDER BY price_to_income_ratio DESC;
GO

-- Housing Age Distribution
WITH
    ranks AS (
        SELECT
            *,
            CASE
        WHEN housing_median_age <= 10 THEN '0-10 years'
        WHEN housing_median_age <= 20 THEN '11-20 years'
        WHEN housing_median_age <= 30 THEN '21-30 years'
        WHEN housing_median_age <= 40 THEN '31-40 years'
        ELSE '40+ years'
    END AS age_group
        FROM house_prices_sample
    )
SELECT
    age_group,
    AVG(median_house_value) AS avg_price,
    COUNT('*') AS cnt
FROM ranks
GROUP BY age_group
ORDER BY age_group ASC;
GO

-- Population Density Analysis
SELECT
    ocean_proximity,
    COUNT(*) as districts,
    ROUND(AVG(population / households), 1) as avg_persons_per_household,
    ROUND(AVG(total_rooms / households), 1) as avg_rooms_per_household,
    ROUND(AVG(total_bedrooms / households), 2) as avg_bedrooms_per_household
FROM house_prices_sample
WHERE households > 0
GROUP BY ocean_proximity
ORDER BY avg_rooms_per_household DESC;
GO

-- High-Value Districts (Top prices)
SELECT
    ROUND(median_house_value, 0) as house_value,
    ROUND(median_income, 2) as median_income,
    ocean_proximity,
    ROUND(population, 0) as population,
    housing_median_age as age
FROM house_prices_sample
WHERE median_house_value > 400000
ORDER BY house_value DESC
LIMIT 20;
GO

-- Affordability Index by Region
-- (Lower is better - shows months of income needed to buy median house)
SELECT
    ocean_proximity,
    COUNT(*) as districts,
    ROUND(AVG(median_house_value) / (AVG(median_income) * 10000 / 12), 1) as months_income_for_house,
    ROUND(AVG(median_income), 2) as avg_income,
    ROUND(AVG(median_house_value), 0) as avg_house_price
FROM house_prices_sample
GROUP BY ocean_proximity
ORDER BY months_income_for_house;
GO

-- Room Distribution Statistics
SELECT
    ocean_proximity,
    ROUND(AVG(total_rooms), 0) as avg_total_rooms,
    ROUND(AVG(total_bedrooms), 0) as avg_bedrooms,
    ROUND(AVG(total_rooms) - AVG(total_bedrooms), 0) as avg_other_rooms,
    ROUND((AVG(total_bedrooms) / AVG(total_rooms)) * 100, 1) as bedroom_percentage
FROM house_prices_sample
GROUP BY ocean_proximity
ORDER BY avg_total_rooms DESC;
GO

WITH
    ranked AS (
        SELECT
            *,
            CASE
        WHEN median_income <= 2 THEN 'Low (<$20k)'
        WHEN median_income <= 4 THEN 'Lower-Middle ($20-40k)'
        WHEN median_income <= 6 THEN 'Middle ($40-60k)'
        WHEN median_income <= 8 THEN 'Upper-Middle ($60-80k)'
        ELSE 'High (>$80k)'
    END AS income_bracket
        FROM house_prices_samplea
    )
SELECT
    COUNT('*') AS districts,
    ROUND(AVG(median_house_value), 0) AS avg_house_price,
    ROUND(MIN(median_house_value), 0) AS min_price,
    ROUND(MAX(median_house_value), 0) AS max_price
FROM ranked
GROUP BY income_bracket
ORDER BY avg_house_price DESC;
GO

-- Coastal vs Inland Comparison
WITH
    ranked AS (
        SELECT
            *,
            CASE
        WHEN ocean_proximity = 'NEAR BAY' then 'Coastal'
        WHEN ocean_proximity = '<1H OCEAN' then 'Coastal'
        WHEN ocean_proximity = 'NEAR OCEAN' then 'Coastal'
        WHEN ocean_proximity = 'ISLAND' then 'Coastal'
        ELSE 'Inland'
    END AS location_type
        FROM house_prices_sample
    )
SELECT
    location_type,
    COUNT('*') AS districts,
    ROUND(AVG(median_house_value), 0) AS avg_price,
    ROUND(AVG(median_income), 2) AS avg_income,
    ROUND(AVG(population), 0) AS avg_population
FROM ranked
GROUP BY location_type
ORDER BY location_type ASC;
GO

-- Find Best Value Districts
-- (High income areas with relatively lower house prices)
SELECT
    ocean_proximity,
    ROUND(median_income, 2) as income,
    ROUND(median_house_value, 0) as house_price,
    ROUND(median_house_value / (median_income * 10000), 2) as price_income_ratio,
    ROUND(total_rooms / households, 1) as rooms_per_household,
    housing_median_age as age
FROM house_prices_sample
WHERE median_income > 4
    AND median_house_value < 300000
    AND households > 0
ORDER BY price_income_ratio
LIMIT 15;
GO
