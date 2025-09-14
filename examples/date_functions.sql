-- Date Function Examples
-- Demonstrates the new date utility functions

-- Example 1: Day of week analysis
WITH dates AS (
    SELECT DATEADD('day', value, '2024-01-01') as date
    FROM RANGE(0, 7)
)
SELECT
    FORMAT_DATE(date, '%Y-%m-%d') as date,
    DAYOFWEEK(date) as day_num,
    DAYNAME(date, 'full') as day_full,
    DAYNAME(date, 'short') as day_short
FROM dates
ORDER BY date;
GO

-- Example 2: Month and quarter analysis
WITH months AS (
    SELECT DATEADD('month', value, '2024-01-01') as date
    FROM RANGE(0, 12)
)
SELECT
    FORMAT_DATE(date, '%Y-%m-%d') as date,
    MONTHNAME(date, 'full') as month_full,
    MONTHNAME(date, 'short') as month_short,
    QUARTER(date) as quarter,
    WEEKOFYEAR(date) as week_num
FROM months
ORDER BY date;
GO

-- Example 3: Leap year checker
WITH years AS (
    SELECT 2020 + value as year
    FROM RANGE(0, 10)
)
SELECT
    year,
    ISLEAPYEAR(year) as is_leap,
    CASE 
        WHEN ISLEAPYEAR(year) THEN 'Leap Year (366 days)'
        ELSE 'Regular Year (365 days)'
    END as year_type
FROM years
ORDER BY year;
GO

-- Example 4: Creating a date dimension table
WITH date_range AS (
    SELECT DATEADD('day', value, '2024-01-01') as date
    FROM RANGE(0, 30)
)
SELECT
    FORMAT_DATE(date, '%Y-%m-%d') as date,
    DAYOFWEEK(date) as day_of_week_num,
    DAYNAME(date, 'short') as day_name,
    CASE
        WHEN DAYOFWEEK(date) = 0 THEN 'Weekend'
        WHEN DAYOFWEEK(date) = 6 THEN 'Weekend'
        ELSE 'Weekday'
    END as day_type,
    WEEKOFYEAR(date) as week_of_year,
    MONTHNAME(date, 'short') as month_name,
    QUARTER(date) as quarter
FROM date_range
ORDER BY date
LIMIT 10;
GO

SELECT
    UNIX_TIMESTAMP(NOW()) AS unix_timestamp;
GO

SELECT
    DAYOFWEEK(NOW()) AS day_of_week,
    MONTH(NOW()) AS month,
    YEAR(NOW()) AS year,
    WEEKOFYEAR(NOW()) AS week_of_year,
    TODAY() as today,
    ISLEAPYEAR(NOW()) as is_leap_year,
    DAYNAME(NOW()) as day_name,
    MONTHNAME(NOW()) as month_name,
    QUARTER(NOW()) as quarter
FROM dual;
GO



