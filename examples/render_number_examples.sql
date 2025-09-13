-- #! ../data/international_sales.csv
-- RENDER_NUMBER Function Examples
-- Demonstrates various number formatting options
-- data: data/international_sales.csv

-- Standard formatting with thousand separators
SELECT
    product,
    amount,
    RENDER_NUMBER(amount) as formatted_standard
FROM international_sales
WHERE region = 'North America'
LIMIT 5;
GO

WITH
    valued_sales AS (
        SELECT *, amount * quantity AS total_value
        FROM international_sales
    )
SELECT product, total_value, RENDER_NUMBER(total_value, 'compact') AS compact_format
FROM valued_sales
WHERE total_value > 10000
ORDER BY total_value DESC
LIMIT 5;
GO

SELECT country, amount, RENDER_NUMBER(amount, 'eu') AS european_format
FROM international_sales
WHERE region = 'Europe'
LIMIT 5;
GO

SELECT
    amount,
    RENDER_NUMBER(amount) AS standard,
    RENDER_NUMBER(amount, 'compact') AS compact,
    RENDER_NUMBER(amount, 'eu') AS european,
    RENDER_NUMBER(amount, 'ch') AS swiss,
    RENDER_NUMBER(amount, 'in') AS indian
FROM international_sales
WHERE amount > 10000
LIMIT 3;
GO

-- Aggregated data with compact formatting
SELECT
    region,
    SUM(amount * quantity) as total_sales,
    RENDER_NUMBER(SUM(amount * quantity), 'compact', 1) as sales_compact
FROM international_sales
GROUP BY region
ORDER BY total_sales DESC;
GO

-- Using different decimal precision
SELECT
    product,
    AVG(amount) as avg_price,
    RENDER_NUMBER(AVG(amount), 'standard', 0) as no_decimals,
    RENDER_NUMBER(AVG(amount), 'standard', 1) as one_decimal,
    RENDER_NUMBER(AVG(amount), 'standard', 3) as three_decimals
FROM international_sales
GROUP BY product;
GO

-- Compact notation with custom precision
WITH
    valued_sales AS (
        SELECT *, amount * quantity AS total_value
        FROM international_sales
    )
SELECT
    country,
    SUM(total_value) as total,
    RENDER_NUMBER(SUM(total_value), 'compact', 0) as compact_no_dec,
    RENDER_NUMBER(SUM(total_value), 'compact', 1) as compact_one_dec,
    RENDER_NUMBER(SUM(total_value), 'compact', 2) as compact_two_dec
FROM valued_sales
GROUP BY country
HAVING total > 20000  -- Note: Use alias 'total' instead of SUM(total_value)
ORDER BY total DESC;
GO
