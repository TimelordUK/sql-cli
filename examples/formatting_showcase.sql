-- Formatting Functions Showcase
-- Quick examples demonstrating RENDER_NUMBER and FORMAT_CURRENCY
-- data: data/international_sales.csv

-- === RENDER_NUMBER Examples ===

-- Standard number formatting with thousand separators
SELECT RENDER_NUMBER(1234567.89) as standard_format;

-- Compact notation (k, M, B, T)
SELECT
    RENDER_NUMBER(1500, 'compact') as thousands,
    RENDER_NUMBER(1500000, 'compact') as millions,
    RENDER_NUMBER(1500000000, 'compact') as billions;

-- Regional number formats
SELECT
    RENDER_NUMBER(1234.56, 'us') as us_format,      -- 1,234.56
    RENDER_NUMBER(1234.56, 'eu') as eu_format,      -- 1.234,56
    RENDER_NUMBER(1234.56, 'ch') as swiss_format,   -- 1'234.56
    RENDER_NUMBER(1234.56, 'in') as indian_format;  -- 1,234.56

-- === FORMAT_CURRENCY Examples ===

-- Basic currency formatting with symbols
SELECT
    FORMAT_CURRENCY(1234.56, 'USD') as dollars,
    FORMAT_CURRENCY(1234.56, 'EUR') as euros,
    FORMAT_CURRENCY(1234.56, 'GBP') as pounds,
    FORMAT_CURRENCY(50000, 'JPY') as yen;

-- Compact currency formats
SELECT
    FORMAT_CURRENCY(3000, 'GBP', 'compact') as compact_symbol,      -- £3k
    FORMAT_CURRENCY(3000, 'GBP', 'compact_code') as compact_code,   -- 3k GBP
    FORMAT_CURRENCY(1500000, 'USD', 'compact') as millions_usd;     -- $1.5M

-- === Real Data Examples ===

-- Format prices from actual data using currency column
SELECT
    country,
    product,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency) as local_price,
    FORMAT_CURRENCY(amount, currency, 'compact_code') as price_code
FROM international_sales
WHERE region = 'Europe'
LIMIT 5;

-- Sales summary with formatted totals
SELECT
    region,
    COUNT(*) as num_sales,
    RENDER_NUMBER(SUM(amount * quantity), 'compact') as total_compact,
    RENDER_NUMBER(AVG(amount), 'standard', 2) as avg_price
FROM international_sales
GROUP BY region
ORDER BY SUM(amount * quantity) DESC;

-- Multi-currency report
SELECT
    product,
    SUM(quantity) as units_sold,
    RENDER_NUMBER(SUM(quantity)) as formatted_units,
    RENDER_NUMBER(COUNT(DISTINCT currency)) as num_currencies
FROM international_sales
GROUP BY product
ORDER BY units_sold DESC;