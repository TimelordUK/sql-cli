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

-- Using compact notation for large values
SELECT
    country,
    amount * quantity as total_sale,
    currency,
    FORMAT_CURRENCY(amount * quantity, currency, 'compact') as compact_symbol,
    FORMAT_CURRENCY(amount * quantity, currency, 'compact_code') as compact_code
FROM international_sales
WHERE amount * quantity > 10000
ORDER BY amount * quantity DESC
LIMIT 8;

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

-- Custom decimal places for different currencies
SELECT
    currency,
    amount,
    FORMAT_CURRENCY(amount, currency) as default_decimals,
    FORMAT_CURRENCY(amount, currency, 'symbol', 0) as no_decimals,
    FORMAT_CURRENCY(amount, currency, 'symbol', 3) as three_decimals
FROM international_sales
WHERE currency IN ('JPY', 'USD', 'EUR')
LIMIT 6;

-- Mixed currency report with appropriate formatting
SELECT
    region,
    country,
    FORMAT_CURRENCY(amount, currency) as item_price,
    quantity,
    FORMAT_CURRENCY(amount * quantity, currency, 'compact') as total_compact
FROM international_sales
ORDER BY region, country
LIMIT 15;

-- Top products by revenue with currency formatting
SELECT
    product,
    COUNT(DISTINCT currency) as num_currencies,
    SUM(amount * quantity) as global_revenue,
    RENDER_NUMBER(SUM(amount * quantity), 'compact') as revenue_compact,
    STRING_AGG(DISTINCT currency, ', ') as currencies_used
FROM international_sales
GROUP BY product
ORDER BY global_revenue DESC;
