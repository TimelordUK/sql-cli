-- Date and time manipulation functions
-- sql-cli supports date/time operations with DATEADD and DATEDIFF

-- Current date and time
SELECT 
    NOW() as current_timestamp,
    TODAY() as current_date;
GO

-- Date arithmetic
SELECT 
    DATEADD('day', 7, '2024-01-01') as week_later,
    DATEADD('month', 3, '2024-01-01') as three_months_later,
    DATEADD('year', 1, '2024-01-01') as next_year,
    DATEADD('hour', 24, NOW()) as tomorrow_same_time;
GO

-- Date differences
SELECT 
    DATEDIFF('day', '2024-01-01', '2024-12-31') as days_in_2024,
    DATEDIFF('month', '2024-01-01', '2024-12-31') as months_diff,
    DATEDIFF('year', '2000-01-01', NOW()) as years_since_2000,
    DATEDIFF('hour', '2024-01-01 00:00:00', '2024-01-02 12:00:00') as hours_diff;
GO

-- TODO: These functions would be useful but are not yet implemented:
-- EXTRACT() - Extract date parts (year, month, day, etc.)
-- DATE_FORMAT() - Format dates in various styles
-- DATE_PARSE() - Parse dates from strings
-- CURRENT_DATE() - Alias for TODAY()
-- CURRENT_TIME() - Get time without date

-- Practical example: Calculate ages and durations
SELECT 
    DATEDIFF('day', '2024-01-01', TODAY()) as days_since_year_start,
    DATEDIFF('year', '1990-01-01', TODAY()) as age_if_born_1990,
    DATEADD('year', 1, TODAY()) as one_year_from_now;
GO