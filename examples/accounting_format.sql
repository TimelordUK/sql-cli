-- #! ../data/international_sales.csv
-- Accounting Format Examples
-- Demonstrates negative numbers displayed in parentheses
-- data: data/international_sales.csv

-- === RENDER_NUMBER with Accounting Format ===

-- Basic accounting format for positive and negative numbers
SELECT
    RENDER_NUMBER(1234.56, 'accounting') as positive,
    RENDER_NUMBER(-1234.56, 'accounting') as negative,
    RENDER_NUMBER(0, 'accounting') as zero;
GO

WITH
    negatives AS (
        SELECT -1 * (amount * quantity) AS neg_amount
        FROM international_sales
    )
SELECT neg_amount, 
  RENDER_NUMBER(neg_amount) AS standard, 
  RENDER_NUMBER(neg_amount, 'accounting') AS accounting
  from negatives;
GO

-- === FORMAT_CURRENCY with Accounting Format ===

-- Currency with accounting format (negatives in parentheses)
SELECT
    FORMAT_CURRENCY(1234.56, 'USD', 'accounting') as positive_usd,
    FORMAT_CURRENCY(-1234.56, 'USD', 'accounting') as negative_usd,
    FORMAT_CURRENCY(-999.99, 'GBP', 'accounting') as negative_gbp,
    FORMAT_CURRENCY(-2500, 'EUR', 'accounting') as negative_eur;
GO

-- Accounting format with currency codes
SELECT
    FORMAT_CURRENCY(1500.00, 'USD', 'accounting_code') as positive_code,
    FORMAT_CURRENCY(-1500.00, 'USD', 'accounting_code') as negative_code,
    FORMAT_CURRENCY(-999.99, 'EUR', 'accounting_code') as euro_code;
GO

WITH
    factors AS (
        SELECT CASE WHEN country.Contains('n') THEN 0.7 ELSE 1.2  END AS factor, *
        FROM international_sales
    )
SELECT *
FROM factors;
GO

-- Create a profit/loss calculation with accounting format
WITH factors AS (
        SELECT CASE WHEN country.Contains('n') THEN 0.7 ELSE 1.2 END AS factor, *
        FROM international_sales
),
profit_loss AS (
    SELECT
        country,
        SUM(amount * quantity) as revenue,
        -- Simulate costs as 70% of revenue
        SUM(amount * quantity) * factor as costs,
        -- Calculate profit (can be negative)
        SUM(amount * quantity) - (SUM(amount * quantity) * factor) as profit
    FROM factors
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
GO

