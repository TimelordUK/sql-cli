-- #! .. data/international_sales.csv
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

-- Compact notation for large numbers
SELECT
    product,
    amount * quantity as total_value,
    RENDER_NUMBER(amount * quantity, 'compact') as compact_format
FROM international_sales
WHERE amount * quantity > 10000
ORDER BY amount * quantity DESC
LIMIT 5;

-- European formatting (period for thousands, comma for decimal)
SELECT
    country,
    amount,
    RENDER_NUMBER(amount, 'eu') as european_format
FROM international_sales
WHERE region = 'Europe'
LIMIT 5;

-- Comparing different formats side by side
SELECT
    amount,
    RENDER_NUMBER(amount) as standard,
    RENDER_NUMBER(amount, 'compact') as compact,
    RENDER_NUMBER(amount, 'eu') as european,
    RENDER_NUMBER(amount, 'ch') as swiss,
    RENDER_NUMBER(amount, 'in') as indian
FROM international_sales
WHERE amount > 10000
LIMIT 3;

-- Aggregated data with compact formatting
SELECT
    region,
    SUM(amount * quantity) as total_sales,
    RENDER_NUMBER(SUM(amount * quantity), 'compact', 1) as sales_compact
FROM international_sales
GROUP BY region
ORDER BY total_sales DESC;

-- Using different decimal precision
SELECT
    product,
    AVG(amount) as avg_price,
    RENDER_NUMBER(AVG(amount), 'standard', 0) as no_decimals,
    RENDER_NUMBER(AVG(amount), 'standard', 1) as one_decimal,
    RENDER_NUMBER(AVG(amount), 'standard', 3) as three_decimals
FROM international_sales
GROUP BY product;

-- Compact notation with custom precision
SELECT
    country,
    SUM(amount * quantity) as total,
    RENDER_NUMBER(SUM(amount * quantity), 'compact', 0) as compact_no_dec,
    RENDER_NUMBER(SUM(amount * quantity), 'compact', 1) as compact_one_dec,
    RENDER_NUMBER(SUM(amount * quantity), 'compact', 2) as compact_two_dec
FROM international_sales
GROUP BY country
HAVING SUM(amount * quantity) > 20000
ORDER BY total DESC;