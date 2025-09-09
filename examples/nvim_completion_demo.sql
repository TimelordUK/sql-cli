-- SQL CLI Neovim Plugin - Autocompletion Demo
-- #!data: ../data/sales_data.csv

-- ============================================
-- AUTOCOMPLETION FEATURES
-- ============================================
-- In Neovim with the SQL CLI plugin loaded:
-- 1. Press <C-Space> (Ctrl+Space) in INSERT mode to trigger completion
-- 2. Completion provides:
--    - Column names from your data file
--    - SQL functions with descriptions
--    - SQL keywords
--
-- Try these examples by typing partial text and pressing <C-Space>:

-- Example 1: Column name completion
-- Type: SELECT reg<C-Space>
-- Result: Suggests 'region' column
SELECT region FROM sales_data;

-- Example 2: Function completion
-- Type: SELECT COU<C-Space>
-- Result: Suggests COUNT( function
SELECT COUNT(*) FROM sales_data;

-- Example 3: Multiple column completion
-- Type: SELECT <C-Space>
-- Result: Shows all available columns
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    product
FROM sales_data;

-- Example 4: WHERE clause completion
-- Type: WHERE sal<C-Space>
-- Result: Suggests 'salesperson' and 'sales_amount'
SELECT * FROM sales_data 
WHERE salesperson = 'Alice';

-- Example 5: Aggregate functions
-- Type: SELECT SU<C-Space>
-- Result: Suggests SUM( function
SELECT 
    region,
    SUM(sales_amount) as total_sales,
    AVG(sales_amount) as avg_sales,
    COUNT(*) as num_sales
FROM sales_data
GROUP BY region;

-- Example 6: String functions
-- Type: SELECT UPP<C-Space>
-- Result: Suggests UPPER( function
SELECT 
    UPPER(region) as region_upper,
    LOWER(salesperson) as person_lower,
    LENGTH(product) as product_length
FROM sales_data;

-- Example 7: SQL keywords
-- Type: SELECT * FROM sales_data ORD<C-Space>
-- Result: Suggests ORDER keyword
SELECT * FROM sales_data 
ORDER BY sales_amount DESC;

-- ============================================
-- SMART DETECTION WITH <leader>sk
-- ============================================
-- Place cursor on any word and press <leader>sk:
-- - If it's a column name, shows column info
-- - If it's a function name, shows function help
-- - Provides context-aware documentation

-- ============================================
-- SCHEMA INSPECTION WITH <leader>sh
-- ============================================
-- Press <leader>sh to see the full table schema
-- Shows all columns with their inferred types
-- Based on actual data analysis