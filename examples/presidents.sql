-- US Presidents: joining across five separate CSV files
--
-- The five data/president_*.csv files each hold one fact about each president,
-- keyed loosely by name. Nothing here is a "database" — READ_CSV stages each
-- file as a CTE and the engine joins them in memory.
--
-- These files are also column-aligned by hand:
--
--     "Index", "Name", "Day", "Month", "Year"
--      1, "George Washington",  22, 2, 1732
--
-- RFC 4180 only recognises a quote that is the *first* byte of a field, so that
-- padding used to hide the quoting entirely and the second column came back
-- named ` "Name"`. The loader now strips padding that hugs a quote before
-- parsing, which is why `SELECT Name` works below.
--
-- Run from the repo root:  ./target/release/sql-cli -f examples/presidents.sql

-- === Three files, one row per president ===
-- Birthdays, heights and birth states all keyed on Name.
WITH
    birthdays AS (SELECT Name, Year FROM read_csv('data/president_birthdays.csv')),
    heights   AS (SELECT Name, "Height (inches)" AS Height FROM read_csv('data/president_heights.csv')),
    states    AS (SELECT Name, "Birth State" AS State FROM read_csv('data/president_states.csv'))
SELECT
    b.Name,
    b.Year,
    h.Height,
    s.State
FROM birthdays b
JOIN heights h ON b.Name = h.Name
JOIN states  s ON b.Name = s.Name
ORDER BY h.Height DESC, b.Name
LIMIT 10;
GO

-- === Joining on a clean key ===
-- birthdays and timelines both carry Index, so this one matches all 45 rows.
-- Note Year arrives as a number even though the file writes it as ` 1732` —
-- padding around an unquoted field is ignored when the type is inferred.
WITH
    birthdays AS (SELECT Index AS Idx, Name, Year FROM read_csv('data/president_birthdays.csv')),
    timelines AS (SELECT Index AS Idx, TermBegin, TermEnd FROM read_csv('data/president_timelines.csv'))
SELECT
    b.Idx,
    b.Name,
    b.Year,
    t.TermBegin,
    t.TermEnd
FROM birthdays b
JOIN timelines t ON b.Idx = t.Idx
WHERE b.Year > 1900
ORDER BY b.Idx;
GO

-- === Where the joins leak ===
-- Name is a lossy join key here: the files disagree on spelling, so the inner
-- joins above only match 24 of 45 presidents. A LEFT JOIN plus IS NULL names
-- the casualties — "Martin Van Buren" vs "Martin van Buren", "George H. W.
-- Bush" vs "George Bush I", and so on.
WITH
    birthdays AS (SELECT Name FROM read_csv('data/president_birthdays.csv')),
    heights   AS (SELECT Name AS HeightName FROM read_csv('data/president_heights.csv'))
SELECT b.Name AS MissingFromHeights
FROM birthdays b
LEFT JOIN heights h ON b.Name = h.HeightName
WHERE h.HeightName IS NULL
ORDER BY b.Name;
GO

-- === Aggregating across the join ===
-- Which birth states produced the most presidents, and how tall were they?
WITH
    birthdays AS (SELECT Name FROM read_csv('data/president_birthdays.csv')),
    heights   AS (SELECT Name, "Height (inches)" AS Height FROM read_csv('data/president_heights.csv')),
    states    AS (SELECT Name, "Birth State" AS State FROM read_csv('data/president_states.csv'))
SELECT
    s.State,
    COUNT(*) AS Presidents,
    AVG(h.Height) AS AvgHeight,
    MAX(h.Height) AS TallestHeight
FROM birthdays b
JOIN heights h ON b.Name = h.Name
JOIN states  s ON b.Name = s.Name
GROUP BY s.State
ORDER BY Presidents DESC, s.State;
GO

-- === Four files, with a dirty column ===
-- president_ratings.csv scores greatness 1-5, but unrated presidents get the
-- literal text NA, which makes the column mixed-type. The file writes it
-- unquoted and padded (`"Donald Trump",          NA`) — padding next to an
-- unquoted value is left alone, since there is no way to tell alignment from
-- data there, so Trim() before comparing.
WITH
    birthdays AS (SELECT Name, Year FROM read_csv('data/president_birthdays.csv')),
    heights   AS (SELECT Name, "Height (inches)" AS Height FROM read_csv('data/president_heights.csv')),
    states    AS (SELECT Name, "Birth State" AS State FROM read_csv('data/president_states.csv')),
    ratings   AS (SELECT Name, Greatness FROM read_csv('data/president_ratings.csv'))
SELECT
    b.Name,
    b.Year,
    s.State,
    h.Height,
    r.Greatness
FROM birthdays b
JOIN heights h ON b.Name = h.Name
JOIN states  s ON b.Name = s.Name
JOIN ratings r ON b.Name = r.Name
WHERE r.Greatness.Trim() <> 'NA'
ORDER BY r.Greatness DESC, b.Name;
GO

-- === Building a real date out of three columns ===
-- The file stores the birthday split across Day/Month/Year. DATETIME() takes
-- expressions, so those columns compose into an actual date rather than a
-- string, and the rest of the date functions work on the result.
WITH
    birthdays AS (SELECT Name, Year, Month, Day FROM read_csv('data/president_birthdays.csv'))
SELECT
    Name,
    DATETIME(Year, Month, Day) AS BirthDate,
    DAYNAME(DATETIME(Year, Month, Day)) AS BornOn,
    QUARTER(DATETIME(Year, Month, Day)) AS Qtr,
    ISLEAPYEAR(Year) AS LeapYear
FROM birthdays
WHERE Year > 1900
ORDER BY DATETIME(Year, Month, Day);
GO

-- === Dates compare as dates ===
-- Not as strings: the comparison below is against a constructed date literal.
WITH
    birthdays AS (SELECT Name, Year, Month, Day FROM read_csv('data/president_birthdays.csv')),
    heights   AS (SELECT Name, "Height (inches)" AS Height FROM read_csv('data/president_heights.csv'))
SELECT
    b.Name,
    DATETIME(b.Year, b.Month, b.Day) AS BirthDate,
    h.Height
FROM birthdays b
JOIN heights h ON b.Name = h.Name
WHERE DATETIME(b.Year, b.Month, b.Day) > DATETIME(1850, 1, 1)
ORDER BY BirthDate;
GO

-- === Does height predict greatness? ===
-- Average height per greatness score, over the presidents who carry both.
WITH
    heights AS (SELECT Name, "Height (inches)" AS Height FROM read_csv('data/president_heights.csv')),
    ratings AS (SELECT Name, Greatness FROM read_csv('data/president_ratings.csv'))
SELECT
    r.Greatness,
    COUNT(*) AS Presidents,
    AVG(h.Height) AS AvgHeight
FROM heights h
JOIN ratings r ON h.Name = r.Name
WHERE r.Greatness.Trim() <> 'NA'
GROUP BY r.Greatness
ORDER BY r.Greatness DESC;
GO
