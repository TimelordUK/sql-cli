-- #! ../data/sales_data.csv

-- Statistical Analysis Examples
-- This demonstrates the comprehensive statistical aggregate functions available in SQL CLI

-- Basic statistical aggregates on the entire dataset
SELECT
    COUNT(*) as total_records,
    AVG(sales_amount) as average_sales_amount,
    MEDIAN(sales_amount) as median_sales_amount,
    MODE(region) as most_common_region,
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
    HAVING COUNT(*) > 5
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
ORDER BY sample_size DESC;
GO

-- MODE function to find most frequent values
-- Useful for categorical data analysis
SELECT
    MODE(product) as most_sold_product,
    MODE(region) as most_active_region,
    MODE(ROUND(sales_amount, -1)) as most_common_price_range
FROM sales_data;
GO

-- Statistical summary for outlier detection
-- Values more than 2 standard deviations from mean might be outliers
WITH stats AS (
    SELECT
        AVG(sales_amount) as mean_sales_amount,
        STDDEV(sales_amount) as std_sales_amount
    FROM sales_data
)
SELECT
    product,
    region,
    sales_amount,
    ROUND((sales_amount - mean_sales_amount) / std_sales_amount, 2) as z_score,
    CASE
        WHEN ABS((sales_amount - mean_sales_amount) / std_sales_amount) > 2 THEN 'Potential Outlier'
        WHEN ABS((sales_amount - mean_sales_amount) / std_sales_amount) > 1 THEN 'Above/Below Average'
        ELSE 'Normal Range'
    END as classification
FROM sales_data, stats
ORDER BY ABS((sales_amount - mean_sales_amount) / std_sales_amount) DESC
LIMIT 10;
GO

-- Coefficient of Variation (CV) - relative variability measure
-- CV = (Standard Deviation / Mean) * 100
SELECT
    product,
    COUNT(*) as n,
    ROUND(AVG(sales_amount), 2) as mean,
    ROUND(STDDEV(sales_amount), 2) as std_dev,
    ROUND((STDDEV(sales_amount) / AVG(sales_amount)) * 100, 2) as coefficient_of_variation
FROM sales_data
GROUP BY product
HAVING COUNT(*) > 3
ORDER BY coefficient_of_variation DESC;
GO

-- Statistical comparison between time periods
-- Analyze how statistics change over time
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