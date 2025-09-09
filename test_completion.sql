-- #!data: data/sales_data.csv
-- Test SQL file for autocompletion

-- Try typing and then pressing <C-x><C-o> to trigger completion:
-- SELECT reg<C-x><C-o>  (should suggest 'region')
-- SELECT COUNT(<C-x><C-o>  (should suggest columns)
-- SELECT * FROM sales_data WHERE <C-x><C-o>  (should suggest columns)

SELECT 
    region,
    salesperson,
    SUM(sales_amount) as total_sales
FROM sales_data
WHERE month = '2024-01'
GROUP BY region, salesperson
ORDER BY total_sales DESC;