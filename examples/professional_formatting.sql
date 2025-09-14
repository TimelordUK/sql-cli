-- Professional Table Formatting Examples
-- Demonstrates how to create publication-quality formatted output

-- ==============================================
-- EXAMPLE 1: Financial Report with Right-Aligned Numbers
-- ==============================================
WITH financial_data AS (
    SELECT
        'Q1 2024' as period,
        1234567.89 as revenue,
        987654.32 as expenses,
        1234567.89 - 987654.32 as profit
    UNION ALL
    SELECT 'Q2 2024', 1456789.12, 1123456.78, 1456789.12 - 1123456.78
    UNION ALL
    SELECT 'Q3 2024', 1678901.34, 1567890.12, 1678901.34 - 1567890.12
    UNION ALL
    SELECT 'Q4 2024', 1890123.56, 1234567.89, 1890123.56 - 1234567.89
)
SELECT
    period,
    LPAD(FORMAT_CURRENCY(revenue, 'USD'), 15, ' ') as revenue,
    LPAD(FORMAT_CURRENCY(expenses, 'USD'), 15, ' ') as expenses,
    LPAD(FORMAT_CURRENCY(profit, 'USD', 'accounting'), 15, ' ') as profit,
    LPAD(RENDER_NUMBER(profit / revenue * 100, 'standard', 1) || '%', 8, ' ') as margin
FROM financial_data
ORDER BY period;
GO

-- ==============================================
-- EXAMPLE 2: ID Numbers with Zero Padding
-- ==============================================
WITH employee_data AS (
    SELECT value as emp_id,
           'EMP-' || LPAD(value, 6, '0') as emp_code,
           50000 + (value * 2500) as salary
    FROM RANGE(1, 10)
)
SELECT
    LPAD(emp_id, 4, ' ') as id,
    emp_code,
    LPAD(FORMAT_CURRENCY(salary, 'USD'), 12, ' ') as salary,
    LPAD(RENDER_NUMBER(salary * 0.0765, 'standard', 2), 10, ' ') as fica_tax
FROM employee_data;
GO

-- ==============================================
-- EXAMPLE 3: Percentage Formatting with Alignment
-- ==============================================
WITH performance_data AS (
    SELECT
        'Product A' as product,
        0.9234 as success_rate,
        0.0523 as error_rate,
        0.0243 as retry_rate
    UNION ALL
    SELECT 'Product B', 0.8567, 0.1233, 0.0200
    UNION ALL
    SELECT 'Product C', 0.9801, 0.0099, 0.0100
)
SELECT
    RPAD(product, 12, ' ') as product,
    LPAD(RENDER_NUMBER(success_rate * 100, 'standard', 2) || '%', 8, ' ') as success,
    LPAD(RENDER_NUMBER(error_rate * 100, 'standard', 2) || '%', 8, ' ') as errors,
    LPAD(RENDER_NUMBER(retry_rate * 100, 'standard', 2) || '%', 8, ' ') as retries
FROM performance_data;
GO

-- ==============================================
-- EXAMPLE 4: Compact Large Numbers (K, M, B notation)
-- ==============================================
WITH sales_data AS (
    SELECT
        'Store 001' as store_id,
        1234 as daily_sales,
        1234 * 30 as monthly_sales,
        1234 * 365 as annual_sales
    UNION ALL
    SELECT 'Store 002', 45678, 45678 * 30, 45678 * 365
    UNION ALL
    SELECT 'Store 003', 234567, 234567 * 30, 234567 * 365
)
SELECT
    store_id,
    LPAD(RENDER_NUMBER(daily_sales, 'compact', 1), 8, ' ') as daily,
    LPAD(RENDER_NUMBER(monthly_sales, 'compact', 1), 8, ' ') as monthly,
    LPAD(RENDER_NUMBER(annual_sales, 'compact', 1), 8, ' ') as annual
FROM sales_data;
GO

-- ==============================================
-- EXAMPLE 5: Mixed Format Professional Report
-- ==============================================
WITH report_data AS (
    SELECT
        LPAD(value, 3, '0') as item_code,
        'SKU-' || LPAD(value * 1000 + value, 8, '0') as sku,
        value * 123.45 as cost,
        value * 123.45 * 1.4 as price,
        value * 10 + MOD(value, 3) * 5 as quantity
    FROM RANGE(1, 8)
)
SELECT
    item_code,
    sku,
    LPAD(FORMAT_CURRENCY(cost, 'USD'), 10, ' ') as unit_cost,
    LPAD(FORMAT_CURRENCY(price, 'USD'), 10, ' ') as unit_price,
    LPAD(quantity, 5, ' ') as qty,
    LPAD(FORMAT_CURRENCY(price * quantity, 'USD'), 12, ' ') as total,
    LPAD(RENDER_NUMBER((price - cost) / cost * 100, 'standard', 1) || '%', 7, ' ') as markup
FROM report_data;
GO

-- ==============================================
-- EXAMPLE 6: Accounting Format with Negatives
-- ==============================================
WITH ledger AS (
    SELECT
        'INV-' || LPAD(value, 6, '0') as transaction_id,
        CASE
            WHEN MOD(value, 3) = 0 THEN 'Credit'
            ELSE 'Debit'
        END as type,
        CASE
            WHEN MOD(value, 3) = 0 THEN value * -1234.56
            ELSE value * 987.65
        END as amount
    FROM RANGE(1, 10)
)
SELECT
    transaction_id,
    RPAD(type, 8, ' ') as type,
    LPAD(RENDER_NUMBER(amount, 'accounting'), 15, ' ') as amount,
    LPAD(FORMAT_CURRENCY(amount, 'USD', 'accounting'), 18, ' ') as amount_usd
FROM ledger
ORDER BY transaction_id;
GO

-- ==============================================
-- EXAMPLE 7: Date and Number Formatting Combined
-- ==============================================
WITH time_series AS (
    SELECT
        DATEADD('day', value, '2024-01-01') as date,
        value * 1000 + MOD(value * 17, 100) as visitors,
        (value * 1000 + MOD(value * 17, 100)) * 0.023 as conversion_rate
    FROM RANGE(0, 7)
)
SELECT
    FORMAT_DATE(date, '%Y-%m-%d') as date,
    FORMAT_DATE(date, '%a') as day,
    LPAD(RENDER_NUMBER(visitors, 'standard'), 8, ' ') as visitors,
    LPAD(RENDER_NUMBER(conversion_rate, 'standard', 2), 8, ' ') as conversions,
    LPAD(RENDER_NUMBER(conversion_rate / visitors * 100, 'standard', 3) || '%', 8, ' ') as rate
FROM time_series
ORDER BY date;
GO