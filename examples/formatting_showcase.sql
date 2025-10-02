-- #! ../data/international_sales.csv
-- Formatting Functions Showcase
-- Quick examples demonstrating RENDER_NUMBER and FORMAT_CURRENCY
-- data: data/international_sales.csv

-- === RENDER_NUMBER Examples ===

-- Standard number formatting with thousand separators
SELECT RENDER_NUMBER(1234567.89) as standard_format;
go

-- Compact notation (k, M, B, T)
SELECT
    RENDER_NUMBER(1500, 'compact') AS thousands,
    RENDER_NUMBER(1500000, 'compact') AS millions,
    RENDER_NUMBER(1500000000, 'compact') AS billions;
go

-- Regional number formats
SELECT
    RENDER_NUMBER(1234.56, 'us') AS us_format,
    RENDER_NUMBER(1234.56, 'eu') AS eu_format,
    RENDER_NUMBER(1234.56, 'ch') AS swiss_format,
    RENDER_NUMBER(1234.56, 'in') AS indian_format
go

-- === FORMAT_CURRENCY Examples ===
SELECT
    FORMAT_CURRENCY(1234.56, 'USD') AS dollars,
    FORMAT_CURRENCY(1234.56, 'EUR') AS euros,
    FORMAT_CURRENCY(1234.56, 'GBP') AS pounds,
    FORMAT_CURRENCY(50000, 'JPY') AS yen;
go

-- Compact currency formats
SELECT
    FORMAT_CURRENCY(3000, 'GBP', 'compact') AS compact_symbol,
    FORMAT_CURRENCY(3000, 'GBP', 'compact_code') AS compact_code,
    FORMAT_CURRENCY(1500000, 'USD', 'compact') AS millions_usd
go

-- === Real Data Examples ===
SELECT
    country,
    product,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency) AS local_price,
    FORMAT_CURRENCY(amount, currency, 'compact_code') AS price_code
FROM international_sales
WHERE region = 'Europe'
LIMIT 5;
go

-- Sales summary with formatted totals
WITH
    summed AS (
        SELECT
            region,
            amount * quantity AS notional,
            COUNT('*') AS num_sales,
            RENDER_NUMBER(SUM(amount * quantity), 'compact') AS total_compact,
            RENDER_NUMBER(AVG(amount), 'standard', 2) AS avg_price
        FROM international_sales
    )
SELECT *
FROM summed
ORDER BY notional DESC;
go

-- Multi-currency report
SELECT
    product,
    SUM(quantity) AS units_sold,
    RENDER_NUMBER(SUM(quantity)) AS formatted_units,
    RENDER_NUMBER(COUNT(DISTINCT currency)) AS num_currencies
FROM international_sales
GROUP BY product
ORDER BY units_sold DESC;
go
