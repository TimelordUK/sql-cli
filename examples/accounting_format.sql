-- Accounting Format Examples
-- Demonstrates negative numbers displayed in parentheses
-- data: data/international_sales.csv

-- === RENDER_NUMBER with Accounting Format ===

-- Basic accounting format for positive and negative numbers
SELECT
    RENDER_NUMBER(1234.56, 'accounting') as positive,
    RENDER_NUMBER(-1234.56, 'accounting') as negative,
    RENDER_NUMBER(0, 'accounting') as zero;

-- Compare standard vs accounting format for negatives
SELECT
    amount,
    RENDER_NUMBER(amount) as standard,
    RENDER_NUMBER(amount, 'accounting') as accounting
FROM (
    SELECT 1234.56 as amount
    UNION ALL
    SELECT -1234.56
    UNION ALL
    SELECT -99999.99
    UNION ALL
    SELECT 0
) AS test_values;

-- === FORMAT_CURRENCY with Accounting Format ===

-- Currency with accounting format (negatives in parentheses)
SELECT
    FORMAT_CURRENCY(1234.56, 'USD', 'accounting') as positive_usd,
    FORMAT_CURRENCY(-1234.56, 'USD', 'accounting') as negative_usd,
    FORMAT_CURRENCY(-999.99, 'GBP', 'accounting') as negative_gbp,
    FORMAT_CURRENCY(-2500, 'EUR', 'accounting') as negative_eur;

-- Accounting format with currency codes
SELECT
    FORMAT_CURRENCY(1500.00, 'USD', 'accounting_code') as positive_code,
    FORMAT_CURRENCY(-1500.00, 'USD', 'accounting_code') as negative_code,
    FORMAT_CURRENCY(-999.99, 'EUR', 'accounting_code') as euro_code;

-- === Real-World Example: Profit/Loss Report ===

-- Create a profit/loss calculation with accounting format
WITH profit_loss AS (
    SELECT
        country,
        SUM(amount * quantity) as revenue,
        -- Simulate costs as 70% of revenue
        SUM(amount * quantity) * 0.7 as costs,
        -- Calculate profit (can be negative)
        SUM(amount * quantity) - (SUM(amount * quantity) * 0.7) as profit
    FROM international_sales
    GROUP BY country
)
SELECT
    country,
    FORMAT_CURRENCY(revenue, 'USD', 'accounting') as revenue,
    FORMAT_CURRENCY(costs, 'USD', 'accounting') as costs,
    FORMAT_CURRENCY(profit, 'USD', 'accounting') as profit,
    CASE
        WHEN profit < 0 THEN 'Loss'
        WHEN profit > 0 THEN 'Profit'
        ELSE 'Break-even'
    END as status
FROM profit_loss
ORDER BY profit DESC
LIMIT 10;

-- === Monthly Performance with Accounting Format ===

-- Simulate monthly performance with some negative months
WITH monthly_data AS (
    SELECT 'January' as month, 15000.50 as amount
    UNION ALL SELECT 'February', -3200.75
    UNION ALL SELECT 'March', 8500.00
    UNION ALL SELECT 'April', -1250.25
    UNION ALL SELECT 'May', 22000.00
    UNION ALL SELECT 'June', -500.50
)
SELECT
    month,
    RENDER_NUMBER(amount, 'accounting') as amount_accounting,
    FORMAT_CURRENCY(amount, 'USD', 'accounting') as amount_usd,
    CASE
        WHEN amount < 0 THEN '📉'
        ELSE '📈'
    END as trend
FROM monthly_data
ORDER BY
    CASE month
        WHEN 'January' THEN 1
        WHEN 'February' THEN 2
        WHEN 'March' THEN 3
        WHEN 'April' THEN 4
        WHEN 'May' THEN 5
        WHEN 'June' THEN 6
    END;