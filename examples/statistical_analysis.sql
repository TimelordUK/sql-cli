-- #! ../data/sales_data.csv

-- Statistical Analysis Examples
-- This demonstrates the comprehensive statistical aggregate functions available in SQL CLI

-- Basic statistical aggregates on the entire dataset
-- Note: MODE() currently only supports numeric columns
SELECT
    COUNT(*) as total_records,
    AVG(sales_amount) as average_sales_amount,
    MEDIAN(sales_amount) as median_sales_amount,
    MODE(sales_amount) as most_common_sales_amount,
    STDDEV(sales_amount) as stddev_population,
    STDDEV_POP(sales_amount) as stddev_pop_explicit,
    STDDEV_SAMP(sales_amount) as stddev_sample,
    VARIANCE(sales_amount) as variance_population,
    VAR_POP(sales_amount) as variance_pop_explicit,
    VAR_SAMP(sales_amount) as variance_sample
FROM sales_data;
GO

-- Statistical analysis grouped by region
-- Shows how different regions compare statistically
SELECT
    region,
    COUNT(*) as sales_count,
    ROUND(AVG(sales_amount), 2) as avg_sales_amount,
    MEDIAN(sales_amount) as median_sales_amount,
    MIN(sales_amount) as min_sales_amount,
    MAX(sales_amount) as max_sales_amount,
    ROUND(STDDEV(sales_amount), 2) as stddev,
    ROUND(VARIANCE(sales_amount), 2) as variance
FROM sales_data
GROUP BY region
ORDER BY avg_sales_amount DESC;
GO

-- Comparing population vs sample statistics
-- Useful when analyzing whether your data is a sample or complete population
-- Note: Using CTE + WHERE workaround instead of HAVING clause
WITH stats AS (
    SELECT
        product,
        COUNT(*) as n,
        ROUND(AVG(sales_amount), 2) as mean,
        ROUND(STDDEV(sales_amount), 2) as stddev_pop,
        ROUND(STDDEV_SAMP(sales_amount), 2) as stddev_samp,
        ROUND(VARIANCE(sales_amount), 2) as var_pop,
        ROUND(VAR_SAMP(sales_amount), 2) as var_samp
    FROM sales_data
    GROUP BY product
)
SELECT
    product,
    n as sample_size,
    mean,
    stddev_pop,
    stddev_samp,
    ROUND(stddev_samp - stddev_pop, 2) as stddev_difference,
    var_pop,
    var_samp,
    ROUND(var_samp - var_pop, 2) as variance_difference
FROM stats
WHERE n > 5  -- Filter using WHERE instead of HAVING
ORDER BY sample_size DESC;
GO

-- MODE function to find most frequent numeric values
-- Note: MODE() currently only supports numeric columns, not text/categorical data
SELECT
    MODE(sales_amount) as most_common_sales_amount,
    MODE(ROUND(sales_amount, -2)) as most_common_hundred_range,
    MODE(ROUND(sales_amount / 1000, 0)) as most_common_thousand_range
FROM sales_data;
GO

-- Finding mode for categorical columns (workaround)
-- Since MODE doesn't work on text, use GROUP BY + ORDER BY + LIMIT
WITH region_counts AS (
    SELECT
        region,
        COUNT(*) as frequency
    FROM sales_data
    GROUP BY region
)
SELECT
    region as most_common_region,
    frequency as occurrences
FROM region_counts
ORDER BY occurrences DESC
LIMIT 1;
GO

-- Similarly for product mode
WITH product_counts AS (
    SELECT
        product,
        COUNT(*) as frequency
    FROM sales_data
    GROUP BY product
)
SELECT
    product as most_common_product,
    frequency as occurrences
FROM product_counts
ORDER BY occurrences DESC
LIMIT 1;
GO


-- Coefficient of Variation (CV) - relative variability measure
-- CV = (Standard Deviation / Mean) * 100
-- Note: Using CTE + WHERE workaround instead of HAVING clause
WITH cv_stats AS (
    SELECT
        product,
        COUNT(*) as n,
        ROUND(AVG(sales_amount), 2) as mean,
        ROUND(STDDEV(sales_amount), 2) as std_dev,
        ROUND((STDDEV(sales_amount) / AVG(sales_amount)) * 100, 2) as coefficient_of_variation
    FROM sales_data
    GROUP BY product
)
SELECT *
FROM cv_stats
WHERE n > 3  -- Filter using WHERE instead of HAVING
ORDER BY coefficient_of_variation DESC;
GO

-- Statistical comparison between time periods
-- Analyze how statistics change over time
-- Note: SQL SUBSTRING() is 1-based (SQL standard); extract YYYY-MM (7 chars)
SELECT
    SUBSTRING(month, 1, 7) as month,
    COUNT(*) as transactions,
    ROUND(AVG(sales_amount), 2) as avg_sales_amount,
    ROUND(MEDIAN(sales_amount), 2) as median_sales_amount,
    ROUND(STDDEV(sales_amount), 2) as volatility,
    ROUND(MIN(sales_amount), 2) as min_sales_amount,
    ROUND(MAX(sales_amount), 2) as max_sales_amount,
    ROUND(MAX(sales_amount) - MIN(sales_amount), 2) as range
FROM sales_data
GROUP BY SUBSTRING(month, 1, 7)
ORDER BY month;
GO
