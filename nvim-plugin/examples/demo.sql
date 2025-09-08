-- SQL CLI Neovim Plugin Demo
-- #!data: ../data/sales_data.csv

-- Example 1: Basic query with aggregation
SELECT 
    region,
    COUNT(*) as sales_count,
    SUM(amount) as total_amount,
    AVG(amount) as avg_amount
FROM sales_data
GROUP BY region
ORDER BY total_amount DESC;
GO

-- Example 2: Using window functions
SELECT 
    date,
    region,
    amount,
    SUM(amount) OVER (PARTITION BY region ORDER BY date) as running_total,
    RANK() OVER (PARTITION BY region ORDER BY amount DESC) as amount_rank
FROM sales_data
WHERE amount > 100
ORDER BY region, date;
GO

-- Example 3: Using mathematical functions
SELECT 
    FIBONACCI(10) as fib_10,
    HARMONIC(100) as harmonic_100,
    GEOMETRIC(1, 2, 10) as powers_of_2,
    CONVERT(100, 'celsius', 'fahrenheit') as temp_conversion
FROM DUAL;
GO

-- Example 4: String manipulation
SELECT 
    product,
    UPPER(product) as upper_case,
    LOWER(product) as lower_case,
    LENGTH(product) as length,
    SUBSTRING(product, 1, 3) as first_three
FROM sales_data
LIMIT 5;