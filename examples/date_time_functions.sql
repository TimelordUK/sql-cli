-- Date and time manipulation functions
-- sql-cli supports comprehensive date/time operations

-- Current date and time
SELECT 
    NOW() as current_timestamp,
    TODAY() as current_date,
    CURRENT_DATE() as also_current_date,
    CURRENT_TIME() as current_time_only;

-- Date arithmetic
SELECT 
    DATEADD('day', 7, '2024-01-01') as week_later,
    DATEADD('month', 3, '2024-01-01') as three_months_later,
    DATEADD('year', 1, '2024-01-01') as next_year,
    DATEADD('hour', 24, NOW()) as tomorrow_same_time;

-- Date differences
SELECT 
    DATEDIFF('day', '2024-01-01', '2024-12-31') as days_in_2024,
    DATEDIFF('month', '2024-01-01', '2024-12-31') as months_diff,
    DATEDIFF('year', '2000-01-01', NOW()) as years_since_2000,
    DATEDIFF('hour', '2024-01-01 00:00:00', '2024-01-02 12:00:00') as hours_diff;

-- Date extraction
SELECT 
    EXTRACT(YEAR FROM '2024-03-15') as year_part,
    EXTRACT(MONTH FROM '2024-03-15') as month_part,
    EXTRACT(DAY FROM '2024-03-15') as day_part,
    EXTRACT(DOW FROM '2024-03-15') as day_of_week,
    EXTRACT(WEEK FROM '2024-03-15') as week_number;

-- Date formatting (supports both US and European formats)
SELECT 
    DATE_FORMAT('2024-03-15', '%Y-%m-%d') as iso_format,
    DATE_FORMAT('2024-03-15', '%d/%m/%Y') as european_format,
    DATE_FORMAT('2024-03-15', '%m/%d/%Y') as us_format,
    DATE_FORMAT('2024-03-15', '%B %d, %Y') as long_format;

-- Date parsing from strings
SELECT 
    DATE_PARSE('15/03/2024', '%d/%m/%Y') as parsed_european,
    DATE_PARSE('03/15/2024', '%m/%d/%Y') as parsed_us,
    DATE_PARSE('March 15, 2024', '%B %d, %Y') as parsed_long;

-- Practical example: Date analysis on transactions
SELECT 
    transaction_date,
    EXTRACT(YEAR FROM transaction_date) as year,
    EXTRACT(MONTH FROM transaction_date) as month,
    EXTRACT(QUARTER FROM transaction_date) as quarter,
    DATEDIFF('day', transaction_date, NOW()) as days_ago,
    CASE 
        WHEN DATEDIFF('day', transaction_date, NOW()) <= 7 THEN 'Last Week'
        WHEN DATEDIFF('day', transaction_date, NOW()) <= 30 THEN 'Last Month'
        WHEN DATEDIFF('day', transaction_date, NOW()) <= 90 THEN 'Last Quarter'
        ELSE 'Older'
    END as recency,
    DATEADD('year', 1, transaction_date) as anniversary_date
FROM transactions
WHERE transaction_date >= DATEADD('year', -1, TODAY())
ORDER BY transaction_date DESC;