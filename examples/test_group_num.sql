-- #! ../data/sales_data.csv
-- Test GROUP_NUM function

-- Example 1: Enumerate unique regions
SELECT DISTINCT
    region,
    GROUP_NUM(region) as region_num
FROM sales_data
ORDER BY region_num;
GO

-- Example 2: Enumerate salespersons and show sales with their group numbers  
SELECT 
    salesperson,
    GROUP_NUM(salesperson) as person_num,
    month,
    sales_amount
FROM sales_data
ORDER BY person_num, month;
GO

-- Example 3: Use GROUP_NUM for both region and salesperson
SELECT 
    region,
    GROUP_NUM(region) as region_num,
    salesperson,
    GROUP_NUM(salesperson) as person_num,
    SUM(sales_amount) as total_sales
FROM sales_data
GROUP BY region, salesperson
ORDER BY region_num, person_num;
GO

-- Example 4: Combine with window functions for advanced analytics
WITH enumerated AS (
    SELECT 
        region,
        GROUP_NUM(region) as region_num,
        salesperson,
        GROUP_NUM(salesperson) as person_num,
        sales_amount
    FROM sales_data
),
aggregated AS (
    SELECT 
        region,
        region_num,
        salesperson,
        person_num,
        SUM(sales_amount) as total_sales
    FROM enumerated
    GROUP BY region, region_num, salesperson, person_num
)
SELECT 
    region,
    region_num,
    salesperson,
    person_num,
    total_sales,
    ROW_NUMBER() OVER (PARTITION BY region_num ORDER BY total_sales DESC) as rank_in_region
FROM aggregated
ORDER BY region_num, rank_in_region;
GO
