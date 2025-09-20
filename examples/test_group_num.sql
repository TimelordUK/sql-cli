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
    GROUP_NUM(salesperson) AS person_num,
    month,
    sales_amount
FROM sales_data
ORDER BY person_num ASC, month ASC;
GO

-- Example 3: Use GROUP_NUM for both region and salesperson
WITH
    grouped AS (
        SELECT
            sales_amount,
            region,
            salesperson,
            GROUP_NUM(region) AS region_num,
            GROUP_NUM(salesperson) AS person_num
        FROM sales_data
    )
SELECT
    region,
    salesperson,
    region_num,
    person_num,
    SUM(sales_amount) AS total_sales
FROM grouped
GROUP BY region, region_num, salesperson, person_num
ORDER BY region_num ASC, person_num ASC;
GO

-- Example 4: Combine with window functions for advanced analytics
WITH
    enumerated AS (
        SELECT
            region,
            GROUP_NUM(region) AS region_num,
            salesperson,
            GROUP_NUM(salesperson) AS person_num,
            sales_amount
        FROM sales_data
    ),
    aggregated AS (
        SELECT
            region,
            region_num,
            salesperson,
            person_num,
            SUM(sales_amount) AS total_sales
        FROM enumerated
        GROUP BY region, region_num, salesperson, person_num
    )
SELECT
    region,
    region_num,
    salesperson,
    person_num,
    total_sales,
    ROW_NUMBER() OVER (PARTITION BY region_num ORDER BY total_sales DESC) AS rank_in_region
FROM aggregated
ORDER BY region_num ASC, rank_in_region ASC;
GO
