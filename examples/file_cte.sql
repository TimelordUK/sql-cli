-- Example of using file:// URLs in WEB CTEs to load local CSV files
-- This allows flexible data loading and joining without traditional table registration

-- Load sales data from local CSV file using file:// URL
WITH WEB sales_data AS (
    URL 'file://data/sales_data.csv'
),
-- Load another CSV file for joining
WEB simple_data AS (
    URL 'file://data/test_simple_strings.csv'
    FORMAT CSV
)
-- Query joining the two CTEs
SELECT
    s.region,
    COUNT(*) as count,
    SUM(s.sales_amount) as total_sales
FROM sales_data s
WHERE s.region IS NOT NULL
GROUP BY s.region
ORDER BY total_sales DESC;
GO

WITH
    WEB trades AS (
        URL 'file://data/trades.json' FORMAT JSON
    )
SELECT *
FROM trades;
go

-- Another example: Load and analyze JSON data
WITH WEB trades AS (
    URL 'file://data/trades.json'
    FORMAT JSON
)
SELECT
    instrumentName,
    COUNT(*) as trade_count,
    AVG(price) as avg_price
FROM trades
GROUP BY instrumentName;
GO

WITH
    WEB sales_data AS (
        URL 'file://data/sales_data.csv'
    )
SELECT *
FROM sales_data;
go

-- Example loading and processing a different CSV file
WITH WEB employees AS (
    URL 'file://data/test_simple_strings.csv'
    FORMAT CSV
)
SELECT
    id,
    name,
    UPPER(status) as status,
    SUBSTRING(email, 1, INSTR(email, '@') - 1) as username
FROM employees
WHERE status = 'Active'
ORDER BY name;
GO
