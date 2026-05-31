-- ============================================================
--  DATE_FORMAT — MySQL-style format specifiers
--  -------------------------------------------------------------
--  DATE_FORMAT translates MySQL-only specifiers to chrono's
--  strftime dialect before formatting, so a MySQL user's format
--  strings behave as expected:
--
--    %i  minute (00-59)        %M  full month name (January)
--    %W  full weekday (Sunday) %s  second (00-59)
--    %f  microseconds
--
--  Shared specifiers (%Y %m %d %H %h %p %a %b %r %T %%) pass
--  through unchanged. No data file needed — fixed literals keep
--  this deterministic.
--
--  Reference instant: 2026-05-31 14:05:09  (a Sunday).
-- ============================================================


-- ---- 1. The MySQL-only specifiers that used to misbehave -------
-- Before 2026-05-31 these delegated straight to strftime, so
-- %W gave the week number (21) and %M gave the minute (05).
SELECT
    DATE_FORMAT('2026-05-31 14:05:09', '%W')      AS weekday_name,
    DATE_FORMAT('2026-05-31 14:05:09', '%M')      AS month_name,
    DATE_FORMAT('2026-05-31 14:05:09', '%i')      AS minute,
    DATE_FORMAT('2026-05-31 14:05:09', '%s')      AS second;
GO


-- ---- 2. Composite MySQL format strings ------------------------
SELECT
    DATE_FORMAT('2026-05-31 14:05:09', '%H:%i:%s')         AS time_24h,
    DATE_FORMAT('2026-05-31 14:05:09', '%r')               AS time_12h,
    DATE_FORMAT('2026-05-31 14:05:09', '%W, %M %d, %Y')    AS long_date,
    DATE_FORMAT('2026-05-31 14:05:09', '%Y-%m-%d')         AS iso_date;
GO


-- ---- 3. Shared specifiers and literal %% are untouched --------
SELECT
    DATE_FORMAT('2026-05-31 14:05:09', '%a %b %d')   AS abbreviated,
    DATE_FORMAT('2026-05-31 14:05:09', '100%% done') AS literal_percent;
GO
