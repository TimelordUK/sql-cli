-- #! ../data/fix_timestamps.csv
-- Flexible Date Parsing Examples
-- Demonstrates PARSE_DATETIME, PARSE_DATETIME_UTC, and DATETIME constructor functions

-- ============================================================================
-- PARSE_DATETIME - Parse dates with custom format strings
-- ============================================================================

-- European date format (DD/MM/YYYY)
SELECT PARSE_DATETIME('15/01/2024', '%d/%m/%Y') as european_date;
GO

-- American date format (MM/DD/YYYY)
SELECT PARSE_DATETIME('01/15/2024', '%m/%d/%Y') as american_date;
GO

-- Date with time
SELECT PARSE_DATETIME('15/01/2024 14:30:45', '%d/%m/%Y %H:%M:%S') as datetime_parsed;
GO

-- Text month names (short and long)
SELECT
    PARSE_DATETIME('Jan 15, 2024', '%b %d, %Y') as short_month,
    PARSE_DATETIME('January 15, 2024', '%B %d, %Y') as long_month;
GO

-- ISO 8601 format
SELECT PARSE_DATETIME('2024-01-15T14:30:45', '%Y-%m-%dT%H:%M:%S') as iso_format;
GO

-- FIX Protocol timestamp format (YYYYMMDD-HH:MM:SS.sss)
SELECT PARSE_DATETIME('20240115-14:30:45.567', '%Y%m%d-%H:%M:%S%.3f') as fix_format;
GO

-- ============================================================================
-- PARSE_DATETIME_UTC - Parse as UTC (auto-detect or custom format)
-- ============================================================================

-- Auto-detect format (1 argument)
SELECT PARSE_DATETIME_UTC('2024-01-15 14:30:00') as auto_detect;
GO

-- Auto-detect FIX format
SELECT PARSE_DATETIME_UTC('20240115-14:30:45.567') as fix_auto_detect;
GO

-- Custom format (2 arguments)
SELECT PARSE_DATETIME_UTC('15/01/2024 14:30', '%d/%m/%Y %H:%M') as custom_format;
GO

-- Parse FIX timestamps from CSV data
SELECT
    order_id,
    transaction_time as original,
    PARSE_DATETIME_UTC(transaction_time) as parsed_utc
FROM fix_timestamps
LIMIT 5;
GO

-- ============================================================================
-- DATETIME - Construct datetime from components
-- ============================================================================

-- Date only (year, month, day)
SELECT DATETIME(2024, 1, 15) as date_only;
GO

-- Date with full time (year, month, day, hour, minute, second)
SELECT DATETIME(2024, 1, 15, 14, 30, 45) as full_datetime;
GO

-- Date with partial time components
SELECT
    DATETIME(2024, 12, 25) as date,
    DATETIME(2024, 12, 25, 18) as with_hour,
    DATETIME(2024, 12, 25, 18, 45) as with_minutes,
    DATETIME(2024, 12, 25, 18, 45, 30) as with_seconds;
GO

-- All datetime values are interpreted as UTC
-- Note: is_utc parameter support (7th argument) is defined but parser currently limits to 6 args
SELECT DATETIME(2024, 1, 15, 14, 30, 0) as utc_datetime;
GO

-- Leap year handling
SELECT
    DATETIME(2024, 2, 29) as leap_day_2024,
    ISLEAPYEAR(2024) as is_leap_year;
GO

-- ============================================================================
-- Combined Usage - Mixing new and existing date functions
-- ============================================================================

-- Calculate date differences using constructed dates
SELECT
    DATETIME(2024, 1, 1) as start_date,
    PARSE_DATETIME('31/12/2024', '%d/%m/%Y') as end_date,
    DATEDIFF('day', DATETIME(2024, 1, 1), PARSE_DATETIME('31/12/2024', '%d/%m/%Y')) as days_between;
GO

-- Calculate millisecond offsets from FIX timestamps
SELECT
    order_id,
    transaction_time,
    DATEDIFF('millisecond',
        DATETIME(2025, 9, 25, 14, 52, 15),
        PARSE_DATETIME_UTC(transaction_time)
    ) as ms_offset
FROM fix_timestamps
LIMIT 5;
GO

-- Extract components from parsed dates
SELECT
    PARSE_DATETIME('15/01/2024', '%d/%m/%Y') as parsed_date,
    YEAR(PARSE_DATETIME('15/01/2024', '%d/%m/%Y')) as year,
    MONTH(PARSE_DATETIME('15/01/2024', '%d/%m/%Y')) as month,
    DAY(PARSE_DATETIME('15/01/2024', '%d/%m/%Y')) as day,
    DAYNAME(PARSE_DATETIME('15/01/2024', '%d/%m/%Y')) as day_name,
    QUARTER(PARSE_DATETIME('15/01/2024', '%d/%m/%Y')) as quarter;
GO

-- Add intervals to parsed/constructed dates
SELECT
    DATETIME(2024, 1, 15) as base_date,
    DATEADD('day', 30, DATETIME(2024, 1, 15)) as plus_30_days,
    DATEADD('month', 3, DATETIME(2024, 1, 15)) as plus_3_months,
    DATEADD('year', 1, DATETIME(2024, 1, 15)) as plus_1_year;
GO

-- Convert to Unix timestamps
SELECT
    PARSE_DATETIME('15/01/2024 14:30:00', '%d/%m/%Y %H:%M:%S') as datetime,
    UNIX_TIMESTAMP(PARSE_DATETIME('15/01/2024 14:30:00', '%d/%m/%Y %H:%M:%S')) as unix_ts;
GO

-- ============================================================================
-- Format String Reference
-- ============================================================================
-- Common format specifiers (chrono strftime):
--   %Y - 4-digit year (2024)
--   %m - Month (01-12)
--   %d - Day of month (01-31)
--   %H - Hour 24h (00-23)
--   %M - Minute (00-59)
--   %S - Second (00-59)
--   %.3f - Milliseconds (.000-.999)
--   %b - Short month name (Jan, Feb, ...)
--   %B - Full month name (January, February, ...)
--   %T - Time in HH:MM:SS format
--
-- Full reference: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
-- ============================================================================
