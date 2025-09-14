-- #! ../data/international_sales.csv

-- FORMAT_CURRENCY Function Examples
-- Demonstrates currency formatting with symbols, codes, and various styles

-- Basic currency formatting using column values
SELECT
    country,
    product,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency) as formatted_price
FROM international_sales
LIMIT 10;
GO

WITH
    values AS (
        SELECT *, amount * quantity AS total_sale
        FROM international_sales
    )
SELECT
    country,
    total_sale,
    currency,
    FORMAT_CURRENCY(total_sale, currency, 'compact') AS compact_symbol,
    FORMAT_CURRENCY(total_sale, currency, 'compact_code') AS compact_code
FROM values
WHERE total_sale > 10000
ORDER BY total_sale DESC
LIMIT 8;
GO

-- Different currency display formats
SELECT
    product,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency, 'symbol') as with_symbol,
    FORMAT_CURRENCY(amount, currency, 'code') as with_code,
    FORMAT_CURRENCY(amount, currency, 'name') as with_name
FROM international_sales
WHERE currency IN ('USD', 'EUR', 'GBP')
LIMIT 5;
GO

-- Regional formatting styles
SELECT
    country,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency, 'symbol') as standard,
    FORMAT_CURRENCY(amount, currency, 'eu') as european_style,
    FORMAT_CURRENCY(amount, currency, 'ch') as swiss_style
FROM international_sales
WHERE region = 'Europe'
LIMIT 6;
GO

-- Aggregated sales by currency with formatting
SELECT
    currency,
    COUNT(*) as transactions,
    SUM(amount * quantity) as total_sales,
    FORMAT_CURRENCY(SUM(amount * quantity), currency, 'compact_code') as formatted_total,
    FORMAT_CURRENCY(AVG(amount), currency) as avg_price
FROM international_sales
GROUP BY currency
ORDER BY total_sales DESC;
GO

SELECT
    currency,
    amount,
    FORMAT_CURRENCY(amount, currency) AS default_decimals,
    FORMAT_CURRENCY(amount, currency, 'symbol', 0) AS no_decimals,
    FORMAT_CURRENCY(amount, currency, 'symbol', 3) AS three_decimals
FROM international_sales
WHERE currency IN ('JPY', 'USD', 'EUR')
LIMIT 6;
GO

SELECT
    region,
    country,
    FORMAT_CURRENCY(amount, currency) AS item_price,
    quantity,
    FORMAT_CURRENCY(amount * quantity, currency, 'compact') AS total_compact
FROM international_sales
ORDER BY region ASC, country ASC
LIMIT 15;
GO

WITH
    values AS (
        SELECT *, amount * quantity AS total_sale
        FROM international_sales
    )
SELECT
    product,
    COUNT(DISTINCT currency) AS num_currencies,
    FORMAT_CURRENCY(SUM(total_sale), currency) AS global_revenue,
    RENDER_NUMBER(SUM(total_sale), 'compact') AS revenue_compact,
    STRING_AGG(DISTINCT currency, ', ') AS currencies_usd
FROM values
GROUP BY product
ORDER BY global_revenue DESC;
GO
