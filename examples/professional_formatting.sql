-- Professional Table Formatting Examples
-- Demonstrates how to create publication-quality formatted output

-- ==============================================
-- EXAMPLE 1: Financial Report with Right-Aligned Numbers
-- ==============================================
-- Note: Using RANGE to generate sample data since UNION ALL is not yet supported
SELECT
    'Q' || value || ' 2024' as period,
    LPAD(FORMAT_CURRENCY(1234567.89 + (value - 1) * 222222.22, 'USD'), 15, ' ') as revenue,
    LPAD(FORMAT_CURRENCY(987654.32 + (value - 1) * 135802.46, 'USD'), 15, ' ') as expenses,
    LPAD(FORMAT_CURRENCY((1234567.89 + (value - 1) * 222222.22) - (987654.32 + (value - 1) * 135802.46), 'USD', 'accounting'), 15, ' ') as profit,
    LPAD(RENDER_NUMBER(((1234567.89 + (value - 1) * 222222.22) - (987654.32 + (value - 1) * 135802.46)) / (1234567.89 + (value - 1) * 222222.22) * 100, 'standard', 1) || '%', 8, ' ') as margin
FROM RANGE(1, 4)
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
-- Note: Using RANGE with CHR for dynamic product names
SELECT
    RPAD('Product ' || CHR(64 + value), 12, ' ') as product,
    LPAD(RENDER_NUMBER(
        CASE
            WHEN value = 1 THEN 92.34
            WHEN value = 2 THEN 85.67
            WHEN value = 3 THEN 98.01
        END, 'standard', 2) || '%', 8, ' ') as success,
    LPAD(RENDER_NUMBER(
        CASE
            WHEN value = 1 THEN 5.23
            WHEN value = 2 THEN 12.33
            WHEN value = 3 THEN 0.99
        END, 'standard', 2) || '%', 8, ' ') as errors,
    LPAD(RENDER_NUMBER(
        CASE
            WHEN value = 1 THEN 2.43
            WHEN value = 2 THEN 2.00
            WHEN value = 3 THEN 1.00
        END, 'standard', 2) || '%', 8, ' ') as retries
FROM RANGE(1, 3);
GO

-- ==============================================
-- EXAMPLE 4: Compact Large Numbers (K, M, B notation)
-- ==============================================
-- Note: Using RANGE with calculated values
SELECT
    'Store ' || LPAD(value, 3, '0') as store_id,
    LPAD(RENDER_NUMBER(1234 * POWER(value, 2), 'compact', 1), 8, ' ') as daily,
    LPAD(RENDER_NUMBER(1234 * POWER(value, 2) * 30, 'compact', 1), 8, ' ') as monthly,
    LPAD(RENDER_NUMBER(1234 * POWER(value, 2) * 365, 'compact', 1), 8, ' ') as annual
FROM RANGE(1, 5);
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
        (value + 1) * 1000 + MOD(value * 17, 100) as visitors,
        ((value + 1) * 1000 + MOD(value * 17, 100)) * 0.023 as conversions
    FROM RANGE(0, 7)
)
SELECT
    FORMAT_DATE(date, '%Y-%m-%d') as date,
    FORMAT_DATE(date, '%a') as day,
    LPAD(RENDER_NUMBER(visitors, 'standard'), 8, ' ') as visitors,
    LPAD(RENDER_NUMBER(conversions, 'standard', 2), 8, ' ') as conversions,
    LPAD(RENDER_NUMBER(conversions / visitors * 100, 'standard', 3) || '%', 8, ' ') as rate
FROM time_series
ORDER BY date;
GO