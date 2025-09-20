-- #! ../data/house_prices_sample.csv

-- House Prices Analysis Examples
-- California housing data with median prices by district

-- Overview: Average house prices by ocean proximity
WITH price_stats AS (
    SELECT
        ocean_proximity,
        COUNT(*) as districts,
        AVG(median_house_value) as avg_value,
        MIN(median_house_value) as min_value,
        MAX(median_house_value) as max_value
    FROM house_prices_sample
    GROUP BY ocean_proximity
)
SELECT
    ocean_proximity,
    districts,
    FORMAT_CURRENCY(avg_value, 'USD') as avg_price,
    FORMAT_CURRENCY(min_value, 'USD') as min_price,
    FORMAT_CURRENCY(max_value, 'USD') as max_price,
    avg_value as sort_value
FROM price_stats
ORDER BY sort_value DESC;
GO

-- Income vs House Price Correlation
-- Note: median_income is in tens of thousands ($10,000 units)
SELECT
    ocean_proximity,
    FORMAT_CURRENCY(AVG(median_income) * 10000, 'USD') AS avg_annual_income,
    FORMAT_CURRENCY(AVG(median_house_value), 'USD') AS avg_house_price,
    ROUND(AVG(median_house_value) / AVG(median_income) * 10000, 2) AS price_to_income_ratio
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
    FORMAT_CURRENCY(AVG(median_house_value), 'USD') AS avg_price,
    COUNT('*') AS cnt
FROM ranks
GROUP BY age_group
ORDER BY age_group ASC;
GO

-- Population Density Analysis
SELECT
    ocean_proximity,
    COUNT('*') AS districts,
    ROUND(AVG(population / households), 1) AS avg_persons_per_household,
    ROUND(AVG(total_rooms / households), 1) AS avg_rooms_per_household,
    ROUND(AVG(total_bedrooms / households), 2) AS avg_bedrooms_per_household
FROM house_prices_sample
WHERE households > 0
GROUP BY ocean_proximity
ORDER BY avg_rooms_per_household DESC;
GO

-- High-Value Districts (Top prices)
SELECT
    FORMAT_CURRENCY(median_house_value, 'USD') AS house_value,
    FORMAT_CURRENCY(median_income * 10000, 'USD') AS annual_income,
    ocean_proximity,
    FORMAT_NUMBER(population, 0) AS population,
    housing_median_age AS age,
    median_house_value AS value_sort,
    median_income AS income_sort
FROM house_prices_sample
WHERE median_house_value > 400000
ORDER BY value_sort DESC, income_sort DESC
LIMIT 20;
GO

-- Affordability Index by Region
-- (Lower is better - shows months of income needed to buy median house)
SELECT
    ocean_proximity,
    COUNT('*') AS districts,
    ROUND(AVG(median_house_value) / AVG(median_income) * 10000 / 12, 1) AS months_income_for_house,
    FORMAT_CURRENCY(AVG(median_income) * 10000, 'USD') AS avg_annual_income,
    FORMAT_CURRENCY(AVG(median_house_value), 'USD') AS avg_house_price
FROM house_prices_sample
GROUP BY ocean_proximity
ORDER BY months_income_for_house ASC;
GO

-- Room Distribution Statistics
SELECT
    ocean_proximity,
    ROUND(AVG(total_rooms), 0) AS avg_total_rooms,
    ROUND(AVG(total_bedrooms), 0) AS avg_bedrooms,
    ROUND(AVG(total_rooms) - AVG(total_bedrooms), 0) AS avg_other_rooms,
    ROUND(AVG(total_bedrooms) / AVG(total_rooms) * 100, 1) AS bedroom_percentage
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
        FROM house_prices_sample
    )
SELECT
    income_bracket,
    COUNT('*') AS districts,
    FORMAT_CURRENCY(AVG(median_house_value), 'USD') AS avg_house_price,
    FORMAT_CURRENCY(MIN(median_house_value), 'USD') AS min_price,
    FORMAT_CURRENCY(MAX(median_house_value), 'USD') AS max_price,
    AVG(median_house_value) AS sort_value
FROM ranked
GROUP BY income_bracket
ORDER BY sort_value DESC;
GO

-- Coastal vs Inland Comparison
WITH
    ranked AS (
        SELECT
            *,
            CASE
        WHEN ocean_proximity = 'NEAR BAY' THEN 'Coastal'
        WHEN ocean_proximity = '<1H OCEAN' THEN 'Coastal'
        WHEN ocean_proximity = 'NEAR OCEAN' THEN 'Coastal'
        WHEN ocean_proximity = 'ISLAND' THEN 'Coastal'
        ELSE 'Inland'
    END AS location_type
        FROM house_prices_sample
    )
SELECT
    location_type,
    COUNT('*') AS districts,
    FORMAT_CURRENCY(AVG(median_house_value), 'USD') AS avg_price,
    FORMAT_CURRENCY(AVG(median_income) * 10000, 'USD') AS avg_annual_income,
    FORMAT_NUMBER(AVG(population), 0) AS avg_population
FROM ranked
GROUP BY location_type
ORDER BY location_type ASC;
GO

-- Find Best Value Districts
-- (High income areas with relatively lower house prices)
SELECT
    ocean_proximity,
    FORMAT_CURRENCY(median_income * 10000, 'USD') AS annual_income,
    FORMAT_CURRENCY(median_house_value, 'USD') AS house_price,
    ROUND(median_house_value / median_income * 10000, 2) AS price_income_ratio,
    ROUND(total_rooms / households, 1) AS rooms_per_household,
    housing_median_age AS age
FROM house_prices_sample
WHERE median_income > 4 AND median_house_value < 300000 AND households > 0
ORDER BY price_income_ratio ASC
LIMIT 15;
GO
