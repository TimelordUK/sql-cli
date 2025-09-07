-- ============================================================================
-- CTE Cookbook: Sales Analytics (Simplified Working Examples)
-- Real-world patterns using only supported SQL features
-- ============================================================================
-- Run: ./target/release/sql-cli data/sales_data.csv -f examples/cte_cookbook_simple.sql -o table
-- ============================================================================

-- ============================================================================
-- Recipe 1: Top Salesperson per Region
-- Classic "top N per group" pattern using ROW_NUMBER
-- ============================================================================
WITH 
-- Step 1: Calculate total sales per person
person_totals AS (
    SELECT 
        salesperson,
        region,
        SUM(sales_amount) AS total_sales,
        COUNT(*) AS num_transactions,
        AVG(sales_amount) AS avg_sale
    FROM test
    GROUP BY salesperson, region
),
-- Step 2: Rank within each region
ranked_by_region AS (
    SELECT 
        salesperson,
        region,
        total_sales,
        num_transactions,
        ROUND(avg_sale, 2) AS avg_sale_amount,
        ROW_NUMBER() OVER (PARTITION BY region ORDER BY total_sales DESC) AS region_rank
    FROM person_totals
)
-- Step 3: Filter to top performer per region
SELECT 
    region,
    salesperson,
    total_sales,
    num_transactions,
    avg_sale_amount,
    region_rank
FROM ranked_by_region
WHERE region_rank = 1
ORDER BY total_sales DESC;
GO

-- ============================================================================
-- Recipe 2: Monthly Sales Growth Analysis  
-- Calculate month-over-month changes using LAG
-- ============================================================================
WITH 
-- Step 1: Monthly aggregates
monthly_totals AS (
    SELECT 
        month,
        SUM(sales_amount) AS monthly_total,
        COUNT(*) AS transactions
    FROM test
    GROUP BY month
),
-- Step 2: Add previous month for comparison
with_previous AS (
    SELECT 
        month,
        monthly_total,
        transactions,
        LAG(monthly_total, 1) OVER (ORDER BY month) AS prev_month_total
    FROM monthly_totals
),
-- Step 3: Calculate growth
with_growth AS (
    SELECT 
        month,
        monthly_total,
        prev_month_total,
        CASE 
            WHEN prev_month_total IS NOT NULL 
            THEN monthly_total - prev_month_total
            ELSE 0
        END AS growth_amount,
        CASE 
            WHEN prev_month_total IS NOT NULL 
            THEN ROUND((monthly_total - prev_month_total) * 100.0 / prev_month_total, 1)
            ELSE 0
        END AS growth_pct
    FROM with_previous
)
-- Final output
SELECT 
    month,
    monthly_total,
    growth_amount,
    growth_pct
FROM with_growth
ORDER BY month;
GO

-- ============================================================================
-- Recipe 3: Product Performance by Region
-- Find which products sell best in each region
-- ============================================================================
WITH 
-- Step 1: Sales by product and region
product_regional AS (
    SELECT 
        product,
        region,
        SUM(sales_amount) AS regional_sales,
        COUNT(*) AS transactions
    FROM test
    GROUP BY product, region
),
-- Step 2: Rank products within each region
ranked_products AS (
    SELECT 
        product,
        region,
        regional_sales,
        transactions,
        ROW_NUMBER() OVER (PARTITION BY region ORDER BY regional_sales DESC) AS product_rank
    FROM product_regional
)
-- Step 3: Top product per region
SELECT 
    region,
    product AS top_product,
    regional_sales,
    transactions
FROM ranked_products
WHERE product_rank = 1
ORDER BY regional_sales DESC;
GO

-- ============================================================================
-- Recipe 4: Sales Performance Tiers
-- Categorize salespeople into performance tiers
-- ============================================================================
WITH 
-- Step 1: Individual performance metrics
individual_stats AS (
    SELECT 
        salesperson,
        SUM(sales_amount) AS total_sales,
        COUNT(*) AS num_sales,
        AVG(sales_amount) AS avg_sale,
        MAX(sales_amount) AS best_sale,
        MIN(sales_amount) AS worst_sale
    FROM test
    GROUP BY salesperson
),
-- Step 2: Add rankings
with_rankings AS (
    SELECT 
        salesperson,
        total_sales,
        num_sales,
        ROUND(avg_sale, 2) AS avg_sale_amount,
        best_sale,
        worst_sale,
        ROW_NUMBER() OVER (ORDER BY total_sales DESC) AS sales_rank
    FROM individual_stats
),
-- Step 3: Assign performance tiers based on rank
with_tiers AS (
    SELECT 
        salesperson,
        total_sales,
        avg_sale_amount,
        best_sale,
        sales_rank,
        CASE 
            WHEN sales_rank <= 2 THEN 'Top Performer'
            WHEN sales_rank <= 4 THEN 'Strong Performer'
            WHEN sales_rank <= 6 THEN 'Average Performer'
            ELSE 'Needs Improvement'
        END AS performance_tier
    FROM with_rankings
)
-- Final output
SELECT 
    sales_rank,
    salesperson,
    total_sales,
    avg_sale_amount,
    performance_tier
FROM with_tiers
ORDER BY sales_rank;
GO

-- ============================================================================
-- Recipe 5: Regional Comparison
-- Compare regions by multiple metrics
-- ============================================================================
WITH 
-- Step 1: Regional aggregates
regional_stats AS (
    SELECT 
        region,
        SUM(sales_amount) AS total_sales,
        COUNT(*) AS transactions,
        AVG(sales_amount) AS avg_transaction,
        MAX(sales_amount) AS highest_sale,
        MIN(sales_amount) AS lowest_sale
    FROM test
    GROUP BY region
),
-- Step 2: Add rankings and ratios
regional_analysis AS (
    SELECT 
        region,
        total_sales,
        transactions,
        ROUND(avg_transaction, 2) AS avg_transaction_size,
        highest_sale,
        lowest_sale,
        ROW_NUMBER() OVER (ORDER BY total_sales DESC) AS sales_rank,
        ROW_NUMBER() OVER (ORDER BY avg_transaction DESC) AS avg_size_rank
    FROM regional_stats
)
-- Final comparison
SELECT 
    sales_rank,
    region,
    total_sales,
    transactions,
    avg_transaction_size,
    CASE 
        WHEN sales_rank = 1 THEN 'Best'
        WHEN sales_rank = 2 THEN '2nd'
        ELSE 'Other'
    END AS performance
FROM regional_analysis
ORDER BY sales_rank;
GO

-- ============================================================================
-- Recipe 6: Performance vs Personal Best
-- Compare each person's sales to their own best
-- ============================================================================
WITH 
-- Step 1: Calculate baseline for each salesperson
personal_stats AS (
    SELECT 
        salesperson,
        AVG(sales_amount) AS avg_sale,
        MAX(sales_amount) AS max_sale,
        MIN(sales_amount) AS min_sale,
        COUNT(*) AS num_sales
    FROM test
    GROUP BY salesperson
),
-- Step 2: Add performance indicators
with_indicators AS (
    SELECT 
        salesperson,
        num_sales,
        ROUND(avg_sale, 2) AS avg_sale_amount,
        max_sale,
        min_sale,
        ROUND((max_sale - min_sale), 2) AS sale_range,
        ROUND(max_sale / avg_sale, 2) AS best_vs_avg_ratio
    FROM personal_stats
),
-- Step 3: Rank by consistency (lower range = more consistent)
ranked AS (
    SELECT 
        salesperson,
        num_sales,
        avg_sale_amount,
        max_sale,
        min_sale,
        sale_range,
        best_vs_avg_ratio,
        ROW_NUMBER() OVER (ORDER BY sale_range ASC) AS consistency_rank
    FROM with_indicators
)
-- Show consistency analysis
SELECT 
    consistency_rank,
    salesperson,
    avg_sale_amount,
    sale_range,
    best_vs_avg_ratio,
    CASE 
        WHEN consistency_rank <= 2 THEN 'Very Consistent'
        WHEN consistency_rank <= 4 THEN 'Fairly Consistent'
        ELSE 'Variable'
    END AS consistency_level
FROM ranked
ORDER BY consistency_rank;
GO

-- ============================================================================
-- Key Patterns Successfully Demonstrated:
-- 
-- 1. TOP N PER GROUP: ROW_NUMBER() with PARTITION BY, then filter
-- 2. TIME SERIES: LAG() to compare with previous periods
-- 3. MULTI-STEP AGGREGATION: Aggregate → Rank → Filter
-- 4. PERFORMANCE TIERS: Use CASE with rankings to categorize
-- 5. BASELINE COMPARISON: Calculate averages, then compare
-- 6. CTE CHAINING: Each CTE builds on previous ones
--
-- All examples use only supported features:
-- ✅ ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...)
-- ✅ LAG() and LEAD() window functions
-- ✅ Standard aggregates: SUM, COUNT, AVG, MAX, MIN
-- ✅ CASE WHEN expressions
-- ✅ CTE chaining (each CTE can reference previous ones)
-- ============================================================================